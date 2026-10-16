//! As cores do `.pptx`: a cor fixa (`a:srgbClr`), a do sistema (`a:sysClr`) e a do tema
//! (`a:schemeClr`), que passa pelo mapa do mestre (`bg1` → `lt1`) até o esquema do tema. Os
//! ajustes `lumMod`/`lumOff` (o "mais claro 40 %" do PowerPoint), `tint` e `shade` são
//! aplicados aqui, para a cor sair pronta. O tema também diz as duas fontes (`+mj-lt`, a dos
//! títulos, e `+mn-lt`, a do corpo).

use std::collections::HashMap;

use crate::xml::No;

/// As cores do tema e o mapa de nomes do mestre.
#[derive(Default)]
pub struct Tema {
    esquema: HashMap<String, String>,
    mapa: HashMap<String, String>,
    fonte_titulo: String,
    fonte_corpo: String,
}

/// Cores com nome (`a:prstClr`) que aparecem na prática.
fn cor_com_nome(nome: &str) -> Option<&'static str> {
    Some(match nome {
        "black" => "000000",
        "white" => "FFFFFF",
        "red" => "FF0000",
        "green" => "008000",
        "blue" => "0000FF",
        "yellow" => "FFFF00",
        "gray" | "grey" => "808080",
        _ => return None,
    })
}

impl Tema {
    /// O tema (`a:theme`) com o mapa de cores do mestre (`p:clrMap`).
    pub fn novo(tema: Option<&No>, mapa: Option<&No>) -> Self {
        let mut esquema = HashMap::new();
        if let Some(cs) = tema.and_then(|t| t.busca("a:clrScheme")) {
            for c in &cs.filhos {
                let nome = c.nome.trim_start_matches("a:").to_string();
                let valor = c.filho("a:srgbClr").and_then(|s| s.attr("val")).or_else(|| c.filho("a:sysClr").and_then(|s| s.attr("lastClr")));
                if let Some(v) = valor {
                    esquema.insert(nome, v.to_uppercase());
                }
            }
        }
        let mapa = mapa.map(|m| m.attrs.iter().map(|(k, v)| (k.clone(), v.clone())).collect()).unwrap_or_default();
        let fonte = |qual: &str| tema.and_then(|t| t.busca(qual)).and_then(|f| f.filho("a:latin")).and_then(|l| l.attr("typeface")).unwrap_or("").to_string();
        Self { esquema, mapa, fonte_titulo: fonte("a:majorFont"), fonte_corpo: fonte("a:minorFont") }
    }

    /// O nome real da fonte: `+mj-lt`/`+mn-lt` viram as fontes do tema.
    pub fn fonte(&self, nome: &str) -> String {
        match nome {
            "+mj-lt" | "+mj-ea" | "+mj-cs" => self.fonte_titulo.clone(),
            "+mn-lt" | "+mn-ea" | "+mn-cs" => self.fonte_corpo.clone(),
            _ => nome.to_string(),
        }
    }

    /// A cor padrão do texto (`tx1` do tema; preto se o tema não diz).
    pub fn cor_do_texto(&self) -> String {
        self.do_tema("tx1").map_or_else(|| "#000000".into(), |h| format!("#{}", h.to_lowercase()))
    }

    /// O hexadecimal de um nome do tema (`tx1`, `accent2`, `dk1`…).
    fn do_tema(&self, nome: &str) -> Option<String> {
        let real = self.mapa.get(nome).map_or(nome, String::as_str);
        self.esquema.get(real).or_else(|| self.esquema.get(nome)).cloned()
    }

    /// A cor dentro de `no` (um `a:solidFill`, `a:fontRef`…), como `#rrggbb`. `None` se não há.
    pub fn cor(&self, no: &No) -> Option<String> {
        for c in &no.filhos {
            let base = match c.nome.as_str() {
                "a:srgbClr" => c.attr("val").map(str::to_uppercase),
                "a:schemeClr" => c.attr("val").and_then(|v| self.do_tema(v)),
                "a:sysClr" => c.attr("lastClr").map(str::to_uppercase),
                "a:prstClr" => c.attr("val").and_then(cor_com_nome).map(String::from),
                _ => None,
            };
            if let Some(hex) = base {
                return Some(ajustar(&hex, c));
            }
        }
        None
    }

