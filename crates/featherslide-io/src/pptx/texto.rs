//! O texto de uma forma do `.pptx` e o estilo dele, com a herança do PowerPoint: a letra do
//! trecho → o estilo da própria forma → o da caixa equivalente no layout → no mestre → os
//! estilos de texto do mestre (`p:txStyles`). O que nenhum nível diz fica com o padrão.
//!
//! O FeatherSlide guarda **um** estilo por caixa: vale o predominante (o que cobre mais letras). Os
//! marcadores viram o caractere "•" no começo do parágrafo (o motor ainda não tem lista).

use featherslide_core::modelo::{Alinhamento, EstiloDoTexto, Vertical};

use super::cores::Tema;
use crate::xml::No;

/// Os níveis de onde o estilo de uma forma pode vir, do mais próximo ao mais geral.
pub struct Heranca<'a> {
    /// O `p:txBody` da própria forma.
    pub proprio: Option<&'a No>,
    /// O `p:txBody` da caixa equivalente no layout e no mestre.
    pub layout: Option<&'a No>,
    pub mestre: Option<&'a No>,
    /// O estilo do mestre para este tipo de caixa (`p:titleStyle`, `p:bodyStyle`, `p:otherStyle`).
    pub estilo_mestre: Option<&'a No>,
}

impl<'a> Heranca<'a> {
    /// Os `a:lvlNpPr` do nível `nivel` (0 = primeiro), do mais próximo ao mais geral.
    fn niveis(&self, nivel: usize) -> Vec<&'a No> {
        let nome = format!("a:lvl{}pPr", nivel + 1);
        [self.proprio, self.layout, self.mestre]
            .into_iter()
            .flatten()
            .filter_map(|t| t.filho("a:lstStyle")?.filho(&nome))
            .chain(self.estilo_mestre.and_then(|e| e.filho(&nome)))
            .collect()
    }

    /// O `a:bodyPr` de cada nível.
    fn corpos(&self) -> Vec<&'a No> {
        [self.proprio, self.layout, self.mestre].into_iter().flatten().filter_map(|t| t.filho("a:bodyPr")).collect()
    }
}

/// O nível (0, 1…) de um parágrafo.
fn nivel_de(p: &No) -> usize {
    p.filho("a:pPr").and_then(|pp| pp.num("lvl")).map_or(0, |n| n.max(0.0) as usize).min(8)
}

/// O marcador do parágrafo (`Some` com o caractere), olhando o próprio `a:pPr` e depois os níveis.
fn marcador(p: &No, niveis: &[&No]) -> Option<String> {
    let proprio = p.filho("a:pPr");
    for pp in proprio.into_iter().chain(niveis.iter().copied()) {
        if pp.filho("a:buNone").is_some() {
            return None;
        }
        if let Some(c) = pp.filho("a:buChar") {
            return c.attr("char").map(|_| "•".to_string());
        }
        if pp.filho("a:buAutoNum").is_some() {
            return Some("•".to_string());
        }
    }
    None
}

/// O texto de um parágrafo: trechos, campos e quebras de linha.
fn texto_do_paragrafo(p: &No) -> String {
    let mut s = String::new();
    for f in &p.filhos {
        match f.nome.as_str() {
            "a:r" | "a:fld" => s.push_str(f.filho("a:t").map_or("", |t| t.texto.as_str())),
            "a:br" => s.push('\n'),
            _ => {}
        }
    }
    s
}

/// O texto inteiro da caixa, um parágrafo por linha, com os marcadores.
pub fn texto(corpo: &No, h: &Heranca) -> String {
    let mut linhas: Vec<String> = corpo
        .filhos("a:p")
        .map(|p| {
            let t = texto_do_paragrafo(p);
            match marcador(p, &h.niveis(nivel_de(p))) {
                Some(m) if !t.is_empty() => format!("{}{m} {t}", "    ".repeat(nivel_de(p))),
                _ => t,
            }
        })
        .collect();
    while linhas.last().is_some_and(|l| l.trim().is_empty()) {
        linhas.pop();
    }
    linhas.join("\n")
}

/// O primeiro parágrafo com texto (é o alinhamento dele que vale para a caixa).
fn primeiro_paragrafo(corpo: &No) -> Option<&No> {
    corpo.filhos("a:p").find(|p| !texto_do_paragrafo(p).trim().is_empty()).or_else(|| corpo.filho("a:p"))
}

