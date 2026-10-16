//! Ler `.pptx` (PowerPoint, e o que o Google Apresentações exporta). Rust puro: o ZIP, o
//! XML em árvore (`crate::xml`) e as regras de herança do PowerPoint — a forma sem posição
//! pega a da caixa equivalente no layout (e esta no mestre), o fundo idem, as cores do tema
//! passam pelo mapa do mestre.
//!
//! O que entra: caixas de texto e de layout (título, subtítulo, corpo), formas (retângulo,
//! arredondado, elipse, setor, triângulo, seta), linhas e conectores, imagens (também a forma
//! preenchida com imagem, e a foto recortada em elipse), sombra, transparência, grupos
//! (achatados), fundo sólido, as formas do layout e do mestre (entram no slide como fundo,
//! [`Papel::Fundo`]), slide oculto e as anotações. O que fica de fora, por ora: tabelas e
//! gráficos (`p:graphicFrame`), fundo com imagem ou gradiente, recorte da imagem (`a:srcRect`),
//! estilo diferente por trecho (vale o predominante), animações e transições.

mod cores;
mod fontes;
mod texto;

use std::collections::HashMap;
use std::rc::Rc;

use featherslide_core::modelo::{Apresentacao, Elemento, Papel, Proporcao, Slide, Sombra, Tipo};

use crate::base64;
use crate::pacote::Pacote;
use crate::xml::No;
use cores::Tema;
use texto::Heranca;

/// EMU por ponto (o `.pptx` mede em EMU: 914 400 por polegada).
const EMU_POR_PT: f64 = 12_700.0;

/// De EMU do slide (ou de dentro de um grupo) para pontos do slide do FeatherSlide.
#[derive(Clone, Copy)]
struct Transformacao {
    ax: f64,
    bx: f64,
    ay: f64,
    by: f64,
}

impl Transformacao {
    /// A transformação de dentro de um grupo (`a:xfrm` com `a:chOff`/`a:chExt`).
    fn do_grupo(self, xfrm: &No) -> Self {
        let par = |nome: &str, a: &str, b: &str| xfrm.filho(nome).map_or((0.0, 0.0), |n| (n.num(a).unwrap_or(0.0), n.num(b).unwrap_or(0.0)));
        let (ox, oy) = par("a:off", "x", "y");
        let (ex, ey) = par("a:ext", "cx", "cy");
        let (cox, coy) = par("a:chOff", "x", "y");
        let (cex, cey) = par("a:chExt", "cx", "cy");
        let kx = if cex > 0.0 { ex / cex } else { 1.0 };
        let ky = if cey > 0.0 { ey / cey } else { 1.0 };
        Self { ax: self.ax * kx, bx: self.ax * (ox - cox * kx) + self.bx, ay: self.ay * ky, by: self.ay * (oy - coy * ky) + self.by }
    }

    /// O retângulo (x, y, largura, altura) em pontos, de um `a:xfrm`.
    fn retangulo(self, xfrm: &No) -> Option<[f32; 4]> {
        let off = xfrm.filho("a:off")?;
        let ext = xfrm.filho("a:ext")?;
        let (x, y) = (off.num("x")?, off.num("y")?);
        let (w, h) = (ext.num("cx").unwrap_or(0.0), ext.num("cy").unwrap_or(0.0));
        Some([(self.ax * x + self.bx) as f32, (self.ay * y + self.by) as f32, (self.ax * w) as f32, (self.ay * h) as f32])
    }
}

/// O layout e o mestre de um slide, já lidos, com o tema.
struct Contexto {
    layout: Option<Rc<No>>,
    mestre: Option<Rc<No>>,
    tema: Rc<Tema>,
    /// As ligações do slide (para as imagens).
    ligacoes: HashMap<String, String>,
    /// Os caminhos do layout e do mestre (as ligações deles, para as imagens do fundo).
    layout_c: Option<String>,
    mestre_c: Option<String>,
}

