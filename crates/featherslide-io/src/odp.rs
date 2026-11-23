//! Ler `.odp` (LibreOffice Impress, OpenDocument). Rust puro: o ZIP, o XML em árvore
//! (`crate::xml`) e a cadeia de estilos do ODF — o estilo do trecho → o do parágrafo → o da
//! caixa → os pais dele (`style:parent-style-name`), procurando nos estilos automáticos do
//! `content.xml` e nos do `styles.xml`.
//!
//! No `.odp` cada caixa já vem com a posição no slide (não há herança de posição como no
//! `.pptx`). Entram: caixas de texto e de layout (título, subtítulo, tópicos), formas
//! (retângulo, arredondado, elipse, triângulo, seta), linhas, imagens, grupos, fundo sólido,
//! slide oculto e anotações. Ficam de fora, por ora: tabelas, gráficos, fundo com imagem ou
//! gradiente e estilo diferente por trecho (vale o predominante).

use std::collections::HashMap;

use featherslide_core::modelo::{Alinhamento, Apresentacao, Elemento, EstiloDoTexto, Papel, Proporcao, Slide, Tipo, Vertical};

use crate::base64;
use crate::pacote::Pacote;
use crate::xml::{self, No};

/// Uma medida do ODF (`2.54cm`, `1in`, `12pt`, `20mm`) em pontos.
pub fn medida(v: &str) -> Option<f64> {
    let v = v.trim();
    let corte = v.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(v.len());
    let (n, unidade) = v.split_at(corte);
    let n: f64 = n.trim().parse().ok()?;
    Some(match unidade {
        "cm" => n * 72.0 / 2.54,
        "mm" => n * 72.0 / 25.4,
        "in" => n * 72.0,
        "pc" => n * 12.0,
        "px" => n * 0.75,
        _ => n,
    })
}

/// Os estilos com nome, dos dois arquivos, e a família de cada um não importa: o nome basta.
struct Estilos {
    por_nome: HashMap<String, No>,
}

impl Estilos {
    fn juntar(arvores: &[&No]) -> Self {
        let mut por_nome = HashMap::new();
        for a in arvores {
            for secao in ["office:automatic-styles", "office:styles", "office:master-styles"] {
                let Some(s) = a.filho(secao) else { continue };
                for e in s.filhos.iter().filter(|e| e.nome == "style:style" || e.nome == "style:page-layout") {
                    if let Some(n) = e.attr("style:name") {
                        por_nome.entry(n.to_string()).or_insert_with(|| e.clone());
                    }
                }
            }
        }
        Self { por_nome }
    }

    /// Procura o atributo `attr` no grupo de propriedades `grupo` do estilo e dos pais dele.
    fn propriedade(&self, estilo: Option<&str>, grupo: &str, attr: &str) -> Option<String> {
        let mut atual = estilo;
        for _ in 0..16 {
            let e = self.por_nome.get(atual?)?;
            if let Some(v) = e.filho(grupo).and_then(|p| p.attr(attr)) {
                return Some(v.to_string());
            }
            atual = e.attr("style:parent-style-name");
        }
        None
    }

    /// A primeira resposta, olhando os estilos na ordem dada (do mais próximo ao mais geral).
    fn primeiro(&self, estilos: &[Option<&str>], grupo: &str, attr: &str) -> Option<String> {
        estilos.iter().find_map(|e| self.propriedade(*e, grupo, attr))
    }
}

struct Leitor<'a> {
    pac: Pacote<'a>,
    estilos: Estilos,
    /// Pontos do `.odp` → pontos do slide.
    sx: f64,
    sy: f64,
    proximo: u64,
    /// O nome de cada slide (`draw:name`), na ordem: o clique "ir ao slide" (`#nome`) vira o número dele.
    paginas: Vec<String>,
}

