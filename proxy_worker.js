/**
 * Cloudflare Worker para Gravity Monitor Proxy
 * 
 * O que é isso? 
 * É um servidor "sem servidor" (Serverless) que roda na borda da internet.
 * Ele recebe um pedido de energia, pega a sua chave secreta armazenada no Cloudflare,
 * repassa para o Electricity Maps, e devolve a resposta.
 */

// 'export default' indica que este é o módulo principal do Worker que o Cloudflare vai executar.
export default {
  
  // 'fetch' é a função disparada toda vez que alguém acessa a URL do seu Worker.
  // Recebe o 'request' (o pedido do usuário), 'env' (as variáveis secretas), e 'ctx' (contexto extra).
  async fetch(request, env, ctx) {
    
    // VERIFICAÇÃO DE SEGURANÇA 1:
    // O Electricity Maps só usa requisições de leitura (GET). Se alguém tentar enviar dados (POST, PUT), bloqueamos.
    if (request.method !== "GET") {
      // Retorna código 405 (Method Not Allowed) informando que só aceitamos GET.
      return new Response("Method Not Allowed", { status: 405 });
    }

    // INTERPRETAÇÃO DA URL:
    // Pega a URL completa que o usuário acessou (ex: https://meu-worker.dev?zone=BR)
    const url = new URL(request.url);
    
    // Extrai o valor do parâmetro 'zone' da URL. No exemplo acima, extrairia "BR".
    const zone = url.searchParams.get("zone");

    // VERIFICAÇÃO DE SEGURANÇA 2:
    // Se o usuário não enviou a zona, não tem como consultar o mapa.
    if (!zone) {
      // Retorna erro 400 (Bad Request) em formato JSON avisando que faltou o parâmetro.
      return new Response(JSON.stringify({ error: "Missing 'zone' parameter" }), {
        status: 400, // 400 significa erro do cliente (digitou a URL errada)
        headers: { "Content-Type": "application/json" } // Avisa ao navegador que a resposta é um JSON
      });
    }

    // RECUPERANDO A CHAVE SECRETA:
    // Pega a chave da API que você salvou no painel do Cloudflare.
    // Como isso roda no servidor do Cloudflare, nenhum hacker consegue ver esse valor.
    const apiKey = env.ELECTRICITY_MAPS_KEY;

    // VERIFICAÇÃO DE SEGURANÇA 3:
    // Se você esqueceu de configurar a chave no painel do Cloudflare, o sistema avisa.
    if (!apiKey) {
      // Retorna erro 500 (Internal Server Error)
      return new Response(JSON.stringify({ error: "Server Configuration Error: API Key missing" }), {
        status: 500, // 500 significa erro do servidor (o dono do servidor esqueceu a chave)
        headers: { "Content-Type": "application/json" }
      });
    }

    // MONTANDO O PEDIDO FINAL:
    // Constrói a URL real do Electricity Maps colando a zona extraída (ex: zone=BR).
    const apiUrl = `https://api.electricitymap.org/v3/carbon-intensity/latest?zone=${zone}`;

    // TENTANDO BUSCAR OS DADOS (bloco try/catch para evitar que o servidor "crache" se der erro na rede)
    try {
      
      // 'fetch' faz a requisição pela internet para o Electricity Maps. O 'await' faz o código esperar a resposta chegar.
      const response = await fetch(apiUrl, {
        headers: {
          "auth-token": apiKey, // Aqui nós injetamos a sua chave secreta no cabeçalho do pedido!
          "Accept": "application/json" // Pedimos para o Electricity Maps responder em JSON
        }
      });

      // Transforma a resposta bruta do Electricity Maps em um objeto JSON manipulável.
      const data = await response.json();

      // RESPOSTA DE SUCESSO:
      // Devolvemos o JSON intacto para quem chamou nosso proxy (o script Python).
      return new Response(JSON.stringify(data), {
        status: response.status, // Repassa o código de status exato (200 OK, 400 Erro, etc)
        headers: {
          "Content-Type": "application/json", // Avisa o Python que estamos mandando um JSON
          // O CORS (Cross-Origin Resource Sharing) permite que qualquer origem ('*') consuma esta API.
          // Isso é ótimo caso você queira usar esse proxy em um site HTML puro depois.
          "Access-Control-Allow-Origin": "*", 
        }
      });
      
    // SE ALGO DER ERRADO NA INTERNET (ex: site fora do ar):
    } catch (error) {
      // Devolve um erro genérico 500 para não quebrar a aplicação do usuário.
      return new Response(JSON.stringify({ error: "Failed to fetch from Electricity Maps" }), {
        status: 500,
        headers: { "Content-Type": "application/json" }
      });
    }
  }, // fim da função fetch
}; // fim do módulo
