// src/lib.rs
use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use serde::Serialize;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

// Submódulos locais que compõem a arquitetura de sensores deste motor FFI.
pub mod cpu;
pub mod ram;
pub mod gpu;
pub mod carbon;

/// Representação serializável da base de dados final produzida no momento da Interrupção pelo usuário.
#[derive(Serialize)]
pub struct ExportData {
    pub timestamp: u64,       // Marcação unix-time garantindo consistência cronológica
    pub watts: f64,           // Demanda energética absoluta dividida pelao tempo ativo
    pub total_joules: f64,    // Unidade universal capturada dos transistores físicos
    pub co2_emissions: f64,   // Conversão termodinâmica processada em massa de carbono
}

/// A classe Raiz exportada para o Python. É decorada com [pyclass] o que diz ao PyO3 
/// para mapear seus tipos de memória como se fossem ponteiros nativos C do CPython!
#[pyclass]
pub struct GravityTracker {
    // Sensores individuais encarregados da infraestrutura de baixo nível.
    pub sensor: cpu::CpuSensor,
    pub ram_sensor: ram::RamSensor,
    pub gpu_sensor: gpu::GpuSensor,
    
    // Auxiliar base de tempo limpa do Rust pra cronometragem de segundos brutos (não corrompe com falhas de CPU clock).
    pub start_time: Option<std::time::Instant>,
    
    // O Coração da Threading Segura: Arc(Ponteiro Atômico) -> Mutex(Cadeado de Memória) -> Float
    // Evita o desastre chamado "Race Condition" se o Tracker Python tentar puxar um Print no mesmíssimo momento 
    // em que a Background Thread do Rust estiver somando o Delta do Módulo RAPL nela.
    pub total_joules_accumulated: Arc<Mutex<f64>>,
    
    // Flag Volátil thread-safe para informar silenciosamente (sem locks de IO) à thread operante se ela deve se matar.
    pub is_running: Arc<AtomicBool>,
    
    // Vetor que age como Memória de Longo Prazo do "Python Context Manager". Armazena os blocos com sufixos _start ou _end!
    pub checkpoints: Vec<(String, f64)>,
}

#[pymethods]
impl GravityTracker {
    /// O método Construtor invisível mapeado para `gravity_monitor.GravityTracker()` no frontend.
    #[new]
    pub fn new() -> Self {
        Self {
            sensor: cpu::CpuSensor::new(),
            ram_sensor: ram::RamSensor::new(),
            gpu_sensor: gpu::GpuSensor::new(),
            start_time: None,
            total_joules_accumulated: Arc::new(Mutex::new(0.0)),
            is_running: Arc::new(AtomicBool::new(false)),
            checkpoints: Vec::new(),
        }
    }

    /// Devolve ao ambiente externo de forma assíncrona se a varredura primária conseguiu dar 'Bind' em GPUs locais.
    pub fn has_gpu(&self) -> PyResult<bool> {
        Ok(self.gpu_sensor.has_gpu)
    }

