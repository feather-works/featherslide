//! Exportar a apresentação em **PDF vetorial**, uma página por slide.
//!
//! O que sai é o `Quadro` do motor, o mesmo que a tela desenha: formas viram caminhos do PDF
//! (nada de imagem), o texto usa a Helvetica padrão (negrito/itálico incluídos) que todo leitor
//! de PDF já traz, e as imagens entram como pixels (RGB, com máscara de transparência quando
//! têm). Tabelas e gráficos saem das mesmas primitivas que a tela pinta.
//!
//! Limites desta versão: fontes próprias não são embutidas (cai na Helvetica; letras fora do
//! Latin-1 viram `?`), e a sombra das formas não é desenhada.

use std::fmt::Write as _;
use std::io::Write as _;

use featherslide_core::dados::Primitiva;
use featherslide_core::medida;
use featherslide_core::quadro::{ElementoDesenhado, Quadro};
use flate2::write::ZlibEncoder;
use flate2::Compression;

/// Uma imagem já pronta para a página: largura, altura e pixels RGBA (como a tela a mostra).
pub type ImagemRgba = (u32, u32, Vec<u8>);

/// Um passo de caminho.
#[derive(Clone, Copy)]
enum Seg {
    M(f32, f32),
    L(f32, f32),
    C(f32, f32, f32, f32, f32, f32),
    Z,
}

type Caminho = Vec<Seg>;

const SEGMENTOS: usize = 64;
/// Constante do círculo por quatro curvas de Bézier.
const K: f32 = 0.552_284_8;

