//! As abas da faixa depois de "Página inicial": Inserir, Design, Transições, Animações,
//! Apresentação e Exibir. O que o motor decide (temas, efeitos, ordem das animações) vem
//! pronto no `Contexto`; aqui só se desenha e se manda a mensagem.

use crate::localidade::texto as tr;
use featherslide_core::booleanas::OperacaoBooleana;
use featherslide_core::modelo::{Animacao, Direcao, InicioAnimacao, Proporcao, Tipo, TipoAnimacao, TipoTransicao, Transicao};
use featherslide_core::temas::TEMAS;
use pelicano::{button, column, container, pick_list, row, text, Space};
use pelicano::{Align, Background, Border, Color, Element, Length, Status, Style, Theme};

use crate::faixa::FaixaMensagem;
use crate::icones::*;
use crate::tema::*;

type Msg = FaixaMensagem;

/// O que as abas precisam saber do documento e da interface.
pub struct Contexto {
    pub tem_selecao: bool,
    /// O tipo do primeiro elemento selecionado (`tabela`, `grafico`…); vazio se não há.
    pub tipo_sel: String,
    pub tema: String,
    pub proporcao: Proporcao,
    pub fundo: String,
    pub transicao: Transicao,
    pub animacao: Option<Animacao>,
    pub slide_oculto: bool,
    pub repetir: bool,
    pub mostrar_painel: bool,
    pub mostrar_notas: bool,
    pub grade: bool,
    pub guias: bool,
    pub regua: bool,
    pub ortografia_automatica: bool,
    pub gramatica_automatica: bool,
    pub classificacao: bool,
    pub zoom: f32,
    /// A ponta escolhida para arredondar (`None` = todas).
    pub canto: Option<usize>,
    pub carrossel_temas: usize,
    /// Quantos elementos estão selecionados (alinhar pede 2, distribuir pede 3).
    pub n_selecao: usize,
    pub painel_selecao: bool,
    /// Onde o texto fica na caixa selecionada: `topo`, `meio` ou `base` (vazio sem seleção).
    pub vertical: String,
    /// Alinhamento horizontal exibido como seleção na Ribbon compartilhada.
    pub alinhamento: String,
    /// Se o modo interativo de enquadramento da foto no palco está ativo.
    pub modo_recorte: bool,
    /// Parâmetros de remoção do fundo da imagem selecionada.
    pub fundo_removido: f32,
    pub fundo_cor: String,
    pub pegando_cor: bool,
    pub fundo_suave: f32,
    /// 0 = cor para alfa, 1 = só o fundo ligado à borda.
    pub fundo_modo: u8,
    /// Refino da borda (filtro guiado) ligado.
    pub fundo_refino: bool,
    /// Ajuste (px, + expande, − contrai) e suavização (px) da borda.
    pub fundo_borda: f32,
    pub fundo_pena: f32,
    /// Configuração completa da imagem selecionada, para controles compactos da faixa.
    pub visual_imagem: featherslide_core::imagem::VisualImagem,
    pub margens_imagem: [f32; 4],
    pub imagem_sombra: bool,
    pub imagem_ajuste_selecionado: &'static str,
    pub imagem_efeito_selecionado: &'static str,
    pub imagem_borda_selecionada: &'static str,
    pub imagem_fundo_detalhes: bool,
    pub imagem_alt_expandido: bool,
    pub imagem_alt_texto: String,
    pub imagem_opacidade_texto: String,
    /// Marcando o watershed (`Some(true)` = manter, `Some(false)` = remover) e quantas marcas há.
    pub marcando: Option<bool>,
    pub marcas: (usize, usize),
    /// Cores (`#rrggbb`, vazia = sem) e espessura do contorno do elemento selecionado.
    pub preenchimento: String,
    pub contorno: String,
    pub espessura: f32,
    pub indice_texto: i8,
    pub sublinhado_duplo: bool,
    pub tachado_duplo: bool,
    pub menu_fonte: Option<crate::menu_fonte::MenuFonte>,
    pub wordart_texto: String,
    pub wordart_modo: featherslide_core::wordart::Transformacao,
    pub wordart_intensidade: f32,
    pub wordart_sombra: bool,
    pub fonte: String,
    pub tamanho_fonte: f32,
    pub cor_texto: String,
}

/// A aba contextual de formato existe para imagem e para as formas desenhadas.
pub fn tem_aba_formato(tipo: &str) -> bool {
    matches!(tipo, "imagem" | "icone" | "caminho" | "wordart" | "retangulo" | "retangulo_arredondado" | "elipse" | "triangulo" | "seta" | "linha" | "setor")
        || featherslide_core::formas::ADICIONAIS.iter().any(|(t, _)| t.nome() == tipo)
}


/// Um grupo de cor: um botão só, com a cor atual, que abre o seletor completo (quadrado de
/// saturação/brilho, matiz, transparência, conta-gotas e campos R, G, B, A e hex).
fn grupo_cor<'a>(titulo: &'static str, atual: &str, alvo: crate::faixa::AlvoCor, rotulo_grupo: &'static str) -> Element<'a, Msg> {
    use crate::faixa::AlvoCor;
    let nome_marca = match alvo {
        AlvoCor::Preenchimento => "cor_preenchimento",
        AlvoCor::Contorno => "cor_contorno",
        AlvoCor::Texto => "cor_texto",
        AlvoCor::Realce => "cor_realce",
        AlvoCor::Fundo => "cor_fundo_imagem",
    };
    let tem_cor = !atual.is_empty();
    let botao = crate::medido::marcar(
        nome_marca,
        dica(
            feather_ui::formatacao::botao_cor(
                "Cor", atual, Some(Msg::AbrirSeletorCor(alvo)), false,
            ),
            "Escolher a cor: matiz, transparência e valores R, G, B, A e hex",
        ),
    );
    grupo(
        column![
            text(titulo).size(11).no_wrap().color(COR_TEXTO_MUTED()),
            botao,
            text(if tem_cor { atual.to_string() } else { "sem cor".to_string() }).size(10).no_wrap().color(COR_TEXTO_MUTED()),
        ]
        .spacing(4)
        .align(Align::Center)
        .into(),
        rotulo_grupo,
    )
}

/// O grupo do contorno: cor e espessura.
fn grupo_contorno_formato<'a>(c: &Contexto) -> Element<'a, Msg> {
    let cor = grupo_cor("Contorno", &c.contorno, crate::faixa::AlvoCor::Contorno, "Contorno");
    let espessura = grupo(
        column![
            text(format!("Espessura: {} pt", (c.espessura * 10.0).round() / 10.0)).size(10).no_wrap().color(COR_TEXTO_MUTED()),
            row![
                dica(pequeno(icone_menos(12.0, "#5f6368"), "", Some(Msg::EspessuraContorno((c.espessura - 0.5).max(0.0))), false), "Contorno mais fino"),
                dica(pequeno(icone_mais(12.0, "#5f6368"), "", Some(Msg::EspessuraContorno((c.espessura + 0.5).min(50.0))), false), "Contorno mais grosso"),
            ]
            .spacing(2),
        ]
        .spacing(4)
        .into(),
        "Espessura",
    );
    row![cor, espessura].spacing(4).align(Align::Center).into()
}

// ------------------------------------------------------------------ peças

/// Um grupo da faixa: os itens em cima, o rótulo embaixo, uma linha fina à direita.
pub fn grupo<'a>(itens: Element<'a, Msg>, rotulo: &'static str) -> Element<'a, Msg> {
    feather_ui::componentes::grupo_faixa(itens, rotulo)
}

fn dica<'a>(conteudo: impl Into<Element<'a, Msg>>, texto: &str) -> Element<'a, Msg> {
    feather_ui::formatacao::dica(conteudo, texto)
}

fn grande<'a>(icone: pelicano::SvgView, rotulo: &str, msg: Option<Msg>, ativo: bool) -> Element<'a, Msg> {
    feather_ui::componentes::botao_grande(icone, rotulo, msg, ativo)
}

/// Como `grande`, com a setinha de menu ao lado do rótulo (desenhada: o caractere ▾ não existe na fonte).
fn grande_menu<'a>(icone: pelicano::SvgView, rotulo: &str, msg: Option<Msg>, ativo: bool) -> Element<'a, Msg> {
    button(column![icone, row![text(rotulo.to_string()).size(11).no_wrap(), icone_seta_baixo(9.0, "#5f6368")].spacing(3).align(Align::Center)].align(Align::Center).spacing(3))
        .padding([4, 8])
        .style(estilo_botao_grande(ativo))
        .on_press_maybe(msg)
        .into()
}

fn pequeno<'a>(icone: pelicano::SvgView, rotulo: &str, msg: Option<Msg>, ativo: bool) -> Element<'a, Msg> {
    feather_ui::componentes::botao_pequeno(icone, rotulo, msg, ativo)
}

/// Um comando que ainda não existe: aparece apagado, com a dica do que fará.
fn em_breve_grande<'a>(icone: pelicano::SvgView, rotulo: &str, o_que: &str) -> Element<'a, Msg> {
    dica(grande(icone, rotulo, None, false), &format!("Em breve: {o_que}"))
}

fn em_breve_pequeno<'a>(icone: pelicano::SvgView, rotulo: &str, o_que: &str) -> Element<'a, Msg> {
    dica(pequeno(icone, rotulo, None, false), &format!("Em breve: {o_que}"))
}

