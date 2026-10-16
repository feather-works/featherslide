//! Implementação e despacho das ferramentas do servidor MCP.
//! Toda ferramenta chama diretamente os métodos do `featherslide-core` e `featherslide-io`.

use std::path::PathBuf;

use base64::Engine;
use featherslide_core::modelo::{Alinhamento, Apresentacao, Proporcao, Tipo, Vertical};
use featherslide_core::sessao::Sessao;
use featherslide_core::Formatacao;
use serde_json::{json, Value};

use crate::agente::Agente;
use crate::fonte::Fonte;
use crate::renderizar::renderizar_png;

pub type Conteudo = Vec<Value>;

fn texto(v: &Value) -> Conteudo {
    vec![json!({ "type": "text", "text": serde_json::to_string_pretty(v).unwrap_or_default() })]
}

fn navegar_slide(s: &mut Sessao, opt_slide: Option<usize>) -> Result<(), String> {
    if let Some(num) = opt_slide {
        if num == 0 {
            return Err("O número do slide deve ser 1 ou maior.".into());
        }
        s.slide_ir(num - 1)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Ferramentas de Arquivo

fn slide_abrir(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    let caminho = args.get("caminho").and_then(Value::as_str).ok_or("Faltou o argumento `caminho`.")?;
    let bytes = std::fs::read(caminho).map_err(|e| format!("Não foi possível ler \"{caminho}\": {e}"))?;
    let ap = featherslide_io::fsdz::ler(&bytes)?;
    ag.sessao = Sessao::nova(ap);
    ag.arquivo = Some(PathBuf::from(caminho));
    let est = ag.sessao.estado();
    Ok(texto(&json!({
        "ok": true,
        "arquivo": caminho,
        "titulo": est.titulo,
        "total_slides": est.slides.len(),
        "proporcao": est.proporcao,
    })))
}

fn slide_novo(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    let prop = match args.get("proporcao").and_then(Value::as_str) {
        Some("4:3") => Proporcao::Padrao,
        _ => Proporcao::Larga,
    };
    let ap = Apresentacao {
        titulo: String::new(),
        autor: String::new(),
        proporcao: prop,
        dimensoes: [0.0; 2],
        slides: Vec::new(),
        proximo_id: 1,
        fontes: Vec::new(),
        tema: String::new(),
        modelo: String::new(),
        metadados: featherslide_core::metadados::Metadados::novo(),
    };
    ag.sessao = Sessao::nova(ap);
    ag.arquivo = None;
    let est = ag.sessao.estado();
    Ok(texto(&json!({
        "ok": true,
        "total_slides": est.slides.len(),
        "proporcao": est.proporcao,
    })))
}

fn slide_salvar(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    let caminho = args
        .get("caminho")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .or_else(|| ag.arquivo.clone())
        .ok_or("Informe o caminho para salvar o arquivo .fsdz.")?;
    let sobrescrever = args.get("sobrescrever").and_then(Value::as_bool).unwrap_or(false);
    if caminho.exists() && !sobrescrever && ag.arquivo.as_deref() != Some(caminho.as_path()) {
        return Err(format!("O arquivo \"{}\" já existe. Use sobrescrever: true para substituir.", caminho.display()));
    }
    if let Some(pai) = caminho.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(pai).map_err(|e| format!("Não foi possível criar a pasta: {e}"))?;
    }
    let tamanho = featherslide_io::fsdz::salvar(ag.sessao.apresentacao(), &caminho)?;
    ag.sessao.marcar_gravado();
    ag.arquivo = Some(caminho.clone());
    Ok(texto(&json!({ "ok": true, "arquivo": caminho.display().to_string(), "bytes": tamanho })))
}

fn slide_importar(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    let caminho = args.get("caminho").and_then(Value::as_str).ok_or("Faltou o argumento `caminho`.")?;
    let bytes = std::fs::read(caminho).map_err(|e| format!("Não foi possível ler \"{caminho}\": {e}"))?;
    let ap = if caminho.to_lowercase().ends_with(".pptx") {
        featherslide_io::pptx::ler(&bytes)?
    } else if caminho.to_lowercase().ends_with(".odp") {
        featherslide_io::odp::ler(&bytes)?
    } else {
        return Err("Extensão não suportada. Use um arquivo .pptx ou .odp.".into());
    };
    ag.sessao = Sessao::nova(ap);
    ag.arquivo = None;
    let est = ag.sessao.estado();
    Ok(texto(&json!({
        "ok": true,
        "importado_de": caminho,
        "titulo": est.titulo,
        "total_slides": est.slides.len(),
        "proporcao": est.proporcao,
    })))
}

// ------------------------------------------------------------------ Consulta e Visualização

fn slide_resumo(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    let ap = ag.sessao.apresentacao();
    if let Some(num) = args.get("slide").and_then(Value::as_u64) {
        let i = (num as usize).checked_sub(1).ok_or("O número do slide deve ser 1 ou maior.")?;
        let s = ap.slides.get(i).ok_or_else(|| format!("Slide {num} não existe."))?;
        return Ok(texto(&json!({
            "slide": num,
            "id": s.id,
            "layout": s.layout,
            "fundo": s.fundo,
            "oculto": s.oculto,
            "notas": s.notas,
            "total_elementos": s.elementos.len(),
            "elementos": s.elementos.iter().map(|e| json!({
                "id": e.id,
                "tipo": e.tipo.nome(),
                "x": e.x,
                "y": e.y,
                "largura": e.largura,
                "altura": e.altura,
                "rotacao": e.rotacao,
                "texto": e.texto,
                "preenchimento": e.preenchimento,
                "contorno": e.contorno,
                "espessura": e.espessura,
                "estilo": {
                    "fonte": e.estilo.fonte,
                    "tamanho": e.estilo.tamanho,
                    "cor": e.estilo.cor,
                    "negrito": e.estilo.negrito,
                    "italico": e.estilo.italico,
                    "sublinhado": e.estilo.sublinhado,
                }
            })).collect::<Vec<_>>(),
        })));
    }

    let est = ag.sessao.estado();
    let lista_slides: Vec<Value> = ap.slides.iter().enumerate().map(|(idx, s)| {
        let preview = s.elementos.iter().find(|e| !e.texto.is_empty()).map(|e| e.texto.as_str()).unwrap_or("");
        json!({
            "slide": idx + 1,
            "id": s.id,
            "layout": s.layout,
            "elementos": s.elementos.len(),
            "texto_principal": preview,
            "tem_notas": !s.notas.is_empty(),
            "oculto": s.oculto,
        })
    }).collect();

    Ok(texto(&json!({
        "arquivo": ag.arquivo.as_ref().map(|p| p.display().to_string()),
        "titulo": est.titulo,
        "proporcao": est.proporcao,
        "total_slides": est.slides.len(),
        "slide_atual": est.atual + 1,
        "sujo": est.sujo,
        "slides": lista_slides,
        "layouts_disponiveis": featherslide_core::layouts::LAYOUTS,
    })))
}

fn slide_ver(ag: &mut Agente, fonte: &Fonte, args: &Value) -> Result<Conteudo, String> {
    if let Some(num) = args.get("slide").and_then(Value::as_u64) {
        navegar_slide(&mut ag.sessao, Some(num as usize))?;
    }
    let i = ag.sessao.atual();
    let quadro = ag.sessao.quadro(i, None, false)?.ok_or("Não foi possível gerar quadro do slide.")?;
    let larg = args.get("largura_px").and_then(Value::as_u64).unwrap_or(1000) as u32;
    let rotulos = args.get("rotulos").and_then(Value::as_bool).unwrap_or(true);
    let bytes_png = renderizar_png(&quadro, larg, rotulos, fonte)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes_png);
    let legenda = format!("Slide {}/{} ({}×{} px). {}", i + 1, ag.sessao.apresentacao().slides.len(), larg, ((quadro.altura * (larg as f32 / quadro.largura)).round() as u32), if rotulos { "Rótulos #id indicam o ID de cada elemento." } else { "" });

    Ok(vec![
        json!({
            "type": "image",
            "data": b64,
            "mimeType": "image/png"
        }),
        json!({
            "type": "text",
            "text": legenda
        }),
    ])
}

