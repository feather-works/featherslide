//! O painel lateral de slides: miniaturas, seções (pastas recolhíveis) e a marca de slide oculto.

use std::collections::{BTreeSet, HashSet};

use featherslide_core::quadro::Quadro;
use featherslide_core::SecaoInfo;
use pelicano::column;
use pelicano::*;

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

impl Program<MiniaturaMensagem> for Miniatura {
    type State = ();

    fn update(&self, _: &mut (), evento: &Event, bounds: Rect, cursor: Option<Point>, publish: &mut dyn FnMut(MiniaturaMensagem)) -> Captured {
        let (Some(i), Some(pos)) = (self.indice, cursor.filter(|p| bounds.contains(*p))) else { return Captured::No };
        let m = match evento {
            Event::MousePressed(MouseButton::Right) => MiniaturaMensagem::Menu(i, pos),
            // Apertar começa um possível arrasto; soltar na mesma miniatura é o clique.
            Event::MousePressed(MouseButton::Left) => MiniaturaMensagem::Pressionou(i),
            Event::MouseReleased(MouseButton::Left) => MiniaturaMensagem::Soltou(i),
            _ => return Captured::No,
        };
        publish(m);
        Captured::Yes
    }

    // Sem cache próprio: o pelicano só redesenha a região que mudou.
    fn draw(&self, _: &(), r: &mut Renderer, bounds: Rect, _: &Theme, _: Option<Point>) {
        r.fill_quad(bounds, Some(&Color::WHITE.into()), Border::default(), None);
        if let Some(q) = &self.quadro {
            let escala = (bounds.width / q.largura).min(bounds.height / q.altura);
            let ox = bounds.x + (bounds.width - q.largura * escala) / 2.0;
            let oy = bounds.y + (bounds.height - q.altura * escala) / 2.0;
            let antes = OPACIDADE.with(|o| o.get());
            if self.oculto || self.arrastando {
                OPACIDADE.with(|o| o.set(0.35));
            }
            pintar_quadro(r, q, ox, oy, escala, &[], (0.0, 0.0), 1.0, None);
            OPACIDADE.with(|o| o.set(antes));
        }
        if self.oculto || self.arrastando {
            // Véu por cima, para o slide parecer apagado/levantado no arrasto.
            r.fill_quad(bounds, Some(&Color::rgb8(255, 255, 255).alpha(0.55).into()), Border::default(), None);
        }
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

impl Program<MiniaturaMensagem> for Cabecalho {
    type State = ();

    fn update(&self, _: &mut (), evento: &Event, bounds: Rect, cursor: Option<Point>, publish: &mut dyn FnMut(MiniaturaMensagem)) -> Captured {
        let Some(pos) = cursor.filter(|p| bounds.contains(*p)) else { return Captured::No };
        let m = match evento {
            // A decisão clique x arrasto ocorre no app ao soltar: não recolher no pressionar.
            Event::MousePressed(MouseButton::Left) => MiniaturaMensagem::PressionouSecao(self.inicio, pos),
            Event::MouseReleased(MouseButton::Left) => MiniaturaMensagem::SoltouSecao(self.inicio),
            Event::MousePressed(MouseButton::Right) => MiniaturaMensagem::MenuSecao(self.inicio, pos),
            _ => return Captured::No,
        };
        publish(m);
        Captured::Yes
    }

    fn draw(&self, _: &(), r: &mut Renderer, bounds: Rect, _: &Theme, cursor: Option<Point>) {
        let pintar = |r: &mut Renderer, area: Rect, cor: Color| r.fill_quad(area, Some(&cor.into()), Border::default(), None);
        let sobre = cursor.is_some_and(|p| bounds.contains(p));
        if sobre {
            pintar(r, bounds, COR_HOVER());
        }
        if let Some(origem) = self.arrastando_secao {
            if origem == self.inicio {
                pintar(r, bounds, Color::rgb8(65, 120, 205).alpha(0.13));
            } else if sobre {
                // Linha de inserção: acima do alvo quando subindo; abaixo quando descendo.
                pintar(r, bounds, Color::rgb8(65, 120, 205).alpha(0.10));
                let y = if origem > self.inicio { 0.0 } else { bounds.height - 3.0 };
                pintar(r, Rect { x: bounds.x, y: bounds.y + y, width: bounds.width, height: 3.0 }, COR_AZUL_ACENTO());
            }
        }
        // A seta: para baixo (aberta) ou para a direita (recolhida).
        let cor = COR_TEXTO_MUTED();
        let (cx, cy) = (bounds.x + 11.0, bounds.y + bounds.height / 2.0);
        pintar_no_canvas(r, if self.aberta { "chevron-down" } else { "chevron-right" }, cx - 7.0, cy - 7.0, 14.0, cor);
        let tamanho = 12.5;
        r.draw_text(&format!("{}  ({})", self.nome, self.quantos), tamanho, Font::SEMIBOLD, COR_TEXTO(), Point::new(bounds.x + 22.0, cy - tamanho * 1.3 / 2.0), None);
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
        button(row![icone_mais(15.0, "#ffffff"), text("Novo slide").size(12.5)].spacing(6).align(Align::Center))
            .width(Length::FILL)
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
                Some((i, nome)) if *i == s.inicio => text_input("Nome da seção", nome.clone())
                    .id(ID_RENOMEAR)
                    .on_input(MiniaturaMensagem::RenomearTexto)
                    .on_submit(MiniaturaMensagem::RenomearConfirmar)
                    .size(12.5)
                    .padding([4, 6])
                    .into(),
                _ => container(program(Cabecalho { inicio: s.inicio, nome: s.nome.clone(), quantos: s.fim - s.inicio, aberta, arrastando_secao }).width(Length::FILL).height(Length::FILL))
                    .width(Length::FILL)
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
            program(Miniatura::nova(info.quadro.clone(), Some(info.indice), info.oculto).com_arrasto(sendo_arrastado)).width(Length::FILL).height(Length::FILL),
        )
        .width(Length::Fixed(124.0))
        .height(Length::Fixed(70.0))
        .style(move |_theme| {
            let (cor_borda, largura, sombra) = if eh_atual {
                (
                    COR_AZUL_ACENTO(),
                    2.0,
                    Some(Shadow { color: Color { a: 0.35, ..COR_AZUL_ACENTO() }, offset: Point::new(0.0, 1.0), blur: 5.0 }),
                )
            } else if sendo_arrastado {
                (
                    Color { a: 0.5, ..COR_AZUL_ACENTO() },
                    1.5,
                    None,
                )
            } else if eh_sel {
                (COR_AZUL_ACENTO(), 2.0, None)
            } else {
                (COR_BORDA(), 1.0, Some(Shadow { color: Color::rgb8(0, 0, 0).alpha(0.08), offset: Point::new(0.0, 1.0), blur: 2.0 }))
            };
            Style {
                background: Some(Background::Color(if sendo_arrastado { Color::rgb8(238, 241, 248).alpha(0.45) } else if eh_sel { Color { a: 0.18, ..COR_AZUL_ACENTO() } } else { Color::WHITE })),
                border: Border::rounded(4.0).color(cor_borda, largura),
                shadow: sombra,
                ..Style::default()
            }
        });

        // A marca de seleção: um visto no canto, para ver de relance quais slides estão marcados.
        let preview: Element<'a, MiniaturaMensagem> = if eh_sel {
            stack![
                preview,
                container(
                    container(crate::icones::icone_check(11.0, "#ffffff")).padding(3).style(|_| Style {
                        background: Some(Background::Color(COR_AZUL_ACENTO())),
                        border: Border::rounded(9.0),
                        ..Style::default()
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
                    container(icone_ocultar(13.0, "#ffffff")).padding(4).style(|_| Style {
                        background: Some(Background::Color(Color::rgb8(30, 30, 30).alpha(0.82))),
                        border: Border::rounded(11.0),
                        ..Style::default()
                    }),
                )
                .width(Length::Fixed(124.0))
                .height(Length::Fixed(70.0))
                .align_x(Align::End)
                .align_y(Align::End)
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
        .align_x(Align::End);
        lista = lista.push(row![numero, botao_slide].spacing(6).align(Align::Start));
    }

    let painel = scrollable(lista).scrollbar_track(COR_TRILHO_ROLAGEM()).scrollbar_color(COR_POLEGAR_ROLAGEM())
        .scrollbar_width(6)
        .scrollbar_spacing(4)
        .width(Length::Fixed(182.0))
        .height(Length::FILL);
    container(painel).height(Length::FILL).style(estilo_painel_miniaturas).into()
}

