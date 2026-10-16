//! As réguas (Exibir → Régua): faixas em centímetros nas bordas de cima e da esquerda do
//! palco, com o zero no centro do slide (como no PowerPoint). Mostram onde o slide começa e
//! termina, a extensão da seleção e a posição do mouse. Só desenho: as medidas vêm do quadro.

use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::{Color, Point, Size};

use crate::tema::*;

/// Largura da faixa da régua, em pixels.
pub const FAIXA: f32 = 20.0;
const PT_POR_CM: f32 = 72.0 / 2.54;
/// Distâncias candidatas entre os números, em cm.
const PASSOS_CM: [f32; 9] = [0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0];

/// O passo (em cm) entre números tal que fiquem a pelo menos 46 px um do outro.
fn passo_dos_numeros(escala: f32) -> f32 {
    PASSOS_CM.into_iter().find(|p| p * PT_POR_CM * escala >= 46.0).unwrap_or(100.0)
}

/// O que a régua precisa saber do palco.
pub struct Medidas {
    pub area: Size,
    /// Canto de cima à esquerda do slide e escala (pixels por ponto).
    pub ox: f32,
    pub oy: f32,
    pub escala: f32,
    /// Tamanho do slide em pontos.
    pub largura: f32,
    pub altura: f32,
    /// A extensão da seleção (x, y, w, h em pontos), se houver.
    pub selecao: Option<[f32; 4]>,
    pub cursor: Option<Point>,
}

pub fn desenhar(frame: &mut Frame, m: &Medidas) {
    let (fundo, tinta, destaque) = (COR_CROMO_2(), COR_TEXTO_MUTED(), COR_PAINEL());
    let acento = COR_AZUL_ACENTO();
    let (sw, sh) = (m.largura * m.escala, m.altura * m.escala);
    // Cima (horizontal) e esquerda (vertical).
    frame.fill_rectangle(Point::new(0.0, 0.0), Size::new(m.area.width, FAIXA), fundo);
    frame.fill_rectangle(Point::new(0.0, 0.0), Size::new(FAIXA, m.area.height), fundo);
    frame.fill_rectangle(Point::new(m.ox.max(FAIXA), 0.0), Size::new((sw - (FAIXA - m.ox).max(0.0)).max(0.0), FAIXA), destaque);
    frame.fill_rectangle(Point::new(0.0, m.oy.max(FAIXA)), Size::new(FAIXA, (sh - (FAIXA - m.oy).max(0.0)).max(0.0)), destaque);
    if let Some([x, y, w, h]) = m.selecao {
        let banda = Color { a: 0.28, ..acento };
        frame.fill_rectangle(Point::new(m.ox + x * m.escala, 0.0), Size::new(w * m.escala, FAIXA), banda);
        frame.fill_rectangle(Point::new(0.0, m.oy + y * m.escala), Size::new(FAIXA, h * m.escala), banda);
    }
    marcas(frame, m, true, tinta);
    marcas(frame, m, false, tinta);
    let traco = Stroke::default().with_color(COR_BORDA()).with_width(1.0);
    frame.stroke(&Path::line(Point::new(0.0, FAIXA), Point::new(m.area.width, FAIXA)), traco);
    frame.stroke(&Path::line(Point::new(FAIXA, 0.0), Point::new(FAIXA, m.area.height)), traco);
    if let Some(c) = m.cursor {
        let linha = Stroke::default().with_color(acento).with_width(1.0);
        if c.x > FAIXA && c.y > FAIXA {
            frame.stroke(&Path::line(Point::new(c.x, 0.0), Point::new(c.x, FAIXA)), linha);
            frame.stroke(&Path::line(Point::new(0.0, c.y), Point::new(FAIXA, c.y)), linha);
        }
    }
}

/// Os tracinhos e os números de uma das réguas.
fn marcas(frame: &mut Frame, m: &Medidas, horizontal: bool, tinta: Color) {
    let (origem, comprimento, lado_slide) = if horizontal { (m.ox, m.area.width, m.largura) } else { (m.oy, m.area.height, m.altura) };
    let px_por_cm = PT_POR_CM * m.escala;
    let numeros = passo_dos_numeros(m.escala);
    let menor = [numeros / 5.0, numeros / 2.0].into_iter().find(|p| p * px_por_cm >= 5.0);
    // O zero fica no centro do slide.
    let zero = origem + lado_slide * m.escala / 2.0;
    let traco = Stroke::default().with_color(tinta).with_width(1.0);
    let (de, ate) = (((FAIXA - zero) / px_por_cm / numeros).floor() as i32 - 1, ((comprimento - zero) / px_por_cm / numeros).ceil() as i32 + 1);
    for n in de..=ate {
        let cm = n as f32 * numeros;
        let pos = zero + cm * px_por_cm;
        if let Some(p) = menor {
            let subs = (numeros / p).round() as i32;
            for s in 1..subs {
                let q = pos + s as f32 * p * px_por_cm;
                if q > FAIXA && q < comprimento {
                    tracinho(frame, q, 3.0, horizontal, traco);
                }
            }
        }
        if pos > FAIXA && pos < comprimento {
            tracinho(frame, pos, 8.0, horizontal, traco);
            let rotulo = if (cm - cm.round()).abs() < 0.001 { format!("{}", cm.abs().round() as i64) } else { format!("{}", cm.abs()) };
            let ponto = if horizontal { Point::new(pos + 2.0, 1.0) } else { Point::new(1.0, pos + 2.0) };
            frame.fill_text(Text { content: rotulo, position: ponto, color: tinta, size: 9.0.into(), ..Text::default() });
        }
    }
}

fn tracinho(frame: &mut Frame, pos: f32, tamanho: f32, horizontal: bool, traco: Stroke) {
    let (a, b) = if horizontal { (Point::new(pos, FAIXA - tamanho), Point::new(pos, FAIXA)) } else { (Point::new(FAIXA - tamanho, pos), Point::new(FAIXA, pos)) };
    frame.stroke(&Path::line(a, b), traco);
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn passo_cresce_quando_a_escala_diminui() {
        assert!(passo_dos_numeros(0.5) > passo_dos_numeros(2.0));
        assert!(passo_dos_numeros(1.0) * PT_POR_CM >= 46.0);
    }
}