/// Um menu suspenso com cara de botão da faixa e um ícone em cima.
pub fn menu_com_icone<'a>(
    icone: pelicano::SvgView,
    rotulo: &'static str,
    opcoes: &'static [&'static str],
    ao_escolher: impl Fn(&'static str) -> Msg + 'a,
    largura: f32,
    ativo: bool,
) -> Element<'a, Msg> {
    column![
        icone,
        pick_list(opcoes, None::<&'static str>, ao_escolher)
            .placeholder(rotulo)
            .size(11)
            .padding([2, 6])
            .width(Length::Fixed(largura))
            .handle_color(COR_AZUL_ACENTO())
            .style(move |t, s| {
                let mut e = estilo_menu_da_faixa(t, s);
                if ativo {
                    e.text_color = Some(COR_AZUL_ACENTO());
                }
                e
            }),
    ]
    .align(Align::Center)
    .spacing(3)
    .into()
}

/// Um menu suspenso comum (com o valor atual).
fn menu<'a>(
    atual: &'static str,
    opcoes: &'static [&'static str],
    ao_escolher: impl Fn(&'static str) -> Msg + 'a,
    largura: f32,
    ligado: bool,
) -> Element<'a, Msg> {
    let seletor = pick_list(opcoes, Some(atual), ao_escolher).size(11).padding([3, 8]).width(Length::Fixed(largura));
    if ligado {
        seletor.into()
    } else {
        seletor.handle_color(COR_AZUL_ACENTO()).style(|t, _| Style { text_color: Some(Color { a: 0.4, ..COR_TEXTO() }), ..estilo_menu_da_faixa(t, Status::Idle) }).into()
    }
}

/// O campo das listas suspensas da faixa (o `estilo_menu_da_faixa` do feather-ui Iced):
/// painel com borda, azul claro com o mouse em cima ou aberto.
pub(crate) fn estilo_menu_da_faixa(_: &Theme, status: Status) -> Style {
    let sobre = matches!(status, Status::Hovered | Status::Pressed);
    let fundo = if sobre { COR_AZUL_FUNDO() } else { COR_PAINEL() };
    Style::default().background(fundo).border(Border::rounded(5.0).color(if sobre { COR_FOCO(COR_AZUL_FUNDO()) } else { COR_BORDA() }, 1.0))
}

/// Um ajuste com `−` e `+` e o valor no meio.
fn passo<'a>(rotulo: &str, valor: String, menos: Msg, mais: Msg, ligado: bool) -> Element<'a, Msg> {
    let icone = match rotulo {
        "Duração" => Some(icone_relogio(12.0, "#5f6368")),
        "Avançar após" => Some(icone_avancar_auto(12.0, "#5f6368")),
        "Atraso" => Some(icone_ampulheta(12.0, "#5f6368")),
        _ => None,
    };
    let botao = |t: &str, m: Msg| {
        let conteudo: Element<'a, Msg> = if t == "−" { icone_menos(13.0, "#5f6368").into() } else { icone_mais(13.0, "#5f6368").into() };
        button(container(conteudo).padding([2, 0])).padding([0, 8]).style(estilo_botao_pequeno(false)).on_press_maybe(ligado.then_some(m))
    };
    column![
        match icone {
            Some(i) => Element::from(row![i, text(rotulo.to_string()).size(10.5).color(COR_TEXTO_MUTED())].spacing(4).align(Align::Center)),
            None => text(rotulo.to_string()).size(10.5).color(COR_TEXTO_MUTED()).into(),
        },
        row![
            botao("−", menos),
            container(text(valor).size(12)).width(Length::Fixed(58.0)).align_x(Align::Center),
            botao("+", mais),
        ]
        .align(Align::Center)
    ]
    .spacing(2)
    .into()
}

