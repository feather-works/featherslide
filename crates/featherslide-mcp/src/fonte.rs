//! Renderização de fontes para imagens de slides.
//! Adaptado de `feathercad/crates/feathercad-mcp/src/fonte.rs`.
//! Usa uma fonte do sistema operacional (Liberation, DejaVu, Arial, Roboto…) via `ab_glyph`;
//! se nenhuma for encontrada, recorre a glifos básicos em bitmap.

use ab_glyph::{point, Font, FontVec, PxScale, ScaleFont};
use tiny_skia::Pixmap;

const CAMINHOS_FONTE: [&str; 8] = [
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    "/Library/Fonts/Arial.ttf",
    "C:\\Windows\\Fonts\\arial.ttf",
];

pub enum Fonte {
    Sistema(FontVec),
    Basica,
}

/// Carrega uma fonte do sistema operacional; se não encontrar, usa glifos básicos.
pub fn carregar() -> Fonte {
    for c in CAMINHOS_FONTE {
        if let Ok(bytes) = std::fs::read(c) {
            if let Ok(f) = FontVec::try_from_vec(bytes) {
                return Fonte::Sistema(f);
            }
        }
    }
    Fonte::Basica
}

/// Ponto de ancoragem do texto.
#[derive(Clone, Copy)]
pub struct Ancora {
    pub h: u8, // 0 = esquerda, 1 = centro, 2 = direita
    pub v: u8, // 0 = linha de base, 1 = base, 2 = meio, 3 = topo
}

pub const ESQUERDA_BASE: Ancora = Ancora { h: 0, v: 0 };
pub const CENTRO_MEIO: Ancora = Ancora { h: 1, v: 2 };

fn misturar(pix: &mut Pixmap, x: i64, y: i64, rgb: [u8; 3], cobertura: f32) {
    if x < 0 || y < 0 || x >= i64::from(pix.width()) || y >= i64::from(pix.height()) || cobertura <= 0.0 {
        return;
    }
    let i = ((y as u32 * pix.width() + x as u32) * 4) as usize;
    let dados = pix.data_mut();
    for k in 0..3 {
        let antes = f32::from(dados[i + k]);
        dados[i + k] = (antes + (f32::from(rgb[k]) - antes) * cobertura.min(1.0)).round() as u8;
    }
}

struct Pintado {
    dx: f32,
    dy: f32,
    c: f32,
}

fn glifos_do_sistema(f: &FontVec, texto: &str, altura: f32) -> (Vec<Pintado>, f32) {
    let escala = PxScale::from(altura);
    let s = f.as_scaled(escala);
    let mut x = 0.0f32;
    let mut fora = Vec::new();
    for ch in texto.chars() {
        let id = s.glyph_id(ch);
        if let Some(o) = f.outline_glyph(id.with_scale_and_position(escala, point(x, 0.0))) {
            let b = o.px_bounds();
            o.draw(|gx, gy, c| fora.push(Pintado { dx: b.min.x + gx as f32, dy: b.min.y + gy as f32, c }));
        }
        x += s.h_advance(id);
    }
    (fora, x)
}

fn bitmap(ch: char) -> [u8; 5] {
    match ch {
        '0' => [7, 5, 5, 5, 7],
        '1' => [2, 6, 2, 2, 7],
        '2' => [7, 1, 7, 4, 7],
        '3' => [7, 1, 7, 1, 7],
        '4' => [5, 5, 7, 1, 1],
        '5' => [7, 4, 7, 1, 7],
        '6' => [7, 4, 7, 5, 7],
        '7' => [7, 1, 1, 1, 1],
        '8' => [7, 5, 7, 5, 7],
        '9' => [7, 5, 7, 1, 7],
        '.' | ',' => [0, 0, 0, 0, 2],
        '-' => [0, 0, 7, 0, 0],
        '#' => [5, 7, 5, 7, 5],
        ':' => [0, 2, 0, 2, 0],
        'a' | 'A' => [7, 5, 7, 5, 5],
        'b' | 'B' => [6, 5, 7, 5, 6],
        'c' | 'C' => [7, 4, 4, 4, 7],
        'd' | 'D' => [6, 5, 5, 5, 6],
        'e' | 'E' => [7, 4, 7, 4, 7],
        's' | 'S' => [7, 4, 7, 1, 7],
        't' | 'T' => [7, 2, 2, 2, 2],
        'x' | 'X' => [5, 5, 2, 5, 5],
        'y' | 'Y' => [5, 5, 2, 2, 2],
        _ => [0, 0, 0, 0, 0],
    }
}

fn glifos_basicos(texto: &str, altura: f32) -> (Vec<Pintado>, f32) {
    let ampliacao = (altura / 7.0).round().max(1.0);
    let mut fora = Vec::new();
    let mut x = 0.0f32;
    for ch in texto.chars() {
        for (linha, bits) in bitmap(ch).iter().enumerate() {
            for col in 0..3 {
                if bits & (4 >> col) != 0 {
                    for py in 0..ampliacao as i32 {
                        for px in 0..ampliacao as i32 {
                            fora.push(Pintado {
                                dx: x + col as f32 * ampliacao + px as f32,
                                dy: (linha as f32 - 5.0) * ampliacao + py as f32,
                                c: 1.0,
                            });
                        }
                    }
                }
            }
        }
        x += 4.0 * ampliacao;
    }
    (fora, x)
}

/// Escreve `texto` com as coordenadas, escala, cor e âncora especificadas.
#[allow(clippy::too_many_arguments)]
pub fn escrever(pix: &mut Pixmap, fonte: &Fonte, texto: &str, x: f32, y: f32, altura: f32, giro: f32, rgb: [u8; 3], ancora: Ancora, halo: bool) {
    let altura = altura.clamp(4.0, 400.0);
    let (glifos, largura) = match fonte {
        Fonte::Sistema(f) => glifos_do_sistema(f, texto, altura),
        Fonte::Basica => glifos_basicos(texto, altura),
    };
    let ox = -largura * f32::from(ancora.h) / 2.0;
    let oy = match ancora.v {
        1 => -altura * 0.2,
        2 => altura * 0.35,
        3 => altura * 0.8,
        _ => 0.0,
    };
    let (s, c) = giro.sin_cos();
    let mut pintar = |dx: f32, dy: f32, cobertura: f32, cor: [u8; 3]| {
        let (u, v) = (dx + ox, dy + oy);
        misturar(pix, (x + u * c - v * s).round() as i64, (y + u * s + v * c).round() as i64, cor, cobertura);
    };
    if halo {
        for g in &glifos {
            for (ax, ay) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
                pintar(g.dx + ax, g.dy + ay, g.c * 0.9, [255, 255, 255]);
            }
        }
    }
    for g in &glifos {
        pintar(g.dx, g.dy, g.c, rgb);
    }
}