/// Lê um `.odp`. Arquivo malformado dá erro em português, nunca pânico.
pub fn ler(bytes: &[u8]) -> Result<Apresentacao, String> {
    let mut pac = Pacote::abrir(bytes, ".odp")?;
    let conteudo = pac.xml("content.xml")?;
    let (titulo, autor, metadados) = crate::metadados_externos::odp(&mut pac);
    let estilos_xml = pac.xml("styles.xml").ok();
    let mut arvores: Vec<&No> = vec![&conteudo];
    if let Some(e) = estilos_xml.as_deref() {
        arvores.push(e);
    }
    let estilos = Estilos::juntar(&arvores);
    let mut pagina = None;
    if let Some(e) = estilos_xml.as_deref() {
        let mut v = Vec::new();
        e.todos("style:page-layout-properties", &mut v);
        pagina = v.first().and_then(|p| Some((medida(p.attr("fo:page-width")?)?, medida(p.attr("fo:page-height")?)?)));
    }
    let (pw, ph) = pagina.unwrap_or((792.0, 445.5));
    let (proporcao, dimensoes) = Proporcao::de_tamanho(pw, ph);
    let (w, h) = if dimensoes[0] > 0.0 { (dimensoes[0], dimensoes[1]) } else { proporcao.tamanho() };
    let apresentacao = conteudo.caminho(&["office:body", "office:presentation"]).ok_or("O .odp não tem apresentação (falta office:presentation).")?;
    let mut leitor = Leitor { pac, estilos, sx: f64::from(w) / pw, sy: f64::from(h) / ph, proximo: 1, paginas: apresentacao.filhos("draw:page").map(|p| p.attr("draw:name").unwrap_or("").to_string()).collect() };
    let mut slides = Vec::new();
    let mestres = estilos_xml.as_deref().and_then(|e| e.filho("office:master-styles"));
    for p in apresentacao.filhos("draw:page") {
        slides.push(leitor.slide(p, mestres)?);
    }
    Ok(Apresentacao { titulo, autor, proporcao, dimensoes, slides, proximo_id: leitor.proximo, fontes: Vec::new(), tema: String::new(), modelo: String::new(), metadados, rodape: Default::default() })
}

/// O `draw:transform` da forma (`rotate (a) translate (x y)`): o ângulo em radianos (anti-horário) e o
/// deslocamento em pontos. Sem `translate`, `None` (escala e inclinação ficam de fora).
fn transformacao(f: &No) -> Option<(f64, f64, f64)> {
    let t = f.attr("draw:transform")?;
    let dentro = |nome: &str| t.find(nome).and_then(|i| { let r = &t[i + nome.len()..]; let a = r.find('(')?; let b = r.find(')')?; Some(r[a + 1..b].to_string()) });
    let a = dentro("rotate").and_then(|v| v.trim().parse::<f64>().ok()).unwrap_or(0.0);
    let d = dentro("translate")?;
    let mut partes = d.split(|c: char| c == ',' || c.is_whitespace()).filter(|p| !p.is_empty());
    Some((a, medida(partes.next()?)?, medida(partes.next().unwrap_or("0"))?))
}

/// As linhas de uma `table:table` (também as de `table:table-header-rows`), com as células repetidas
/// (`table:number-columns-repeated`) abertas; a célula coberta por uma mesclagem fica vazia.
fn grade_da_tabela(t: &No) -> Vec<Vec<String>> {
    let mut grade = Vec::new();
    for f in &t.filhos {
        match f.nome.as_str() {
            "table:table-header-rows" | "table:table-rows" => grade.extend(grade_da_tabela(f)),
            "table:table-row" => {
                let mut linha = Vec::new();
                for c in f.filhos.iter().filter(|c| c.nome == "table:table-cell" || c.nome == "table:covered-table-cell") {
                    let texto = if c.nome == "table:table-cell" { c.filhos("text:p").map(texto_do_paragrafo).collect::<Vec<_>>().join("\n") } else { String::new() };
                    let vezes = c.attr("table:number-columns-repeated").and_then(|n| n.parse::<usize>().ok()).unwrap_or(1).clamp(1, 64);
                    linha.extend(std::iter::repeat_n(texto, vezes));
                }
                grade.push(linha);
            }
            _ => {}
        }
    }
    grade
}

