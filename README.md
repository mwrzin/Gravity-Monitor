# Gravity Monitor 🌍⚡

Uma biblioteca de alto desempenho em **Rust** exportada para computação em **Python** focada na telemetria de precisão de infraestruturas via Green AI.

Desenvolvido com excelência na **Universidade Federal do Pará (UFPA)**, com foco em conscientização termodinâmica, engenharia de baixo-nível e avaliação eficiente de processamento pesado para Inteligência Artificial.

## Diferenciais

- **Amostragem Mínima de 100ms Inviolável:** Uma background thread crua do Sistema Operacional puro coleta Deltas atômicos independente das falhas de Clock ou lentidões do script de sua IA. Diferente do padrão normal (ex: *CodeCarbon*), o **Gravity** não sofre gaps temporais.
- **Profiling por Escopo Analítico:** Uso inovador de *Context Managers* e anotações via Tags que medem o Joules e Custo em tempo real para blocos localizados (ex: `limpeza_de_dados`, `treinamento_pesado`).
- **Equivalência Termodinâmica Regional:** Um sistema revolucionário de conversão C++ de Joules brutos transistores em equivalências palpáveis (ex: O impacto energético local equiparado e traduzido para elevação das *Marés de Salinópolis* em dias quentes no Pará).

## Pré-Requisitos e Compatibilidade

A telemetria de Hardware puro é uma ciência restrita em segurança arquitetural. Siga os requisitos abaixo dependendo do seu ecossistema alvo:

### 🐧 Linux (Recomendado para Datacenters AI)
- **Privilégios:** O monitoramento nativo limpo exige a leitura dos Model-Specific Registers (MSRs) do chip e do sensor RAPL de fábrica ou o uso do perf_event_paranoid. É **obrigatório** rodar o seu treinamento usando `$ sudo python script.py`.
- **Dependências de Build:** Rust Toolchain instalada (`cargo`), Python 3.12+, GCC/Build-essentials limpos instalados em sua distribuição.

### 🪟 Windows (Recomendado para Estações Locais)
- **Privilégios:** Como a Microsoft bloqueia incondicionalmente a manipulação de MSRs por ferramentas externas via kernel Ring-0 sem drivers verificados/assinados, a CPU e a RAM operam em **Simulação Heurística (Fallback Anti-Crash)** via detecção atômica. O Gravity Monitor não quebra no Windows.
- **Placas de Vídeo (NVML):** Telemetria em 100% de precisão para GPUs Nvidia utilizando a injeção nativa de `nvml.dll`.
- **Pre-Flight Configuration:** No Windows, recomenda-se iniciar o monitor setando manualmente o TDP máximo teórico da peça por meio do nosso terminal interativo TUI (veja abaixo).

## Como Instalar e Compilar (C-Bindings)

Esse projeto utiliza Python estrito atrelado a módulos de Foreign Function Interface (FFI) compilados isoladamente pela base em Rust. Instancie assim:

```bash
# 1. Ative seu Python Virtual Environment (venv)
# 2. Instale as pontes de build e a dependência nativa no PyPi
pip install maturin sysinfo

# 3. Compile e construa os binários estáticos injetando magicamente na sua Venv (Modo Otimizado)
maturin develop --release
```

## Como Usar

O uso é massivamente integrado. Exporte seus testes, treine sua IA e receba os dados transparentemente nos coletores:

```python
import time
import gravity_monitor

# Fator base: emissão em gCO2 por kWh (Ex: Matriz do Brasil = 82.0)
BRAZIL_INTENSITY = 82.0

# Inicia de instante limpo o Coletor de Hardware Background
tracker = gravity_monitor.GravityTracker()
tracker.start()

# Demarca zonas exatas de consumo na esteira de Profiling do código
with tracker.scope("treinamento_ia"):
    print("[...] Treinando modelo (simulação de alto uso)...")
    time.sleep(3) # Carga intensiva aqui

# Coleta de resultados
watts = tracker.get_power()
print(f"Potência final consumida: {watts:.2f} W")

# Métrica consciente de Impacto Climático
print(f"Impacto da Computação: {tracker.get_local_impact()}")

# Destruidor Seguro no Python FFI
tracker.stop()
```

## Licença
Este projeto possui licença MIT. Veja o arquivo de [LICENÇA](LICENSE) para maiores detalhes. Projetado e elaborado de pesquisadores, para pesquisadores.
