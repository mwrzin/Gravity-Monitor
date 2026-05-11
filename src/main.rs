// =====================================================================
// ARQUIVO: src/main.rs
// OBJETIVO: É o arquivo de entrada (entry point) se você for rodar o 
// projeto puramente em Rust (sem usar o Python). 
// =====================================================================

use std::thread; // Biblioteca padrão do Rust para criar e controlar Threads (processos em paralelo)
use std::time::Duration; // Biblioteca para lidar com a passagem do tempo (segundos, milissegundos)
use gravity_monitor::GravityTracker; // Importa a classe principal do monitor que criamos no `lib.rs`

// Define uma constante com a intensidade de carbono do Brasil (em gCO2/kWh). 
// Constantes em Rust ficam na memória global e são super rápidas de acessar.
const BRAZIL_INTENSITY: f64 = 82.0;

// A função `main` é a primeira coisa que o computador executa ao rodar um programa compilado em Rust.
fn main() {
    // Instancia o nosso rastreador. 
    // Usamos `mut` (mutável) porque o estado do rastreador vai mudar (os joules vão aumentar).
    let mut tracker = GravityTracker::new(None, None, None);

    // Imprime uma mensagem de boas vindas na tela (macro println!)
    println!("--- Monitor Antigravity: Consumo em Watts e CO2 ---");

    // Tenta iniciar a Thread de monitoramento em background.
    // O `if let Err(e)` é o jeito elegante do Rust de dizer: "Se der erro, capture o erro na variável 'e'".
    if let Err(e) = tracker.start() {
        // eprint! imprime no canal de Erros do terminal (stderr), que geralmente fica em vermelho
        eprintln!("Erro ao iniciar o monitoramento: {}", e);
        return; // Aborta a execução do programa porque o sensor não iniciou
    }

    // Cria um loop que vai rodar exatamente 10 vezes (simulando um treinamento de IA que dura 10 segundos)
    for _ in 0..10 {
        // Pausa esta thread (a principal) por 1 segundo. 
        // Enquanto isso, a thread secreta do tracker continua rodando 10 vezes por segundo (100ms) lá no fundo!
        thread::sleep(Duration::from_secs(1));
        
        // Pede os Watts atuais do rastreador
        // O `match` é o Switch Case "bombado" do Rust que obriga você a lidar com o Sucesso (Ok) e a Falha (Err)
        match tracker.get_power() {
            Ok(watts) => println!("Potência média: {:.2} W", watts), // Se deu certo, imprime os Watts formatado com 2 casas decimais
            Err(e) => eprintln!("Erro ao ler a potência: {}", e), // Se deu erro (ex: start() não foi chamado), avisa a tela
        }

        // Pede as Emissões de CO2 baseadas na intensidade do Brasil
        match tracker.get_emissions(BRAZIL_INTENSITY) {
            Ok(co2) => println!("Emissão acumulada de CO2: {:.6} gCO2", co2),
            Err(e) => eprintln!("Erro ao ler a emissão: {}", e),
        }
    }
}