fn cor(hex: &str) -> Color {
    let v = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0);
    Color::rgb8((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

fn segundos(v: f32) -> String {
    format!("{v:.2} s").replace('.', ",")
}

fn direcao_de(nome: &str) -> Direcao {
    Direcao::TODOS.into_iter().find(|d| d.nome() == nome).unwrap_or_default()
}

const DIRECOES: [&str; 4] = ["Da esquerda", "Da direita", "De cima", "De baixo"];
const INICIOS: [&str; 3] = ["Ao clicar", "Com a anterior", "Após a anterior"];

// ------------------------------------------------------------------ Inserir

const TABELAS: [&str; 8] = ["2 × 2", "3 × 3", "4 × 4", "5 × 3", "6 × 4", "8 × 5", "10 × 6", "12 × 8"];
const EQUACOES: [&str; 8] = ["E = mc²", "a² + b² = c²", "x = (−b ± √(b² − 4ac)) / 2a", "∑ xᵢ / n", "∫ f(x) dx", "F = m · a", "A = π · r²", "e^(iπ) + 1 = 0"];

pub fn inserir<'a>(c: &Contexto) -> Element<'a, Msg> {
    let tabelas = grupo(
        menu_com_icone(icone_tabela(22.0, "#1a73e8"), "Tabela", &TABELAS, |t| {
            let mut p = t.split(" × ").filter_map(|n| n.parse::<usize>().ok());
            Msg::InserirTabela(p.next().unwrap_or(3), p.next().unwrap_or(3))
        }, 80.0, false),
        "Tabelas",
    );
    let imagens = {
        let mut linha = row![
            grande(icone_imagem(22.0, "#1a73e8"), "Imagem…", Some(Msg::InserirImagem), false),
        ];
        if c.tipo_sel == "imagem" {
            linha = linha.push(grupo_recorte_imagem(c));
        }
        linha = linha.push(column![
            pequeno(icone_sorriso(13.0, "#5f6368"), "Ícones", Some(Msg::AbrirFiguras(crate::galeria_figuras::Figuras::Icones)), false),
            pequeno(icone_formas(13.0, "#5f6368"), "SVG…", Some(Msg::InserirSvg), false),
            pequeno(icone_duplicar(13.0, "#5f6368"), "Converter SVG", (c.tipo_sel == "caminho").then_some(Msg::ConverterSvg), false),
            pequeno(icone_salvar(13.0, "#5f6368"), "Exportar SVG…", (c.tipo_sel == "caminho" || c.tipo_sel == "icone").then_some(Msg::ExportarSvg), false),
            em_breve_pequeno(icone_video(13.0, "#5f6368"), "Vídeo", "vídeos do computador dentro do slide"),
            em_breve_pequeno(icone_audio(13.0, "#5f6368"), "Áudio", "som e narração no slide"),
        ].spacing(2));
        grupo(
            linha.spacing(6).align(Align::Center).into(),
            "Mídia",
        )
    };
    let ilustracoes = grupo(
        row![
            crate::medido::marcar("formas", grande_menu(icone_formas(22.0, "#1a73e8"), "Formas", Some(Msg::AbrirFiguras(crate::galeria_figuras::Figuras::Formas)), false)),
            crate::medido::marcar("grafico", grande_menu(icone_grafico(22.0, "#1a73e8"), "Gráfico", Some(Msg::AbrirFiguras(crate::galeria_figuras::Figuras::Graficos)), false)),
            crate::medido::marcar("diagramas", grande_menu(icone_organizar(22.0, "#1a73e8"), "Diagramas", Some(Msg::AbrirFiguras(crate::galeria_figuras::Figuras::Diagramas)), false)),
            pequeno(icone_organizar(13.0, "#5f6368"), tr("diagrama_editar"), c.tem_selecao.then_some(Msg::DiagramaEditar), false),
        ]
        .spacing(6)
        .align(Align::Center)
        .into(),
        "Ilustrações",
    );
    let passo = |rotulo: &'static str, menos: Msg, mais: Msg, ligado: bool| -> Element<'a, Msg> {
        let ic = if rotulo == "Girar" { icone_girar(13.0, "#5f6368") } else { icone_angulo(13.0, "#5f6368") };
        row![
            ic,
            text(rotulo).size(11).width(Length::Fixed(48.0)),
            button(row![icone_menos(11.0, "#5f6368"), text("15°").size(11)].spacing(2).align(Align::Center)).padding([2, 6]).style(estilo_botao_pequeno(false)).on_press_maybe(ligado.then_some(menos)),
            button(text("+15°").size(11)).padding([2, 6]).style(estilo_botao_pequeno(false)).on_press_maybe(ligado.then_some(mais)),
        ]
        .spacing(4)
        .align(Align::Center)
        .into()
    };
    let setor = c.tipo_sel == "setor";
    let girar = grupo(
        column![
            passo("Girar", Msg::GirarSelecao(-15.0), Msg::GirarSelecao(15.0), c.tem_selecao),
            passo("Início", Msg::AnguloSetor { fim: false, delta: -15.0 }, Msg::AnguloSetor { fim: false, delta: 15.0 }, setor),
            passo("Fim", Msg::AnguloSetor { fim: true, delta: -15.0 }, Msg::AnguloSetor { fim: true, delta: 15.0 }, setor),
        ]
        .spacing(2)
        .into(),
        "Girar e ângulos",
    );
    let forma_com_pontas = matches!(c.tipo_sel.as_str(), "retangulo" | "retangulo_arredondado" | "triangulo" | "seta");
    let pontas: &[&str] = match c.tipo_sel.as_str() {
        "retangulo" | "retangulo_arredondado" => &["↖", "↗", "↘", "↙"],
        "triangulo" => &["▲", "◢", "◣"],
        _ => &[],
    };
    let chip = |icone: usize, rotulo: &str, ativo: bool, msg: Msg| -> Element<'a, Msg> {
        let b = button(icone_ponta(icone, 14.0, "#5f6368")).padding([3, 6]).style(estilo_botao_pequeno(ativo)).on_press_maybe(forma_com_pontas.then_some(msg));
        dica(b, rotulo)
    };
    let mut chips = row![chip(0, "Todas as pontas", c.canto.is_none(), Msg::CantoSel(None))].spacing(3);
    let nomes: &[&str] = match c.tipo_sel.as_str() {
        "triangulo" => &["Ponta de cima", "Ponta direita", "Ponta esquerda"],
        _ => &["Canto superior esquerdo", "Canto superior direito", "Canto inferior direito", "Canto inferior esquerdo"],
    };
    for (i, _) in pontas.iter().enumerate() {
        let icone = if c.tipo_sel == "triangulo" { 5 + i } else { 1 + i };
        chips = chips.push(chip(icone, nomes[i], c.canto == Some(i), Msg::CantoSel(Some(i))));
    }
    let cantos = grupo(
        column![
            chips,
            row![
                icone_arredondar(13.0, "#5f6368"),
                text("Arredondar").size(11).width(Length::Fixed(62.0)),
                button(icone_menos(13.0, "#5f6368")).padding([4, 9]).style(estilo_botao_pequeno(false)).on_press_maybe(forma_com_pontas.then_some(Msg::Arredondar(-0.08))),
                button(icone_mais(13.0, "#5f6368")).padding([4, 9]).style(estilo_botao_pequeno(false)).on_press_maybe(forma_com_pontas.then_some(Msg::Arredondar(0.08))),
            ]
            .spacing(4)
            .align(Align::Center),
        ]
        .spacing(4)
        .into(),
        "Pontas",
    );
    let texto = grupo(
        row![
            grande(icone_caixa_texto(22.0, "#1a73e8"), "Caixa de texto", Some(Msg::Inserir(Tipo::Texto)), false),
            column![
                pequeno(icone_wordart(13.0, "#5f6368"), "WordArt", Some(Msg::InserirWordArt), false),
                pequeno(icone_calendario(13.0, "#5f6368"), "Data e hora", Some(Msg::InserirDataHora), false),
                pequeno(icone_numero(13.0, "#5f6368"), "Nº do slide", Some(Msg::InserirNumeroSlide), false),
            ]
            .spacing(2),
        ]
        .spacing(6)
        .align(Align::Center)
        .into(),
        "Texto",
    );
    let simbolos = grupo(
        row![
            menu_com_icone(icone_equacao(22.0, "#1a73e8"), "Equação", &EQUACOES, |e| Msg::InserirEquacao(e.to_string()), 84.0, false),
            crate::medido::marcar("simbolo", grande_menu(icone_simbolo(22.0, "#1a73e8"), "Símbolo", Some(Msg::AbrirSimbolos), false)),
        ]
        .spacing(6)
        .align(Align::Center)
        .into(),
        "Símbolos",
    );
    let links = grupo(
        column![
            pequeno(icone_link(13.0, "#5f6368"), "Link", Some(Msg::LinkAbrir), false),
            pequeno(icone_comentario(13.0, "#5f6368"), "Comentário", Some(Msg::ComentarioAbrir), false),
        ]
        .spacing(2)
        .into(),
        "Links",
    );

    let mut abas: Vec<Element<'a, Msg>> = vec![tabelas, imagens, ilustracoes, girar, cantos, texto, simbolos, links];
    match c.tipo_sel.as_str() {
        "tabela" => {
            abas.push(grupo(
                row![
                    column![
                        pequeno(icone_linha_mais(13.0, "#5f6368"), "Linha", Some(Msg::TabelaEstrutura("linha+")), false),
                        pequeno(icone_linha_menos(13.0, "#5f6368"), "Linha", Some(Msg::TabelaEstrutura("linha-")), false),
                    ]
                    .spacing(2),
                    column![
                        pequeno(icone_coluna_mais(13.0, "#5f6368"), "Coluna", Some(Msg::TabelaEstrutura("coluna+")), false),
                        pequeno(icone_coluna_menos(13.0, "#5f6368"), "Coluna", Some(Msg::TabelaEstrutura("coluna-")), false),
                    ]
                    .spacing(2),
                ]
                .spacing(8)
                .into(),
                "Tabela selecionada",
            ));
            abas.push(grupo(
                column![
                    text(format!("Bordas: {:.1} pt", c.espessura)).size(11),
                    row![
                        dica(pequeno(icone_menos(12.0, "#5f6368"), "", Some(Msg::EspessuraContorno((c.espessura - 0.5).max(0.0))), false), "Diminuir a espessura das bordas da tabela; zero oculta as bordas"),
                        dica(pequeno(icone_mais(12.0, "#5f6368"), "", Some(Msg::EspessuraContorno((c.espessura + 0.5).min(50.0))), false), "Aumentar a espessura das bordas da tabela"),
                    ].spacing(4),
                ].spacing(4).into(),
                "Bordas da tabela",
            ));
        }
        "grafico" => abas.push(grupo(
            column![
                grande(icone_dados(22.0, "#1a73e8"), "Editar dados…", Some(Msg::EditarDadosGrafico), false),
                pequeno(icone_salvar(13.0, "#5f6368"), "Exportar gráfico SVG…", Some(Msg::ExportarGraficoSvg), false),
            ].spacing(4).into(),
            "Gráfico selecionado",
        )),
        _ => {}
    }
    row(abas).spacing(4).align(Align::Center).into()
}

// ------------------------------------------------------------------ Organizar

pub fn organizar<'a>(c: &Contexto) -> Element<'a, Msg> {
    let rotulos = ["Esquerda", "Centro", "Direita", "Topo", "Meio", "Base"];
    let comandos = ["esquerda", "centro", "direita", "topo", "meio", "base"];
    let botoes = |n_minimo: usize, f: fn(&'static str) -> Msg, c: &Contexto| -> Element<'a, Msg> {
        let ativo = c.n_selecao >= n_minimo;
        let mut linha_h = row![].spacing(2);
        let mut linha_v = row![].spacing(2);
        for i in 0..6 {
            let b = button(icone_alinhar(i, 22.0, "#5f6368")).padding([2, 4]).style(estilo_botao_pequeno(false)).on_press_maybe(ativo.then_some(f(comandos[i])));
            let b = dica(b, &format!("{} ({})", rotulos[i], if i < 3 { "horizontal" } else { "vertical" }));
            if i < 3 { linha_h = linha_h.push(b) } else { linha_v = linha_v.push(b) }
        }
        column![linha_h, linha_v].spacing(2).into()
    };
    let na_pagina = grupo(botoes(1, Msg::AlinharNoSlide, c), "Alinhar na página");
    let entre_si = grupo(botoes(2, Msg::AlinharEntreSi, c), "Alinhar entre si");
    let distribuir = grupo(
        column![
            dica(
                button(icone_alinhar(6, 18.0, "#5f6368")).padding([4, 8]).style(estilo_botao_pequeno(false)).on_press_maybe((c.n_selecao >= 3).then_some(Msg::Distribuir("h"))),
                "Mesmo espaço entre os elementos, da esquerda para a direita (3 ou mais)"
            ),
            dica(
                button(icone_alinhar(7, 18.0, "#5f6368")).padding([4, 8]).style(estilo_botao_pequeno(false)).on_press_maybe((c.n_selecao >= 3).then_some(Msg::Distribuir("v"))),
                "Mesmo espaço entre os elementos, de cima para baixo (3 ou mais)"
            ),
        ]
        .spacing(4)
        .into(),
        "Distribuir",
    );
    let centralizar = grupo(
        column![
            dica(button(icone_alinhar(1, 22.0, "#5f6368")).padding([2, 6]).style(estilo_botao_pequeno(false)).on_press_maybe((c.n_selecao >= 1).then_some(Msg::AlinharNoSlide("centro"))), "Centralizar na horizontal"),
            dica(button(icone_alinhar(4, 22.0, "#5f6368")).padding([2, 6]).style(estilo_botao_pequeno(false)).on_press_maybe((c.n_selecao >= 1).then_some(Msg::AlinharNoSlide("meio"))), "Centralizar na vertical"),
        ]
        .spacing(4)
        .into(),
        "Centralizar na página",
    );
    let agrupar = grupo(
        column![
            dica(button(icone_agrupar(18.0, "#5f6368")).padding([4, 8]).style(estilo_botao_pequeno(false)).on_press_maybe((c.n_selecao >= 2).then_some(Msg::Agrupar)), "Agrupar duas ou mais formas selecionadas"),
            dica(button(icone_desagrupar(18.0, "#5f6368")).padding([4, 8]).style(estilo_botao_pequeno(false)).on_press_maybe((c.n_selecao >= 1).then_some(Msg::Desagrupar)), "Desagrupar formas"),
        ]
        .spacing(4)
        .into(),
        "Agrupar",
    );
    let painel = grupo(
        grande(icone_painel(22.0, "#1a73e8"), "Painel de seleção", Some(Msg::PainelSelecao), c.painel_selecao),
        "Itens do slide",
    );
    let linha = row![centralizar, na_pagina, entre_si, distribuir, agrupar, painel].spacing(4).align(Align::Center);
    linha.into()
}

/// Operações sobre as formas selecionadas, na aba contextual de formato.
fn grupo_mesclar_formas<'a>(c: &Contexto) -> Element<'a, Msg> {
    // FTS-0363–0367: comandos calculados integralmente no motor Rust.
    grupo(
        column![
            row![
                dica(button(icone_mesclar_formas(OperacaoBooleana::Uniao, 19.0, "#5f6368")).padding([5, 8]).style(estilo_botao_pequeno(false))
                    .on_press_maybe((c.n_selecao >= 2).then_some(Msg::MesclarFormas(OperacaoBooleana::Uniao))),
                    "União — junta as áreas das formas selecionadas"),
                dica(button(icone_mesclar_formas(OperacaoBooleana::Intersecao, 19.0, "#5f6368")).padding([5, 8]).style(estilo_botao_pequeno(false))
                    .on_press_maybe((c.n_selecao >= 2).then_some(Msg::MesclarFormas(OperacaoBooleana::Intersecao))),
                    "Interseção — mantém apenas a região comum"),
                dica(button(icone_mesclar_formas(OperacaoBooleana::Subtracao, 19.0, "#5f6368")).padding([5, 8]).style(estilo_botao_pequeno(false))
                    .on_press_maybe((c.n_selecao >= 2).then_some(Msg::MesclarFormas(OperacaoBooleana::Subtracao))),
                    "Subtração — retira da primeira forma a área coberta pelas demais"),
                dica(button(icone_mesclar_formas(OperacaoBooleana::Combinacao, 19.0, "#5f6368")).padding([5, 8]).style(estilo_botao_pequeno(false))
                    .on_press_maybe((c.n_selecao >= 2).then_some(Msg::MesclarFormas(OperacaoBooleana::Combinacao))),
                    "Combinação — mantém as áreas exclusivas de cada forma"),
                dica(button(icone_mesclar_formas(OperacaoBooleana::Fragmentacao, 19.0, "#5f6368")).padding([5, 8]).style(estilo_botao_pequeno(false))
                    .on_press_maybe((c.n_selecao >= 2).then_some(Msg::MesclarFormas(OperacaoBooleana::Fragmentacao))),
                    "Fragmentação — separa cada região em formas editáveis"),
            ].spacing(3),
        ].spacing(4).into(),
        "Mesclar formas",
    )
}

/// Mini controle reutilizável: rótulo + diminuir / valor / aumentar, sem presets.
/// Mantém os valores e limites do motor Rust.
fn imagem_ajuste_curto<'a>(
    titulo: &'static str, valor: f32, min: f32, max: f32, _passo: f32,
    diminuir: Msg, aumentar: Msg, graus: bool,
) -> Element<'a, Msg> {
    let numero = if graus {
        format!("{valor:.0}°")
    } else {
        format!("{:.0}%", valor * 100.0)
    };
    row![
        text(titulo).size(10).color(COR_TEXTO_MUTED())
            .width(Length::Fixed(63.0)),
        button(icone_menos(12.0, "#5f6368"))
            .padding([3, 4]).style(estilo_botao_pequeno(false))
            .on_press_maybe((valor > min + 0.0001).then_some(diminuir)),
        text(numero).size(10).color(COR_TEXTO())
            .width(Length::Fixed(37.0)),
        button(icone_mais(12.0, "#5f6368"))
            .padding([3, 4]).style(estilo_botao_pequeno(false))
            .on_press_maybe((valor < max - 0.0001).then_some(aumentar)),
    ].spacing(2).align(Align::Center).into()
}

