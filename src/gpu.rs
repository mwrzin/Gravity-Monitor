use nvml_wrapper::Nvml;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Estrutura massiva para telemetria em Unidades de Processamento Gráfico (NVIDIA).
/// Mantém as sessões da API C da NVML presas e protegidas em blocos Thread-Safe (Arc/Mutex).
#[derive(Clone)]
pub struct GpuSensor {
    pub manual_override_power: Option<f64>,
    pub has_gpu: bool,
    nvml: Option<Arc<Mutex<Nvml>>>,               // Instância subjacente da NVML. O Option lida com a falta desta silenciosamente.
    last_time: Arc<Mutex<Instant>>,               // Auxiliar de controle de tempo delta para Integração de Potência Preditiva
    accumulated_joules: Arc<Mutex<f64>>,          // Acumulador persistente pra chips voltados aos Consumidores padrão
}

impl GpuSensor {
    /// Tenta buscar silenciosamente os binários da driver NVIDIA local e estabelecer a pipeline.
    pub fn new(manual_override_power: Option<f64>) -> Self {
        // Nvml::init varre as dependências nativas (libnvidia-ml.so no C-Level).
        match Nvml::init() {
            Ok(nvml) => {
                // Confirma operatividade testando um PING primitivo no barramento PCI da Placa 0.
                match nvml.device_by_index(0) {
                    Ok(_) => Self {
                        manual_override_power,
                        has_gpu: true,
                        nvml: Some(Arc::new(Mutex::new(nvml))),
                        last_time: Arc::new(Mutex::new(Instant::now())),
                        accumulated_joules: Arc::new(Mutex::new(0.0)),
                    },
                    Err(_) => Self {
                        // Se o bind passou mas falhou a leitura por PCI (ex: erro no container/docker de driver), cai forasteiro.
                        manual_override_power,
                        has_gpu: false,
                        nvml: None,
                        last_time: Arc::new(Mutex::new(Instant::now())),
                        accumulated_joules: Arc::new(Mutex::new(0.0)),
                    }
                }
            }
            Err(_) => Self {
                // Caso seja hardware Intel, macOS, ou drivers da Nvidia ausentes, ele instacia uma Placa de Vídeo virtual morta sem causar bugs de Python.
                manual_override_power,
                has_gpu: false,
                nvml: None,
                last_time: Arc::new(Mutex::new(Instant::now())),
                accumulated_joules: Arc::new(Mutex::new(0.0)),
            }
        }
    }

    /// Lê a potência, lidando ativamente com os chips Datacenter (mJ brutos) e chips Domésticos (mW instantâneos transitorios).
    pub fn read_joules(&self) -> Result<f64, String> {
        // Se houver substituição manual, simula um workload agressivo na GPU
        if let Some(override_watts) = self.manual_override_power {
            let mut last = self.last_time.lock().unwrap();
            let mut acc = self.accumulated_joules.lock().unwrap();
            let now = Instant::now();
            let delta_s = now.duration_since(*last).as_secs_f64();
            *last = now;
            *acc += override_watts * delta_s;
            return Ok(*acc);
        }

        if !self.has_gpu {
            // Silencia a telemetria caso seja rodado num hardware desprovido de NVIDIA. O sistema continuará focado em CPU/RAM perfeitamente.
            return Ok(0.0);
        }

        if let Some(nvml_mutex) = &self.nvml {
            // Tenta obter acesso restritivo à ponte do NVML via lock, blindando conflitos multi-threading.
            if let Ok(nvml) = nvml_mutex.lock() {
                // Obtém um handle referenciando a GPU principal
                if let Ok(device) = nvml.device_by_index(0) {
                    // Cenario 1: Datacenter/Workstations Modernas (Tesla, A100, Hopper) possuem registrador interno físico global da energia desde o Bootup.
                    if let Ok(millijoules) = device.total_energy_consumption() {
                        return Ok(millijoules as f64 / 1000.0);
                    }
                    
                    // Cenario 2: GPU Consumer (Trilhas cortadas, sem acesso a total_energy_consumption).
                    // Temos apenas o power_usage(), que emite os "Mili-Watts desse exato e efêmero momento".
                    if let Ok(milliwatts) = device.power_usage() {
                        let mut last = self.last_time.lock().unwrap();
                        let mut acc = self.accumulated_joules.lock().unwrap();
                        
                        // Extração temporal (Delta de Tempo) desde a última medição em Segundos (Precisão flutuante limpa).
                        let now = Instant::now();
                        let delta_s = now.duration_since(*last).as_secs_f64();
                        *last = now; // Atualiza a fita de tempo
                        
                        // Executa integração matemática do Consumo. Energia(Joules) = Potência(Watts) * Tempo(s).
                        let watts = milliwatts as f64 / 1000.0;
                        *acc += watts * delta_s; 
                        
                        // Agora este método consegue imitar um contador natural que sempre cresce simulando perfeição arquitetural da API!
                        return Ok(*acc);
                    }
                }
            }
        }
        Ok(0.0)
    }
}