    /// O Famoso Inicializador. Destrava e delega toda a computação à Threading Limpa do Sistema Operacional puro.
    pub fn start(&mut self) -> PyResult<()> {
        if self.is_running.load(Ordering::SeqCst) {
            return Err(PyRuntimeError::new_err("O monitoramento já está em andamento."));
        }

        // Zera contadores na fita do tempo oficial (Milissegundo Zero)
        self.start_time = Some(std::time::Instant::now());
        self.is_running.store(true, Ordering::SeqCst);
        *self.total_joules_accumulated.lock().unwrap() = 0.0;

        // Clone das referências seguras `Arc` permitindo que a Thread capture-as pra sempre até o .stop() sem quebrar o ownership do Rust
        let cpu_sensor_clone = self.sensor.clone();
        let ram_sensor_clone = self.ram_sensor.clone();
        let gpu_sensor_clone = self.gpu_sensor.clone();
        let is_running_clone = self.is_running.clone();
        let acc_clone = self.total_joules_accumulated.clone();

        // Extrai a Medição T0 Pura fora da thread (Limpando o risco absurdo do "Falso Pico Delta" no milissegundo inicial)
        let mut last_cpu = cpu_sensor_clone.read_joules().unwrap_or(0.0);
        let mut last_ram = ram_sensor_clone.read_joules().unwrap_or(0.0);
        let mut last_gpu = gpu_sensor_clone.read_joules().unwrap_or(0.0);

        // Dispara a Main Background Thread (completamente imune a lag e sleep de Python)
        thread::spawn(move || {
            while is_running_clone.load(Ordering::SeqCst) {
                // Ciclo Perfeito de 100ms exigido à risca para garantir precisão atômica dos Deltas
                thread::sleep(Duration::from_millis(100));

                let current_cpu = cpu_sensor_clone.read_joules().unwrap_or(last_cpu);
                let current_ram = ram_sensor_clone.read_joules().unwrap_or(last_ram);
                let current_gpu = gpu_sensor_clone.read_joules().unwrap_or(last_gpu);

                // Deduzir Carga Integral Gasta! (Quantos Joules o PC sugou fisicamente nesses míseros 100ms de vida?)
                let mut delta_cpu = current_cpu - last_cpu;
                let mut delta_ram = current_ram - last_ram;
                let mut delta_gpu = current_gpu - last_gpu;
                
                // Tratar "Reset" de Kernel / Spikes Sujos que acontecem se o PC entrar em suspensão ou a bateria falhar
                if delta_cpu < 0.0 { delta_cpu = 0.0; }
                if delta_ram < 0.0 { delta_ram = 0.0; }
                if delta_gpu < 0.0 { delta_gpu = 0.0; }

                last_cpu = current_cpu;
                last_ram = current_ram;
                last_gpu = current_gpu;

                // Salva atômicamente (Lock global no mutex com soma absoluta e rapida desockagem)
                let mut acc = acc_clone.lock().unwrap();
                *acc += delta_cpu + delta_ram + delta_gpu;
            }
        });

        println!("Monitoramento iniciado em background (100ms amostragem)...");
        Ok(())
    }

    /// O Termo de Segurança de Finalização.
    pub fn stop(&mut self) -> PyResult<()> {
        // Envia o pulso de morte pra flag atômica para o laço while fechar sem violência.
        self.is_running.store(false, Ordering::SeqCst);
        
        // Graceful shutdown: adiciona um microdelay (150ms > 100ms) forçando a thread a conseguir depositar sua ultima gota do Mutex!
        thread::sleep(Duration::from_millis(150));
        println!("Monitoramento em background encerrado.");
        Ok(())
    }

    /// API de requisição externa em tempo real para os Watts (Deltas puros divididos por Timeline Secs).
    pub fn get_power(&self) -> PyResult<f64> {
        match self.start_time {
            Some(time) => {
                let delta_seconds = time.elapsed().as_secs_f64();
                let acc = *self.total_joules_accumulated.lock().unwrap();
                if delta_seconds > 0.0 {
                    Ok(acc / delta_seconds)
                } else {
                    Ok(0.0)
                }
            }
            None => Err(PyRuntimeError::new_err("O monitoramento ainda não foi iniciado. Chame o método start() primeiro.")),
        }
    }

    /// API Criativa e Conscientizacional para Regionalização Local das métricas climáticas.
    pub fn get_local_impact(&self) -> PyResult<String> {
        let acc = *self.total_joules_accumulated.lock().unwrap();
        Ok(carbon::get_salinas_equivalence(acc))
    }

    /// Mapeia o conversor físico de Poluição Equivalente em cima das Normativas de Intensidade Regionais do Brasil/Outros Países
    pub fn get_emissions(&self, intensity: f64) -> PyResult<f64> {
        match self.start_time {
            Some(_) => {
                let acc = *self.total_joules_accumulated.lock().unwrap();
                Ok(carbon::calculate_emissions(acc, intensity))
            },
            None => Err(PyRuntimeError::new_err("O monitoramento ainda não foi iniciado. Chame o método start() primeiro.")),
        }
    }

    /// Injeta Ativamente o Marcador do Bloco "Scope" ao Array global de Checkpoints para posterior análise final de Perfilagem (Profiling)
    pub fn mark_step(&mut self, step_name: &str) {
        let current_joules = *self.total_joules_accumulated.lock().unwrap();
        self.checkpoints.push((step_name.to_string(), current_joules));
    }

