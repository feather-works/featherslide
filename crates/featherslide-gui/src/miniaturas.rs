//! O painel lateral de slides: miniaturas, seções (pastas recolhíveis) e a marca de slide oculto.

use std::collections::{BTreeSet, HashSet};

use featherslide_core::quadro::Quadro;
use featherslide_core::SecaoInfo;
use iced::border::Radius;
use iced::mouse;
use iced::widget::canvas::{self, Frame, Text};
use iced::widget::{button, column, container, row, scrollable, stack, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length, Point, Rectangle, Renderer, Shadow, Size, Theme, Vector};

use crate::desenho::pintar_quadro;
use crate::icones::*;
use crate::palco::OPACIDADE;
use crate::tema::*;

#[derive(Debug, Clone)]
pub struct InfoMiniatura {
    pub indice: usize,
    pub quadro: Option<Quadro>,
    pub oculto: bool,
}

/// A miniatura é o próprio slide pintado em escala pequena (apagada se o slide está oculto).
pub struct Miniatura {
    pub quadro: Option<Quadro>,
    pub indice: Option<usize>,
    pub oculto: bool,
    pub arrastando: bool,
}

impl Miniatura {
    pub fn nova(quadro: Option<Quadro>, indice: Option<usize>, oculto: bool) -> Self {
        Self { quadro, indice, oculto, arrastando: false }
    }

    pub fn com_arrasto(mut self, arrastando: bool) -> Self {
        self.arrastando = arrastando;
        self
    }
}

/// O que a miniatura guarda entre um quadro e outro: a geometria pronta e de que ela foi feita.
#[derive(Default)]
pub struct EstadoMiniatura {
    cache: canvas::Cache,
    chave: std::cell::Cell<u64>,
}

impl canvas::Program<MiniaturaMensagem> for Miniatura {
    type State = EstadoMiniatura;

    fn update(&self, _: &mut EstadoMiniatura, evento: canvas::Event, bounds: Rectangle, cursor: mouse::Cursor) -> (iced::event::Status, Option<MiniaturaMensagem>) {
        let (Some(i), Some(pos)) = (self.indice, cursor.position().filter(|p| bounds.contains(*p))) else { return (iced::event::Status::Ignored, None) };
        match evento {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => (iced::event::Status::Captured, Some(MiniaturaMensagem::Menu(i, pos))),
            // Apertar começa um possível arrasto; soltar na mesma miniatura é o clique.
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => (iced::event::Status::Captured, Some(MiniaturaMensagem::Pressionou(i))),
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => (iced::event::Status::Captured, Some(MiniaturaMensagem::Soltou(i))),
            _ => (iced::event::Status::Ignored, None),
        }
    }

    fn draw(&self, estado: &EstadoMiniatura, renderer: &Renderer, _: &Theme, bounds: Rectangle, _: mouse::Cursor) -> Vec<canvas::Geometry> {
        // Pinta de novo só se o slide, o tema ou a marca de oculto mudaram.
        let chave = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            (self.quadro.as_ref().map(|q| q.versao), self.oculto, self.arrastando, escuro(), acento_hex()).hash(&mut h);
            // A versão recomeça em cada apresentação: o conteúdo também entra na conta.
            if let Some(q) = &self.quadro {
                q.fundo.hash(&mut h);
                for e in &q.elementos {
                    (e.id, e.x.to_bits(), e.y.to_bits(), e.largura.to_bits(), e.altura.to_bits(), &e.preenchimento, e.rotacao.to_bits()).hash(&mut h);
                    e.linhas.first().map(|l| &l.texto).hash(&mut h);
                }
            }
            h.finish()
        };
        if estado.chave.get() != chave {
            estado.cache.clear();
            estado.chave.set(chave);
        }
        vec![estado.cache.draw(renderer, bounds.size(), |frame| {
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::WHITE);
            if let Some(q) = &self.quadro {
                let escala = (bounds.width / q.largura).min(bounds.height / q.altura);
                let ox = (bounds.width - q.largura * escala) / 2.0;
                let oy = (bounds.height - q.altura * escala) / 2.0;
                let antes = OPACIDADE.with(|o| o.get());
                if self.oculto || self.arrastando {
                    OPACIDADE.with(|o| o.set(0.35));
                }
                pintar_quadro(frame, q, ox, oy, escala, &[], (0.0, 0.0), 1.0, None);
                OPACIDADE.with(|o| o.set(antes));
            }
            if self.oculto || self.arrastando {
                // Véu por cima, para o slide parecer apagado/levantado no arrasto.
                frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::from_rgba8(255, 255, 255, 0.55));
            }
        })]
    }
}

/// O cabeçalho de seção: clique recolhe/expande, arrasto move o grupo inteiro,
/// botão direito abre o menu da seção.
struct Cabecalho {
    inicio: usize,
    nome: String,
    quantos: usize,
    aberta: bool,
    /// Índice do cabeçalho de origem durante um arrasto, para a prévia visual.
    arrastando_secao: Option<usize>,
}

