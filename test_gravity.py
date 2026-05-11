# =====================================================================
# ARQUIVO: test_gravity.py
# OBJETIVO: Este é o arquivo principal que o Usuário final (Pesquisador) roda.
# Ele simula um treinamento de Inteligência Artificial usando nossa biblioteca.
# =====================================================================

import time             # Biblioteca padrão para pausar a execução (simular o tempo passando)
import gravity_monitor  # Importa a nossa poderosa biblioteca escrita em Rust (C-Bindings)

# Intensidade de Carbono Base: Um valor fixo inicial usado antes de perguntar ao Cloudflare
BRAZIL_INTENSITY = 82.0

def main():
    import gravity_cli # Importa o nosso menu de terminal interativo
    print("--- Inicializando Gravity Monitor via Python ---")
    
    # 1. EXECUTA O PRE-FLIGHT (Menu do Terminal)
    # Chama a função configure(), que interage com o usuário perguntando sobre PUE e Região.
    conf = gravity_cli.configure()
    
    # Extrai as configurações que o usuário digitou no menu.
    # O .get() é excelente porque evita erros: se a chave "pue" não existir, ele retorna 1.0 (o padrão local)
    pue = conf.get("pue", 1.0) if conf else 1.0
    carbon_intensity = conf.get("carbon_intensity") if conf else None
    carbon_intensity_source = conf.get("carbon_intensity_source") if conf else None
    
    # 2. INSTANCIANDO O MOTOR DE RUST NO PYTHON
    # Aqui a mágica acontece. Estamos criando um objeto `GravityTracker`.
    # Diferente de classes Python normais, este objeto aloca memória diretamente no C/Rust!
    # Isso impede que o "Garbage Collector" do Python atrapalhe a cronometragem.
    tracker = gravity_monitor.GravityTracker(pue=pue, carbon_intensity=carbon_intensity, carbon_intensity_source=carbon_intensity_source)
    
    # 3. OVERRIDE DINÂMICO (Modo Manual)
    # Se o usuário escolheu o modo 2 no menu (onde ele digita os Watts na mão)...
    if conf and "cpu_tdp" in conf:
        tracker.cpu_tdp = conf.get("cpu_tdp")
        tracker.gpu_wattage = conf.get("gpu_wattage")
        tracker.ram_capacity = conf.get("ram_capacity")
        print("\n[+] Override ativado. Os próximos passos rodarão no modo de simulação Anti-Crash!")
    
    # 4. VERIFICAÇÃO DE GPU
    # Chama a função em Rust para ver se uma placa NVIDIA física foi achada.
    if tracker.has_gpu():
        print("\n[+] GPU NVIDIA detectada! Adicionando NVML hardware aos cálculos de Joules cumulativos.")
    else:
        print("\n[-] Nenhuma GPU suportada detectada via NVML (Fallback para 0.0W/Estimação).")
    
    # 5. BLOCO PRINCIPAL (Tentativa de Execução)
    try:
        # Dá o comando de partida para a thread em Rust. Ela vai começar a captar Joules a cada 100ms em background.
        # Pode gerar um 'RuntimeError' se o cara não tiver rodando com 'sudo' (permissão negada no Linux).
        tracker.start()
        print("Monitoramento iniciado com sucesso!\n")
        
        # O BLOCO DE ESCOPO 1: LIMPEZA DE DADOS
        # A instrução 'with' no Python chama duas funções secretas no Rust: __enter__ no começo e __exit__ no fim.
        # Isso demarca na linha do tempo exatamente onde esse trecho de código gastou energia.
        with tracker.scope("limpeza_de_dados"):
            print("[...] Limpezando dados...")
            time.sleep(1) # Simula que a limpeza de dados demorou 1 segundo
            
        # O BLOCO DE ESCOPO 2: TREINAMENTO DE IA
        with tracker.scope("treinamento_ia"):
            print("[...] Treinando modelo (simulação de alto uso)...")
            time.sleep(3) # Simula que treinar o modelo demorou 3 segundos consumindo muita energia
            
        # 6. RESULTADOS EM TEMPO REAL
        # Puxa os dados da Thread do Rust pra tela do Python sem precisar pausar a Thread.
        watts = tracker.get_power()
        co2 = tracker.get_emissions(BRAZIL_INTENSITY)
        print(f"\n=> Finalizado: Potência final {watts:.2f} W  |  Emissão total: {co2:.6f} gCO2")
        
        print("\n=== Relatório de Etapas (Checkpoints) ===")
        try:
            # Chama o motor Rust para processar a matemática de quanto cada 'Scope' consumiu sozinho e em %
            report_data = tracker.get_report(BRAZIL_INTENSITY)
            # O Python varre o Dicionário que o Rust devolveu
            for passo, metrics in report_data.items():
                print(f" -> [{passo}]")
                print(f"      Hardware Energia: {metrics['hardware_joules']:.4f} Joules")
                print(f"      Facility Energia (c/ PUE): {metrics['total_facility_joules']:.4f} Joules")
                print(f"      Fração do Total: {metrics['percentage']:.2f}%")
                print(f"      Emissão: {metrics['co2_equivalent']:.6f} gCO2")
        except TypeError as e:
            print("Aguardando nova compilacao", e)
            
        # 7. FECHAMENTO SEGURO
        # Manda a thread do Rust morrer com calma (150ms pra drenar o buffer de memória).
        tracker.stop()
        
        # Pede pro próprio Rust salvar o JSON no disco (muito mais rápido que o Python serializar).
        tracker.export_json('resultado.json', BRAZIL_INTENSITY)
        print("=> Resultados finais salvos com sucesso em 'resultado.json'.")
        
        print("\n=== Impacto Regional ===")
        # Pega a metáfora (Salinópolis) direto do arquivo C++ (ou Rust).
        print(f"{tracker.get_local_impact()}")
        
    except RuntimeError as e:
        # SE CAIR AQUI: É porque a Thread do Rust rejeitou iniciar por conta das permissões restritas do Kernel Linux (RAPL)
        print(f"\n[Erro RuntimeError do Rust] Falha ao ler os sensores de hardware:")
        print(f" -> Detalhes do erro: {e}")
        print(" -> Dica: Você precisa rodar este script em nível de superusuário.")
        print("          Ex: sudo python3 test_gravity.py")
        
    except KeyboardInterrupt:
        # SE CAIR AQUI: É porque o usuário deu "Ctrl+C" no meio do processo pra matar o script violentamente.
        print("\nSaindo... Monitoramento encerrado pelo usuário (Ctrl+C).")
        try:
            # Tenta um desligamento cirúrgico de emergência pra pelo menos salvar os dados até aquele segundo!
            tracker.stop()
            tracker.export_json('resultado.json', BRAZIL_INTENSITY)
            print("=> Resultados finais salvos com sucesso em 'resultado.json'.")
        except Exception as e:
            print(f"Erro ao salvar os resultados finais: {e}")

# Só executa a 'main()' se o usuário estiver rodando este arquivo diretamente (e não importando ele).
if __name__ == '__main__':
    main()
