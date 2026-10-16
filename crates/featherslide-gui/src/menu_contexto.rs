//! O menu do botão direito: sobre um elemento, sobre o vazio do slide e sobre uma miniatura.
//! Aqui só se monta a lista; cada item manda o mesmo comando que o botão da faixa.

use featherslide_core::Ordem;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Vector};

use crate::faixa::FaixaMensagem;
use crate::tema::*;

pub const LARGURA: f32 = 238.0;
const ALTURA_ITEM: f32 = 29.0;

#[derive(Clone, Debug, PartialEq)]
pub enum TipoMenu {
    /// Sobre um elemento (o tipo dele: `tabela`, `grafico`, `texto`…).
    Elemento(String),
    /// Sobre o vazio do slide.
    Vazio,
    /// Sobre uma miniatura (o slide já foi tornado o atual).
    Slide { oculto: bool, primeiro: bool, ultimo: bool, marcados: usize },
    /// Sobre o cabeçalho de uma seção (o slide que a abre; se está recolhida).
    Secao { inicio: usize, recolhida: bool, primeira: bool, ultima: bool },
    /// Sobre uma palavra errada: as correções e, abaixo, o menu normal do elemento.
    Ortografia { sugestoes: Vec<String>, base: Box<TipoMenu> },
    /// Sobre um trecho com problema de gramática: a explicação, a troca sugerida e o menu normal.
    Gramatica { mensagem: String, sugestao: String, base: Box<TipoMenu> },
}

#[derive(Clone, Debug)]
pub struct MenuCtx {
    /// Onde abriu, em pixels da janela.
    pub x: f32,
    pub y: f32,
    pub tipo: TipoMenu,
}