fn imagem_grupo_opacidade<'a>(c: &Contexto) -> Element<'a, Msg> {
    grupo(
        column![
            text("Opacidade").size(10).color(COR_TEXTO_MUTED()),
            row![
                pelicano::text_input("0–100", c.imagem_opacidade_texto.clone())
                    .on_input(Msg::ImagemOpacidadeTexto)
                    .on_submit(Msg::ImagemOpacidadeAplicar)
                    .size(12).padding([5, 5]).width(Length::Fixed(53.0)),
                text("%").size(11).color(COR_TEXTO_MUTED()),
                dica(
                    button(icone_check(13.0, "#5f6368"))
                        .padding([4, 5]).style(estilo_botao_pequeno(false))
                        .on_press(Msg::ImagemOpacidadeAplicar),
                    "Aplicar porcentagem (também funciona com Enter)",
                ),
            ].spacing(2).align(Align::Center),
        ].spacing(8).into(),
        "Transparência",
    )
}

fn imagem_grupo_contorno<'a>(c: &Contexto) -> Element<'a, Msg> {
    let tem_cor = !c.contorno.trim().is_empty();
    let [r,g,b,_] = crate::seletor_cor::rgba_de(&c.contorno);
    let amostra = container(Space::new(Length::Fixed(18.0),Length::Fixed(14.0)))
        .style(move |_| Style {
            background: Some(Background::Color(if tem_cor { Color::rgb8(r,g,b) } else { COR_PAINEL() })),
            border: Border { color: COR_BORDA(), width: 1.0, radius: [3.0; 4] },
            ..Default::default()
        });
    let cor = crate::medido::marcar(
        "cor_contorno",
        dica(
            button(row![
                amostra,
                icone_seta_baixo(10.0, "#5f6368")
            ].spacing(4).align(Align::Center))
                .padding([4, 6]).style(estilo_botao_pequeno(false))
                .on_press(Msg::AbrirSeletorCor(crate::faixa::AlvoCor::Contorno)),
            "Escolher a cor ou remover o contorno",
        )
    );
    grupo(
        column![
            text("Cor · Espessura").size(10).color(COR_TEXTO_MUTED()),
            row![
                cor,
                dica(
                    button(icone_menos(12.0, "#5f6368")).padding([4, 4])
                        .style(estilo_botao_pequeno(false))
                        .on_press(Msg::EspessuraContorno((c.espessura - 0.5).max(0.0))),
                    "Diminuir espessura",
                ),
                text(format!("{:.1}", c.espessura)).size(10),
                dica(
                    button(icone_mais(12.0, "#5f6368")).padding([4, 4])
                        .style(estilo_botao_pequeno(false))
                        .on_press(Msg::EspessuraContorno((c.espessura + 0.5).min(50.0))),
                    "Aumentar espessura",
                ),
            ].spacing(3).align(Align::Center),
        ].spacing(8).into(),
        "Contorno",
    )
}

fn imagem_grupo_fundo<'a>(c: &Contexto) -> Element<'a, Msg> {
    const MODOS: [&str; 6] = [
        "Cor para alfa", "Contíguo", "Ignorar brilho (LAB)",
        "Watershed", "Por marcas", "Cor exata",
    ];
    let modo = MODOS[(c.fundo_modo as usize).min(5)];
    let tem_fundo = c.fundo_removido > 0.001;
    let tem_cor = !c.fundo_cor.is_empty();
    let [r,g,b,_] = crate::seletor_cor::rgba_de(&c.fundo_cor);
    let amostra = container(Space::new(Length::Fixed(17.0),Length::Fixed(15.0)))
        .style(move |_| Style {
            background: Some(Background::Color(if tem_cor { Color::rgb8(r,g,b) } else { COR_PAINEL() })),
            border: Border { color: COR_BORDA(), width: 1.0, radius: [3.0; 4] },
            ..Default::default()
        });
    let cor = crate::medido::marcar("cor_fundo_imagem", dica(
        button(row![amostra, icone_seta_baixo(10.0, "#5f6368")].spacing(4).align(Align::Center))
            .padding([4, 5]).style(estilo_botao_pequeno(false))
            .on_press(Msg::AbrirSeletorCor(crate::faixa::AlvoCor::Fundo)),
        "Escolher a cor removida do fundo",
    ));
    let alternar = dica(
        button(row![
            icone_cortar_figura(15.0, "#0b8043"),
            text(if tem_fundo { "Restaurar fundo" } else { "Remover fundo" }).size(11),
        ].spacing(4).align(Align::Center))
            .padding([5, 7]).style(estilo_botao_pequeno(tem_fundo))
            .on_press(Msg::ImagemRemoverFundo(if tem_fundo { 0.0 } else { 0.25 })),
        "A transparência é calculada sem modificar a foto original",
    );
    let menu = pick_list(&MODOS, Some(modo), move |m| {
        let i = MODOS.iter().position(|x| *x == m).unwrap_or(0);
        Msg::ImagemModoFundo(i as u8)
    })
    .size(10).padding([3, 5]).width(Length::Fixed(146.0)).style(estilo_menu_da_faixa).handle_color(COR_AZUL_ACENTO());
    let expansor = dica(
        button(row![
            if c.imagem_fundo_detalhes {
                Element::from(icone_menos(12.0, "#5f6368"))
            } else {
                Element::from(icone_mais(12.0, "#5f6368"))
            },
            text("Ajustes").size(10),
        ].spacing(3).align(Align::Center))
            .padding([4, 6]).style(estilo_botao_pequeno(c.imagem_fundo_detalhes))
            .on_press(Msg::ImagemFundoDetalhes(!c.imagem_fundo_detalhes)),
        "Exibir tolerância, suavidade e bordas",
    );
    let avancado: Element<'a, Msg> = if c.imagem_fundo_detalhes {
        let porcentagem = |nome: &'static str, atual: f32, passo: f32, suave: bool| -> Element<'a, Msg> {
            let msg = |v| if suave { Msg::ImagemSuavidadeFundo(v) } else { Msg::ImagemRemoverFundo(v) };
            imagem_ajuste_curto(nome, atual, 0.0, 1.0, passo,
                msg((atual-passo).max(0.0)),
                msg((atual+passo).min(1.0)), false)
        };
        column![
            row![
                porcentagem("Tolerância", c.fundo_removido, 0.05, false),
                porcentagem("Suavidade", c.fundo_suave, 0.05, true),
            ].spacing(9),
            row![
                imagem_ajuste_curto(
                    "Borda px", c.fundo_borda, -30.0, 30.0, 1.0,
                    Msg::ImagemAjusteBorda((c.fundo_borda-1.0).max(-30.0), c.fundo_pena),
                    Msg::ImagemAjusteBorda((c.fundo_borda+1.0).min(30.0), c.fundo_pena), true
                ),
                imagem_ajuste_curto(
                    "Pena px", c.fundo_pena, 0.0, 30.0, 1.0,
                    Msg::ImagemAjusteBorda(c.fundo_borda, (c.fundo_pena-1.0).max(0.0)),
                    Msg::ImagemAjusteBorda(c.fundo_borda, (c.fundo_pena+1.0).min(30.0)), true
                ),
            ].spacing(9),
        ].spacing(3).into()
    } else { Space::new(0.0, 0).into() };
    let marcas: Element<'a, Msg> = if matches!(c.fundo_modo, 3 | 4) {
        row![
            button(text(format!("Manter {}", c.marcas.0)).size(10))
                .padding([3, 5]).style(estilo_botao_pequeno(c.marcando == Some(true)))
                .on_press(Msg::ImagemMarcar(true)),
            button(text(format!("Remover {}", c.marcas.1)).size(10))
                .padding([3, 5]).style(estilo_botao_pequeno(c.marcando == Some(false)))
                .on_press(Msg::ImagemMarcar(false)),
            dica(
                button(icone_limpar(13.0, "#5f6368"))
                    .padding([3, 5]).style(estilo_botao_pequeno(false))
                    .on_press(Msg::ImagemLimparMarcas),
                "Limpar marcas do objeto e do fundo",
            ),
        ].spacing(4).align(Align::Center).into()
    } else { Space::new(0.0, 0).into() };

    grupo(
        column![
            row![
                alternar,
                cor,
                dica(
                    button(icone_conta_gotas(14.0, "#5f6368"))
                        .padding([5, 6]).style(estilo_botao_pequeno(c.pegando_cor))
                        .on_press(Msg::ImagemPegarCor),
                    "Escolher cor com o conta-gotas no slide",
                ),
                dica(
                    button(text("Auto").size(10)).padding([5, 5])
                        .style(estilo_botao_pequeno(!tem_cor))
                        .on_press(Msg::ImagemCorDoFundo(String::new())),
                    "Usar cor detectada nos cantos da imagem",
                ),
            ].spacing(4).align(Align::Center),
            row![
                menu,
                expansor,
                dica(
                    pelicano::checkbox("Refinar", c.fundo_refino)
                        .on_toggle(Msg::ImagemRefinarBorda)
                        .size(13),
                    "Suavizar a transição das bordas da máscara",
                ),
            ].spacing(5).align(Align::Center),
            avancado,
            marcas,
        ].spacing(5).into(),
        "Fundo",
    )
}