fn f(v: f32) -> String {
    let s = format!("{v:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn escrever_caminho(c: &Caminho, saida: &mut String) {
    for s in c {
        let _ = match *s {
            Seg::M(x, y) => writeln!(saida, "{} {} m", f(x), f(y)),
            Seg::L(x, y) => writeln!(saida, "{} {} l", f(x), f(y)),
            Seg::C(a, b, c, d, e, g) => writeln!(saida, "{} {} {} {} {} {} c", f(a), f(b), f(c), f(d), f(e), f(g)),
            Seg::Z => writeln!(saida, "h"),
        };
    }
}

// ------------------------------------------------------------------ cores

/// `#rrggbb` ou `#rrggbbaa` → (r, g, b em 0..1, alfa em 0..1). Vazia ou inválida: `None`.
fn cor(s: &str) -> Option<([f32; 3], f32)> {
    let h = s.trim().trim_start_matches('#');
    if h.len() != 6 && h.len() != 8 {
        return None;
    }
    let v = u32::from_str_radix(h, 16).ok()?;
    let (rgb, a) = if h.len() == 6 { (v, 255) } else { (v >> 8, v & 0xff) };
    let c = |sh: u32| ((rgb >> sh) & 0xff) as f32 / 255.0;
    Some(([c(16), c(8), c(0)], a as f32 / 255.0))
}

// ------------------------------------------------------------------ geometria das formas

fn elipse(x: f32, y: f32, w: f32, h: f32) -> Caminho {
    let (rx, ry) = (w.abs() / 2.0, h.abs() / 2.0);
    let (cx, cy) = (x + rx, y + ry);
    let (ox, oy) = (rx * K, ry * K);
    vec![
        Seg::M(cx - rx, cy),
        Seg::C(cx - rx, cy - oy, cx - ox, cy - ry, cx, cy - ry),
        Seg::C(cx + ox, cy - ry, cx + rx, cy - oy, cx + rx, cy),
        Seg::C(cx + rx, cy + oy, cx + ox, cy + ry, cx, cy + ry),
        Seg::C(cx - ox, cy + ry, cx - rx, cy + oy, cx - rx, cy),
        Seg::Z,
    ]
}

fn ponto_na_elipse(cx: f32, cy: f32, rx: f32, ry: f32, graus: f32) -> (f32, f32) {
    let a = graus.to_radians();
    (cx + rx * a.cos(), cy + ry * a.sin())
}

fn setor(cx: f32, cy: f32, rx: f32, ry: f32, de: f32, ate: f32) -> Caminho {
    let ate = if ate <= de { ate + 360.0 } else { ate };
    let mut c = vec![Seg::M(cx, cy)];
    for i in 0..=SEGMENTOS {
        let (x, y) = ponto_na_elipse(cx, cy, rx, ry, de + (ate - de) * i as f32 / SEGMENTOS as f32);
        c.push(Seg::L(x, y));
    }
    c.push(Seg::Z);
    c
}

fn poligono(pontos: &[(f32, f32)]) -> Caminho {
    let mut c: Caminho = pontos.iter().enumerate().map(|(i, &(x, y))| if i == 0 { Seg::M(x, y) } else { Seg::L(x, y) }).collect();
    c.push(Seg::Z);
    c
}

/// Polígono com cantos arredondados (raio = fração do lado menor, por vértice), como a tela.
fn poligono_arredondado(pontos: &[(f32, f32)], raios: &[f32], menor: f32) -> Caminho {
    let n = pontos.len();
    let unit = |de: (f32, f32), para: (f32, f32)| {
        let (dx, dy) = (para.0 - de.0, para.1 - de.1);
        let d = (dx * dx + dy * dy).sqrt().max(0.0001);
        (dx / d, dy / d, d)
    };
    let mut c = Caminho::new();
    for i in 0..n {
        let (ant, v, prox) = (pontos[(i + n - 1) % n], pontos[i], pontos[(i + 1) % n]);
        let (ax, ay, da) = unit(v, ant);
        let (bx, by, db) = unit(v, prox);
        let r = (raios.get(i).copied().unwrap_or(0.0).clamp(0.0, 0.5) * menor).min(da / 2.0).min(db / 2.0);
        let (antes, depois) = ((v.0 + ax * r, v.1 + ay * r), (v.0 + bx * r, v.1 + by * r));
        let ponto = if r < 0.5 { v } else { antes };
        c.push(if i == 0 { Seg::M(ponto.0, ponto.1) } else { Seg::L(ponto.0, ponto.1) });
        if r >= 0.5 {
            // Quadrática (controle no vértice) → cúbica.
            let (c1, c2) = ((ponto.0 + 2.0 / 3.0 * (v.0 - ponto.0), ponto.1 + 2.0 / 3.0 * (v.1 - ponto.1)), (depois.0 + 2.0 / 3.0 * (v.0 - depois.0), depois.1 + 2.0 / 3.0 * (v.1 - depois.1)));
            c.push(Seg::C(c1.0, c1.1, c2.0, c2.1, depois.0, depois.1));
        }
    }
    c.push(Seg::Z);
    c
}

/// Retângulo com raio `rc` nos cantos de cima e `rb` nos de baixo.
fn retangulo_arredondado(x: f32, y: f32, w: f32, h: f32, rc: f32, rb: f32) -> Caminho {
    let (r1, r2) = (rc.min(w / 2.0).min(h / 2.0).max(0.0), rb.min(w / 2.0).min(h / 2.0).max(0.0));
    let (k1, k2) = (r1 * (1.0 - K), r2 * (1.0 - K));
    vec![
        Seg::M(x + r1, y),
        Seg::L(x + w - r1, y),
        Seg::C(x + w - k1, y, x + w, y + k1, x + w, y + r1),
        Seg::L(x + w, y + h - r2),
        Seg::C(x + w, y + h - k2, x + w - k2, y + h, x + w - r2, y + h),
        Seg::L(x + r2, y + h),
        Seg::C(x + k2, y + h, x, y + h - k2, x, y + h - r2),
        Seg::L(x, y + r1),
        Seg::C(x, y + k1, x + k1, y, x + r1, y),
        Seg::Z,
    ]
}

fn seta_pontos(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    let ponta = (w * 0.4).min(h);
    vec![(x, y + h * 0.3), (x + w - ponta, y + h * 0.3), (x + w - ponta, y), (x + w, y + h / 2.0), (x + w - ponta, y + h), (x + w - ponta, y + h * 0.7), (x, y + h * 0.7)]
}

/// O contorno de uma forma no retângulo (x, y, w, h); `eh_imagem` = é o recorte de uma imagem.
fn caminho_da_forma(tipo: &str, x: f32, y: f32, w: f32, h: f32, aj: &[f32], eh_imagem: bool) -> Caminho {
    if let Some(pontos) = featherslide_core::formas::pontos(tipo, x, y, w, h) {
        return poligono(&pontos.into_iter().map(|[x, y]| (x, y)).collect::<Vec<_>>());
    }
    let menor = w.abs().min(h.abs());
    let retangulo = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
    let triangulo = [(x + w / 2.0, y), (x + w, y + h), (x, y + h)];
    match tipo {
        "retangulo" | "retangulo_arredondado" if aj.len() == 4 && !eh_imagem => poligono_arredondado(&retangulo, aj, menor),
        "triangulo" if aj.len() == 3 => poligono_arredondado(&triangulo, aj, menor),
        "seta" if aj.len() == 7 => poligono_arredondado(&seta_pontos(x, y, w, h), aj, menor),
        "elipse" => elipse(x, y, w, h),
        "setor" => setor(x + w / 2.0, y + h / 2.0, (w / 2.0).abs(), (h / 2.0).abs(), aj.first().copied().unwrap_or(0.0), aj.get(1).copied().unwrap_or(270.0)),
        "retangulo_arredondado" => {
            let cima = if eh_imagem { aj.get(3).copied().unwrap_or(0.16) } else { aj.first().copied().unwrap_or(0.16) };
            let baixo = if eh_imagem { cima } else { aj.get(1).copied().unwrap_or(cima) };
            retangulo_arredondado(x, y, w, h, (menor / 2.0).min(menor * cima), (menor / 2.0).min(menor * baixo))
        }
        "triangulo" => poligono(&triangulo),
        "seta" => poligono(&seta_pontos(x, y, w, h)),
        "linha" => vec![Seg::M(x, y), Seg::L(x + w, y + h)],
        _ => poligono(&retangulo),
    }
}

// ------------------------------------------------------------------ o documento

#[derive(Default)]
struct Documento {
    /// Objetos já serializados (o número é o índice + 1).
    objetos: Vec<Vec<u8>>,
}

impl Documento {
    fn reservar(&mut self) -> usize {
        self.objetos.push(Vec::new());
        self.objetos.len()
    }

    fn definir(&mut self, n: usize, corpo: Vec<u8>) {
        self.objetos[n - 1] = corpo;
    }

    fn novo(&mut self, corpo: Vec<u8>) -> usize {
        let n = self.reservar();
        self.definir(n, corpo);
        n
    }

    fn fluxo(&mut self, dicionario: &str, dados: &[u8]) -> usize {
        let mut o = format!("<< {dicionario} /Length {} >>\nstream\n", dados.len()).into_bytes();
        o.extend_from_slice(dados);
        o.extend_from_slice(b"\nendstream");
        self.novo(o)
    }

    fn serializar(&self, raiz: usize, info: usize) -> Vec<u8> {
        let mut s = b"%PDF-1.5\n%\xE2\xE3\xCF\xD3\n".to_vec();
        let mut posicoes = Vec::new();
        for (i, o) in self.objetos.iter().enumerate() {
            posicoes.push(s.len());
            s.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
            s.extend_from_slice(o);
            s.extend_from_slice(b"\nendobj\n");
        }
        let xref = s.len();
        s.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", self.objetos.len() + 1).as_bytes());
        for p in posicoes {
            s.extend_from_slice(format!("{p:010} 00000 n \n").as_bytes());
        }
        s.extend_from_slice(format!("trailer\n<< /Size {} /Root {raiz} 0 R /Info {info} 0 R >>\nstartxref\n{xref}\n%%EOF\n", self.objetos.len() + 1).as_bytes());
        s
    }
}

fn comprimir(dados: &[u8]) -> Vec<u8> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    let _ = e.write_all(dados);
    e.finish().unwrap_or_default()
}

