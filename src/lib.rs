// =====================================================================
// ARQUIVO: src/lib.rs
// OBJETIVO: O coração pulsante escrito em Rust de extrema performance.
// Aqui criamos a Classe que o Python vai enxergar e usar como se fosse dele.
// =====================================================================

use pyo3::prelude::*; // Importa a magia do PyO3 que permite traduzir memória do Rust para o Python
use pyo3::exceptions::PyRuntimeError; // Para jogar erros (Exceptions) bonitas na tela do Python se algo falhar
use serde::Serialize; // Para podermos converter os dados de Rust direto para um arquivo JSON hiper-rápido
use std::fs::File; // Para manipular a criação e escrita de arquivos no HD
use std::sync::{Arc, Mutex}; // Para proteger nossa Memória quando várias "Threads" (Processos) tentarem acessar ao mesmo tempo
use std::sync::atomic::{AtomicBool, Ordering}; // Booleanos ultra rápidos à prova de falhas para comunicação entre threads
use std::thread; // Para rodarmos a leitura de energia escondidos em segundo plano sem travar o Python
use std::time::Duration; // Para definir pausas perfeitas (ex: 100 milissegundos)

// Declaração dos sub-arquivos (módulos) do nosso projeto, onde ficam as lógicas complexas de cada peça de hardware.
pub mod cpu;
pub mod ram;
pub mod gpu;
pub mod carbon;
pub mod permissions;

/// Esta "Struct" (Estrutura) funciona como um molde para a exportação do JSON final.
/// O comando #[derive(Serialize)] diz para o Rust: "Escreva o código sozinho que converte isso pra JSON pra mim".
#[derive(Serialize)]
pub struct ExportData {
    pub timestamp: u64,           // Marcação do relógio mundial (Unix Epoch)
    pub watts: f64,               // Consumo total em Watts
    pub hardware_joules: f64,     // Consumo bruto extraído direto da placa mãe
    pub hardware_microjoules: f64,// A mesma energia bruta, mas em microjoules (Joules * 1.000.000)
    pub total_facility_joules: f64, // Consumo considerando a ineficiência do ar-condicionado do prédio (PUE)
    pub co2_emissions: f64,       // Massa física de gás carbônico gerada
    pub estimated_fallback: bool, // Se for True, significa que não tínhamos root/sudo e precisamos estimar matematicamente
    pub pue_applied: f64,         // Fator PUE aplicado
    pub intensity_source: String, // Qual foi a ponte usada? (API Direta, Proxy do Cloudflare, ou Banco Offline?)
}

/// A "Classe" Raiz exportada para o Python. A macro #[pyclass] avisa o compilador do C que
/// essa estrutura de memória do Rust deve ser embalada em um objeto Python!
#[pyclass]
pub struct GravityTracker {
    // ----------------------------------------------------
    // OS SENSORES
    // ----------------------------------------------------
    pub sensor: cpu::CpuSensor,
    pub ram_sensor: ram::RamSensor,
    pub gpu_sensor: gpu::GpuSensor,
    
    // ----------------------------------------------------
    // CONTROLES DE TEMPO E ESTADO
    // ----------------------------------------------------
    pub start_time: Option<std::time::Instant>, // Cronômetro de altíssima precisão
    
    // O Coração da Threading Segura: Arc(Ponteiro Inteligente) -> Mutex(Cadeado de Memória) -> f64 (Número decimal)
    // O Mutex impede o desastre chamado "Race Condition". Imagine duas pessoas tentando editar o mesmo
    // arquivo de texto ao mesmo tempo: um apaga o trabalho do outro. O Mutex cria uma "fila" instantânea.
    pub total_joules_accumulated: Arc<Mutex<f64>>,
    
    // Boolean atômico que serve de botão liga/desliga para a Thread secreta.
    pub is_running: Arc<AtomicBool>,
    
    // Vetor (Lista) que guarda o nome de cada passo e a energia gasta no momento exato (o famoso _start e _end)
    pub checkpoints: Vec<(String, f64)>,