fn imagem_grupo_cor<'a>(c: &Contexto) -> Element<'a, Msg> {
    const CAMPOS: [&str;6] = [
        "Brilho", "Contraste", "Nitidez", "Saturação", "Temperatura", "Matiz",
    ];
    let (campo, valor, min, max, passo, graus) = match c.imagem_ajuste_selecionado {
        "Contraste" => ("contraste", c.visual_imagem.contraste, -1.0, 1.0, 0.05, false),
        "Nitidez" => ("nitidez", c.visual_imagem.nitidez, 0.0, 1.0, 0.05, false),
        "Saturação" => ("saturacao", c.visual_imagem.saturacao, 0.0, 2.0, 0.05, false),
        "Temperatura" => ("temperatura", c.visual_imagem.temperatura, -1.0, 1.0, 0.05, false),
        "Matiz" => ("matiz", c.visual_imagem.matiz, -180.0, 180.0, 5.0, true),
        _ => ("brilho", c.visual_imagem.brilho, -1.0, 1.0, 0.05, false),
    };
    let selecionado = if CAMPOS.contains(&c.imagem_ajuste_selecionado) {
        c.imagem_ajuste_selecionado
    } else { "Brilho" };
    let lista = feather_ui::componentes::lista_tipada(
        "Ajuste da imagem", &CAMPOS, Some(selecionado),
        Msg::ImagemAjusteSelecionado, 147.0,
    );
    grupo(
        column![
            lista,
            imagem_ajuste_curto(
                "Ajustar", valor, min, max, passo,
                Msg::ImagemVisual(campo, (valor-passo).max(min)),
                Msg::ImagemVisual(campo, (valor+passo).min(max)), graus,
            ),
            dica(
                button(row![
                    icone_limpar(13.0, "#5f6368"),
                    text("Redefinir cores").size(10),
                ].spacing(5).align(Align::Center))
                    .padding([3, 5]).style(estilo_botao_pequeno(false))
                    .on_press(Msg::ImagemResetarVisual),
                "Repor ajustes cromáticos, filtros e efeitos ao padrão",
            ),
        ].spacing(7).into(),
        "Cor",
    )
}

fn imagem_grupo_efeitos<'a>(c: &Contexto) -> Element<'a, Msg> {
    const FILTROS: [&str;6] = [
        "Original", "Cinza", "Sépia", "Negativo", "Posterizar", "Solarizar",
    ];
    const EFEITOS: [&str;3] = ["Reflexo", "Brilho externo", "Perspectiva"];
    let filtro = FILTROS[c.visual_imagem.filtro.min(5) as usize];
    let (campo, valor, min, max, passo) = match c.imagem_efeito_selecionado {
        "Brilho externo" => ("brilho_externo", c.visual_imagem.brilho_externo, 0.0, 1.0, 0.1),
        "Perspectiva" => ("perspectiva", c.visual_imagem.perspectiva, -1.0, 1.0, 0.1),
        _ => ("reflexo", c.visual_imagem.reflexo, 0.0, 1.0, 0.1),
    };
    let filtro_menu = pick_list(&FILTROS, Some(filtro), move |selecionado| {
        let i = FILTROS.iter().position(|v| *v == selecionado).unwrap_or(0);
        Msg::ImagemVisual("filtro", i as f32)
    }).size(10).padding([3, 4]).width(Length::Fixed(113.0)).style(estilo_menu_da_faixa).handle_color(COR_AZUL_ACENTO());
    let efeito_menu = pick_list(
        &EFEITOS, Some(if EFEITOS.contains(&c.imagem_efeito_selecionado) {
            c.imagem_efeito_selecionado
        } else { "Reflexo" }), Msg::ImagemEfeitoSelecionado,
    ).size(10).padding([3, 4]).width(Length::Fixed(112.0)).style(estilo_menu_da_faixa).handle_color(COR_AZUL_ACENTO());
    grupo(
        column![
            row![
                filtro_menu,
                dica(button(icone_girar(14.0, "#5f6368"))
                    .padding([4, 6]).style(estilo_botao_pequeno(c.visual_imagem.inverter_horizontal))
                    .on_press(Msg::ImagemVisual("inverter_horizontal",
                        f32::from(!c.visual_imagem.inverter_horizontal))),
                    "Espelhar horizontalmente"),
                dica(button(icone_seta_cima(14.0, "#5f6368"))
                    .padding([4, 6]).style(estilo_botao_pequeno(c.visual_imagem.inverter_vertical))
                    .on_press(Msg::ImagemVisual("inverter_vertical",
                        f32::from(!c.visual_imagem.inverter_vertical))),
                    "Espelhar verticalmente"),
                dica(button(icone_ajustar(14.0, "#5f6368"))
                    .padding([4, 6]).style(estilo_botao_pequeno(c.imagem_sombra))
                    .on_press(Msg::ImagemSombra(!c.imagem_sombra)),
                    "Ativar ou remover sombra"),
            ].spacing(4).align(Align::Center),
            row![
                efeito_menu,
                button(icone_menos(12.0, "#5f6368"))
                    .padding([3, 4]).style(estilo_botao_pequeno(false))
                    .on_press_maybe((valor > min + 0.0001)
                        .then_some(Msg::ImagemVisual(campo, (valor-passo).max(min)))),
                text(format!("{:.0}%", valor*100.0)).size(10)
                    .width(Length::Fixed(36.0)),
                button(icone_mais(12.0, "#5f6368"))
                    .padding([3, 4]).style(estilo_botao_pequeno(false))
                    .on_press_maybe((valor < max - 0.0001)
                        .then_some(Msg::ImagemVisual(campo, (valor+passo).min(max)))),
            ].spacing(3).align(Align::Center),
        ].spacing(9).into(),
        "Efeitos",
    )
}

fn imagem_grupo_arquivo<'a>(c: &Contexto) -> Element<'a, Msg> {
    const REDUZIR: [&str;3] = ["Resolução", "1280 px", "800 px"];
    const COMPACTAR: [&str;3] = ["Compactar", "JPEG 85", "JPEG 60"];
    let red = pick_list(&REDUZIR, Some("Resolução"), |v| {
        Msg::ImagemReduzir(if v == "800 px" {800} else {1280})
    }).size(10).padding([3, 5]).width(Length::Fixed(105.0)).style(estilo_menu_da_faixa).handle_color(COR_AZUL_ACENTO());
    let comp = pick_list(&COMPACTAR, Some("Compactar"), |v| {
        Msg::ImagemComprimir(if v == "JPEG 60" {60} else {85})
    }).size(10).padding([3, 5]).width(Length::Fixed(107.0)).style(estilo_menu_da_faixa).handle_color(COR_AZUL_ACENTO());
    let alt: Element<'a, Msg> = if c.imagem_alt_expandido {
        row![
            pelicano::text_input("Descrição acessível", c.imagem_alt_texto.clone())
                .on_input(Msg::ImagemTextoAltAlterado)
                .on_submit(Msg::ImagemTextoAltAplicar)
                .size(11).padding([4, 6]).width(Length::Fixed(174.0)),
            dica(
                button(icone_check(14.0, "#5f6368"))
                    .padding([4, 6]).style(estilo_botao_pequeno(false))
                    .on_press(Msg::ImagemTextoAltAplicar),
                "Salvar texto alternativo",
            ),
        ].spacing(4).align(Align::Center).into()
    } else { Space::new(0.0, 0).into() };
    grupo(
        column![
            row![
                dica(
                    button(row![
                        icone_substituir(14.0, "#5f6368"),
                        text("Trocar foto").size(10),
                    ].spacing(4).align(Align::Center))
                        .padding([4, 6]).style(estilo_botao_pequeno(false))
                        .on_press(Msg::ImagemSubstituir),
                    "Trocar a foto mantendo formato e efeitos",
                ),
                dica(
                    button(row![
                        text("ALT").size(10),
                        if c.imagem_alt_expandido {
                            Element::from(icone_menos(12.0, "#5f6368"))
                        } else { Element::from(icone_mais(12.0, "#5f6368")) },
                    ].spacing(4).align(Align::Center))
                        .padding([4, 5]).style(estilo_botao_pequeno(c.imagem_alt_expandido))
                        .on_press(Msg::ImagemAltExpandir(!c.imagem_alt_expandido)),
                    "Editar descrição acessível da imagem",
                ),
            ].spacing(5).align(Align::Center),
            row![red, comp].spacing(4).align(Align::Center),
            alt,
        ].spacing(7).into(),
        "Arquivo",
    )
}

