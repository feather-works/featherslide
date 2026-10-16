//! Importação SVG vetorial local. Recursos sem equivalente editável são recusados explicitamente.
use featherslide_core::{
    dados::Primitiva,
    modelo::{Elemento, Tipo},
    vetores::Segmento,
};
use usvg::{tiny_skia_path::PathSegment, Node, Paint};

#[derive(Clone, Debug)]
pub struct Desenho {
    pub tamanho: [f32; 2],
    pub elementos: Vec<Elemento>,
}

/// Desativa qualquer resolução de arquivos externos. Nunca busca recursos na rede.
pub fn importar(bytes: &[u8]) -> Result<Desenho, String> {
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("SVG excede 8 MB.".into());
    }
    validar_xml(bytes)?;
    let opcoes = usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_string: Box::new(|_, _| None),
            resolve_data: Box::new(|_, _, _| None),
        },
        ..Default::default()
    };
    let arvore = usvg::Tree::from_data(bytes, &opcoes).map_err(|e| format!("SVG inválido: {e}"))?;
    let tamanho = [arvore.size().width(), arvore.size().height()];
    let mut elementos = Vec::new();
    ler_grupo(arvore.root(), &mut elementos)?;
    if elementos.is_empty() {
        return Err("SVG sem caminhos vetoriais visíveis.".into());
    }
    Ok(Desenho { tamanho, elementos })
}

