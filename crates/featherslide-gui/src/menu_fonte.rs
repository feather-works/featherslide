//! Os menus suspensos do grupo Fonte (Maiúsculas e minúsculas, Espaçamento entre
//! letras): um botão com ícone na faixa abre, logo embaixo dele, uma lista com a
//! prévia de cada opção, no mesmo padrão flutuante do seletor de cor.
use iced::widget::{button, column, container, mouse_area, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Vector};

use featherslide_core::texto_formato::Caixa;

use crate::app::Mensagem;
use crate::faixa::FaixaMensagem;
use crate::icones::{icone_check, icone_letras, icone_seta_baixo, icone_sublinhado_duplo, icone_tachado_duplo};
use crate::tema::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuFonte {
    Caixa,
    Espacamento,
    /// Sublinhado e tachado duplos.
    Tracos,
}

impl MenuFonte {
    fn ancora(self) -> &'static str {
        match self {
            MenuFonte::Caixa => "menu_caixa",
            MenuFonte::Espacamento => "menu_espacamento",
            MenuFonte::Tracos => "menu_tracos",
        }
    }
}

/// Os níveis do espaçamento entre letras, em pontos.
pub const ESPACAMENTOS: [(&str, f32); 5] = [
    ("Muito justo", -2.0),
    ("Justo", -1.0),
    ("Normal", 0.0),
    ("Amplo", 2.0),
    ("Muito amplo", 4.0),
];

fn icone_da_caixa(c: Caixa) -> &'static str {
    match c {
        Caixa::Frase => "letter-case",
        Caixa::Minusculas => "letter-case-lower",
        Caixa::Maiusculas => "letter-case-upper",
        Caixa::CadaPalavra => "letter-case",
        Caixa::Alternar => "letter-case-toggle",
    }
}

/// O botão da faixa: ícone + seta, azul quando o menu está aberto.
pub fn botao<'a>(menu: MenuFonte, aberto: bool, ligado: bool) -> Element<'a, FaixaMensagem> {
    let cor = if aberto { "#1a73e8" } else { "#5f6368" };
    let (icone, dica) = match menu {
        MenuFonte::Caixa => (icone_letras("letter-case", 15.0, cor), "Maiúsculas e minúsculas"),
        MenuFonte::Espacamento => (icone_letras("letter-spacing", 15.0, cor), "Espaçamento entre letras"),
        MenuFonte::Tracos => (icone_sublinhado_duplo(14.0, cor), "Sublinhado e tachado duplos"),
    };
    let b = button(
        row![icone, icone_seta_baixo(9.0, "#5f6368")]
            .spacing(2)
            .align_y(Alignment::Center),
    )
    .padding([3, 5])
    .style(estilo_botao_pequeno(aberto))
    .on_press_maybe(ligado.then_some(FaixaMensagem::AbrirMenuFonte(menu)));
    crate::medido::marcar(menu.ancora(), feather_ui::formatacao::dica(b, dica))
}

/// Uma linha do menu: ícone (ou espaço), a prévia e a marca da opção atual.
fn item<'a>(icone: Option<Element<'a, FaixaMensagem>>, rotulo: String, atual: bool, msg: FaixaMensagem) -> Element<'a, FaixaMensagem> {
    let icone = icone.unwrap_or_else(|| Space::with_width(16.0).into());
    let marca: Element<'a, FaixaMensagem> = if atual { icone_check(14.0, "#1a73e8").into() } else { Space::with_width(14.0).into() };
    button(row![icone, text(rotulo).size(13), Space::with_width(Length::Fill), marca].spacing(10).align_y(Alignment::Center))
        .width(Length::Fill)
        .padding([6, 10])
        .style(estilo_botao_pequeno(false))
        .on_press(msg)
        .into()
}

pub fn sobrepor<'a>(base: Element<'a, Mensagem>, menu: MenuFonte, espacamento_atual: f32, duplos: (bool, bool), largura_janela: f32) -> Element<'a, Mensagem> {
    const LARGURA: f32 = 250.0;
    let (esquerda, topo) = crate::medido::ancora(menu.ancora(), LARGURA, largura_janela, (420.0, 128.0));
    let (titulo, itens): (&str, Vec<Element<'a, FaixaMensagem>>) = match menu {
        MenuFonte::Caixa => (
            "Maiúsculas e minúsculas",
            Caixa::TODAS
                .iter()
                .map(|&c| {
                    let rotulo = match c {
                        Caixa::Frase => "Primeira letra da frase.".to_string(),
                        _ => c.rotulo().to_string(),
                    };
                    item(Some(icone_letras(icone_da_caixa(c), 16.0, "#5f6368").into()), rotulo, false, FaixaMensagem::Caixa(c))
                })
                .collect(),
        ),
        MenuFonte::Espacamento => (
            "Espaçamento entre letras",
            ESPACAMENTOS
                .iter()
                .map(|&(nome, pt)| {
                    let valor = if pt == 0.0 { "0 pt".to_string() } else { format!("{pt:+} pt") };
                    item(None, format!("{nome}  ({valor})"), (espacamento_atual - pt).abs() < 0.01, FaixaMensagem::EspacamentoLetras(pt))
                })
                .collect(),
        ),
        MenuFonte::Tracos => (
            "Traço duplo",
            vec![
                item(Some(icone_sublinhado_duplo(16.0, "#5f6368").into()), "Sublinhado duplo".into(), duplos.0, FaixaMensagem::SublinhadoDuplo),
                item(Some(icone_tachado_duplo(16.0, "#5f6368").into()), "Tachado duplo".into(), duplos.1, FaixaMensagem::TachadoDuplo),
            ],
        ),
    };
    let conteudo = column![text(titulo).size(11).color(COR_TEXTO_MUTED())].push(column(itens).spacing(1)).spacing(6);
    let painel = container(Element::from(conteudo).map(Mensagem::Faixa))
        .padding(8)
        .width(Length::Fixed(LARGURA))
        .style(|_| container::Style {
            background: Some(Background::Color(COR_PAINEL())),
            text_color: Some(COR_TEXTO()),
            border: Border { color: COR_BORDA(), width: 1.0, radius: 10.0.into() },
            shadow: Shadow { color: Color::from_rgba8(0, 0, 0, 0.3), offset: Vector::new(0.0, 6.0), blur_radius: 18.0 },
        });
    let sobreposto = container(painel).padding(iced::Padding { top: topo, left: esquerda, right: 0.0, bottom: 0.0 });
    let fundo = mouse_area(Space::new(Length::Fill, Length::Fill)).on_press(Mensagem::Faixa(FaixaMensagem::FecharMenuFonte));
    iced::widget::stack![base, fundo, sobreposto].into()
}