/// Aba de imagens compacta: tudo acessível na faixa, sem janela modal de ajustes.
pub fn imagem<'a>(c: &Contexto) -> Element<'a, Msg> {
    if c.tipo_sel == "icone" { return formato_icone(c); }
    if c.tipo_sel == "wordart" { return formato_wordart(c); }
    if c.tipo_sel != "imagem" {
        let mut grupos = row![];
        if c.tipo_sel=="caminho" {
            grupos=grupos.push(grupo_cor("Preenchimento",&c.preenchimento,crate::faixa::AlvoCor::Preenchimento,"Preenchimento")).push(grupo_contorno_formato(c));
        }
        if c.n_selecao > 1 { grupos = grupos.push(grupo_mesclar_formas(c)); }
        if c.tipo_sel == "caminho" {
            grupos = grupos.push(grupo(column![
                pequeno(icone_duplicar(13.0, "#5f6368"), "Converter SVG", Some(Msg::ConverterSvg), false),
                pequeno(icone_salvar(13.0, "#5f6368"), "Exportar SVG…", Some(Msg::ExportarSvg), false),
            ].spacing(4).into(), "Vetores"));
        }
        return grupos.spacing(4).align(Align::Center).into();
    }

    row![
        grupo_recorte_imagem(c),
        imagem_grupo_fundo(c),
        imagem_grupo_opacidade(c),
        imagem_grupo_contorno(c),
        imagem_grupo_cor(c),
        imagem_grupo_efeitos(c),
        imagem_grupo_arquivo(c),
    ].spacing(0).align(Align::Center).into()
}

/// Ferramentas do texto artístico selecionado; mudanças ficam no histórico do motor.
fn formato_wordart<'a>(c: &Contexto) -> Element<'a, Msg> {
    use featherslide_core::wordart::Transformacao;
    let texto = grupo(column![
        pelicano::text_input("Texto do WordArt", c.wordart_texto.clone()).on_input(Msg::WordArtTexto).size(13).width(230),
        row![
            {
                const FONTES_WORDART: [&str; 5] = ["Liberation Sans", "DejaVu Sans", "Liberation Serif", "DejaVu Serif", "Liberation Mono"];
                let atual = FONTES_WORDART.iter().copied().find(|f| *f == c.fonte);
                pick_list(&FONTES_WORDART, atual, |f: &str| Msg::FonteEscolhida(f.to_string())).size(12).width(170)
            },
            pequeno(icone_menos(12.0, "#5f6368"), "", Some(Msg::DiminuirFonte), false),
            pequeno(icone_mais(12.0, "#5f6368"), "", Some(Msg::AumentarFonte), false),
        ].spacing(3),
        text(format!("{} pt", c.tamanho_fonte)).size(10),
    ].spacing(3).into(), "Texto e fonte");
    let estilos = grupo(column![
        row![pequeno(icone_wordart(14.0, "#1a73e8"), "Sólido", Some(Msg::WordArtEstilo(0)), false), pequeno(icone_wordart(14.0, "#1a73e8"), "Contornado", Some(Msg::WordArtEstilo(1)), false)].spacing(4),
        row![pequeno(icone_wordart(14.0, "#fbbc04"), "Dourado", Some(Msg::WordArtEstilo(2)), false), pequeno(icone_wordart(14.0, "#e665a0"), "Rosa", Some(Msg::WordArtEstilo(3)), false)].spacing(4),
    ].spacing(4).into(), "Estilos");
    let transformar = grupo(column![
        pick_list(&Transformacao::TODOS, Some(c.wordart_modo), move |m| Msg::WordArtTransformacao(m, 0.5)).size(12),
        row![
            pequeno(icone_menos(12.0, "#5f6368"), "", Some(Msg::WordArtTransformacao(c.wordart_modo, c.wordart_intensidade - 0.1)), false),
            text(format!("{:.0}%", c.wordart_intensidade * 100.0)).size(11),
            pequeno(icone_mais(12.0, "#5f6368"), "", Some(Msg::WordArtTransformacao(c.wordart_modo, c.wordart_intensidade + 0.1)), false),
        ].spacing(4),
        pequeno(icone_wordart(12.0, "#5f6368"), "Sombra", Some(Msg::WordArtSombra(!c.wordart_sombra)), c.wordart_sombra),
    ].spacing(4).into(), "Transformar");
    row![texto, estilos, grupo_cor("Cor do texto", &c.cor_texto, crate::faixa::AlvoCor::Texto, "Preenchimento"), grupo_contorno_formato(c), transformar].spacing(4).align(Align::Center).into()
}

/// Controles próprios dos ícones vetoriais selecionados.
fn formato_icone<'a>(c: &Contexto) -> Element<'a, Msg> {
    let cor = grupo_cor("Cor do ícone", &c.preenchimento, crate::faixa::AlvoCor::Preenchimento, "Cor do ícone");
    let traco = grupo(
        column![
            text(format!("Espessura: {:.1}", c.espessura)).size(11),
            row![
                pequeno(icone_menos(14.0, "#5f6368"), "", Some(Msg::EspessuraContorno((c.espessura - 0.25).max(0.25))), false),
                pequeno(icone_mais(14.0, "#5f6368"), "", Some(Msg::EspessuraContorno((c.espessura + 0.25).min(8.0))), false),
                pequeno(icone_voltar(14.0, "#5f6368"), "Padrão", Some(Msg::EspessuraContorno(1.0)), false),
            ].spacing(4),
        ].spacing(6).into(), "Traço",
    );
    let tamanho = grupo(column![
        pequeno(icone_zoom_menos(15.0, "#5f6368"), "Diminuir 10%", Some(Msg::EscalarSelecao(0.9)), false),
        pequeno(icone_zoom_mais(15.0, "#5f6368"), "Aumentar 10%", Some(Msg::EscalarSelecao(1.1)), false),
    ].spacing(6).into(), "Tamanho proporcional");
    let desenho = grupo(row![
        grande(icone_sorriso(22.0, "#1a73e8"), "Trocar ícone", Some(Msg::AbrirFiguras(crate::galeria_figuras::Figuras::TrocarIcone)), false),
        grande(icone_girar(22.0, "#1a73e8"), "Girar 90°", Some(Msg::GirarSelecao(90.0)), false),
    ].spacing(6).into(), "Desenho");
    row![cor, traco, tamanho, desenho].spacing(4).align(Align::Center).into()
}

/// Recorte e enquadramento empilhados. A foto é movida diretamente no palco.
pub fn grupo_recorte_imagem<'a>(c: &Contexto) -> Element<'a, Msg> {
    const BORDAS: [&str;4] = ["Esquerda", "Topo", "Direita", "Base"];
    let (borda, atual) = match c.imagem_borda_selecionada {
        "Topo" => ("topo", c.margens_imagem[1]),
        "Direita" => ("direita", c.margens_imagem[2]),
        "Base" => ("base", c.margens_imagem[3]),
        _ => ("esquerda", c.margens_imagem[0]),
    };
    let corte = dica(
        button(row![
            icone_cortar_figura(16.0, "#5f6368"),
            text("Cortar como figura").size(10),
            icone_seta_baixo(10.0, "#5f6368")
        ].spacing(4).align(Align::Center))
            .padding([4, 5]).style(estilo_botao_pequeno(false))
            .on_press(Msg::AbrirFiguras(crate::galeria_figuras::Figuras::Recorte)),
        "Aplicar recorte como círculo, retângulo ou outra forma",
    );
    let enquadrar = dica(
        button(row![
            icone_enquadramento(16.0, "#5f6368"),
            text(if c.modo_recorte { "Concluir" } else { "Enquadrar" }).size(10),
        ].spacing(4).align(Align::Center))
            .padding([4, 5]).style(estilo_botao_pequeno(c.modo_recorte))
            .on_press(Msg::AlternarModoRecorte),
        "Arraste a foto no slide para reposicionar e use a roda para zoom",
    );
    let bordas = pick_list(
        &BORDAS, Some(c.imagem_borda_selecionada),
        Msg::ImagemBordaSelecionada,
    ).size(10).padding([2, 5]).width(Length::Fixed(89.0)).style(estilo_menu_da_faixa).handle_color(COR_AZUL_ACENTO());
    grupo(
        column![
            corte,
            enquadrar,
            row![
                dica(bordas, "Cortar uma margem da imagem"),
                button(icone_menos(12.0, "#5f6368"))
                    .padding([3, 3]).style(estilo_botao_pequeno(false))
                    .on_press_maybe((atual > 0.001)
                        .then_some(Msg::ImagemCortarMargem(borda, (atual-0.05).max(0.0)))),
                text(format!("{:.0}%", atual*100.0)).size(10)
                    .width(Length::Fixed(31.0)),
                button(icone_mais(12.0, "#5f6368"))
                    .padding([3, 3]).style(estilo_botao_pequeno(false))
                    .on_press_maybe((atual < 0.45-0.001)
                        .then_some(Msg::ImagemCortarMargem(borda, (atual+0.05).min(0.45)))),
            ].spacing(2).align(Align::Center),
        ].spacing(4).into(),
        "Recorte",
    )
}