/// Texto do PDF em WinAnsi, já entre parênteses e com escapes. Fora do Latin-1: `?`.
fn texto_winansi(t: &str) -> Vec<u8> {
    let mut s = vec![b'('];
    for ch in t.chars() {
        let b = match ch {
            '€' => 0x80,
            '…' => 0x85,
            '‘' => 0x91,
            '’' => 0x92,
            '“' => 0x93,
            '”' => 0x94,
            '•' => 0x95,
            '–' => 0x96,
            '—' => 0x97,
            '™' => 0x99,
            c if (c as u32) < 0x100 && !(0x80..0xa0).contains(&(c as u32)) && (c as u32) >= 0x20 => c as u8,
            _ => b'?',
        };
        if matches!(b, b'(' | b')' | b'\\') {
            s.push(b'\\');
        }
        s.push(b);
    }
    s.push(b')');
    s
}

/// Os recursos que uma página usa: imagens e opacidades.
#[derive(Default)]
struct Recursos {
    imagens: Vec<(String, usize)>,
    alfas: Vec<u32>,
}

impl Recursos {
    /// O nome do estado gráfico com esta opacidade (em %), criando se preciso.
    fn alfa(&mut self, a: f32) -> String {
        let pct = (a.clamp(0.0, 1.0) * 100.0).round() as u32;
        if !self.alfas.contains(&pct) {
            self.alfas.push(pct);
        }
        format!("/A{pct} gs\n")
    }
}

/// Fonte do PDF: 0 normal, 1 negrito, 2 itálico, 3 negrito-itálico.
fn fonte_n(negrito: bool, italico: bool) -> usize {
    usize::from(negrito) + 2 * usize::from(italico)
}

struct Pagina<'a> {
    doc: &'a mut Documento,
    rec: Recursos,
    c: String,
    altura: f32,
}