/// De onde vêm as formas que se está lendo: o slide, ou o layout/mestre (o fundo).
#[derive(Clone, Copy)]
struct Origem<'r> {
    ligacoes: &'r HashMap<String, String>,
    fundo: bool,
}

struct Leitor<'a> {
    pac: Pacote<'a>,
    t: Transformacao,
    /// Pontos do `.pptx` → pontos do slide (o tamanho das letras e das linhas).
    escala: f32,
    proximo: u64,
    temas: HashMap<String, Rc<Tema>>,
}

/// Lê um `.pptx`. Arquivo malformado dá erro em português, nunca pânico.
pub fn ler(bytes: &[u8]) -> Result<Apresentacao, String> {
    let mut pac = Pacote::abrir(bytes, ".pptx")?;
    if !pac.tem("ppt/presentation.xml") {
        return Err("Este arquivo não é uma apresentação do PowerPoint (falta ppt/presentation.xml).".into());
    }
    let pres = pac.xml("ppt/presentation.xml")?;
    let tam = pres.filho("p:sldSz");
    let (cx, cy) = (tam.and_then(|t| t.num("cx")).unwrap_or(12_192_000.0), tam.and_then(|t| t.num("cy")).unwrap_or(6_858_000.0));
    let proporcao = if cx / cy > 1.55 { Proporcao::Larga } else { Proporcao::Padrao };
    let (w, h) = proporcao.tamanho();
    let sx = f64::from(w) / cx;
    let sy = f64::from(h) / cy;
    let rels = pac.ligacoes("ppt/presentation.xml");
    let caminhos: Vec<String> = pres.filho("p:sldIdLst").map(|l| l.filhos("p:sldId").filter_map(|s| s.attr("r:id")).filter_map(|id| rels.get(id).cloned()).collect()).unwrap_or_default();
    let fontes = fontes::ler_fontes(&pres, &mut pac, &rels);
    let (titulo, autor, metadados) = crate::metadados_externos::pptx(&mut pac);
    let mut leitor = Leitor { pac, t: Transformacao { ax: sx, bx: 0.0, ay: sy, by: 0.0 }, escala: (sx.min(sy) * EMU_POR_PT) as f32, proximo: 1, temas: HashMap::new() };
    let mut slides = Vec::new();
    for c in caminhos {
        slides.push(leitor.slide(&c)?);
    }
    Ok(Apresentacao { titulo, autor, proporcao, dimensoes: [0.0; 2], slides, proximo_id: leitor.proximo, fontes, tema: String::new(), modelo: String::new(), metadados })
}

/// A primeira ligação cujo caminho contém `pasta` (o layout de um slide, o mestre de um layout…).
fn ligada(rels: &HashMap<String, String>, pasta: &str) -> Option<String> {
    let mut v: Vec<&String> = rels.values().filter(|c| c.contains(pasta)).collect();
    v.sort();
    v.first().map(|c| (*c).clone())
}

/// O tipo e o índice de uma caixa de layout (`p:ph`), se a forma é uma.
fn ph_de(forma: &No) -> Option<(String, Option<String>)> {
    let nv = forma.filhos.iter().find(|f| f.nome.starts_with("p:nv"))?;
    let ph = nv.caminho(&["p:nvPr", "p:ph"])?;
    Some((ph.attr("type").unwrap_or("body").to_string(), ph.attr("idx").map(String::from)))
}

/// Título e título centralizado contam como o mesmo tipo na hora de casar com o layout.
fn mesmo_tipo(a: &str, b: &str) -> bool {
    fn norm(t: &str) -> &str {
        match t {
            "ctrTitle" => "title",
            "obj" => "body",
            _ => t,
        }
    }
    norm(a) == norm(b)
}

