//! Editor de imagem com navegação por categorias.
//! Só a interface muda: os comandos continuam indo ao motor Rust,
//! mantendo efeitos reversíveis e compressão apenas sob comando explícito.
use featherslide_core::imagem::VisualImagem;
use iced::widget::{
    button, column, container, mouse_area, row, scrollable, slider, text, text_input, tooltip, Space,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Vector};

use crate::faixa::FaixaMensagem as Msg;
use crate::icones::*;
use crate::tema::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Pagina {
    Enquadrar,
    #[default]
    Cor,
    Efeitos,
    Arquivo,
}

/// Um botão que se explica ao passar o mouse, sem ocupar a área do painel com texto.
fn dica<'a>(conteudo: impl Into<Element<'a, Msg>>, ajuda: &'static str) -> Element<'a, Msg> {
    tooltip(
        conteudo,
        container(text(ajuda).size(11)).padding([5, 8]).style(estilo_dica),
        tooltip::Position::Bottom,
    ).into()
}

/// Uma superfície discreta que segue o tema claro/escuro do FeatherSlide.
fn cartao<'a>(conteudo: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    container(conteudo)
        .padding(16)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(COR_PAINEL_FAIXA())),
            border: Border {
                color: COR_BORDA_SUAVE(),
                width: 1.0,
                radius: 11.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn titulo_bloco<'a>(nome: &'static str, descricao: &'static str) -> Element<'a, Msg> {
    column![
        text(nome).size(14).color(COR_TEXTO()),
        text(descricao).size(11).color(COR_TEXTO_MUTED()),
    ]
    .spacing(4)
    .into()
}

fn opcao<'a>(
    rotulo: &'static str, campo: &'static str, valor: f32, selecionado: bool,
) -> Element<'a, Msg> {
    button(text(rotulo).size(12))
        .padding([9, 12])
        .style(estilo_botao_pequeno(selecionado))
        .on_press(Msg::ImagemVisual(campo, valor))
        .into()
}

/// Sliders com valor explícito e retorno rápido ao padrão: sem fileiras de +/-.
fn controle<'a>(
    nome: &'static str, campo: &'static str, atual: f32, padrao: f32,
    minimo: f32, maximo: f32, passo: f32, graus: bool,
) -> Element<'a, Msg> {
    let valor = if graus {
        format!("{:.0}°", atual)
    } else {
        format!("{:.0}%", atual * 100.0)
    };
    let barra = slider(minimo..=maximo, atual, move |v| Msg::ImagemVisual(campo, v))
        .step(passo)
        .width(Length::Fill);
    let redefinir = dica(
        button(icone_limpar(15.0, "#5f6368"))
            .padding([4, 6])
            .style(estilo_botao_pequeno(false))
            .on_press_maybe(((atual - padrao).abs() > 0.001)
                .then_some(Msg::ImagemVisual(campo, padrao))),
        "Restaurar o valor padrão",
    );
    column![
        row![
            text(nome).size(12).color(COR_TEXTO()),
            Space::with_width(Length::Fill),
            text(valor).size(12).color(COR_TEXTO_MUTED()),
        ]
        .align_y(Alignment::Center),
        row![barra, redefinir].spacing(10).align_y(Alignment::Center),
    ]
    .spacing(6)
    .into()
}

fn aba<'a>(
    nome: &'static str,
    icone: iced::widget::svg::Svg<'static, iced::Theme>,
    valor: Pagina, ativa: bool,
) -> Element<'a, Msg> {
    button(
        row![icone, text(nome).size(12)]
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .width(Length::FillPortion(1))
    .padding([10, 13])
    .style(estilo_botao_pequeno(ativa))
    .on_press(Msg::ImagemPagina(valor))
    .into()
}