    /// O preenchimento sólido de `pai` (o `a:solidFill` filho dele). `Some("")` se é `a:noFill`.
    pub fn preenchimento(&self, pai: &No) -> Option<String> {
        if pai.filho("a:noFill").is_some() {
            return Some(String::new());
        }
        pai.filho("a:solidFill").and_then(|s| self.cor(s))
    }
}

fn rgb(hex: &str) -> (f64, f64, f64) {
    let n = |i: usize| f64::from(u8::from_str_radix(hex.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0)) / 255.0;
    (n(0), n(2), n(4))
}

fn hex((r, g, b): (f64, f64, f64)) -> String {
    let c = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", c(r), c(g), c(b))
}

fn para_hsl((r, g, b): (f64, f64, f64)) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < 1e-9 {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h / 6.0, s, l)
}

fn de_hsl((h, s, l): (f64, f64, f64)) -> (f64, f64, f64) {
    if s == 0.0 {
        return (l, l, l);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let canal = |mut t: f64| {
        t = t.rem_euclid(1.0);
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    (canal(h + 1.0 / 3.0), canal(h), canal(h - 1.0 / 3.0))
}

/// Aplica os ajustes filhos da cor (`a:lumMod`, `a:lumOff`, `a:tint`, `a:shade`) e a
/// transparência (`a:alpha`, que sai como `#rrggbbaa`).
fn ajustar(base: &str, cor: &No) -> String {
    let mut c = rgb(base);
    let fator = |n: &No| n.num("val").unwrap_or(100_000.0) / 100_000.0;
    let (mut lm, mut lo, mut alfa) = (1.0, 0.0, 1.0);
    for a in &cor.filhos {
        match a.nome.as_str() {
            "a:alpha" => alfa = fator(a).clamp(0.0, 1.0),
            "a:lumMod" => lm = fator(a),
            "a:lumOff" => lo = fator(a),
            "a:shade" => c = (c.0 * fator(a), c.1 * fator(a), c.2 * fator(a)),
            "a:tint" => {
                let t = fator(a);
                c = (1.0 - (1.0 - c.0) * t, 1.0 - (1.0 - c.1) * t, 1.0 - (1.0 - c.2) * t);
            }
            _ => {}
        }
    }
    if lm != 1.0 || lo != 0.0 {
        let (h, s, l) = para_hsl(c);
        c = de_hsl((h, s, (l * lm + lo).clamp(0.0, 1.0)));
    }
    if alfa < 1.0 {
        format!("{}{:02x}", hex(c), (alfa * 255.0).round() as u8)
    } else {
        hex(c)
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::xml;

    #[test]
    fn cor_do_tema_passa_pelo_mapa_e_pelos_ajustes() {
        let tema = xml::ler(br#"<a:theme><a:clrScheme><a:dk1><a:sysClr lastClr="000000"/></a:dk1><a:lt1><a:srgbClr val="FFFFFF"/></a:lt1><a:accent1><a:srgbClr val="4472C4"/></a:accent1></a:clrScheme></a:theme>"#, "t").unwrap();
        let mapa = xml::ler(br#"<p:clrMap bg1="lt1" tx1="dk1"/>"#, "m").unwrap();
        let t = Tema::novo(Some(&tema), Some(&mapa));
        let f = xml::ler(br#"<a:solidFill><a:schemeClr val="tx1"/></a:solidFill>"#, "f").unwrap();
        assert_eq!(t.cor(&f).as_deref(), Some("#000000"));
        let claro = xml::ler(br#"<a:solidFill><a:schemeClr val="accent1"><a:lumMod val="40000"/><a:lumOff val="60000"/></a:schemeClr></a:solidFill>"#, "f").unwrap();
        let c = t.cor(&claro).unwrap();
        assert_ne!(c, "#4472c4");
        assert!(c.as_str() > "#a0", "mais claro: {c}");
        let transparente = xml::ler(br#"<a:solidFill><a:schemeClr val="lt1"><a:alpha val="9410"/></a:schemeClr></a:solidFill>"#, "f").unwrap();
        assert_eq!(t.cor(&transparente).as_deref(), Some("#ffffff18"));
        let nada = xml::ler(br#"<p:spPr><a:noFill/></p:spPr>"#, "n").unwrap();
        assert_eq!(t.preenchimento(&nada).as_deref(), Some(""));
    }
}
