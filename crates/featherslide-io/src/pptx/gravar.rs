//! Grava a apresentação como `.pptx`, para abrir no PowerPoint, no Google Apresentações e no
//! Impress: slides no tamanho certo, caixas de texto (fonte, tamanho, cor, negrito, itálico,
//! sublinhado, alinhamento, margem), formas prontas com cor e contorno, fotos, giro, fundo
//! sólido e anotações do apresentador.
//!
//! shortcut: tabelas, gráficos, ícones, WordArt e caminhos vetoriais ainda não têm equivalente e
//! ficam de fora (a função devolve quantos); animações e transições também. Gravar cada um quando
//! a pessoa pedir — a leitura em `mod.rs` já mostra o XML de cada coisa.

use std::fmt::Write as _;
use std::io::{Cursor, Write};

use featherslide_core::modelo::{Alinhamento, Apresentacao, Elemento, Papel, Slide, Tipo, Vertical};
use zip::write::SimpleFileOptions;

use super::EMU_POR_PT;
use crate::base64::de_data_url;

/// O que a gravação fez: os bytes do `.pptx` e quantos elementos ficaram de fora.
pub struct Gravado {
    pub bytes: Vec<u8>,
    pub de_fora: usize,
}

const NS: &str = r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main""#;
const R_SLIDE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide";
const R_LAYOUT: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout";
const R_MESTRE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster";
const R_TEMA: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme";
const R_IMAGEM: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
const R_NOTAS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide";
const R_MESTRE_NOTAS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesMaster";

fn emu(pt: f32) -> i64 {
    (f64::from(pt) * EMU_POR_PT).round() as i64
}

fn esc(t: &str) -> String {
    let mut s = String::with_capacity(t.len());
    for c in t.chars() {
        match c {
            '&' => s.push_str("&amp;"),
            '<' => s.push_str("&lt;"),
            '>' => s.push_str("&gt;"),
            '"' => s.push_str("&quot;"),
            // Controle fora de \t\n\r é proibido em XML 1.0.
            c if (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => {}
            c => s.push(c),
        }
    }
    s
}

/// `#rrggbb` → `RRGGBB` (vazio ou inválido: `None`).
fn hex(cor: &str) -> Option<String> {
    let h = cor.trim().strip_prefix('#')?;
    (h.len() >= 6 && h[..6].chars().all(|c| c.is_ascii_hexdigit())).then(|| h[..6].to_ascii_uppercase())
}

/// A geometria pronta do PowerPoint para um tipo (`None`: sem equivalente, fica de fora).
fn geometria(t: Tipo) -> Option<&'static str> {
    Some(match t {
        Tipo::Texto | Tipo::Retangulo => "rect",
        Tipo::RetanguloArredondado => "roundRect",
        Tipo::Elipse | Tipo::Circulo => "ellipse",
        Tipo::Triangulo => "triangle",
        Tipo::TrianguloRetangulo => "rtTriangle",
        Tipo::Seta => "rightArrow",
        Tipo::SetaEsquerda => "leftArrow",
        Tipo::SetaDupla => "leftRightArrow",
        Tipo::SetaCima => "upArrow",
        Tipo::SetaBaixo => "downArrow",
        Tipo::Chevron => "chevron",
        Tipo::Linha => "line",
        Tipo::Setor => "pie",
        Tipo::Losango => "diamond",
        Tipo::Pentagono => "pentagon",
        Tipo::Hexagono => "hexagon",
        Tipo::Heptagono => "heptagon",
        Tipo::Octogono => "octagon",
        Tipo::Decagono => "decagon",
        Tipo::Dodecagono => "dodecagon",
        Tipo::Trapezio => "trapezoid",
        Tipo::Paralelogramo => "parallelogram",
        Tipo::Estrela => "star5",
        Tipo::Estrela4 => "star4",
        Tipo::Estrela6 => "star6",
        Tipo::Estrela8 => "star8",
        Tipo::Estrela10 => "star10",
        Tipo::Estrela12 => "star12",
        Tipo::Cruz => "plus",
        Tipo::Documento => "flowChartDocument",
        Tipo::Anel => "donut",
        Tipo::Arco => "arc",
        Tipo::Corda => "chord",
        Tipo::Cubo => "cube",
        Tipo::Cilindro => "can",
        Tipo::Moldura => "frame",
        Tipo::CantoEmL => "corner",
        Tipo::Placa => "plaque",
        Tipo::Sol => "sun",
        Tipo::Lua => "moon",
        Tipo::Coracao => "heart",
        Tipo::Raio => "lightningBolt",
        Tipo::Nuvem => "cloud",
        Tipo::Imagem | Tipo::Tabela | Tipo::Grafico | Tipo::Icone | Tipo::WordArt | Tipo::Caminho => return None,
    })
}