impl Pagina<'_> {
    fn cor_de_preenchimento(&mut self, s: &str) -> bool {
        let Some((rgb, a)) = cor(s) else { return false };
        let _ = writeln!(self.c, "{} {} {} rg", f(rgb[0]), f(rgb[1]), f(rgb[2]));
        if a < 0.999 {
            let g = self.rec.alfa(a);
            self.c.push_str(&g);
        }
        true
    }

    fn cor_de_traco(&mut self, s: &str, espessura: f32) -> bool {
        let Some((rgb, a)) = cor(s) else { return false };
        let _ = writeln!(self.c, "1 J 1 j {} {} {} RG {} w", f(rgb[0]), f(rgb[1]), f(rgb[2]), f(espessura.max(0.25)));
        if a < 0.999 {
            let g = self.rec.alfa(a);
            self.c.push_str(&g);
        }
        true
    }

    fn preencher(&mut self, caminho: &Caminho, cor_s: &str) {
        self.c.push_str("q\n");
        if self.cor_de_preenchimento(cor_s) {
            escrever_caminho(caminho, &mut self.c);
            self.c.push_str("f\n");
        }
        self.c.push_str("Q\n");
    }

    fn tracar(&mut self, caminho: &Caminho, cor_s: &str, espessura: f32) {
        self.c.push_str("q\n");
        if self.cor_de_traco(cor_s, espessura) {
            escrever_caminho(caminho, &mut self.c);
            self.c.push_str("S\n");
        }
        self.c.push_str("Q\n");
    }

    fn ret(&mut self, x: f32, y: f32, w: f32, h: f32, preenchimento: &str, contorno: &str, espessura: f32) {
        let c = poligono(&[(x, y), (x + w, y), (x + w, y + h), (x, y + h)]);
        self.preencher(&c, preenchimento);
        self.tracar(&c, contorno, espessura);
    }

    /// Uma linha de texto com a linha de base em (x, y).
    fn texto(&mut self, t: &str, x: f32, y: f32, tamanho: f32, cor_s: &str, negrito: bool, italico: bool) {
        if t.is_empty() {
            return;
        }
        self.c.push_str("q\n");
        if !self.cor_de_preenchimento(cor_s) {
            self.c.push_str("0.125 0.129 0.141 rg\n");
        }
        let _ = writeln!(self.c, "BT /F{} {} Tf 1 0 0 -1 {} {} Tm", fonte_n(negrito, italico), f(tamanho), f(x), f(y));
        // O conteúdo da página é UTF-8 só nos números; o texto vai em bytes WinAnsi.
        self.c.push_str(&texto_como_latin1(&texto_winansi(t)));
        self.c.push_str(" Tj ET\nQ\n");
    }

    fn primitivas(&mut self, el: &ElementoDesenhado) {
        let [bx, by, bw, bh] = el.base;
        let sx = if bw.abs() > 0.01 { el.largura / bw } else { 1.0 };
        let sy = if bh.abs() > 0.01 { el.altura / bh } else { 1.0 };
        let px = |x: f32| el.x + (x - bx) * sx;
        let py = |y: f32| el.y + (y - by) * sy;
        for p in &el.primitivas {
            match p {
                Primitiva::Ret { x, y, w, h, preenchimento, contorno, espessura } => self.ret(px(*x), py(*y), w * sx, h * sy, preenchimento, contorno, *espessura),
                Primitiva::Linha { x1, y1, x2, y2, cor, espessura } => self.tracar(&vec![Seg::M(px(*x1), py(*y1)), Seg::L(px(*x2), py(*y2))], cor, *espessura),
                Primitiva::Setor { cx, cy, r, de, ate, preenchimento, contorno, espessura } => {
                    let raio = r * sx.min(sy);
                    let c = setor(px(*cx), py(*cy), raio, raio, *de, *ate);
                    self.preencher(&c, preenchimento);
                    self.tracar(&c, contorno, *espessura);
                }
                Primitiva::Poli { pontos, cor, espessura } if pontos.len() > 1 => {
                    let c: Caminho = pontos.iter().enumerate().map(|(i, q)| if i == 0 { Seg::M(px(q[0]), py(q[1])) } else { Seg::L(px(q[0]), py(q[1])) }).collect();
                    self.tracar(&c, cor, *espessura);
                }
                Primitiva::Poli { .. } => {}
                Primitiva::Poligono { pontos, preenchimento } if pontos.len() > 2 => {
                    let c = poligono(&pontos.iter().map(|q| (px(q[0]), py(q[1]))).collect::<Vec<_>>());
                    self.preencher(&c, preenchimento);
                }
                Primitiva::Poligono { .. } => {}
                Primitiva::Vetor { segmentos, par, preenchimento, contorno, espessura } => {
                    use featherslide_core::vetores::Segmento;
                    let mut c = Vec::new(); let mut atual = [0.0,0.0]; let mut inicio = atual;
                    for s in segmentos {
                        match s {
                            Segmento::M(p) => { c.push(Seg::M(px(p[0]),py(p[1]))); inicio=*p; atual=*p; },
                            Segmento::L(p) => { c.push(Seg::L(px(p[0]),py(p[1]))); atual=*p; },
                            Segmento::C(a,b,p) => { c.push(Seg::C(px(a[0]),py(a[1]),px(b[0]),py(b[1]),px(p[0]),py(p[1]))); atual=*p; },
                            Segmento::Q(a,p) => {
                                let b = [atual[0]+(a[0]-atual[0])*2.0/3.0,atual[1]+(a[1]-atual[1])*2.0/3.0];
                                let d = [p[0]+(a[0]-p[0])*2.0/3.0,p[1]+(a[1]-p[1])*2.0/3.0];
                                c.push(Seg::C(px(b[0]),py(b[1]),px(d[0]),py(d[1]),px(p[0]),py(p[1]))); atual=*p;
                            },
                            Segmento::Z => { c.push(Seg::Z); atual=inicio; },
                        }
                    }
                    self.c.push_str("q\n");
                    if self.cor_de_preenchimento(preenchimento) {
                        escrever_caminho(&c,&mut self.c); self.c.push_str(if *par { "f*\n" } else { "f\n" });
                    }
                    self.c.push_str("Q\n"); self.tracar(&c,contorno,*espessura);
                }
                Primitiva::Caminho { contornos, preenchimento, contorno, espessura } => {
                    let mut c = Vec::new();
                    for pontos in contornos.iter().filter(|p| p.len() > 2) {
                        c.extend(poligono(&pontos.iter().map(|q| (px(q[0]), py(q[1]))).collect::<Vec<_>>()));
                    }
                    self.preencher(&c, preenchimento);
                    self.tracar(&c, contorno, *espessura);
                }
                Primitiva::Texto { x, y, tamanho, cor, negrito, alinh, texto } => {
                    let largura = medida::largura(texto, *tamanho, *negrito);
                    let x0 = match *alinh {
                        "c" => px(*x) - largura / 2.0,
                        "d" => px(*x) - largura,
                        _ => px(*x),
                    };
                    // `y` é o meio da linha: a linha de base fica ~0,35 do corpo abaixo.
                    self.texto(texto, x0, py(*y) + tamanho * 0.35, *tamanho, cor, *negrito, false);
                }
            }
        }
    }

    fn imagem(&mut self, el: &ElementoDesenhado, pixels: &ImagemRgba) {
        let (iw, ih, rgba) = pixels;
        if rgba.len() < (*iw as usize) * (*ih as usize) * 4 || *iw == 0 || *ih == 0 {
            return;
        }
        let rgb: Vec<u8> = rgba.chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]).collect();
        let alfa: Vec<u8> = rgba.chunks_exact(4).map(|p| p[3]).collect();
        let mascara = alfa.iter().any(|&a| a != 255).then(|| {
            let dic = format!("/Type /XObject /Subtype /Image /Width {iw} /Height {ih} /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode");
            self.doc.fluxo(&dic, &comprimir(&alfa))
        });
        let mut dic = format!("/Type /XObject /Subtype /Image /Width {iw} /Height {ih} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode");
        if let Some(m) = mascara {
            let _ = write!(dic, " /SMask {m} 0 R");
        }
        let obj = self.doc.fluxo(&dic, &comprimir(&rgb));
        let nome = format!("Im{}", self.rec.imagens.len());
        self.rec.imagens.push((nome.clone(), obj));
        self.c.push_str("q\n");
        if el.opacidade < 0.999 {
            let g = self.rec.alfa(el.opacidade);
            self.c.push_str(&g);
        }
        // O quadrado unitário da imagem tem y para cima; a página aqui tem y para baixo.
        let _ = writeln!(self.c, "{} 0 0 {} {} {} cm /{nome} Do\nQ", f(el.largura), f(-el.altura), f(el.x), f(el.y + el.altura));
    }

    fn forma(&mut self, el: &ElementoDesenhado) {
        if el.tipo == "wordart" { return; }
        let eh_imagem = el.tipo == "imagem";
        let tipo = if eh_imagem && !el.recorte.is_empty() { el.recorte } else { el.tipo };
        if !eh_imagem && el.preenchimento.is_empty() && el.contorno.is_empty() {
            return;
        }
        let c = caminho_da_forma(tipo, el.x, el.y, el.largura, el.altura, &el.ajustes, eh_imagem);
        let espessura = el.espessura.max(0.5);
        if el.tipo == "linha" {
            let linha = if el.contorno.is_empty() { &el.preenchimento } else { &el.contorno };
            self.tracar(&c, linha, espessura);
            return;
        }
        if !eh_imagem {
            self.preencher(&c, &el.preenchimento);
        }
        self.tracar(&c, &el.contorno, espessura);
    }

    fn linhas_de_texto(&mut self, el: &ElementoDesenhado) {
        for l in el.linhas.iter().filter(|l| !l.texto.is_empty()) {
            if !el.realce.is_empty() {
                self.ret(l.x, l.y - el.tamanho * 0.88, l.largura, el.tamanho * 1.2, &el.realce, "", 0.0);
            }
            if l.glifos.is_empty() { self.texto(&l.texto, l.x, l.y, el.tamanho, &el.cor, el.negrito, el.italico); }
            else { for g in &l.glifos {self.texto(&g.texto,g.x,g.y,g.tamanho,&el.cor,el.negrito,el.italico);} }
            let grossura = (el.tamanho / 16.0).max(0.5);
            let tracos = |base: f32, duplo: bool| if duplo { vec![base - grossura, base + grossura] } else { vec![base] };
            if el.sublinhado {
                for y in tracos(l.y + el.tamanho * 0.1 + if el.sublinhado_duplo { grossura } else { 0.0 }, el.sublinhado_duplo) {
                    self.ret(l.x, y, l.largura, grossura, &el.cor, "", 0.0);
                }
            }
            if el.tachado {
                for y in tracos(l.y - el.tamanho * 0.28, el.tachado_duplo) {
                    self.ret(l.x, y, l.largura, grossura, &el.cor, "", 0.0);
                }
            }
        }
    }

    fn elemento(&mut self, el: &ElementoDesenhado, imagem: &dyn Fn(&ElementoDesenhado) -> Option<ImagemRgba>) {
        self.c.push_str("q\n");
        if el.rotacao != 0.0 {
            let (cx, cy) = (el.x + el.largura / 2.0, el.y + el.altura / 2.0);
            let (s, c) = el.rotacao.to_radians().sin_cos();
            let _ = writeln!(self.c, "1 0 0 1 {} {} cm {} {} {} {} 0 0 cm 1 0 0 1 {} {} cm", f(cx), f(cy), f(c), f(s), f(-s), f(c), f(-cx), f(-cy));
        }
        if el.tipo == "imagem" {
            if let Some(px) = imagem(el) {
                self.imagem(el, &px);
            }
        }
        let personalizada = el.tipo != "imagem" && (el.gradiente.is_some() || el.opacidade != 1.0 || el.sombra.is_some());
        if el.tipo != "imagem" {
            if let Some((px,margem))=raster_sombra(el) {
                let mut camada=el.clone(); let sombra=el.sombra.as_ref().unwrap();
                camada.x+=sombra.dx-margem;camada.y+=sombra.dy-margem;camada.largura+=2.0*margem;camada.altura+=2.0*margem;camada.opacidade=1.0;
                self.imagem(&camada,&px);
            }
        }
        if personalizada {
            if let Some(px) = raster_forma(el) { let mut camada=el.clone();let m=margem_forma(el);camada.x-=m;camada.y-=m;camada.largura+=2.0*m;camada.altura+=2.0*m;camada.opacidade=1.0;self.imagem(&camada, &px); }
        } else {
            self.forma(el);
            if !el.primitivas.is_empty() { self.primitivas(el); }
        }
        self.linhas_de_texto(el);
        self.c.push_str("Q\n");
    }
}