/// O texto de um parágrafo (`text:p`, `text:h`): trechos, espaços (`text:s`), tabulações e quebras.
fn texto_do_paragrafo(p: &No) -> String {
    let mut s = String::new();
    for f in &p.filhos {
        match f.nome.as_str() {
            xml::TEXTO => s.push_str(&f.texto),
            "text:span" | "text:a" => s.push_str(&texto_do_paragrafo(f)),
            // Teto: `text:c="1e12"` num arquivo de poucos kB pedia memória sem fim.
            "text:s" => s.push_str(&" ".repeat(f.num("text:c").map_or(1, |n| n.clamp(0.0, 1000.0) as usize))),
            "text:tab" => s.push('\t'),
            "text:line-break" => s.push('\n'),
            _ => {}
        }
    }
    s
}

/// As linhas de texto de uma caixa (parágrafos e listas, com "•" nos itens).
fn linhas(no: &No, nivel: usize, saida: &mut Vec<String>) {
    for f in &no.filhos {
        match f.nome.as_str() {
            "text:p" | "text:h" if nivel > 0 => saida.push(format!("{}• {}", "    ".repeat(nivel - 1), texto_do_paragrafo(f))),
            "text:p" | "text:h" => saida.push(texto_do_paragrafo(f)),
            "text:list" => linhas(f, nivel + 1, saida),
            "text:list-item" | "text:list-header" => linhas(f, nivel, saida),
            _ => {}
        }
    }
}

fn texto_da_caixa(caixa: &No) -> String {
    let mut v = Vec::new();
    linhas(caixa, 0, &mut v);
    while v.last().is_some_and(|l| l.trim().is_empty()) {
        v.pop();
    }
    v.join("\n")
}

/// Os trechos com texto de uma caixa: (estilo do trecho, estilo do parágrafo, letras).
fn trechos<'b>(no: &'b No, paragrafo: Option<&'b str>, saida: &mut Vec<(Option<&'b str>, Option<&'b str>, usize)>) {
    let par = if matches!(no.nome.as_str(), "text:p" | "text:h") { no.attr("text:style-name") } else { paragrafo };
    let letras = no.texto.trim().chars().count();
    if letras > 0 {
        let span = if no.nome == "text:span" { no.attr("text:style-name") } else { None };
        saida.push((span, par, letras));
    }
    for f in &no.filhos {
        trechos(f, par, saida);
    }
}

fn cor(v: Option<String>) -> Option<String> {
    v.filter(|c| c.starts_with('#') && c.len() == 7).map(|c| c.to_lowercase())
}

/// Segundos de um tempo do ODF: `2s`, `0.5s`, `PT5S` ou `PT00H01M05S`.
fn segundos(v: &str) -> Option<f32> {
    let v = v.trim();
    if let Some(iso) = v.strip_prefix("PT") {
        let (mut total, mut numero) = (0.0_f32, String::new());
        for c in iso.chars() {
            match c {
                'H' | 'M' | 'S' => {
                    let n: f32 = numero.parse().ok()?;
                    total += n * match c { 'H' => 3600.0, 'M' => 60.0, _ => 1.0 };
                    numero.clear();
                }
                c => numero.push(c),
            }
        }
        return Some(total);
    }
    v.strip_suffix('s').unwrap_or(v).parse().ok()
}

