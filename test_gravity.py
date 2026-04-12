import time
import gravity_monitor

# Intensidade de Carbono Base: Valor médio indicativo para matrizes do Brasil em gCO2 por quilowatt-hora.
BRAZIL_INTENSITY = 82.0

def main():
    print("--- Inicializando Gravity Monitor via Python ---")
    
    # Instancia o subobjeto exportado puro do Rust via C-Bindings FFI (PyO3). Nenhum lixo de memória sujará o root script!
    tracker = gravity_monitor.GravityTracker()
    
    # Analisa na mosca as permissões para coleta Massiva de Hardware no nó da Placa de Vídeo
    if tracker.has_gpu():
        print("[+] GPU NVIDIA detectada! Adicionando NVML hardware aos cálculos de Joules cumulativos.")
    else:
        print("[-] Nenhuma GPU suportada detectada via NVML (Fallback para 0.0W GPU).")
    
    try:
        # Tenta iniciar a Thread paralela atômica e inviolável (Quebra com RuntimeError limpo caso rode sem 'sudo' na raiz linux)
        tracker.start()
        print("Monitoramento iniciado com sucesso!\n")
        
        # Simula inferência ou Pipeline de Deep Machine Learning Real
        # A mágica do Python FFI permite interceptar __enter__ e isolar Joules e Frações apenas neste espaço!
        with tracker.scope("limpeza_de_dados"):
            print("[...] Limpezando dados...")
            time.sleep(1) # Aqui estaria acontecendo um processamento brutal na Matriz
            
        # O marcador "treinamento_ia" capta e aloca uma tag Start, calculando no final a subtracao delta apenas deste percurso longo
        with tracker.scope("treinamento_ia"):
            print("[...] Treinando modelo (simulação de alto uso)...")
            time.sleep(3) # Carga Pesada Fictícia
            
        # Extração assíncrona on-the-fly sem precisar pausar os loops do backend
        watts = tracker.get_power()
        co2 = tracker.get_emissions(BRAZIL_INTENSITY)
        print(f"\n=> Finalizado: Potência final {watts:.2f} W  |  Emissão total: {co2:.6f} gCO2")
        
        print("\n=== Relatório de Etapas (Checkpoints) ===")
        try:
            # Requisita a Hash Table completa e já parseada das anotações Scope convertidas magicamente pra Dicionario!
            report_data = tracker.get_report(BRAZIL_INTENSITY)
            for passo, metrics in report_data.items():
                print(f" -> [{passo}]")
                print(f"      Energia: {metrics['joules']:.4f} Joules")
                print(f"      Fração do Total: {metrics['percentage']:.2f}%")
                print(f"      Emissão: {metrics['co2_equivalent']:.6f} gCO2")
        except TypeError as e:
            print("Aguardando nova compilacao", e)
            
        # Fechamento Gracioso Seguro (Aguarda ~150ms internamente no Buffer C pra drenar total_joules pra memória final)
        tracker.stop()
        
        # Joga as anotações globais finais p/ um serializador Rust exportando `resultado.json` mais veloz e puro!
        tracker.export_json('resultado.json', BRAZIL_INTENSITY)
        print("=> Resultados finais salvos com sucesso em 'resultado.json'.")
        
        print("\n=== Impacto Regional ===")
        # Mostra de fato num cálculo C++ os Princípios de Fisica Termodinâmica das simulações p/ Contexto Humano!
        print(f"🌊 {tracker.get_local_impact()}")
        
    except RuntimeError as e:
        # Tratador robusto para avisar o dev caso ele esqueca o usuário Root do script num ambiente virtual c/ restrições RAPL/ACPI.
        print(f"\n[Erro RuntimeError do Rust] Falha ao ler os sensores de hardware:")
        print(f" -> Detalhes do erro: {e}")
        print(" -> Dica: Você precisa rodar este script em nível de superusuário.")
        print("          Ex: sudo python3 test_gravity.py")
    except KeyboardInterrupt:
        print("\nSaindo... Monitoramento encerrado pelo usuário (Ctrl+C).")
        try:
            # Em acidentes ou crashes severos, garante serialização antes da destruição orginal da Stack C de objetos do PyO3.
            tracker.stop()
            tracker.export_json('resultado.json', BRAZIL_INTENSITY)
            print("=> Resultados finais salvos com sucesso em 'resultado.json'.")
        except Exception as e:
            print(f"Erro ao salvar os resultados finais: {e}")

if __name__ == '__main__':
    main()