/// A caixa equivalente (`p:sp`) numa árvore de layout ou mestre: pelo índice, senão pelo tipo.
fn caixa_equivalente<'b>(arvore: &'b No, tipo: &str, idx: Option<&str>) -> Option<&'b No> {
    let mut formas = Vec::new();
    arvore.todos("p:sp", &mut formas);
    let com_ph: Vec<(&No, (String, Option<String>))> = formas.into_iter().filter_map(|f| ph_de(f).map(|p| (f, p))).collect();
    idx.and_then(|i| com_ph.iter().find(|(_, (_, j))| j.as_deref() == Some(i)))
        .or_else(|| com_ph.iter().find(|(_, (t, _))| mesmo_tipo(t, tipo)))
        .map(|(f, _)| *f)
}

/// O fundo: o do slide, senão o do layout, senão o do mestre.
fn fundo(arvores: &[Option<&No>], tema: &Tema) -> String {
    for a in arvores.iter().flatten() {
        let Some(bg) = a.caminho(&["p:cSld", "p:bg"]) else { continue };
        if let Some(c) = bg.filho("p:bgPr").and_then(|p| tema.preenchimento(p)).filter(|c| !c.is_empty()) {
            return c;
        }
        if let Some(c) = bg.filho("p:bgRef").and_then(|r| tema.cor(r)) {
            return c;
        }
    }
    String::new()
}

/// O nome do layout do FeatherSlide que mais se parece com o tipo do layout do PowerPoint.
fn nome_do_layout(layout: Option<&No>) -> &'static str {
    match layout.and_then(|l| l.attr("type")) {
        Some("title") => "titulo",
        Some("obj" | "tx") => "titulo_conteudo",
        Some("secHead") => "secao",
        Some("twoObj" | "twoTxTwoObj") => "dois_conteudos",
        Some("titleOnly") => "somente_titulo",
        _ => "em_branco",
    }
}

/// O tipo do FeatherSlide para uma geometria pronta do PowerPoint.
fn tipo_da_forma(prst: &str, preenchida: bool) -> Tipo {
    match prst {
        "line" | "straightConnector1" | "bentConnector2" | "bentConnector3" | "curvedConnector3" => Tipo::Linha,
        "ellipse" | "circle" => Tipo::Elipse,
        "pie" => Tipo::Setor,
        "roundRect" | "round2SameRect" | "snipRoundRect" => Tipo::RetanguloArredondado,
        "triangle" | "rtTriangle" => Tipo::Triangulo,
        "rightArrow" | "leftArrow" | "notchedRightArrow" | "stripedRightArrow" => Tipo::Seta,
        _ if preenchida => Tipo::Retangulo,
        _ => Tipo::Texto,
    }
}

/// O valor de um ajuste da geometria (`<a:gd name="adj1" fmla="val 50000"/>`).
fn ajuste(geom: Option<&No>, nome: &str, padrao: f64) -> f64 {
    geom.and_then(|g| g.filho("a:avLst"))
        .and_then(|av| av.filhos("a:gd").find(|gd| gd.attr("name") == Some(nome)))
        .and_then(|gd| gd.attr("fmla"))
        .and_then(|f| f.strip_prefix("val "))
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(padrao)
}

/// Os ajustes do FeatherSlide (`Elemento::ajustes`) para uma geometria pronta do PowerPoint.
fn ajustes_da_forma(prst: &str, geom: Option<&No>, xfrm: &No) -> Vec<f32> {
    let fracao = |nome: &str, padrao: f64| (ajuste(geom, nome, padrao) / 100_000.0) as f32;
    match prst {
        // Ângulos em 60 000 avos de grau, horário a partir das 3 horas (como o canvas).
        "pie" => vec![(ajuste(geom, "adj1", 0.0) / 60_000.0) as f32, (ajuste(geom, "adj2", 16_200_000.0) / 60_000.0) as f32],
        "roundRect" => vec![fracao("adj", 16_667.0); 2],
        "round2SameRect" => {
            let (cima, baixo) = (fracao("adj1", 16_667.0), fracao("adj2", 0.0));
            if xfrm.attr("flipV") == Some("1") {
                vec![baixo, cima]
            } else {
                vec![cima, baixo]
            }
        }
        _ => Vec::new(),
    }
}

