//! A apresentação em tela cheia: o slide no tamanho da tela, as animações de cada passo, a
//! transição entre slides e uma barra de controles discreta. O motor diz **o que** acontece
//! (os passos de animação, a transição de cada slide); aqui se conta o tempo e se pinta.

use std::time::Instant;

use featherslide_core::modelo::{Direcao, Transicao, TipoTransicao};
use featherslide_core::quadro::Quadro;
use pelicano::{Captured, Color, CursorIcon, Event, MouseButton, Point, Program, Rect as Rectangle, Renderer, Size, Theme};

use crate::desenho::{pintar_quadro, preencher_ret, Efeitos};
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
    /// Ponteiro laser (tecla L ou Ctrl+L, como no PowerPoint).
    pub laser: bool,
    /// Caneta (Ctrl+P): arrastar risca o slide.
    pub caneta: bool,
    /// Grade de slides (G ou -): todos em miniatura, clique vai a um deles.
    pub grade: bool,
    /// A tinta de cada slide (índice → traços em pontos do slide); fica até o fim da apresentação.
    pub tinta: std::collections::HashMap<usize, Vec<Vec<[f32; 2]>>>,
    pub exibido_em: Instant,
}

impl Default for Reproducao {
    fn default() -> Self {
        Self { passo: 0, inicio_passo: None, transicao: None, preta: false, branca: false, laser: false, caneta: false, grade: false, tinta: Default::default(), exibido_em: Instant::now() }
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
    /// Caneta: começa um traço / continua o traço (pontos do slide).
    TintaInicio([f32; 2]),
    TintaPonto([f32; 2]),
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
    /// Ponteiro laser: um ponto vermelho no lugar do mouse.
    pub laser: bool,
    /// Caneta ligada: o mouse risca em vez de avançar.
    pub caneta: bool,
    /// A tinta deste slide, em pontos do slide.
    pub tinta: Vec<Vec<[f32; 2]>>,
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
    /// O cursor em pontos do slide (tela cheia, sem zoom nem deslocamento).
    fn no_slide(&self, bounds: Rectangle, cursor: Option<Point>) -> Option<[f32; 2]> {
        let q = &self.quadro;
        let escala = (bounds.width / q.largura).min(bounds.height / q.altura);
        let (ox, oy) = ((bounds.width - q.largura * escala) / 2.0, (bounds.height - q.altura * escala) / 2.0);
        let p = cursor.filter(|p| bounds.contains(*p)).map(|p| Point::new(p.x - bounds.x, p.y - bounds.y))?;
        Some([(p.x - ox) / escala, (p.y - oy) / escala])
    }

    /// O gatilho ou o link sob o cursor, se houver (o gatilho tem prioridade).
    fn alvo(&self, bounds: Rectangle, cursor: Option<Point>) -> Option<Acao> {
        let [x, y] = self.no_slide(bounds, cursor)?;
        let dentro = |[gx, gy, gw, gh]: &[f32; 4]| x >= *gx && x <= gx + gw && y >= *gy && y <= gy + gh;
        if let Some((_, d)) = self.gatilhos.iter().rev().find(|(r, _)| dentro(r)) {
            return Some(Acao::Ir(*d));
        }
        self.links.iter().rev().find(|(r, _)| dentro(r)).map(|(_, id)| Acao::Link(*id))
    }
}

impl Program<Acao> for Programa {
    /// A roda acumulada (o pelicano reparte um passo da roda em vários eventos) e se a caneta está riscando.
    type State = (f32, bool);

    fn update(&self, estado: &mut (f32, bool), evento: &Event, bounds: Rectangle, cursor: Option<Point>, publish: &mut dyn FnMut(Acao)) -> Captured {
        let (roda, riscando) = (&mut estado.0, &mut estado.1);
        let dentro = cursor.is_some();
        if self.caneta && self.tela.is_none() {
            match *evento {
                Event::MousePressed(MouseButton::Left) if dentro => {
                    *riscando = true;
                    if let Some(p) = self.no_slide(bounds, cursor) { publish(Acao::TintaInicio(p)); }
                    return Captured::Yes;
                }
                Event::CursorMoved(_) if *riscando => {
                    if let Some(p) = self.no_slide(bounds, cursor) { publish(Acao::TintaPonto(p)); }
                    return Captured::Yes;
                }
                Event::MouseReleased(MouseButton::Left) => *riscando = false,
                _ => {}
            }
        }
        let acao = match *evento {
            Event::MousePressed(MouseButton::Left) if dentro && (!self.gatilhos.is_empty() || !self.links.is_empty()) && self.tela.is_none() => {
                Some(self.alvo(bounds, cursor).unwrap_or(Acao::Proximo))
            }
            Event::MousePressed(MouseButton::Left) if dentro => Some(Acao::Proximo),
            Event::MousePressed(MouseButton::Right) if dentro => Some(Acao::Anterior),
            Event::Wheel { dy, .. } => {
                *roda += dy;
                // Um passo da roda (48 px) = um slide.
                let acao = if *roda <= -24.0 { Some(Acao::Proximo) } else if *roda >= 24.0 { Some(Acao::Anterior) } else { None };
                if acao.is_some() {
                    *roda = 0.0;
                }
                return if acao.is_some() || dy != 0.0 { acao.into_iter().for_each(publish); Captured::Yes } else { Captured::No };
            }
            _ => None,
        };
        match acao {
            Some(a) => {
                publish(a);
                Captured::Yes
            }
            None => Captured::No,
        }
    }

