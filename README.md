# Gravity Monitor 🌍⚡

Uma biblioteca de alto desempenho em **Rust** exportada para computação em **Python** focada na telemetria de precisão de infraestruturas via Green AI.

Desenvolvido com excelência na **Universidade Federal do Pará (UFPA)**, com foco em conscientização termodinâmica, engenharia de baixo-nível e avaliação eficiente de processamento pesado para Inteligência Artificial.

## Diferenciais

- **Amostragem Mínima de 100ms Inviolável:** Uma background thread crua do Sistema Operacional puro coleta Deltas atômicos independente das falhas de Clock ou lentidões do script de sua IA. Diferente do padrão normal (ex: *CodeCarbon*), o **Gravity** não sofre gaps temporais.
- **Profiling por Escopo Analítico:** Uso inovador de *Context Managers* e anotações via Tags que medem o Joules e Custo em tempo real para blocos localizados (ex: `limpeza_de_dados`, `treinamento_pesado`).
- **Equivalência Termodinâmica Regional:** Um sistema revolucionário de conversão C++ de Joules brutos transistores em equivalências palpáveis (ex: O impacto energético local equiparado e traduzido para elevação das *Marés de Salinópolis* em dias quentes no Pará).

## Instalação e Compilação C

Esse projeto utiliza Python estrito com módulos FFI (Foreign Function Interfaces) do sistema construídos em Rust. Para construí-lo, o pacote universal **maturin** deve ser usado.

```bash
# 1. Instale o maturim no seu ambiente python
pip install maturin

# 2. Compile e injete o pacote no ambiente virtual ativamente (Release Mode / Otimizado)
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
