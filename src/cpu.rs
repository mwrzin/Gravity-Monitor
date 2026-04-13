use std::fs;
use std::io;
use crate::permissions::SystemCapabilities;

/// Estrutura responsável pela leitura da energia consumida pela CPU.
/// Exige a interface nativa 'powercap' do kernel Linux (RAPL - Running Average Power Limit).
#[derive(Clone)]
pub struct CpuSensor {
    // Caminho no sistema de arquivos virtual que aponta para o registro de microjoules do processador 0.
    pub path: String,
}

impl CpuSensor {
    /// Construtor: Inicializa a estrutura injetando o caminho fixo padrão do hardware Intel/AMD.
    #[cfg(target_os = "linux")]
    pub fn new() -> Self {
        Self {
            // O caminho /sys/class/powercap/intel-rapl:0/energy_uj armazena o contador bruto de consumo do pacote da placa-mãe.
            path: String::from("/sys/class/powercap/intel-rapl:0/energy_uj"),
        }
    }

    #[cfg(target_os = "windows")]
    pub fn new() -> Self {
        Self {
            path: String::new(),
        }
    }

    /// Executa a leitura crua do arquivo do Kernel e converte a energia para a métrica de Joules padrão.
    #[cfg(target_os = "linux")]
    pub fn read_joules(&self) -> Result<f64, String> {
        // Tenta abrir o arquivo físico exposto em memória pelo Linux
        match fs::read_to_string(&self.path) {
            Ok(content) => {
                let microjoules: f64 = content.trim().parse().unwrap_or(0.0);
                Ok(microjoules / 1_000_000.0)
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::PermissionDenied {
                    Err(SystemCapabilities::get_elevation_message())
                } else {
                    Err(format!("Erro ao acessar o sensor de energia: {}", e))
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    pub fn read_joules(&self) -> Result<f64, String> {
        // Stub para futura implementação MSR via driver no Windows
        Err(SystemCapabilities::get_elevation_message())
    }
}