impl canvas::Program<MiniaturaMensagem> for Cabecalho {
    type State = ();

    fn update(&self, _: &mut (), evento: canvas::Event, bounds: Rectangle, cursor: mouse::Cursor) -> (iced::event::Status, Option<MiniaturaMensagem>) {
        let Some(pos) = cursor.position().filter(|p| bounds.contains(*p)) else { return (iced::event::Status::Ignored, None) };
        match evento {
            // A decisão clique x arrasto ocorre no app ao soltar: não recolher no pressionar.
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => (iced::event::Status::Captured, Some(MiniaturaMensagem::PressionouSecao(self.inicio, pos))),
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => (iced::event::Status::Captured, Some(MiniaturaMensagem::SoltouSecao(self.inicio))),
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => (iced::event::Status::Captured, Some(MiniaturaMensagem::MenuSecao(self.inicio, pos))),
            _ => (iced::event::Status::Ignored, None),
        }
    }

    fn draw(&self, _: &(), renderer: &Renderer, _: &Theme, bounds: Rectangle, cursor: mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let sobre = cursor.is_over(bounds);
        if sobre {
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), COR_HOVER());
        }
        if let Some(origem) = self.arrastando_secao {
            if origem == self.inicio {
                frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::from_rgba8(65, 120, 205, 0.13));
            } else if sobre {
                // Linha de inserção: acima do alvo quando subindo; abaixo quando descendo.
                frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::from_rgba8(65, 120, 205, 0.10));
                let y = if origem > self.inicio { 0.0 } else { bounds.height - 3.0 };
                frame.fill_rectangle(Point::new(0.0, y), Size::new(bounds.width, 3.0), COR_AZUL_ACENTO());
            }
        }
        // A seta: para baixo (aberta) ou para a direita (recolhida).
        let cor = COR_TEXTO_MUTED();
        let (cx, cy) = (11.0, bounds.height / 2.0);
        pintar_no_canvas(&mut frame, if self.aberta { "chevron-down" } else { "chevron-right" }, cx - 7.0, cy - 7.0, 14.0, cor);
        frame.fill_text(Text {
            content: format!("{}  ({})", self.nome, self.quantos),
            position: Point::new(22.0, cy),
            color: COR_TEXTO(),
            size: 12.5.into(),
            font: iced::Font { weight: iced::font::Weight::Semibold, ..iced::Font::DEFAULT },
            vertical_alignment: iced::alignment::Vertical::Center,
            ..Text::default()
        });
        vec![frame.into_geometry()]
    }
}