/// O tipo e os ajustes de uma forma que recorta imagem (a foto redonda é `ellipse`).
fn recorte_da_imagem(prst: &str) -> Tipo {
    match prst {
        "ellipse" | "circle" => Tipo::Elipse,
        "roundRect" | "round2SameRect" | "round1Rect" | "round2DiagRect" | "flowChartAlternateProcess" => Tipo::RetanguloArredondado,
        "triangle" | "isoscelesTriangle" | "rtTriangle" => Tipo::Triangulo,
        "pie" => Tipo::Setor,
        "rightArrow" | "leftArrow" | "upArrow" | "downArrow" => Tipo::Seta,
        _ => Tipo::Retangulo,
    }
}

/// Aplica o corte de `a:srcRect` (porcentagens l, t, r, b em 1/1000 de %) aos bytes da imagem.
fn cortar_imagem_src_rect(bytes: &[u8], src_rect: &No) -> Option<(Vec<u8>, &'static str)> {
    let l = src_rect.num("l").unwrap_or(0.0).max(0.0) / 100_000.0;
    let t = src_rect.num("t").unwrap_or(0.0).max(0.0) / 100_000.0;
    let r = src_rect.num("r").unwrap_or(0.0).max(0.0) / 100_000.0;
    let b = src_rect.num("b").unwrap_or(0.0).max(0.0) / 100_000.0;
    if l <= 0.0005 && t <= 0.0005 && r <= 0.0005 && b <= 0.0005 {
        return None;
    }
    let img = image::load_from_memory(bytes).ok()?;
    let iw = img.width() as f64;
    let ih = img.height() as f64;
    let x0 = ((l * iw).round() as u32).min(img.width().saturating_sub(1));
    let y0 = ((t * ih).round() as u32).min(img.height().saturating_sub(1));
    let x1 = (((1.0 - r) * iw).round() as u32).clamp(x0 + 1, img.width());
    let y1 = (((1.0 - b) * ih).round() as u32).clamp(y0 + 1, img.height());
    let w = (x1 - x0).max(1);
    let h = (y1 - y0).max(1);
    let cropped = img.crop_imm(x0, y0, w, h);
    let mut buf = std::io::Cursor::new(Vec::new());
    cropped.write_to(&mut buf, image::ImageFormat::Png).ok()?;
    Some((buf.into_inner(), "image/png"))
}

