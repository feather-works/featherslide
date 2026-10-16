//! Tabela e gráfico de verdade: os dados (uma grade de textos, gravada em texto compacto no
//! `Elemento::dados`) e o desenho deles em **primitivas** (retângulos, linhas, setores e textos
//! já posicionados em pontos do slide). A interface só pinta as primitivas — a conta do
//! layout, das escalas e dos ângulos é toda daqui.

use serde::Serialize;

use crate::medida;
use crate::modelo::{Alinhamento, Elemento};

/// Uma grade de textos: linhas × colunas.
pub type Grade = Vec<Vec<String>>;

/// Uma peça do desenho de uma tabela ou de um gráfico, em pontos do slide.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(tag = "k")]
pub enum Primitiva {
    /// Curvas SVG preservadas: os segmentos são vetores, nunca uma imagem rasterizada.
    Vetor { segmentos: Vec<crate::vetores::Segmento>, par: bool, preenchimento: String, contorno: String, espessura: f32 },
    /// Retângulo; cor vazia = sem preenchimento / sem contorno.
    Ret { x: f32, y: f32, w: f32, h: f32, preenchimento: String, contorno: String, espessura: f32 },
    Linha { x1: f32, y1: f32, x2: f32, y2: f32, cor: String, espessura: f32 },
    /// Fatia de elipse (graus, horário a partir das 3 horas), centro e raio.
    Setor { cx: f32, cy: f32, r: f32, de: f32, ate: f32, preenchimento: String, contorno: String, espessura: f32 },
    /// Poligonal aberta.
    Poli { pontos: Vec<[f32; 2]>, cor: String, espessura: f32 },
    /// Polígono preenchido (os ícones).
    Poligono { pontos: Vec<[f32; 2]>, preenchimento: String },
    /// Caminho composto fechado; preserva os vazios internos das letras.
    Caminho { contornos: Vec<Vec<[f32; 2]>>, preenchimento: String, contorno: String, espessura: f32 },
    /// Texto ancorado em (`x`, `y`): `y` é o meio da linha; `alinh` diz se `x` é o começo
    /// (`e`), o centro (`c`) ou o fim (`d`) do texto.
    Texto { x: f32, y: f32, tamanho: f32, cor: String, negrito: bool, alinh: &'static str, texto: String },
}

// ------------------------------------------------------------------ texto compacto

fn escapar(c: &str) -> String {
    c.replace('\\', "\\\\").replace('\t', "\\t").replace('\n', "\\n")
}

fn desescapar(c: &str) -> String {
    let mut saida = String::new();
    let mut it = c.chars();
    while let Some(ch) = it.next() {
        if ch != '\\' {
            saida.push(ch);
            continue;
        }
        match it.next() {
            Some('t') => saida.push('\t'),
            Some('n') => saida.push('\n'),
            Some(outro) => saida.push(outro),
            None => {}
        }
    }
    saida
}

/// A grade em texto: linhas separadas por `\n`, células por `\t` (com escape).
pub fn codificar(grade: &Grade) -> String {
    grade.iter().map(|l| l.iter().map(|c| escapar(c)).collect::<Vec<_>>().join("\t")).collect::<Vec<_>>().join("\n")
}

pub fn decodificar(texto: &str) -> Grade {
    if texto.is_empty() {
        return Vec::new();
    }
    texto.split('\n').map(|l| l.split('\t').map(desescapar).collect()).collect()
}

/// Deixa a grade retangular (todas as linhas com o mesmo número de colunas) e com ao menos 1×1.
fn retangular(mut g: Grade) -> Grade {
    if g.is_empty() {
        g.push(vec![String::new()]);
    }
    let colunas = g.iter().map(Vec::len).max().unwrap_or(1).max(1);
    for l in &mut g {
        l.resize(colunas, String::new());
    }
    g
}

pub fn ler_tabela(e: &Elemento) -> Grade {
    retangular(decodificar(&e.dados))
}

pub fn gravar_tabela(g: &Grade) -> String {
    codificar(g)
}

/// O gráfico: o tipo (`colunas`, `barras`, `linhas`, `pizza`), o título e a grade de dados
/// (linha 0 = nomes das séries a partir da coluna 1; coluna 0 = nomes das categorias).
#[derive(Clone, Debug, PartialEq)]
pub struct Grafico {
    pub tipo: String,
    pub titulo: String,
    pub grade: Grade,
}

pub fn ler_grafico(e: &Elemento) -> Grafico {
    let mut linhas = decodificar(&e.dados);
    let cabeca = if linhas.is_empty() { Vec::new() } else { linhas.remove(0) };
    let tipo = cabeca.first().cloned().filter(|t| !t.is_empty()).unwrap_or_else(|| "colunas".into());
    let titulo = cabeca.get(1).cloned().unwrap_or_default();
    Grafico { tipo, titulo, grade: retangular(linhas) }
}

pub fn gravar_grafico(g: &Grafico) -> String {
    let mut tudo = vec![vec![g.tipo.clone(), g.titulo.clone()]];
    tudo.extend(g.grade.iter().cloned());
    codificar(&tudo)
}

/// Os dados de exemplo de um gráfico novo.
pub fn grafico_exemplo(tipo: &str) -> Grafico {
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    let titulo = match tipo {
        "barras" => "Gráfico de barras",
        "pizza" => "Gráfico de pizza",
        "linhas" => "Gráfico de linhas",
        _ => "Gráfico de colunas",
    };
    let grade = if tipo == "pizza" {
        vec![s(&["", "Vendas"]), s(&["Produto A", "45"]), s(&["Produto B", "30"]), s(&["Produto C", "25"])]
    } else {
        vec![s(&["", "Série 1", "Série 2"]), s(&["Categoria 1", "40", "28"]), s(&["Categoria 2", "55", "42"]), s(&["Categoria 3", "35", "60"]), s(&["Categoria 4", "70", "50"])]
    };
    Grafico { tipo: tipo.into(), titulo: titulo.into(), grade }
}

// ------------------------------------------------------------------ tabela

const PASSO: f32 = 1.2;
const MARGEM_CELULA: f32 = 6.0;

fn ret(x: f32, y: f32, w: f32, h: f32, preenchimento: &str, contorno: &str, espessura: f32) -> Primitiva {
    Primitiva::Ret { x, y, w, h, preenchimento: preenchimento.into(), contorno: contorno.into(), espessura }
}

fn texto(x: f32, y: f32, tamanho: f32, cor: &str, negrito: bool, alinh: &'static str, t: &str) -> Primitiva {
    Primitiva::Texto { x, y, tamanho, cor: cor.into(), negrito, alinh, texto: t.into() }
}

fn caixa(e: &Elemento) -> (f32, f32, f32, f32) {
    (e.x.min(e.x + e.largura), e.y.min(e.y + e.altura), e.largura.abs(), e.altura.abs())
}

/// O retângulo (x, y, largura, altura) da célula (`linha`, `coluna`).
pub fn celula_retangulo(e: &Elemento, linha: usize, coluna: usize) -> Option<[f32; 4]> {
    let g = ler_tabela(e);
    let (n, m) = (g.len(), g[0].len());
    if linha >= n || coluna >= m {
        return None;
    }
    let (x, y, w, h) = caixa(e);
    let (cw, rh) = (w / m as f32, h / n as f32);
    Some([x + coluna as f32 * cw, y + linha as f32 * rh, cw, rh])
}

/// A célula sob o ponto (x, y) do slide.
pub fn celula_no_ponto(e: &Elemento, px: f32, py: f32) -> Option<(usize, usize)> {
    let g = ler_tabela(e);
    let (n, m) = (g.len(), g[0].len());
    let (x, y, w, h) = caixa(e);
    if px < x || py < y || px > x + w || py > y + h || w <= 0.0 || h <= 0.0 {
        return None;
    }
    let c = (((px - x) / (w / m as f32)) as usize).min(m - 1);
    let r = (((py - y) / (h / n as f32)) as usize).min(n - 1);
    Some((r, c))
}

pub fn primitivas_tabela(e: &Elemento) -> Vec<Primitiva> {
    let g = ler_tabela(e);
    let (n, m) = (g.len(), g[0].len());
    let (x, y, w, h) = caixa(e);
    let (cw, rh) = (w / m as f32, h / n as f32);
    let borda = if e.espessura <= 0.0 { "" } else if e.contorno.is_empty() { "#bdc1c6" } else { e.contorno.as_str() };
    let cabecalho = if e.preenchimento.is_empty() { "#e8f0fe" } else { e.preenchimento.as_str() };
    let est = &e.estilo;
    let mut p = Vec::new();
    for (r, linha) in g.iter().enumerate() {
        for (c, celula) in linha.iter().enumerate() {
            let (cx, cy) = (x + c as f32 * cw, y + r as f32 * rh);
            let fundo = if r == 0 { cabecalho } else if r % 2 == 0 { "#f8f9fa" } else { "#ffffff" };
            p.push(ret(cx, cy, cw, rh, fundo, borda, e.espessura));
            if celula.is_empty() {
                continue;
            }
            let negrito = est.negrito || r == 0;
            let util = (cw - 2.0 * MARGEM_CELULA).max(est.tamanho);
            let mut linhas: Vec<String> = celula.split('\n').flat_map(|par| medida::quebrar(par, est.tamanho, negrito, util)).collect();
            let cabem = ((rh / (est.tamanho * PASSO)).floor() as usize).max(1);
            linhas.truncate(cabem);
            let k = linhas.len() as f32;
            for (i, l) in linhas.iter().enumerate() {
                let ty = cy + rh / 2.0 + (i as f32 - (k - 1.0) / 2.0) * est.tamanho * PASSO;
                let (tx, alinh) = match est.alinhamento {
                    Alinhamento::Centro => (cx + cw / 2.0, "c"),
                    Alinhamento::Direita => (cx + cw - MARGEM_CELULA, "d"),
                    _ => (cx + MARGEM_CELULA, "e"),
                };
                p.push(texto(tx, ty, est.tamanho, &est.cor, negrito, alinh, l));
            }
        }
    }
    p
}

// ------------------------------------------------------------------ gráfico

const CORES: [&str; 6] = ["#1a73e8", "#34a853", "#fbbc04", "#ea4335", "#8e24aa", "#00acc1"];
const CINZA: &str = "#5f6368";
const GRADE: &str = "#e0e3e7";

fn numero(s: &str) -> f32 {
    s.trim().replace(',', ".").parse::<f32>().unwrap_or(0.0).max(0.0)
}

fn formatar(v: f32) -> String {
    if (v - v.round()).abs() < 0.005 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.1}")
    }
}

