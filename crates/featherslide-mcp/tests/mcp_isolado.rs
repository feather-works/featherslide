//! Testes integrados do servidor MCP do FeatherSlide em modo isolado (stdio / chamadas diretas).

use featherslide_mcp::agente::Agente;
use featherslide_mcp::fonte::Fonte;
use featherslide_mcp::rpc;
use serde_json::{json, Value};

fn chamar(ag: &mut Agente, fonte: &Fonte, id: i64, ferramenta: &str, args: Value) -> Value {
    let msg = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {
            "name": ferramenta,
            "arguments": args
        }
    });
    rpc::atender(ag, fonte, &msg).expect("Resposta esperada para tools/call")
}

#[test]
fn ciclo_de_edicao_e_arquivos_no_mcp() {
    let mut ag = Agente::default();
    let fonte = Fonte::Basica;

    // 1. Nova apresentação
    let r_novo = chamar(&mut ag, &fonte, 1, "slide_novo", json!({ "proporcao": "16:9" }));
    assert_eq!(r_novo["result"]["isError"], false);

    // 2. Resumo da apresentação
    let r_resumo = chamar(&mut ag, &fonte, 2, "slide_resumo", json!({}));
    assert_eq!(r_resumo["result"]["isError"], false);
    let texto_resumo = r_resumo["result"]["content"][0]["text"].as_str().unwrap();
    assert!(texto_resumo.contains("total_slides"));

    // 3. Inserir slide
    let r_ins = chamar(&mut ag, &fonte, 3, "slide_inserir", json!({ "layout": "duas_colunas" }));
    assert_eq!(r_ins["result"]["isError"], false);

    // 4. Inserir elemento (retângulo com texto)
    let r_elem = chamar(&mut ag, &fonte, 4, "slide_elemento_inserir", json!({
        "tipo": "retangulo",
        "x": 100.0,
        "y": 150.0,
        "largura": 300.0,
        "altura": 120.0,
        "texto": "Bloco de Teste MCP",
        "preenchimento": "#4285f4",
        "contorno": "#1a56c4",
        "espessura": 2.0
    }));
    assert_eq!(r_elem["result"]["isError"], false);
    let id_elem: u64 = serde_json::from_str::<Value>(r_elem["result"]["content"][0]["text"].as_str().unwrap()).unwrap()["id"].as_u64().unwrap();

    // 5. Editar elemento
    let r_edit = chamar(&mut ag, &fonte, 5, "slide_elemento_editar", json!({
        "id": id_elem,
        "texto": "Texto Atualizado pela IA",
        "largura": 350.0
    }));
    assert_eq!(r_edit["result"]["isError"], false);

    // 6. Visualizar slide gerando imagem PNG
    let r_ver = chamar(&mut ag, &fonte, 6, "slide_ver", json!({ "largura_px": 640 }));
    assert_eq!(r_ver["result"]["isError"], false);
    let conteudo = r_ver["result"]["content"].as_array().unwrap();
    assert_eq!(conteudo[0]["type"], "image");
    assert_eq!(conteudo[0]["mimeType"], "image/png");
    let b64 = conteudo[0]["data"].as_str().unwrap();
    assert!(!b64.is_empty());

    // 7. Salvar e reabrir
    let pasta = std::env::temp_dir().join(format!("teste_mcp_fsdz_{}", std::process::id()));
    std::fs::create_dir_all(&pasta).unwrap();
    let arq = pasta.join("apresentacao_teste.fsdz");
    let r_salvar = chamar(&mut ag, &fonte, 7, "slide_salvar", json!({
        "caminho": arq.to_str().unwrap(),
        "sobrescrever": true
    }));
    assert_eq!(r_salvar["result"]["isError"], false);
    assert!(arq.exists());

    let r_abrir = chamar(&mut ag, &fonte, 8, "slide_abrir", json!({
        "caminho": arq.to_str().unwrap()
    }));
    assert_eq!(r_abrir["result"]["isError"], false);

    // 8. Desfazer
    let r_desfazer = chamar(&mut ag, &fonte, 9, "slide_desfazer", json!({}));
    assert_eq!(r_desfazer["result"]["isError"], false);

    let _ = std::fs::remove_dir_all(&pasta);
}