/// A transição do slide pelo `anim:transitionFilter` que o Impress grava na página.
fn transicao(filtro: Option<&No>, apos: f32) -> featherslide_core::modelo::Transicao {
    use featherslide_core::modelo::{Direcao, TipoTransicao, Transicao};
    let padrao = Transicao { apos: apos.clamp(0.0, 86_400.0), ..Transicao::default() };
    let Some(f) = filtro else { return padrao };
    let sub = f.attr("smil:subtype").unwrap_or("");
    let ao_contrario = f.attr("smil:direction") == Some("reverse");
    // `fromLeft` = vem da esquerda; o `barWipe` diz para onde varre (`leftToRight` = vem da esquerda).
    let mut direcao = if sub.starts_with("fromLeft") || sub == "leftToRight" || sub == "fromTopLeft" || sub == "fromBottomLeft" { Direcao::Esquerda }
        else if sub.starts_with("fromTop") || sub == "topToBottom" { Direcao::Cima }
        else if sub.starts_with("fromBottom") { Direcao::Baixo }
        else { Direcao::Direita };
    let tipo = match f.attr("smil:type") {
        Some("fade") => TipoTransicao::Esmaecer,
        Some("pushWipe") => TipoTransicao::Empurrar,
        Some("slideWipe") if ao_contrario => TipoTransicao::Revelar,
        Some("slideWipe") => TipoTransicao::Deslizar,
        Some("barWipe") => {
            if ao_contrario {
                direcao = match direcao { Direcao::Esquerda => Direcao::Direita, Direcao::Direita => Direcao::Esquerda, Direcao::Cima => Direcao::Baixo, Direcao::Baixo => Direcao::Cima };
            }
            TipoTransicao::Cortina
        }
        // shortcut: os efeitos do Impress que ainda não temos tocam como esmaecer; mapear um a um quando houver o efeito.
        _ => TipoTransicao::Esmaecer,
    };
    let duracao = f.attr("smil:dur").and_then(segundos).unwrap_or(0.7).clamp(0.05, 10.0);
    Transicao { tipo, direcao, duracao, apos: padrao.apos }
}