    // Aviso permanente caso a gente caia na estimação (sem privilégios do Linux)
    pub estimated_fallback: Arc<AtomicBool>,

    // ----------------------------------------------------
    // METADADOS DO AMBIENTE
    // ----------------------------------------------------
    pub pue: f64,
    pub carbon_intensity: Option<f64>,
    pub carbon_intensity_source: Option<String>,
}

// O bloco #[pymethods] significa: Todas as funções aqui dentro podem ser chamadas pelo Python!
#[pymethods]
impl GravityTracker {
    
    /// O Construtor: Chamado quando o Python faz `tracker = gravity_monitor.GravityTracker()`
    #[new]
    #[pyo3(signature = (pue=None, carbon_intensity=None, carbon_intensity_source=None))]
    pub fn new(pue: Option<f64>, carbon_intensity: Option<f64>, carbon_intensity_source: Option<String>) -> Self {
        // Se estivermos compilando no Windows, chama uma função especial que tenta abrir comunicação com drivers
        #[cfg(target_os = "windows")]
        permissions::SystemCapabilities::check_windows_drivers();

        Self {
            sensor: cpu::CpuSensor::new(None),
            ram_sensor: ram::RamSensor::new(None),
            gpu_sensor: gpu::GpuSensor::new(None),
            start_time: None,
            // Mutex começa trancando o número "0.0" dentro dele
            total_joules_accumulated: Arc::new(Mutex::new(0.0)),
            is_running: Arc::new(AtomicBool::new(false)),
            checkpoints: Vec::new(),
            estimated_fallback: Arc::new(AtomicBool::new(false)),
            pue: pue.unwrap_or(1.0), // Se o Python mandou Vazio (None), vira 1.0
            carbon_intensity,
            carbon_intensity_source,
        }
    }

    // Setters que permitem o Python injetar configurações de emergência/override diretamente na memória do Rust
    #[setter]
    pub fn set_cpu_tdp(&mut self, value: Option<f64>) { self.sensor.manual_override_tdp = value; }

    #[setter]
    pub fn set_gpu_wattage(&mut self, value: Option<f64>) { self.gpu_sensor.manual_override_power = value; }

    #[setter]
    pub fn set_ram_capacity(&mut self, value: Option<f64>) { self.ram_sensor.manual_override_capacity = value; }

    /// Retorna pro Python `True` ou `False` se encontrou fisicamente uma placa da NVIDIA na máquina.
    pub fn has_gpu(&self) -> PyResult<bool> {
        Ok(self.gpu_sensor.has_gpu)
    }

