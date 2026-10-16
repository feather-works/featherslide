//! Esquemas das ferramentas do MCP (JSON Schema).
//! Toda ferramenta usa o prefixo `slide_` e nomes descritivos em português.

use serde_json::{json, Value};

fn ferramenta(nome: &'static str, descricao: &'static str, props: Value, obrigatorios: &[&str]) -> Value {
    json!({
        "name": nome,
        "description": descricao,
        "inputSchema": {
            "type": "object",
            "properties": props,
            "required": obrigatorios,
        }
    })
}

fn prop_str(desc: &'static str) -> Value {
    json!({ "type": "string", "description": desc })
}

fn prop_num(desc: &'static str) -> Value {
    json!({ "type": "number", "description": desc })
}

fn prop_int(desc: &'static str) -> Value {
    json!({ "type": "integer", "description": desc })
}

fn prop_bool(desc: &'static str) -> Value {
    json!({ "type": "boolean", "description": desc })
}

pub fn todas() -> Vec<Value> {
    vec![
        ferramenta("slide_abrir", "Abre uma apresentação salva em arquivo .fsdz.", json!({ "caminho": prop_str("Caminho do arquivo .fsdz no disco") }), &["caminho"]),
        ferramenta("slide_novo", "Cria uma nova apresentação em branco (ou com slide inicial).", json!({ "proporcao": prop_str("Proporção da tela: '16:9' ou '4:3' (padrão: '16:9')") }), &[]),
        ferramenta("slide_salvar", "Salva a apresentação atual em arquivo .fsdz.", json!({ "caminho": prop_str("Caminho para salvar o arquivo .fsdz"), "sobrescrever": prop_bool("Se true, permite sobrescrever arquivo existente") }), &[]),
        ferramenta("slide_importar", "Importa uma apresentação de arquivo PowerPoint (.pptx) ou LibreOffice (.odp).", json!({ "caminho": prop_str("Caminho do arquivo .pptx ou .odp") }), &["caminho"]),
        ferramenta("slide_resumo", "Retorna o resumo da apresentação inteira ou os detalhes completos de um slide específico.", json!({ "slide": prop_int("Número do slide (1-based). Se omitido, resume a apresentação inteira.") }), &[]),
        ferramenta("slide_ver", "Gera uma imagem PNG do slide para visualização pela IA.", json!({ "slide": prop_int("Número do slide (1-based, padrão: atual)"), "largura_px": prop_int("Largura da imagem em pixels (padrão: 1000)"), "rotulos": prop_bool("Se true, mostra os IDs (#id) de cada elemento para facilitar edições") }), &[]),
        ferramenta("slide_inserir", "Insere um novo slide na apresentação.", json!({ "layout": prop_str("Nome do layout: 'titulo', 'titulo_conteudo', 'secao', 'duas_colunas', 'comparacao', 'so_titulo', 'em_branco'"), "depois_de": prop_int("Posição (1-based) após a qual inserir o slide") }), &[]),
        ferramenta("slide_duplicar", "Cria uma cópia do slide atual ou do slide especificado.", json!({ "slide": prop_int("Número do slide (1-based, padrão: atual)") }), &[]),
        ferramenta("slide_excluir", "Exclui um slide da apresentação.", json!({ "slide": prop_int("Número do slide (1-based, padrão: atual)") }), &[]),
        ferramenta("slide_mover", "Move um slide de uma posição para outra.", json!({ "de": prop_int("Posição de origem (1-based)"), "para": prop_int("Posição de destino (1-based)") }), &["de", "para"]),
        ferramenta("slide_layout", "Altera o layout do slide.", json!({ "layout": prop_str("Novo layout: 'titulo', 'titulo_conteudo', 'secao', 'duas_colunas', 'comparacao', 'so_titulo', 'em_branco'"), "slide": prop_int("Número do slide (1-based, padrão: atual)") }), &["layout"]),
        ferramenta("slide_fundo", "Define a cor de fundo do slide.", json!({ "cor": prop_str("Cor hexadecimal (ex: '#ffffff' ou '#1e1e1e')"), "slide": prop_int("Número do slide (1-based, padrão: atual)"), "todos": prop_bool("Se true, aplica a todos os slides") }), &["cor"]),
        ferramenta("slide_notas", "Define as anotações do apresentador do slide.", json!({ "notas": prop_str("Texto das notas"), "slide": prop_int("Número do slide (1-based, padrão: atual)") }), &["notas"]),
        ferramenta("slide_elemento_inserir", "Insere um novo elemento no slide (texto, retângulo, elipse, linha, imagem).", json!({
            "tipo": prop_str("Tipo do elemento: 'texto', 'retangulo', 'elipse', 'linha', 'imagem', 'setor'"),
            "slide": prop_int("Número do slide (1-based, padrão: atual)"),
            "x": prop_num("Coordenada X em pontos"),
            "y": prop_num("Coordenada Y em pontos"),
            "largura": prop_num("Largura em pontos"),
            "altura": prop_num("Altura em pontos"),
            "texto": prop_str("Texto a inserir (para caixas de texto e formas)"),
            "preenchimento": prop_str("Cor de preenchimento (ex: '#4285f4')"),
            "contorno": prop_str("Cor do contorno"),
            "espessura": prop_num("Espessura da linha/contorno em pontos"),
            "imagem": prop_str("Dados da imagem (para tipo 'imagem')"),
            "fonte": prop_str("Família da fonte"),
            "tamanho": prop_num("Tamanho da fonte em pontos"),
            "cor": prop_str("Cor do texto"),
            "negrito": prop_bool("Se true, aplica negrito"),
            "italico": prop_bool("Se true, aplica itálico"),
            "sublinhado": prop_bool("Se true, aplica sublinhado")
        }), &["tipo"]),
        ferramenta("slide_elemento_editar", "Modifica propriedades de um elemento existente no slide.", json!({
            "id": prop_int("ID do elemento a editar"),
            "slide": prop_int("Número do slide (1-based, padrão: onde estiver o elemento)"),
            "x": prop_num("Nova coordenada X"),
            "y": prop_num("Nova coordenada Y"),
            "largura": prop_num("Nova largura"),
            "altura": prop_num("Nova altura"),
            "rotacao": prop_num("Rotação em graus"),
            "texto": prop_str("Novo texto"),
            "preenchimento": prop_str("Cor de preenchimento"),
            "contorno": prop_str("Cor do contorno"),
            "espessura": prop_num("Espessura do contorno"),
            "fonte": prop_str("Família da fonte"),
            "tamanho": prop_num("Tamanho da fonte em pontos"),
            "cor": prop_str("Cor do texto"),
            "negrito": prop_bool("Negrito"),
            "italico": prop_bool("Itálico"),
            "sublinhado": prop_bool("Sublinhado"),
            "alinhamento": prop_str("Alinhamento horizontal: 'esquerda', 'centro', 'direita'"),
            "vertical": prop_str("Alinhamento vertical: 'topo', 'meio', 'base'")
        }), &["id"]),
        ferramenta("slide_elemento_excluir", "Exclui um elemento pelo ID.", json!({ "id": prop_int("ID do elemento a excluir"), "slide": prop_int("Número do slide (1-based)") }), &["id"]),
        ferramenta("slide_desfazer", "Desfaz a última edição na apresentação.", json!({}), &[]),
    ]
}
