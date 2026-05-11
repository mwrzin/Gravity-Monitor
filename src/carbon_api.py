import os
import requests # Biblioteca famosa do Python para fazer requisições na internet (baixar páginas, acessar APIs)
import json     # Biblioteca para ler e escrever arquivos no formato JSON (JavaScript Object Notation)

def load_local_env():
    """
    Tenta carregar o arquivo .env se existir, sem depender de bibliotecas externas (como python-dotenv).
    Isso mantém a sua biblioteca leve e com menos dependências para o usuário instalar.
    """
    # Descobre o caminho absoluto do arquivo atual, sobe um diretório (dirname duas vezes) e aponta para o '.env'
    env_path = os.path.join(os.path.dirname(os.path.dirname(__file__)), '.env')
    
    # Se o arquivo .env existir na pasta raiz...
    if os.path.exists(env_path):
        # Abre o arquivo em modo de leitura ('r')
        with open(env_path, 'r') as f:
            # Lê linha por linha
            for line in f:
                line = line.strip() # Remove espaços em branco e quebras de linha do começo e fim
                # Se a linha não estiver vazia, não for um comentário (#) e tiver um sinal de igual...
                if line and not line.startswith('#') and '=' in line:
                    key, value = line.split('=', 1) # Divide a linha no primeiro '=' que encontrar
                    # Só injeta no sistema se a variável já não existir (não sobrescreve o sistema operacional real)
                    if key not in os.environ:
                        os.environ[key] = value.strip('"\'') # Salva removendo aspas duplas ou simples

# Executa a função imediatamente quando este arquivo for importado por outro script
load_local_env()


def load_fallback_data():
    """
    Lê o banco de dados offline (carbon_data.json) para o caso de não haver internet ou API Key.
    """
    try:
        # Pega o caminho exato do 'carbon_data.json' que deve estar na mesma pasta deste script ('src')
        data_path = os.path.join(os.path.dirname(__file__), 'carbon_data.json')
        with open(data_path, 'r') as f:
            return json.load(f) # Converte o arquivo de texto JSON para um Dicionário Python real
    except Exception:
        # Se o arquivo foi deletado ou deu erro, retorna um valor global padrão de sobrevivência extrema
        return {"GLOBAL_AVERAGE": 475.0}

# Cria duas variáveis globais constantes que ficarão na memória o tempo todo
FALLBACK_INTENSITIES = load_fallback_data() 
DEFAULT_FALLBACK = FALLBACK_INTENSITIES.get("GLOBAL_AVERAGE", 475.0)


def get_zone_code(country: str, state: str = None) -> str:
    """
    Mapeia nomes comuns de países e estados (ex: 'Brazil', 'PA') para os códigos ISO 
    usados internamente pelo Electricity Maps (ex: 'BR-N').
    """
    # Dicionário de tradução de Países
    country_map = {
        "brazil": "BR",
        "norway": "NO",
        "usa": "US",
        "united states": "US",
        "uk": "GB",
        "united kingdom": "GB",
        "germany": "DE",
        "france": "FR",
        "india": "IN"
    }
    
    # Dicionário de tradução de Estados Americanos
    state_map = {
        "texas": "TEX",
        "california": "CA",
        "new york": "NY",
        "florida": "FL"
    }

    # Transforma 'Brazil ' em 'brazil' para não dar erro se o usuário digitar com maiúscula
    c = country.lower().strip()
    
    # Tenta achar no dicionário. Se não achar, chuta o que o usuário digitou mas em MAIÚSCULO (ex: 'MX' viraria 'MX')
    zone = country_map.get(c, c.upper()) 

    # Se o usuário também digitou um estado...
    if state:
        s = state.lower().strip()
        
        # Regra específica pros EUA (ex: US-TEX)
        if zone == "US":
            state_zone = state_map.get(s, s.upper())
            zone = f"US-{state_zone}"
            
        # Regra específica pro BRASIL (divide os 27 estados nas 4 zonas macro do grid energético)
        elif zone == "BR":
            norte = ['ac', 'ap', 'am', 'pa', 'ro', 'rr', 'to']
            nordeste = ['al', 'ba', 'ce', 'ma', 'pb', 'pe', 'pi', 'rn', 'se']
            sul = ['pr', 'rs', 'sc']
            centro_sul = ['go', 'mt', 'ms', 'df', 'es', 'mg', 'rj', 'sp']
            
            if s in norte:
                zone = "BR-N"
            elif s in nordeste:
                zone = "BR-NE"
            elif s in sul:
                zone = "BR-S"
            elif s in centro_sul:
                zone = "BR-CS"
            else:
                zone = "BR" # Fallback: Se não souber qual estado é, consulta a média do Brasil inteiro
                
    return zone # Retorna a string montada