fn enquadrar<'a>(visual: &VisualImagem, margens: [f32; 4]) -> Element<'a, Msg> {
    let modos = cartao(
        column![
            titulo_bloco(
                "Como a foto ocupa o espaço",
                "A imagem continua inteira no arquivo original.",
            ),
            row![
                opcao("Preencher", "encaixe", 0.0, visual.encaixe == 0),
                opcao("Ajustar", "encaixe", 1.0, visual.encaixe == 1),
                Space::with_width(Length::Fill),
                dica(
                    button(
                        row![
                            text("↔").size(18),
                            text("Horizontal").size(11),
                        ].spacing(5).align_y(Alignment::Center),
                    )
                    .padding([8, 10])
                    .style(estilo_botao_pequeno(visual.inverter_horizontal))
                    .on_press(Msg::ImagemVisual(
                        "inverter_horizontal",
                        f32::from(!visual.inverter_horizontal),
                    )),
                    "Espelhar da esquerda para a direita",
                ),
                dica(
                    button(
                        row![
                            text("↕").size(18),
                            text("Vertical").size(11),
                        ].spacing(5).align_y(Alignment::Center),
                    )
                    .padding([8, 10])
                    .style(estilo_botao_pequeno(visual.inverter_vertical))
                    .on_press(Msg::ImagemVisual(
                        "inverter_vertical",
                        f32::from(!visual.inverter_vertical),
                    )),
                    "Espelhar de cima para baixo",
                ),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(16),
    );

    let cortar = cartao(
        column![
            titulo_bloco(
                "Corte pelas bordas",
                "Arraste os controles para ocultar partes da foto sem apagar pixels.",
            ),
            row![
                column![
                    controle("Esquerda", "corte_esquerda", margens[0], 0.0, 0.0, 0.45, 0.05, false),
                    controle("Topo", "corte_topo", margens[1], 0.0, 0.0, 0.45, 0.05, false),
                ].spacing(17).width(Length::FillPortion(1)),
                column![
                    controle("Direita", "corte_direita", margens[2], 0.0, 0.0, 0.45, 0.05, false),
                    controle("Base", "corte_base", margens[3], 0.0, 0.0, 0.45, 0.05, false),
                ].spacing(17).width(Length::FillPortion(1)),
            ].spacing(25),
        ].spacing(17),
    );

    column![modos, cortar]
        .spacing(14)
        .into()
}

fn cor<'a>(visual: &VisualImagem) -> Element<'a, Msg> {
    let luz = cartao(
        column![
            titulo_bloco("Luz e definição", "Corrija a exposição e destaque os detalhes."),
            controle("Brilho", "brilho", visual.brilho, 0.0, -1.0, 1.0, 0.05, false),
            controle("Contraste", "contraste", visual.contraste, 0.0, -1.0, 1.0, 0.05, false),
            controle("Nitidez", "nitidez", visual.nitidez, 0.0, 0.0, 1.0, 0.05, false),
        ].spacing(19),
    );
    let cores = cartao(
        column![
            titulo_bloco("Cor", "Controle a intensidade e o tom da imagem."),
            controle("Saturação", "saturacao", visual.saturacao, 1.0, 0.0, 2.0, 0.05, false),
            controle("Temperatura", "temperatura", visual.temperatura, 0.0, -1.0, 1.0, 0.05, false),
            controle("Matiz", "matiz", visual.matiz, 0.0, -180.0, 180.0, 5.0, true),
        ].spacing(19),
    );
    row![
        container(luz).width(Length::FillPortion(1)),
        container(cores).width(Length::FillPortion(1)),
    ]
    .spacing(14)
    .into()
}

fn filtro<'a>(nome: &'static str, numero: u8, atual: u8) -> Element<'a, Msg> {
    let ativo = atual == numero;
    button(
        row![
            if ativo {
                Element::from(icone_check(15.0, "#1a73e8"))
            } else {
                Element::from(icone_ajustar(15.0, "#5f6368"))
            },
            text(nome).size(12),
        ]
        .spacing(7)
        .align_y(Alignment::Center),
    )
    .padding([10, 11])
    .width(Length::FillPortion(1))
    .style(estilo_botao_pequeno(ativo))
    .on_press(Msg::ImagemVisual("filtro", f32::from(numero)))
    .into()
}