// ------------------------------------------------------------------ Design

const FUNDOS: [(&str, &str); 7] = [
    ("Branco", "#ffffff"),
    ("Cinza claro", "#f1f3f4"),
    ("Creme", "#fbf4e6"),
    ("Azul claro", "#e8f0fe"),
    ("Verde claro", "#e6f4ea"),
    ("Escuro", "#202124"),
    ("Preto", "#000000"),
];

const NOMES_DE_FUNDO: [&str; 7] = ["Branco", "Cinza claro", "Creme", "Azul claro", "Verde claro", "Escuro", "Preto"];

pub fn design<'a>(c: &Contexto) -> Element<'a, Msg> {
    let visiveis = 7usize;
    let inicio_carrossel = c.carrossel_temas.min(TEMAS.len().saturating_sub(visiveis));
    let mut galeria = row![].spacing(5).align(Align::Center);
    galeria = galeria.push(button(icone_seta_esq(14.0, "#5f6368")).padding([14, 5]).style(estilo_botao_pequeno(false)).on_press_maybe((inicio_carrossel > 0).then_some(Msg::CarrosselTemas(-(visiveis as i32) / 2))));
    for t in TEMAS.iter().skip(inicio_carrossel).take(visiveis) {
        let atual = c.tema == t.id || (c.tema.is_empty() && t.id == "office");
        let (fundo, titulo, destaque) = (cor(t.fundo), cor(t.titulo), cor(t.destaque));
        let amostra = button(
            column![
                text("Aa").size(16).color(titulo),
                container(Space::new(Length::Fixed(26.0), 0.0)).height(Length::Fixed(4.0)).style(move |_| Style {
                    background: Some(Background::Color(destaque)),
                    border: Border { radius: [2.0; 4], ..Border::default() },
                    ..Default::default()
                }),
            ]
            .align(Align::Center)
            .spacing(3),
        )
        .padding([6, 8])
        .width(Length::Fixed(54.0))
        .style(move |_, _| Style {
            background: Some(Background::Color(fundo)),
            text_color: Some(titulo),
            border: Border { color: if atual { COR_AZUL_ACENTO() } else { COR_BORDA() }, width: if atual { 2.5 } else { 1.0 }, radius: [6.0; 4] },
            ..Default::default()
        })
        .on_press(Msg::TemaAplicar(t.id.to_string()));
        galeria = galeria.push(dica(amostra, t.nome));
    }
    galeria = galeria.push(button(icone_seta_dir(14.0, "#5f6368")).padding([14, 5]).style(estilo_botao_pequeno(false)).on_press_maybe((inicio_carrossel + visiveis < TEMAS.len()).then_some(Msg::CarrosselTemas(visiveis as i32 / 2))));
    let temas = grupo(galeria.into(), "Temas");

    let mut variantes = row![].spacing(4).align(Align::Center);
    for (nome, hex) in FUNDOS {
        let ativa = c.fundo.eq_ignore_ascii_case(hex);
        let amostra = button(Space::new(Length::Fixed(18.0), Length::Fixed(18.0)))
            .padding(2)
            .style(move |_, _| Style {
                background: Some(Background::Color(cor(hex))),
                border: Border { color: if ativa { COR_AZUL_ACENTO() } else { COR_BORDA() }, width: if ativa { 2.5 } else { 1.0 }, radius: [14.0; 4] },
                ..Default::default()
            })
            .on_press(Msg::FundoSlide(hex.to_string()));
        variantes = variantes.push(dica(amostra, nome));
    }
    let variantes = grupo(variantes.into(), "Variantes");

    let tamanho = grupo(
        column![
            pequeno(icone_painel(13.0, "#5f6368"), "Widescreen 16:9", Some(Msg::Proporcao(Proporcao::Larga)), c.proporcao == Proporcao::Larga),
            pequeno(icone_normal(13.0, "#5f6368"), "Padrão 4:3", Some(Msg::Proporcao(Proporcao::Padrao)), c.proporcao == Proporcao::Padrao),
        ]
        .spacing(2)
        .into(),
        "Tamanho do slide",
    );

    let personalizar = grupo(
        row![
            menu_com_icone(icone_fundo(22.0, "#1a73e8"), "Formatar fundo", &NOMES_DE_FUNDO, |n| {
                let hex = FUNDOS.iter().find(|(nome, _)| *nome == n).map_or("#ffffff", |(_, h)| h);
                Msg::FundoSlide(hex.to_string())
            }, 100.0, false),
            column![
                pequeno(icone_duplicar(13.0, "#5f6368"), "Fundo em todos", Some(Msg::FundoTodos(if c.fundo.is_empty() { "#ffffff".into() } else { c.fundo.clone() })), false),
                pequeno(icone_cancelar(13.0, "#5f6368"), "Limpar fundo", Some(Msg::FundoSlide(String::new())), false),
            ]
            .spacing(2),
            column![
                dica(pequeno(icone_selecionar(13.0, "#5f6368"), "Editar formas do fundo", Some(Msg::FundoLiberar), false), "Solta as formas do fundo do layout (bolinhas, barras…) para selecionar, mover e apagar"),
                dica(pequeno(icone_organizar(13.0, "#5f6368"), "Enviar seleção ao fundo", c.tem_selecao.then_some(Msg::SelecaoParaFundo), false), "O elemento fica atrás de tudo e deixa de ser selecionável"),
            ]
            .spacing(2),
        ]
        .spacing(8)
        .align(Align::Center)
        .into(),
        "Personalizar",
    );
    let configurar = grupo(grande(icone_painel(22.0, "#1a73e8"), "Configurar slide…", Some(Msg::ConfiguracaoSlide), false), "Tamanho e fundo");
    row![temas, variantes, tamanho, personalizar, configurar].spacing(4).align(Align::Center).into()
}

// ------------------------------------------------------------------ Transições

pub fn transicoes<'a>(c: &Contexto) -> Element<'a, Msg> {
    let t = &c.transicao;
    let mut efeitos = row![].spacing(3).align(Align::Center);
    for tipo in TipoTransicao::TODOS {
        let icone = if tipo == TipoTransicao::Nenhuma { icone_cancelar(20.0, "#1a73e8") } else { icone_transicao(20.0, "#1a73e8") };
        efeitos = efeitos.push(pelicano::mouse_area(grande(icone, tipo.nome(), Some(Msg::TransicaoTipo(tipo)), t.tipo == tipo))
            .on_enter(Msg::PreviaTransicao(tipo)).on_exit(Msg::PreviaEncerrar));
    }
    let efeito = grupo(efeitos.into(), "Transição para este slide");
    let ligado = t.tipo.tem_direcao();
    let opcoes = grupo(
        column![
            text("Direção").size(10.5).color(COR_TEXTO_MUTED()),
            menu(t.direcao.nome(), &DIRECOES, |n| Msg::TransicaoDirecao(direcao_de(n)), 120.0, ligado),
        ]
        .spacing(2)
        .into(),
        "Opções de efeito",
    );
    let apos = if t.apos <= 0.0 { "Ao clicar".to_string() } else { segundos(t.apos) };
    let intervalo = grupo(
        row![
            passo("Duração", segundos(t.duracao), Msg::TransicaoDuracao(-0.1), Msg::TransicaoDuracao(0.1), t.tipo != TipoTransicao::Nenhuma),
            passo("Avançar após", apos, Msg::TransicaoApos(-1.0), Msg::TransicaoApos(1.0), true),
        ]
        .spacing(12)
        .into(),
        "Intervalo",
    );
    let aplicar = grupo(
        row![
            grande(icone_duplicar(20.0, "#1a73e8"), "Aplicar a todos", Some(Msg::TransicaoTodos), false),
            grande(icone_executar(20.0, "#1a73e8"), "Visualizar", Some(Msg::ApresentarDoAtual), false),
        ]
        .spacing(4)
        .into(),
        "Aplicar",
    );
    row![efeito, opcoes, intervalo, aplicar].spacing(4).align(Align::Center).into()
}

// ------------------------------------------------------------------ Animações

