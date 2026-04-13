// src/permissions.rs

use std::fs;

/// Módulo encarregado de investigar e sugerir ajustes nas capacidades do sistema.
pub struct SystemCapabilities;

impl SystemCapabilities {
    /// Detecta problemas de privilégios e retorna a mensagem amigável apropriada
    /// de acordo com a plataforma em que foi compilado.
    pub fn get_elevation_message() -> String {
        #[cfg(target_os = "linux")]
        {
            // Checa ativamente a restrição moderna do Kernel Linux.
            let mut dica_extra = String::new();
            if let Ok(content) = fs::read_to_string("/proc/sys/kernel/perf_event_paranoid") {
                let status: i32 = content.trim().parse().unwrap_or(2);
                if status >= 2 {
                    dica_extra = format!(
"\n\n[DICA KERNEL]: O seu nível de perf_event_paranoid está restritivo ({}). \
Para testar métodos de telemetria sem root no futuro, considere:
sudo sh -c 'echo 1 > /proc/sys/kernel/perf_event_paranoid'", status);
                }
            }

            format!(
                "A permissão para ler sensores de hardware foi negada. \
Tente executar seu script com 'sudo'. Alternativamente, para evitar o uso de root, conceda as capacidades binárias ao interpretador Python:
'sudo setcap cap_sys_admin,cap_sys_rawio+ep <caminho_do_python>'{}", dica_extra
            )
        }

        #[cfg(target_os = "windows")]
        {
            String::from(
                "A permissão para acesso de hardware de baixo nível foi negada. \
Por favor, reinicie e utilize a opção 'Executar como Administrador' em seu Terminal/Python."
            )
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            String::from("A permissão foi negada. O sistema atual não é totalmente suportado.")
        }
    }

    /// Verificador de infraestrutura inicial. 
    /// No Windows irá procurar pelos drivers essenciais (MSRs etc).
    #[cfg(target_os = "windows")]
    pub fn check_windows_drivers() {
        println!("[Gravity Monitor] Aviso: Computador Windows Detectado. O sistema precisará do driver Intel Power Gadget ou WinRing0 para acesso a MSRs no futuro.");
        // Futura checagem de system32/drivers/WinRing0.sys ou similar.
    }
}
