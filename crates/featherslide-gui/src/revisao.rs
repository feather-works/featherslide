//! Os painéis de Inserir → Link e Inserir → Comentário: uma caixa por cima do editor. A
//! interface só pede e mostra; validar o endereço e guardar os comentários é do motor.

use iced::widget::{button, column, container, mouse_area, row, scrollable, text, text_input, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Vector};

use crate::app::{Mensagem, Revisao};
use crate::tema::*;
use featherslide_core::modelo::Comentario;

pub const ID_CAMPO: &str = "campo_revisao";

#[derive(Debug, Clone)]
pub enum RevisaoMsg {
    Texto(String),
    Confirmar,
    RemoverLink,
    Fechar,
    Resolver(usize, bool),
    Excluir(usize),
    ExcluirTodos,
}

/// A caixa, com um fundo que fecha ao clicar fora.
pub fn sobrepor<'a>(base: Element<'a, Mensagem>, revisao: &'a Revisao, comentarios: &'a [Comentario], tem_link: bool, largura_janela: f32) -> Element<'a, Mensagem> {
    let fundo = mouse_area(Space::new(Length::Fill, Length::Fill)).on_press(Mensagem::Revisao(RevisaoMsg::Fechar));
    let caixa = match revisao {
        Revisao::Link(texto) => caixa_link(texto, tem_link),
        Revisao::Comentario(texto) => caixa_comentarios(texto, comentarios),
    };
    let largura = 380.0;
    let esquerda = ((largura_janela - largura) / 2.0).max(8.0);
    let painel = container(caixa.map(Mensagem::Revisao)).padding(iced::Padding { top: 150.0, left: esquerda, right: 0.0, bottom: 0.0 });
    iced::widget::stack![base, fundo, painel].into()
}

fn moldura<'a>(corpo: Element<'a, RevisaoMsg>) -> Element<'a, RevisaoMsg> {
    container(corpo)
        .width(Length::Fixed(380.0))
        .padding(14)
        .style(|_| container::Style {
            background: Some(Background::Color(COR_PAINEL())),
            text_color: Some(COR_TEXTO()),
            border: Border { color: COR_BORDA(), width: 1.0, radius: 8.0.into() },
            shadow: Shadow { color: Color::from_rgba8(0, 0, 0, 0.3), offset: Vector::new(0.0, 6.0), blur_radius: 18.0 },
        })
        .into()
}

fn caixa_link<'a>(texto: &'a str, tem_link: bool) -> Element<'a, RevisaoMsg> {
    let mut botoes = row![].spacing(8).align_y(Alignment::Center);
    if tem_link {
        botoes = botoes.push(button(text("Remover link").size(12.5)).padding([5, 10]).style(estilo_botao_secundario).on_press(RevisaoMsg::RemoverLink));
    }
    botoes = botoes
        .push(Space::new(Length::Fill, Length::Shrink))
        .push(button(text("Cancelar").size(12.5)).padding([5, 10]).style(estilo_botao_secundario).on_press(RevisaoMsg::Fechar))
        .push(button(text("Aplicar").size(12.5)).padding([5, 10]).style(estilo_botao_primario).on_press(RevisaoMsg::Confirmar));
    moldura(
        column![
            text("Link").size(14),
            text("Endereço da página ou e-mail. Abre ao clicar durante a apresentação.").size(11).color(COR_TEXTO_MUTED()),
            text_input("https://exemplo.com", texto).id(text_input::Id::new(ID_CAMPO)).on_input(RevisaoMsg::Texto).on_submit(RevisaoMsg::Confirmar).size(13).padding([6, 8]),
            botoes,
        ]
        .spacing(8)
        .into(),
    )
}

fn caixa_comentarios<'a>(texto: &'a str, comentarios: &'a [Comentario]) -> Element<'a, RevisaoMsg> {
    let mut lista = column![].spacing(6);
    if comentarios.is_empty() {
        lista = lista.push(text("Nenhum comentário neste slide.").size(12).color(COR_TEXTO_MUTED()));
    }
    for (i, c) in comentarios.iter().enumerate() {
        let cor = if c.resolvido { COR_TEXTO_MUTED() } else { COR_TEXTO() };
        let rotulo = if c.resolvido { "Reabrir" } else { "Resolver" };
        lista = lista.push(
            container(
                column![
                    text(format!("{}{}", c.autor, if c.resolvido { " — resolvido" } else { "" })).size(11).color(COR_TEXTO_MUTED()),
                    text(c.texto.as_str()).size(13).color(cor),
                    row![
                        button(text(rotulo).size(11.5)).padding([2, 8]).style(estilo_botao_secundario).on_press(RevisaoMsg::Resolver(i, !c.resolvido)),
                        button(text("Excluir").size(11.5)).padding([2, 8]).style(estilo_botao_secundario).on_press(RevisaoMsg::Excluir(i)),
                    ]
                    .spacing(6),
                ]
                .spacing(3),
            )
            .padding(8)
            .width(Length::Fill)
            .style(|_| container::Style { background: Some(Background::Color(COR_PAINEL_FAIXA())), border: Border { color: COR_BORDA_SUAVE(), width: 1.0, radius: 6.0.into() }, ..Default::default() }),
        );
    }
    moldura(
        column![
            text("Comentários").size(14),
            scrollable(lista).height(Length::Fixed(190.0)),
            text_input("Escreva um comentário…", texto).id(text_input::Id::new(ID_CAMPO)).on_input(RevisaoMsg::Texto).on_submit(RevisaoMsg::Confirmar).size(13).padding([6, 8]),
            row![
                button(text("Excluir todos").size(12.5)).padding([5, 10]).style(estilo_botao_secundario).on_press(RevisaoMsg::ExcluirTodos),
                Space::new(Length::Fill, Length::Shrink),
                button(text("Fechar").size(12.5)).padding([5, 10]).style(estilo_botao_secundario).on_press(RevisaoMsg::Fechar),
                button(text("Comentar").size(12.5)).padding([5, 10]).style(estilo_botao_primario).on_press(RevisaoMsg::Confirmar),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8)
        .into(),
    )
}
