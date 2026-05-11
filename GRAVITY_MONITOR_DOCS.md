# 🌍⚡ Gravity Monitor - Documentação Oficial

Este documento resume tudo o que foi planejado, arquitetado e desenvolvido na nossa biblioteca híbrida de medição energética e rastreamento de carbono.

---

## 1. O Coração do Sistema (Rust + PyO3)
O maior diferencial desta biblioteca é não depender da lentidão do Python para ler sensores. O motor foi escrito em **Rust** para garantir velocidade atômica e segurança de memória.

- **Comunicação Direta no Hardware:** A biblioteca acessa os registradores MSR (Model-Specific Registers) do Linux e a interface **RAPL** da Intel/AMD para ler o consumo em Joules direto da placa-mãe.
- **Multithreading Perfeita:** Uma Thread invisível em Rust acorda a cada 100 milissegundos, varre o hardware e soma a energia gasta em um "cofre" protegido por um `Mutex` e ponteiros inteligentes (`Arc`). Isso garante que, mesmo que o script de Inteligência Artificial em Python trave ou durma, a contagem de energia jamais perca um milissegundo de precisão.
- **Integração com GPU:** O sistema se conecta com as bibliotecas C da NVIDIA (NVML) para puxar o consumo instantâneo de placas de vídeo.
- **Exportação JSON Nativa (Serde):** Criamos a estrutura `ExportData` no Rust, que serializa os relatórios finais (incluindo `hardware_microjoules`) diretamente para o HD, garantindo uma performance insana sem sobrecarregar a memória do Python.

## 2. A Ponte para o Desenvolvedor (Python Context Manager)
Toda a complexidade do C++ e Rust foi escondida do pesquisador de Inteligência Artificial através de comandos mágicos do Python.

- **Checkpoints com `with`:** Através do comando `with tracker.scope("treinando_modelo"):`, o Python aciona gatilhos debaixo dos panos no Rust (`__enter__` e `__exit__`), marcando exatamente quantos Joules e gCO2 foram gastos apenas naquele trecho específico do código de IA.
- **Relatório Percentual:** Ao final, o sistema devolve uma matriz multidimensional dissecando qual fase do programa consumiu mais energia e gerou mais emissões.
- **Fator PUE:** A biblioteca considera a ineficiência natural de ar-condicionados e no-breaks dos Datacenters (Power Usage Effectiveness), multiplicando o consumo da placa pelo desperdício de resfriamento.

## 3. Inteligência de Carbono Híbrida em Cascata
O arquivo `carbon_api.py` introduziu um sistema revolucionário de rastreio de poluição para o desenvolvedor final não precisar de nenhum arquivo secreto `.env`.

A arquitetura usa uma lógica em **CASCATA (Anti-Crash)**:
1. **Cloudflare Proxy (Prioridade Máxima):** Criamos um Servidor Serverless `proxy_worker.js` que esconde a sua chave API e despacha os pedidos para o *Electricity Maps*.
2. **API Direta:** Caso o desenvolvedor queira usar sua própria chave.
3. **Banco de Dados Offline JSON (O Salva-vidas):** Se tudo falhar (sem internet, sem chaves, proxy fora do ar), o sistema consome arquivos `.json` minificados que carregam médias globais de carbono para que o script do pesquisador **nunca feche por erro de conexão**.
4. **Mapeamento de Regiões:** O sistema entende estados brasileiros (SP, PA, AM) e roteia automaticamente para os Subsistemas Elétricos corretos do ONS (BR-N, BR-NE, BR-S, BR-CS).

## 4. Interface do Terminal e Automatização em Deep Learning
Para que os pesquisadores de Machine Learning tenham máxima facilidade:

- **Terminal TUI (`gravity_cli.py`):** Um menu inicial lindo para simulação de hardware manual e customização de limites de consumo da Placa e Processador.
- **Modo Fantasma (Padrão de Fábrica):** Foi embutida a opção de invocar `gravity_cli.configure(interactive=False)`. Com isso, a biblioteca entra em modo furtivo: ela ignora qualquer pergunta no terminal, busca a intensidade de carbono do Brasil na internet em tempo real, ativa todos os sensores do Linux e já começa a gravar tudo de forma invisível no plano de fundo. Isso é vital para usar o Gravity em scripts automatizados no PyTorch.

## 5. Foco Didático Excepcional
A biblioteca não é apenas uma ferramenta, é um material de estudo.
- Absolutamente todos os arquivos vitais (desde o `proxy_worker.js`, os scripts em `Python`, até os complexos `main.rs` e `lib.rs`) foram dissecados e reescritos com **comentários didáticos em português**, linha a linha.
- O resultado térmico foi transladado para uma linguagem compreensível por seres humanos, traduzindo Microjoules puros em **"Litros de Água movidos nas marés de Salinópolis"**.

---
*Documento gerado em 24 de Abril de 2026. Antigravity System.*
