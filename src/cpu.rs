use std::fs;
use std::io;
use crate::permissions::SystemCapabilities;

#[cfg(target_os = "windows")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "windows")]
use sysinfo::System;
#[cfg(target_os = "windows")]
use winapi::um::libloaderapi::LoadLibraryA;
#[cfg(target_os = "windows")]
use std::ffi::CString;

#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "linux")]
use sysinfo::System;

/// Estrutura responsável pela leitura da energia consumida pela CPU.
/// Exige a interface nativa 'powercap' do kernel Linux ou bibliotecas do Windows.
#[derive(Clone)]
pub struct CpuSensor {
    pub manual_override_tdp: Option<f64>,
    
    #[cfg(target_os = "linux")]
    pub path: String,

    pub fallback_system: Arc<Mutex<System>>,
    pub estimated_joules: Arc<Mutex<f64>>,
    pub last_update: Arc<Mutex<std::time::Instant>>,
}

impl CpuSensor {
    /// Construtor: Inicializa a estrutura e pré-carrega ferramentas do SO alvo.
    #[cfg(target_os = "linux")]
    pub fn new(manual_override_tdp: Option<f64>) -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_usage();
        Self {
            manual_override_tdp,
            path: String::from("/sys/class/powercap/intel-rapl:0/energy_uj"),
            fallback_system: Arc::new(Mutex::new(sys)),
            estimated_joules: Arc::new(Mutex::new(0.0)),
            last_update: Arc::new(Mutex::new(std::time::Instant::now())),
        }
    }

    #[cfg(target_os = "windows")]
    pub fn new(manual_override_tdp: Option<f64>) -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_usage(); // Refresh inicial para computar o delta de porcentagem da CPU.
        Self {
            manual_override_tdp,
            fallback_system: Arc::new(Mutex::new(sys)),
            estimated_joules: Arc::new(Mutex::new(0.0)),
            last_update: Arc::new(Mutex::new(std::time::Instant::now())),
        }
    }

    /// Executa a leitura crua do arquivo do Kernel e converte a energia em Joules.
    #[cfg(target_os = "linux")]
    pub fn read_joules(&self) -> Result<f64, String> {
        if self.manual_override_tdp.is_some() {
            return self.estimate_joules_fallback();
        }

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
                    Err(format!("Erro ao acessar o sensor de energia RAPL: {}", e))
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    pub fn read_joules(&self) -> Result<f64, String> {
        if self.manual_override_tdp.is_some() {
            return self.estimate_joules_fallback();
        }
        // Verifica a presença do driver de baixo nível "EnergyLib.dll" instalado (Ex: Intel Power Gadget)
        unsafe {
            let lib_name = CString::new("EnergyLib32.dll").unwrap();
            let handle = LoadLibraryA(lib_name.as_ptr());
            if !handle.is_null() {}
        }
        
        // Aciona o Fallback Estimate propagando Warning ao loop
        let joules = self.estimate_joules_fallback().unwrap_or(0.0);
        Err(format!("Low-level block. Fallback estimate: {}", joules))
    }

    /// Executa a rotina matemática de TDP baseando-se por CPU usage via sysinfo
    fn estimate_joules_fallback(&self) -> Result<f64, String> {
        let mut sys = self.fallback_system.lock().unwrap();
        let mut joules = self.estimated_joules.lock().unwrap();
        let mut last = self.last_update.lock().unwrap();

        let now = std::time::Instant::now();
        let delta_secs = now.duration_since(*last).as_secs_f64();
        
        sys.refresh_cpu_usage();
        let global_cpu = sys.global_cpu_info().cpu_usage() as f64; // % Global
        
        // O TDP é o providenciado pelo override ou 65W padrão de desktop p/ estimação.
        let max_tdp = self.manual_override_tdp.unwrap_or(65.0); 
        
        // P = V * I -> Consumo estimado em Joules nesse delta de tempo.
        let watts_estimados = max_tdp * (global_cpu / 100.0);
        *joules += watts_estimados * delta_secs;
        *last = now;

        Ok(*joules)
    }
}