// ------------------------------------------------------------------ Manipulação de Slides

fn slide_inserir(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    if let Some(depois) = args.get("depois_de").and_then(Value::as_u64) {
        navegar_slide(&mut ag.sessao, Some(depois as usize))?;
    }
    let layout = args.get("layout").and_then(Value::as_str).unwrap_or("titulo_conteudo");
    let mud = ag.sessao.slide_novo(layout)?;
    let atual = ag.sessao.atual() + 1;
    Ok(texto(&json!({ "ok": true, "slide": atual, "layout": layout, "versao": mud.versao })))
}

fn slide_duplicar(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    let opt = args.get("slide").and_then(Value::as_u64).map(|n| n as usize);
    navegar_slide(&mut ag.sessao, opt)?;
    let mud = ag.sessao.slide_duplicar()?;
    let atual = ag.sessao.atual() + 1;
    Ok(texto(&json!({ "ok": true, "slide_duplicado": atual, "versao": mud.versao })))
}

fn slide_excluir(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    let opt = args.get("slide").and_then(Value::as_u64).map(|n| n as usize);
    navegar_slide(&mut ag.sessao, opt)?;
    let mud = ag.sessao.slide_excluir()?;
    let total = ag.sessao.apresentacao().slides.len();
    Ok(texto(&json!({ "ok": true, "total_restante": total, "versao": mud.versao })))
}