fn efeitos<'a>(visual: &VisualImagem, sombra: bool) -> Element<'a, Msg> {
    let filtros = cartao(
        column![
            titulo_bloco("Filtros", "Escolha uma aparência pronta. Você pode ajustar as cores depois."),
            row![
                filtro("Original", 0, visual.filtro),
                filtro("P&B", 1, visual.filtro),
                filtro("Sépia", 2, visual.filtro),
            ].spacing(9),
            row![
                filtro("Negativo", 3, visual.filtro),
                filtro("Pôster", 4, visual.filtro),
                filtro("Solarizar", 5, visual.filtro),
            ].spacing(9),
        ].spacing(12),
    );
    let acabamento = cartao(
        column![
            titulo_bloco("Acabamento", "Destaques e profundidade, sem alterar a foto original."),
            row![
                dica(
                    button(row![
                        icone_ajustar(17.0, "#5f6368"),
                        text(if sombra { "Sombra ativada" } else { "Adicionar sombra" }).size(12),
                    ].spacing(8).align_y(Alignment::Center))
                        .padding([9, 12])
                        .style(estilo_botao_pequeno(sombra))
                        .on_press(Msg::ImagemSombra(!sombra)),
                    "Ativar/desativar sombra projetada",
                ),
                Space::with_width(Length::Fill),
                text("Efeitos podem ser desfeitos com Ctrl+Z")
                    .size(11).color(COR_TEXTO_MUTED()),
            ].align_y(Alignment::Center),
            controle("Reflexo", "reflexo", visual.reflexo, 0.0, 0.0, 1.0, 0.05, false),
            controle(
                "Brilho externo", "brilho_externo",
                visual.brilho_externo, 0.0, 0.0, 1.0, 0.05, false,
            ),
            controle(
                "Perspectiva 2D", "perspectiva",
                visual.perspectiva, 0.0, -1.0, 1.0, 0.05, false,
            ),
        ]
        .spacing(18),
    );
    column![filtros, acabamento].spacing(14).into()
}

fn arquivo<'a>(visual: &VisualImagem, alternativa: &'a str) -> Element<'a, Msg> {
    let origem = cartao(
        column![
            titulo_bloco("Imagem de origem", "Troque a foto sem perder posição, formato e efeitos."),
            row![
                button(row![
                    icone_substituir(18.0, "#5f6368"),
                    text("Substituir imagem…").size(12),
                ].spacing(9).align_y(Alignment::Center))
                    .padding([10, 13])
                    .style(estilo_botao_secundario)
                    .on_press(Msg::ImagemSubstituir),
                Space::with_width(Length::Fill),
                text(if visual.preservar_original { "Original preservado" } else { "Fonte otimizada" })
                    .size(11).color(COR_TEXTO_MUTED()),
            ].align_y(Alignment::Center),
        ].spacing(14),
    );
    let acessibilidade = cartao(
        column![
            titulo_bloco(
                "Texto alternativo",
                "Descreva a imagem para facilitar a acessibilidade.",
            ),
            row![
                text_input("Ex.: Fotografia de uma subestação elétrica", alternativa)
                    .on_input(Msg::ImagemTextoAltAlterado)
                    .on_submit(Msg::ImagemTextoAltAplicar)
                    .size(12)
                    .padding([11, 12])
                    .width(Length::Fill),
                dica(
                    button(icone_check(18.0, "#5f6368"))
                        .padding([10, 12])
                        .style(estilo_botao_secundario)
                        .on_press(Msg::ImagemTextoAltAplicar),
                    "Salvar texto alternativo",
                ),
            ].spacing(10),
        ].spacing(13),
    );
    let otimizar = cartao(
        column![
            titulo_bloco(
                "Tamanho do arquivo",
                "As ações abaixo alteram a fonte incorporada. Ctrl+Z restaura a anterior.",
            ),
            row![
                column![
                    text("Reduzir resolução").size(12).color(COR_TEXTO()),
                    row![
                        button(text("1280 px").size(12)).padding([9, 13])
                            .style(estilo_botao_secundario).on_press(Msg::ImagemReduzir(1280)),
                        button(text("800 px").size(12)).padding([9, 13])
                            .style(estilo_botao_secundario).on_press(Msg::ImagemReduzir(800)),
                    ].spacing(7),
                ].spacing(10).width(Length::FillPortion(1)),
                column![
                    text("Compactar").size(12).color(COR_TEXTO()),
                    row![
                        button(text("Alta · 85%").size(12)).padding([9, 13])
                            .style(estilo_botao_secundario).on_press(Msg::ImagemComprimir(85)),
                        button(text("Menor · 60%").size(12)).padding([9, 13])
                            .style(estilo_botao_secundario).on_press(Msg::ImagemComprimir(60)),
                    ].spacing(7),
                ].spacing(10).width(Length::FillPortion(1)),
            ].spacing(15),
            text("Fotos com transparência permanecem PNG para manter o canal alfa.")
                .size(11).color(COR_TEXTO_MUTED()),
        ].spacing(15),
    );
    column![origem, acessibilidade, otimizar].spacing(14).into()
}