impl Leitor<'_> {
    fn novo_id(&mut self) -> u64 {
        self.proximo += 1;
        self.proximo - 1
    }

    fn slide(&mut self, p: &No, mestres: Option<&No>) -> Result<Slide, String> {
        let mestre = p.attr("draw:master-page-name").and_then(|n| mestres?.filhos("style:master-page").find(|m| m.attr("style:name") == Some(n)));
        let fundo_de = |estilo: Option<&str>| -> Option<String> {
            let e = &self.estilos;
            (e.propriedade(estilo, "style:drawing-page-properties", "draw:fill").as_deref() == Some("solid"))
                .then(|| cor(e.propriedade(estilo, "style:drawing-page-properties", "draw:fill-color")))
                .flatten()
        };
        let fundo = fundo_de(p.attr("draw:style-name")).or_else(|| fundo_de(mestre.and_then(|m| m.attr("draw:style-name")))).unwrap_or_default();
        let oculto = self.estilos.propriedade(p.attr("draw:style-name"), "style:drawing-page-properties", "presentation:visibility").as_deref() == Some("hidden");
        let mut elementos = Vec::new();
        self.formas(p, &mut elementos)?;
        let notas = p.filho("presentation:notes").map(|n| {
            let mut v = Vec::new();
            n.todos("draw:frame", &mut v);
            v.into_iter().filter(|f| f.attr("presentation:class") == Some("notes")).filter_map(|f| f.filho("draw:text-box")).map(texto_da_caixa).collect::<Vec<_>>().join("\n")
        });
        let layout = match p.attr("presentation:presentation-page-layout-name") {
            Some(n) if n.contains("Title") && n.contains("Content") => "titulo_conteudo",
            _ => "em_branco",
        };
        let estilo = |attr: &str| self.estilos.propriedade(p.attr("draw:style-name"), "style:drawing-page-properties", attr);
        let apos = (estilo("presentation:transition-type").as_deref() == Some("automatic"))
            .then(|| estilo("presentation:duration").as_deref().and_then(segundos)).flatten().unwrap_or(0.0);
        let transicao = transicao(p.busca("anim:transitionFilter"), apos);
        Ok(Slide { id: self.novo_id(), layout: layout.into(), fundo, fundo_config: Default::default(), ocultar_mestre: false, elementos, notas: notas.unwrap_or_default(), oculto, transicao, secao: String::new(), comentarios: Vec::new() })
    }

    fn formas(&mut self, pai: &No, saida: &mut Vec<Elemento>) -> Result<(), String> {
        for f in &pai.filhos {
            let antes = saida.len();
            match f.nome.as_str() {
                "draw:frame" => saida.extend(self.quadro(f)?),
                "draw:custom-shape" | "draw:rect" | "draw:ellipse" | "draw:line" | "draw:connector" => saida.extend(self.forma(f)),
                "draw:g" => self.formas(f, saida)?,
                _ => {}
            }
            if f.nome != "draw:g" {
                let (link, destino) = self.clique(f);
                let giro = transformacao(f).map_or(0.0, |(a, _, _)| -a.to_degrees() as f32);
                for e in &mut saida[antes..] {
                    e.link.clone_from(&link);
                    e.destino = destino;
                    if giro.abs() > 0.01 {
                        e.rotacao = giro.rem_euclid(360.0);
                    }
                }
            }
        }
        Ok(())
    }

    /// O clique da forma (`presentation:event-listener` com `action="show"`): o endereço do hiperlink,
    /// ou, com `#nome`, o número do slide a que leva.
    fn clique(&self, f: &No) -> (String, u32) {
        let ouvinte = f.filho("office:event-listeners").and_then(|l| {
            l.filhos("presentation:event-listener").find(|e| e.attr("script:event-name") == Some("dom:click") && e.attr("presentation:action") == Some("show"))
        });
        let Some(alvo) = ouvinte.and_then(|o| o.attr("xlink:href")).map(str::trim).filter(|a| !a.is_empty()) else { return (String::new(), 0) };
        match alvo.strip_prefix('#') {
            Some(nome) => {
                let nome = nome.replace("%20", " ");
                (String::new(), self.paginas.iter().position(|p| *p == nome).map_or(0, |i| i as u32 + 1))
            }
            None => (alvo.to_string(), 0),
        }
    }

    /// O retângulo em pontos do slide (`svg:x`…; na linha, os dois pontos).
    fn retangulo(&self, f: &No) -> Option<[f32; 4]> {
        let m = |a: &str| f.attr(a).and_then(medida);
        let (x, y, w, h) = if f.nome == "draw:line" || f.nome == "draw:connector" {
            let (x1, y1) = (m("svg:x1")?, m("svg:y1")?);
            (x1, y1, m("svg:x2")? - x1, m("svg:y2")? - y1)
        } else {
            let (w, h) = (m("svg:width").unwrap_or(0.0), m("svg:height").unwrap_or(0.0));
            match (m("svg:x"), m("svg:y"), transformacao(f)) {
                (Some(x), Some(y), None) => (x, y, w, h),
                (x, y, Some((a, tx, ty))) => {
                    // Girada (o LibreOffice tira o `svg:x`/`svg:y`): a caixa nasce na origem, gira `a`
                    // (anti-horário) em volta do canto e anda até (tx, ty). O centro dá a posição.
                    let (sen, cos) = a.sin_cos();
                    let (cx, cy) = (w / 2.0 * cos + h / 2.0 * sen + tx + x.unwrap_or(0.0), -w / 2.0 * sen + h / 2.0 * cos + ty + y.unwrap_or(0.0));
                    (cx - w / 2.0, cy - h / 2.0, w, h)
                }
                _ => return None,
            }
        };
        Some([(x * self.sx) as f32, (y * self.sy) as f32, (w * self.sx) as f32, (h * self.sy) as f32])
    }

    /// O estilo do texto da caixa: o predominante entre os trechos, com a cadeia de estilos.
    fn estilo(&self, caixa: Option<&No>, estilo_da_forma: &[Option<&str>]) -> EstiloDoTexto {
        let mut lista = Vec::new();
        if let Some(c) = caixa {
            trechos(c, None, &mut lista);
        }
        let (span, par) = lista.iter().max_by_key(|t| t.2).map_or((None, caixa.and_then(|c| c.busca("text:p")).and_then(|p| p.attr("text:style-name"))), |t| (t.0, t.1));
        let mut cadeia = vec![span, par];
        cadeia.extend_from_slice(estilo_da_forma);
        let e = &self.estilos;
        let txt = |a: &str| e.primeiro(&cadeia, "style:text-properties", a);
        let padrao = EstiloDoTexto::default();
        let escala = self.sx.min(self.sy) as f32;
        EstiloDoTexto {
            fonte: txt("style:font-name").or_else(|| txt("fo:font-family")).map(|f| f.trim_matches('\'').to_string()).unwrap_or(padrao.fonte),
            tamanho: txt("fo:font-size").and_then(|t| medida(&t)).map_or(18.0, |t| ((t as f32 * escala) * 10.0).round() / 10.0),
            cor: cor(txt("fo:color")).unwrap_or_else(|| "#000000".into()),
            negrito: txt("fo:font-weight").is_some_and(|w| w == "bold" || w.parse::<u32>().is_ok_and(|n| n >= 600)),
            italico: txt("fo:font-style").is_some_and(|s| s == "italic" || s == "oblique"),
            sublinhado: txt("style:text-underline-style").is_some_and(|u| u != "none"),
            tachado: txt("style:text-line-through-style").is_some_and(|s| s != "none"),
            sublinhado_duplo: txt("style:text-underline-type").is_some_and(|t| t == "double"),
            tachado_duplo: txt("style:text-line-through-type").is_some_and(|t| t == "double"),
            alinhamento: match e.primeiro(&cadeia, "style:paragraph-properties", "fo:text-align").as_deref() {
                Some("center") => Alinhamento::Centro,
                Some("end" | "right") => Alinhamento::Direita,
                Some("justify") => Alinhamento::Justificado,
                _ => Alinhamento::Esquerda,
            },
            vertical: match e.primeiro(estilo_da_forma, "style:graphic-properties", "draw:textarea-vertical-align").as_deref() {
                Some("middle") => Vertical::Meio,
                Some("bottom") => Vertical::Base,
                _ => Vertical::Topo,
            },
            ..padrao
        }
    }

    /// Preenchimento, contorno e espessura do estilo da forma.
    fn cores_da_forma(&self, estilos: &[Option<&str>]) -> (String, String, f32) {
        let e = &self.estilos;
        let g = |a: &str| e.primeiro(estilos, "style:graphic-properties", a);
        let preenchimento = if g("draw:fill").as_deref() == Some("solid") { cor(g("draw:fill-color")).unwrap_or_default() } else { String::new() };
        let contorno = if g("draw:stroke").as_deref().is_some_and(|s| s != "none") { cor(g("svg:stroke-color")).unwrap_or_else(|| "#000000".into()) } else { String::new() };
        let espessura = g("svg:stroke-width").and_then(|w| medida(&w)).map_or(1.0, |w| (w as f32 * self.sx.min(self.sy) as f32).max(0.25));
        (preenchimento, contorno, espessura)
    }

    /// A margem interna do estilo (`fo:padding-*`, ou `fo:padding` para os quatro lados), na escala
    /// do slide; `None` se o estilo não diz nada (fica a margem padrão).
    fn margem_da_forma(&self, estilos: &[Option<&str>]) -> Option<[f32; 4]> {
        use featherslide_core::quadro::{MARGEM_X, MARGEM_Y};
        let e = &self.estilos;
        let todos = e.primeiro(estilos, "style:graphic-properties", "fo:padding").and_then(|v| medida(&v));
        let lado = |nome: &str, escala: f64| {
            let v = e.primeiro(estilos, "style:graphic-properties", nome).and_then(|v| medida(&v)).or(todos)?;
            // O .fsdz aceita de 0 a 1000 pt.
            Some(((v * escala) as f32).clamp(0.0, 1000.0))
        };
        let lados = [lado("fo:padding-left", self.sx), lado("fo:padding-top", self.sy), lado("fo:padding-right", self.sx), lado("fo:padding-bottom", self.sy)];
        lados.iter().any(Option::is_some).then(|| {
            let [a, b, c, d] = lados;
            [a.unwrap_or(MARGEM_X), b.unwrap_or(MARGEM_Y), c.unwrap_or(MARGEM_X), d.unwrap_or(MARGEM_Y)]
        })
    }

    /// Uma `draw:frame`: caixa de texto (de layout ou não) ou imagem.
    fn quadro(&mut self, f: &No) -> Result<Option<Elemento>, String> {
        let Some([x, y, w, h]) = self.retangulo(f) else { return Ok(None) };
        let estilos = [f.attr("presentation:style-name"), f.attr("draw:style-name"), f.attr("draw:text-style-name")];
        // Tabela: a grade do `table:table`. A imagem ao lado é só a prévia (`.svm`), que não abre.
        if let Some(tabela) = f.filho("table:table") {
            let grade = grade_da_tabela(tabela);
            if grade.is_empty() {
                return Ok(None);
            }
            let mut e = self.elemento(Tipo::Tabela, Papel::Livre, [x, y, w, h]);
            e.dados = featherslide_core::dados::gravar_tabela(&grade);
            return Ok(Some(e));
        }
        if let Some(img) = f.filho("draw:image").and_then(|i| i.attr("xlink:href")).filter(|h| self.pac.tem(h)).map(String::from) {
            let bytes = self.pac.parte(&img)?;
            let tipo = if img.to_ascii_lowercase().ends_with(".png") { "image/png" } else if img.to_ascii_lowercase().ends_with(".svg") { "image/svg+xml" } else { "image/jpeg" };
            let mut e = self.elemento(Tipo::Imagem, Papel::Livre, [x, y, w, h]);
            e.imagem = base64::para_data_url(tipo, &bytes).into();
            return Ok(Some(e));
        }
        let caixa = f.filho("draw:text-box");
        let papel = match f.attr("presentation:class") {
            Some("title") => Papel::Titulo,
            Some("subtitle") => Papel::Subtitulo,
            Some("outline") => Papel::Corpo,
            Some("notes" | "page" | "header" | "footer" | "date-time" | "page-number") => return Ok(None),
            _ => Papel::Livre,
        };
        let texto = caixa.map(texto_da_caixa).unwrap_or_default();
        if texto.is_empty() && papel == Papel::Livre {
            return Ok(None);
        }
        let mut e = self.elemento(Tipo::Texto, papel, [x, y, w, h]);
        e.texto = texto;
        e.estilo = self.estilo(caixa, &estilos);
        (e.preenchimento, e.contorno, e.espessura) = self.cores_da_forma(&estilos);
        e.margem = self.margem_da_forma(&estilos);
        if !e.preenchimento.is_empty() || !e.contorno.is_empty() {
            e.tipo = Tipo::Retangulo;
        }
        Ok(Some(e))
    }

    /// Uma forma (`draw:custom-shape`, `draw:rect`, `draw:ellipse`, `draw:line`).
    fn forma(&mut self, f: &No) -> Option<Elemento> {
        let r = self.retangulo(f)?;
        let geometria = f.filho("draw:enhanced-geometry").and_then(|g| g.attr("draw:type")).unwrap_or("rectangle");
        let tipo = match (f.nome.as_str(), geometria) {
            ("draw:line" | "draw:connector", _) => Tipo::Linha,
            ("draw:ellipse", _) | (_, "ellipse" | "circle") => Tipo::Elipse,
            (_, "round-rectangle") => Tipo::RetanguloArredondado,
            // Forma que veio de um .pptx: o LibreOffice guarda o nome do PowerPoint (`ooxml-roundRect`).
            (_, g) if g.starts_with("ooxml-") => crate::pptx::tipo_da_forma(&g["ooxml-".len()..], true),
            (_, "isosceles-triangle" | "right-triangle") => Tipo::Triangulo,
            (_, "right-arrow" | "left-arrow") => Tipo::Seta,
            _ => Tipo::Retangulo,
        };
        let estilos = [f.attr("draw:style-name"), f.attr("draw:text-style-name"), None];
        let mut e = self.elemento(tipo, Papel::Livre, r);
        (e.preenchimento, e.contorno, e.espessura) = self.cores_da_forma(&estilos);
        if tipo == Tipo::Linha && e.contorno.is_empty() {
            e.contorno = "#000000".into();
        }
        e.texto = texto_da_caixa(f);
        e.estilo = self.estilo(Some(f), &estilos);
        e.margem = self.margem_da_forma(&estilos);
        Some(e)
    }

    fn elemento(&mut self, tipo: Tipo, papel: Papel, [x, y, w, h]: [f32; 4]) -> Elemento {
        Elemento {
            id: self.novo_id(),
            tipo,
            papel,
            x,
            y,
            largura: w,
            altura: h,
            ..Elemento::default()
        }
    }
}

