//! O protocolo MCP (JSON-RPC 2.0 por stdin/stdout).
//! Suporta `initialize`, `tools/list`, `tools/call` e `ping`. 100% local.

use serde_json::{json, Value};

use crate::agente::Agente;
use crate::esquemas;
use crate::ferramentas::chamar;
use crate::fonte::Fonte;

const VERSOES: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCOES: &str = "FeatherSlide: apresentações da suíte FeatherOffice, 100% local. COMO TRABALHAR: comece por slide_resumo para conhecer a apresentação e os slides existentes; use slide_ver para ver a imagem renderizada do slide com os rótulos de ID; manipule slides com slide_inserir, slide_duplicar, slide_excluir, slide_mover, slide_layout e slide_fundo; insira e formate elementos com slide_elemento_inserir e slide_elemento_editar (texto, coordenadas x/y/largura/altura, cores, preenchimento, contorno, fontes e alinhamentos); desfaça erros com slide_desfazer. Ao vivo: com o aplicativo aberto, as ferramentas agem diretamente na tela do usuário.";

fn resposta(id: &Value, resultado: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": resultado })
}

fn erro(id: &Value, codigo: i32, mensagem: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": codigo, "message": mensagem } })
}

fn inicializar(params: &Value) -> Value {
    let pedida = params.get("protocolVersion").and_then(Value::as_str).unwrap_or(VERSOES[0]);
    let versao = if VERSOES.contains(&pedida) { pedida } else { VERSOES[0] };
    json!({
        "protocolVersion": versao,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "featherslide", "version": env!("CARGO_PKG_VERSION") },
        "instructions": INSTRUCOES
    })
}

fn chamar_ferramenta(ag: &mut Agente, fonte: &Fonte, params: &Value) -> Value {
    let nome = params.get("name").and_then(Value::as_str).unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
    let versao_antes = ag.sessao.estado().versao;
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| chamar(ag, fonte, nome, &args))) {
        Ok(Ok(conteudo)) => json!({ "content": conteudo, "isError": false }),
        Ok(Err(e)) => json!({ "content": [{ "type": "text", "text": e }], "isError": true }),
        Err(causa) => {
            let motivo = causa.downcast_ref::<&str>().map(|s| (*s).to_string()).or_else(|| causa.downcast_ref::<String>().cloned()).unwrap_or_else(|| "motivo desconhecido".into());
            if ag.sessao.estado().versao != versao_antes {
                let _ = ag.sessao.desfazer();
            }
            let texto = format!("Erro interno na ferramenta `{nome}`: {motivo}. O estado anterior foi restaurado.");
            json!({ "content": [{ "type": "text", "text": texto }], "isError": true })
        }
    }
}

/// Atende uma mensagem JSON-RPC. Retorna `None` para notificações (sem `id`).
pub fn atender(ag: &mut Agente, fonte: &Fonte, msg: &Value) -> Option<Value> {
    let metodo = msg.get("method").and_then(Value::as_str)?;
    let id = msg.get("id")?;
    let params = msg.get("params").cloned().unwrap_or_else(|| json!({}));
    Some(match metodo {
        "initialize" => resposta(id, inicializar(&params)),
        "ping" => resposta(id, json!({})),
        "tools/list" => resposta(id, json!({ "tools": esquemas::todas() })),
        "tools/call" => resposta(id, chamar_ferramenta(ag, fonte, &params)),
        outro => erro(id, -32601, &format!("Método desconhecido: {outro}")),
    })
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializacao_e_listagem_de_ferramentas() {
        let mut ag = Agente::default();
        let fonte = Fonte::Basica;
        let r1 = atender(&mut ag, &fonte, &json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} })).unwrap();
        assert_eq!(r1["result"]["serverInfo"]["name"], "featherslide");

        let r2 = atender(&mut ag, &fonte, &json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" })).unwrap();
        let tools = r2["result"]["tools"].as_array().unwrap();
        assert!(tools.iter().any(|t| t["name"] == "slide_resumo"));
        assert!(tools.iter().any(|t| t["name"] == "slide_ver"));
        assert!(tools.iter().any(|t| t["name"] == "slide_inserir"));
    }

    #[test]
    fn panico_capturado_com_desfazer() {
        let mut ag = Agente::default();
        let fonte = Fonte::Basica;
        let r = atender(&mut ag, &fonte, &json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": { "name": "slide_inexistente", "arguments": {} } })).unwrap();
        assert_eq!(r["result"]["isError"], true);
    }
}