pub fn sobrepor<'a>(
    base: Element<'a, crate::app::Mensagem>,
    visual: &'a VisualImagem,
    alternativa: &'a str,
    sombra: bool,
    margens: [f32; 4],
    largura_janela: f32,
    altura_janela: f32,
    pagina: Pagina,
) -> Element<'a, crate::app::Mensagem> {
    let fundo = mouse_area(
        container(Space::new(Length::Fill, Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.02, 0.04, 0.09, 0.38))),
                ..Default::default()
            }),
    )
    .on_press(crate::app::Mensagem::Faixa(Msg::ImagemFecharAjustes));

    let cabecalho = row![
        container(icone_imagem(22.0, "#1a73e8"))
            .padding(11)
            .style(|_| container::Style {
                background: Some(Background::Color(COR_AZUL_FUNDO())),
                border: Border { radius: 12.0.into(), ..Default::default() },
                ..Default::default()
            }),
        column![
            text("Editar imagem").size(20).color(COR_TEXTO()),
            text("Ajustes aplicados diretamente à apresentação")
                .size(11).color(COR_TEXTO_MUTED()),
        ].spacing(3),
        Space::with_width(Length::Fill),
        dica(
            button(icone_fechar_janela(18.0, "#5f6368"))
                .padding([9, 10])
                .style(estilo_botao_pequeno(false))
                .on_press(Msg::ImagemFecharAjustes),
            "Fechar ajustes",
        ),
    ].spacing(13).align_y(Alignment::Center);

    let navegacao = row![
        aba("Enquadrar", icone_enquadramento(17.0, "#5f6368"), Pagina::Enquadrar, pagina == Pagina::Enquadrar),
        aba("Cor", icone_paleta(17.0, "#5f6368"), Pagina::Cor, pagina == Pagina::Cor),
        aba("Efeitos", icone_ajustar(17.0, "#5f6368"), Pagina::Efeitos, pagina == Pagina::Efeitos),
        aba("Arquivo", icone_pasta(17.0, "#5f6368"), Pagina::Arquivo, pagina == Pagina::Arquivo),
    ].spacing(7);

    let conteudo = match pagina {
        Pagina::Enquadrar => enquadrar(visual, margens),
        Pagina::Cor => cor(visual),
        Pagina::Efeitos => efeitos(visual, sombra),
        Pagina::Arquivo => arquivo(visual, alternativa),
    };
    let altura_conteudo = (altura_janela - 270.0).clamp(185.0, 400.0);
    let corpo = scrollable(conteudo)
        .height(Length::Fixed(altura_conteudo))
        .direction(iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Scrollbar::new()
                .width(5).scroller_width(5).spacing(3),
        ));

    let rodape = row![
        icone_check(16.0, "#5f6368"),
        text("Ajustes reversíveis").size(11).color(COR_TEXTO_MUTED()),
        Space::with_width(Length::Fill),
        dica(
            button(row![
                icone_limpar(15.0, "#5f6368"),
                text("Redefinir").size(12),
            ].spacing(7).align_y(Alignment::Center))
                .padding([9, 12])
                .style(estilo_botao_secundario)
                .on_press(Msg::ImagemResetarVisual),
            "Zerar correções de cor e efeitos; não altera a imagem original",
        ),
        button(text("Concluir").size(12))
            .padding([10, 19])
            .style(estilo_botao_primario)
            .on_press(Msg::ImagemFecharAjustes),
    ].spacing(11).align_y(Alignment::Center);

    let largura_painel = (largura_janela - 30.0).min(730.0).max(300.0);
    let painel = container(
        column![
            cabecalho,
            navegacao,
            corpo,
            rodape,
        ]
        .spacing(17),
    )
    .width(Length::Fixed(largura_painel))
    .padding(21)
    .style(|_| container::Style {
        background: Some(Background::Color(COR_PAINEL())),
        text_color: Some(COR_TEXTO()),
        border: Border {
            color: COR_BORDA(),
            width: 1.0,
            radius: 16.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
            offset: Vector::new(0.0, 12.0),
            blur_radius: 30.0,
        },
    });

    let x = ((largura_janela - largura_painel) / 2.0).max(4.0);
    let y = ((altura_janela - altura_conteudo - 222.0) / 2.0).max(10.0);
    let janela = container(Element::from(painel).map(crate::app::Mensagem::Faixa))
        .padding(iced::Padding {
            top: y, left: x, right: 0.0, bottom: 0.0,
        });
    iced::widget::stack![base, fundo, janela].into()
}