#[derive(Debug, Clone)]
pub enum MiniaturaMensagem {
    /// Apertou o botão esquerdo na miniatura (pode virar clique ou arrasto).
    Pressionou(usize),
    /// Soltou o botão esquerdo sobre a miniatura.
    Soltou(usize),
    Novo,
    /// Uma ação sobre os slides marcados.
    Lote(Lote),
    /// Botão direito na miniatura: o slide e onde (pixels da janela).
    Menu(usize, Point),
    /// Início do gesto de arrastar pelo cabeçalho (posição na janela).
    PressionouSecao(usize, Point),
    /// Fim do gesto sobre um cabeçalho (mesmo = clique; outro = mover).
    SoltouSecao(usize),
    /// Botão direito no cabeçalho de uma seção.
    MenuSecao(usize, Point),
    RenomearTexto(String),
    RenomearConfirmar,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lote {
    NovaSecao,
}

/// O id do campo de renomear seção (para dar foco).
pub const ID_RENOMEAR: &str = "renomear_secao";

pub fn view_miniaturas<'a>(
    slides: &[InfoMiniatura],
    slide_atual: usize,
    selecionados: &BTreeSet<usize>,
    arrastando: Option<usize>,
    arrastando_secao: Option<usize>,
    secoes: &[SecaoInfo],
    recolhidas: &HashSet<usize>,
    renomeando: Option<&(usize, String)>,
) -> Element<'a, MiniaturaMensagem> {
    let mut lista = column![].spacing(12).padding([8, 10]);
    lista = lista.push(
        button(row![icone_mais(15.0, "#ffffff"), text("Novo slide").size(12.5)].spacing(6).align_y(Alignment::Center))
            .width(Length::Fill)
            .padding([7, 12])
            .style(estilo_botao_primario)
            .on_press(MiniaturaMensagem::Novo),
    );

    let mut escondendo = false;
    for info in slides {
        // O cabeçalho da seção que começa neste slide.
        if let Some(s) = secoes.iter().find(|s| s.inicio == info.indice) {
            let aberta = !recolhidas.contains(&s.inicio);
            escondendo = !aberta;
            let cabecalho: Element<'a, MiniaturaMensagem> = match renomeando {
                Some((i, nome)) if *i == s.inicio => text_input("Nome da seção", nome)
                    .id(text_input::Id::new(ID_RENOMEAR))
                    .on_input(MiniaturaMensagem::RenomearTexto)
                    .on_submit(MiniaturaMensagem::RenomearConfirmar)
                    .size(12.5)
                    .padding([4, 6])
                    .into(),
                _ => container(canvas::Canvas::new(Cabecalho { inicio: s.inicio, nome: s.nome.clone(), quantos: s.fim - s.inicio, aberta, arrastando_secao }).width(Length::Fill).height(Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fixed(26.0))
                    .into(),
            };
            lista = lista.push(cabecalho);
        }
        if escondendo {
            continue;
        }
        let eh_atual = info.indice == slide_atual;
        let eh_sel = selecionados.len() > 1 && selecionados.contains(&info.indice);
        let sendo_arrastado = arrastando == Some(info.indice);
        let num_slide = format!("{}", info.indice + 1);

        let preview = container(
            canvas::Canvas::new(Miniatura::nova(info.quadro.clone(), Some(info.indice), info.oculto).com_arrasto(sendo_arrastado)).width(Length::Fill).height(Length::Fill),
        )
        .width(Length::Fixed(124.0))
        .height(Length::Fixed(70.0))
        .style(move |_theme| {
            let (cor_borda, largura, sombra) = if eh_atual {
                (
                    COR_AZUL_ACENTO(),
                    2.0,
                    Shadow { color: Color { a: 0.35, ..COR_AZUL_ACENTO() }, offset: Vector::new(0.0, 1.0), blur_radius: 5.0 },
                )
            } else if sendo_arrastado {
                (
                    Color { a: 0.5, ..COR_AZUL_ACENTO() },
                    1.5,
                    Shadow::default(),
                )
            } else if eh_sel {
                (COR_AZUL_ACENTO(), 2.0, Shadow::default())
            } else {
                (COR_BORDA(), 1.0, Shadow { color: Color::from_rgba8(0, 0, 0, 0.08), offset: Vector::new(0.0, 1.0), blur_radius: 2.0 })
            };
            container::Style {
                background: Some(Background::Color(if sendo_arrastado { Color::from_rgba8(238, 241, 248, 0.45) } else if eh_sel { Color { a: 0.18, ..COR_AZUL_ACENTO() } } else { Color::WHITE })),
                border: Border { color: cor_borda, width: largura, radius: Radius::from(4.0) },
                shadow: sombra,
                text_color: None,
            }
        });

        // A marca de seleção: um visto no canto, para ver de relance quais slides estão marcados.
        let preview: Element<'a, MiniaturaMensagem> = if eh_sel {
            stack![
                preview,
                container(
                    container(crate::icones::icone_check(11.0, "#ffffff")).padding(3).style(|_| container::Style {
                        background: Some(Background::Color(COR_AZUL_ACENTO())),
                        border: Border { radius: Radius::from(9.0), ..Border::default() },
                        ..container::Style::default()
                    }),
                )
                .width(Length::Fixed(124.0))
                .height(Length::Fixed(70.0))
                .padding(4),
            ]
            .into()
        } else {
            preview.into()
        };

        // A marca de oculto: um olho riscado no canto da miniatura.
        let preview: Element<'a, MiniaturaMensagem> = if info.oculto {
            stack![
                preview,
                container(
                    container(icone_ocultar(13.0, "#ffffff")).padding(4).style(|_| container::Style {
                        background: Some(Background::Color(Color::from_rgba8(30, 30, 30, 0.82))),
                        border: Border { radius: Radius::from(11.0), ..Border::default() },
                        ..container::Style::default()
                    }),
                )
                .width(Length::Fixed(124.0))
                .height(Length::Fixed(70.0))
                .align_x(Alignment::End)
                .align_y(iced::alignment::Vertical::Bottom)
                .padding(4),
            ]
            .into()
        } else {
            preview.into()
        };

        let botao_slide = preview;

        let numero = container(
            text(num_slide).size(11).color(if info.oculto { Color { a: 0.55, ..COR_TEXTO_MUTED() } } else { COR_TEXTO_MUTED() }),
        )
        .width(Length::Fixed(18.0))
        .align_x(Alignment::End);
        lista = lista.push(row![numero, botao_slide].spacing(6).align_y(Alignment::Start));
    }

    // A barra fica num canal próprio ao lado (`spacing`), em vez de ficar por cima das miniaturas.
    let painel = scrollable(lista)
        .direction(scrollable::Direction::Vertical(scrollable::Scrollbar::new().width(6).scroller_width(6).spacing(4)))
        .width(Length::Fixed(182.0))
        .height(Length::Fill);
    container(painel).height(Length::Fill).style(estilo_painel_miniaturas).into()
}

#[allow(dead_code)]
fn _tamanho_nao_usado(_: Size) {}
