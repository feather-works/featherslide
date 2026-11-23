//! A transição do slide no `.pptx` (`p:transition`), nos dois sentidos. O PowerPoint grava a
//! duração exata (`p14:dur`) dentro de um `mc:AlternateContent`, com uma cópia sem ela para quem não
//! conhece o `p14`; aqui se faz o mesmo.

use featherslide_core::modelo::{Direcao, TipoTransicao, Transicao};

use crate::xml::No;

/// O `dir` do `.pptx` diz para onde o slide novo anda; a [`Direcao`] diz de onde ele vem.
fn direcao_de(dir: Option<&str>) -> Direcao {
    match dir.and_then(|d| d.chars().next()) {
        Some('r') => Direcao::Esquerda,
        Some('u') => Direcao::Baixo,
        Some('d') => Direcao::Cima,
        _ => Direcao::Direita, // "l", o padrão do formato
    }
}

fn dir_de(d: Direcao) -> &'static str {
    match d {
        Direcao::Esquerda => "r",
        Direcao::Direita => "l",
        Direcao::Cima => "d",
        Direcao::Baixo => "u",
    }
}

/// A transição do slide (o primeiro `p:transition`, que no `mc:Choice` é o que tem a duração exata).
pub(crate) fn ler(slide: &No) -> Transicao {
    let Some(t) = slide.busca("p:transition") else { return Transicao::default() };
    let duracao = t.num("p14:dur").map(|ms| ms as f32 / 1000.0).unwrap_or(match t.attr("spd") {
        Some("fast") => 0.5,
        Some("slow") => 1.0,
        _ => 0.75,
    });
    let apos = t.num("advTm").map_or(0.0, |ms| (ms / 1000.0) as f32);
    let efeito = t.filhos.first();
    let tipo = match efeito.map(|e| e.nome.as_str()) {
        None => TipoTransicao::Nenhuma,
        Some("p:fade") => TipoTransicao::Esmaecer,
        Some("p:cover") => TipoTransicao::Deslizar,
        Some("p:push") => TipoTransicao::Empurrar,
        Some("p:pull") => TipoTransicao::Revelar,
        Some("p:wipe") => TipoTransicao::Cortina,
        Some("p:zoom") => TipoTransicao::Zoom,
        // shortcut: os efeitos que ainda não temos (dividir, persiana, vórtice…) tocam como esmaecer; mapear um a um quando houver o efeito.
        Some(_) => TipoTransicao::Esmaecer,
    };
    Transicao { tipo, direcao: direcao_de(efeito.and_then(|e| e.attr("dir"))), duracao: duracao.clamp(0.0, 60.0), apos: apos.clamp(0.0, 86_400.0) }
}

/// O XML da transição, para ir logo depois do `p:clrMapOvr` (vazio sem transição nem avanço automático).
pub(crate) fn gravar(t: &Transicao) -> String {
    let efeito = match t.tipo {
        TipoTransicao::Nenhuma => String::new(),
        TipoTransicao::Esmaecer => "<p:fade/>".into(),
        TipoTransicao::Zoom => "<p:zoom/>".into(),
        TipoTransicao::Deslizar => format!(r#"<p:cover dir="{}"/>"#, dir_de(t.direcao)),
        TipoTransicao::Empurrar => format!(r#"<p:push dir="{}"/>"#, dir_de(t.direcao)),
        TipoTransicao::Revelar => format!(r#"<p:pull dir="{}"/>"#, dir_de(t.direcao)),
        TipoTransicao::Cortina => format!(r#"<p:wipe dir="{}"/>"#, dir_de(t.direcao)),
    };
    if efeito.is_empty() && t.apos <= 0.0 {
        return String::new();
    }
    let spd = if t.duracao < 0.6 { "fast" } else if t.duracao < 0.9 { "med" } else { "slow" };
    let avanco = if t.apos > 0.0 { format!(r#" advTm="{}""#, (t.apos * 1000.0).round() as u64) } else { String::new() };
    let dur = (t.duracao * 1000.0).round() as u64;
    format!(
        r#"<mc:AlternateContent xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"><mc:Choice xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main" Requires="p14"><p:transition spd="{spd}" p14:dur="{dur}"{avanco}>{efeito}</p:transition></mc:Choice><mc:Fallback><p:transition spd="{spd}"{avanco}>{efeito}</p:transition></mc:Fallback></mc:AlternateContent>"#
    )
}

#[cfg(test)]
mod testes {
    use super::*;

    fn slide_com(xml: &str) -> No {
        let doc = format!(r#"<p:sld xmlns:p="p" xmlns:mc="mc" xmlns:p14="p14">{xml}</p:sld>"#);
        crate::xml::ler(doc.as_bytes(), "slide1.xml").unwrap()
    }

    #[test]
    fn cada_transicao_vai_e_volta() {
        for tipo in TipoTransicao::TODOS {
            for direcao in Direcao::TODOS {
                let t = Transicao { tipo, direcao, duracao: 1.2, apos: if tipo == TipoTransicao::Nenhuma { 3.0 } else { 0.0 } };
                let lida = ler(&slide_com(&gravar(&t)));
                assert_eq!(lida.tipo, tipo);
                assert_eq!(lida.duracao, 1.2, "{tipo:?}");
                assert_eq!(lida.apos, t.apos);
                if tipo.tem_direcao() {
                    assert_eq!(lida.direcao, direcao, "{tipo:?}");
                }
            }
        }
        assert!(gravar(&Transicao::default()).is_empty(), "sem transição, nada no slide");
    }

    #[test]
    fn le_o_que_o_powerpoint_grava() {
        // "Empurrar, De baixo", 2 s, avança depois de 5 s; sem p14, a velocidade decide.
        let t = ler(&slide_com(r#"<p:transition spd="slow" advTm="5000"><p:push dir="u"/></p:transition>"#));
        assert_eq!((t.tipo, t.direcao, t.duracao, t.apos), (TipoTransicao::Empurrar, Direcao::Baixo, 1.0, 5.0));
        let t = ler(&slide_com(r#"<p:transition spd="med"><p:randomBar/></p:transition>"#));
        assert_eq!(t.tipo, TipoTransicao::Esmaecer, "efeito que não temos ainda vira esmaecer");
        assert_eq!(ler(&slide_com("")).tipo, TipoTransicao::Nenhuma);
    }
}