/// O estilo de um trecho (`a:rPr`, ou nenhum) no nível dado, com a herança resolvida.
fn estilo_do_trecho(trecho: Option<&No>, niveis: &[&No], h: &Heranca, tema: &Tema, escala: f32) -> EstiloDoTexto {
    let fontes: Vec<&No> = trecho.into_iter().chain(niveis.iter().filter_map(|n| n.filho("a:defRPr"))).collect();
    let achar = |f: &dyn Fn(&No) -> Option<String>| fontes.iter().find_map(|n| f(n));
    let padrao = EstiloDoTexto::default();
    let tamanho = achar(&|n| n.attr("sz").map(String::from)).and_then(|s| s.parse::<f32>().ok()).map_or(18.0, |s| s / 100.0);
    let fator = h.corpos().first().and_then(|b| b.filho("a:normAutofit")).and_then(|a| a.num("fontScale")).map_or(1.0, |f| f as f32 / 100_000.0);
    let liga = |nome: &str| achar(&|n| n.attr(nome).map(String::from)).is_some_and(|v| v == "1" || v == "true");
    EstiloDoTexto {
        fonte: achar(&|n| n.filho("a:latin").and_then(|l| l.attr("typeface")).map(|f| tema.fonte(f))).filter(|f| !f.is_empty()).unwrap_or(padrao.fonte),
        tamanho: (tamanho * fator * escala * 10.0).round() / 10.0,
        cor: achar(&|n| n.filho("a:solidFill").and_then(|s| tema.cor(s))).unwrap_or_else(|| tema.cor_do_texto()),
        negrito: liga("b"),
        italico: liga("i"),
        sublinhado: achar(&|n| n.attr("u").map(String::from)).is_some_and(|u| u != "none"),
        sublinhado_duplo: achar(&|n| n.attr("u").map(String::from)).is_some_and(|u| u.starts_with("dbl")),
        tachado: achar(&|n| n.attr("strike").map(String::from)).is_some_and(|s| s != "noStrike"),
        tachado_duplo: achar(&|n| n.attr("strike").map(String::from)).is_some_and(|s| s == "dblStrike"),
        ..padrao
    }
}

/// O estilo da caixa, já com a herança resolvida: o **predominante** (o que cobre mais letras),
/// porque a caixa guarda um estilo só — um rótulo em negrito no começo não deixa tudo em negrito.
/// `escala` converte pontos do `.pptx` para os do slide.
pub fn estilo(corpo: Option<&No>, h: &Heranca, tema: &Tema, escala: f32) -> EstiloDoTexto {
    let mut contagem: Vec<(EstiloDoTexto, usize)> = Vec::new();
    for p in corpo.into_iter().flat_map(|c| c.filhos("a:p")) {
        let niveis = h.niveis(nivel_de(p));
        for r in p.filhos("a:r") {
            let letras = r.filho("a:t").map_or(0, |t| t.texto.trim().chars().count());
            if letras == 0 {
                continue;
            }
            let e = estilo_do_trecho(r.filho("a:rPr"), &niveis, h, tema, escala);
            match contagem.iter_mut().find(|(x, _)| *x == e) {
                Some((_, n)) => *n += letras,
                None => contagem.push((e, letras)),
            }
        }
    }
    let paragrafo = corpo.and_then(primeiro_paragrafo);
    let niveis = h.niveis(paragrafo.map_or(0, nivel_de));
    let base = contagem.into_iter().max_by_key(|(_, n)| *n).map(|(e, _)| e).unwrap_or_else(|| estilo_do_trecho(None, &niveis, h, tema, escala));
    let alinhar = paragrafo.and_then(|p| p.filho("a:pPr")).and_then(|pp| pp.attr("algn")).or_else(|| niveis.iter().find_map(|n| n.attr("algn")));
    let ancora = h.corpos().iter().find_map(|b| b.attr("anchor"));
    EstiloDoTexto {
        alinhamento: match alinhar {
            Some("ctr") => Alinhamento::Centro,
            Some("r") => Alinhamento::Direita,
            _ => Alinhamento::Esquerda,
        },
        vertical: match ancora {
            Some("ctr") => Vertical::Meio,
            Some("b") => Vertical::Base,
            _ => Vertical::Topo,
        },
        ..base
    }
}