    /// O INICIALIZADOR: Onde a Thread secreta de 100ms é criada!
    pub fn start(&mut self) -> PyResult<()> {
        if self.is_running.load(Ordering::SeqCst) {
            return Err(PyRuntimeError::new_err("O monitoramento já está em andamento."));
        }

        // Bate o cronômetro do Ponto Zero e liga as Flags
        self.start_time = Some(std::time::Instant::now());
        self.is_running.store(true, Ordering::SeqCst);
        *self.total_joules_accumulated.lock().unwrap() = 0.0;

        // O Rust obriga que a Thread paralela receba "Clones" das referências de memória. 
        // Ele não deixa uma Thread roubar o original e depois a Thread Principal morrer e deixar lixo pra trás.
        let cpu_sensor_clone = self.sensor.clone();
        let ram_sensor_clone = self.ram_sensor.clone();
        let gpu_sensor_clone = self.gpu_sensor.clone();
        let is_running_clone = self.is_running.clone();
        let acc_clone = self.total_joules_accumulated.clone();
        let fallback_clone = self.estimated_fallback.clone();

        // Leitura Zero (Antes de começar o Loop): Tenta ler a energia bruta que estava rolando no PC antes do script.
        let last_cpu = match cpu_sensor_clone.read_joules() {
            Ok(v) => v, // Se leu os registradores do Linux com sucesso, devolve o valor.
            Err(e) => {
                // Se deu Permissão Negada (falta de Sudo), avisa o Python enviando um Warning amarelo na tela do cara!
                Python::with_gil(|py| {
                    if let Ok(warnings) = py.import_bound("warnings") {
                        let msg = format!("Falha de Hardware - Ativando Fallback Anti-Crash.\nMotivo: {}", e);
                        let _ = warnings.call_method1("warn", (msg, py.get_type_bound::<pyo3::exceptions::PyRuntimeWarning>()));
                    }
                });
                // Ativa permanentemente o selo de simulação matemática (Fallback)
                fallback_clone.store(true, Ordering::SeqCst);
                0.0
            }
        };
        let mut last_cpu_mut = last_cpu;
        let mut last_ram = ram_sensor_clone.read_joules().unwrap_or(0.0);
        let mut last_gpu = gpu_sensor_clone.read_joules().unwrap_or(0.0);

        // NASCIMENTO DA THREAD DE BACKGROUND
        // Essa thread roda diretamente no Processador (via C/Rust), escapando de toda a lentidão e bloqueio do Python (GIL).
        thread::spawn(move || {
            // Enquanto o botão de power estiver ligado (True)...
            while is_running_clone.load(Ordering::SeqCst) {
                
                // Dorme estritamente por 100 milissegundos.
                thread::sleep(Duration::from_millis(100));

                // Acorda e bate fotos instantâneas do consumo atual do PC
                let current_cpu = cpu_sensor_clone.read_joules().unwrap_or(last_cpu_mut);
                let current_ram = ram_sensor_clone.read_joules().unwrap_or(last_ram);
                let current_gpu = gpu_sensor_clone.read_joules().unwrap_or(last_gpu);

                // Deduzir o Gasto Físico: (Consumo Agora - Consumo da leitura passada = A Energia Gasta nesses últimos 100ms)
                let mut delta_cpu = current_cpu - last_cpu_mut;
                let mut delta_ram = current_ram - last_ram;
                let mut delta_gpu = current_gpu - last_gpu;
                
                // Previne valores negativos (Ex: a placa de vídeo reiniciou a fita de contador no nível do Kernel)
                if delta_cpu < 0.0 { delta_cpu = 0.0; }
                if delta_ram < 0.0 { delta_ram = 0.0; }
                if delta_gpu < 0.0 { delta_gpu = 0.0; }

                // Atualiza a fita do tempo pra próxima passada daqui a 100ms
                last_cpu_mut = current_cpu;
                last_ram = current_ram;
                last_gpu = current_gpu;

                // Destranca o Cadeado do Cofre (Mutex), adiciona a nossa energia no montante geral, e tranca na mesma hora!
                let mut acc = acc_clone.lock().unwrap();
                *acc += delta_cpu + delta_ram + delta_gpu;
            }
        });

        println!("Monitoramento iniciado em background (100ms amostragem)...");
        Ok(())
    }


    /// Desliga o monitoramento seguro e mata a Thread paralela
    pub fn stop(&mut self) -> PyResult<()> {
        // Envia Falso pro Loop. Na próxima vez que o Loop acordar, ele morre naturalmente.
        self.is_running.store(false, Ordering::SeqCst);
        
        // Espera 150ms. Por que 150? Porque o Loop acorda de 100 em 100ms.
        // Assim temos 100% de certeza que demos tempo pra ele terminar o cálculo e depositar a última conta no Mutex.
        thread::sleep(Duration::from_millis(150));
        println!("Monitoramento em background encerrado.");
        Ok(())
    }

