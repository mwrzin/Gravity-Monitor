use std::fs;
use std::io;
use sysinfo::System;
use std::time::SystemTime;
use crate::permissions::SystemCapabilities;

/// Estrutura responsável por extrair ou simular matematicamente o consumo energético das Memórias (DRAM).
#[derive(Clone)]
pub struct RamSensor {
    pub path: String,               // Localização do RAPL para DRAM, se suportado fisicamente.
    pub has_rapl: bool,             // Flag estática que indica se a placa mãe vazou o sensor via ACPI.
    pub total_memory_bytes: u64,    // Buffer imutável guardando o total da RAM em bytes via sysinfo.
}

impl RamSensor {
    /// Inicializa a telemetria da memória verificando o suporte no kernel antes da pre-alocação.
    #[cfg(target_os = "linux")]
    pub fn new() -> Self {
        // Nas plataformas Intel modernas, a DRAM (Memória) é rastreada sob o canal sub-RAPL :0:0.
        let path = String::from("/sys/class/powercap/intel-rapl:0:0/energy_uj");
        
        // Verifica silenciosamente se este computador suporta e expõe tal diretório.
        let has_rapl = fs::metadata(&path).is_ok();
        
        // Carrega o instanciador do Sistema Operacional puro cross-platform pela biblioteca sysinfo.
        let mut sys = System::new();
        sys.refresh_memory(); // Atualiza ponteiros de status de RAM
        
        Self {
            path,
            has_rapl,
            total_memory_bytes: sys.total_memory(), // Extrai o físico fixado pra usar em simulações.
        }
    }

    #[cfg(target_os = "windows")]
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_memory();
        Self {
            path: String::new(),
            has_rapl: false,
            total_memory_bytes: sys.total_memory(),
        }
    }

    /// Processa a leitura, priorizando sempre a interface elétrica real antes da simulação.
    #[cfg(target_os = "linux")]
    pub fn read_joules(&self) -> Result<f64, String> {
        if self.has_rapl {
            // Caminho Rápido: O hardware provê o consumo exato da DRAM acumulado em MicroJoules.
            match fs::read_to_string(&self.path) {
                Ok(content) => {
                    let microjoules: f64 = content.trim().parse().unwrap_or(0.0);
                    Ok(microjoules / 1_000_000.0) // Redução direta do offset pra Joule nativo
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::PermissionDenied {
                        Err(SystemCapabilities::get_elevation_message())
                    } else {
                        // Fallback reativo: permissão correta mas lida falhou na via, usa estimativa.
                        self.estimate_joules()
                    }
                }
            }
        } else {
            // Caminho de Degradação Graciosa: Falta de apoio da interface nativa RAPL
            self.estimate_joules()
        }
    }

    #[cfg(target_os = "windows")]
    pub fn read_joules(&self) -> Result<f64, String> {
        // No Windows, por enquanto só é liberado estimativa se MSR ainda falhar:
        // Porém, se for exigido ler MSR no futuro, vamos retornar o SystemCapabilities
        // No momento a versão fallback já ajuda o Monitor enquanto não temos WinRing0.
        self.estimate_joules()
    }

    
    /// Função de Fallback que aplica Física para calcular gasto baseado em volume fixo.
    fn estimate_joules(&self) -> Result<f64, String> {
        // Converte a quantidade de bytes da RAM pra Gigabytes
        let gb = self.total_memory_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        
        // Atribuição de tolerância base de hardware: Assume em média ~0.375 Watts passivos gastos POR Gigabyte DDR4!
        let estimated_power_watts = gb * 0.375;
        
        // Recolhe o timestamp exato que o sistema ligou e cruza com Agora para saber "há quantos segundos a RAM tá sugando energia".
        let now_secs = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
            
        // Potência (W) x Tempo (s) = Energia exata acumulada (Joules).
        Ok(now_secs * estimated_power_watts)
    }
}