fn slide_mover(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    let de = args.get("de").and_then(Value::as_u64).ok_or("Faltou o argumento `de`.")? as usize;
    let para = args.get("para").and_then(Value::as_u64).ok_or("Faltou o argumento `para`.")? as usize;
    if de == 0 || para == 0 {
        return Err("Os índices 'de' e 'para' devem ser 1 ou maiores.".into());
    }
    let mud = ag.sessao.slide_mover(de - 1, para - 1)?;
    Ok(texto(&json!({ "ok": true, "de": de, "para": para, "versao": mud.versao })))
}

fn slide_layout(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    let opt = args.get("slide").and_then(Value::as_u64).map(|n| n as usize);
    navegar_slide(&mut ag.sessao, opt)?;
    let layout = args.get("layout").and_then(Value::as_str).ok_or("Faltou o argumento `layout`.")?;
    let mud = ag.sessao.slide_layout(layout)?;
    Ok(texto(&json!({ "ok": true, "slide": ag.sessao.atual() + 1, "layout": layout, "versao": mud.versao })))
}

fn slide_fundo(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    let opt = args.get("slide").and_then(Value::as_u64).map(|n| n as usize);
    navegar_slide(&mut ag.sessao, opt)?;
    let cor = args.get("cor").and_then(Value::as_str).ok_or("Faltou o argumento `cor`.")?;
    let todos = args.get("todos").and_then(Value::as_bool).unwrap_or(false);
    let mud = ag.sessao.slide_fundo(cor, todos)?;
    Ok(texto(&json!({ "ok": true, "cor": cor, "todos": todos, "versao": mud.versao })))
}

fn slide_notas(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    let opt = args.get("slide").and_then(Value::as_u64).map(|n| n as usize);
    navegar_slide(&mut ag.sessao, opt)?;
    let notas = args.get("notas").and_then(Value::as_str).ok_or("Faltou o argumento `notas`.")?;
    let mud = ag.sessao.notas_definir(notas)?;
    Ok(texto(&json!({ "ok": true, "slide": ag.sessao.atual() + 1, "versao": mud.versao })))
}

// ------------------------------------------------------------------ Elementos