/// Os bytes do conteúdo vão como Latin-1 dentro de uma `String` (cada byte vira o char de mesmo
/// valor) e são devolvidos como bytes na gravação.
fn texto_como_latin1(b: &[u8]) -> String {
    b.iter().map(|&x| x as char).collect()
}

fn latin1_como_bytes(s: &str) -> Vec<u8> {
    s.chars().map(|c| c as u32 as u8).collect()
}

/// Gera o PDF: uma página por quadro, no tamanho do slide (1 ponto = 1 ponto do PDF).
/// `imagem` entrega os pixels já enquadrados de uma imagem (a interface aplica recorte, zoom e
/// "remover fundo"); `None` pula a imagem.
pub fn exportar(titulo: &str, quadros: &[Quadro], imagem: &dyn Fn(&ElementoDesenhado) -> Option<ImagemRgba>) -> Vec<u8> {
    let mut doc = Documento::default();
    let raiz = doc.reservar();
    let paginas = doc.reservar();
    let info = doc.novo([b"<< /Title ".as_slice(), &utf16_hex(titulo), b" /Producer (FeatherSlide) >>"].concat());
    let fontes: Vec<usize> = ["Helvetica", "Helvetica-Bold", "Helvetica-Oblique", "Helvetica-BoldOblique"]
        .iter()
        .map(|n| doc.novo(format!("<< /Type /Font /Subtype /Type1 /BaseFont /{n} /Encoding /WinAnsiEncoding >>").into_bytes()))
        .collect();
    let mut filhos = Vec::new();
    for q in quadros {
        let pagina = doc.reservar();
        let mut p = Pagina { doc: &mut doc, rec: Recursos::default(), c: String::new(), altura: q.altura };
        if let Some((rgb, _)) = cor(&q.fundo) {
            let _ = writeln!(p.c, "{} {} {} rg 0 0 {} {} re f", f(rgb[0]), f(rgb[1]), f(rgb[2]), f(q.largura), f(q.altura));
        }
        // Daqui em diante o (0,0) é o canto de cima à esquerda, como no slide.
        let _ = writeln!(p.c, "1 0 0 -1 0 {} cm", f(p.altura));
        for el in &q.elementos {
            p.elemento(el, imagem);
        }
        let (rec, conteudo) = (p.rec, p.c);
        let fluxo = doc.fluxo("/Filter /FlateDecode", &comprimir(&latin1_como_bytes(&conteudo)));
        let mut recursos = String::from("<< /Font << ");
        for (i, n) in fontes.iter().enumerate() {
            let _ = write!(recursos, "/F{i} {n} 0 R ");
        }
        recursos.push_str(">> /XObject << ");
        for (nome, n) in &rec.imagens {
            let _ = write!(recursos, "/{nome} {n} 0 R ");
        }
        recursos.push_str(">> /ExtGState << ");
        for pct in &rec.alfas {
            let a = f(*pct as f32 / 100.0);
            let _ = write!(recursos, "/A{pct} << /ca {a} /CA {a} >> ");
        }
        recursos.push_str(">> >>");
        doc.definir(pagina, format!("<< /Type /Page /Parent {paginas} 0 R /MediaBox [0 0 {} {}] /Resources {recursos} /Contents {fluxo} 0 R >>", f(q.largura), f(q.altura)).into_bytes());
        filhos.push(pagina);
    }
    let lista: String = filhos.iter().map(|n| format!("{n} 0 R ")).collect();
    doc.definir(paginas, format!("<< /Type /Pages /Kids [{lista}] /Count {} >>", filhos.len()).into_bytes());
    doc.definir(raiz, format!("<< /Type /Catalog /Pages {paginas} 0 R >>").into_bytes());
    doc.serializar(raiz, info)
}