    /// Devolve para o Python (mesmo com o código rodando em tempo real) a Potência Média em Watts.
    pub fn get_power(&self) -> PyResult<f64> {
        match self.start_time {
            Some(time) => {
                let delta_seconds = time.elapsed().as_secs_f64();
                let acc = *self.total_joules_accumulated.lock().unwrap();
                if delta_seconds > 0.0 {
                    // P=E/T (Potência(W) = Energia(J) divido pelo Tempo(s))
                    Ok(acc / delta_seconds)
                } else {
                    Ok(0.0)
                }
            }
            None => Err(PyRuntimeError::new_err("O monitoramento ainda não foi iniciado. Chame o método start() primeiro.")),
        }
    }

    /// Converte a brincadeira em impactos visuais do Brasil (Salinópolis/Praia)
    pub fn get_local_impact(&self) -> PyResult<String> {
        let acc = *self.total_joules_accumulated.lock().unwrap();
        Ok(carbon::get_salinas_equivalence(acc))
    }

    /// Calcula a pegada térmica exata combinando a ineficiência (PUE) e multiplicando pela sujeira da sua Rede Elétrica.
    pub fn get_emissions(&self, intensity: f64) -> PyResult<f64> {
        match self.start_time {
            Some(_) => {
                let acc = *self.total_joules_accumulated.lock().unwrap();
                let facility_joules = acc * self.pue;
                // Usa a intensidade ao vivo do Maps/Proxy, se não achou, usa o que o Python mandou na unha
                let actual_intensity = self.carbon_intensity.unwrap_or(intensity);
                Ok(carbon::calculate_emissions(facility_joules, actual_intensity))
            },
            None => Err(PyRuntimeError::new_err("O monitoramento ainda não foi iniciado. Chame o método start() primeiro.")),
        }
    }

    /// O Famoso sistema de "Etiquetas". O Python diz "Oxe, marque aqui que eu to no Passo X".
    /// O Rust pega o nível de energia do Cofre naquele EXATO MILISSEGUNDO e anota num caderninho (Vetor).
    pub fn mark_step(&mut self, step_name: &str) {
        let current_joules = *self.total_joules_accumulated.lock().unwrap();
        self.checkpoints.push((step_name.to_string(), current_joules));
    }

    /// Ao final do treinamento, o Python chama o `get_report`.
    /// Aqui o Rust faz uma conta pesada transformando as Etiquetas Start/End numa fatia perfeita de Bolo percentual.
    pub fn get_report(&self, intensity: f64) -> PyResult<std::collections::HashMap<String, std::collections::HashMap<String, f64>>> {
        let mut report = std::collections::HashMap::new();
        let mut starts = std::collections::HashMap::new();
        let total_consumed = *self.total_joules_accumulated.lock().unwrap();

        for (name, joules) in &self.checkpoints {
            if name.ends_with("_start") {
                // Remove o sufíxo "_start" e guarda o nome (Ex: treinamento_ia_start -> treinamento_ia)
                let base_name = name.trim_end_matches("_start");
                starts.insert(base_name.to_string(), *joules);
            } else if name.ends_with("_end") {
                let base_name = name.trim_end_matches("_end");
                // Se a gente achar a Etiqueta Start daquele nome... 
                if let Some(start_joules) = starts.get(base_name) {
                    // Subtrai: (Total do Fim) - (Total que tava no Começo) = Energia pura consumida só nessa fatia
                    let cost = joules - start_joules;
                    let facility_cost = cost * self.pue;
                    // Regra de três básica pra descobrir se essa fatia usou 10% ou 80% de todo o consumo geral
                    let percentage = if total_consumed > 0.0 { (cost / total_consumed) * 100.0 } else { 0.0 };
                    let actual_intensity = self.carbon_intensity.unwrap_or(intensity);
                    let co2 = carbon::calculate_emissions(facility_cost, actual_intensity);
                    
                    let mut data = std::collections::HashMap::new();
                    data.insert("hardware_joules".to_string(), cost);
                    data.insert("total_facility_joules".to_string(), facility_cost);
                    data.insert("percentage".to_string(), percentage);
                    data.insert("co2_equivalent".to_string(), co2);
                    
                    report.insert(base_name.to_string(), data);
                }
            }
        }
        Ok(report) // Retorna o Dicionário gigantesco e o PyO3 magicamente converte num "dict" pro Python
    }