def get_live_intensity(country: str, state: str = None) -> float:
    """
    Função Principal. Tenta buscar a intensidade de carbono em tempo real.
    Segue a Arquitetura em Cascata: Proxy -> Chave Direta -> Banco de Dados Local.
    Retorna o número de gCO2/kWh e a string de 'fonte' (para sabermos de onde o dado veio).
    """
    # 1. Traduz as palavras humanas para o código do servidor
    zone = get_zone_code(country, state)
    
    # 2. Busca as variáveis de ambiente
    proxy_url = os.getenv("GRAVITY_PROXY_URL")     # Ex: https://seu-worker.dev
    api_key = os.getenv("ELECTRICITY_MAPS_KEY")    # Ex: token123xyz

    # 3. Lógica em Cascata: Quem nós vamos consultar?
    if proxy_url:
        # Caminho A: Existe um proxy configurado! O usuário não precisa de chave.
        url = f"{proxy_url}?zone={zone}" # Cola o zone na URL do proxy
        headers = {} # Não manda chave nenhuma, o Proxy cuida disso
        source_name = "realtime_api_proxy" # Etiqueta para o log
        
    elif api_key:
        # Caminho B: Não tem proxy, mas o usuário tem a chave secreta direta.
        url = f"https://api.electricitymap.org/v3/carbon-intensity/latest?zone={zone}"
        headers = {"auth-token": api_key} # Envia a chave pelo cabeçalho
        source_name = "realtime_api_electricity_maps"
        
    else:
        # Caminho C (O Fallback Local): O usuário não tem NADA. Nem proxy, nem chave.
        print("\n[INFO] Neither GRAVITY_PROXY_URL nor ELECTRICITY_MAPS_KEY configured.")
        print(f"Falling back to local offline database for zone: {zone}")
        # Retorna o valor fixo lido do carbon_data.json
        return FALLBACK_INTENSITIES.get(zone, DEFAULT_FALLBACK), f"offline_database_{zone}"

    # 4. TENTATIVA DE COMUNICAÇÃO COM A INTERNET
    try:
        # Usa a biblioteca requests para visitar a URL com um limite de tempo (timeout) de 5 segundos pra não travar
        response = requests.get(url, headers=headers, timeout=5)
        
        # Se a internet retornar erro (400, 404, 500), isso aqui "levanta" o erro e joga direto pro 'except' lá embaixo
        response.raise_for_status()
        
        # Converte a resposta crua da internet num Dicionário Python
        data = response.json()
        
        # Extrai só a variável de carbono de dentro do dicionário
        intensity = data.get("carbonIntensity")
        
        if intensity is not None:
            # SUCESSO! Converte pra número flutuante (decimal) e retorna
            return float(intensity), source_name
        else:
            # Deu erro de formatação na API deles
            print(f"\n[WARNING] Unexpected API response format for zone {zone}.")
            return FALLBACK_INTENSITIES.get(zone, DEFAULT_FALLBACK), f"fallback_api_error_{zone}"
            
    except requests.exceptions.RequestException as e:
        # CAIU AQUI SE: Deu timeout, não tem internet, ou deu Erro 400/500 lá no raise_for_status().
        # Avisa na tela, mas não trava o programa (Anti-Crash!)
        print(f"\n[WARNING] Failed to fetch live data: {e}")
        # Recorre silenciosamente ao banco de dados offline
        return FALLBACK_INTENSITIES.get(zone, DEFAULT_FALLBACK), f"fallback_api_error_{zone}"
