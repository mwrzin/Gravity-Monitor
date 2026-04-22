"""
gravity_cli.py
Terminal User Interface (TUI) para configuração de Override Manual do Gravity Monitor.
"""

def configure():
    """
    Exibe um prompt interativo no terminal para o pesquisador
    substituir manualmente as especificações do hardware.
    Retorna o dicionário de configuração pra injetar no Rust.
    """
    print("==============================================")
    print("🌍⚡ Gravity Monitor - Pre-Flight Configuration")
    print("==============================================\n")
    print("Escolha o modo de detecção de hardware:")
    print("1) [Auto-Detection] Buscar sensores nativos (MSR/RAPL/NVML)")
    print("2) [Manual Override] Forçar limites teóricos de hardware (Fallback heuristics)")
    
    choice = input("\nSelecione o modo (1 ou 2) [Padrão: 1]: ").strip()
    
    if choice == "2":
        print("\n--- Configuração de Override Manual ---")
        try:
            cpu_tdp_raw = input("   > CPU Max TDP (Watts) [Padrão: 65.0]: ").strip()
            cpu_tdp = float(cpu_tdp_raw) if cpu_tdp_raw else 65.0
            
            gpu_pwr_raw = input("   > GPU Max Power Limit (Watts) [Padrão: 250.0]: ").strip()
            gpu_pwr = float(gpu_pwr_raw) if gpu_pwr_raw else 250.0
            
            ram_cap_raw = input("   > RAM Capacity (GB) [Padrão: 16.0]: ").strip()
            ram_cap = float(ram_cap_raw) if ram_cap_raw else 16.0
            
            config = {
                "cpu_tdp": cpu_tdp,
                "gpu_wattage": gpu_pwr,
                "ram_capacity": ram_cap
            }
            print(f"\n[+] Configuração Aplicada: {config}\n")
            return config
        except ValueError:
            print("\n[ERRO] Valor inserido não é numérico. Utilizando fallback Auto-Detection por segurança.\n")
            return None
    else:
        print("\n[INFO] Prosseguindo com Auto-Detection puro.\n")
        return None

if __name__ == "__main__":
    # Teste de importação de FFI nativo pra comprovar vínculo
    try:
        import gravity_monitor
        conf = configure()
        tracker = gravity_monitor.GravityTracker()
        
        if conf:
            tracker.cpu_tdp = conf.get("cpu_tdp")
            tracker.gpu_wattage = conf.get("gpu_wattage")
            tracker.ram_capacity = conf.get("ram_capacity")
            
        print("Motor atômico Rust (GravityTracker) inicializado com sucesso!")
    except ImportError:
        print("Biblioteca nativa Gravity Monitor não encontada! Rode 'maturin develop' primeiro.")
