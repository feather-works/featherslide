//! Edição e reprodução das apresentações personalizadas (FTS-0121–0123).
use std::collections::BTreeSet;
use featherslide_core::sessao::Sessao;
use iced::widget::{button, column, container, mouse_area, row, scrollable, text, text_input, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Vector};

use crate::app::Mensagem;
use crate::tema::*;

#[derive(Default)]
pub struct Estado {
    pub nome_novo: String,
    pub selecionada: Option<usize>,
}

/// Modal local: lista de roteiros e sequência editável, sem abrir janelas extras.
pub fn sobrepor<'a>(
    base: Element<'a, Mensagem>,
    estado: &'a Estado,
    sessao: &'a Sessao,
    marcados: &BTreeSet<usize>,
    largura_janela: f32,
) -> Element<'a, Mensagem> {
    let fundo = mouse_area(Space::new(Length::Fill, Length::Fill))
        .on_press(Mensagem::PersonalizadasFechar);

    let mut lista = column![].spacing(3);
    for (i, p) in sessao.personalizadas().iter().enumerate() {
        lista = lista.push(button(text(format!("{}  ({} slides)", p.nome, p.slides.len())).size(12))
            .width(Length::Fill).padding([6, 8])
            .style(estilo_botao_pequeno(estado.selecionada == Some(i)))
            .on_press(Mensagem::PersonalizadaSelecionar(i)));
    }

    let criar = row![
        text_input("Nome da nova apresentação", &estado.nome_novo)
            .on_input(Mensagem::PersonalizadaNome).on_submit(Mensagem::PersonalizadaCriar)
            .padding([6, 8]).size(12).width(Length::Fill),
        button(text("Criar com slides marcados").size(11))
            .style(estilo_botao_primario).padding([7, 9])
            .on_press(Mensagem::PersonalizadaCriar),
    ].align_y(Alignment::Center).spacing(6);

    let mut detalhe = column![].spacing(5);
    if let Some(p) = estado.selecionada.and_then(|i| sessao.personalizadas().get(i)) {
        detalhe = detalhe.push(text(format!("Roteiro: {}", p.nome)).size(13));
        for (pos, item) in p.slides.iter().enumerate() {
            let slide = sessao.apresentacao().slides.iter().position(|s| s.id == item.id);
            let rotulo = match slide {
                Some(i) => format!("{}. Slide {}{}", pos+1, i+1, if item.oculto { "  [oculto nesta sequência]" } else { "" }),
                None => format!("{}. Slide excluído (ID {})", pos+1, item.id),
            };
            let botao = |rotulo: &'static str, msg| button(text(rotulo).size(12))
                .style(estilo_botao_pequeno(false)).padding([4, 7]).on_press_maybe(msg);
            detalhe = detalhe.push(row![
                text(rotulo).size(11.5).width(Length::Fill),
                botao("↑", (pos > 0).then_some(Mensagem::PersonalizadaMover(pos, -1))),
                botao("↓", (pos+1 < p.slides.len()).then_some(Mensagem::PersonalizadaMover(pos, 1))),
                botao(if item.oculto { "Exibir" } else { "Ocultar" }, Some(Mensagem::PersonalizadaOcultar(pos))),
                botao("×", (p.slides.len() > 1).then_some(Mensagem::PersonalizadaRemover(pos))),
            ].spacing(3).align_y(Alignment::Center));
        }
        detalhe = detalhe.push(row![
            button(text("Acrescentar marcados").size(12))
                .style(estilo_botao_secundario).on_press(Mensagem::PersonalizadaAdicionar),
            button(text("Excluir roteiro").size(12))
                .style(estilo_botao_secundario).on_press(Mensagem::PersonalizadaExcluir),
            button(text("Apresentar roteiro").size(12))
                .style(estilo_botao_primario).on_press(Mensagem::PersonalizadaIniciar),
        ].spacing(6));
    } else {
        detalhe = detalhe.push(text("Crie ou escolha um roteiro acima para definir ordem e slides ocultos.").size(12));
    }
    let painel = container(column![
        row![
            text("Apresentações personalizadas").size(18).width(Length::Fill),
            button(text("Fechar").size(12)).on_press(Mensagem::PersonalizadasFechar)
                .style(estilo_botao_secundario),
        ].align_y(Alignment::Center),
        text(format!("Marque slides nas miniaturas antes de criar ou acrescentar ({} marcados).", marcados.len()))
            .size(11.5).color(COR_TEXTO_MUTED()),
        criar,
        scrollable(lista).height(Length::Fixed(110.0)),
        scrollable(detalhe).height(Length::Fixed(290.0)),
    ].spacing(12))
    .width(Length::Fixed(760.0))
    .padding(16)
    .style(|_| container::Style {
        background: Some(Background::Color(COR_PAINEL())),
        text_color: Some(COR_TEXTO()),
        border: Border { color: COR_BORDA(), width: 1.0, radius: 8.0.into() },
        shadow: Shadow { color: Color::from_rgba8(0,0,0,0.25),
            offset: Vector::new(0.0, 6.0), blur_radius: 18.0 },
    });
    let x = ((largura_janela - 760.0)/2.0).max(8.0);
    let pop = container(painel).padding(iced::Padding {
        top: 70.0, left: x, right: 0.0, bottom: 0.0,
    });
    iced::widget::stack![base, fundo, pop].into()
}
