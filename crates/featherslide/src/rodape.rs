//! O diálogo "Cabeçalho e rodapé" (Inserir → Data e hora / Nº do slide / Rodapé): data, número do
//! slide e texto embaixo de todos os slides. O motor desenha (`featherslide_core::rodape`); aqui o rascunho.
use featherslide_core::modelo::Rodape;
use pelicano::column;
use pelicano::*;

use crate::app::Mensagem;
use crate::tema::*;

/// Um campo do rascunho que mudou.
#[derive(Clone, Debug)]
pub enum Campo {
    Data(bool),
    DataFixa(String),
    Numero(bool),
    Texto(String),
    SemNoTitulo(bool),
    /// O número do primeiro slide ("Começar em").
    Inicio(u32),
}

pub fn mudar(r: &mut Rodape, c: Campo) {
    match c {
        Campo::Data(v) => r.data = v,
        Campo::DataFixa(v) => r.data_fixa = v,
        Campo::Numero(v) => r.numero = v,
        Campo::Texto(v) => r.texto = v,
        Campo::SemNoTitulo(v) => r.sem_no_titulo = v,
        Campo::Inicio(v) => r.numero_inicial = v.min(featherslide_core::rodape::NUMERO_INICIAL_MAXIMO),
    }
}

pub fn sobrepor<'a>(base: Element<'a, Mensagem>, r: &'a Rodape, largura_janela: f32) -> Element<'a, Mensagem> {
    let fundo = mouse_area(Space::new(Length::FILL, Length::FILL)).on_press(Mensagem::RodapeFechar);
    let campo = |m: fn(String) -> Campo| move |v: String| Mensagem::RodapeMudar(m(v));
    let painel = container(column![
        row![
            text("Cabeçalho e rodapé").size(18).width(Length::FILL),
            button(text("Fechar").size(12)).on_press(Mensagem::RodapeFechar).style(estilo_botao_secundario),
        ]
        .align(Align::Center),
        text("Aparecem embaixo de todos os slides e se atualizam sozinhos.").size(11.5).color(COR_TEXTO_MUTED()),
        checkbox("Data (à esquerda)", r.data).on_toggle(|v| Mensagem::RodapeMudar(Campo::Data(v))),
        text_input("Vazio = a data de hoje, atualizada ao abrir", &r.data_fixa).on_input(campo(Campo::DataFixa)).padding([6, 8]).size(12),
        row![
            checkbox("Número do slide (à direita)", r.numero).on_toggle(|v| Mensagem::RodapeMudar(Campo::Numero(v))),
            Space::new(Length::FILL, Length::Fixed(1.0)),
            text("Começar em").size(12),
            button(text("−").size(12)).style(estilo_botao_secundario).on_press(Mensagem::RodapeMudar(Campo::Inicio(r.numero_inicial.saturating_sub(1)))),
            text(r.numero_inicial.to_string()).size(12),
            button(text("+").size(12)).style(estilo_botao_secundario).on_press(Mensagem::RodapeMudar(Campo::Inicio(r.numero_inicial + 1))),
        ]
        .spacing(8)
        .align(Align::Center),
        text("Rodapé (no meio)").size(12),
        text_input("Por exemplo: nome da empresa · evento", &r.texto).on_input(campo(Campo::Texto)).padding([6, 8]).size(12),
        checkbox("Não mostrar no slide de título", r.sem_no_titulo).on_toggle(|v| Mensagem::RodapeMudar(Campo::SemNoTitulo(v))),
        row![
            Space::new(Length::FILL, Length::Fixed(1.0)),
            button(text("Aplicar a todos").size(12)).style(estilo_botao_primario).padding([7, 12]).on_press(Mensagem::RodapeAplicar),
        ],
    ]
    .spacing(10))
    .width(Length::Fixed(460.0))
    .padding(16)
    .style(|_| Style {
        background: Some(Background::Color(COR_PAINEL())),
        text_color: Some(COR_TEXTO()),
        border: Border { color: COR_BORDA(), width: 1.0, radius: [8.0; 4] },
        shadow: Some(Shadow { color: Color::rgb8(0, 0, 0).alpha(0.25), offset: Point::new(0.0, 6.0), blur: 18.0 }),
        ..Default::default()
    });
    let x = ((largura_janela - 460.0) / 2.0).max(8.0);
    let pop = container(painel).padding(Padding { top: 90.0, left: x, right: 0.0, bottom: 0.0 });
    stack![base, fundo, pop].into()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn campo_comecar_em_fica_no_limite() {
        let mut r = Rodape::default();
        assert_eq!(r.numero_inicial, 1);
        mudar(&mut r, Campo::Inicio(0));
        assert_eq!(r.numero_inicial, 0);
        mudar(&mut r, Campo::Inicio(123_456));
        assert_eq!(r.numero_inicial, featherslide_core::rodape::NUMERO_INICIAL_MAXIMO);
    }
}