enum Item {
    Acao(&'static str, &'static str, FaixaMensagem),
    Desligado(&'static str),
    /// Desligado, com uma dica ao passar o mouse (o que vai existir).
    DesligadoComDica(&'static str, &'static str),
    /// Texto que explica, sem ação (desligado).
    Aviso(String),
    /// Item com texto que só se sabe na hora (uma sugestão de correção), em negrito.
    Sugestao(String, FaixaMensagem),
    Sep,
}

fn itens(tipo: &TipoMenu) -> Vec<Item> {
    use FaixaMensagem as F;
    use Item::{Acao, Sep};
    match tipo {
        TipoMenu::Elemento(t) => {
            let mut v = vec![
                Acao("Recortar", "Ctrl+X", F::Recortar),
                Acao("Copiar", "Ctrl+C", F::Copiar),
                Acao("Colar", "Ctrl+V", F::Colar),
                Acao("Duplicar", "Ctrl+D", F::Duplicar),
                Acao("Excluir", "Del", F::Excluir),
                Sep,
                Acao("Trazer para a frente", "", F::Organizar(Ordem::Frente)),
                Acao("Enviar para trás", "", F::Organizar(Ordem::Tras)),
                Acao("Avançar uma camada", "", F::Organizar(Ordem::Avancar)),
                Acao("Recuar uma camada", "", F::Organizar(Ordem::Recuar)),
                Sep,
                Acao("Centralizar no slide", "", F::AlinharNoSlide("centro")),
                Acao("Centralizar na vertical", "", F::AlinharNoSlide("meio")),
            ];
            match t.as_str() {
                "imagem" => v.extend([
                    Sep,
                    Acao("Cortar como figura…", "", F::AbrirFiguras(crate::galeria_figuras::Figuras::Recorte)),
                    Acao("Ajustar enquadramento (arrastar foto)", "", F::AlternarModoRecorte),
                    Acao("Centralizar foto no recorte", "", F::ImagemResetarRecorte),
                ]),
                "tabela" => v.extend([
                    Sep,
                    Acao("Inserir linha", "", F::TabelaEstrutura("linha+")),
                    Acao("Excluir última linha", "", F::TabelaEstrutura("linha-")),
                    Acao("Inserir coluna", "", F::TabelaEstrutura("coluna+")),
                    Acao("Excluir última coluna", "", F::TabelaEstrutura("coluna-")),
                    Sep,
                    Item::DesligadoComDica("Abrir no FeatherSheet", "Ainda vai ser construído: abrirá esta tabela no FeatherSheet, com fórmulas e dados de verdade."),
                ]),
                "icone" => v.extend([
                    Sep,
                    Acao("Formato do ícone…", "", F::TrocarAba(crate::faixa::AbaDaFaixa::Imagem)),
                    Acao("Trocar ícone…", "", F::AbrirFiguras(crate::galeria_figuras::Figuras::TrocarIcone)),
                    Acao("Cor do ícone…", "", F::AbrirSeletorCor(crate::faixa::AlvoCor::Preenchimento)),
                    Acao("Aumentar 10%", "", F::EscalarSelecao(1.1)),
                    Acao("Diminuir 10%", "", F::EscalarSelecao(0.9)),
                ]),
                "grafico" => v.extend([Sep, Acao("Editar dados do gráfico…", "", F::EditarDadosGrafico)]),
                _ => {}
            }
            v.extend([Sep, Acao("Enviar ao fundo", "", F::SelecaoParaFundo)]);
            v
        }
        TipoMenu::Vazio => vec![
            Acao("Colar", "Ctrl+V", F::Colar),
            Acao("Selecionar tudo", "Ctrl+A", F::SelecionarTudo),
            Sep,
            Acao("Novo slide", "Ctrl+M", F::SlideNovo),
            Acao("Editar formas do fundo", "", F::FundoLiberar),
        ],
        TipoMenu::Ortografia { sugestoes, base } => {
            let mut v: Vec<Item> = sugestoes.iter().map(|s| Item::Sugestao(s.clone(), F::OrtografiaTrocar(s.clone()))).collect();
            if v.is_empty() {
                v.push(Item::Desligado("Sem sugestões"));
            }
            v.extend([Sep, Acao("Ignorar tudo", "", F::OrtografiaIgnorarTudo), Acao("Adicionar ao dicionário", "", F::OrtografiaAdicionar), Sep]);
            v.extend(itens(base));
            v
        }
        TipoMenu::Gramatica { mensagem, sugestao, base } => {
            let rotulo = if sugestao.is_empty() { "Tirar o espaço".to_string() } else { sugestao.clone() };
            let mut v = vec![Item::Aviso(mensagem.clone()), Item::Sugestao(rotulo, F::OrtografiaTrocar(sugestao.clone())), Sep, Acao("Ignorar este aviso", "", F::GramaticaIgnorar), Sep];
            v.extend(itens(base));
            v
        }
        TipoMenu::Secao { inicio, recolhida, primeira, ultima } => vec![
            Acao(if *recolhida { "Expandir seção" } else { "Recolher seção" }, "", F::SecoesAlternar(*inicio)),
            Acao("Recolher todas", "", F::SecoesRecolherTodas),
            Acao("Expandir todas", "", F::SecoesExpandirTodas),
            Sep,
            if *primeira { Item::Desligado("Mover seção para cima") } else { Acao("Mover seção para cima", "", F::SecaoMover(*inicio, -1)) },
            if *ultima { Item::Desligado("Mover seção para baixo") } else { Acao("Mover seção para baixo", "", F::SecaoMover(*inicio, 1)) },
            Acao("Mover slides marcados para esta seção", "", F::SlidesParaSecao(*inicio)),
            Sep,
            Acao("Renomear seção", "", F::SecaoRenomear(*inicio)),
            Acao("Remover seção (mantém os slides)", "", F::SecaoRemover(*inicio, false)),
            Acao("Remover seção e seus slides", "", F::SecaoRemover(*inicio, true)),
            Sep,
            Acao("Remover todas as seções", "", F::SecoesRemoverTodas),
        ],
        TipoMenu::Slide { oculto, marcados, .. } if *marcados > 1 => {
            // Vários slides marcados: as ações valem para todos eles.
            vec![
                Acao("Criar seção com os slides marcados", "", F::SecaoDosMarcados),
                Sep,
                Acao("Copiar slides marcados", "Ctrl+C", F::SlidesCopiar),
                Acao("Recortar slides marcados", "Ctrl+X", F::SlidesRecortar),
                Acao("Colar — manter formatação", "Ctrl+V", F::SlidesColar(featherslide_core::EstiloColagem::ManterOrigem)),
                Acao("Colar — usar tema de destino", "", F::SlidesColar(featherslide_core::EstiloColagem::AdaptarAoTema)),
                Sep,
                Acao("Duplicar slides marcados", "", F::SlideDuplicar),
                Acao(if *oculto { "Mostrar slides marcados" } else { "Ocultar slides marcados" }, "", F::OcultarSlide),
                Acao("Excluir slides marcados", "", F::SlideExcluir),
                Acao("Aplicar layout aos marcados…", "", F::AbrirGaleria(false)),
                Acao("Restaurar layouts marcados", "", F::SlidesRestaurarLayout),
                Sep,
                Acao("Marcar todos os slides", "", F::SlidesSelecionarTodos),
                Acao("Desmarcar", "", F::DesmarcarSlides),
            ]
        }
        TipoMenu::Slide { oculto, primeiro, ultimo, .. } => {
            let mut v = vec![
                Acao("Inserir slide antes", "", F::SlideInserirAntes),
                Acao("Inserir slide depois", "Ctrl+M", F::SlideInserirDepois),
                Acao("Duplicar slide", "", F::SlideDuplicar),
                Acao("Copiar slide", "Ctrl+C", F::SlidesCopiar),
                Acao("Recortar slide", "Ctrl+X", F::SlidesRecortar),
                Acao("Colar — manter original", "Ctrl+V", F::SlidesColar(featherslide_core::EstiloColagem::ManterOrigem)),
                Acao("Colar — adaptar ao tema", "", F::SlidesColar(featherslide_core::EstiloColagem::AdaptarAoTema)),
                Acao("Excluir slide", "", F::SlideExcluir), Sep,
            ];
            v.push(Acao(if *oculto { "Mostrar slide" } else { "Ocultar slide" }, "", F::OcultarSlide));
            v.push(Sep);
            v.push(if *primeiro { Item::Desligado("Mover para cima") } else { Acao("Mover para cima", "", F::SlideMover(-1)) });
            v.push(if *ultimo { Item::Desligado("Mover para baixo") } else { Acao("Mover para baixo", "", F::SlideMover(1)) });
            v.extend([
                Sep,
                Acao("Alterar layout…", "", F::AbrirGaleria(false)),
                Acao("Restaurar layout", "", F::SlidesRestaurarLayout),
                Acao("Reutilizar slides de outro arquivo…", "", F::SlidesImportar),
                Acao("Importar todos os slides…", "", F::SlidesImportarTodos),
                Acao("Exportar este slide como PDF…", "", F::SlideExportarPdf),
                Acao("Ir para o slide número…", "", F::IrParaNumero),
                Sep,
                Acao("Adicionar seção aqui", "", F::SecaoNova),
                Acao("Marcar todos os slides", "", F::SlidesSelecionarTodos),
                Acao("Apresentar a partir daqui", "", F::ApresentarDoAtual),
            ]);
            v
        }
    }
}

/// A altura aproximada do menu, para não abrir para fora da janela.
pub fn altura(tipo: &TipoMenu) -> f32 {
    itens(tipo).iter().map(|i| if matches!(i, Item::Sep) { 9.0 } else { ALTURA_ITEM }).sum::<f32>() + 12.0
}

pub fn view<'a>(tipo: &TipoMenu) -> Element<'a, FaixaMensagem> {
    let mut col = column![].spacing(0).width(Length::Fixed(LARGURA));
    for item in itens(tipo) {
        col = match item {
            Item::Sep => col.push(
                container(container(Space::with_height(1)).width(Length::Fill).style(estilo_linha)).padding([4, 6]),
            ),
            Item::Acao(rotulo, atalho, msg) => col.push(
                button(
                    row![text(rotulo).size(13).width(Length::Fill), text(atalho).size(11.5).color(COR_TEXTO_MUTED())]
                        .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .padding([5, 12])
                .style(estilo_botao_pequeno(false))
                .on_press(msg),
            ),
            Item::Aviso(texto) => col.push(
                container(text(texto).size(12).color(COR_TEXTO_MUTED()).width(Length::Fixed(LARGURA - 24.0))).padding([5, 12]),
            ),
            Item::Sugestao(texto, msg) => col.push(
                button(text(texto).size(13).font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT }))
                    .width(Length::Fill)
                    .padding([5, 12])
                    .style(estilo_botao_pequeno(false))
                    .on_press(msg),
            ),
            Item::DesligadoComDica(rotulo, dica) => col.push(
                iced::widget::tooltip(
                    button(text(rotulo).size(13).color(COR_TEXTO_MUTED())).width(Length::Fill).padding([5, 12]).style(estilo_botao_pequeno(false)),
                    container(text(dica).size(12)).padding([5, 9]).max_width(260).style(estilo_dica),
                    iced::widget::tooltip::Position::Right,
                ),
            ),
            Item::Desligado(rotulo) => col.push(
                button(text(rotulo).size(13)).width(Length::Fill).padding([5, 12]).style(estilo_botao_pequeno(false)),
            ),
        };
    }
    container(col)
        .padding([6, 0])
        .style(|_| container::Style {
            background: Some(Background::Color(COR_PAINEL())),
            border: Border { color: COR_BORDA(), width: 1.0, radius: 8.0.into() },
            shadow: Shadow { color: Color::from_rgba8(0, 0, 0, 0.28), offset: Vector::new(0.0, 4.0), blur_radius: 14.0 },
            ..Default::default()
        })
        .into()
}
