# =====================================================================
# ARQUIVO: gravity_cli.py
# OBJETIVO: A Interface do Terminal (TUI) onde o usuário faz o "Pre-Flight"
# (Configuração de parâmetros) antes de ligar a inteligência do Gravity Monitor.
# =====================================================================

import sys # Interage com variáveis do sistema e configurações do interpretador
import os  # Interage com arquivos e pastas do sistema operacional

# MÁGICA DE IMPORTAÇÃO: Adiciona a pasta "src" aos lugares onde o Python procura por código.
# Sem isso, importar o 'carbon_api' ia dar o erro 'ModuleNotFoundError'
sys.path.append(os.path.join(os.path.dirname(__file__), "src"))
import carbon_api # Importa a nossa biblioteca híbrida (Proxy/Offline) de carbono

def configure(interactive=True):
    """
    Exibe um prompt interativo no terminal para o pesquisador
    substituir manualmente as especificações do hardware.
    Se 'interactive=False', ignora as perguntas e retorna o padrão de fábrica 
    ideal para rodar scripts de Deep Learning de forma 100% automatizada.
    Retorna o dicionário de configuração pra injetar no Rust.
    """
    # Se o dev programou o bot pra rodar sozinho (sem interação humana), devolve o padrão na hora.
    if not interactive:
        # Tenta buscar do Brasil em tempo real como fallback, pois o Electricity Maps não tem zona "Mundial"
        intensity, source = carbon_api.get_live_intensity("brazil")
        return {"pue": 1.0, "carbon_intensity": intensity, "carbon_intensity_source": source}

    print("==============================================")
    print("Gravity Monitor - Configuracao Inicial")
    print("==============================================\n")
    print("Como deseja iniciar o monitoramento?")
    print("1) [Padrão de Fábrica] Início Imediato (Autodetectar Sensores, PUE 1.0, Offline DB)")
    print("2) [Configuração Manual] Ajustar PUE, Forçar Limites de CPU/GPU e Configurar Carbono em Tempo Real")
    
    # Coleta a opção digitada pelo usuário e remove espaços invisíveis no começo e fim usando `.strip()`
    choice = input("\nSelecione o modo (1 ou 2) [Padrão: 1]: ").strip()
    
    # SE O USUÁRIO QUER DIGITAR O HARDWARE NA MÃO (Opção 2)
    if choice == "2":
        print("\n--- Configuração de Override Manual ---")
        try:
            # Pede o Limite de Carga do Processador em Watts
            cpu_tdp_raw = input("   > CPU Max TDP (Watts) [Padrão: 65.0]: ").strip()
            # O 'if' na mesma linha checa: se o usuário digitou algo, converte pra Float. Se ele só deu Enter (vazio), usa 65.0.
            cpu_tdp = float(cpu_tdp_raw) if cpu_tdp_raw else 65.0
            
            # Pede o Limite da Placa de Vídeo
            gpu_pwr_raw = input("   > GPU Max Power Limit (Watts) [Padrão: 250.0]: ").strip()
            gpu_pwr = float(gpu_pwr_raw) if gpu_pwr_raw else 250.0
            
            # Pede a Capacidade de Memória RAM
            ram_cap_raw = input("   > RAM Capacity (GB) [Padrão: 16.0]: ").strip()
            ram_cap = float(ram_cap_raw) if ram_cap_raw else 16.0
            
            # Pede a Eficiência do Prédio/Datacenter (PUE)
            pue_raw = input("   > Fator PUE (1.0 para Local, 1.2-1.5 para Nuvem) [Padrão: 1.0]: ").strip()
            pue = float(pue_raw) if pue_raw else 1.0
            
            # Pergunta sobre conectividade API de Carbono
            fetch_live = input("\n   > Deseja buscar a Intensidade de Carbono em Tempo Real? (S/N) [Padrão: N]: ").strip().upper()
            carbon_intensity = None
            carbon_source = None
            
            # Se o usuário digitou "S" (Sim), ativa a internet (ou o banco de dados)
            if fetch_live == 'S':
                country = input("     > País (ex: Brazil, USA): ").strip()
                state = input("     > Estado/Região (ex: Texas, SP) [Opcional]: ").strip()
                # Chama a nossa cascata super inteligente (Proxy -> Chave -> Offline)
                intensity, source = carbon_api.get_live_intensity(country, state)
                carbon_intensity = intensity
                carbon_source = source
                print(f"     [+] Intensidade Atual da Rede: {carbon_intensity} gCO2/kWh (Fonte: {carbon_source})")

            # Empacota todas as variáveis capturadas num Dicionário para devolver pra quem chamou a função
            config = {
                "cpu_tdp": cpu_tdp,
                "gpu_wattage": gpu_pwr,
                "ram_capacity": ram_cap,
                "pue": pue,
                "carbon_intensity": carbon_intensity,
                "carbon_intensity_source": carbon_source
            }
            print(f"\n[+] Configuração Aplicada: {config}\n")
            return config
            
        except ValueError:
            # Se o usuário sacanear e digitar letras onde pediu número, esse except captura o Crash do Python!
            print("\n[ERRO] Valor inserido não é numérico. Utilizando Padrão de Fábrica por segurança.\n")
            return {"pue": 1.0, "carbon_intensity": None, "carbon_intensity_source": None}
            
    # SE O USUÁRIO QUER O PADRÃO DE FÁBRICA (Opção 1)
    else:
        print("\n[INFO] Modo Padrão de Fábrica selecionado. Motores aquecendo de forma totalmente autônoma...")
        print("[INFO] Buscando intensidade de carbono em tempo real (Padrão: Brasil)...")
        intensity, source = carbon_api.get_live_intensity("brazil")
        
        # Retorna o dicionário padrão com os dados reais puxados da rede
        return {
            "pue": 1.0,
            "carbon_intensity": intensity,
            "carbon_intensity_source": source
        }

# Isso é para quando a pessoa roda: `python gravity_cli.py` diretamente do terminal para testar
if __name__ == "__main__":
    # Teste de importação de FFI nativo pra comprovar vínculo
    try:
        import gravity_monitor
        conf = configure(interactive=True)
        pue = conf.get("pue", 1.0) if conf else 1.0
        # Cria a instância pra ver se a compilação do Rust deu certo
        tracker = gravity_monitor.GravityTracker(pue=pue)
        
        # Se for manual, já injeta direto
        if conf and "cpu_tdp" in conf:
            tracker.cpu_tdp = conf.get("cpu_tdp")
            tracker.gpu_wattage = conf.get("gpu_wattage")
            tracker.ram_capacity = conf.get("ram_capacity")
            
        print("Motor atômico Rust (GravityTracker) inicializado com sucesso!")
    except ImportError:
        # Se não importou, a pessoa esqueceu de construir o código C no ambiente virtual
        print("Biblioteca nativa Gravity Monitor não encontada! Rode 'maturin develop' primeiro.")
