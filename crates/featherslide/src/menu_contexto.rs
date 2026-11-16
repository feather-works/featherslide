//! O menu do botão direito: sobre um elemento, sobre o vazio do slide e sobre uma miniatura.
//! Aqui só se monta a lista; cada item manda o mesmo comando que o botão da faixa.

use crate::localidade::tr;
use featherslide_core::Ordem;
use pelicano::{button, column, container, row, text, Space};
use pelicano::{Align, Background, Border, Color, Element, Length, Point, Shadow, Style};

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
                Acao(tr("Recortar"), "Ctrl+X", F::Recortar),
                Acao(tr("Copiar"), "Ctrl+C", F::Copiar),
                Acao(tr("Colar"), "Ctrl+V", F::Colar),
                Acao(tr("Duplicar"), "Ctrl+D", F::Duplicar),
                Acao(tr("Excluir"), tr("Del"), F::Excluir),
                Sep,
                Acao(tr("Trazer para a frente"), "", F::Organizar(Ordem::Frente)),
                Acao(tr("Enviar para trás"), "", F::Organizar(Ordem::Tras)),
                Acao(tr("Avançar uma camada"), "", F::Organizar(Ordem::Avancar)),
                Acao(tr("Recuar uma camada"), "", F::Organizar(Ordem::Recuar)),
                Sep,
                Acao(tr("Centralizar no slide"), "", F::AlinharNoSlide("centro")),
                Acao(tr("Centralizar na vertical"), "", F::AlinharNoSlide("meio")),
            ];
            match t.as_str() {
                "imagem" => v.extend([
                    Sep,
                    Acao(tr("Cortar como figura…"), "", F::AbrirFiguras(crate::galeria_figuras::Figuras::Recorte)),
                    Acao(tr("Ajustar enquadramento (arrastar foto)"), "", F::AlternarModoRecorte),
                    Acao(tr("Centralizar foto no recorte"), "", F::ImagemResetarRecorte),
                ]),
                "tabela" => v.extend([
                    Sep,
                    Acao(tr("Inserir linha"), "", F::TabelaEstrutura("linha+")),
                    Acao(tr("Excluir última linha"), "", F::TabelaEstrutura("linha-")),
                    Acao(tr("Inserir coluna"), "", F::TabelaEstrutura("coluna+")),
                    Acao(tr("Excluir última coluna"), "", F::TabelaEstrutura("coluna-")),
                    Sep,
                    Item::DesligadoComDica(tr("Abrir no FeatherSheet"), tr("Ainda vai ser construído: abrirá esta tabela no FeatherSheet, com fórmulas e dados de verdade.")),
                ]),
                "icone" => v.extend([
                    Sep,
                    Acao(tr("Formato do ícone…"), "", F::TrocarAba(crate::faixa::AbaDaFaixa::Imagem)),
                    Acao(tr("Trocar ícone…"), "", F::AbrirFiguras(crate::galeria_figuras::Figuras::TrocarIcone)),
                    Acao(tr("Cor do ícone…"), "", F::AbrirSeletorCor(crate::faixa::AlvoCor::Preenchimento)),
                    Acao(tr("Aumentar 10%"), "", F::EscalarSelecao(1.1)),
                    Acao(tr("Diminuir 10%"), "", F::EscalarSelecao(0.9)),
                ]),
                "grafico" => v.extend([Sep, Acao(tr("Editar dados do gráfico…"), "", F::EditarDadosGrafico)]),
                _ => {}
            }
            v.extend([Sep, Acao(tr("Enviar ao fundo"), "", F::SelecaoParaFundo)]);
            v
        }
        TipoMenu::Vazio => vec![
            Acao(tr("Colar"), "Ctrl+V", F::Colar),
            Acao(tr("Selecionar tudo"), "Ctrl+A", F::SelecionarTudo),
            Sep,
            Acao(tr("Novo slide"), "Ctrl+M", F::SlideNovo),
            Acao(tr("Editar formas do fundo"), "", F::FundoLiberar),
        ],
        TipoMenu::Ortografia { sugestoes, base } => {
            let mut v: Vec<Item> = sugestoes.iter().map(|s| Item::Sugestao(s.clone(), F::OrtografiaTrocar(s.clone()))).collect();
            if v.is_empty() {
                v.push(Item::Desligado(tr("Sem sugestões")));
            }
            v.extend([Sep, Acao(tr("Ignorar tudo"), "", F::OrtografiaIgnorarTudo), Acao(tr("Adicionar ao dicionário"), "", F::OrtografiaAdicionar), Sep]);
            v.extend(itens(base));
            v
        }
        TipoMenu::Gramatica { mensagem, sugestao, base } => {
            let rotulo = if sugestao.is_empty() { tr("Tirar o espaço").to_string() } else { sugestao.clone() };
            let mut v = vec![Item::Aviso(mensagem.clone()), Item::Sugestao(rotulo, F::OrtografiaTrocar(sugestao.clone())), Sep, Acao(tr("Ignorar este aviso"), "", F::GramaticaIgnorar), Sep];
            v.extend(itens(base));
            v
        }
        TipoMenu::Secao { inicio, recolhida, primeira, ultima } => vec![
            Acao(if *recolhida { tr("Expandir seção") } else { tr("Recolher seção") }, "", F::SecoesAlternar(*inicio)),
            Acao(tr("Recolher todas"), "", F::SecoesRecolherTodas),
            Acao(tr("Expandir todas"), "", F::SecoesExpandirTodas),
            Sep,
            if *primeira { Item::Desligado(tr("Mover seção para cima")) } else { Acao(tr("Mover seção para cima"), "", F::SecaoMover(*inicio, -1)) },
            if *ultima { Item::Desligado(tr("Mover seção para baixo")) } else { Acao(tr("Mover seção para baixo"), "", F::SecaoMover(*inicio, 1)) },
            Acao(tr("Mover slides marcados para esta seção"), "", F::SlidesParaSecao(*inicio)),
            Sep,
            Acao(tr("Renomear seção"), "", F::SecaoRenomear(*inicio)),
            Acao(tr("Remover seção (mantém os slides)"), "", F::SecaoRemover(*inicio, false)),
            Acao(tr("Remover seção e seus slides"), "", F::SecaoRemover(*inicio, true)),
            Sep,
            Acao(tr("Remover todas as seções"), "", F::SecoesRemoverTodas),
        ],
        TipoMenu::Slide { oculto, marcados, .. } if *marcados > 1 => {
            // Vários slides marcados: as ações valem para todos eles.
            vec![
                Acao(tr("Criar seção com os slides marcados"), "", F::SecaoDosMarcados),
                Sep,
                Acao(tr("Copiar slides marcados"), "Ctrl+C", F::SlidesCopiar),
                Acao(tr("Recortar slides marcados"), "Ctrl+X", F::SlidesRecortar),
                Acao(tr("Colar — manter formatação"), "Ctrl+V", F::SlidesColar(featherslide_core::EstiloColagem::ManterOrigem)),
                Acao(tr("Colar — usar tema de destino"), "", F::SlidesColar(featherslide_core::EstiloColagem::AdaptarAoTema)),
                Sep,
                Acao(tr("Duplicar slides marcados"), "", F::SlideDuplicar),
                Acao(if *oculto { tr("Mostrar slides marcados") } else { tr("Ocultar slides marcados") }, "", F::OcultarSlide),
                Acao(tr("Excluir slides marcados"), "", F::SlideExcluir),
                Acao(tr("Aplicar layout aos marcados…"), "", F::AbrirGaleria(false)),
                Acao(tr("Restaurar layouts marcados"), "", F::SlidesRestaurarLayout),
                Sep,
                Acao(tr("Marcar todos os slides"), "", F::SlidesSelecionarTodos),
                Acao(tr("Desmarcar"), "", F::DesmarcarSlides),
            ]
        }
        TipoMenu::Slide { oculto, primeiro, ultimo, .. } => {
            let mut v = vec![
                Acao(tr("Inserir slide antes"), "", F::SlideInserirAntes),
                Acao(tr("Inserir slide depois"), "Ctrl+M", F::SlideInserirDepois),
                Acao(tr("Duplicar slide"), "", F::SlideDuplicar),
                Acao(tr("Copiar slide"), "Ctrl+C", F::SlidesCopiar),
                Acao(tr("Recortar slide"), "Ctrl+X", F::SlidesRecortar),
                Acao(tr("Colar — manter original"), "Ctrl+V", F::SlidesColar(featherslide_core::EstiloColagem::ManterOrigem)),
                Acao(tr("Colar — adaptar ao tema"), "", F::SlidesColar(featherslide_core::EstiloColagem::AdaptarAoTema)),
                Acao(tr("Excluir slide"), "", F::SlideExcluir), Sep,
            ];
            v.push(Acao(if *oculto { tr("Mostrar slide") } else { tr("Ocultar slide") }, "", F::OcultarSlide));
            v.push(Sep);
            v.push(if *primeiro { Item::Desligado(tr("Mover para cima")) } else { Acao(tr("Mover para cima"), "", F::SlideMover(-1)) });
            v.push(if *ultimo { Item::Desligado(tr("Mover para baixo")) } else { Acao(tr("Mover para baixo"), "", F::SlideMover(1)) });
            v.extend([
                Sep,
                Acao(tr("Alterar layout…"), "", F::AbrirGaleria(false)),
                Acao(tr("Restaurar layout"), "", F::SlidesRestaurarLayout),
                Acao(tr("Reutilizar slides de outro arquivo…"), "", F::SlidesImportar),
                Acao(tr("Importar todos os slides…"), "", F::SlidesImportarTodos),
                Acao(tr("Exportar este slide como PDF…"), "", F::SlideExportarPdf),
                Acao(tr("Ir para o slide número…"), "", F::IrParaNumero),
                Sep,
                Acao(tr("Adicionar seção aqui"), "", F::SecaoNova),
                Acao(tr("Marcar todos os slides"), "", F::SlidesSelecionarTodos),
                Acao(tr("Apresentar a partir daqui"), "", F::ApresentarDoAtual),
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
                container(container(Space::new(0.0, 1)).width(Length::FILL).style(estilo_linha)).padding([4, 6]),
            ),
            Item::Acao(rotulo, atalho, msg) => col.push(
                button(
                    row![text(rotulo).size(13).width(Length::FILL), text(atalho).size(11.5).color(COR_TEXTO_MUTED())]
                        .align(Align::Center),
                )
                .width(Length::FILL)
                .padding([5, 12])
                .style(estilo_botao_pequeno(false))
                .on_press(msg),
            ),
            Item::Aviso(texto) => col.push(
                container(text(texto).size(12).color(COR_TEXTO_MUTED()).width(Length::Fixed(LARGURA - 24.0))).padding([5, 12]),
            ),
            Item::Sugestao(texto, msg) => col.push(
                button(text(texto).size(13).font(pelicano::Font { weight: 700, ..pelicano::Font::DEFAULT }))
                    .width(Length::FILL)
                    .padding([5, 12])
                    .style(estilo_botao_pequeno(false))
                    .on_press(msg),
            ),
            Item::DesligadoComDica(rotulo, dica) => col.push(
                pelicano::tooltip(
                    button(text(rotulo).size(13).color(COR_TEXTO_MUTED())).width(Length::FILL).padding([5, 12]).style(estilo_botao_pequeno(false)),
                    dica,
                    pelicano::TooltipPosition::Right,
                )
                .padding([5, 9])
                .style(estilo_dica),
            ),
            Item::Desligado(rotulo) => col.push(
                button(text(rotulo).size(13)).width(Length::FILL).padding([5, 12]).style(estilo_botao_pequeno(false)),
            ),
        };
    }
    container(col)
        .padding([6, 0])
        .style(|_| Style {
            background: Some(Background::Color(COR_PAINEL())),
            border: Border { color: COR_BORDA(), width: 1.0, radius: [8.0; 4] },
            shadow: Some(Shadow { color: Color::rgb8(0, 0, 0).alpha(0.28), offset: Point::new(0.0, 4.0), blur: 14.0 }),
            ..Default::default()
        })
        .into()
}