fn validar_xml(bytes: &[u8]) -> Result<(), String> {
    use quick_xml::{events::Event, Reader};
    let mut leitor = Reader::from_reader(bytes);
    let mut numero = 0;
    loop {
        match leitor
            .read_event()
            .map_err(|e| format!("XML inválido: {e}"))?
        {
            Event::Start(e) | Event::Empty(e) => {
                numero += 1;
                if numero > 20_000 {
                    return Err("SVG tem elementos demais.".into());
                }
                let nome = e.local_name();
                if !matches!(
                    nome.as_ref(),
                    "svg"
                        | "g"
                        | "path"
                        | "rect"
                        | "circle"
                        | "ellipse"
                        | "polygon"
                        | "polyline"
                        | "line"
                        | "defs"
                        | "use"
                        | "symbol"
                        | "title"
                        | "desc"
                        | "style"
                        | "metadata"
                ) {
                    return Err(format!(
                        "Recurso SVG ainda sem conversão editável: {}.",
                        nome.as_ref()
                    ));
                }
            }
            Event::DocType(_) => return Err("SVG com DTD não é aceito.".into()),
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(())
}

fn ler_grupo(grupo: &usvg::Group, elementos: &mut Vec<Elemento>) -> Result<(), String> {
    if grupo.clip_path().is_some()
        || grupo.mask().is_some()
        || !grupo.filters().is_empty()
        || grupo.opacity().get() != 1.0
    {
        return Err(
            "Máscaras, recortes, filtros e opacidade de grupo ainda não são convertíveis.".into(),
        );
    }
    for no in grupo.children() {
        match no {
            Node::Group(g) => ler_grupo(g, elementos)?,
            Node::Path(p) if p.is_visible() => {
                if elementos.len() >= 10_000 {
                    return Err("SVG excede 10.000 caminhos.".into());
                }
                elementos.push(ler_caminho(p)?);
            }
            Node::Path(_) => {}
            _ => {
                return Err(
                    "SVG contém uma imagem ou texto que precisa ser convertido em caminhos.".into(),
                )
            }
        }
    }
    Ok(())
}

fn cor(paint: &Paint, opacidade: f32) -> Result<String, String> {
    let Paint::Color(c) = paint else {
        return Err("Gradientes e padrões ainda não têm edição vetorial.".into());
    };
    Ok(format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        c.red,
        c.green,
        c.blue,
        (opacidade * 255.0).round() as u8
    ))
}

fn ler_caminho(p: &usvg::Path) -> Result<Elemento, String> {
    let caminho = p
        .data()
        .clone()
        .transform(p.abs_transform())
        .ok_or("Transformação SVG inválida.")?;
    let caixa = caminho.bounds();
    let (w, h) = (caixa.width().max(0.001), caixa.height().max(0.001));
    let ponto = |p: usvg::tiny_skia_path::Point| {
        format!("{} {}", (p.x - caixa.x()) / w, (p.y - caixa.y()) / h)
    };
    let par = p
        .fill()
        .is_some_and(|f| f.rule() == usvg::FillRule::EvenOdd);
    let mut dados = format!("SVG1\n{}\n", if par { "par" } else { "normal" });
    for s in caminho.segments() {
        let linha = match s {
            PathSegment::MoveTo(p) => format!("M {}", ponto(p)),
            PathSegment::LineTo(p) => format!("L {}", ponto(p)),
            PathSegment::QuadTo(a, b) => format!("Q {} {}", ponto(a), ponto(b)),
            PathSegment::CubicTo(a, b, c) => format!("C {} {} {}", ponto(a), ponto(b), ponto(c)),
            PathSegment::Close => "Z".into(),
        };
        dados.push_str(&linha);
        dados.push('\n');
    }
    let preenchimento = p
        .fill()
        .map(|f| cor(f.paint(), f.opacity().get()))
        .transpose()?
        .unwrap_or_default();
    let mut contorno = String::new();
    let mut espessura = 0.0;
    if let Some(s) = p.stroke() {
        if s.dasharray().is_some()
            || s.linecap() != usvg::LineCap::Butt
            || s.linejoin() != usvg::LineJoin::Miter
        {
            return Err(
                "Traços tracejados e pontas/junções especiais ainda não são convertíveis.".into(),
            );
        }
        contorno = cor(s.paint(), s.opacity().get())?;
        let t = p.abs_transform();
        let sx = t.sx.hypot(t.ky);
        let sy = t.kx.hypot(t.sy);
        if (sx - sy).abs() > 0.0001 {
            return Err("Contorno com escala não uniforme ainda não é convertível.".into());
        }
        espessura = s.width().get() * sx;
    }
    Ok(Elemento {
        tipo: Tipo::Caminho,
        x: caixa.x(),
        y: caixa.y(),
        largura: w,
        altura: h,
        dados,
        preenchimento,
        contorno,
        espessura,
        ..Default::default()
    })
}

/// Caminho SVG com curvas preservadas, também usado na exportação do slide.
pub fn caminho(segmentos: &[Segmento]) -> String {
    let p = |p: &[f32; 2]| format!("{} {}", p[0], p[1]);
    segmentos
        .iter()
        .map(|s| match s {
            Segmento::M(a) => format!("M{}", p(a)),
            Segmento::L(a) => format!("L{}", p(a)),
            Segmento::Q(a, b) => format!("Q{} {}", p(a), p(b)),
            Segmento::C(a, b, c) => format!("C{} {} {}", p(a), p(b), p(c)),
            Segmento::Z => "Z".into(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn exportar(elementos: &[Elemento], tamanho: [f32; 2]) -> Result<String, String> {
    if elementos.is_empty() {
        return Err("Selecione ao menos um vetor para exportar.".into());
    }
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\">",
        tamanho[0], tamanho[1]
    );
    for e in elementos {
        let primitivas = if e.tipo == Tipo::Icone {
            featherslide_core::icones::primitivas(e)
        } else if e.tipo == Tipo::Caminho {
            featherslide_core::booleanas::primitivas(e)
        } else {
            return Err(
                "A seleção contém elementos que não são vetores SVG, ícones ou caminhos.".into(),
            );
        };
        svg.push_str(&format!(
            "<g transform=\"rotate({} {} {})\">",
            e.rotacao,
            e.x + e.largura / 2.0,
            e.y + e.altura / 2.0
        ));
        for primitiva in primitivas {
            svg.push_str(&exportar_primitiva(&primitiva)?);
        }
        svg.push_str("</g>");
    }
    svg.push_str("</svg>");
    Ok(svg)
}

fn tinta(cor: &str) -> Result<String, String> {
    if cor.is_empty() {
        return Ok("none".into());
    }
    if !cor.starts_with('#')
        || !matches!(cor.len(), 4 | 7 | 9)
        || !cor[1..].bytes().all(|c| c.is_ascii_hexdigit())
    {
        return Err("Cor vetorial inválida.".into());
    }
    Ok(cor.into())
}

fn exportar_primitiva(p: &Primitiva) -> Result<String, String> {
    let (d, preenchimento, contorno, espessura, par) = match p {
        Primitiva::Vetor {
            segmentos,
            preenchimento,
            contorno,
            espessura,
            par,
        } => (
            caminho(segmentos),
            preenchimento.as_str(),
            contorno.as_str(),
            *espessura,
            *par,
        ),
        Primitiva::Caminho {
            contornos,
            preenchimento,
            contorno,
            espessura,
        } => {
            let mut segmentos = Vec::new();
            for pontos in contornos {
                for (i, p) in pontos.iter().enumerate() {
                    segmentos.push(if i == 0 {
                        Segmento::M(*p)
                    } else {
                        Segmento::L(*p)
                    });
                }
                segmentos.push(Segmento::Z);
            }
            (
                caminho(&segmentos),
                preenchimento.as_str(),
                contorno.as_str(),
                *espessura,
                false,
            )
        }
        Primitiva::Poli {
            pontos,
            cor,
            espessura,
        } => {
            let s: Vec<_> = pontos
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    if i == 0 {
                        Segmento::M(*p)
                    } else {
                        Segmento::L(*p)
                    }
                })
                .collect();
            (caminho(&s), "", cor.as_str(), *espessura, false)
        }
        _ => return Err("Primitiva sem exportação SVG.".into()),
    };
    Ok(format!(
        "<path d=\"{d}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{espessura}\" fill-rule=\"{}\"/>",
        tinta(preenchimento)?,
        tinta(contorno)?,
        if par { "evenodd" } else { "nonzero" }
    ))
}