/// O `a:xfrm` de um elemento (posição, tamanho, giro).
fn xfrm(e: &Elemento, prefixo: &str) -> String {
    let rot = if e.rotacao.rem_euclid(360.0) > 0.01 { format!(r#" rot="{}""#, (e.rotacao.rem_euclid(360.0) * 60_000.0).round() as i64) } else { String::new() };
    let (x, w, flip_h) = if e.largura < 0.0 { (e.x + e.largura, -e.largura, r#" flipH="1""#) } else { (e.x, e.largura, "") };
    let (y, h, flip_v) = if e.altura < 0.0 { (e.y + e.altura, -e.altura, r#" flipV="1""#) } else { (e.y, e.altura, "") };
    format!(r#"<{prefixo}:xfrm{rot}{flip_h}{flip_v}><a:off x="{}" y="{}"/><a:ext cx="{}" cy="{}"/></{prefixo}:xfrm>"#, emu(x), emu(y), emu(w).max(0), emu(h).max(0))
}

/// O corpo de texto (`p:txBody`) de uma caixa.
fn corpo(e: &Elemento) -> String {
    let est = &e.estilo;
    let [esq, cima, dir, baixo] = e.margens();
    let ancora = match est.vertical {
        Vertical::Topo => "t",
        Vertical::Meio => "ctr",
        Vertical::Base => "b",
    };
    let algn = match est.alinhamento {
        Alinhamento::Esquerda => "l",
        Alinhamento::Centro => "ctr",
        Alinhamento::Direita => "r",
        Alinhamento::Justificado => "just",
    };
    let mut s = format!(
        r#"<p:txBody><a:bodyPr wrap="square" lIns="{}" tIns="{}" rIns="{}" bIns="{}" anchor="{ancora}"{}><a:noAutofit/></a:bodyPr><a:lstStyle/>"#,
        emu(esq),
        emu(cima),
        emu(dir),
        emu(baixo),
        if est.colunas == 2 { r#" numCol="2""# } else { "" }
    );
    let mut rpr = format!(r#" lang="pt-BR" sz="{}""#, (est.tamanho * 100.0).round().clamp(100.0, 400_000.0) as i64);
    if est.negrito {
        rpr.push_str(r#" b="1""#);
    }
    if est.italico {
        rpr.push_str(r#" i="1""#);
    }
    if est.sublinhado || est.sublinhado_duplo {
        rpr.push_str(if est.sublinhado_duplo { r#" u="dbl""# } else { r#" u="sng""# });
    }
    if est.tachado || est.tachado_duplo {
        rpr.push_str(if est.tachado_duplo { r#" strike="dblStrike""# } else { r#" strike="sngStrike""# });
    }
    let mut dentro = String::new();
    if let Some(c) = hex(&est.cor) {
        let _ = write!(dentro, r#"<a:solidFill><a:srgbClr val="{c}"/></a:solidFill>"#);
    }
    if !est.fonte.trim().is_empty() {
        let _ = write!(dentro, r#"<a:latin typeface="{}"/>"#, esc(&est.fonte));
    }
    let entrelinha = if est.entrelinha > 0.0 { format!(r#"<a:lnSpc><a:spcPct val="{}"/></a:lnSpc>"#, (est.entrelinha * 100_000.0).round() as i64) } else { String::new() };
    let paragrafos: Vec<&str> = if e.texto.is_empty() { vec![""] } else { e.texto.split('\n').collect() };
    for p in paragrafos {
        let _ = write!(s, r#"<a:p><a:pPr algn="{algn}">{entrelinha}</a:pPr>"#);
        if !p.is_empty() {
            let _ = write!(s, r#"<a:r><a:rPr{rpr}>{dentro}</a:rPr><a:t>{}</a:t></a:r>"#, esc(p));
        }
        let _ = write!(s, r#"<a:endParaRPr{rpr}/></a:p>"#);
    }
    s.push_str("</p:txBody>");
    s
}

/// O tipo de caixa de layout do PowerPoint para o papel (título vira título no outro programa).
fn ph(papel: Papel, idx: usize) -> String {
    match papel {
        Papel::Titulo => r#"<p:nvPr><p:ph type="title"/></p:nvPr>"#.into(),
        Papel::Subtitulo => format!(r#"<p:nvPr><p:ph type="subTitle" idx="{idx}"/></p:nvPr>"#),
        Papel::Corpo => format!(r#"<p:nvPr><p:ph idx="{idx}"/></p:nvPr>"#),
        _ => "<p:nvPr/>".into(),
    }
}

/// Uma forma ou caixa de texto.
fn forma(e: &Elemento, id: usize, geom: &str) -> String {
    let nome = if e.tipo == Tipo::Texto { "Caixa de texto" } else { "Forma" };
    let caixa = if e.tipo == Tipo::Texto { r#" txBox="1""# } else { "" };
    let mut s = format!(r#"<p:sp><p:nvSpPr><p:cNvPr id="{id}" name="{nome} {id}"/><p:cNvSpPr{caixa}/>{}</p:nvSpPr><p:spPr>{}<a:prstGeom prst="{geom}"><a:avLst/></a:prstGeom>"#, ph(e.papel, id), xfrm(e, "a"));
    match hex(&e.preenchimento) {
        Some(c) if geom != "line" && geom != "arc" => {
            let _ = write!(s, r#"<a:solidFill><a:srgbClr val="{c}"/></a:solidFill>"#);
        }
        _ => s.push_str("<a:noFill/>"),
    }
    match hex(&e.contorno) {
        Some(c) => {
            let _ = write!(s, r#"<a:ln w="{}"><a:solidFill><a:srgbClr val="{c}"/></a:solidFill></a:ln>"#, emu(e.espessura.max(0.25)));
        }
        None => s.push_str("<a:ln><a:noFill/></a:ln>"),
    }
    s.push_str("</p:spPr>");
    if geom != "line" {
        s.push_str(&corpo(e));
    }
    s.push_str("</p:sp>");
    s
}

/// Uma foto (`p:pic`) ligada à mídia `rid`.
fn foto(e: &Elemento, id: usize, rid: &str) -> String {
    format!(
        r#"<p:pic><p:nvPicPr><p:cNvPr id="{id}" name="Imagem {id}" descr="{}"/><p:cNvPicPr><a:picLocks noChangeAspect="1"/></p:cNvPicPr><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:embed="{rid}"/><a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr>{}<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:pic>"#,
        esc(&e.texto_alternativo),
        xfrm(e, "a")
    )
}

fn rels(itens: &[(String, &str, String)]) -> String {
    let mut s = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#);
    for (id, tipo, alvo) in itens {
        let _ = write!(s, r#"<Relationship Id="{id}" Type="{tipo}" Target="{}"/>"#, esc(alvo));
    }
    s.push_str("</Relationships>");
    s
}

/// A mídia de um slide: (nome no pacote, bytes).
type Midia = Vec<(String, Vec<u8>)>;

/// O XML de um slide, as ligações dele e as fotos que ele leva.
fn slide(sl: &Slide, n: usize, midia: &mut Midia, de_fora: &mut usize) -> (String, String) {
    let mut ligacoes: Vec<(String, &str, String)> = vec![("rId1".into(), R_LAYOUT, "../slideLayouts/slideLayout1.xml".into())];
    let mut arvore = String::new();
    let mut id = 2;
    for e in sl.elementos.iter().filter(|e| !e.oculto) {
        id += 1;
        if e.tipo == Tipo::Imagem {
            match de_data_url(&e.imagem) {
                Some((mime, bytes)) => {
                    let ext = match mime.as_str() {
                        "image/jpeg" => "jpeg",
                        "image/gif" => "gif",
                        "image/bmp" => "bmp",
                        "image/svg+xml" => "svg",
                        "image/webp" => "webp",
                        _ => "png",
                    };
                    let nome = format!("imagem{}_{}.{ext}", n, midia.len() + 1);
                    let rid = format!("rId{}", ligacoes.len() + 1);
                    ligacoes.push((rid.clone(), R_IMAGEM, format!("../media/{nome}")));
                    midia.push((nome, bytes));
                    arvore.push_str(&foto(e, id, &rid));
                }
                None => *de_fora += 1,
            }
            continue;
        }
        match geometria(e.tipo) {
            Some(g) => arvore.push_str(&forma(e, id, g)),
            None => *de_fora += 1,
        }
    }
    let fundo = hex(&sl.fundo).map_or(String::new(), |c| format!(r#"<p:bg><p:bgPr><a:solidFill><a:srgbClr val="{c}"/></a:solidFill><a:effectLst/></p:bgPr></p:bg>"#));
    let oculto = if sl.oculto { r#" show="0""# } else { "" };
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:sld {NS}{oculto}><p:cSld>{fundo}<p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>{arvore}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>"#
    );
    if !sl.notas.trim().is_empty() {
        ligacoes.push((format!("rId{}", ligacoes.len() + 1), R_NOTAS, format!("../notesSlides/notesSlide{n}.xml")));
    }
    (xml, rels(&ligacoes))
}

/// As anotações de um slide.
fn notas(texto: &str, n: usize) -> (String, String) {
    let paragrafos: String = texto.split('\n').map(|p| if p.is_empty() { "<a:p/>".to_string() } else { format!("<a:p><a:r><a:rPr lang=\"pt-BR\"/><a:t>{}</a:t></a:r></a:p>", esc(p)) }).collect();
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:notes {NS}><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:sp><p:nvSpPr><p:cNvPr id="2" name="Imagem do slide"/><p:cNvSpPr><a:spLocks noGrp="1" noRot="1" noChangeAspect="1"/></p:cNvSpPr><p:nvPr><p:ph type="sldImg"/></p:nvPr></p:nvSpPr><p:spPr/></p:sp><p:sp><p:nvSpPr><p:cNvPr id="3" name="Anotações"/><p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr><p:ph type="body" idx="1"/></p:nvPr></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/>{paragrafos}</p:txBody></p:sp></p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:notes>"#
    );
    let r = rels(&[("rId1".into(), R_MESTRE_NOTAS, "../notesMasters/notesMaster1.xml".into()), ("rId2".into(), R_SLIDE, format!("../slides/slide{n}.xml"))]);
    (xml, r)
}

const TEMA: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="FeatherSlide"><a:themeElements><a:clrScheme name="FeatherSlide"><a:dk1><a:srgbClr val="202124"/></a:dk1><a:lt1><a:srgbClr val="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="44546A"/></a:dk2><a:lt2><a:srgbClr val="E7E6E6"/></a:lt2><a:accent1><a:srgbClr val="1A73E8"/></a:accent1><a:accent2><a:srgbClr val="EA4335"/></a:accent2><a:accent3><a:srgbClr val="FBBC04"/></a:accent3><a:accent4><a:srgbClr val="34A853"/></a:accent4><a:accent5><a:srgbClr val="FF6D01"/></a:accent5><a:accent6><a:srgbClr val="46BDC6"/></a:accent6><a:hlink><a:srgbClr val="1A73E8"/></a:hlink><a:folHlink><a:srgbClr val="681DA8"/></a:folHlink></a:clrScheme><a:fontScheme name="FeatherSlide"><a:majorFont><a:latin typeface="Liberation Sans"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Liberation Sans"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme><a:fmtScheme name="FeatherSlide"><a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:fillStyleLst><a:lnStyleLst><a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="19050"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="28575"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln></a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:bgFillStyleLst></a:fmtScheme></a:themeElements></a:theme>"#;

const GRUPO_VAZIO: &str = r#"<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>"#;
const MAPA_CORES: &str = r#"<p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>"#;

/// Grava a apresentação como `.pptx` em memória.
pub fn gravar(ap: &Apresentacao) -> Result<Gravado, String> {
    let (w, h) = ap.tamanho();
    let mut partes: Vec<(String, Vec<u8>)> = Vec::new();
    let mut midia: Midia = Vec::new();
    let mut de_fora = 0;
    let mut tipos = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Default Extension="png" ContentType="image/png"/><Default Extension="jpeg" ContentType="image/jpeg"/><Default Extension="gif" ContentType="image/gif"/><Default Extension="bmp" ContentType="image/bmp"/><Default Extension="svg" ContentType="image/svg+xml"/><Default Extension="webp" ContentType="image/webp"/><Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/><Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/><Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/><Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/><Override PartName="/ppt/theme/theme2.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/><Override PartName="/ppt/notesMasters/notesMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.notesMaster+xml"/><Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>"#);
    let mut lista_slides = String::new();
    let mut rels_pres: Vec<(String, &str, String)> = vec![("rId1".into(), R_MESTRE, "slideMasters/slideMaster1.xml".into()), ("rId2".into(), R_TEMA, "theme/theme1.xml".into()), ("rId3".into(), R_MESTRE_NOTAS, "notesMasters/notesMaster1.xml".into())];
    for (i, sl) in ap.slides.iter().enumerate() {
        let k = i + 1;
        let (xml, r) = slide(sl, k, &mut midia, &mut de_fora);
        partes.push((format!("ppt/slides/slide{k}.xml"), xml.into_bytes()));
        partes.push((format!("ppt/slides/_rels/slide{k}.xml.rels"), r.into_bytes()));
        let _ = write!(tipos, r#"<Override PartName="/ppt/slides/slide{k}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>"#);
        if !sl.notas.trim().is_empty() {
            let (xml, r) = notas(&sl.notas, k);
            partes.push((format!("ppt/notesSlides/notesSlide{k}.xml"), xml.into_bytes()));
            partes.push((format!("ppt/notesSlides/_rels/notesSlide{k}.xml.rels"), r.into_bytes()));
            let _ = write!(tipos, r#"<Override PartName="/ppt/notesSlides/notesSlide{k}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml"/>"#);
        }
        let rid = format!("rId{}", rels_pres.len() + 1);
        let _ = write!(lista_slides, r#"<p:sldId id="{}" r:id="{rid}"/>"#, 256 + i);
        rels_pres.push((rid, R_SLIDE, format!("slides/slide{k}.xml")));
    }
    tipos.push_str("</Types>");
    let (cx, cy) = (emu(w), emu(h));
    let pres = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:presentation {NS} saveSubsetFonts="1"><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst><p:notesMasterIdLst><p:notesMasterId r:id="rId3"/></p:notesMasterIdLst><p:sldIdLst>{lista_slides}</p:sldIdLst><p:sldSz cx="{cx}" cy="{cy}"/><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#
    );
    let mestre = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:sldMaster {NS}><p:cSld><p:bg><p:bgRef idx="1001"><a:schemeClr val="bg1"/></p:bgRef></p:bg><p:spTree>{GRUPO_VAZIO}</p:spTree></p:cSld>{MAPA_CORES}<p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst><p:txStyles><p:titleStyle><a:lvl1pPr><a:defRPr sz="4400"/></a:lvl1pPr></p:titleStyle><p:bodyStyle><a:lvl1pPr><a:defRPr sz="2400"/></a:lvl1pPr></p:bodyStyle><p:otherStyle><a:lvl1pPr><a:defRPr sz="1800"/></a:lvl1pPr></p:otherStyle></p:txStyles></p:sldMaster>"#
    );
    let layout = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:sldLayout {NS} type="blank" preserve="1"><p:cSld name="Em branco"><p:spTree>{GRUPO_VAZIO}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sldLayout>"#);
    let mestre_notas = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:notesMaster {NS}><p:cSld><p:spTree>{GRUPO_VAZIO}</p:spTree></p:cSld>{MAPA_CORES}</p:notesMaster>"#);
    let core = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>{}</dc:title><dc:creator>{}</dc:creator></cp:coreProperties>"#,
        esc(&ap.titulo),
        esc(&ap.autor)
    );
    let mut fixas: Vec<(String, Vec<u8>)> = vec![
        ("[Content_Types].xml".into(), tipos.into_bytes()),
        ("_rels/.rels".into(), rels(&[("rId1".into(), "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument", "ppt/presentation.xml".into()), ("rId2".into(), "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties", "docProps/core.xml".into())]).into_bytes()),
        ("docProps/core.xml".into(), core.into_bytes()),
        ("ppt/presentation.xml".into(), pres.into_bytes()),
        ("ppt/_rels/presentation.xml.rels".into(), rels(&rels_pres).into_bytes()),
        ("ppt/slideMasters/slideMaster1.xml".into(), mestre.into_bytes()),
        ("ppt/slideMasters/_rels/slideMaster1.xml.rels".into(), rels(&[("rId1".into(), R_LAYOUT, "../slideLayouts/slideLayout1.xml".into()), ("rId2".into(), R_TEMA, "../theme/theme1.xml".into())]).into_bytes()),
        ("ppt/slideLayouts/slideLayout1.xml".into(), layout.into_bytes()),
        ("ppt/slideLayouts/_rels/slideLayout1.xml.rels".into(), rels(&[("rId1".into(), R_MESTRE, "../slideMasters/slideMaster1.xml".into())]).into_bytes()),
        ("ppt/theme/theme1.xml".into(), TEMA.as_bytes().to_vec()),
        ("ppt/theme/theme2.xml".into(), TEMA.as_bytes().to_vec()),
        ("ppt/notesMasters/notesMaster1.xml".into(), mestre_notas.into_bytes()),
        ("ppt/notesMasters/_rels/notesMaster1.xml.rels".into(), rels(&[("rId1".into(), R_TEMA, "../theme/theme2.xml".into())]).into_bytes()),
    ];
    fixas.append(&mut partes);
    fixas.extend(midia.into_iter().map(|(nome, b)| (format!("ppt/media/{nome}"), b)));
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opcoes = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (nome, conteudo) in fixas {
        zip.start_file(nome, opcoes).map_err(|e| e.to_string())?;
        zip.write_all(&conteudo).map_err(|e| e.to_string())?;
    }
    let bytes = zip.finish().map_err(|e| e.to_string())?.into_inner();
    Ok(Gravado { bytes, de_fora })
}

/// Grava o `.pptx` em `destino` (troca atômica: um erro no meio não estraga o arquivo que já existia).
pub fn salvar(ap: &Apresentacao, destino: &std::path::Path) -> Result<Gravado, String> {
    let g = gravar(ap)?;
    crate::arquivo::gravar_atomico(destino, &g.bytes)?;
    Ok(g)
}
