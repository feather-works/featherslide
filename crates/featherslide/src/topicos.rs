//! A estrutura de tópicos (Exibir): a apresentação inteira como texto, para escrever a fala
//! antes do visual. O motor (`Sessao::topicos`, `topicos_aplicar`) faz a conta; aqui só o painel.
use pelicano::column;
use pelicano::*;

use crate::app::Mensagem;
use crate::tema::*;

/// O texto em edição no painel (o `Content` do editor guarda o cursor e o desfazer dele).
pub struct Estado {
    pub texto: Content,
}

pub fn sobrepor<'a>(base: Element<'a, Mensagem>, estado: &'a Estado, largura_janela: f32) -> Element<'a, Mensagem> {
    let fundo = mouse_area(Space::new(Length::FILL, Length::FILL)).on_press(Mensagem::TopicosFechar);
    let painel = container(column![
        row![
            text("Estrutura de tópicos").size(18).width(Length::FILL),
            button(text("Fechar").size(12)).on_press(Mensagem::TopicosFechar).style(estilo_botao_secundario),
        ]
        .align(Align::Center),
        text("Um título por linha; o corpo do slide vai embaixo, com Tab no começo. Títulos a mais viram slides novos; nenhum slide é apagado. Ctrl+Z desfaz.")
            .size(11.5)
            .color(COR_TEXTO_MUTED()),
        container(text_editor(&estado.texto).size(13.0).wrap(true).id("featherslide.topicos"))
            .height(Length::Fixed(380.0))
            .width(Length::FILL),
        row![
            Space::new(Length::FILL, Length::Fixed(1.0)),
            button(text("Aplicar aos slides").size(12)).style(estilo_botao_primario).padding([7, 12]).on_press(Mensagem::TopicosAplicar),
        ],
    ]
    .spacing(12))
    .width(Length::Fixed(720.0))
    .padding(16)
    .style(|_| Style {
        background: Some(Background::Color(COR_PAINEL())),
        text_color: Some(COR_TEXTO()),
        border: Border { color: COR_BORDA(), width: 1.0, radius: [8.0; 4] },
        shadow: Some(Shadow { color: Color::rgb8(0, 0, 0).alpha(0.25), offset: Point::new(0.0, 6.0), blur: 18.0 }),
        ..Default::default()
    });
    let x = ((largura_janela - 720.0) / 2.0).max(8.0);
    let pop = container(painel).padding(Padding { top: 70.0, left: x, right: 0.0, bottom: 0.0 });
    stack![base, fundo, pop].into()
}
