//! A apresentação em tela cheia: o slide no tamanho da tela, as animações de cada passo, a
//! transição entre slides e uma barra de controles discreta. O motor diz **o que** acontece
//! (os passos de animação, a transição de cada slide); aqui se conta o tempo e se pinta.

use std::time::Instant;

use featherslide_core::modelo::{Direcao, Transicao, TipoTransicao};
use featherslide_core::quadro::Quadro;
use iced::mouse;
use iced::widget::canvas::{self, event, Frame, Geometry, Program};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme, Vector};

use crate::desenho::{pintar_quadro, Efeitos};
use crate::palco::OPACIDADE;

/// Uma transição acontecendo: o slide de saída (já com todas as animações feitas), o efeito e o início.
pub struct EmCurso {
    pub de: Quadro,
    pub tr: Transicao,
    pub inicio: Instant,
}

/// Onde a apresentação está: quantos passos de animação já foram disparados, a transição
/// em curso, as telas preta/branca e há quanto tempo o slide está na tela.
pub struct Reproducao {
    /// Quantos passos de animação do slide atual já foram disparados.
    pub passo: usize,
    /// Quando o último passo começou (`None`: já terminou).
    pub inicio_passo: Option<Instant>,
    pub transicao: Option<EmCurso>,
    pub preta: bool,
    pub branca: bool,
    pub exibido_em: Instant,
}

impl Default for Reproducao {
    fn default() -> Self {
        Self { passo: 0, inicio_passo: None, transicao: None, preta: false, branca: false, exibido_em: Instant::now() }
    }
}