    /// Retorna um Dicionário PyO3 Multidimensional Analisando Perfil Integral.
    pub fn get_report(&self, intensity: f64) -> PyResult<std::collections::HashMap<String, std::collections::HashMap<String, f64>>> {
        // Aloca os Buffers Hash Maps do Rust que o PyO3 vai traduzir em `dict()` mágicos.
        let mut report = std::collections::HashMap::new();
        let mut starts = std::collections::HashMap::new();
        let total_consumed = *self.total_joules_accumulated.lock().unwrap();

        // O Parser Cruza arrays de [str, f64]: Busca matches cruzados de `_start` e `_end`.
        for (name, joules) in &self.checkpoints {
            if name.ends_with("_start") {
                let base_name = name.trim_end_matches("_start");
                starts.insert(base_name.to_string(), *joules);
            } else if name.ends_with("_end") {
                let base_name = name.trim_end_matches("_end");
                // Custo Absoluto Exato: Subtração da Linha do Tempo Start em relação a Linha de Chegada End.
                if let Some(start_joules) = starts.get(base_name) {
                    let cost = joules - start_joules;
                    let percentage = if total_consumed > 0.0 { (cost / total_consumed) * 100.0 } else { 0.0 };
                    let co2 = carbon::calculate_emissions(cost, intensity);
                    
                    // Empacota toda info numa subpasta do dicionário por nome
                    let mut data = std::collections::HashMap::new();
                    data.insert("joules".to_string(), cost);
                    data.insert("percentage".to_string(), percentage);
                    data.insert("co2_equivalent".to_string(), co2);
                    
                    report.insert(base_name.to_string(), data);
                }
            }
        }
        Ok(report)
    }

    /// Instanciador Nativo mágico pra FFI `tracker.scope('passo')`. Sequestra o próprio PyNode referencial (`slf`).
    pub fn scope(slf: Py<Self>, step_name: String) -> PyResult<TrackerScope> {
        Ok(TrackerScope {
            tracker: slf.clone(),
            step_name,
        })
    }

    /// Persiste silenciosamente no disco local o arquivo JSON em blocos assícronos para leitura analítica.
    pub fn export_json(&self, filename: &str, intensity: f64) -> PyResult<()> {
        match self.start_time {
            Some(time) => {
                let delta_seconds = time.elapsed().as_secs_f64();
                let acc = *self.total_joules_accumulated.lock().unwrap();
                
                let watts = if delta_seconds > 0.0 { acc / delta_seconds } else { 0.0 };
                let co2 = carbon::calculate_emissions(acc, intensity);
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let data = ExportData {
                    timestamp,
                    watts,
                    total_joules: acc,
                    co2_emissions: co2,
                };

                let file = File::create(filename).map_err(|e| PyRuntimeError::new_err(format!("Erro ao criar arquivo: {}", e)))?;
                // O Rust embute Serializadores de JSON nativos mais velozes e mais seguros que o modulo `json` do root Python!
                serde_json::to_writer_pretty(file, &data).map_err(|e| PyRuntimeError::new_err(format!("Erro ao serializar JSON: {}", e)))?;
                Ok(())
            }
            None => Err(PyRuntimeError::new_err("O monitoramento ainda não foi iniciado. Chame o método start() primeiro.")),
        }
    }
}

/// A Ponte ContextManager para o Python
#[pyclass]
pub struct TrackerScope {
    tracker: Py<GravityTracker>,
    step_name: String,
}

#[pymethods]
impl TrackerScope {
    /// O Gatilho de Ativação do Bloco (Chamado por C baixo-nivel toda vez que uma sintaxe de IA `with tracker.scope:` abre chave)
    fn __enter__(&self, py: Python<'_>) -> PyResult<()> {
        let mut tracker = self.tracker.borrow_mut(py);
        tracker.mark_step(&format!("{}_start", self.step_name));
        Ok(())
    }

    /// O Destrutor Natural. Tranca o Timestamp Final mesmo que houvesse uma interrupção inesperada pelo Python!
    #[allow(unused_variables)]
    fn __exit__(
        &self,
        py: Python<'_>,
        exc_type: Bound<'_, PyAny>,
        exc_val: Bound<'_, PyAny>,
        traceback: Bound<'_, PyAny>,
    ) -> PyResult<bool> {
        let mut tracker = self.tracker.borrow_mut(py);
        tracker.mark_step(&format!("{}_end", self.step_name));
        Ok(false) // Retornar False garante não mascarar/silenciar Crashes críticos pra Aplicação do Dev acima.
    }
}

// O Bootloader Principal FFI do C Extensions. Exporta oficialmente tudo dentro de `gravity_monitor.so`.
#[pymodule]
fn gravity_monitor(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<GravityTracker>()?;
    m.add_class::<TrackerScope>()?;
    Ok(())
}