fn aplicar_formatos_extras(ag: &mut Agente, args: &Value) {
    if let Some(f) = args.get("fonte").and_then(Value::as_str) {
        let _ = ag.sessao.formatar(Formatacao::Fonte(f.into()));
    }
    if let Some(t) = args.get("tamanho").and_then(Value::as_f64) {
        let _ = ag.sessao.formatar(Formatacao::Tamanho(t as f32));
    }
    if let Some(c) = args.get("cor").and_then(Value::as_str) {
        let _ = ag.sessao.formatar(Formatacao::CorDoTexto(c.into()));
    }
    if let Some(true) = args.get("negrito").and_then(Value::as_bool) {
        let _ = ag.sessao.formatar(Formatacao::Negrito);
    }
    if let Some(true) = args.get("italico").and_then(Value::as_bool) {
        let _ = ag.sessao.formatar(Formatacao::Italico);
    }
    if let Some(true) = args.get("sublinhado").and_then(Value::as_bool) {
        let _ = ag.sessao.formatar(Formatacao::Sublinhado);
    }
    if let Some(p) = args.get("preenchimento").and_then(Value::as_str) {
        let _ = ag.sessao.formatar(Formatacao::Preenchimento(p.into()));
    }
    if let Some(c) = args.get("contorno").and_then(Value::as_str) {
        let _ = ag.sessao.formatar(Formatacao::Contorno(c.into()));
    }
    if let Some(e) = args.get("espessura").and_then(Value::as_f64) {
        let _ = ag.sessao.formatar(Formatacao::Espessura(e as f32));
    }
    if let Some(a) = args.get("alinhamento").and_then(Value::as_str) {
        let al = match a {
            "centro" => Some(Alinhamento::Centro),
            "direita" => Some(Alinhamento::Direita),
            "esquerda" => Some(Alinhamento::Esquerda),
            _ => None,
        };
        if let Some(al) = al {
            let _ = ag.sessao.formatar(Formatacao::Alinhar(al));
        }
    }
    if let Some(v) = args.get("vertical").and_then(Value::as_str) {
        let vt = match v {
            "topo" => Some(Vertical::Topo),
            "base" => Some(Vertical::Base),
            "meio" => Some(Vertical::Meio),
            _ => None,
        };
        if let Some(vt) = vt {
            let _ = ag.sessao.formatar(Formatacao::Vertical(vt));
        }
    }
}

fn slide_elemento_inserir(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    let opt = args.get("slide").and_then(Value::as_u64).map(|n| n as usize);
    navegar_slide(&mut ag.sessao, opt)?;

    let nome_tipo = args.get("tipo").and_then(Value::as_str).ok_or("Faltou o argumento `tipo`.")?;
    let tipo = Tipo::pelo_nome(nome_tipo).ok_or_else(|| format!("Tipo desconhecido: '{nome_tipo}'"))?;

    let rect = match (args.get("x").and_then(Value::as_f64), args.get("y").and_then(Value::as_f64), args.get("largura").and_then(Value::as_f64), args.get("altura").and_then(Value::as_f64)) {
        (Some(x), Some(y), Some(w), Some(h)) => Some([x as f32, y as f32, w as f32, h as f32]),
        _ => None,
    };
    let img = args.get("imagem").and_then(Value::as_str).map(String::from);

    ag.sessao.elemento_inserir(tipo, rect, img)?;
    let id = *ag.sessao.estado().selecao.first().ok_or("Erro ao obter elemento inserido.")?;

    if let Some(txt) = args.get("texto").and_then(Value::as_str) {
        let _ = ag.sessao.texto_definir(id, txt);
    }
    aplicar_formatos_extras(ag, args);

    Ok(texto(&json!({
        "ok": true,
        "id": id,
        "slide": ag.sessao.atual() + 1,
        "tipo": nome_tipo,
    })))
}