impl Leitor<'_> {
    /// A sombra externa (`a:effectLst/a:outerShdw`) de um `p:spPr`.
    fn sombra(&self, sp_pr: Option<&No>, tema: &Tema) -> Sombra {
        let Some(s) = sp_pr.and_then(|p| p.caminho(&["a:effectLst", "a:outerShdw"])) else { return Sombra::default() };
        let pt = |emu: f64| (emu / EMU_POR_PT) as f32 * self.escala;
        let dist = pt(s.num("dist").unwrap_or(0.0));
        let dir = (s.num("dir").unwrap_or(0.0) / 60_000.0).to_radians() as f32;
        Sombra { cor: tema.cor(s).unwrap_or_else(|| "#00000080".into()), desfoque: pt(s.num("blurRad").unwrap_or(0.0)), dx: dist * dir.cos(), dy: dist * dir.sin() }
    }

    fn novo_id(&mut self) -> u64 {
        self.proximo += 1;
        self.proximo - 1
    }

    /// O tema de um mestre (lido uma vez por mestre).
    fn tema(&mut self, mestre: &str, arvore: Option<&No>) -> Rc<Tema> {
        if let Some(t) = self.temas.get(mestre) {
            return t.clone();
        }
        let rels = self.pac.ligacoes(mestre);
        let tema = ligada(&rels, "theme/").and_then(|c| self.pac.xml(&c).ok());
        let t = Rc::new(Tema::novo(tema.as_deref(), arvore.and_then(|m| m.filho("p:clrMap"))));
        self.temas.insert(mestre.to_string(), t.clone());
        t
    }

    fn contexto(&mut self, caminho: &str) -> Contexto {
        let ligacoes = self.pac.ligacoes(caminho);
        let layout_c = ligada(&ligacoes, "slideLayouts/");
        let layout = layout_c.as_deref().and_then(|c| self.pac.xml(c).ok());
        let mestre_c = layout_c.as_ref().map(|c| self.pac.ligacoes(c)).and_then(|r| ligada(&r, "slideMasters/"));
        let mestre = mestre_c.as_deref().and_then(|c| self.pac.xml(c).ok());
        let tema = self.tema(mestre_c.as_deref().unwrap_or(""), mestre.as_deref());
        Contexto { layout, mestre, tema, ligacoes, layout_c, mestre_c }
    }

    fn slide(&mut self, caminho: &str) -> Result<Slide, String> {
        let arvore = self.pac.xml(caminho)?;
        let ctx = self.contexto(caminho);
        let mut elementos = Vec::new();
        let t = self.t;
        // As formas do mestre e do layout ficam atrás das do slide (`showMasterSp="0"` esconde).
        let mostra = |a: Option<&No>| a.map_or(true, |n| n.attr("showMasterSp") != Some("0"));
        if mostra(Some(&arvore)) {
            let mestre = if mostra(ctx.layout.as_deref()) { ctx.mestre.clone().zip(ctx.mestre_c.clone()) } else { None };
            for (fundo, c) in [mestre, ctx.layout.clone().zip(ctx.layout_c.clone())].into_iter().flatten() {
                let ligacoes = self.pac.ligacoes(&c);
                if let Some(raiz) = fundo.caminho(&["p:cSld", "p:spTree"]) {
                    self.formas(raiz, t, &ctx, Origem { ligacoes: &ligacoes, fundo: true }, &mut elementos)?;
                }
            }
        }
        if let Some(raiz) = arvore.caminho(&["p:cSld", "p:spTree"]) {
            self.formas(raiz, t, &ctx, Origem { ligacoes: &ctx.ligacoes, fundo: false }, &mut elementos)?;
        }
        let notas = self.notas(&ctx.ligacoes);
        Ok(Slide {
            id: self.novo_id(),
            layout: nome_do_layout(ctx.layout.as_deref()).into(),
            fundo: fundo(&[Some(&arvore), ctx.layout.as_deref(), ctx.mestre.as_deref()], &ctx.tema),
            fundo_config: Default::default(),
            ocultar_mestre: false,
            elementos,
            notas,
            oculto: arvore.attr("show") == Some("0"),
            transicao: Default::default(),
            secao: String::new(),
            comentarios: Vec::new(),
        })
    }

    /// As formas de uma árvore de formas (o slide ou um grupo), em ordem de pintura.
    fn formas(&mut self, raiz: &No, t: Transformacao, ctx: &Contexto, o: Origem, saida: &mut Vec<Elemento>) -> Result<(), String> {
        for f in &raiz.filhos {
            // No fundo, as caixas de layout (título, corpo…) são moldes, não conteúdo.
            if o.fundo && ph_de(f).is_some() {
                continue;
            }
            match f.nome.as_str() {
                "p:sp" if f.caminho(&["p:spPr", "a:blipFill", "a:blip"]).is_some() => saida.extend(self.imagem(f, t, ctx, o)?),
                "p:sp" | "p:cxnSp" => saida.extend(self.forma(f, t, ctx, o)),
                "p:pic" => saida.extend(self.imagem(f, t, ctx, o)?),
                "p:grpSp" => {
                    let interna = f.caminho(&["p:grpSpPr", "a:xfrm"]).map_or(t, |x| t.do_grupo(x));
                    self.formas(f, interna, ctx, o, saida)?;
                }
                "mc:AlternateContent" => {
                    // A escolha (`mc:Choice`) costuma pedir um recurso novo (equação, ink);
                    // o `mc:Fallback` é o que todo leitor entende — muitas vezes uma imagem.
                    if let Some(escolha) = f.filho("mc:Fallback").or_else(|| f.filho("mc:Choice")) {
                        self.formas(escolha, t, ctx, o, saida)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Uma forma (caixa de texto, forma pronta, linha), com a herança do layout e do mestre.
    fn forma(&mut self, f: &No, t: Transformacao, ctx: &Contexto, o: Origem) -> Option<Elemento> {
        let ph = ph_de(f);
        let (tipo_ph, idx) = ph.clone().map_or((None, None), |(a, b)| (Some(a), b));
        let no_layout = tipo_ph.as_deref().and_then(|tp| ctx.layout.as_deref().and_then(|l| caixa_equivalente(l, tp, idx.as_deref())));
        let no_mestre = tipo_ph.as_deref().and_then(|tp| ctx.mestre.as_deref().and_then(|m| caixa_equivalente(m, tp, None)));
        let sp_pr = f.filho("p:spPr");
        let xfrm = [Some(f), no_layout, no_mestre].into_iter().flatten().find_map(|n| n.caminho(&["p:spPr", "a:xfrm"]).filter(|x| x.filho("a:off").is_some()))?;
        let [mut x, mut y, mut w, mut h] = t.retangulo(xfrm)?;
        let tema = &ctx.tema;
        let preenchimento = sp_pr.and_then(|s| tema.preenchimento(s)).or_else(|| f.caminho(&["p:style", "a:fillRef"]).filter(|r| r.attr("idx") != Some("0")).and_then(|r| tema.cor(r))).unwrap_or_default();
        let ln = sp_pr.and_then(|s| s.filho("a:ln"));
        let contorno = ln.and_then(|l| tema.preenchimento(l)).or_else(|| f.caminho(&["p:style", "a:lnRef"]).filter(|r| r.attr("idx") != Some("0")).and_then(|r| tema.cor(r))).unwrap_or_default();
        let espessura = ln.and_then(|l| l.num("w")).map_or(1.0, |e| (e / EMU_POR_PT) as f32 * self.escala).max(0.25);
        let geom = sp_pr.and_then(|s| s.filho("a:prstGeom"));
        let prst = geom.and_then(|g| g.attr("prst")).unwrap_or("rect");
        let tipo = tipo_da_forma(prst, !preenchimento.is_empty() || !contorno.is_empty());
        if tipo == Tipo::Linha {
            if xfrm.attr("flipH") == Some("1") {
                (x, w) = (x + w, -w);
            }
            if xfrm.attr("flipV") == Some("1") {
                (y, h) = (y + h, -h);
            }
        }
        let corpo = f.filho("p:txBody");
        let estilo_mestre = ctx.mestre.as_deref().and_then(|m| m.filho("p:txStyles")).and_then(|ts| match tipo_ph.as_deref() {
            Some("title" | "ctrTitle") => ts.filho("p:titleStyle"),
            Some(_) => ts.filho("p:bodyStyle"),
            None => ts.filho("p:otherStyle"),
        });
        let heranca = Heranca { proprio: corpo, layout: no_layout.and_then(|n| n.filho("p:txBody")), mestre: no_mestre.and_then(|n| n.filho("p:txBody")), estilo_mestre };
        let texto = corpo.map(|c| texto::texto(c, &heranca)).unwrap_or_default();
        let papel = match tipo_ph.as_deref() {
            _ if o.fundo => Papel::Fundo,
            Some("title" | "ctrTitle") => Papel::Titulo,
            Some("subTitle") => Papel::Subtitulo,
            Some("body" | "obj") => Papel::Corpo,
            Some(_) if texto.is_empty() => return None,
            _ => Papel::Livre,
        };
        if tipo == Tipo::Texto && texto.is_empty() && !papel.do_layout() {
            return None;
        }
        Some(Elemento {
            id: self.novo_id(),
            tipo,
            papel,
            x,
            y,
            largura: w,
            altura: h,
            rotacao: xfrm.num("rot").map_or(0.0, |r| (r / 60_000.0) as f32),
            preenchimento,
            contorno,
            espessura,
            texto,
            estilo: texto::estilo(corpo, &heranca, tema, self.escala),
            ajustes: ajustes_da_forma(prst, geom, xfrm),
            sombra: self.sombra(sp_pr, tema),
            ..Elemento::default()
        })
    }

    /// Uma imagem (`p:pic`, ou `p:sp` preenchida com imagem): os bytes vêm da ligação e vão
    /// para a apresentação como `data:`. A geometria da forma recorta a imagem.
    fn imagem(&mut self, f: &No, t: Transformacao, ctx: &Contexto, o: Origem) -> Result<Option<Elemento>, String> {
        let sp_pr = f.filho("p:spPr");
        let Some(xfrm) = sp_pr.and_then(|s| s.filho("a:xfrm")) else { return Ok(None) };
        let Some([x, y, w, h]) = t.retangulo(xfrm) else { return Ok(None) };
        let blip_fill = f.filho("p:blipFill").or_else(|| f.caminho(&["p:spPr", "a:blipFill"]));
        let blip = blip_fill.and_then(|bf| bf.filho("a:blip"));
        let Some(alvo) = blip.and_then(|b| b.attr("r:embed")).and_then(|id| o.ligacoes.get(id)) else { return Ok(None) };
        if !self.pac.tem(alvo) {
            return Ok(None);
        }
        let mut bytes = self.pac.parte(alvo)?;
        let mut tipo = match alvo.rsplit('.').next().map(str::to_ascii_lowercase).as_deref() {
            Some("jpg" | "jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("bmp") => "image/bmp",
            Some("webp") => "image/webp",
            _ => "image/png",
        };
        if let Some(src_rect) = blip_fill.and_then(|bf| bf.filho("a:srcRect")) {
            if let Some((cortados, novo_tipo)) = cortar_imagem_src_rect(&bytes, src_rect) {
                bytes = cortados;
                tipo = novo_tipo;
            }
        }
        let geom = sp_pr.and_then(|s| s.filho("a:prstGeom"));
        let prst = geom.and_then(|g| g.attr("prst")).unwrap_or("rect");
        Ok(Some(Elemento {
            id: self.novo_id(),
            tipo: Tipo::Imagem,
            papel: if o.fundo { Papel::Fundo } else { Papel::Livre },
            x,
            y,
            largura: w,
            altura: h,
            rotacao: xfrm.num("rot").map_or(0.0, |r| (r / 60_000.0) as f32),
            imagem: base64::para_data_url(tipo, &bytes).into(),
            recorte: recorte_da_imagem(prst),
            ajustes: ajustes_da_forma(prst, geom, xfrm),
            sombra: self.sombra(sp_pr, &ctx.tema),
            ..Elemento::default()
        }))
    }

    /// As anotações do apresentador (a caixa de corpo da página de anotações).
    fn notas(&mut self, ligacoes: &HashMap<String, String>) -> String {
        let Some(c) = ligada(ligacoes, "notesSlides/") else { return String::new() };
        let Ok(arvore) = self.pac.xml(&c) else { return String::new() };
        let mut formas = Vec::new();
        arvore.todos("p:sp", &mut formas);
        let corpo = formas.into_iter().find(|f| ph_de(f).is_some_and(|(t, _)| t == "body"));
        let heranca = Heranca { proprio: None, layout: None, mestre: None, estilo_mestre: None };
        corpo.and_then(|f| f.filho("p:txBody")).map(|tx| texto::texto(tx, &heranca)).unwrap_or_default()
    }
}
