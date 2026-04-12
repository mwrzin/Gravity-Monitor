use std::fs;
use std::io;

/// Estrutura responsável pela leitura da energia consumida pela CPU.
/// Exige a interface nativa 'powercap' do kernel Linux (RAPL - Running Average Power Limit).
#[derive(Clone)]
pub struct CpuSensor {
    // Caminho no sistema de arquivos virtual que aponta para o registro de microjoules do processador 0.
    pub path: String,
}

impl CpuSensor {
    /// Construtor: Inicializa a estrutura injetando o caminho fixo padrão do hardware Intel/AMD.
    pub fn new() -> Self {
        Self {
            // O caminho /sys/class/powercap/intel-rapl:0/energy_uj armazena o contador bruto de consumo do pacote da placa-mãe.
            path: String::from("/sys/class/powercap/intel-rapl:0/energy_uj"),
        }
    }

    /// Executa a leitura crua do arquivo do Kernel e converte a energia para a métrica de Joules padrão.
    pub fn read_joules(&self) -> Result<f64, String> {
        // Tenta abrir o arquivo físico exposto em memória pelo Linux
        match fs::read_to_string(&self.path) {
            Ok(content) => {
                // Converte a String capturada para um número de ponto flutuante (fallback para 0.0 em caso de lixo no registrador)
                let microjoules: f64 = content.trim().parse().unwrap_or(0.0);
                // Retorna dividindo por 1 Milhão, normalizando de Microjoules(uJ) para Joules absolutos.
                Ok(microjoules / 1_000_000.0)
            }
            Err(e) => {
                // Intercepta falhas de leitura detalhadas: O uso mais comum é bater no ErrorKind::PermissionDenied.
                // Acesso a chaves RAPL exige rodar a simulação inteira em modo 'sudo/root'.
                if e.kind() == io::ErrorKind::PermissionDenied {
                    Err("Permissão negada ao ler o sensor de energia da CPU. Tente executar o programa com 'sudo'.".to_string())
                } else {
                    Err(format!("Erro ao acessar o sensor de energia: {}", e))
                }
            }
        }
    }
}