/// Uma cadeia hexadecimal UTF-16BE com BOM (o título com acentos).
fn utf16_hex(t: &str) -> Vec<u8> {
    let mut s = String::from("<FEFF");
    for u in t.encode_utf16() {
        let _ = write!(s, "{u:04X}");
    }
    s.push('>');
    s.into_bytes()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn winansi_escapa_e_troca() {
        assert_eq!(texto_winansi("a(b)\\ção – ✓"), b"(a\\(b\\)\\\\\xe7\xe3o \x96 ?)".to_vec());
    }

    #[test]
    fn cores() {
        assert_eq!(cor("#ff0000").unwrap().0, [1.0, 0.0, 0.0]);
        assert!((cor("#00000080").unwrap().1 - 0.5).abs() < 0.01);
        assert!(cor("").is_none());
    }
    #[test]
    fn novas_formas_exportam_o_contorno_do_motor() {
        for &(tipo, _) in featherslide_core::formas::ADICIONAIS {
            if featherslide_core::novas_formas::suporta(tipo) {
                let e = featherslide_core::modelo::Elemento {
                    tipo, x: 10.0, y: 20.0, largura: 200.0, altura: 100.0,
                    preenchimento: "#4285f4".into(), contorno: "#000000".into(),
                    ..Default::default()
                };
                assert!(!featherslide_core::novas_formas::primitivas(&e).is_empty());
            } else {
                let pontos = featherslide_core::formas::pontos(tipo.nome(), 10.0, 20.0, 200.0, 100.0).unwrap();
                let caminho = caminho_da_forma(tipo.nome(), 10.0, 20.0, 200.0, 100.0, &[], false);
                assert_eq!(caminho.len(), pontos.len() + 1);
                assert!(matches!(caminho[0], Seg::M(x, y) if x == pontos[0][0] && y == pontos[0][1]));
            }
        }
    }

}
/// Aparência personalizada com a mesma geometria usada pelo PDF.
/// Buffer limitado a 1024 px no maior lado; pixels retornados sem premultiplicação.
pub fn raster_forma(el: &ElementoDesenhado) -> Option<ImagemRgba> {
    use featherslide_core::estilo_forma::TipoGradiente;
    use tiny_skia::*;
    if !el.largura.is_finite()
        || !el.altura.is_finite()
        || el.largura <= 0.0
        || el.altura <= 0.0
        || !el.espessura.is_finite()
        || !(0.0..=50.0).contains(&el.espessura)
        || !el.opacidade.is_finite()
    {
        return None;
    }
    let escala = (1024.0 / el.largura.max(el.altura)).min(2.0);
    let w = (el.largura * escala).ceil().max(1.0) as u32;
    let h = (el.altura * escala).ceil().max(1.0) as u32;
    let borda = (margem_forma(el) * escala).round() as u32;
    let mut pix = Pixmap::new(w + 2 * borda, h + 2 * borda)?;
    let cor = |c: &str| -> Option<Color> {
        featherslide_core::estilo_forma::validar_cor(c, false).ok()?;
        let n = u32::from_str_radix(&c[1..7], 16).ok()?;
        Some(Color::from_rgba8(
            (n >> 16) as u8,
            (n >> 8) as u8,
            n as u8,
            if c.len() == 9 {
                u8::from_str_radix(&c[7..], 16).ok()?
            } else {
                255
            },
        ))
    };
    let mut paint = Paint::default();
    let preencher = if let Some(g) = &el.gradiente {
        g.validar().ok()?;
        let stops: Vec<_> = g
            .pontos
            .iter()
            .map(|p| Some(GradientStop::new(p.posicao, cor(&p.cor)?)))
            .collect::<Option<_>>()?;
        paint.shader = match g.tipo {
            TipoGradiente::Linear => {
                let (s, c) = g.angulo.to_radians().sin_cos();
                let raio = (el.largura * c.abs() + el.altura * s.abs()) / 2.0;
                LinearGradient::new(
                    Point::from_xy(el.largura / 2.0 - c * raio, el.altura / 2.0 - s * raio),
                    Point::from_xy(el.largura / 2.0 + c * raio, el.altura / 2.0 + s * raio),
                    stops,
                    SpreadMode::Pad,
                    Transform::identity(),
                )?
            }
            TipoGradiente::Radial => RadialGradient::new(
                Point::from_xy(el.largura / 2.0, el.altura / 2.0),
                Point::from_xy(el.largura / 2.0, el.altura / 2.0),
                el.largura.hypot(el.altura) / 2.0,
                stops,
                SpreadMode::Pad,
                Transform::identity(),
            )?,
        };
        true
    } else if let Some(c) = cor(&el.preenchimento) {
        paint.set_color(c);
        true
    } else {
        false
    };
    let mut paths = Vec::new();
    for p in &el.primitivas {
        if let Primitiva::Caminho { contornos, .. } = p {
            let mut b = PathBuilder::new();
            for cont in contornos {
                if let Some(a) = cont.first() {
                    b.move_to(a[0] - el.x, a[1] - el.y);
                    for p in &cont[1..] {
                        b.line_to(p[0] - el.x, p[1] - el.y);
                    }
                    b.close();
                }
            }
            if let Some(p) = b.finish() {
                paths.push(p);
            }
        }
    }
    if paths.is_empty() {
        let c = caminho_da_forma(el.tipo, 0.0, 0.0, el.largura, el.altura, &el.ajustes, false);
        let mut b = PathBuilder::new();
        for s in c {
            match s {
                Seg::M(x, y) => b.move_to(x, y),
                Seg::L(x, y) => b.line_to(x, y),
                Seg::C(x1, y1, x2, y2, x, y) => b.cubic_to(x1, y1, x2, y2, x, y),
                Seg::Z => b.close(),
            }
        }
        paths.push(b.finish()?);
    }
    let t = Transform::from_row(escala, 0.0, 0.0, escala, borda as f32, borda as f32);
    for p in paths {
        if preencher {
            pix.fill_path(&p, &paint, FillRule::Winding, t, None);
        }
        if let Some(c) = cor(&el.contorno) {
            if el.espessura > 0.0 {
                let mut stroke_paint = Paint::default();
                stroke_paint.set_color(c);
                pix.stroke_path(
                    &p,
                    &stroke_paint,
                    &Stroke {
                        width: el.espessura,
                        ..Stroke::default()
                    },
                    t,
                    None,
                );
            }
        }
    }
    let mut rgba = pix.data().to_vec();
    for p in rgba.chunks_exact_mut(4) {
        let a = p[3] as u32;
        if a > 0 {
            for v in &mut p[..3] {
                *v = ((*v as u32 * 255 + a / 2) / a).min(255) as u8;
            }
        }
        p[3] = (a as f32 * el.opacidade.clamp(0.0, 1.0)).round() as u8;
    }
    Some((pix.width(), pix.height(), rgba))
}