pub fn animacoes<'a>(c: &Contexto) -> Element<'a, Msg> {
    let ligado = c.tem_selecao;
    let a = c.animacao.clone().unwrap_or_default();
    // 19 efeitos não cabem em uma única faixa horizontal. Manter os
    // três mais frequentes como atalhos e oferecer TODOS no menu compacto.
    let mut favoritos = row![].spacing(3).align(Align::Center);
    for tipo in [TipoAnimacao::Aparecer, TipoAnimacao::Esmaecer, TipoAnimacao::Zoom] {
        let botao = grande(icone_animacao(20.0, "#1a73e8"), tipo.nome(),
            ligado.then_some(Msg::AnimTipo(tipo)), c.animacao.is_some() && a.tipo == tipo);
        favoritos = favoritos.push(if ligado {
            Element::from(pelicano::mouse_area(botao)
                .on_enter(Msg::PreviaAnimacao(tipo)).on_exit(Msg::PreviaEncerrar))
        } else { botao });
    }
    let seletor = pick_list(&TipoAnimacao::TODOS, Some(a.tipo), Msg::AnimTipo)
        .size(12).padding([5, 8]).width(Length::Fixed(170.0));
    let animacao = grupo(column![
        favoritos,
        row![
            text("Efeito").size(11).color(COR_TEXTO_MUTED()),
            seletor,
        ].spacing(6).align(Align::Center)
    ].spacing(3).into(), "Entrada e ênfase");
    let opcoes = grupo(
        column![
            text("Direção").size(10.5).color(COR_TEXTO_MUTED()),
            menu(a.direcao.nome(), &DIRECOES, |n| Msg::AnimDirecao(direcao_de(n)), 120.0, ligado && a.tipo.tem_direcao()),
        ]
        .spacing(2)
        .into(),
        "Opções de efeito",
    );
    let inicio = INICIOS.iter().copied().find(|n| *n == a.inicio.nome()).unwrap_or("Ao clicar");
    let ligado_efeito = ligado && a.tipo != TipoAnimacao::Nenhuma;
    let intervalo = grupo(
        row![
            column![
                text("Iniciar").size(10.5).color(COR_TEXTO_MUTED()),
                menu(inicio, &INICIOS, |n| Msg::AnimInicio(InicioAnimacao::TODOS.into_iter().find(|i| i.nome() == n).unwrap_or_default()), 130.0, ligado_efeito),
            ]
            .spacing(2),
            passo("Duração", segundos(a.duracao), Msg::AnimDuracao(-0.1), Msg::AnimDuracao(0.1), ligado_efeito),
            passo("Atraso", segundos(a.atraso), Msg::AnimAtraso(-0.1), Msg::AnimAtraso(0.1), ligado_efeito),
        ]
        .spacing(12)
        .into(),
        "Intervalo",
    );
    let ordem = grupo(
        column![
            pequeno(icone_subir(13.0, "#5f6368"), "Mover para antes", ligado_efeito.then_some(Msg::AnimOrdem(-1)), false),
            pequeno(icone_descer(13.0, "#5f6368"), "Mover para depois", ligado_efeito.then_some(Msg::AnimOrdem(1)), false),
        ]
        .spacing(2)
        .into(),
        "Ordem",
    );
    let avancado = grupo(
        row![
            grande(icone_lista(20.0, "#1a73e8"), "Painel de animação", Some(Msg::AbrirPainelAnimacao), false),
            grande(icone_executar(20.0, "#1a73e8"), "Visualizar", Some(Msg::ApresentarDoAtual), false),
        ]
        .spacing(4)
        .into(),
        "Avançado",
    );
    row![animacao, opcoes, intervalo, ordem, avancado].spacing(4).align(Align::Center).into()
}

// ------------------------------------------------------------------ Apresentação

pub fn apresentacao<'a>(c: &Contexto) -> Element<'a, Msg> {
    let iniciar = grupo(
        row![
            grande(icone_apresentar(24.0, "#1a73e8"), "Do começo (F5)", Some(Msg::Apresentar), false),
            grande(icone_apresentar(24.0, "#5f6368"), "Do slide atual", Some(Msg::ApresentarDoAtual), false),
        ]
        .spacing(4)
        .into(),
        "Iniciar apresentação",
    );
    let configurar = grupo(
        column![
            pequeno(icone_ocultar(13.0, "#5f6368"), "Ocultar slide", Some(Msg::OcultarSlide), c.slide_oculto),
            pequeno(icone_repetir(13.0, "#5f6368"), "Repetir até Esc", Some(Msg::Repetir), c.repetir),
            pequeno(icone_lista(13.0, "#5f6368"), "Apresentações personalizadas", Some(Msg::PersonalizadasAbrir), false),
        ]
        .spacing(2)
        .into(),
        "Configurar",
    );
    let monitores = grupo(
        row![
            grande(icone_monitor(22.0, "#5f6368"), "Modo do apresentador", Some(Msg::ApresentarComNotas), false),
        ]
        .into(),
        "Monitores",
    );
    let ensaio = grupo(
        column![
            em_breve_pequeno(icone_relogio(13.0, "#5f6368"), "Ensaiar intervalos", "medir o tempo de cada slide e gravar na transição"),
            em_breve_pequeno(icone_gravar(13.0, "#5f6368"), "Gravar apresentação", "gravar tela, voz e ponteiro"),
        ]
        .spacing(2)
        .into(),
        "Ensaio",
    );
    row![iniciar, configurar, monitores, ensaio].spacing(4).align(Align::Center).into()
}

// ------------------------------------------------------------------ Exibir

pub fn revisao<'a>(c: &Contexto) -> Element<'a, Msg> {
    let texto = grupo(
        row![
            grande(icone_ortografia(22.0, "#1a73e8"), "Ortografia", Some(Msg::OrtografiaAbrir), false),
            column![
                pequeno(icone_ortografia(13.0, "#5f6368"), "Sublinhar erros", Some(Msg::OrtografiaAuto), c.ortografia_automatica),
                pequeno(icone_ortografia(13.0, "#1a73e8"), "Sublinhar gramática", Some(Msg::GramaticaAuto), c.gramatica_automatica),
            ]
            .spacing(2),
        ]
        .spacing(6)
        .align(Align::Center)
        .into(),
        "Revisão de texto",
    );
    let comentarios = grupo(
        row![
            grande(icone_comentario(22.0, "#1a73e8"), "Comentários", Some(Msg::ComentarioAbrir), false),
            column![pequeno(icone_comentario(13.0, "#5f6368"), "Excluir todos", Some(Msg::ComentariosExcluirTodos), false)].spacing(2),
        ]
        .spacing(6)
        .align(Align::Center)
        .into(),
        "Comentários",
    );
    use crate::ferramentas_texto::Modo;
    let ferramentas = grupo(row![
        column![
            pequeno(icone_ortografia(13.0, "#5f6368"), tr("dicionario"), Some(Msg::FerramentasTexto(Modo::Dicionario)), false),
            pequeno(icone_ortografia(13.0, "#5f6368"), tr("sinonimos"), Some(Msg::FerramentasTexto(Modo::Sinonimos)), false),
            pequeno(icone_ortografia(13.0, "#5f6368"), tr("autocorrecao"), Some(Msg::FerramentasTexto(Modo::Autocorrecao)), false),
        ].spacing(2),
        column![
            pequeno(icone_ortografia(13.0, "#5f6368"), tr("idioma_curto"), Some(Msg::FerramentasTexto(Modo::Idioma)), false),
            pequeno(icone_ortografia(13.0, "#5f6368"), tr("traducao_curta"), Some(Msg::FerramentasTexto(Modo::Traducao)), false),
            pequeno(icone_ortografia(13.0, "#5f6368"), tr("contar"), Some(Msg::FerramentasTexto(Modo::Contagem)), false),
        ].spacing(2),
    ].spacing(8).into(), tr("grupo"));
    row![texto, ferramentas, comentarios].spacing(8).into()
}

pub fn exibir<'a>(c: &Contexto) -> Element<'a, Msg> {
    let modos = grupo(
        row![
            grande(icone_normal(22.0, "#1a73e8"), "Normal", Some(Msg::ExibirNormal), !c.classificacao),
            grande(icone_classificacao(22.0, "#1a73e8"), "Classificação", Some(Msg::Classificacao), c.classificacao),
            em_breve_grande(icone_apresentar(22.0, "#5f6368"), "Modo de leitura", "ler a apresentação numa janela, sem tela cheia"),
        ]
        .spacing(4)
        .into(),
        "Modos de exibição",
    );
    let mostrar = grupo(
        row![
            column![
                pequeno(icone_painel(13.0, "#5f6368"), "Painel de slides", Some(Msg::MostrarPainel), c.mostrar_painel),
                pequeno(icone_notas_painel(13.0, "#5f6368"), "Anotações", Some(Msg::MostrarNotas), c.mostrar_notas),
            ]
            .spacing(2),
            column![
                pequeno(icone_grade(13.0, "#5f6368"), "Grade", Some(Msg::Grade), c.grade),
                pequeno(icone_guias(13.0, "#5f6368"), "Guias", Some(Msg::Guias), c.guias),
            ]
            .spacing(2),
            column![pequeno(icone_regua(13.0, "#5f6368"), "Régua", Some(Msg::Regua), c.regua)].spacing(2),
        ]
        .spacing(10)
        .into(),
        "Mostrar",
    );
    let zoom = grupo(
        row![
            grande(icone_zoom_menos(20.0, "#1a73e8"), "Menos", Some(Msg::ZoomMenos), false),
            column![
                text(format!("{:.0} %", c.zoom * 100.0)).size(15),
                button(text("100 %").size(11)).padding([2, 8]).style(estilo_botao_pequeno(false)).on_press(Msg::ZoomReset),
            ]
            .align(Align::Center)
            .spacing(4),
            grande(icone_zoom_mais(20.0, "#1a73e8"), "Mais", Some(Msg::ZoomMais), false),
            grande(icone_ajustar(20.0, "#1a73e8"), "Ajustar", Some(Msg::ZoomReset), false),
        ]
        .spacing(4)
        .align(Align::Center)
        .into(),
        "Zoom",
    );
    row![modos, mostrar, zoom].spacing(4).align(Align::Center).into()
}