    /// O `scope` cria um objeto secundário chamado `TrackerScope`.
    /// É isso que permite o comando lindo `with tracker.scope("treinar_modelo"):` existir no Python!
    pub fn scope(slf: Py<Self>, step_name: String) -> PyResult<TrackerScope> {
        Ok(TrackerScope {
            tracker: slf.clone(),
            step_name,
        })
    }

    /// Escreve um JSON no HD em altíssima velocidade pulando as barreiras do Python.
    pub fn export_json(&self, filename: &str, intensity: f64) -> PyResult<()> {
        match self.start_time {
            Some(time) => {
                let delta_seconds = time.elapsed().as_secs_f64();
                let acc = *self.total_joules_accumulated.lock().unwrap();
                
                let watts = if delta_seconds > 0.0 { acc / delta_seconds } else { 0.0 };
                let facility_joules = acc * self.pue;
                let actual_intensity = self.carbon_intensity.unwrap_or(intensity);
                let co2 = carbon::calculate_emissions(facility_joules, actual_intensity);
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                // Monta a caixa (Struct) que criamos no começo do arquivo
                let data = ExportData {
                    timestamp,
                    watts,
                    hardware_joules: acc,
                    hardware_microjoules: acc * 1_000_000.0,
                    total_facility_joules: facility_joules,
                    co2_emissions: co2,
                    estimated_fallback: self.estimated_fallback.load(Ordering::SeqCst),
                    pue_applied: self.pue,
                    intensity_source: self.carbon_intensity_source.clone().unwrap_or_else(|| "fallback_argument".to_string()),
                };

                let file = File::create(filename).map_err(|e| PyRuntimeError::new_err(format!("Erro ao criar arquivo: {}", e)))?;
                // Empurra o JSON diretamente pro arquivo de forma linda (`pretty`)
                serde_json::to_writer_pretty(file, &data).map_err(|e| PyRuntimeError::new_err(format!("Erro ao serializar JSON: {}", e)))?;
                Ok(())
            }
            None => Err(PyRuntimeError::new_err("O monitoramento ainda não foi iniciado. Chame o método start() primeiro.")),
        }
    }
}

/// A Segunda Classe do Rust: Representa o escopo individual `with tracker.scope(...)` do Python
#[pyclass]
pub struct TrackerScope {
    tracker: Py<GravityTracker>,
    step_name: String,
}

#[pymethods]
impl TrackerScope {
    /// A Magia Negra (Dunder Method __enter__). 
    /// O Python chama isso sozinho quando entra na palavra "with".
    /// Nós imediatamente mandamos o Rust marcar a "Etiqueta Start".
    fn __enter__(&self, py: Python<'_>) -> PyResult<()> {
        let mut tracker = self.tracker.borrow_mut(py);
        tracker.mark_step(&format!("{}_start", self.step_name));
        Ok(())
    }

    /// O Destrutor Natural (__exit__). 
    /// O Python chama isso sozinho quando a indentação do "with" acaba, ou até MESMO SE O SCRIPT DER ERRO!
    /// Isso é genial, porque nós sempre marcamos o _end mesmo se o modelo de IA der crash na memória,
    /// garantindo que o desenvolvedor tenha a energia gasta até o segundo antes do Kernel Panic.
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
        Ok(false) // Retornar 'false' significa "Pode prosseguir com o Erro pro usuário, não vou calar a Exception".
    }
}

/// O BOOTLOADER. O Carregador principal do módulo C Extension.
/// Aqui dizemos pra biblioteca CPython do C++ que nosso nome oficial é `gravity_monitor` 
/// e exportamos as Classes que criamos pro ecossistema Python nativamente!
#[pymodule]
fn gravity_monitor(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<GravityTracker>()?;
    m.add_class::<TrackerScope>()?;
    Ok(())
}