#[cfg(test)]
mod testes {
    use super::medida;

    #[test]
    fn transicao_do_impress() {
        use featherslide_core::modelo::{Direcao, TipoTransicao};
        let filtro = |xml: &str| crate::xml::ler(xml.as_bytes(), "t").unwrap();
        // O que o Impress grava para "Empurrar de baixo, 2 s".
        let t = super::transicao(Some(&filtro(r#"<anim:transitionFilter smil:dur="2s" smil:type="pushWipe" smil:subtype="fromBottom"/>"#)), 5.0);
        assert_eq!((t.tipo, t.direcao, t.duracao, t.apos), (TipoTransicao::Empurrar, Direcao::Baixo, 2.0, 5.0));
        let t = super::transicao(Some(&filtro(r#"<anim:transitionFilter smil:type="slideWipe" smil:subtype="fromLeft" smil:direction="reverse"/>"#)), 0.0);
        assert_eq!((t.tipo, t.direcao), (TipoTransicao::Revelar, Direcao::Esquerda));
        let t = super::transicao(Some(&filtro(r#"<anim:transitionFilter smil:type="barWipe" smil:subtype="leftToRight" smil:direction="reverse"/>"#)), 0.0);
        assert_eq!((t.tipo, t.direcao), (TipoTransicao::Cortina, Direcao::Direita));
        assert_eq!(super::transicao(None, 3.0).tipo, TipoTransicao::Nenhuma);
        assert_eq!(super::segundos("PT00H01M05S"), Some(65.0));
        assert_eq!(super::segundos("PT5S"), Some(5.0));
    }

    #[test]
    fn espaco_repetido_tem_teto() {
        let p = crate::xml::ler(br#"<text:p>a<text:s text:c="1e12"/>b<text:s/></text:p>"#, "t").unwrap();
        assert_eq!(super::texto_do_paragrafo(&p).len(), 1003);
    }

    #[test]
    fn medidas_do_odf_em_pontos() {
        assert!((medida("2.54cm").unwrap() - 72.0).abs() < 1e-9);
        assert!((medida("1in").unwrap() - 72.0).abs() < 1e-9);
        assert_eq!(medida("12pt"), Some(12.0));
        assert!((medida("25.4mm").unwrap() - 72.0).abs() < 1e-9);
        assert_eq!(medida("abc"), None);
    }
}