    fn mouse_interaction(&self, _: &(f32, bool), bounds: Rectangle, cursor: Option<Point>) -> CursorIcon {
        if (self.laser || self.caneta) && self.tela.is_none() {
            CursorIcon::Crosshair
        } else if self.tela.is_none() && self.alvo(bounds, cursor).is_some() {
            CursorIcon::Pointer
        } else {
            CursorIcon::Default
        }
    }

    fn draw(&self, _: &(f32, bool), frame: &mut Renderer, bounds: Rectangle, _: &Theme, cursor: Option<Point>) {
        let canto = Point::new(bounds.x, bounds.y);
        preencher_ret(frame, canto, bounds.size(), self.fundo_externo);
        if let Some(cor) = self.tela {
            preencher_ret(frame, canto, bounds.size(), cor);
            return;
        }

        let q = &self.quadro;
        let escala = (bounds.width / q.largura).min(bounds.height / q.altura) * self.escala_relativa;
        let (sw, sh) = (q.largura * escala, q.altura * escala);
        // Em coordenadas da janela.
        let (ox, oy) = (bounds.x + (bounds.width - sw) / 2.0 + self.pan.0, bounds.y + (bounds.height - sh) / 2.0 + self.pan.1);
        let sem_efeitos = Efeitos::new();

        // Sem `with_clip` (com antialiasing ele fazia o slide sumir): tudo se pinta em coordenadas
        // da janela e, no fim, o que passou da borda do slide é coberto de preto.
        let f = &mut *frame;
        let pintar = |f: &mut Renderer, quadro: &Quadro, dx: f32, dy: f32, efeitos: Option<&Efeitos>| {
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
                        // A camada escala em volta do centro do slide.
                        f.push_layer(Rectangle::new(Point::new(ox, oy), Size::new(sw, sh)), 0.6 + 0.4 * e, 0.0, 1.0);
                        pintar(f, q, 0.0, 0.0, Some(&self.efeitos));
                        f.pop_layer();
                        OPACIDADE.with(|o| o.set(1.0));
                    }
                }
            }
        }
        // A moldura preta: o que a transição desenhou fora do slide some aqui.
        let (bx, by, w, h) = (bounds.x, bounds.y, bounds.width, bounds.height);
        let (lx, ly) = (ox - bx, oy - by);
        preencher_ret(frame, Point::new(bx, by), Size::new(w, ly.max(0.0)), self.fundo_externo);
        preencher_ret(frame, Point::new(bx, oy + sh), Size::new(w, (h - ly - sh).max(0.0)), self.fundo_externo);
        preencher_ret(frame, Point::new(bx, oy), Size::new(lx.max(0.0), sh), self.fundo_externo);
        preencher_ret(frame, Point::new(ox + sw, oy), Size::new((w - lx - sw).max(0.0), sh), self.fundo_externo);
        // A tinta da caneta, por cima do slide (não durante a transição: o slide ainda está entrando).
        if self.transicao.is_none() {
            let traco = pelicano::Stroke::new((3.0 * escala).max(2.0)).cap(pelicano::LineCap::Round).join(pelicano::LineJoin::Round);
            for t in self.tinta.iter().filter(|t| !t.is_empty()) {
                let ponto = |p: &[f32; 2]| Point::new(ox + p[0] * escala, oy + p[1] * escala);
                let caminho = pelicano::Path::new(|b| {
                    b.move_to(ponto(&t[0]));
                    // Um clique sem arrasto vira um pontinho.
                    if t.len() == 1 { b.line_to(Point::new(ponto(&t[0]).x + 0.1, ponto(&t[0]).y)); }
                    for p in &t[1..] { b.line_to(ponto(p)); }
                });
                if let Some(c) = caminho { frame.stroke_path(&c, &traco, Color::rgb(0.9, 0.1, 0.1)); }
            }
        }
        if let Some(c) = cursor.filter(|c| self.laser && bounds.contains(*c)) {
            // O brilho em volta e o ponto, do tamanho do laser do PowerPoint.
            for (raio, cor) in [(14.0, Color::rgba(1.0, 0.1, 0.1, 0.25)), (7.0, Color::rgb(0.95, 0.05, 0.05))] {
                let caixa = Rectangle::new(Point::new(c.x - raio, c.y - raio), Size::new(raio * 2.0, raio * 2.0));
                frame.fill_quad(caixa, Some(&cor.into()), pelicano::Border { radius: [raio; 4], ..Default::default() }, None);
            }
        }
    }
}
