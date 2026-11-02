//! A galeria de layouts (Novo slide e Layout): cartões com o esboço de cada um, como no PowerPoint.

use featherslide_core::layouts::LAYOUTS;
use featherslide_core::quadro::Quadro;
use pelicano::{button, column, container, mouse_area, program, row, text, Space};
use pelicano::{Align, Background, Border, Color, Element, Length, Point, Shadow, Status, Style};

use crate::app::Mensagem;
use crate::miniaturas::{Miniatura, MiniaturaMensagem};
use crate::faixa::FaixaMensagem;
use crate::tema::*;

pub const NOMES: [(&str, &str); 14] = [
    ("titulo", "Slide de título"),
    ("titulo_conteudo", "Título e conteúdo"),
    ("secao", "Cabeçalho de seção"),
    ("dois_conteudos", "Dois conteúdos"),
    ("comparacao", "Comparação"),
    ("tres_colunas", "Três colunas"),
    ("quatro_quadrantes", "Quatro quadrantes"),
    ("titulo_lateral", "Título ao lado"),
    ("numeros", "Números"),
    ("linha_do_tempo", "Linha do tempo"),
    ("citacao", "Citação"),
    ("somente_conteudo", "Somente conteúdo"),
    ("somente_titulo", "Somente título"),
    ("em_branco", "Em branco"),
];

fn cartao<'a>(layout: &'static str, nome: &'static str, quadro: Quadro) -> Element<'a, FaixaMensagem> {
    button(
        column![
            Element::from(program(Miniatura::nova(Some(quadro), None, false)).width(Length::Fixed(132.0)).height(Length::Fixed(74.0)))
                .map(|_: MiniaturaMensagem| FaixaMensagem::FecharGaleria),
            text(nome).size(12),
        ]
        .spacing(6)
        .align(Align::Center),
    )
    .padding(8)
    .style(|_, status| Style {
        background: Some(Background::Color(if matches!(status, Status::Hovered | Status::Pressed) { COR_HOVER() } else { Color::TRANSPARENT })),
        text_color: Some(COR_TEXTO()),
        border: Border { color: if matches!(status, Status::Hovered | Status::Pressed) { COR_AZUL_ACENTO() } else { Color::TRANSPARENT }, width: 1.5, radius: [6.0; 4] },
        shadow: None, ..Style::default() })
    .on_press(FaixaMensagem::EscolherLayout(layout))
    .into()
}

/// A galeria sobre a tela; clicar fora fecha. `novo`: cria um slide; senão troca o layout do atual.
pub fn sobrepor<'a>(base: Element<'a, Mensagem>, novo: bool, sessao: &featherslide_core::sessao::Sessao, largura_janela: f32) -> Element<'a, Mensagem> {
    let fundo = mouse_area(Space::new(Length::FILL, Length::FILL)).on_press(Mensagem::Faixa(FaixaMensagem::FecharGaleria));
    let mut grade = column![].spacing(4);
    for par in NOMES.chunks(4) {
        let mut linha = row![].spacing(4);
        for (layout, nome) in par {
            linha = linha.push(cartao(layout, nome, sessao.quadro_de_layout(layout)));
        }
        grade = grade.push(linha);
    }
    let titulo = if novo { "Novo slide" } else { "Layout do slide" };
    let painel = container(column![text(titulo).size(12).color(COR_TEXTO_MUTED()), grade].spacing(8))
        .padding(12)
        .style(|_| Style {
            background: Some(Background::Color(COR_PAINEL())),
            text_color: Some(COR_TEXTO()),
            border: Border { color: COR_BORDA(), width: 1.0, radius: [8.0; 4] },
            shadow: Some(Shadow { color: Color::rgb8(0, 0, 0).alpha(0.3), offset: Point::new(0.0, 6.0), blur: 18.0 }), ..Style::default() });
    let (esquerda, topo) = crate::medido::ancora("novo_slide", 640.0, largura_janela, (110.0, 128.0));
    let sobreposto = container(painel.map_fixed()).padding(pelicano::Padding { top: topo, left: esquerda, right: 0.0, bottom: 0.0 });
    pelicano::stack![base, fundo, sobreposto].into()
}

trait MapFixed<'a> {
    fn map_fixed(self) -> Element<'a, Mensagem>;
}

impl<'a> MapFixed<'a> for pelicano::Container<'a, FaixaMensagem> {
    fn map_fixed(self) -> Element<'a, Mensagem> {
        Element::from(self).map(Mensagem::Faixa)
    }
}

#[allow(dead_code)]
const _: [&str; 14] = LAYOUTS;