/// Camada de sombra com margem para preservar o desfoque fora da forma.
pub fn raster_sombra(el: &ElementoDesenhado) -> Option<(ImagemRgba, f32)> {
    let sombra = el.sombra.as_ref()?;
    let (w, h, px) = raster_forma(el)?;
    let borda = margem_forma(el);
    let escala = w as f32 / (el.largura + 2.0 * borda);
    let raio = (sombra.desfoque * escala).clamp(0.0, 28.0);
    let margem = (raio * 3.0).ceil() as u32 + 2;
    let mut img = image::RgbaImage::new(w + 2 * margem, h + 2 * margem);
    let origem = image::RgbaImage::from_raw(w, h, px)?;
    image::imageops::overlay(&mut img, &origem, margem as i64, margem as i64);
    let img = crate::imagem_compartilhada::gerar_sombra(&img, &sombra.cor, raio);
    Some((
        (img.width(), img.height(), img.into_raw()),
        margem as f32 / escala + borda,
    ))
}

/// Espaço externo reservado ao contorno e à antialiasing da camada raster.
pub fn margem_forma(el: &ElementoDesenhado) -> f32 {
    let escala = (1024.0 / el.largura.max(el.altura)).min(2.0);
    ((el.espessura.max(0.0) * escala / 2.0).ceil() + 1.0) / escala
}
