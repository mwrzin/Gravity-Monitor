use std::thread;
use std::time::Duration;
use gravity_monitor::GravityTracker;

const BRAZIL_INTENSITY: f64 = 82.0;

fn main() {
    let mut tracker = GravityTracker::new();

    println!("--- Monitor Antigravity: Consumo em Watts e CO2 ---");

    if let Err(e) = tracker.start() {
        eprintln!("Erro ao iniciar o monitoramento: {}", e);
        return;
    }

    for _ in 0..10 {
        thread::sleep(Duration::from_secs(1));
        
        match tracker.get_power() {
            Ok(watts) => println!("Potência média: {:.2} W", watts),
            Err(e) => eprintln!("Erro ao ler a potência: {}", e),
        }

        match tracker.get_emissions(BRAZIL_INTENSITY) {
            Ok(co2) => println!("Emissão acumulada de CO2: {:.6} gCO2", co2),
            Err(e) => eprintln!("Erro ao ler a emissão: {}", e),
        }
    }
}