/// O passo "redondo" da escala e o topo dela, para ~4 divisões.
fn escala_bonita(maximo: f32) -> (f32, f32) {
    let m = maximo.max(1e-6);
    let bruto = m / 4.0;
    let mag = 10f32.powf(bruto.log10().floor());
    let norm = bruto / mag;
    let passo = mag * if norm <= 1.0 { 1.0 } else if norm <= 2.0 { 2.0 } else if norm <= 2.5 { 2.5 } else if norm <= 5.0 { 5.0 } else { 10.0 };
    (passo, passo * (m / passo).ceil().max(1.0))
}

/// Corta o texto com reticências para caber em `max` pontos.
fn caber(t: &str, tamanho: f32, max: f32) -> String {
    if medida::largura(t, tamanho, false) <= max {
        return t.to_string();
    }
    let mut s: String = t.to_string();
    while !s.is_empty() && medida::largura(&format!("{s}…"), tamanho, false) > max {
        s.pop();
    }
    format!("{s}…")
}

pub fn primitivas_grafico(e: &Elemento) -> Vec<Primitiva> {
    let g = ler_grafico(e);
    let (x, y, w, h) = caixa(e);
    let fundo = if e.preenchimento.is_empty() { "#ffffff" } else { e.preenchimento.as_str() };
    let borda = if e.contorno.is_empty() { "#dadce0" } else { e.contorno.as_str() };
    let mut p = vec![ret(x, y, w, h, fundo, borda, 1.0)];

    let cats: Vec<&str> = g.grade.iter().skip(1).map(|l| l[0].as_str()).collect();
    let series: Vec<&str> = g.grade[0].iter().skip(1).map(String::as_str).collect();
    let valor = |c: usize, s: usize| -> f32 { g.grade.get(c + 1).and_then(|l| l.get(s + 1)).map_or(0.0, |v| numero(v)) };
    if cats.is_empty() || series.is_empty() {
        p.push(texto(x + w / 2.0, y + h / 2.0, 13.0, CINZA, false, "c", "Sem dados"));
        return p;
    }

    let topo_titulo = if g.titulo.is_empty() { 0.0 } else { 34.0 };
    if !g.titulo.is_empty() {
        p.push(texto(x + w / 2.0, y + 20.0, 15.0, "#202124", true, "c", &caber(&g.titulo, 15.0, w - 20.0)));
    }
    let com_legenda = g.tipo != "pizza" && series.len() > 1;
    let rodape = if com_legenda { 24.0 } else { 0.0 };

    let desenhar_legenda = |p: &mut Vec<Primitiva>, nomes: &[&str], x0: f32, y0: f32, vertical: bool, largura: f32| {
        let mut cx = x0;
        for (i, n) in nomes.iter().enumerate() {
            let cor = CORES[i % CORES.len()];
            let nome = caber(n, 11.0, largura.min(110.0));
            if vertical {
                p.push(ret(x0, y0 + i as f32 * 20.0 - 5.0, 10.0, 10.0, cor, "", 0.0));
                p.push(texto(x0 + 16.0, y0 + i as f32 * 20.0, 11.0, CINZA, false, "e", &nome));
            } else {
                p.push(ret(cx, y0 - 5.0, 10.0, 10.0, cor, "", 0.0));
                p.push(texto(cx + 14.0, y0, 11.0, CINZA, false, "e", &nome));
                cx += 14.0 + medida::largura(&nome, 11.0, false) + 18.0;
            }
        }
    };

    match g.tipo.as_str() {
        "pizza" => {
            let legenda_w = 130.0_f32.min(w * 0.35);
            let area_x = x + 12.0;
            let area_w = w - 24.0 - legenda_w;
            let area_y = y + topo_titulo + 8.0;
            let area_h = h - topo_titulo - 20.0;
            let r = (area_w.min(area_h) / 2.0).max(10.0);
            let (cx, cy) = (area_x + area_w / 2.0, area_y + area_h / 2.0);
            let total: f32 = (0..cats.len()).map(|c| valor(c, 0)).sum();
            if total <= 0.0 {
                p.push(texto(x + w / 2.0, y + h / 2.0, 13.0, CINZA, false, "c", "Sem dados"));
                return p;
            }
            let mut inicio = 270.0_f32;
            for c in 0..cats.len() {
                let v = valor(c, 0);
                if v <= 0.0 {
                    continue;
                }
                let varre = 360.0 * v / total;
                let (de, ate) = (inicio.rem_euclid(360.0), (inicio + varre).rem_euclid(360.0));
                let cor = CORES[c % CORES.len()];
                if varre >= 359.9 {
                    p.push(Primitiva::Setor { cx, cy, r, de: 0.0, ate: 359.99, preenchimento: cor.into(), contorno: "#ffffff".into(), espessura: 1.5 });
                } else {
                    p.push(Primitiva::Setor { cx, cy, r, de, ate, preenchimento: cor.into(), contorno: "#ffffff".into(), espessura: 1.5 });
                }
                if varre > 14.0 {
                    let meio = (inicio + varre / 2.0).to_radians();
                    p.push(texto(cx + meio.cos() * r * 0.62, cy + meio.sin() * r * 0.62, 12.0, "#ffffff", true, "c", &format!("{:.0}%", 100.0 * v / total)));
                }
                inicio += varre;
            }
            desenhar_legenda(&mut p, &cats, x + w - legenda_w, y + h / 2.0 - (cats.len() as f32 - 1.0) * 10.0, true, legenda_w - 20.0);
        }
        "barras" => {
            let esq = 92.0_f32.min(w * 0.3);
            let (px, py) = (x + esq, y + topo_titulo + 12.0);
            let (pw, ph) = (w - esq - 20.0, h - topo_titulo - 12.0 - 26.0 - rodape);
            let maximo = (0..cats.len()).flat_map(|c| (0..series.len()).map(move |s| (c, s))).map(|(c, s)| valor(c, s)).fold(0.0, f32::max);
            let (passo, topo) = escala_bonita(maximo);
            let mut v = 0.0;
            while v <= topo + passo * 0.01 {
                let gx = px + pw * v / topo;
                p.push(Primitiva::Linha { x1: gx, y1: py, x2: gx, y2: py + ph, cor: GRADE.into(), espessura: 1.0 });
                p.push(texto(gx, py + ph + 12.0, 10.0, CINZA, false, "c", &formatar(v)));
                v += passo;
            }
            let gh = ph / cats.len() as f32;
            let bh = gh * 0.7 / series.len() as f32;
            for (c, cat) in cats.iter().enumerate() {
                let gy = py + c as f32 * gh;
                p.push(texto(px - 6.0, gy + gh / 2.0, 11.0, CINZA, false, "d", &caber(cat, 11.0, esq - 10.0)));
                for s in 0..series.len() {
                    let largura = pw * valor(c, s) / topo;
                    p.push(ret(px, gy + gh * 0.15 + s as f32 * bh, largura, bh, CORES[s % CORES.len()], "", 0.0));
                }
            }
            p.push(Primitiva::Linha { x1: px, y1: py, x2: px, y2: py + ph, cor: CINZA.into(), espessura: 1.0 });
            if com_legenda {
                desenhar_legenda(&mut p, &series, x + 16.0, y + h - 14.0, false, 110.0);
            }
        }
        tipo => {
            // colunas e linhas: eixo vertical com escala, categorias embaixo
            let (px, py) = (x + 44.0, y + topo_titulo + 12.0);
            let (pw, ph) = (w - 44.0 - 16.0, h - topo_titulo - 12.0 - 30.0 - rodape);
            let maximo = (0..cats.len()).flat_map(|c| (0..series.len()).map(move |s| (c, s))).map(|(c, s)| valor(c, s)).fold(0.0, f32::max);
            let (passo, topo) = escala_bonita(maximo);
            let mut v = 0.0;
            while v <= topo + passo * 0.01 {
                let gy = py + ph - ph * v / topo;
                p.push(Primitiva::Linha { x1: px, y1: gy, x2: px + pw, y2: gy, cor: GRADE.into(), espessura: 1.0 });
                p.push(texto(px - 6.0, gy, 10.0, CINZA, false, "d", &formatar(v)));
                v += passo;
            }
            let gw = pw / cats.len() as f32;
            for (c, cat) in cats.iter().enumerate() {
                p.push(texto(px + c as f32 * gw + gw / 2.0, py + ph + 14.0, 11.0, CINZA, false, "c", &caber(cat, 11.0, gw - 4.0)));
            }
            if tipo == "linhas" {
                for s in 0..series.len() {
                    let cor = CORES[s % CORES.len()];
                    let pontos: Vec<[f32; 2]> = (0..cats.len()).map(|c| [px + c as f32 * gw + gw / 2.0, py + ph - ph * valor(c, s) / topo]).collect();
                    p.push(Primitiva::Poli { pontos: pontos.clone(), cor: cor.into(), espessura: 2.5 });
                    for q in pontos {
                        p.push(Primitiva::Setor { cx: q[0], cy: q[1], r: 4.0, de: 0.0, ate: 359.99, preenchimento: cor.into(), contorno: "#ffffff".into(), espessura: 1.0 });
                    }
                }
            } else {
                let bw = gw * 0.7 / series.len() as f32;
                for c in 0..cats.len() {
                    for s in 0..series.len() {
                        let altura = ph * valor(c, s) / topo;
                        let bx = px + c as f32 * gw + gw * 0.15 + s as f32 * bw;
                        p.push(ret(bx, py + ph - altura, bw, altura, CORES[s % CORES.len()], "", 0.0));
                        if series.len() == 1 && cats.len() <= 8 {
                            p.push(texto(bx + bw / 2.0, py + ph - altura - 8.0, 10.0, CINZA, false, "c", &formatar(valor(c, s))));
                        }
                    }
                }
            }
            p.push(Primitiva::Linha { x1: px, y1: py + ph, x2: px + pw, y2: py + ph, cor: CINZA.into(), espessura: 1.0 });
            if com_legenda {
                desenhar_legenda(&mut p, &series, x + 16.0, y + h - 14.0, false, 110.0);
            }
        }
    }
    p
}