fn slide_elemento_editar(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    let id = args.get("id").and_then(Value::as_u64).ok_or("Faltou o argumento `id`.")?;
    let opt = args.get("slide").and_then(Value::as_u64).map(|n| n as usize);
    navegar_slide(&mut ag.sessao, opt)?;

    if !ag.sessao.selecionar_por_id(id) {
        let mut achou = false;
        for (idx, sl) in ag.sessao.apresentacao().slides.iter().enumerate() {
            if sl.elementos.iter().any(|e| e.id == id) {
                ag.sessao.slide_ir(idx)?;
                ag.sessao.selecionar_por_id(id);
                achou = true;
                break;
            }
        }
        if !achou {
            return Err(format!("Elemento #{id} não encontrado na apresentação."));
        }
    }

    let elem_atual = ag.sessao.slide().elementos.iter().find(|e| e.id == id).cloned().ok_or("Elemento não encontrado no slide.")?;
    let novo_x = args.get("x").and_then(Value::as_f64).map(|v| v as f32).unwrap_or(elem_atual.x);
    let novo_y = args.get("y").and_then(Value::as_f64).map(|v| v as f32).unwrap_or(elem_atual.y);
    let novo_w = args.get("largura").and_then(Value::as_f64).map(|v| v as f32).unwrap_or(elem_atual.largura);
    let novo_h = args.get("altura").and_then(Value::as_f64).map(|v| v as f32).unwrap_or(elem_atual.altura);

    if (novo_x != elem_atual.x) || (novo_y != elem_atual.y) || (novo_w != elem_atual.largura) || (novo_h != elem_atual.altura) {
        ag.sessao.elemento_geometria(id, [novo_x, novo_y, novo_w, novo_h])?;
    }
    if let Some(rot) = args.get("rotacao").and_then(Value::as_f64) {
        ag.sessao.elemento_rotacao(id, rot as f32)?;
    }
    if let Some(txt) = args.get("texto").and_then(Value::as_str) {
        ag.sessao.texto_definir(id, txt)?;
    }
    aplicar_formatos_extras(ag, args);

    Ok(texto(&json!({ "ok": true, "id": id, "slide": ag.sessao.atual() + 1 })))
}

fn slide_elemento_excluir(ag: &mut Agente, args: &Value) -> Result<Conteudo, String> {
    let id = args.get("id").and_then(Value::as_u64).ok_or("Faltou o argumento `id`.")?;
    let opt = args.get("slide").and_then(Value::as_u64).map(|n| n as usize);
    navegar_slide(&mut ag.sessao, opt)?;

    if !ag.sessao.selecionar_por_id(id) {
        return Err(format!("Elemento #{id} não está no slide atual."));
    }
    let mud = ag.sessao.selecao_excluir()?;
    Ok(texto(&json!({ "ok": true, "excluido": id, "versao": mud.versao })))
}

fn slide_desfazer(ag: &mut Agente) -> Result<Conteudo, String> {
    match ag.sessao.desfazer() {
        Some(mud) => Ok(texto(&json!({ "ok": true, "desfeito": true, "versao": mud.versao }))),
        None => Ok(texto(&json!({ "ok": true, "desfeito": false, "mensagem": "Nada a desfazer." }))),
    }
}

// ------------------------------------------------------------------ Despacho Geral

pub fn chamar(ag: &mut Agente, fonte: &Fonte, nome: &str, args: &Value) -> Result<Conteudo, String> {
    match nome {
        "slide_abrir" => slide_abrir(ag, args),
        "slide_novo" => slide_novo(ag, args),
        "slide_salvar" => slide_salvar(ag, args),
        "slide_importar" => slide_importar(ag, args),
        "slide_resumo" => slide_resumo(ag, args),
        "slide_ver" => slide_ver(ag, fonte, args),
        "slide_inserir" => slide_inserir(ag, args),
        "slide_duplicar" => slide_duplicar(ag, args),
        "slide_excluir" => slide_excluir(ag, args),
        "slide_mover" => slide_mover(ag, args),
        "slide_layout" => slide_layout(ag, args),
        "slide_fundo" => slide_fundo(ag, args),
        "slide_notas" => slide_notas(ag, args),
        "slide_elemento_inserir" => slide_elemento_inserir(ag, args),
        "slide_elemento_editar" => slide_elemento_editar(ag, args),
        "slide_elemento_excluir" => slide_elemento_excluir(ag, args),
        "slide_desfazer" => slide_desfazer(ag),
        outro => Err(format!("Ferramenta desconhecida: `{outro}`")),
    }
}