impl Reproducao {
    /// O andamento (0 a 1) da transição em curso.
    pub fn progresso(&self) -> Option<f32> {
        let t = self.transicao.as_ref()?;
        Some((t.inicio.elapsed().as_secs_f32() / t.tr.duracao.max(0.05)).clamp(0.0, 1.0))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Acao {
    Proximo,
    Anterior,
    /// Clicou num gatilho: vai para o slide.
    Ir(usize),
    /// Clicou num elemento com hiperlink (o id do elemento).
    Link(u64),
}

/// O que o canvas da apresentação pinta num instante.
pub struct Programa {
    pub quadro: Quadro,
    /// Fundo fora do slide: preto na apresentação, cor da mesa na prévia do editor.
    pub fundo_externo: Color,
    /// Enquadramento do slide: margem e zoom do editor, ou 1 na tela cheia.
    pub escala_relativa: f32,
    pub pan: (f32, f32),
    pub efeitos: Efeitos,
    /// A transição: o slide que sai, o efeito e o andamento (0 a 1).
    pub transicao: Option<(Quadro, Transicao, f32)>,
    /// Tela preta ou branca (teclas B e W).
    pub tela: Option<Color>,
    /// Os gatilhos do slide: retângulo (pontos do slide) e o slide de destino.
    pub gatilhos: Vec<([f32; 4], usize)>,
    /// Os elementos com hiperlink: retângulo (pontos do slide) e o id do elemento.
    pub links: Vec<([f32; 4], u64)>,
}

fn suavizar(p: f32) -> f32 {
    p * p * (3.0 - 2.0 * p)
}

/// De onde o slide novo vem, como vetor unitário.
fn de_onde(d: Direcao) -> (f32, f32) {
    match d {
        Direcao::Esquerda => (-1.0, 0.0),
        Direcao::Direita => (1.0, 0.0),
        Direcao::Cima => (0.0, -1.0),
        Direcao::Baixo => (0.0, 1.0),
    }
}

impl Programa {
    /// O gatilho ou o link sob o cursor, se houver (o gatilho tem prioridade).
    fn alvo(&self, bounds: Rectangle, cursor: mouse::Cursor) -> Option<Acao> {
        let q = &self.quadro;
        let escala = (bounds.width / q.largura).min(bounds.height / q.altura);
        let (ox, oy) = ((bounds.width - q.largura * escala) / 2.0, (bounds.height - q.altura * escala) / 2.0);
        let p = cursor.position_in(bounds)?;
        let (x, y) = ((p.x - ox) / escala, (p.y - oy) / escala);
        let dentro = |[gx, gy, gw, gh]: &[f32; 4]| x >= *gx && x <= gx + gw && y >= *gy && y <= gy + gh;
        if let Some((_, d)) = self.gatilhos.iter().rev().find(|(r, _)| dentro(r)) {
            return Some(Acao::Ir(*d));
        }
        self.links.iter().rev().find(|(r, _)| dentro(r)).map(|(_, id)| Acao::Link(*id))
    }
}

impl Program<Acao> for Programa {
    type State = ();

    fn update(&self, _: &mut (), evento: canvas::Event, bounds: Rectangle, cursor: mouse::Cursor) -> (event::Status, Option<Acao>) {
        let dentro = cursor.position().is_some();
        match evento {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if dentro && (!self.gatilhos.is_empty() || !self.links.is_empty()) && self.tela.is_none() => {
                let acao = self.alvo(bounds, cursor).unwrap_or(Acao::Proximo);
                (event::Status::Captured, Some(acao))
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if dentro => (event::Status::Captured, Some(Acao::Proximo)),
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) if dentro => (event::Status::Captured, Some(Acao::Anterior)),
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let y = match delta {
                    mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => y,
                };
                if y < 0.0 {
                    (event::Status::Captured, Some(Acao::Proximo))
                } else if y > 0.0 {
                    (event::Status::Captured, Some(Acao::Anterior))
                } else {
                    (event::Status::Ignored, None)
                }
            }
            _ => (event::Status::Ignored, None),
        }
    }

    fn mouse_interaction(&self, _: &(), bounds: Rectangle, cursor: mouse::Cursor) -> mouse::Interaction {
        if self.tela.is_none() && self.alvo(bounds, cursor).is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(&self, _: &(), renderer: &Renderer, _: &Theme, bounds: Rectangle, _: mouse::Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), self.fundo_externo);
        if let Some(cor) = self.tela {
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), cor);
            return vec![frame.into_geometry()];
        }

        let q = &self.quadro;
        let escala = (bounds.width / q.largura).min(bounds.height / q.altura) * self.escala_relativa;
        let (sw, sh) = (q.largura * escala, q.altura * escala);
        let (ox, oy) = ((bounds.width - sw) / 2.0 + self.pan.0, (bounds.height - sh) / 2.0 + self.pan.1);
        let sem_efeitos = Efeitos::new();

        // Sem `with_clip` (com antialiasing ele fazia o slide sumir): tudo se pinta em coordenadas
        // da janela e, no fim, o que passou da borda do slide é coberto de preto.
        let f = &mut frame;
        let pintar = |f: &mut Frame, quadro: &Quadro, dx: f32, dy: f32, efeitos: Option<&Efeitos>| {
            pintar_quadro(f, quadro, ox + dx, oy + dy, escala, &[], (0.0, 0.0), 1.0, efeitos);
        };
        match &self.transicao {
            None => pintar(f, q, 0.0, 0.0, Some(&self.efeitos)),
            Some((antigo, tr, p)) => {
                let e = suavizar(*p);
                let (vx, vy) = de_onde(tr.direcao);
                let (novo_dx, novo_dy) = (vx * sw * (1.0 - e), vy * sh * (1.0 - e));
                match tr.tipo {
                    TipoTransicao::Nenhuma => pintar(f, q, 0.0, 0.0, Some(&self.efeitos)),
                    // A cortina (varredura) pedia recorte; sem ele, fica como o esmaecer.
                    TipoTransicao::Esmaecer | TipoTransicao::Cortina => {
                        pintar(f, antigo, 0.0, 0.0, Some(&sem_efeitos));
                        OPACIDADE.with(|o| o.set(e));
                        pintar(f, q, 0.0, 0.0, Some(&self.efeitos));
                        OPACIDADE.with(|o| o.set(1.0));
                    }
                    TipoTransicao::Deslizar => {
                        pintar(f, antigo, 0.0, 0.0, Some(&sem_efeitos));
                        pintar(f, q, novo_dx, novo_dy, Some(&self.efeitos));
                    }
                    TipoTransicao::Empurrar => {
                        pintar(f, antigo, -vx * sw * e, -vy * sh * e, Some(&sem_efeitos));
                        pintar(f, q, novo_dx, novo_dy, Some(&self.efeitos));
                    }
                    TipoTransicao::Revelar => {
                        pintar(f, q, 0.0, 0.0, Some(&self.efeitos));
                        pintar(f, antigo, -vx * sw * e, -vy * sh * e, Some(&sem_efeitos));
                    }
                    TipoTransicao::Zoom => {
                        pintar(f, antigo, 0.0, 0.0, Some(&sem_efeitos));
                        OPACIDADE.with(|o| o.set(e));
                        f.with_save(|g| {
                            let centro = Point::new(ox + sw / 2.0, oy + sh / 2.0);
                            g.translate(Vector::new(centro.x, centro.y));
                            g.scale(0.6 + 0.4 * e);
                            g.translate(Vector::new(-centro.x, -centro.y));
                            pintar(g, q, 0.0, 0.0, Some(&self.efeitos));
                        });
                        OPACIDADE.with(|o| o.set(1.0));
                    }
                }
            }
        }
        // A moldura preta: o que a transição desenhou fora do slide some aqui.
        let (w, h) = (bounds.width, bounds.height);
        frame.fill_rectangle(Point::new(0.0, 0.0), Size::new(w, oy.max(0.0)), self.fundo_externo);
        frame.fill_rectangle(Point::new(0.0, oy + sh), Size::new(w, (h - oy - sh).max(0.0)), self.fundo_externo);
        frame.fill_rectangle(Point::new(0.0, oy), Size::new(ox.max(0.0), sh), self.fundo_externo);
        frame.fill_rectangle(Point::new(ox + sw, oy), Size::new((w - ox - sw).max(0.0), sh), self.fundo_externo);
        vec![frame.into_geometry()]
    }
}
