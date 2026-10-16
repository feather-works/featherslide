//! Faixa de opções (Ribbon) do FeatherSlide reproduzindo a estética e organização visual exata
//! com ícones vetoriais SVG reais desenhados para a suíte.
//! Migração incremental APP-004: grupos, botões compactos, dicas e abas
//! usam os componentes Iced compartilhados de feather-ui. Eventos e comandos
//! permanecem no FeatherSlide; F1/F12/Ctrl+F1 usam o roteador global.

use crate::localidade::texto as tr;
use featherslide_core::modelo::{Alinhamento, Direcao, InicioAnimacao, Proporcao, Tipo, TipoAnimacao, TipoTransicao};
use iced::widget::{button, column, container, pick_list, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

use crate::icones::*;
use crate::tema::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AbaDaFaixa {
    #[default]
    Inicio,
    Inserir,
    Organizar,
    Design,
    Transicoes,
    Animacoes,
    Apresentacao,
    Revisao,
    Exibir,
    /// Aba contextual: só aparece com uma imagem selecionada.
    Imagem,
}

/// Qual cor o seletor está mexendo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlvoCor {
    Preenchimento,
    Contorno,
    /// A cor das letras.
    Texto,
    /// A cor de fundo atrás das letras.
    Realce,
    /// A cor que o remover fundo da imagem tira (vazia = a dos cantos).
    Fundo,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FaixaMensagem {
    /// Abre (ou fecha) o seletor de cor do preenchimento ou do contorno.
    AbrirSeletorCor(AlvoCor),
    FecharSeletorCor,
    /// Um canal (0 = R, 1 = G, 2 = B, 3 = A) mudou num deslizador.
    CanalCor(AlvoCor, usize, u8),
    /// Uma cor pronta da grade (vazia = sem cor).
    CorEscolhida(AlvoCor, String),
    /// Arrasto no quadrado: saturação e brilho (0 a 1).
    SeletorSV(AlvoCor, f32, f32),
    /// Arrasto na barra de matiz (graus).
    SeletorMatiz(AlvoCor, f32),
    /// Arrasto na barra de transparência (0 a 1).
    SeletorAlfa(AlvoCor, f32),
    /// Liga ou desliga o conta-gotas: o próximo clique no slide escolhe a cor.
    ConteGotas(AlvoCor),
    /// Um campo numérico (R, G, B ou A) digitado.
    TextoCanal(AlvoCor, usize, String),
    /// O código hexadecimal digitado.
    TextoCor(AlvoCor, String),
    TrocarAba(AbaDaFaixa),
    /// Opacidade (0..1) da imagem selecionada.
    ImagemOpacidade(f32),
    /// Entrada percentual livre, sem botões com valores predefinidos.
    ImagemOpacidadeTexto(String),
    ImagemOpacidadeAplicar,
    /// Controles compactos incorporados à própria faixa Formato da Imagem.
    ImagemAjusteSelecionado(&'static str),
    ImagemEfeitoSelecionado(&'static str),
    ImagemBordaSelecionada(&'static str),
    ImagemFundoDetalhes(bool),
    ImagemAltExpandir(bool),
    ImagemCortarMargem(&'static str, f32),
    /// Abre/fecha o painel com ajustes e filtros avançados.
    ImagemAbrirAjustes,
    /// Troca a aba do painel avançado, sem modificar o documento.
    ImagemPagina(crate::imagem_painel::Pagina),
    ImagemFecharAjustes,
    ImagemVisual(&'static str, f32),
    ImagemResetarVisual,
    ImagemTextoAltAlterado(String),
    ImagemTextoAltAplicar,
    ImagemSombra(bool),
    ImagemSubstituir,
    ImagemReduzir(u32),
    ImagemComprimir(u8),
    /// Tolerância do "remover fundo" da imagem selecionada (0 desliga).
    ImagemRemoverFundo(f32),
    /// Cor do fundo a remover (`#rrggbb`; vazia = a dos cantos).
    ImagemCorDoFundo(String),
    /// Liga o conta-gotas: o próximo clique na imagem do slide escolhe a cor do fundo.
    ImagemPegarCor,
    /// Suavidade (0 duro … 1 suave) da borda do remover fundo.
    ImagemSuavidadeFundo(f32),
    /// Técnica do remover fundo: 0 = cor para alfa, 1 = só o fundo ligado à borda.
    ImagemModoFundo(u8),
    /// Liga ou desliga o refino da borda (filtro guiado).
    ImagemRefinarBorda(bool),
    /// Ajuste da borda do fundo removido: (expandir/contrair em px, suavizar em px).
    ImagemAjusteBorda(f32, f32),
    /// Liga a marcação do watershed: `true` = marcar o que manter, `false` = o que remover.
    ImagemMarcar(bool),
    /// Apaga as marcas do watershed.
    ImagemLimparMarcas,
    // Ações de arquivo
    Novo,
    Abrir,
    Salvar,
    SalvarComo,
    SalvarCopia,
    /// Abre o painel de propriedades e estatísticas da apresentação.
    Propriedades,
    ExportarPdf,
    Desfazer,
    Refazer,
    // Slides
    SlideNovo,
    SlideDuplicar,
    SlideExcluir,
    SlideInserirAntes,
    SlideInserirDepois,
    SlidesCopiar,
    SlidesRecortar,
    SlidesColar(featherslide_core::EstiloColagem),
    SlidesSelecionarTodos,
    SlidesRestaurarLayout,
    SlidesImportar,
    SlidesImportarTodos,
    SlideExportarPdf,
    IrParaNumero,
    // Transferência
    Colar,
    ColarSemFormatacao,
    ColarOriginal,
    Pincel,
    FerramentasTexto(crate::ferramentas_texto::Modo),
    Copiar,
    Recortar,
    Duplicar,
    Excluir,
    SelecionarTudo,
    // Fonte
    IndiceTexto(i8),
    EspacamentoLetras(f32),
    Negrito,
    Italico,
    Sublinhado,
    Tachado,
    SublinhadoDuplo,
    TachadoDuplo,
    /// Maiúsculas e minúsculas: no trecho em edição ou no texto inteiro da seleção.
    Caixa(featherslide_core::texto_formato::Caixa),
    AbrirMenuFonte(crate::menu_fonte::MenuFonte),
    FecharMenuFonte,
    AumentarFonte,
    DiminuirFonte,
    LimparFormatacao,
    CorDoTexto(String),
    // Parágrafo
    Marcadores,
    Numeracao,
    DiminuirRecuo,
    AumentarRecuo,
    Alinhar(Alinhamento),
    /// Onde o texto fica dentro da caixa: topo, meio ou base.
    AlinharVertical(featherslide_core::modelo::Vertical),
    // Inserir elementos
    Inserir(Tipo),
    InserirGaleria(&'static str),
    InserirConector(&'static str),
    DesenhoLivre,
    InserirImagem,
    InserirSvg,
    ConverterSvg,
    ExportarSvg,
    InserirTabela(usize, usize),
    InserirGrafico(String),
    /// Modelo de diagrama vetorial nativo da galeria de categorias 9.1.
    DiagramaInserir(featherslide_core::diagramas::Categoria),
    DiagramaEditar,
    InserirIcone(String),
    /// O texto da busca na galeria de ícones.
    BuscaIcones(String),
    InserirWordArt,
    WordArtTexto(String),
    WordArtEstilo(usize),
    WordArtTransformacao(featherslide_core::wordart::Transformacao, f32),
    WordArtSombra(bool),
    InserirDataHora,
    InserirNumeroSlide,
    InserirSimbolo(String),
    AbrirSimbolos,
    /// Cria uma seção com os slides marcados no painel lateral.
    SecaoDosMarcados,
    MarcarTodosSlides,
    DesmarcarSlides,
    /// Mostra ou esconde o painel de seleção (lista dos itens do slide).
    PainelSelecao,
    Agrupar,
    /// Ctrl+G: desagrupa se a seleção já tem grupo; senão, agrupa.
    AgruparOuDesagrupar,
    Desagrupar,
    /// União, interseção, subtração, combinação ou fragmentação das formas selecionadas.
    MesclarFormas(featherslide_core::booleanas::OperacaoBooleana),
    /// Esconde (`true`) ou mostra o item do slide.
    OcultarItem(u64, bool),
    /// Seleciona o item pela lista (`true`: soma à seleção).
    SelecionarItem(u64, bool),
    /// Anda o carrossel de temas da aba Design (positivo avança).
    CarrosselTemas(i32),
    AbrirFiguras(crate::galeria_figuras::Figuras),
    FecharFiguras,
    /// Qual ponta (canto) o arredondar mexe: `None` = todas.
    CantoSel(Option<usize>),
    /// Arredonda (+) ou desarredonda (−) as pontas da forma.
    Arredondar(f32),
    /// Abre as Configurações já na seção Modelos (criar, importar, exportar).
    AbrirConfigModelos,
    /// Abre a galeria de layouts: `true` cria um slide novo, `false` troca o layout do atual.
    AbrirGaleria(bool),
    EscolherLayout(&'static str),
    FecharGaleria,
    /// Volta à tela de início (o documento continua aberto).
    IrParaInicio,
    FecharSimbolos,
    // Desenho / Estilos
    FormatoForma,
    Preenchimento(String),
    Contorno(String),
    /// Espessura do contorno, em pontos.
    EspessuraContorno(f32),
    /// Altera a forma de corte da imagem (elipse, retangulo_arredondado, etc.).
    RecorteForma(Tipo),
    /// Ajusta a posição e o zoom da foto dentro do recorte.
    ImagemAjustarRecorte { dx: f32, dy: f32, zoom: f32 },
    /// Centraliza a foto de volta à posição padrão dentro do recorte.
    ImagemResetarRecorte,
    /// Ativa ou desativa o modo interativo de enquadramento da foto no palco.
    AlternarModoRecorte,
    // Design
    ConfiguracaoSlide,
    Proporcao(Proporcao),
    FundoSlide(String),
    // Apresentação
    Apresentar,
    ApresentarDoAtual,
    ApresentarComNotas,
    PersonalizadasAbrir,
    // Exibição
    ZoomMais,
    ZoomMenos,
    ZoomReset,
    // Janelas
    // Inserir
    InserirEquacao(String),
    IconeSubstituir(String),
    EscalarSelecao(f32),
    TabelaEstrutura(&'static str),
    EditarDadosGrafico,
    /// Exporta o gráfico selecionado pela API Charts/Drawing compartilhada.
    ExportarGraficoSvg,
    // Design
    TemaAplicar(String),
    FundoTodos(String),
    /// Solta as formas do fundo do layout (bolinhas, barras…) para editar.
    FundoLiberar,
    // Link e comentários
    Regua,
    LinkAbrir,
    Imprimir,
    OrtografiaAbrir,
    /// Troca a palavra errada do menu de contexto pela sugestão escolhida.
    OrtografiaTrocar(String),
    OrtografiaIgnorarTudo,
    OrtografiaAdicionar,
    /// Liga ou desliga o sublinhado ondulado.
    OrtografiaAuto,
    /// Liga ou desliga o sublinhado azul da gramática.
    GramaticaAuto,
    /// Esconde o aviso de gramática do menu de contexto.
    GramaticaIgnorar,
    ComentariosExcluirTodos,
    ComentarioAbrir,
    // Seções
    SecaoNova,
    SecaoRenomear(usize),
    SecaoRemover(usize, bool),
    /// Reordena uma seção inteira, sem misturar slides de outros grupos.
    SecaoMover(usize, i32),
    /// Move os slides marcados para a seção cujo cabeçalho foi escolhido.
    SlidesParaSecao(usize),
    SecoesRemoverTodas,
    SecoesRecolherTodas,
    SecoesExpandirTodas,
    SecoesAlternar(usize),
    /// "renomear" ou "remover" a seção do slide atual.
    SecaoDoSlideAtual(&'static str),
    Organizar(featherslide_core::Ordem),
    AlinharNoSlide(&'static str),
    /// Alinha os selecionados entre si (`esquerda`, `centro`, `direita`, `topo`, `meio`, `base`).
    AlinharEntreSi(&'static str),
    /// Distribui com o mesmo espaço (`h` ou `v`).
    Distribuir(&'static str),
    SlideMover(i32),
    /// Manda a seleção para o fundo.
    SelecaoParaFundo,
    // Transições
    TransicaoTipo(TipoTransicao),
    /// Prévia temporária ao entrar no botão de efeito.
    PreviaTransicao(TipoTransicao),
    PreviaAnimacao(TipoAnimacao),
    PreviaEncerrar,
    TransicaoDirecao(Direcao),
    TransicaoDuracao(f32),
    TransicaoApos(f32),
    TransicaoTodos,
    // Animações
    AnimTipo(TipoAnimacao),
    AnimInicio(InicioAnimacao),
    AnimDirecao(Direcao),
    AnimDuracao(f32),
    /// Gira os elementos selecionados (graus, horário).
    GirarSelecao(f32),
    /// Muda o ângulo inicial (`fim == false`) ou final do círculo parcial selecionado.
    AnguloSetor { fim: bool, delta: f32 },
    AnimAtraso(f32),
    AnimOrdem(i32),
    AbrirPainelAnimacao,
    // Apresentação e Exibir
    OcultarSlide,
    Repetir,
    MostrarPainel,
    MostrarNotas,
    Grade,
    Guias,
    Classificacao,
    ExibirNormal,
    FonteEscolhida(String),
    TamanhoTexto(String),
    TamanhoAplicar,
    AbrirAjuda,
    AbrirMonitor,
    AbrirConfig,
}

/// Um rótulo de menu com a seta para baixo (ícone: a fonte do sistema não tem o triângulo).
fn com_seta<'a>(rotulo: &'static str, tamanho: u16) -> Element<'a, FaixaMensagem> {
    row![text(rotulo).size(tamanho), icone_seta_baixo(9.0, "#5f6368")].spacing(3).align_y(Alignment::Center).into()
}

/// As famílias de fonte oferecidas na faixa.
pub const FONTES: [&str; 10] = [
    "Liberation Sans",
    "Arial",
    "Helvetica",
    "DejaVu Sans",
    "Noto Sans",
    "Liberation Serif",
    "Times New Roman",
    "Georgia",
    "Liberation Mono",
    "Courier New",
];

fn dica_cor<'a>(conteudo: impl Into<Element<'a, FaixaMensagem>>, texto: &'static str) -> Element<'a, FaixaMensagem> {
    feather_ui::formatacao::dica(conteudo, texto)
}

pub fn view_faixa<'a>(
    aba_atual: AbaDaFaixa,
    _pode_desfazer: bool,
    _pode_refazer: bool,
    tem_selecao: bool,
    negrito_ativo: bool,
    italico_ativo: bool,
    sublinhado_ativo: bool,
    tachado_ativo: bool,
    fonte_atual: &str,
    tamanho_txt: &str,
    ctx: &crate::faixa_abas::Contexto,
) -> Element<'a, FaixaMensagem> {
    // A aba contextual some junto com a seleção da imagem.
    let tem_formato = crate::faixa_abas::tem_aba_formato(&ctx.tipo_sel);
    let aba_atual = if aba_atual == AbaDaFaixa::Imagem && !tem_formato { AbaDaFaixa::Inicio } else { aba_atual };
    // 1. Barra superior de Abas (Ribbon Header)
    let btn_aba = |aba: AbaDaFaixa, rotulo: &'static str| -> Element<'a, FaixaMensagem> {
        // Preserva as mensagens de Slide e a aba contextual especial separada.
        feather_ui::ribbon::aba(
            &feather_ui::ribbon::AbaRibbon::nova(aba, rotulo),
            &aba_atual,
            FaixaMensagem::TrocarAba,
        )
    };

    let tab_formato: Element<'a, FaixaMensagem> = if tem_formato {
            let ativa = aba_atual == AbaDaFaixa::Imagem;
            let titulo: Element<'a, FaixaMensagem> = text(if ctx.tipo_sel == "imagem" { "Formato da Imagem" } else if ctx.tipo_sel == "icone" { "Formato do Ícone" } else if ctx.tipo_sel == "wordart" { "Formato do WordArt" } else { "Formato da Forma" }).size(13).wrapping(iced::widget::text::Wrapping::None).font(iced::Font {
                weight: iced::font::Weight::Semibold,
                ..iced::Font::DEFAULT
            }).into();
            button(titulo).padding([7, 14]).on_press(FaixaMensagem::TrocarAba(AbaDaFaixa::Imagem)).style(move |_: &iced::Theme, status| {
                // Verde de "aba de contexto", nos dois temas.
                let escuro = crate::tema::escuro();
                let (texto, normal, destaque) = if escuro {
                    (iced::Color::from_rgb8(0x81, 0xc9, 0x95), iced::Color::from_rgb8(0x1f, 0x3a, 0x2b), iced::Color::from_rgb8(0x2a, 0x52, 0x3b))
                } else {
                    (iced::Color::from_rgb8(0x0b, 0x80, 0x43), iced::Color::from_rgb8(0xe6, 0xf4, 0xea), iced::Color::from_rgb8(0xce, 0xea, 0xd6))
                };
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(if ativa || status == iced::widget::button::Status::Hovered { destaque } else { normal })),
                    text_color: texto,
                    border: iced::Border { color: if ativa { texto } else { iced::Color::TRANSPARENT }, width: if ativa { 2.0 } else { 0.0 }, radius: iced::border::Radius::from(3.0) },
                    shadow: iced::Shadow::default(),
                }
            })
            .into()
        } else {
            Space::with_width(0).into()
        };

    let barra_abas = row![
        button(row![icone_casa(14.0, "#5f6368"), text("Início").size(13)].spacing(6).align_y(Alignment::Center)).padding([7, 12]).style(estilo_botao_secundario).on_press(FaixaMensagem::IrParaInicio),
        Space::with_width(6),
        btn_aba(AbaDaFaixa::Inicio, "Página Inicial"),
        btn_aba(AbaDaFaixa::Inserir, "Inserir"),
        btn_aba(AbaDaFaixa::Organizar, "Organizar"),
        btn_aba(AbaDaFaixa::Design, "Design"),
        btn_aba(AbaDaFaixa::Transicoes, "Transições"),
        btn_aba(AbaDaFaixa::Animacoes, "Animações"),
        btn_aba(AbaDaFaixa::Apresentacao, "Apresentação"),
        btn_aba(AbaDaFaixa::Revisao, "Revisão"),
        btn_aba(AbaDaFaixa::Exibir, "Exibir"),
        tab_formato,
        Space::with_width(Length::Fill),
        // No canto direito da barra de abas: Monitor (F12), Ajuda (F1) e Configurações.
        button(text("F12").size(13)).padding([6, 9]).style(estilo_botao_pequeno(false)).on_press(FaixaMensagem::AbrirMonitor),
        feather_ui::componentes::botao_icone(icone_ajuda(19.0, "#5f6368"), "Ajuda (F1)", Some(FaixaMensagem::AbrirAjuda), false),
        feather_ui::componentes::botao_icone(icone_config(19.0, "#5f6368"), "Configurações", Some(FaixaMensagem::AbrirConfig), false),
        Space::with_width(8),
    ]
    .spacing(2)
    .align_y(Alignment::Center);

    // A aparência da forma é editada no painel lateral, sem repetir uma Ribbon vazia.
    if aba_atual==AbaDaFaixa::Imagem && ctx.n_selecao<2
        && !matches!(ctx.tipo_sel.as_str(),"imagem"|"icone"|"wordart"|"caminho") {
        return column![container(barra_abas).padding([0,8]).width(Length::Fill).style(estilo_barra_abas),container(Space::with_height(1)).width(Length::Fill).style(estilo_linha)].into();
    }
    // Helpers para montagem dos grupos
    let criar_grupo = |itens: Element<'a, FaixaMensagem>, rotulo: &'static str| -> Element<'a, FaixaMensagem> {
        feather_ui::componentes::grupo_faixa(itens, rotulo)
    };

    // 2. Conteúdo da aba selecionada
    let conteudo: Element<'a, FaixaMensagem> = match aba_atual {
        AbaDaFaixa::Inicio => {
            // Grupo 1: Área de transferência
            let grupo_transferencia = {
                let btn_colar = button(
                    column![icone_colar(22.0, "#1a73e8"), com_seta("Colar", 11)]
                        .align_x(Alignment::Center)
                        .spacing(3),
                )
                .padding([4, 8])
                .style(estilo_botao_grande(false))
                .on_press(FaixaMensagem::Colar);

                let pilha_acoes = column![
                    feather_ui::componentes::botao_pequeno(
                        icone_recortar(13.0, "#5f6368"),
                        "Recortar",
                        tem_selecao.then_some(FaixaMensagem::Recortar),
                        false,
                    ),
                    feather_ui::componentes::botao_pequeno(
                        icone_copiar(13.0, "#5f6368"),
                        "Copiar",
                        tem_selecao.then_some(FaixaMensagem::Copiar),
                        false,
                    ),
                    feather_ui::componentes::botao_pequeno(
                        icone_duplicar(13.0, "#5f6368"),
                        "Duplicar",
                        tem_selecao.then_some(FaixaMensagem::Duplicar),
                        false,
                    ),
                ]
                .spacing(2);

                criar_grupo(
                    row![btn_colar, pilha_acoes, column![
                        button(row![feather_ui::icones::icone("clipboard",13.0),text(tr("sem_formatacao")).size(11)].spacing(5).align_y(Alignment::Center)).style(estilo_botao_pequeno(false)).on_press(FaixaMensagem::ColarSemFormatacao),
                        button(row![feather_ui::icones::icone("copy",13.0),text(tr("original")).size(11)].spacing(5).align_y(Alignment::Center)).style(estilo_botao_pequeno(false)).on_press(FaixaMensagem::ColarOriginal),
                        button(row![feather_ui::icones::icone("brush",13.0),text(tr("pincel")).size(11)].spacing(5).align_y(Alignment::Center)).style(estilo_botao_pequeno(false)).on_press_maybe(tem_selecao.then_some(FaixaMensagem::Pincel)),
                    ].spacing(2)].spacing(6).align_y(Alignment::Center).into(),
                    "Área de transferência",
                )
            };

            // Grupo 2: Slides
            let grupo_slides = {
                let btn_novo = button(
                    column![icone_novo_slide(22.0, "#1a73e8"), com_seta("Novo slide", 11)]
                        .align_x(Alignment::Center)
                        .spacing(3),
                )
                .padding([4, 8])
                .style(estilo_botao_grande(false))
                .on_press(FaixaMensagem::AbrirGaleria(true));
                let btn_novo: Element<'a, FaixaMensagem> = crate::medido::marcar("novo_slide", btn_novo);

                let pilha_slides = column![
                    feather_ui::componentes::botao_pequeno(
                        icone_duplicar(13.0, "#5f6368"),
                        "Duplicar slide",
                        Some(FaixaMensagem::SlideDuplicar),
                        false,
                    ),
                    feather_ui::componentes::botao_pequeno(
                        icone_excluir(13.0, "#5f6368"),
                        "Excluir slide",
                        Some(FaixaMensagem::SlideExcluir),
                        false,
                    ),
                ]
                .spacing(2);

                let transferencia = row![
                    dica_cor(button(icone_copiar(17.0, "#5f6368")).padding([5, 7])
                        .style(estilo_botao_pequeno(false)).on_press(FaixaMensagem::SlidesCopiar), "Copiar slides selecionados"),
                    dica_cor(button(icone_recortar(17.0, "#5f6368")).padding([5, 7])
                        .style(estilo_botao_pequeno(false)).on_press(FaixaMensagem::SlidesRecortar), "Recortar slides selecionados"),
                    dica_cor(button(icone_colar(17.0, "#5f6368")).padding([5, 7])
                        .style(estilo_botao_pequeno(false))
                        .on_press(FaixaMensagem::SlidesColar(featherslide_core::EstiloColagem::ManterOrigem)), "Colar slides preservando formatação"),
                ].spacing(2);
                criar_grupo(
                    row![btn_novo, pilha_slides, transferencia].spacing(6).align_y(Alignment::Center).into(),
                    "Slides",
                )
            };

            // Grupo 3: Fonte — linha 1: família, tamanho e caixa; linha 2: estilo,
            // índices e cores, em blocos separados (como no PowerPoint).
            let grupo_fonte = {
                use crate::menu_fonte::MenuFonte;
                let atual = FONTES.iter().copied().find(|n| n.eq_ignore_ascii_case(fonte_atual));
                let cor = |ativo: bool| if ativo { "#1a73e8" } else { "#202124" };
                let menu = |m: MenuFonte| crate::menu_fonte::botao(m, ctx.menu_fonte == Some(m), tem_selecao);
                let separador = || container(Space::new(Length::Fixed(1.0), Length::Fixed(16.0))).style(|_| container::Style {
                    background: Some(iced::Background::Color(COR_BORDA())),
                    ..container::Style::default()
                });
                let linha_1 = row![
                    pick_list(&FONTES[..], atual, |n| FaixaMensagem::FonteEscolhida(n.to_string()))
                        .placeholder(if fonte_atual.is_empty() { "Fonte" } else { fonte_atual })
                        .text_size(12)
                        .padding([4, 8])
                        .width(Length::Fixed(150.0)),
                    text_input("", tamanho_txt)
                        .on_input(FaixaMensagem::TamanhoTexto)
                        .on_submit(FaixaMensagem::TamanhoAplicar)
                        .size(12)
                        .padding([4, 6])
                        .width(Length::Fixed(46.0)),
                    feather_ui::componentes::botao_icone(
                        icone_fonte_mais(16.0, "#5f6368"),
                        "Aumentar tamanho da fonte",
                        tem_selecao.then_some(FaixaMensagem::AumentarFonte),
                        false,
                    ),
                    feather_ui::componentes::botao_icone(
                        icone_fonte_menos(16.0, "#5f6368"),
                        "Diminuir tamanho da fonte",
                        tem_selecao.then_some(FaixaMensagem::DiminuirFonte),
                        false,
                    ),
                    separador(),
                    menu(MenuFonte::Caixa),
                    feather_ui::componentes::botao_icone(
                        icone_limpar(14.0, "#5f6368"),
                        "Limpar toda a formatação",
                        tem_selecao.then_some(FaixaMensagem::LimparFormatacao),
                        false,
                    ),
                ]
                .spacing(4)
                .align_y(Alignment::Center);

                let estilo = row![
                    feather_ui::componentes::botao_icone(icone_negrito(14.0, cor(negrito_ativo)), "Negrito (Ctrl+B)", tem_selecao.then_some(FaixaMensagem::Negrito), negrito_ativo),
                    feather_ui::componentes::botao_icone(icone_italico(14.0, cor(italico_ativo)), "Itálico (Ctrl+I)", tem_selecao.then_some(FaixaMensagem::Italico), italico_ativo),
                    feather_ui::componentes::botao_icone(icone_sublinhado(14.0, cor(sublinhado_ativo && !ctx.sublinhado_duplo)), "Sublinhado (Ctrl+U)", tem_selecao.then_some(FaixaMensagem::Sublinhado), sublinhado_ativo && !ctx.sublinhado_duplo),
                    feather_ui::componentes::botao_icone(icone_tachado(14.0, cor(tachado_ativo && !ctx.tachado_duplo)), "Tachado", tem_selecao.then_some(FaixaMensagem::Tachado), tachado_ativo && !ctx.tachado_duplo),
                    menu(MenuFonte::Tracos),
                ]
                .spacing(1)
                .align_y(Alignment::Center);
                let indices = row![
                    feather_ui::componentes::botao_icone(icone_indice_texto(14.0, true), "Sobrescrito (x²): marque o trecho ou clique e digite", tem_selecao.then_some(FaixaMensagem::IndiceTexto(1)), ctx.indice_texto == 1),
                    feather_ui::componentes::botao_icone(icone_indice_texto(14.0, false), "Subscrito (x₂): marque o trecho ou clique e digite", tem_selecao.then_some(FaixaMensagem::IndiceTexto(-1)), ctx.indice_texto == -1),
                    menu(MenuFonte::Espacamento),
                ]
                .spacing(1)
                .align_y(Alignment::Center);
                let cores = row![
                    crate::medido::marcar(
                        "cor_texto",
                        dica_cor(
                            button(com_seta("A", 11))
                                .padding([2, 6])
                                .style(estilo_botao_pequeno(false))
                                .on_press_maybe(tem_selecao.then_some(FaixaMensagem::AbrirSeletorCor(AlvoCor::Texto))),
                            "Cor do texto",
                        ),
                    ),
                    crate::medido::marcar(
                        "cor_realce",
                        dica_cor(
                            button(com_seta("Fundo", 11))
                                .padding([2, 6])
                                .style(estilo_botao_pequeno(false))
                                .on_press_maybe(tem_selecao.then_some(FaixaMensagem::AbrirSeletorCor(AlvoCor::Realce))),
                            "Cor de fundo do texto (marca-texto)",
                        ),
                    ),
                ]
                .spacing(1)
                .align_y(Alignment::Center);
                let linha_2 = row![estilo, separador(), indices, separador(), cores]
                    .spacing(6)
                    .align_y(Alignment::Center);

                criar_grupo(
                    column![linha_1, linha_2].spacing(6).into(),
                    "Fonte",
                )
            };

            // Grupo 4: Parágrafo
            let grupo_paragrafo = {
                let linha_1 = row![
                    button(
                        row![icone_lista(13.0, "#5f6368"), text("Lista").size(11)]
                            .spacing(4)
                            .align_y(Alignment::Center),
                    )
                    .padding([2, 5])
                    .style(estilo_botao_pequeno(false))
                    .on_press_maybe(tem_selecao.then_some(FaixaMensagem::Marcadores)),
                    button(
                        row![icone_lista_num(13.0, "#5f6368"), text("Num").size(11)]
                            .spacing(4)
                            .align_y(Alignment::Center),
                    )
                    .padding([2, 5])
                    .style(estilo_botao_pequeno(false))
                    .on_press_maybe(tem_selecao.then_some(FaixaMensagem::Numeracao)),
                    button(icone_recuo_menos(13.0, "#5f6368"))
                        .padding([4, 6])
                        .style(estilo_botao_pequeno(false))
                        .on_press_maybe(tem_selecao.then_some(FaixaMensagem::DiminuirRecuo)),
                    button(icone_recuo_mais(13.0, "#5f6368"))
                        .padding([4, 6])
                        .style(estilo_botao_pequeno(false))
                        .on_press_maybe(tem_selecao.then_some(FaixaMensagem::AumentarRecuo)),
                ]
                .spacing(3);

                let vertical_botao = |i: usize, dica_txt: &str, v: featherslide_core::modelo::Vertical, nome: &str| -> Element<'a, FaixaMensagem> {
                    let ativo = ctx.vertical == nome;
                    let b = button(icone_texto_vertical(i, 18.0, "#5f6368")).padding([2, 4]).style(estilo_botao_pequeno(ativo)).on_press_maybe(tem_selecao.then_some(FaixaMensagem::AlinharVertical(v)));
                    feather_ui::formatacao::dica(b, dica_txt)
                };
                // Alinhamento tipado e tooltip agora são construídos pelo
                // feather-ui; o comando continua sendo do motor FeatherSlide.
                let linha_2 = feather_ui::alinhamento::seletor_horizontal(
                    feather_ui::alinhamento::Horizontal::de_identificador(&ctx.alinhamento),
                    tem_selecao,
                    true,
                    |opcao| FaixaMensagem::Alinhar(match opcao {
                        feather_ui::alinhamento::Horizontal::Esquerda => Alinhamento::Esquerda,
                        feather_ui::alinhamento::Horizontal::Centro => Alinhamento::Centro,
                        feather_ui::alinhamento::Horizontal::Direita => Alinhamento::Direita,
                        feather_ui::alinhamento::Horizontal::Justificado => Alinhamento::Justificado,
                    }),
                );
                let linha_3 = row![
                    vertical_botao(0, "Alinhar o texto ao topo", featherslide_core::modelo::Vertical::Topo, "topo"),
                    vertical_botao(1, "Centralizar o texto na vertical (meio)", featherslide_core::modelo::Vertical::Meio, "meio"),
                    vertical_botao(2, "Alinhar o texto à base", featherslide_core::modelo::Vertical::Base, "base"),
                ]
                .spacing(3)
                .align_y(Alignment::Center);

                criar_grupo(
                    column![linha_1, linha_2, linha_3].spacing(3).into(),
                    "Parágrafo",
                )
            };

            // Grupo 6: Edição
            let grupo_edicao = {
                let pilha_edicao = column![
                    button(
                        row![icone_buscar(13.0, "#5f6368"), text("Localizar").size(11)]
                            .spacing(5)
                            .align_y(Alignment::Center),
                    )
                    .padding([2, 6])
                    .style(estilo_botao_pequeno(false)),
                    button(
                        row![icone_substituir(13.0, "#5f6368"), text("Substituir").size(11)]
                            .spacing(5)
                            .align_y(Alignment::Center),
                    )
                    .padding([2, 6])
                    .style(estilo_botao_pequeno(false)),
                    button(
                        row![icone_selecionar(13.0, "#5f6368"), text("Selecionar tudo").size(11)]
                            .spacing(5)
                            .align_y(Alignment::Center),
                    )
                    .padding([2, 6])
                    .style(estilo_botao_pequeno(false))
                    .on_press(FaixaMensagem::SelecionarTudo),
                ]
                .spacing(2);

                criar_grupo(pilha_edicao.into(), "Edição")
            };

            row![
                grupo_transferencia,
                grupo_slides,
                grupo_fonte,
                grupo_paragrafo,
                grupo_edicao,
            ]
            .spacing(4)
            .align_y(Alignment::Center)
            .into()
        }
        AbaDaFaixa::Inserir => crate::faixa_abas::inserir(ctx),
        AbaDaFaixa::Organizar => crate::faixa_abas::organizar(ctx),
        AbaDaFaixa::Design => crate::faixa_abas::design(ctx),
        AbaDaFaixa::Transicoes => crate::faixa_abas::transicoes(ctx),
        AbaDaFaixa::Animacoes => crate::faixa_abas::animacoes(ctx),
        AbaDaFaixa::Apresentacao => crate::faixa_abas::apresentacao(ctx),
        AbaDaFaixa::Revisao => crate::faixa_abas::revisao(ctx),
        AbaDaFaixa::Exibir => crate::faixa_abas::exibir(ctx),
        AbaDaFaixa::Imagem => crate::faixa_abas::imagem(ctx),
    };

    // A faixa contextual de imagem usa mais linhas, em vez de abrir um painel modal.
    // Quando a janela for estreita, os grupos continuam acessíveis por rolagem horizontal.
    let imagem = aba_atual == AbaDaFaixa::Imagem && ctx.tipo_sel == "imagem";
    let altura = if imagem {
        if ctx.imagem_fundo_detalhes { 173.0 }
        else if ctx.imagem_alt_expandido || matches!(ctx.fundo_modo, 3 | 4) { 151.0 }
        else { 138.0 }
    } else { 106.0 };
    let conteudo = if imagem {
        // O construtor iced::widget::scrollable começa sempre em modo vertical.
        // Se o conteúdo herdar Fill na altura, o Iced 0.13 dispara panic
        // ANTES de .direction(Horizontal) ser aplicado. Um contêiner Shrink
        // garante que a própria construção da faixa seja segura.
        let faixa_largura_natural = container(conteudo)
            .width(Length::Shrink)
            .height(Length::Shrink);
        iced::widget::scrollable(faixa_largura_natural)
            .direction(iced::widget::scrollable::Direction::Horizontal(
                iced::widget::scrollable::Scrollbar::new()
                    .width(5).scroller_width(5).spacing(2),
            ))
            .height(Length::Fixed(altura - 10.0))
            .into()
    } else { conteudo };
    let painel_corpo = container(conteudo)
        .padding(iced::Padding { top: 5.0, right: 10.0, bottom: 2.0, left: 10.0 })
        .width(Length::Fill)
        .height(Length::Fixed(altura))
        .style(estilo_corpo_faixa);

    column![
        container(barra_abas).padding([0, 8]).width(Length::Fill).style(estilo_barra_abas),
        container(Space::with_height(1)).width(Length::Fill).style(estilo_linha),
        painel_corpo,
        container(Space::with_height(1)).width(Length::Fill).style(estilo_linha),
    ]
    .into()
}
