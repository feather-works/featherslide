//! O aplicativo principal FeatherSlide no Iced: coordena o motor em Rust,
//! a faixa de opções (Ribbon), o palco de desenho, a lista de miniaturas e as anotações.

use std::path::PathBuf;

use featherslide_core::modelo::Tipo;
use featherslide_core::quadro::{Dicas, Quadro};
use featherslide_core::sessao::Sessao;
use featherslide_core::Formatacao;
use feather_tables::Navegacao;
use featherslide_io::{base64, fsdz};
use iced::border::Radius;
use iced::keyboard;
use iced::window;
use iced::widget::{
    button, canvas, column, container, row, text, text_input, Space,
};
use iced::{Alignment, Background, Border, Color, Element, Event, Length, Point, Subscription, Task};

use crate::ajuda;
use crate::apresentacao::{self, Acao, Programa, Reproducao};
use crate::configuracao::{combo_de, Configuracao};
use crate::configuracoes;
use crate::dados_janela;
use crate::menu_contexto::{self, MenuCtx, TipoMenu};
use crate::miniaturas::{Miniatura, ID_RENOMEAR};
use crate::monitor;
use crate::painel_animacao;
use crate::propriedades;
use crate::reutilizar_slides;
use crate::personalizadas;
use crate::recentes::{self, Recente};
use crate::registro;
use featherslide_core::cursor::{self, Disposicao, Movimento};
use crate::faixa::{view_faixa, AbaDaFaixa, FaixaMensagem};
use crate::icones::*;
use crate::miniaturas::{view_miniaturas, InfoMiniatura, MiniaturaMensagem};
use crate::palco::{PalcoMensagem, PalcoPrograma};
use crate::tema::*;

pub struct FeatherSlideApp {
    pub sessao: Sessao,
    pub arquivos: crate::arquivos::Estado,
    pub caminho_arquivo: Option<PathBuf>,
    pub nome_arquivo: String,
    pub aba_faixa: AbaDaFaixa,
    /// Ctrl+F1 recolhe só a apresentação da Ribbon, sem alterar dados do slide.
    pub faixa_recolhida: bool,
    /// Conta-gotas do "remover fundo" ligado: o próximo clique no palco escolhe a cor.
    pub pegando_cor: bool,
    pub desenho_livre: bool,
    pub conector_modo: Option<&'static str>,
    pub conector_origem: Option<u64>,
    pub zoom: f32,
    pub em_apresentacao: bool,
    /// Onde a apresentação está (passo de animação, transição, tela preta…).
    pub repro: Reproducao,
    pub previa: Option<crate::previa::Previa>,
    pub em_tela_inicio: bool,
    pub texto_edicao: String,
    /// Onde está a barrinha de escrita dentro de `texto_edicao` (em bytes; o motor garante o limite de letra).
    pub cursor_edicao: usize,
    /// A outra ponta da seleção de texto (o cursor é a ponta que se move); `None` = sem seleção.
    pub ancora_edicao: Option<usize>,
    pub editando_id: Option<u64>,
    pub status: String,
    pub ajuda: ajuda::Estado,
    pub monitor: monitor::Estado,
    pub config_ui: configuracoes::Estado,
    /// Rascunho editável das propriedades, sem modificar o documento até confirmar.
    pub propriedades: Option<propriedades::Estado>,
    pub configuracao_slide: Option<crate::configuracao_slide::Estado>,
    pub config: Configuracao,
    /// Segundos com alteração pendente, para o salvamento automático.
    pub auto_inicio: std::time::Instant,
    pub auto_versao: Option<u64>,
    pub rascunho: PathBuf,
    /// O que se digita no campo de tamanho da fonte, e se o usuário está digitando (senão mostra o do elemento).
    pub tamanho_txt: String,
    pub tamanho_editando: bool,
    /// A célula de tabela em edição: (elemento, linha, coluna).
    pub editando_celula: Option<(u64, usize, usize)>,
    /// O gráfico cujos dados a janela "Dados do gráfico" mostra.
    pub dados_id: Option<u64>,
    /// As apresentações recentes da página inicial (carregadas em segundo plano).
    pub recentes: Vec<Recente>,
    /// O menu do botão direito aberto, se houver.
    pub menu: Option<MenuCtx>,
    /// O quadro mais recente de cada slide (pelo id), para as miniaturas não refazerem o layout a cada tela.
    pub cache_quadros: std::cell::RefCell<std::collections::HashMap<u64, Quadro>>,
    /// As seções recolhidas no painel de slides (pelo índice do slide que abre cada uma) e a que está sendo renomeada.
    pub secoes_recolhidas: std::collections::HashSet<usize>,
    /// Os slides marcados no painel lateral (Ctrl/Shift+clique); o atual sempre conta.
    pub slides_sel: std::collections::BTreeSet<usize>,
    /// O teclado (Ctrl+C/X/V/A) age sobre miniaturas enquanto elas têm o foco.
    pub foco_miniaturas: bool,
    /// Cópia interna de slides, mantida mesmo ao trocar de documento aberto.
    pub slides_copiados: Option<featherslide_core::SlidesCopiados>,
    /// Número do slide em digitação, quando o diálogo Ir para slide está aberto.
    pub ir_numero: Option<String>,
    /// Fonte temporária da importação seletiva de slides.
    pub reutilizacao: Option<reutilizar_slides::Estado>,
    /// Editor de sequências personalizadas (somente estado da interface).
    pub personalizadas: Option<personalizadas::Estado>,
    /// IDs da sequência em reprodução, sem depender dos índices físicos atuais.
    pub roteiro_personalizado: Option<Vec<u64>>,
    /// A miniatura apertada (pode virar clique, reordenação ou gatilho no slide).
    pub arrastando_slide: Option<usize>,
    /// O cabeçalho da seção que está sendo arrastada (slide inicial).
    pub arrastando_secao: Option<usize>,
    /// Onde o mouse foi pressionado sobre o cabeçalho, para distinguir clique de arrasto.
    pub inicio_arrasto_secao: Option<Point>,
    /// Posição do cursor enquanto arrasta um slide ou seção (prévia flutuante).
    pub cursor_arrasto: Option<Point>,
    pub painel_simbolos: bool,
    /// Link ou Comentário aberto por cima do editor (o texto digitado).
    pub revisao: Option<Revisao>,
    /// O painel de ortografia, quando aberto.
    pub ortografia: Option<crate::ortografia::Ortografia>,
    pub formato_forma: Option<crate::formato_forma::Estado>,
    pub ferramentas_texto: Option<crate::ferramentas_texto::Estado>,
    pub editor_diagrama: Option<crate::editor_diagrama::Estado>,
    pub pincel: Option<featherslide_core::ferramentas_texto::Pincel>,
    /// O corretor, guardado depois de carregado (o dicionário leva quase 1 s para abrir).
    pub corretor: Option<featherslide_ortografia::Corretor>,
    /// Já pediu para carregar o dicionário (não pede de novo).
    pub corretor_carregando: bool,
    /// A caixa de impressão (Ctrl+P), quando aberta.
    pub impressao: Option<crate::imprimir::Impressao>,
    /// O PDF que está sendo salvo pelo "Imprimir → Salvar como PDF" leva só o slide atual?
    pub pdf_so_atual: bool,
    /// A palavra errada sob o botão direito, à espera da escolha no menu.
    pub erro_do_menu: Option<featherslide_ortografia::Erro>,
    /// A chave do aviso de gramática sob o botão direito (para "Ignorar este aviso").
    pub chave_gramatica_do_menu: Option<String>,
    pub n_monitores: usize,
    /// A janela da plateia (segundo monitor) e o início da apresentação, para o cronômetro.
    pub plateia: Option<window::Id>,
    pub cronometro: crate::cronometro::Cronometro,
    pub modo_apresentador: bool,
    pub monitores_apresentacao: Vec<crate::monitores::Monitor>,
    pub monitor_plateia: String,
    pub ultimo_clique_barra: Option<std::time::Instant>,
    /// O texto da caixa Pesquisar da barra de título e o resultado ("2 de 5").
    pub pesquisa: String,
    pub pesquisa_resultado: String,
    /// O painel de seleção (lista dos itens do slide) à direita.
    pub painel_selecao: bool,
    /// A tela de início mostra todos os modelos em grade (em vez do carrossel).
    pub ver_todos_modelos: bool,
    pub carrossel_temas: usize,
    pub galeria_figuras: Option<crate::galeria_figuras::Figuras>,
    /// O que a pessoa digitou na busca de ícones.
    pub busca_icones: String,
    /// O seletor de cor aberto (preenchimento ou contorno) e o código hexadecimal em edição.
    pub seletor_cor: Option<crate::faixa::AlvoCor>,
    /// O menu Aa/Espaçamento aberto embaixo do botão.
    pub menu_fonte: Option<crate::menu_fonte::MenuFonte>,
    /// x²/x₂ ligado sem trecho marcado: (posição do cursor, nível) para o que for
    /// digitado ali. Mover o cursor desliga, porque a posição deixa de bater.
    pub indice_digitacao: Option<(usize, i8)>,
    /// Passo do modo de captura (0 = preparar, 1 = fotografar, 2 = feito).
    pub passo_captura: u8,
    pub texto_cor: String,
    /// Matiz (graus), saturação, brilho e transparência (0 a 1) do seletor, guardados à parte
    /// para o matiz não se perder quando a cor passa por cinza ou preto.
    pub hsva_cor: [f32; 4],
    /// Conta-gotas ligado: o próximo clique no palco escolhe a cor deste alvo.
    pub conta_gotas: Option<crate::faixa::AlvoCor>,
    /// Marcando o watershed: `Some(true)` = o que manter, `Some(false)` = o que remover.
    pub marcando: Option<bool>,
    /// O elemento com ajuste de enquadramento ativo no palco.
    pub ajustando_recorte: Option<u64>,
    pub painel_imagem: bool,
    pub pagina_imagem: crate::imagem_painel::Pagina,
    pub imagem_alt_rascunho: String,
    /// Rascunhos e grupos compactos da aba de imagem; não são dados do .fsdz.
    pub imagem_alt_rascunho_id: Option<u64>,
    pub imagem_opacidade_rascunho: Option<(u64, String)>,
    pub imagem_ajuste_selecionado: &'static str,
    pub imagem_efeito_selecionado: &'static str,
    pub imagem_borda_selecionada: &'static str,
    pub imagem_fundo_detalhes: bool,
    pub imagem_alt_expandido: bool,
    /// A ponta da forma que o botão Arredondar mexe (`None` = todas).
    pub canto_sel: Option<usize>,
    /// Os modelos do usuário (pasta de modelos) e a posição do carrossel da tela de início.
    pub meus_modelos: Vec<crate::modelos_usuario::MeuModelo>,
    pub carrossel: usize,
    /// O deslocamento do slide na tela de edição (zoom), em pixels.
    pub pan: (f32, f32),
    /// A galeria de layouts aberta: `Some(true)` = novo slide, `Some(false)` = trocar o layout.
    pub galeria_layouts: Option<bool>,
    /// Os modelos da tela de início, cada um com o quadro do primeiro slide (para o cartão).
    pub modelos: Vec<(&'static featherslide_core::modelos::Modelo, Option<Quadro>)>,
    /// Voltou ao início com um documento aberto: dá para continuar editando.
    pub pode_continuar: bool,
    pub modificadores: keyboard::Modifiers,
    pub renomeando_secao: Option<(usize, String)>,
    /// O tamanho da janela principal (para o menu não abrir para fora).
    pub tamanho_janela: iced::Size,
    /// Repetir a apresentação até Esc; grade e guias no palco; e a exibição em classificação de slides.
    pub repetir: bool,
    pub grade: bool,
    pub guias: bool,
    pub regua: bool,
    pub classificacao: bool,
    /// Janela principal e auxiliares geridas pelo mesmo template dos demais aplicativos.
    pub janelas: feather_ui::janelas::Janelas<Janela>,
}

/// As janelas do sistema além da principal: cada uma é aberta e fechada pelo mesmo atalho/botão.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Janela {
    Monitor,
    Ajuda,
    Config,
    Dados,
    Animacoes,
    /// A tela da plateia na apresentação com dois monitores.
    Plateia,
}

/// Uma ação acabou de mudar o estado: o próximo desenho da tela é medido e vai para as Métricas.
static ACAO_PENDENTE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// `Palco(MenuContexto { .. })` → `Palco::MenuContexto`; `Tique` → `Tique`.
fn nome_da_acao(depurado: &str) -> String {
    let ident = |s: &str| s.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect::<String>();
    let primeiro = ident(depurado);
    let resto = &depurado[primeiro.len()..];
    match resto.strip_prefix('(') {
        Some(dentro) => {
            let segundo = ident(dentro);
            if segundo.chars().next().is_some_and(char::is_uppercase) && !matches!(segundo.as_str(), "Some" | "None") {
                format!("{primeiro}::{segundo}")
            } else {
                primeiro
            }
        }
        None => primeiro,
    }
}

/// A altura da barra de título (e dos botões de janela dentro dela).
const BARRA_ALTURA: f32 = 42.0;

/// A caixa de Inserir → Link ou Inserir → Comentário, com o texto em digitação.
#[derive(Debug, Clone)]
pub enum Revisao {
    Link(String),
    Comentario(String),
}

#[derive(Debug, Clone)]
pub enum Mensagem {
    Arquivos(crate::arquivos::Acao),
    Faixa(FaixaMensagem),
    Revisao(crate::revisao::RevisaoMsg),
    Ortografia(crate::ortografia::OrtMsg),
    FormatoForma(crate::formato_forma::Msg),
    FerramentasTexto(crate::ferramentas_texto::Msg),
    EditorDiagrama(crate::editor_diagrama::Msg),
    ColarTextoSimples(Option<String>),
    /// Modo de captura (`FEATHERSLIDE_CAPTURAR=arquivo.png`): prepara a tela e fotografa.
    CapturarTela,
    TelaCapturada(window::Screenshot),
    ColarOriginalDoSistema(Option<String>),
    Imprimir(crate::imprimir::ImprMsg),
    /// O que veio da área de transferência para colar na caixa em edição.
    ColarNoTexto(Option<String>),
    /// O PDF foi para a impressora (ou falhou).
    ImpressaoResultado(Result<String, String>),
    /// O dicionário que a pessoa escolheu para instalar.
    IdiomaEscolhido(Option<PathBuf>),
    /// O dicionário terminou de carregar (ou falhou).
    OrtografiaPronta(std::sync::Arc<std::sync::Mutex<Option<Result<featherslide_ortografia::Corretor, String>>>>),
    Miniatura(MiniaturaMensagem),
    NumeroSlideTexto(String),
    NumeroSlideConfirmar,
    NumeroSlideFechar,
    ReutilizarArquivoEscolhido(Option<PathBuf>),
    ReutilizarAlternar(usize),
    ReutilizarTodos,
    ReutilizarFechar,
    ReutilizarConfirmar(featherslide_core::EstiloColagem),
    PersonalizadasFechar,
    PersonalizadaNome(String),
    PersonalizadaCriar,
    PersonalizadaSelecionar(usize),
    PersonalizadaAdicionar,
    PersonalizadaExcluir,
    PersonalizadaMover(usize, i32),
    PersonalizadaOcultar(usize),
    PersonalizadaRemover(usize),
    PersonalizadaIniciar,
    Palco(PalcoMensagem),
    // Navegação entre telas
    NovaApresentacao,
    /// Uma apresentação nova a partir do modelo (id em `modelos.rs`).
    NovoDoModelo(&'static str),
    /// Da edição de volta à tela de início.
    IrParaInicio,
    ContinuarEditando,
    /// Move o carrossel de modelos (-1 volta, 1 avança).
    Carrossel(i32),
    /// Alterna entre o carrossel e a grade com todos os modelos.
    VerTodosModelos,
    /// Arrastar a janela pela barra de título própria.
    ArrastarJanela(window::Id),
    Minimizar(window::Id),
    AlternarMaximizar(window::Id),
    /// Fecha a janela (a principal encerra o programa).
    FecharJanela(window::Id),
    /// Digitou na caixa Pesquisar (procura o texto nos slides).
    Pesquisa(String),
    /// Enter na caixa Pesquisar: vai ao próximo resultado.
    PesquisaProximo,
    /// Uma apresentação nova a partir de um modelo do usuário.
    NovoDeMeuModelo(std::path::PathBuf),
    ModeloImportarEscolhido(Option<std::path::PathBuf>),
    ModeloExportarEscolhido(std::path::PathBuf, Option<std::path::PathBuf>),
    AbrirArquivo,
    Configuracoes,
    AlternarFaixa,
    AjudaMsg(ajuda::Msg),
    MonitorMsg(monitor::Msg),
    ConfigMsg(configuracoes::Msg),
    Propriedades(propriedades::Msg),
    ConfiguracaoSlide(crate::configuracao_slide::Msg),
    FundoImagemLida(Option<String>),
    DadosMsg(dados_janela::Msg),
    AnimacaoMsg(painel_animacao::Msg),
    Tique,
    ColarDoSistema(Option<String>),
    /// A imagem que estava na área de transferência (print de tela); sem imagem, cola texto.
    ColarImagem(Option<(String, f32, f32)>),
    // Arquivo
    RecentesCarregados(Vec<Recente>),
    /// A imagem escolhida: o `data:` URL e o tamanho natural em pixels.
    ImagemLida(Option<(String, f32, f32)>),
    SvgLido(Option<Result<featherslide_io::svg::Desenho, String>>),
    SvgExportado(Result<String, String>),
    ImagemArquivoSubstituida(u64, Option<String>),
    ImagemRecomprimida(u64, Result<String, String>),
    ArquivoEscolhidoParaAbrir(Option<PathBuf>),
    ArquivoEscolhidoParaSalvar(Option<PathBuf>),
    CopiaEscolhida(Option<PathBuf>),
    PdfEscolhido(Option<PathBuf>),
    // Edição de texto
    // Apresentação e Zoom
    Zoom(f32),
    ApresentacaoAcao(Acao),
    /// Um quadro da apresentação (transição, animação e avanço automático).
    QuadroApresentacao,
    QuadroPrevia,
    /// Um item do menu do botão direito foi escolhido.
    MenuAcao(FaixaMensagem),
    FecharMenu,
    ClassificacaoClique(usize),
    JanelaRedimensionada(window::Id, iced::Size),
    ApresentacaoProximo,
    ApresentacaoAnterior,
    ApresentacaoSair,
    CronometroAlternar,
    CronometroReiniciar,
    ApresentadorMonitor(String),
    TelaApresentacao(u8),
    // Notas
    NotasMudou(String),
    // Evento de teclado
    Teclado(Event),
    TecladoJanela(Event, window::Id),
    JanelaFechada(window::Id),
    /// Soltou o botão esquerdo em qualquer lugar: acaba o arrasto de miniatura.
    SoltouMouse,
    LimparArraste,
    /// O cursor se moveu (para a prévia flutuante acompanhar enquanto arrasta).
    CursorMoveu(Point),
}

impl Default for FeatherSlideApp {
    fn default() -> Self {
        Self {
            sessao: Sessao::default(),
            arquivos: crate::arquivos::Estado::default(),
            caminho_arquivo: None,
            nome_arquivo: "Apresentação sem título".into(),
            aba_faixa: AbaDaFaixa::Inicio,
            faixa_recolhida: false,
            pegando_cor: false,
            desenho_livre: false,
            conector_modo: None,
            conector_origem: None,
            zoom: 1.0,
            em_apresentacao: false,
            repro: Reproducao::default(),
            previa: None,
            em_tela_inicio: false,
            texto_edicao: String::new(),
            cursor_edicao: 0,
            ancora_edicao: None,
            editando_id: None,
            status: "Salvo".into(),
            ajuda: ajuda::Estado::default(),
            monitor: monitor::Estado::default(),
            config_ui: configuracoes::Estado::default(),
            propriedades: None,
            configuracao_slide: None,
            config: Configuracao::carregar(),
            auto_inicio: std::time::Instant::now(),
            auto_versao: None,
            rascunho: crate::configuracao::novo_rascunho(),
            tamanho_txt: String::new(),
            tamanho_editando: false,
            editando_celula: None,
            dados_id: None,
            recentes: Vec::new(),
            menu: None,
            cache_quadros: Default::default(),
            secoes_recolhidas: Default::default(),
            slides_sel: Default::default(),
            foco_miniaturas: false,
            slides_copiados: None,
            ir_numero: None,
            reutilizacao: None,
            personalizadas: None,
            roteiro_personalizado: None,
            arrastando_slide: None,
            arrastando_secao: None,
            inicio_arrasto_secao: None,
            cursor_arrasto: None,
            painel_simbolos: false,
            revisao: None,
            ortografia: None,
            formato_forma: None,
            ferramentas_texto: None,
            editor_diagrama: None,
            pincel: None,
            corretor: None,
            corretor_carregando: false,
            impressao: None,
            pdf_so_atual: false,
            erro_do_menu: None,
            chave_gramatica_do_menu: None,
            n_monitores: 0,
            plateia: None,
            cronometro: Default::default(),
            modo_apresentador: false,
            monitores_apresentacao: Vec::new(),
            monitor_plateia: "Janela".into(),
            ultimo_clique_barra: None,
            pesquisa: String::new(),
            pesquisa_resultado: String::new(),
            painel_selecao: false,
            ver_todos_modelos: false,
            carrossel_temas: 0,
            galeria_figuras: None,
            busca_icones: String::new(),
            seletor_cor: None,
            menu_fonte: None,
            indice_digitacao: None,
            passo_captura: 0,
            texto_cor: String::new(),
            hsva_cor: [0.0, 0.0, 0.0, 1.0],
            conta_gotas: None,
            marcando: None,
            ajustando_recorte: None,
            painel_imagem: false,
            pagina_imagem: crate::imagem_painel::Pagina::Cor,
            imagem_alt_rascunho: String::new(),
            imagem_alt_rascunho_id: None,
            imagem_opacidade_rascunho: None,
            imagem_ajuste_selecionado: "Brilho",
            imagem_efeito_selecionado: "Reflexo",
            imagem_borda_selecionada: "Esquerda",
            imagem_fundo_detalhes: false,
            imagem_alt_expandido: false,
            canto_sel: None,
            meus_modelos: Vec::new(),
            carrossel: 0,
            pan: (0.0, 0.0),
            galeria_layouts: None,
            modelos: Vec::new(),
            pode_continuar: false,
            modificadores: keyboard::Modifiers::default(),
            renomeando_secao: None,
            tamanho_janela: iced::Size::new(1280.0, 800.0),
            repetir: false,
            grade: false,
            guias: false,
            regua: false,
            classificacao: false,
            janelas: feather_ui::janelas::Janelas::default(),
        }
    }
}

impl FeatherSlideApp {
    pub fn new() -> (Self, Task<Mensagem>) {
        let config = Configuracao::carregar();
        let (id, abrir) = window::open(window::Settings { size: iced::Size::new(1280.0, 800.0), decorations: !config.barra_propria, exit_on_close_request: false, ..Default::default() });
        let mut app = Self { em_tela_inicio: true, config, ..Self::default() };
        app.janelas.principal = Some(id);
        app.arquivos.ativo = false;
        app.atualizar_arquivos();

        app.abrir_argumentos(std::env::args_os().skip(1));
        app.meus_modelos = crate::modelos_usuario::listar();
        app.n_monitores = crate::monitores::listar().len();
        app.modelos = featherslide_core::modelos::MODELOS
            .iter()
            .map(|m| (m, featherslide_core::modelos::criar(m.id).and_then(|ap| Sessao::nova(ap).quadro(0, None, false).ok().flatten())))
            .collect();
        app.aplicar_dicas();
        definir_tema(escuro_resolvido(app.config.tema_auto, app.config.escuro), app.config.acento);
        feather_ui::marca::definir_marca(feather_ui::marca::Produto::FeatherSlide,
            app.config.logo_usa_acento());
        let caminhos = recentes::caminhos(&app.config);
        let carregar = Task::perform(
            async move { tokio::task::spawn_blocking(move || recentes::carregar(caminhos)).await.unwrap_or_default() },
            Mensagem::RecentesCarregados,
        );
        (app, Task::batch([abrir.map(|_| Mensagem::Tique), carregar]))
    }

    /// As dicas das caixas vazias ("Clique para adicionar um título"), na língua da interface.
    pub(crate) fn aplicar_dicas(&mut self) {
        self.sessao.definir_dicas(Dicas {
            titulo: ajuda::t("dicas.titulo"),
            subtitulo: ajuda::t("dicas.subtitulo"),
            corpo: ajuda::t("dicas.corpo"),
        });
    }

    pub fn titulo(&self, janela: window::Id) -> String {
        match self.tipo_de(janela) {
            Some(Janela::Monitor) => "Monitor do FeatherSlide".into(),
            Some(Janela::Ajuda) => "Ajuda do FeatherSlide".into(),
            Some(Janela::Config) => "Configurações do FeatherSlide".into(),
            Some(Janela::Dados) => "Dados do gráfico".into(),
            Some(Janela::Animacoes) => "Painel de animação".into(),
            Some(Janela::Plateia) => "FeatherSlide — apresentação".into(),
            None => format!("FeatherSlide — {}", self.nome_arquivo),
        }
    }

    pub fn tema(&self, _janela: window::Id) -> iced::Theme {
        tema_iced()
    }

    fn tipo_de(&self, id: window::Id) -> Option<Janela> {
        self.janelas.tipo(id)
    }

    fn aberta(&self, j: Janela) -> bool {
        self.janelas.aberta(j)
    }

    /// Abre ou fecha pelo gerenciador comum da suíte.
    fn alternar(&mut self, j: Janela) -> Task<Mensagem> {
        let tamanho = match j {
            Janela::Monitor => iced::Size::new(980.0, 640.0),
            Janela::Ajuda => iced::Size::new(1190.0, 760.0),
            Janela::Config => iced::Size::new(900.0, 640.0),
            Janela::Dados => iced::Size::new(720.0, 520.0),
            Janela::Animacoes => iced::Size::new(520.0, 560.0),
            Janela::Plateia => iced::Size::new(1280.0, 720.0),
        };
        let ja_aberta = self.janelas.aberta(j);
        let task = self.janelas.alternar(j, tamanho, self.config.barra_propria);
        if j == Janela::Monitor && !ja_aberta {
            self.monitor.amostrar();
        }
        task.map(|_| Mensagem::Tique)
    }

    /// Mede cada mensagem e deixa `[ação] … — N ms` no registro (o Monitor, F12, mostra).
    pub fn update(&mut self, mensagem: Mensagem) -> Task<Mensagem> {
        if self.arquivos.pendente.is_some() && !matches!(&mensagem,
            Mensagem::Arquivos(crate::arquivos::Acao::Confirmar | crate::arquivos::Acao::Cancelar)
                | Mensagem::Tique | Mensagem::JanelaRedimensionada(..)) {
            return Task::none();
        }
        let nome = match &mensagem {
            Mensagem::Teclado(_) | Mensagem::TecladoJanela(..) | Mensagem::Tique | Mensagem::QuadroPrevia | Mensagem::Zoom(_) | Mensagem::SoltouMouse | Mensagem::LimparArraste | Mensagem::ArrastarJanela(_) => None,
            m => Some(nome_da_acao(&format!("{m:?}"))),
        };
        if nome.is_some() {
            ACAO_PENDENTE.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        if let Mensagem::Teclado(Event::Keyboard(keyboard::Event::ModifiersChanged(m))) | Mensagem::TecladoJanela(Event::Keyboard(keyboard::Event::ModifiersChanged(m)), _) = &mensagem {
            self.modificadores = *m;
        }
        let status_antes = self.status.clone();
        let inicio = std::time::Instant::now();
        let so_visual = matches!(&mensagem, Mensagem::Tique | Mensagem::QuadroPrevia | Mensagem::QuadroApresentacao | Mensagem::CursorMoveu(_) | Mensagem::Zoom(_));
        let protegido = (self.arquivos.ativo && self.arquivos.leitura && !so_visual).then(|| (self.rascunho.clone(), self.sessao.clone()));
        let versao_antes = self.sessao.estado().versao;
        let sair = if self.em_apresentacao && matches!(&mensagem,
            Mensagem::Arquivos(_) | Mensagem::NovaApresentacao | Mensagem::NovoDoModelo(_) | Mensagem::NovoDeMeuModelo(_)
                | Mensagem::ArquivoEscolhidoParaAbrir(Some(_))) { self.sair_da_apresentacao() } else { Task::none() };
        let selecao_forma_antes = self.formato_forma.as_ref().map(|_| self.sessao.estado().selecao.clone());
        let tarefa = Task::batch([self.tratar(mensagem), sair]);
        if selecao_forma_antes.is_some_and(|antes| antes != self.sessao.estado().selecao) && self.formato_forma.is_some() {
            self.sessao.confirmar_previa();
            self.formato_forma = self.sessao.estado().selecao.first()
                .and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == *id))
                .filter(|e| featherslide_core::estilo_forma::aceita(e.tipo))
                .map(crate::formato_forma::Estado::novo);
        }
        if let Some((id, original)) = protegido {
            if self.rascunho == id && self.sessao.apresentacao() != original.apresentacao() {
                self.sessao = original;
                self.limpar_interface_documento();
                self.status = "Apresentação somente para leitura; use Duplicar para editar uma cópia".into();
            }
        }
        if self.sessao.estado().versao != versao_antes {
            self.auto_inicio = std::time::Instant::now();
        }
        // Tudo que a barra de status disse também vai ao Console (F12); erro vai com a marca `[erro]`.
        if self.status != status_antes {
            let minusculo = self.status.to_lowercase();
            let falha = minusculo.starts_with("erro") || minusculo.contains("não ") || minusculo.contains("falta") || minusculo.contains("precisa") || minusculo.contains("fora do") || minusculo.contains("ilegível");
            registro::registrar(format!("{} {}", if falha { "[erro]" } else { "[status]" }, self.status));
        }
        if let Some(n) = nome {
            registro::registrar(format!("[ação] {n} — {:.1} ms", inicio.elapsed().as_secs_f64() * 1000.0));
        }
        // O dicionário leva quase 1 s para abrir: carrega uma vez, em segundo plano, sem travar a tela.
        if self.config.ortografia_automatica && self.corretor.is_none() && !self.corretor_carregando {
            self.corretor_carregando = true;
            return Task::batch([tarefa, Self::carregar_corretor(self.config.idiomas_ortografia.clone())]);
        }
        tarefa
    }

    /// As linhas desenhadas do elemento `id` (texto já quebrado), o tamanho da fonte e se é negrito.
    fn linhas_do_elemento(&self, id: u64) -> Option<(Vec<featherslide_core::quadro::LinhaDeTexto>, f32, bool)> {
        let q = self.sessao.quadro(self.sessao.atual(), None, true).ok().flatten()?;
        let e = q.elementos.into_iter().find(|e| e.id == id)?;
        (!e.dica).then_some((e.linhas, e.tamanho, e.negrito))
    }

    /// Onde o clique em (x, y) põe o cursor na caixa `id` (no fim do texto se não há linhas).
    fn cursor_no_clique(&self, id: u64, x: f32, y: f32) -> usize {
        match self.linhas_do_elemento(id) {
            Some((linhas, tamanho, negrito)) => cursor::cursor_no_ponto(&self.texto_edicao, &Disposicao { linhas: &linhas, tamanho, negrito }, x, y),
            None => self.texto_edicao.len(),
        }
    }

    fn carregar_corretor(idiomas: Vec<String>) -> Task<Mensagem> {
        Task::perform(
            async move {
                let r = featherslide_ortografia::Corretor::carregar(&idiomas);
                std::sync::Arc::new(std::sync::Mutex::new(Some(r)))
            },
            Mensagem::OrtografiaPronta,
        )
    }

    /// As palavras erradas do slide aberto, já posicionadas (vazio se desligado ou sem dicionário).
    fn ondulados_atuais(&self) -> Vec<featherslide_ortografia::Ondulado> {
        use featherslide_ortografia::TipoOndulado;
        let (true, Some(c)) = (self.config.ortografia_automatica || self.config.gramatica_automatica, self.corretor.as_ref()) else { return Vec::new() };
        let atual = self.sessao.atual();
        let mut v = match self.sessao.quadro(atual, None, true) {
            Ok(Some(q)) => c.ondulados(atual, self.sessao.slide(), &q, self.config.gramatica_automatica),
            _ => Vec::new(),
        };
        v.retain(|o| o.tipo == TipoOndulado::Gramatica || self.config.ortografia_automatica);
        v
    }

    fn tratar(&mut self, mensagem: Mensagem) -> Task<Mensagem> {
        match mensagem {
            Mensagem::Arquivos(acao) => return self.tratar_arquivos(acao),
            Mensagem::ConfiguracaoSlide(acao) => {
                use crate::configuracao_slide::Msg as C;
                match acao {
                    C::Fechar => self.configuracao_slide = None,
                    C::Imagem => {
                        return Task::perform(async {
                            let arquivo = rfd::AsyncFileDialog::new().add_filter("Imagem", &["png", "jpg", "jpeg", "webp", "bmp"]).pick_file().await?;
                            let caminho = arquivo.path().to_path_buf();
                            tokio::task::spawn_blocking(move || {
                                let bytes = std::fs::read(caminho).ok()?;
                                let formato = image::guess_format(&bytes).ok()?;
                                image::load_from_memory(&bytes).ok()?;
                                let mime = match formato { image::ImageFormat::Jpeg => "image/jpeg", image::ImageFormat::WebP => "image/webp", image::ImageFormat::Bmp => "image/bmp", _ => "image/png" };
                                Some(base64::para_data_url(mime, &bytes))
                            }).await.ok().flatten()
                        }, Mensagem::FundoImagemLida);
                    }
                    C::AplicarTamanho | C::AplicarFundo => {
                        if self.arquivos.leitura { self.status = "Arquivo aberto somente para leitura".into(); return Task::none(); }
                        if let Some(e) = self.configuracao_slide.clone() {
                            let resultado = if matches!(acao, C::AplicarTamanho) {
                                e.tamanho().and_then(|(w, h, ajuste)| self.sessao.tamanho_slide_definir(w, h, ajuste))
                            } else {
                                e.fundo().and_then(|f| self.sessao.fundo_configurar(&e.cor, f, e.todos, e.mestre))
                            };
                            if let Some(p) = &mut self.configuracao_slide { p.aviso = match resultado { Ok(_) => "Configuração aplicada. Ctrl+Z desfaz.".into(), Err(e) => e }; }
                        }
                    }
                    outra => if let Some(e) = &mut self.configuracao_slide { e.tratar(outra); },
                }
            }
            Mensagem::FundoImagemLida(url) => {
                if let Some(e) = &mut self.configuracao_slide {
                    if let Some(url) = url { e.fundo.imagem = url.into(); e.fundo.tipo = "imagem".into(); }
                }
            }
            Mensagem::Propriedades(acao) => {
                match acao {
                    propriedades::Msg::Fechar => self.propriedades = None,
                    propriedades::Msg::Salvar => {
                        if self.arquivos.leitura {
                            if let Some(p) = &mut self.propriedades {
                                p.aviso = "Arquivo somente para leitura: salve uma cópia antes de editar.".into();
                            }
                        } else if let Some(p) = self.propriedades.clone() {
                            match self.sessao.propriedades_definir(p.titulo, p.autor, p.dados) {
                                Ok(_) => { self.propriedades = None; self.status = "Propriedades atualizadas".into(); }
                                Err(e) => { if let Some(estado) = &mut self.propriedades { estado.aviso = e; } }
                            }
                        }
                    }
                    outra => { if let Some(p) = &mut self.propriedades { p.tratar(outra); } }
                }
                return Task::none();
            },
            Mensagem::ReutilizarArquivoEscolhido(Some(path)) => {
                match featherslide_io::documentos::ler(&path) {
                    Ok(ap) if !ap.slides.is_empty() => {
                        self.reutilizacao = Some(reutilizar_slides::Estado::novo(ap, path));
                    },
                    Ok(_) => self.status = "O arquivo não possui slides para reutilizar.".into(),
                    Err(e) => self.status = format!("Não foi possível reutilizar slides: {e}"),
                }
            }
            Mensagem::ReutilizarArquivoEscolhido(None) => {}
            Mensagem::ReutilizarAlternar(i) => {
                if let Some(estado) = &mut self.reutilizacao {
                    if i < estado.apresentacao.slides.len() && !estado.selecionados.remove(&i) {
                        estado.selecionados.insert(i);
                    }
                }
            }
            Mensagem::ReutilizarTodos => {
                if let Some(e) = &mut self.reutilizacao {
                    e.selecionados = (0..e.apresentacao.slides.len()).collect();
                }
            }
            Mensagem::ReutilizarFechar => self.reutilizacao = None,
            Mensagem::ReutilizarConfirmar(estilo) => {
                if let Some(e) = &self.reutilizacao {
                    let selecionados: Vec<_> = e.selecionados.iter().copied().collect();
                    match featherslide_core::SlidesCopiados::da_apresentacao(&e.apresentacao, &selecionados) {
                        Ok(transferencia) => {
                            let pos = self.sessao.atual() + 1;
                            match self.sessao.slides_colar(pos, &transferencia, estilo) {
                                Ok(_) => {
                                    self.status = format!("{} slides reutilizados", transferencia.quantidade());
                                    self.slides_sel = (pos..pos+transferencia.quantidade()).collect();
                                    self.reutilizacao = None;
                                },
                                Err(err) => self.status = err,
                            }
                        }
                        Err(err) => self.status = err,
                    }
                }
            }
            Mensagem::NumeroSlideTexto(t) => self.ir_numero = Some(t.chars().filter(char::is_ascii_digit).take(8).collect()),
            Mensagem::NumeroSlideFechar => self.ir_numero = None,
            Mensagem::NumeroSlideConfirmar => {
                if let Some(t) = self.ir_numero.clone() {
                    match t.parse::<usize>() {
                        Ok(n) if n > 0 && n <= self.sessao.apresentacao().slides.len() => {
                            match self.sessao.slide_ir(n - 1) {
                                Ok(()) => {
                                    self.slides_sel = [n - 1].into();
                                    self.foco_miniaturas = true;
                                    self.ir_numero = None;
                                    self.status = format!("Slide {n} de {}", self.sessao.apresentacao().slides.len());
                                },
                                Err(e) => self.status = e,
                            }
                        }
                        _ => self.status = format!("Informe um número de 1 a {}.", self.sessao.apresentacao().slides.len()),
                    }
                }
            }
            Mensagem::NovaApresentacao => {
                self.trocar_sessao(Sessao::default());
                self.aplicar_dicas();
                self.caminho_arquivo = None;
                self.nome_arquivo = "Apresentação sem título".into();
                self.em_tela_inicio = false;
                self.pode_continuar = false;
                self.status = "Novo documento criado".into();
            }
            Mensagem::NovoDoModelo(id) => {
                if let Some(ap) = featherslide_core::modelos::criar(id) {
                    self.trocar_sessao(Sessao::nova(ap));
                    self.aplicar_dicas();
                    self.caminho_arquivo = None;
                    self.nome_arquivo = "Apresentação sem título".into();
                    self.em_tela_inicio = false;
                    self.pode_continuar = false;
                    self.slides_sel.clear();
                    self.status = format!("Novo documento a partir do modelo \"{id}\"");
                }
            }
            Mensagem::IrParaInicio => {
                self.em_tela_inicio = true;
                self.pode_continuar = true;
                self.menu = None;
                self.painel_simbolos = false;
            }
            Mensagem::ContinuarEditando => self.em_tela_inicio = false,
            Mensagem::Pesquisa(t) => {
                self.pesquisa = t;
                let r = self.sessao.busca_definir(&self.pesquisa, Default::default());
                self.pesquisa_resultado = self.resultado_da_pesquisa(r.total, r.atual);
                if r.slide.is_some() {
                    self.editando_id = None;
                }
            }
            Mensagem::PesquisaProximo => {
                let r = self.sessao.busca_ir(1);
                self.pesquisa_resultado = self.resultado_da_pesquisa(r.total, r.atual);
                self.editando_id = None;
            }
            Mensagem::ArrastarJanela(id) => {
                // Dois cliques seguidos na barra maximizam ou restauram; um só começa a arrastar.
                return feather_ui::janelas::arrastar_ou_maximizar(id, &mut self.ultimo_clique_barra);
            }
            Mensagem::Minimizar(id) => return window::minimize(id, true),
            Mensagem::AlternarMaximizar(id) => return window::toggle_maximize(id),
            Mensagem::FecharJanela(id) => {
                if Some(id) == self.janelas.principal {
                    return self.tratar_arquivos(crate::arquivos::Acao::Sair);
                }
                self.janelas.retirar(id);
                return window::close(id);
            }
            Mensagem::VerTodosModelos => self.ver_todos_modelos = !self.ver_todos_modelos,
            Mensagem::Carrossel(d) => {
                let total = self.modelos.len() + self.meus_modelos.len();
                self.carrossel = (self.carrossel as i32 + d).clamp(0, total.saturating_sub(1) as i32) as usize;
            }
            Mensagem::NovoDeMeuModelo(caminho) => match crate::modelos_usuario::abrir(&caminho) {
                Ok(ap) => {
                    self.trocar_sessao(Sessao::nova(ap));
                    self.aplicar_dicas();
                    self.caminho_arquivo = None;
                    self.nome_arquivo = "Apresentação sem título".into();
                    self.em_tela_inicio = false;
                    self.pode_continuar = false;
                    self.status = "Novo documento a partir de um modelo seu".into();
                }
                Err(e) => self.status = format!("Erro ao abrir o modelo: {e}"),
            },
            Mensagem::ModeloImportarEscolhido(Some(origem)) => {
                match crate::modelos_usuario::importar(&origem) {
                    Ok(_) => self.status = "Modelo importado".into(),
                    Err(e) => self.status = format!("Erro ao importar o modelo: {e}"),
                }
                self.meus_modelos = crate::modelos_usuario::listar();
            }
            Mensagem::ModeloImportarEscolhido(None) => {}
            Mensagem::ModeloExportarEscolhido(modelo, Some(destino)) => match crate::modelos_usuario::exportar(&modelo, &destino) {
                Ok(()) => self.status = "Modelo exportado".into(),
                Err(e) => self.status = format!("Erro ao exportar o modelo: {e}"),
            },
            Mensagem::ModeloExportarEscolhido(_, None) => {}
            Mensagem::AbrirArquivo => {
                return self.abrir_dialogo_abrir();
            }
            Mensagem::Configuracoes => return self.alternar(Janela::Config),
            Mensagem::AlternarFaixa => self.faixa_recolhida = !self.faixa_recolhida,
            Mensagem::JanelaFechada(id) => {
                if Some(id) == self.janelas.principal {
                    return self.tratar_arquivos(crate::arquivos::Acao::Sair);
                }
                self.janelas.retirar(id);
                if self.plateia == Some(id) {
                    self.plateia = None;
                    return self.sair_da_apresentacao();
                }
            }
            Mensagem::TecladoJanela(ev, id) => match self.tipo_de(id) {
                None | Some(Janela::Plateia) => return self.tratar(Mensagem::Teclado(ev)),
                Some(j) => {
                    if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = &ev {
                        // Regravando um atalho: a próxima combinação vale para ele.
                        if j == Janela::Config {
                            if let Some(atalho) = self.config_ui.gravando {
                                if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
                                    self.config_ui.gravando = None;
                                    self.config_ui.aviso.clear();
                                    return Task::none();
                                }
                                if let Some(combo) = combo_de(key, *modifiers) {
                                    match self.config.comando_de(&combo) {
                                        Some(outro) if outro != atalho => {
                                            self.config_ui.aviso = format!("{combo} já é de outro atalho.");
                                        }
                                        _ => {
                                            self.config.atalhos.insert(atalho.to_string(), combo);
                                            self.config.gravar();
                                            self.config_ui.gravando = None;
                                            self.config_ui.aviso.clear();
                                        }
                                    }
                                }
                                return Task::none();
                            }
                        }
                        if j==Janela::Ajuda && modifiers.control() && !modifiers.shift() && !modifiers.alt()
                            && matches!(key, keyboard::Key::Character(k) if k.eq_ignore_ascii_case("f")) {
                            return feather_ui::ajuda::focar_busca::<Mensagem>();
                        }
                        use feather_ui::atalhos::PainelGlobal;
                        let painel = match j {
                            Janela::Monitor => Some(PainelGlobal::Monitor),
                            Janela::Ajuda => Some(PainelGlobal::Ajuda),
                            Janela::Config => Some(PainelGlobal::Configuracoes),
                            // Janelas locais preservam somente o fechamento por Esc.
                            _ => None,
                        };
                        let fecha = painel.is_some_and(|p|
                            feather_ui::atalhos::fechar_painel(p, key, *modifiers))
                            || (painel.is_none()
                                && matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)));
                        if fecha {
                            return self.alternar(j);
                        }
                    }
                }
            },
            Mensagem::AjudaMsg(m) => self.ajuda.update(m),
            Mensagem::MonitorMsg(monitor::Msg::Copiar(texto)) => return iced::clipboard::write(texto),
            Mensagem::MonitorMsg(m) => self.monitor.update(m),
            Mensagem::ConfigMsg(m) => return self.tratar_config(m),
            Mensagem::AnimacaoMsg(m) => painel_animacao::atualizar(&mut self.sessao, m),
            Mensagem::DadosMsg(m) => {
                if let Some(id) = self.dados_id {
                    dados_janela::atualizar(&mut self.sessao, id, m);
                }
            }
            Mensagem::ColarDoSistema(texto) => match self.sessao.colar(&texto.unwrap_or_default()) {
                Ok(_) => self.status = "Colado".into(),
                Err(e) => self.status = e,
            },
            Mensagem::FormatoForma(m) => {
                match m {
                    crate::formato_forma::Msg::Fechar => {self.sessao.cancelar_previa();self.formato_forma=None;},
                    crate::formato_forma::Msg::Aplicar => {
                        if let Some(e)=&mut self.formato_forma {
                            let r=e.aparencia().and_then(|a|self.sessao.forma_aparencia_previa(a));
                            e.aviso=match r{Ok(_)=>{self.sessao.confirmar_previa();"Aparência aplicada. Ctrl+Z desfaz.".into()},Err(e)=>e};
                        }
                    },
                    m=>if let Some(e)=&mut self.formato_forma {
                        let antes=e.aparencia().ok();e.tratar(m);
                        match e.aparencia(){Ok(a) if antes.as_ref()!=Some(&a)=>{e.aviso=match self.sessao.forma_aparencia_previa(a){Ok(_)=>String::new(),Err(e)=>e};},Err(erro)=>e.aviso=erro,_=>{}}
                    },
                }
            },
            Mensagem::FerramentasTexto(m) => return self.tratar_ferramentas_texto(m),
            Mensagem::EditorDiagrama(m) => {
                use crate::editor_diagrama::Msg;
                match m {
                    Msg::Fechar => self.editor_diagrama = None,
                    Msg::Selecionar(i) => if let Some(e) = &mut self.editor_diagrama { e.selecionado = e.edicao.nos.get(i).map(|n| n.id); },
                    Msg::Texto(i, s) => if let Some(e) = &mut self.editor_diagrama { if let Some(no) = e.edicao.nos.get_mut(i) { e.selecionado = Some(no.id); no.texto = s; } },
                    Msg::Adicionar(indice,filho) => if let Some(e)=&mut self.editor_diagrama {
                        let pai=indice.and_then(|i|e.edicao.nos.get(i)).and_then(|n|if filho {Some(n.id)}else{n.pai});
                        let ids: Vec<_> = e.edicao.nos.iter().map(|n| n.id).collect();
                        match e.edicao.adicionar_a(crate::localidade::texto("diagrama_novo").into(),pai) {
                            Err(aviso) => e.aviso=aviso,
                            Ok(()) => e.selecionado = e.edicao.nos.iter().find(|n| !ids.contains(&n.id)).map(|n| n.id),
                        }
                    },
                    Msg::Excluir(i) => if let Some(e)=&mut self.editor_diagrama {
                        if let Err(aviso)=e.edicao.excluir(i){e.aviso=aviso;}
                        if !e.edicao.nos.iter().any(|n| Some(n.id) == e.selecionado) {
                            e.selecionado = e.edicao.nos.get(i.min(e.edicao.nos.len().saturating_sub(1))).map(|n| n.id);
                        }
                    },
                    Msg::Mover(i,d) => if let Some(e)=&mut self.editor_diagrama { e.edicao.mover(i,d); },
                    Msg::Arranjo(a) => if let Some(e)=&mut self.editor_diagrama { e.edicao.arranjo_definir(a); },
                    Msg::Nivel(i,b) => if let Some(e)=&mut self.editor_diagrama { e.edicao.nivel_alterar(i,b); },
                    Msg::Aplicar => if let Some(e) = &mut self.editor_diagrama {
                        match self.sessao.diagrama_textos_definir(&e.edicao) {
                            Ok(_) => { self.editor_diagrama = None; self.status = "Textos do diagrama atualizados. Ctrl+Z desfaz.".into(); },
                            Err(aviso) => e.aviso = aviso,
                        }
                    },
                }
            }
            Mensagem::ColarOriginalDoSistema(texto) => {
                self.status = match self.sessao.colar_com_formatacao_original(&texto.unwrap_or_default()) {
                    Ok(_) => "Colado com formatação original".into(), Err(e) => e,
                };
            }
            Mensagem::CapturarTela => match self.passo_captura {
                0 => {
                    // `FEATHERSLIDE_CAPTURAR_TELA=inicio`: a tela Início, sem documento.
                    if std::env::var("FEATHERSLIDE_CAPTURAR_TELA").as_deref() == Ok("inicio") {
                        self.passo_captura = 1;
                        return Task::none();
                    }
                    // Uma caixa de texto selecionada, para a faixa aparecer ligada.
                    self.em_tela_inicio = false;
                    if let Ok(a) = std::env::var("FEATHERSLIDE_CAPTURAR_ABA") {
                        use crate::faixa::AbaDaFaixa as A;
                        self.aba_faixa = match a.as_str() { "inserir" => A::Inserir, "organizar" => A::Organizar, "design" => A::Design, "transicoes" => A::Transicoes, "animacoes" => A::Animacoes, "apresentacao" => A::Apresentacao, "revisao" => A::Revisao, "exibir" => A::Exibir, _ => self.aba_faixa };
                    }
                    if self.sessao.elemento_inserir(Tipo::Texto, None, None).is_ok() {
                        if let Some(id) = self.sessao.slide().elementos.last().map(|e| e.id) {
                            let _ = self.sessao.texto_definir(id, "Exemplo de texto");
                            self.sessao.selecionar_por_id(id);
                        }
                    }
                    if let Ok(m) = std::env::var("FEATHERSLIDE_CAPTURAR_MENU") {
                        use crate::menu_fonte::MenuFonte;
                        self.menu_fonte = match m.as_str() { "caixa" => Some(MenuFonte::Caixa), "espacamento" => Some(MenuFonte::Espacamento), "tracos" => Some(MenuFonte::Tracos), _ => None };
                    }
                    self.passo_captura = 1;
                }
                1 => {
                    self.passo_captura = 2;
                    if let Some(id) = self.janelas.principal {
                        return window::screenshot(id).map(Mensagem::TelaCapturada);
                    }
                }
                _ => {}
            },
            Mensagem::TelaCapturada(s) => {
                if let Some(caminho) = std::env::var_os("FEATHERSLIDE_CAPTURAR") {
                    let _ = image::save_buffer(&caminho, &s.bytes, s.size.width, s.size.height, image::ColorType::Rgba8);
                }
                return iced::exit();
            }
            Mensagem::ColarTextoSimples(texto) => {
                if let Some((id,r,c)) = self.editando_celula {
                    if let Some(texto) = texto {
                        let texto = texto.replace("\r\n", "\n").replace('\r', "\n");
                        cursor::substituir(&mut self.texto_edicao, &mut self.cursor_edicao, self.ancora_edicao.take(), &texto);
                        if let Err(e) = featherslide_tabelas::slide_editar_celula(&mut self.sessao, id,r,c,&self.texto_edicao) { self.status = e; }
                    }
                    return Task::none();
                }
                if self.editando_id.is_some() { return self.tratar(Mensagem::ColarNoTexto(texto)); }
                self.status = match self.sessao.colar_sem_formatacao(&texto.unwrap_or_default()) {
                    Ok(_) => "Texto colado sem formatação".into(), Err(e) => e,
                };
            }
            Mensagem::Tique => {
                if self.aberta(Janela::Monitor) {
                    self.monitor.amostrar();
                }
                self.salvar_automaticamente(std::time::Instant::now());
                self.salvar_documentos_em_segundo_plano(std::time::Instant::now());
                if self.arquivos.checkpoint.elapsed().as_secs() >= 30 {
                    if let Err(e) = self.checkpoints() { self.status = format!("Não consegui preservar a recuperação: {e}"); }
                }
            }
            Mensagem::Faixa(f) => return self.tratar_faixa(f),
            Mensagem::Miniatura(m) => {
                self.foco_miniaturas = true;
                match m {
                    MiniaturaMensagem::Pressionou(i) => {
                        self.arrastando_secao = None;
                        self.inicio_arrasto_secao = None;
                        self.arrastando_slide = Some(i);
                        self.cursor_arrasto = None;
                        self.status = "Solte sobre o slide para criar a visão dele, ou sobre outra miniatura para reordenar".into();
                    }
                    MiniaturaMensagem::Soltou(i) => {
                        // Soltar uma seção sobre a miniatura de outro grupo também
                        // reposiciona o grupo inteiro, não somente seu slide inicial.
                        if self.arrastando_secao.is_some() {
                            self.soltar_secao(i, false);
                            return Task::none();
                        }
                        self.cursor_arrasto = None;
                        match self.arrastando_slide.take() {
                            Some(de) if de == i => self.clicar_slide(i),
                            Some(de) => {
                                let alvos = if self.slides_sel.contains(&de) && self.slides_sel.len() > 1 {
                                    self.slides_sel.iter().copied().collect::<Vec<_>>()
                                } else { vec![de] };
                                match self.sessao.slides_reordenar(&alvos, i) {
                                    Ok(_) => {
                                        let pos = i.min(self.sessao.apresentacao().slides.len() - alvos.len());
                                        self.slides_sel = (pos..pos+alvos.len()).collect();
                                        self.status = format!("{} slides reordenados", alvos.len());
                                    }
                                    Err(e) => self.status = e,
                                }
                            }
                            None => {}
                        }
                    }
                    MiniaturaMensagem::PressionouSecao(i, pos) => {
                        self.arrastando_slide = None;
                        self.arrastando_secao = Some(i);
                        self.inicio_arrasto_secao = Some(pos);
                        self.cursor_arrasto = None;
                        self.menu = None;
                    }
                    MiniaturaMensagem::SoltouSecao(i) => {
                        self.soltar_secao(i, true);
                    }
                    MiniaturaMensagem::MenuSecao(i, pos) => {
                        self.arrastando_secao = None;
                        self.inicio_arrasto_secao = None;
                        self.cursor_arrasto = None;
                        let recolhida = self.secoes_recolhidas.contains(&i);
                        let secoes = self.sessao.secoes();
                        let posicao = secoes.iter().position(|s| s.inicio == i).unwrap_or(0);
                        self.menu = Some(MenuCtx { x: pos.x, y: pos.y, tipo: TipoMenu::Secao {
                            inicio: i, recolhida, primeira: posicao == 0, ultima: posicao + 1 == secoes.len()
                        } });
                    }
                    MiniaturaMensagem::RenomearTexto(t) => {
                        if let Some((_, nome)) = self.renomeando_secao.as_mut() {
                            *nome = t;
                        }
                    }
                    MiniaturaMensagem::RenomearConfirmar => {
                        if let Some((i, nome)) = self.renomeando_secao.take() {
                            let _ = self.sessao.secao_renomear(i, &nome);
                        }
                    }
                    MiniaturaMensagem::Menu(i, pos) => {
                        // Botão direito dentro da seleção mantém todos marcados; fora dela, marca só este.
                        if !self.slides_sel.contains(&i) {
                            self.slides_sel = [i].into();
                        }
                        let _ = self.sessao.slide_ir(i);
                        self.editando_id = None;
                        let total = self.sessao.apresentacao().slides.len();
                        let oculto = self.sessao.slide().oculto;
                        self.menu = Some(MenuCtx { x: pos.x, y: pos.y, tipo: TipoMenu::Slide { oculto, primeiro: i == 0, ultimo: i + 1 >= total, marcados: self.slides_sel.len() } });
                    }
                    MiniaturaMensagem::Lote(acao) => {
                        use crate::miniaturas::Lote;
                        match acao {
                            Lote::NovaSecao => {
                                let alvos = self.alvos_de_slides();
                                let nome = self.sessao.secao_nome_sugerido();
                                // Se o único slide já é o cabeçalho de uma seção,
                                // criar outra seção com uma página nova, sem substituir a atual.
                                let inserir_slide = alvos.len() == 1
                                    && !self.sessao.apresentacao().slides[alvos[0]].secao.is_empty();
                                let resultado = if inserir_slide {
                                    let _ = self.sessao.slide_ir(alvos[0]);
                                    self.sessao.secao_adicionar(&nome).map(|_| self.sessao.atual())
                                } else {
                                    self.sessao.secao_dos_slides(&alvos, &nome)
                                };
                                match resultado {
                                    Ok(inicio) => {
                                        let quantidade = if inserir_slide { 1 } else { alvos.len() };
                                        self.slides_sel = (inicio..inicio + quantidade).collect();
                                        self.renomeando_secao = Some((inicio, nome));
                                        self.status = format!("Seção criada com {} slide(s)", quantidade);
                                        return iced::widget::text_input::focus(iced::widget::text_input::Id::new(ID_RENOMEAR));
                                    }
                                    Err(e) => self.status = e,
                                }
                            }
                        }
                    }
                    MiniaturaMensagem::Novo => {
                        let _ = self.sessao.slide_novo("titulo_conteudo");
                        self.status = "Novo slide adicionado".into();
                    }
                }
            }
            Mensagem::Palco(p) => {
                self.foco_miniaturas = false;
                match p {
                    PalcoMensagem::GradientePonto{indice,posicao}=>return self.tratar(Mensagem::FormatoForma(crate::formato_forma::Msg::MoverPonto(indice,posicao))),
                    PalcoMensagem::GradienteAngulo(a)=>return self.tratar(Mensagem::FormatoForma(crate::formato_forma::Msg::Campo("angulo",format!("{a:.1}")))),
                    PalcoMensagem::DesenhoLivre(pontos) => {
                        self.desenho_livre=false;
                        if let Err(e)=self.sessao.desenho_livre_inserir(&pontos){self.status=e;}
                    }
                    PalcoMensagem::CancelarDesenho => {self.desenho_livre=false;}
                    PalcoMensagem::Clicou { x, y, somar, duplo } => {
                        if self.pincel.is_some() {
                            if let Some(id) = self.sessao.elemento_no_ponto(x, y) {
                                let p = self.pincel.take().unwrap();
                                self.status = match self.sessao.pincel_aplicar(id, &p) { Ok(_) => "Formatação aplicada".into(), Err(e) => e };
                            }
                            return Task::none();
                        }
                        if let Some(estilo)=self.conector_modo {
                            if let Some(id)=self.sessao.selecionar_no_ponto(x,y,false) {
                                if let Some(origem)=self.conector_origem {
                                    self.sessao.selecionar_por_id(origem);
                                    if id!=origem {
                                        self.sessao.selecionar_no_ponto(x,y,true);
                                        match self.sessao.conector_inserir(estilo) {
                                            Ok(_)=>{self.conector_modo=None;self.conector_origem=None;self.status="Conector criado".into();}
                                            Err(e)=>self.status=e,
                                        }
                                    } else {self.status="Clique em outra forma para o destino".into();}
                                } else {self.conector_origem=Some(id);self.status="Agora clique na forma de destino. Esc cancela.".into();}
                            } else {self.status="Clique sobre uma forma. Esc cancela.".into();}
                            return Task::none();
                        }
                        if let Some(manter) = self.marcando {
                            // Um clique sem arrasto marca um quadradinho em volta do ponto.
                            self.adicionar_marca(manter, x - 6.0, y - 6.0, 12.0, 12.0);
                            return Task::none();
                        }
                        if let Some(alvo) = self.conta_gotas.take() {
                            match self.cor_no_slide(x, y) {
                                Some(hex) => {
                                    self.status = format!("Cor pega: {hex}");
                                    self.aplicar_cor(alvo, hex);
                                }
                                None => self.status = "Não achei uma cor nesse ponto".into(),
                            }
                            return Task::none();
                        }
                        if self.pegando_cor {
                            self.pegando_cor = false;
                            let alvo = self.sessao.estado().selecao.first().copied()
                                .and_then(|id| self.sessao.quadro(self.sessao.atual(), None, true).ok().flatten().and_then(|q| q.elementos.into_iter().find(|e| e.id == id)));
                            match alvo.and_then(|el| crate::desenho::cor_dominante_na_area(&el, x, y, 0.0, 0.0)) {
                                Some(cor) => {
                                    let _ = self.sessao.formatar_previa(Formatacao::FundoCor(cor.clone()));
                                    self.sessao.confirmar_previa();
                                    self.status = format!("Cor do fundo a remover: {cor}");
                                }
                                None => self.status = "Clique sobre a imagem selecionada para pegar a cor".into(),
                            }
                            return Task::none();
                        }
                        // Clicar dentro da caixa que está sendo escrita só muda a posição da barrinha.
                        if let Some(id) = self.editando_id {
                            let dentro = self.sessao.slide().elementos.iter().find(|e| e.id == id).is_some_and(|e| x >= e.x && x <= e.x + e.largura && y >= e.y && y <= e.y + e.altura);
                            if dentro && !somar {
                                self.cursor_edicao = self.cursor_no_clique(id, x, y);
                                return Task::none();
                            }
                        }
                        self.tamanho_editando = false;
                        let _ = self.sessao.selecionar_no_ponto(x, y, somar);
                        let estado = self.sessao.estado();
                        // A barra de texto só abre no duplo clique: abrir no clique simples
                        // muda a altura do palco no meio do arrasto e o objeto "foge".
                        if !duplo {
                            self.editando_celula = None;
                            if let Some(crop_id) = self.ajustando_recorte {
                                if !estado.selecao.contains(&crop_id) {
                                    self.ajustando_recorte = None;
                                }
                            }
                        }
                        match estado.selecao.first() {
                            Some(&id) if duplo => {
                                let tipo = self.sessao.slide().elementos.iter().find(|e| e.id == id).map(|e| e.tipo);
                                match tipo {
                                    Some(Tipo::Tabela) => {
                                        if let Some((r, c)) = self.sessao.tabela_celula_no_ponto(id, x, y) {
                                            self.editando_id = None;
                                            self.editando_celula = Some((id, r, c));
                                            self.texto_edicao = self.sessao.tabela_celula_texto(id, r, c).unwrap_or_default();
                                            self.cursor_edicao = self.texto_edicao.len();
                                        }
                                    }
                                    Some(Tipo::Grafico) => {
                                        self.dados_id = Some(id);
                                        if !self.aberta(Janela::Dados) {
                                            return self.alternar(Janela::Dados);
                                        }
                                    }
                                    Some(Tipo::Imagem) => {
                                        if self.ajustando_recorte == Some(id) {
                                            self.ajustando_recorte = None;
                                            self.status = "Ajuste de enquadramento concluído".into();
                                        } else {
                                            self.ajustando_recorte = Some(id);
                                            self.status = "Modo de enquadramento: arraste a foto para posicionar no recorte, use a roda do mouse para zoom, Esc/Enter para sair".into();
                                        }
                                    }
                                    _ => {
                                        self.editando_id = Some(id);
                                        if let Some(el) = self.sessao.slide().elementos.iter().find(|e| e.id == id) {
                                            self.texto_edicao = el.texto.clone();
                                        }
                                        self.cursor_edicao = self.cursor_no_clique(id, x, y);
                                        self.ancora_edicao = None;
                                    }
                                }
                            }
                            Some(_) => {}
                            None => {
                                self.editando_id = None;
                                self.editando_celula = None;
                                self.ajustando_recorte = None;
                            }
                        }
                    }
                    PalcoMensagem::Area { x, y, w, h, somar } => {
                        if let Some(manter) = self.marcando {
                            self.adicionar_marca(manter, x, y, w, h);
                            return Task::none();
                        }
                        if self.pegando_cor {
                            self.pegando_cor = false;
                            let alvo = self.sessao.estado().selecao.first().copied()
                                .and_then(|id| self.sessao.quadro(self.sessao.atual(), None, true).ok().flatten().and_then(|q| q.elementos.into_iter().find(|e| e.id == id)));
                            match alvo.and_then(|el| crate::desenho::cor_dominante_na_area(&el, x, y, w, h)) {
                                Some(cor) => {
                                    let _ = self.sessao.formatar_previa(Formatacao::FundoCor(cor.clone()));
                                    self.sessao.confirmar_previa();
                                    self.status = format!("Cor mais comum da região: {cor} — removida do fundo");
                                }
                                None => self.status = "Selecione uma região dentro da imagem".into(),
                            }
                            return Task::none();
                        }
                        self.sessao.selecionar_na_area([x, y, w, h], somar);
                    }
                    PalcoMensagem::MenuContexto { x, y, janela } => {
                        // Clicar com o direito num elemento o seleciona (se já não está na seleção).
                        let sobre = self.sessao.elemento_no_ponto(x, y);
                        if let Some(id) = sobre {
                            if !self.sessao.estado().selecao.contains(&id) {
                                self.sessao.selecionar_por_id(id);
                            }
                        } else {
                            self.sessao.desmarcar();
                        }
                        self.editando_id = None;
                        self.editando_celula = None;
                        let mut tipo = match sobre.and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == id)) {
                            Some(e) => TipoMenu::Elemento(e.tipo.nome().to_string()),
                            None => TipoMenu::Vazio,
                        };
                        // Sobre uma palavra sublinhada, o menu abre com as correções.
                        self.erro_do_menu = None;
                        let sobre_palavra = self.ondulados_atuais().into_iter().find(|w| x >= w.x && x <= w.x + w.largura && y >= w.y - w.tamanho && y <= w.y + w.tamanho * 0.35);
                        self.chave_gramatica_do_menu = None;
                        if let (Some(w), Some(c)) = (sobre_palavra, self.corretor.as_ref()) {
                            if w.tipo == featherslide_ortografia::TipoOndulado::Gramatica {
                                self.chave_gramatica_do_menu = Some(w.chave);
                                tipo = TipoMenu::Gramatica { mensagem: w.mensagem, sugestao: w.sugestao, base: Box::new(tipo) };
                            } else {
                                let sugestoes = c.sugestoes(&w.erro.palavra);
                                tipo = TipoMenu::Ortografia { sugestoes, base: Box::new(tipo) };
                            }
                            self.erro_do_menu = Some(w.erro);
                        }
                        self.menu = Some(MenuCtx { x: janela.x, y: janela.y, tipo });
                    }
                    PalcoMensagem::TextoPressionou { x, y, duplo } => {
                        if let Some(id) = self.editando_id {
                            let c = self.cursor_no_clique(id, x, y);
                            if duplo {
                                let (a, b) = cursor::palavra_em(&self.texto_edicao, c);
                                self.ancora_edicao = Some(a);
                                self.cursor_edicao = b;
                            } else {
                                // Com Shift, estende a seleção que já existe; sem, começa outra daqui.
                                if !self.modificadores.shift() || self.ancora_edicao.is_none() {
                                    self.ancora_edicao = Some(c);
                                }
                                self.cursor_edicao = c;
                            }
                        }
                    }
                    PalcoMensagem::TextoArrastou { x, y } => {
                        if let Some(id) = self.editando_id {
                            self.cursor_edicao = self.cursor_no_clique(id, x, y);
                        }
                    }
                    PalcoMensagem::Pan { x, y } => self.pan = (x, y),
                    PalcoMensagem::Cantos { raios } => {
                        let r = (raios[0] * 100.0).round() / 100.0;
                        if let Some(el) = self.sessao.slide().elementos.iter().find(|e| self.sessao.estado().selecao.contains(&e.id)) {
                            if el.tipo == Tipo::Imagem {
                                let pan_x = el.ajustes.first().copied().unwrap_or(0.0);
                                let pan_y = el.ajustes.get(1).copied().unwrap_or(0.0);
                                let zoom = el.ajustes.get(2).copied().unwrap_or(1.0);
                                let _ = self.sessao.selecao_ajustes(vec![pan_x, pan_y, zoom, r]);
                            } else {
                                let _ = self.sessao.selecao_ajustes(raios.iter().map(|r| (r * 100.0).round() / 100.0).collect());
                            }
                        }
                    }
                    PalcoMensagem::Girou { id, graus } => {
                        let _ = self.sessao.elemento_rotacao(id, graus);
                    }
                    PalcoMensagem::SoltouSlide { slide, x, y } => {
                        self.arrastando_slide = None;
                        self.cursor_arrasto = None;
                        if slide == self.sessao.atual() {
                            self.status = "Solte um slide diferente do que está aberto".into();
                        } else {
                            let visao = self.gerar_visao_slide(slide);
                            match self.sessao.gatilho_inserir(slide, Some([x, y]), visao) {
                                Ok(_) => self.status = format!("Visão do slide {} adicionada (clique leva a ele)", slide + 1),
                                Err(e) => self.status = e,
                            }
                        }
                    }
                    PalcoMensagem::Zoom(d) => {
                        self.zoom = (self.zoom + d).clamp(0.2, 4.0);
                        if self.zoom <= 1.0 {
                            self.pan = (0.0, 0.0);
                        }
                    }
                    PalcoMensagem::Redimensionou { id, x, y, w, h } => {
                        let resultado = if self.sessao.estado().selecao.len() > 1 {
                            self.sessao.selecao_redimensionar(id, [x, y, w, h])
                        } else { self.sessao.elemento_geometria(id, [x, y, w, h]) };
                        if let Err(e) = resultado { self.status = e; }
                    }
                    PalcoMensagem::Arrastou { dx, dy } => {
                        let _ = self.sessao.selecao_mover(dx, dy);
                    }
                    PalcoMensagem::ArrastouRecorte { id, dx, dy } => {
                        if let Some(el) = self.sessao.slide().elementos.iter().find(|e| e.id == id) {
                            let delta_x = dx / el.largura.max(1.0);
                            let delta_y = dy / el.altura.max(1.0);
                            let _ = self.sessao.imagem_ajustar_recorte(delta_x, delta_y, 0.0);
                        }
                    }
                    PalcoMensagem::ZoomRecorte { id, delta } => {
                        if self.ajustando_recorte == Some(id) {
                            let _ = self.sessao.imagem_ajustar_recorte(0.0, 0.0, delta);
                        }
                    }
                }
            }
            Mensagem::NotasMudou(notas) => {
                let _ = self.sessao.notas_definir(&notas);
            }
            Mensagem::Zoom(delta) => {
                self.zoom = (self.zoom + delta).clamp(0.2, 4.0);
                if self.zoom <= 1.0 {
                    self.pan = (0.0, 0.0);
                }
            }
            Mensagem::ApresentacaoAcao(Acao::Ir(i)) => {
                self.repro.preta = false;
                self.repro.branca = false;
                self.ir_na_apresentacao(i, true);
            }
            Mensagem::ApresentacaoAcao(Acao::Link(id)) => {
                // Só por clique explícito do usuário; o app não abre nada sozinho.
                if let Some(url) = self.sessao.link_do_elemento(self.sessao.atual(), id) {
                    if let Err(e) = abrir_endereco(&url) {
                        self.status = format!("Não consegui abrir o link: {e}");
                    }
                }
            }
            Mensagem::Revisao(m) => return self.tratar_revisao(m),
            Mensagem::Ortografia(m) => return self.tratar_ortografia(m),
            Mensagem::Imprimir(m) => return self.tratar_impressao(m),
            Mensagem::ColarNoTexto(Some(colado)) => {
                if let Some(id) = self.editando_id {
                    let t: String = colado.replace("\r\n", "\n").replace('\r', "\n").chars().filter(|c| !c.is_control() || *c == '\n').collect();
                    if !t.is_empty() {
                        cursor::substituir(&mut self.texto_edicao, &mut self.cursor_edicao, self.ancora_edicao.take(), &t);
                        let _ = self.sessao.texto_definir(id, &self.texto_edicao);
                    }
                }
            }
            Mensagem::ColarNoTexto(None) => {}
            Mensagem::ImpressaoResultado(r) => {
                self.impressao = None;
                self.status = match r {
                    Ok(m) => m,
                    Err(e) => format!("Erro ao imprimir: {e}"),
                };
            }
            Mensagem::IdiomaEscolhido(Some(dic)) => {
                match featherslide_ortografia::instalar_dicionario(&dic) {
                    Ok(codigo) => {
                        if !self.config.idiomas_ortografia.contains(&codigo) {
                            self.config.idiomas_ortografia.push(codigo.clone());
                        }
                        self.config.gravar();
                        self.config_ui.atualizar_ortografia();
                        self.config_ui.aviso = format!("Idioma {codigo} instalado.");
                        self.recarregar_corretor();
                    }
                    Err(e) => self.config_ui.aviso = e,
                }
            }
            Mensagem::IdiomaEscolhido(None) => {}
            Mensagem::OrtografiaPronta(caixa) => {
                let resultado = caixa.lock().ok().and_then(|mut c| c.take());
                match resultado {
                    Some(Ok(c)) => self.corretor = Some(c),
                    Some(Err(e)) => {
                        if let Some(o) = self.ortografia.as_mut() {
                            o.aviso = e.clone();
                        }
                        registro::registrar(format!("[erro] ortografia: {e}"));
                    }
                    None => {}
                }
                return self.ortografia_iniciar();
            }
            Mensagem::PersonalizadasFechar => self.personalizadas = None,
            Mensagem::PersonalizadaNome(nome) => {
                if let Some(p) = self.personalizadas.as_mut() { p.nome_novo = nome; }
            }
            Mensagem::PersonalizadaSelecionar(i) => {
                if let Some(p) = self.personalizadas.as_mut() { p.selecionada = Some(i); }
            }
            Mensagem::PersonalizadaCriar => {
                if let Some(p) = self.personalizadas.as_ref() {
                    let nome = p.nome_novo.clone();
                    let indices = self.alvos_de_slides();
                    match self.sessao.personalizada_criar(&nome, &indices) {
                        Ok(_) => {
                            let i = self.sessao.personalizadas().len()-1;
                            if let Some(estado) = self.personalizadas.as_mut() {
                                estado.selecionada = Some(i);
                                estado.nome_novo.clear();
                            }
                            self.status = "Apresentação personalizada criada".into();
                        }
                        Err(e) => self.status = e,
                    }
                }
            }
            Mensagem::PersonalizadaAdicionar => {
                if let Some(i) = self.personalizadas.as_ref().and_then(|p| p.selecionada) {
                    if let Err(e) = self.sessao.personalizada_adicionar(i, &self.alvos_de_slides()) {
                        self.status = e;
                    }
                }
            }
            Mensagem::PersonalizadaExcluir => {
                if let Some(i) = self.personalizadas.as_ref().and_then(|p| p.selecionada) {
                    match self.sessao.personalizada_excluir(i) {
                        Ok(_) => if let Some(p) = self.personalizadas.as_mut() { p.selecionada = None; },
                        Err(e) => self.status = e,
                    }
                }
            }
            Mensagem::PersonalizadaMover(pos, direcao) => {
                if let Some(i) = self.personalizadas.as_ref().and_then(|p| p.selecionada) {
                    if let Err(e) = self.sessao.personalizada_mover(i, pos, direcao) {
                        self.status = e;
                    }
                }
            }
            Mensagem::PersonalizadaOcultar(pos) => {
                if let Some(i) = self.personalizadas.as_ref().and_then(|p| p.selecionada) {
                    if let Err(e) = self.sessao.personalizada_ocultar(i, pos) {
                        self.status = e;
                    }
                }
            }
            Mensagem::PersonalizadaRemover(pos) => {
                if let Some(i) = self.personalizadas.as_ref().and_then(|p| p.selecionada) {
                    if let Err(e) = self.sessao.personalizada_remover_slide(i, pos) {
                        self.status = e;
                    }
                }
            }
            Mensagem::PersonalizadaIniciar => {
                if let Some(i) = self.personalizadas.as_ref().and_then(|p| p.selecionada) {
                    match self.sessao.personalizada_ordem(i) {
                        Ok(indices) => {
                            self.roteiro_personalizado = Some(indices.iter().map(|&i| self.sessao.apresentacao().slides[i].id).collect());
                            self.personalizadas = None;
                            return self.comecar_apresentacao(true);
                        }
                        Err(e) => self.status = e,
                    }
                }
            }
            Mensagem::ApresentacaoProximo | Mensagem::ApresentacaoAcao(Acao::Proximo) => return self.avancar(),
            Mensagem::ApresentacaoAnterior | Mensagem::ApresentacaoAcao(Acao::Anterior) => self.voltar(),
            Mensagem::ApresentacaoSair => return self.sair_da_apresentacao(),
            Mensagem::CronometroAlternar => self.cronometro.alternar(),
            Mensagem::CronometroReiniciar => self.cronometro.reiniciar(),
            Mensagem::ApresentadorMonitor(nome) => return self.escolher_monitor_plateia(nome),
            Mensagem::TelaApresentacao(tela) => {
                self.repro.preta = tela == 1 && !self.repro.preta;
                self.repro.branca = tela == 2 && !self.repro.branca;
            }
            Mensagem::MenuAcao(f) => {
                self.menu = None;
                return self.tratar_faixa(f);
            }
            Mensagem::FecharMenu => self.menu = None,
            Mensagem::SoltouMouse => return Task::done(Mensagem::LimparArraste),
            Mensagem::LimparArraste => {
                self.arrastando_slide = None;
                self.arrastando_secao = None;
                self.inicio_arrasto_secao = None;
                self.cursor_arrasto = None;
            }
            Mensagem::CursorMoveu(pos) => {
                if self.arrastando_slide.is_some() || self.arrastando_secao.is_some() {
                    self.cursor_arrasto = Some(pos);
                }
            }
            Mensagem::ClassificacaoClique(i) => {
                if self.sessao.slide_ir(i).is_ok() { self.classificacao = false; }
            }
            Mensagem::JanelaRedimensionada(id, tamanho) => {
                if Some(id) == self.janelas.principal {
                    self.tamanho_janela = tamanho;
                }
            }
            Mensagem::QuadroPrevia => {
                if self.previa.as_ref().is_some_and(|p| p.terminou()) { self.previa = None; }
            }
            Mensagem::QuadroApresentacao => {
                if self.repro.transicao.as_ref().is_some_and(|t| t.inicio.elapsed().as_secs_f32() >= t.tr.duracao) {
                    self.repro.transicao = None;
                    self.repro.exibido_em = std::time::Instant::now();
                }
                let apos = self.sessao.slide().transicao.apos;
                if apos > 0.0 && self.repro.transicao.is_none() && self.repro.exibido_em.elapsed().as_secs_f32() >= apos {
                    return self.proximo_slide();
                }
            }
            Mensagem::RecentesCarregados(lista) => self.recentes = lista,
            Mensagem::ColarImagem(None) => return iced::clipboard::read().map(Mensagem::ColarDoSistema),
            Mensagem::ColarImagem(Some(img)) => return self.tratar(Mensagem::ImagemLida(Some(img))),
            Mensagem::ImagemLida(Some((url, w, h))) => {
                // Cabe em 70 % do slide, na proporção da imagem, no meio.
                let (sl, sa) = self.sessao.apresentacao().tamanho();
                let escala = (sl * 0.7 / w).min(sa * 0.7 / h);
                let (largura, altura) = (w * escala, h * escala);
                let rect = [(sl - largura) / 2.0, (sa - altura) / 2.0, largura, altura];
                match self.sessao.elemento_inserir(Tipo::Imagem, Some(rect), Some(url)) {
                    Ok(_) => self.status = "Imagem inserida".into(),
                    Err(e) => self.status = e,
                }
            }
            Mensagem::ImagemLida(None) => {}
            Mensagem::SvgLido(Some(resultado)) => {
                self.status = match resultado.and_then(|d| self.sessao.svg_inserir(d.elementos,d.tamanho)) {
                    Ok(_) => "SVG importado como grupo vetorial".into(), Err(e) => e,
                };
            }
            Mensagem::SvgLido(None) => {}
            Mensagem::SvgExportado(resultado) => { self.status = resultado.unwrap_or_else(|e| e); }
            Mensagem::ImagemArquivoSubstituida(id, Some(imagem)) => {
                if self.sessao.estado().selecao.contains(&id) {
                    match self.sessao.imagem_substituir(imagem, false) {
                        Ok(_) => self.status = "Imagem substituída mantendo tamanho e efeitos".into(),
                        Err(e) => self.status = e,
                    }
                }
            }
            Mensagem::ImagemArquivoSubstituida(_, None) => {}
            Mensagem::ImagemRecomprimida(id, resultado) => {
                if self.sessao.estado().selecao.contains(&id) {
                    match resultado.and_then(|url| self.sessao.imagem_substituir(url, true).map(|_| ())) {
                        Ok(()) => self.status = "Imagem compactada. Ctrl+Z recupera os bytes anteriores.".into(),
                        Err(e) => self.status = format!("Não foi possível compactar: {e}"),
                    }
                }
            }
            Mensagem::ArquivoEscolhidoParaAbrir(Some(p)) => {
                if let Err(e) = self.abrir_documento(p, false) { self.status = e; }
            }
            Mensagem::ArquivoEscolhidoParaAbrir(None) => {}
            Mensagem::PdfEscolhido(Some(mut p)) => {
                if p.extension().map_or(true, |e| !e.eq_ignore_ascii_case("pdf")) {
                    p.set_extension("pdf");
                }
                let so_atual = std::mem::take(&mut self.pdf_so_atual);
                self.status = match self.exportar_pdf(&p, so_atual) {
                    Ok(paginas) => format!("PDF exportado: {paginas} páginas em {}", p.file_name().unwrap_or_default().to_string_lossy()),
                    Err(e) => format!("Não consegui exportar o PDF: {e}"),
                };
            }
            Mensagem::PdfEscolhido(None) => {}
            Mensagem::CopiaEscolhida(Some(p)) => { self.salvar_copia(p); }
            Mensagem::CopiaEscolhida(None) => {}
            Mensagem::ArquivoEscolhidoParaSalvar(caminho) => {
                if let Some(mut p) = caminho {
                    if p.extension().map_or(true, |e| !e.eq_ignore_ascii_case("fsdz")) { p.set_extension("fsdz"); }
                    if self.arquivos.leitura && self.caminho_arquivo.as_ref().is_some_and(|atual| atual == &p || atual.canonicalize().ok().zip(p.canonicalize().ok()).is_some_and(|(a,b)| a == b)) {
                        self.status = "Escolha outro destino para a apresentação somente leitura".into();
                        return Task::none();
                    }
                    if !self.arquivos.leitura { self.atualizar_imagens_de_gatilhos(); }
                    // O formato gravado é sempre `.fsdz`: o nome também.
                    if p.extension().map_or(true, |e| !e.eq_ignore_ascii_case("fsdz")) {
                        p.set_extension("fsdz");
                    }
                    match featherslide_io::documentos::salvar(self.sessao.apresentacao(), &p) {
                        Ok(_) => {
                            self.config.registrar_recente(&p);
                            self.caminho_arquivo = Some(p.clone());
                            self.nome_arquivo = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                            self.sessao.marcar_gravado();
                            let _ = std::fs::remove_file(&self.rascunho);
                            self.status = "Salvo".into();
                            if self.arquivos.painel { self.atualizar_arquivos(); }
                        }
                        Err(e) => self.status = format!("Não consegui salvar: {e}"),
                    }
                }
            }
            Mensagem::Teclado(Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                text,
                ..
            })) => {
                if self.formato_forma.is_some() && matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
                    self.sessao.cancelar_previa();
                    self.formato_forma = None;
                    return Task::none();
                }
                if self.editor_diagrama.is_some() {
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) { self.editor_diagrama = None; }
                    return Task::none();
                }
                if self.ferramentas_texto.is_some() {
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) { self.ferramentas_texto = None; }
                    return Task::none();
                }
                if self.pincel.is_some() && matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
                    self.pincel = None; self.status = "Pincel cancelado".into(); return Task::none();
                }
                if self.conector_modo.is_some() && matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
                    self.conector_modo=None;self.conector_origem=None;self.status="Conector cancelado".into();return Task::none();
                }
                if self.configuracao_slide.is_some() {
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) { self.configuracao_slide = None; }
                    return Task::none();
                }
                if let Some(acao) = feather_ui::atalhos::acao_global(&key, modifiers) {
                    use feather_ui::atalhos::AcaoGlobal;
                    return match acao {
                        AcaoGlobal::Ajuda => self.alternar(Janela::Ajuda),
                        AcaoGlobal::Monitor => self.alternar(Janela::Monitor),
                        AcaoGlobal::AlternarFaixa => self.tratar(Mensagem::AlternarFaixa),
                    };
                }
                if self.menu.is_some() {
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
                        self.menu = None;
                    }
                    return Task::none();
                }
                // Escrevendo numa célula de tabela.
                if let (Some((id, r, c)), false, false) = (self.editando_celula, self.em_apresentacao, modifiers.control()) {
                    use keyboard::key::Named;
                    let mut destino: Option<(usize, usize)> = None;
                    let mut movimento: Option<Navegacao> = None;
                    match key {
                        keyboard::Key::Named(Named::Escape) => self.editando_celula = None,
                        keyboard::Key::Named(Named::Backspace) => {
                            if cursor::apagar_antes(&mut self.texto_edicao, &mut self.cursor_edicao) {
                                if let Err(erro) = featherslide_tabelas::slide_editar_celula(&mut self.sessao, id, r, c, &self.texto_edicao) { self.status = erro; }
                            }
                        }
                        keyboard::Key::Named(Named::Delete) => {
                            if cursor::apagar_depois(&mut self.texto_edicao, self.cursor_edicao) {
                                if let Err(erro) = featherslide_tabelas::slide_editar_celula(&mut self.sessao, id, r, c, &self.texto_edicao) { self.status = erro; }
                            }
                        }
                        keyboard::Key::Named(Named::Home) => self.cursor_edicao = 0,
                        keyboard::Key::Named(Named::End) => self.cursor_edicao = self.texto_edicao.len(),
                        // Esquerda e direita andam dentro do texto; só nas pontas passam para a célula do lado.
                        keyboard::Key::Named(Named::ArrowRight) if self.cursor_edicao < self.texto_edicao.len() => {
                            self.cursor_edicao = cursor::mover(&self.texto_edicao, self.cursor_edicao, Movimento::Direita, None);
                        }
                        keyboard::Key::Named(Named::ArrowLeft) if self.cursor_edicao > 0 => {
                            self.cursor_edicao = cursor::mover(&self.texto_edicao, self.cursor_edicao, Movimento::Esquerda, None);
                        }
                        keyboard::Key::Named(Named::Tab) => {
                            movimento = Some(if modifiers.shift() {
                                Navegacao::ShiftTab
                            } else {
                                Navegacao::Tab
                            });
                        }
                        keyboard::Key::Named(Named::Enter | Named::ArrowDown) => movimento = Some(Navegacao::Baixo),
                        keyboard::Key::Named(Named::ArrowUp) => movimento = Some(Navegacao::Cima),
                        keyboard::Key::Named(Named::ArrowRight) => movimento = Some(Navegacao::Direita),
                        keyboard::Key::Named(Named::ArrowLeft) => movimento = Some(Navegacao::Esquerda),
                        _ => {
                            if let Some(t) = text {
                                let t: String = t.chars().filter(|ch| !ch.is_control()).collect();
                                if !t.is_empty() {
                                    cursor::inserir(&mut self.texto_edicao, &mut self.cursor_edicao, &t);
                                    crate::ferramentas_texto::corrigir(&mut self.texto_edicao, &mut self.cursor_edicao, &self.config);
                                    if let Err(erro) = featherslide_tabelas::slide_editar_celula(&mut self.sessao, id, r, c, &self.texto_edicao) { self.status = erro; }
                                }
                            }
                        }
                    }
                    if let Some(movimento) = movimento {
                        match featherslide_tabelas::slide_navegar_celula(
                            &self.sessao, id, r, c, movimento,
                        ) {
                            Ok(posicao) => destino = Some(posicao),
                            Err(erro) => self.status = erro,
                        }
                    }
                    if let Some((nr, nc)) = destino {
                        self.editando_celula = Some((id, nr, nc));
                        self.texto_edicao = self.sessao.tabela_celula_texto(id, nr, nc).unwrap_or_default();
                        // Chegando pela esquerda o cursor fica no fim do texto; pela direita, no começo.
                        self.cursor_edicao = if (nr, nc) < (r, c) { self.texto_edicao.len() } else { 0 };
                    }
                    return Task::none();
                }
                // Escrever direto no card: o texto vai ao motor a cada tecla e o palco repinta o
                // próprio elemento. O cursor anda com as setas (Ctrl = palavra), Home e End; com
                // Shift as setas selecionam; Ctrl+A (ou Ctrl+T) seleciona tudo; Ctrl+C/X/V copiam.
                let navegacao = matches!(key, keyboard::Key::Named(keyboard::key::Named::ArrowLeft | keyboard::key::Named::ArrowRight | keyboard::key::Named::ArrowUp | keyboard::key::Named::ArrowDown | keyboard::key::Named::Home | keyboard::key::Named::End));
                let letra_ctrl = match &key {
                    keyboard::Key::Character(c) if modifiers.control() => Some(c.to_lowercase()),
                    _ => None,
                };
                let atalho_de_texto = letra_ctrl.as_deref().is_some_and(|c| matches!(c, "a" | "t" | "c" | "x" | "v"));
                if let (Some(id), false, true) = (self.editando_id, self.em_apresentacao, !modifiers.control() || navegacao || atalho_de_texto) {
                    use keyboard::key::Named;
                    let selecao = cursor::intervalo(&self.texto_edicao, self.ancora_edicao, self.cursor_edicao);
                    if let Some(letra) = letra_ctrl.as_deref().filter(|_| atalho_de_texto) {
                        match letra {
                            "a" | "t" => {
                                self.ancora_edicao = Some(0);
                                self.cursor_edicao = self.texto_edicao.len();
                            }
                            "c" => {
                                if let Some((a, b)) = selecao {
                                    return iced::clipboard::write(self.texto_edicao[a..b].to_string());
                                }
                            }
                            "x" => {
                                if let Some((a, b)) = selecao {
                                    let copiado = self.texto_edicao[a..b].to_string();
                                    cursor::apagar_intervalo(&mut self.texto_edicao, &mut self.cursor_edicao, (a, b));
                                    self.ancora_edicao = None;
                                    let _ = self.sessao.texto_definir(id, &self.texto_edicao);
                                    return iced::clipboard::write(copiado);
                                }
                            }
                            _ => return iced::clipboard::read().map(Mensagem::ColarNoTexto),
                        }
                        return Task::none();
                    }
                    let movimento = match (&key, modifiers.control()) {
                        (keyboard::Key::Named(Named::ArrowLeft), false) => Some(Movimento::Esquerda),
                        (keyboard::Key::Named(Named::ArrowRight), false) => Some(Movimento::Direita),
                        (keyboard::Key::Named(Named::ArrowLeft), true) => Some(Movimento::PalavraEsquerda),
                        (keyboard::Key::Named(Named::ArrowRight), true) => Some(Movimento::PalavraDireita),
                        (keyboard::Key::Named(Named::ArrowUp), _) => Some(Movimento::Cima),
                        (keyboard::Key::Named(Named::ArrowDown), _) => Some(Movimento::Baixo),
                        (keyboard::Key::Named(Named::Home), false) => Some(Movimento::InicioDaLinha),
                        (keyboard::Key::Named(Named::End), false) => Some(Movimento::FimDaLinha),
                        (keyboard::Key::Named(Named::Home), true) => Some(Movimento::InicioDoTexto),
                        (keyboard::Key::Named(Named::End), true) => Some(Movimento::FimDoTexto),
                        _ => None,
                    };
                    if let Some(m) = movimento {
                        let shift = modifiers.shift();
                        // Sem Shift, esquerda e direita só recolhem a seleção para a ponta certa.
                        if let (Some((a, b)), false, true) = (selecao, shift, matches!(m, Movimento::Esquerda | Movimento::Direita)) {
                            self.cursor_edicao = if m == Movimento::Esquerda { a } else { b };
                            self.ancora_edicao = None;
                            return Task::none();
                        }
                        if shift && self.ancora_edicao.is_none() {
                            self.ancora_edicao = Some(self.cursor_edicao);
                        } else if !shift {
                            self.ancora_edicao = None;
                        }
                        let linhas = self.linhas_do_elemento(id);
                        let disp = linhas.as_ref().map(|(l, tam, neg)| Disposicao { linhas: l, tamanho: *tam, negrito: *neg });
                        self.cursor_edicao = cursor::mover(&self.texto_edicao, self.cursor_edicao, m, disp.as_ref());
                        return Task::none();
                    }
                    match key {
                        keyboard::Key::Named(Named::Escape) => {
                            self.editando_id = None;
                            self.ancora_edicao = None;
                        }
                        keyboard::Key::Named(Named::Backspace) | keyboard::Key::Named(Named::Delete) if selecao.is_some() => {
                            if let Some(sel) = selecao {
                                cursor::apagar_intervalo(&mut self.texto_edicao, &mut self.cursor_edicao, sel);
                            }
                            self.ancora_edicao = None;
                            let _ = self.sessao.texto_definir(id, &self.texto_edicao);
                        }
                        keyboard::Key::Named(Named::Backspace) => {
                            if cursor::apagar_antes(&mut self.texto_edicao, &mut self.cursor_edicao) {
                                let _ = self.sessao.texto_definir(id, &self.texto_edicao);
                            }
                        }
                        keyboard::Key::Named(Named::Delete) => {
                            if cursor::apagar_depois(&mut self.texto_edicao, self.cursor_edicao) {
                                let _ = self.sessao.texto_definir(id, &self.texto_edicao);
                            }
                        }
                        keyboard::Key::Named(Named::Enter) => {
                            cursor::substituir(&mut self.texto_edicao, &mut self.cursor_edicao, self.ancora_edicao.take(), "\n");
                            crate::ferramentas_texto::corrigir(&mut self.texto_edicao, &mut self.cursor_edicao, &self.config);
                            let _ = self.sessao.texto_definir(id, &self.texto_edicao);
                        }
                        _ => {
                            if let Some(t) = text {
                                let t: String = t.chars().filter(|c| !c.is_control()).collect();
                                if !t.is_empty() {
                                    let inicio = cursor::intervalo(&self.texto_edicao, self.ancora_edicao, self.cursor_edicao).map_or(self.cursor_edicao, |(a, _)| a);
                                    // O que se digita herda x²/x₂: o ligado no botão ou o da letra anterior.
                                    let nivel = if self.ancora_edicao.is_some() { 0 } else { self.nivel_da_digitacao() };
                                    cursor::substituir(&mut self.texto_edicao, &mut self.cursor_edicao, self.ancora_edicao.take(), &t);
                                    crate::ferramentas_texto::corrigir(&mut self.texto_edicao, &mut self.cursor_edicao, &self.config);
                                    if nivel != 0 && inicio < self.cursor_edicao {
                                        let _ = self.sessao.texto_definir_nivelado(id, &self.texto_edicao, inicio, self.cursor_edicao, nivel);
                                    } else {
                                        let _ = self.sessao.texto_definir(id, &self.texto_edicao);
                                    }
                                    if self.indice_digitacao.is_some() {
                                        self.indice_digitacao = Some((self.cursor_edicao, nivel));
                                    }
                                }
                            }
                        }
                    }
                    return Task::none();
                }
                if self.ajustando_recorte.is_some() {
                    use keyboard::key::Named;
                    match &key {
                        keyboard::Key::Named(Named::Escape | Named::Enter) => {
                            self.ajustando_recorte = None;
                            self.status = "Ajuste de enquadramento concluído".into();
                            return Task::none();
                        }
                        keyboard::Key::Named(Named::ArrowLeft) => {
                            let _ = self.sessao.imagem_ajustar_recorte(-0.02, 0.0, 0.0);
                            return Task::none();
                        }
                        keyboard::Key::Named(Named::ArrowRight) => {
                            let _ = self.sessao.imagem_ajustar_recorte(0.02, 0.0, 0.0);
                            return Task::none();
                        }
                        keyboard::Key::Named(Named::ArrowUp) => {
                            let _ = self.sessao.imagem_ajustar_recorte(0.0, -0.02, 0.0);
                            return Task::none();
                        }
                        keyboard::Key::Named(Named::ArrowDown) => {
                            let _ = self.sessao.imagem_ajustar_recorte(0.0, 0.02, 0.0);
                            return Task::none();
                        }
                        keyboard::Key::Character(ref c) if (c == "+" || c == "=") && !modifiers.control() => {
                            let _ = self.sessao.imagem_ajustar_recorte(0.0, 0.0, 0.05);
                            return Task::none();
                        }
                        keyboard::Key::Character(ref c) if c == "-" && !modifiers.control() => {
                            let _ = self.sessao.imagem_ajustar_recorte(0.0, 0.0, -0.05);
                            return Task::none();
                        }
                        _ => {}
                    }
                }
                if self.em_apresentacao {
                    use keyboard::key::Named;
                    match &key {
                        keyboard::Key::Named(Named::Escape) => return self.sair_da_apresentacao(),
                        keyboard::Key::Named(Named::ArrowRight | Named::ArrowDown | Named::Space | Named::PageDown | Named::Enter) => return self.avancar(),
                        keyboard::Key::Named(Named::ArrowLeft | Named::ArrowUp | Named::PageUp | Named::Backspace) => self.voltar(),
                        keyboard::Key::Named(Named::Home) => self.ir_na_apresentacao(self.primeiro_visivel(), false),
                        keyboard::Key::Named(Named::End) => {
                            let ultimo = self.sessao.apresentacao().slides.len().saturating_sub(1);
                            self.ir_na_apresentacao(ultimo, false);
                        }
                        keyboard::Key::Character(c) => match c.to_lowercase().as_str() {
                            "n" => return self.avancar(),
                            "p" => self.voltar(),
                            "b" => {
                                self.repro.preta = !self.repro.preta;
                                self.repro.branca = false;
                            }
                            "w" => {
                                self.repro.branca = !self.repro.branca;
                                self.repro.preta = false;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                    return Task::none();
                }

                // Atalhos configuráveis (Configurações → Atalhos)
                if modifiers.control() && modifiers.shift() && matches!(&key, keyboard::Key::Character(c) if c.eq_ignore_ascii_case("v")) {
                    return self.tratar_faixa(FaixaMensagem::ColarSemFormatacao);
                }
                if let Some(combo) = combo_de(&key, modifiers) {
                    if let Some(id) = self.config.comando_de(&combo) {
                        let faixa = match id {
                            "slide_novo" => Some(FaixaMensagem::SlideNovo),
                            "agrupar" => Some(FaixaMensagem::AgruparOuDesagrupar),
                            "duplicar" => Some(FaixaMensagem::Duplicar),
                            "negrito" => Some(FaixaMensagem::Negrito),
                            "italico" => Some(FaixaMensagem::Italico),
                            "sublinhado" => Some(FaixaMensagem::Sublinhado),
                            "desfazer" => Some(FaixaMensagem::Desfazer),
                            "refazer" => Some(FaixaMensagem::Refazer),
                            "salvar" => Some(FaixaMensagem::Salvar),
                            "salvar_como" => Some(FaixaMensagem::SalvarComo),
                            "salvar_copia" => Some(FaixaMensagem::SalvarCopia),
                            "ortografia" => Some(FaixaMensagem::OrtografiaAbrir),
                            "exportar_pdf" => Some(FaixaMensagem::ExportarPdf),
                            "imprimir" => Some(FaixaMensagem::Imprimir),
                            "abrir" => Some(FaixaMensagem::Abrir),
                            "novo" => Some(FaixaMensagem::Novo),
                            "do_inicio" => Some(FaixaMensagem::Apresentar),
                            "do_atual" => Some(FaixaMensagem::ApresentarDoAtual),
                            _ => None,
                        };
                        if let Some(f) = faixa {
                            return self.tratar_faixa(f);
                        }
                        if id == "ajuda" {
                            return self.alternar(Janela::Ajuda);
                        }
                    }
                }
                // Atalhos de miniaturas: Ctrl+A/C/X/V atuam sobre SLIDES, não objetos.
                if modifiers.control() && self.foco_miniaturas && self.editando_id.is_none() {
                    match &key {
                        keyboard::Key::Character(c) if c.eq_ignore_ascii_case("a") => {
                            return self.tratar_faixa(FaixaMensagem::SlidesSelecionarTodos);
                        }
                        keyboard::Key::Character(c) if c.eq_ignore_ascii_case("c") => {
                            return self.tratar_faixa(FaixaMensagem::SlidesCopiar);
                        }
                        keyboard::Key::Character(c) if c.eq_ignore_ascii_case("x") => {
                            return self.tratar_faixa(FaixaMensagem::SlidesRecortar);
                        }
                        keyboard::Key::Character(c) if c.eq_ignore_ascii_case("v") => {
                            return self.tratar_faixa(FaixaMensagem::SlidesColar(featherslide_core::EstiloColagem::ManterOrigem));
                        }
                        _ => {}
                    }
                }
                // Atalhos comuns do editor
                if modifiers.control() {
                    match key {
                        keyboard::Key::Character(ref c) if c == "s" || c == "S" => {
                            return self.salvar_ou_dialogo();
                        }
                        keyboard::Key::Character(ref c) if c == "z" || c == "Z" => {
                            if modifiers.shift() {
                                let _ = self.sessao.refazer();
                            } else {
                                let _ = self.sessao.desfazer();
                            }
                        }
                        keyboard::Key::Character(ref c) if c == "y" || c == "Y" => {
                            let _ = self.sessao.refazer();
                        }
                        keyboard::Key::Character(ref c) if c == "m" || c == "M" => {
                            let _ = self.sessao.slide_novo("titulo_conteudo");
                        }
                        keyboard::Key::Character(ref c) if c == "+" || c == "=" => self.zoom = (self.zoom + 0.1).clamp(0.2, 4.0),
                        keyboard::Key::Character(ref c) if c == "-" => self.zoom = (self.zoom - 0.1).clamp(0.2, 4.0),
                        keyboard::Key::Character(ref c) if c == "0" => {
                            self.zoom = 1.0;
                            self.pan = (0.0, 0.0);
                        }
                        keyboard::Key::Character(ref c) if c.eq_ignore_ascii_case("c") && self.editando_id.is_none() => {
                            return self.tratar_faixa(FaixaMensagem::Copiar);
                        }
                        keyboard::Key::Character(ref c) if c.eq_ignore_ascii_case("x") && self.editando_id.is_none() => {
                            return self.tratar_faixa(FaixaMensagem::Recortar);
                        }
                        keyboard::Key::Character(ref c) if c.eq_ignore_ascii_case("v") && self.editando_id.is_none() => {
                            return self.tratar_faixa(FaixaMensagem::Colar);
                        }
                        keyboard::Key::Character(ref c) if c == "a" || c == "A" => {
                            self.sessao.selecionar_tudo();
                        }
                        _ => {}
                    }
                } else {
                    match key {
                        keyboard::Key::Named(keyboard::key::Named::F5) => {
                            return self.comecar_apresentacao(true);
                        }
                        keyboard::Key::Named(keyboard::key::Named::Delete) => {
                            if self.editando_id.is_none() {
                                let _ = self.sessao.selecao_excluir();
                            }
                        }
                        keyboard::Key::Named(keyboard::key::Named::Escape) => {
                            if self.editando_id.is_some() {
                                self.editando_id = None;
                            } else {
                                self.sessao.desmarcar();
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Task::none()
    }

    // ---------------------------------------------------------------- apresentação

    fn primeiro_visivel(&self) -> usize {
        let slides = &self.sessao.apresentacao().slides;
        if let Some(roteiro) = &self.roteiro_personalizado {
            return roteiro.iter().find_map(|id| slides.iter().position(|s| s.id == *id)).unwrap_or(0);
        }
        slides.iter().position(|s| !s.oculto).unwrap_or(0)
    }

    /// O próximo slide não oculto depois de `de` (`passo` = 1) ou o anterior (`passo` = -1).
    fn visivel_a_partir_de(&self, de: usize, passo: i32) -> Option<usize> {
        let slides = &self.sessao.apresentacao().slides;
        if let Some(roteiro) = &self.roteiro_personalizado {
            let id_atual = slides.get(de)?.id;
            let origem = roteiro.iter().position(|id| *id == id_atual)?;
            let mut pos = origem as i32 + passo;
            while pos >= 0 && (pos as usize) < roteiro.len() {
                if let Some(i) = slides.iter().position(|s| s.id == roteiro[pos as usize]) {
                    return Some(i);
                }
                pos += passo;
            }
            return None;
        }
        let mut i = de as i32 + passo;
        while i >= 0 && (i as usize) < slides.len() {
            if !slides[i as usize].oculto {
                return Some(i as usize);
            }
            i += passo;
        }
        None
    }

    fn comecar_apresentacao(&mut self, do_inicio: bool) -> Task<Mensagem> {
        if do_inicio {
            let primeiro = self.primeiro_visivel();
            let _ = self.sessao.slide_ir(primeiro);
        }
        self.repro = Reproducao::default();
        self.em_apresentacao = true;
        self.cronometro.reiniciar();
        self.editando_id = None;
        self.editando_celula = None;
        self.monitores_apresentacao = crate::monitores::listar();
        let externo = if self.config.dois_monitores { crate::monitores::da_plateia(&self.monitores_apresentacao).cloned() } else { None };
        if self.modo_apresentador || externo.is_some() {
            self.modo_apresentador = true;
            return self.abrir_plateia(externo.as_ref());
        }
        match self.janelas.principal {
            Some(id) => window::change_mode(id, window::Mode::Fullscreen),
            None => Task::none(),
        }
    }

    /// A plateia também pode ficar numa janela: permite ensaiar com uma única tela.
    fn abrir_plateia(&mut self, monitor: Option<&crate::monitores::Monitor>) -> Task<Mensagem> {
        let settings = match monitor {
            Some(m) => {
                self.monitor_plateia = m.nome.clone();
                window::Settings { size: iced::Size::new(m.largura as f32, m.altura as f32), position: window::Position::Specific(Point::new(m.x as f32, m.y as f32)), decorations: false, ..Default::default() }
            }
            None => {
                self.monitor_plateia = "Janela".into();
                window::Settings { size: iced::Size::new(960.0, 540.0), ..Default::default() }
            }
        };
        let (id, abrir) = window::open(settings);
        self.janelas.registrar(id, Janela::Plateia);
        self.plateia = Some(id);
        let abrir = abrir.map(|_| Mensagem::Tique);
        if monitor.is_some() { abrir.chain(window::change_mode(id, window::Mode::Fullscreen)) } else { abrir }
    }

    fn escolher_monitor_plateia(&mut self, nome: String) -> Task<Mensagem> {
        let Some(id) = self.plateia else { return Task::none() };
        let monitor = self.monitores_apresentacao.iter().find(|m| m.nome == nome).cloned();
        self.monitor_plateia = nome;
        let janela = window::change_mode(id, window::Mode::Windowed);
        match monitor {
            Some(m) => janela.chain(window::move_to(id, Point::new(m.x as f32, m.y as f32)))
                .chain(window::resize(id, iced::Size::new(m.largura as f32, m.altura as f32)))
                .chain(window::change_mode(id, window::Mode::Fullscreen)),
            None => janela.chain(window::resize(id, iced::Size::new(960.0, 540.0))),
        }
    }

    fn sair_da_apresentacao(&mut self) -> Task<Mensagem> {
        self.roteiro_personalizado = None;
        self.em_apresentacao = false;
        self.repro = Reproducao::default();
        self.cronometro = Default::default();
        self.modo_apresentador = false;
        if let Some(id) = self.plateia.take() {
            self.janelas.retirar(id);
            return window::close(id);
        }
        match self.janelas.principal {
            Some(id) => window::change_mode(id, window::Mode::Windowed),
            None => Task::none(),
        }
    }

    /// Vai para o slide `i` da apresentação, com a transição dele se `com_transicao`.
    fn ir_na_apresentacao(&mut self, i: usize, com_transicao: bool) {
        let antigo = self.sessao.quadro(self.sessao.atual(), None, false).ok().flatten();
        if self.sessao.slide_ir(i).is_err() {
            return;
        }
        let tr = self.sessao.slide().transicao.clone();
        self.repro.passo = 0;
        self.repro.inicio_passo = None;
        self.repro.exibido_em = std::time::Instant::now();
        self.repro.transicao = match antigo {
            Some(de) if com_transicao && tr.tipo != featherslide_core::modelo::TipoTransicao::Nenhuma => Some(apresentacao::EmCurso { de, tr, inicio: std::time::Instant::now() }),
            _ => None,
        };
    }

    /// Passa para o slide seguinte (o primeiro, se repetir; senão sai).
    fn proximo_slide(&mut self) -> Task<Mensagem> {
        match self.visivel_a_partir_de(self.sessao.atual(), 1) {
            Some(i) => self.ir_na_apresentacao(i, true),
            None if self.repetir => self.ir_na_apresentacao(self.primeiro_visivel(), true),
            None => return self.sair_da_apresentacao(),
        }
        Task::none()
    }

    /// Clique/seta para a frente: termina o que está acontecendo; senão dispara o próximo passo
    /// de animação; senão passa de slide.
    fn avancar(&mut self) -> Task<Mensagem> {
        self.repro.preta = false;
        self.repro.branca = false;
        if self.repro.transicao.take().is_some() {
            self.repro.exibido_em = std::time::Instant::now();
            return Task::none();
        }
        let passos = self.sessao.passos_de_animacao(self.sessao.atual());
        if let (Some(inicio), Some(ultimo)) = (self.repro.inicio_passo, self.repro.passo.checked_sub(1).and_then(|k| passos.get(k))) {
            if inicio.elapsed().as_secs_f32() < ultimo.duracao {
                self.repro.inicio_passo = None;
                return Task::none();
            }
        }
        if self.repro.passo < passos.len() {
            self.repro.passo += 1;
            self.repro.inicio_passo = Some(std::time::Instant::now());
            return Task::none();
        }
        self.proximo_slide()
    }

    /// Seta para trás: desfaz o último passo de animação; senão volta ao slide anterior (com tudo já na tela).
    fn voltar(&mut self) {
        self.repro.preta = false;
        self.repro.branca = false;
        self.repro.transicao = None;
        if self.repro.passo > 0 {
            self.repro.passo -= 1;
            self.repro.inicio_passo = None;
            return;
        }
        if let Some(i) = self.visivel_a_partir_de(self.sessao.atual(), -1) {
            self.ir_na_apresentacao(i, false);
            self.repro.passo = self.sessao.passos_de_animacao(i).len();
        }
    }

    /// O andamento da animação de cada elemento do slide atual neste instante.
    fn efeitos_atuais(&self) -> crate::desenho::Efeitos {
        let atual = self.sessao.atual();
        let animacoes = self.sessao.animacoes_do_slide(atual);
        let mut efeitos = crate::desenho::Efeitos::new();
        for (k, passo) in self.sessao.passos_de_animacao(atual).iter().enumerate() {
            for (id, comeco) in &passo.elementos {
                let Some((_, anim)) = animacoes.iter().find(|(i, _)| i == id) else { continue };
                let p = if k + 1 < self.repro.passo {
                    1.0
                } else if k + 1 == self.repro.passo {
                    match self.repro.inicio_passo {
                        Some(t) => ((t.elapsed().as_secs_f32() - comeco) / anim.duracao.max(0.05)).clamp(0.0, 1.0),
                        None => 1.0,
                    }
                } else {
                    0.0
                };
                efeitos.insert(*id, (p, anim.clone()));
            }
        }
        efeitos
    }

    /// Põe outra apresentação no lugar da atual. Ids e versões recomeçam em cada uma, então
    /// o que estava guardado das miniaturas da anterior não vale mais.
    pub(crate) fn trocar_sessao(&mut self, nova: Sessao) {
        self.guardar_documento();
        self.sessao = nova;
        self.personalizadas = None;
        self.roteiro_personalizado = None;
        self.arquivos.ativo = true;
        self.arquivos.leitura = false;
        self.limpar_interface_documento();
        self.auto_inicio = std::time::Instant::now();
        self.auto_versao = None;
        self.rascunho = crate::configuracao::novo_rascunho();
        self.cache_quadros.borrow_mut().clear();
        self.slides_sel.clear();
        self.secoes_recolhidas.clear();
        self.pan = (0.0, 0.0);
    }

    /// Clique numa miniatura: Ctrl marca/desmarca, Shift marca o intervalo, sem tecla vai só a ele.
    fn clicar_slide(&mut self, i: usize) {
        let atual = self.sessao.atual();
        if self.modificadores.control() {
            if self.slides_sel.is_empty() {
                self.slides_sel.insert(atual);
            }
            if !self.slides_sel.remove(&i) {
                self.slides_sel.insert(i);
            }
            if self.slides_sel.is_empty() {
                self.slides_sel.insert(i);
            }
        } else if self.modificadores.shift() {
            let (a, b) = (atual.min(i), atual.max(i));
            self.slides_sel = (a..=b).collect();
        } else {
            self.slides_sel = [i].into();
        }
        let _ = self.sessao.slide_ir(i);
        self.editando_id = None;
        if self.slides_sel.len() > 1 {
            self.status = format!("{} slides marcados", self.slides_sel.len());
        }
    }

    /// Termina o arrasto de uma seção. Se soltou no mesmo cabeçalho sem
    /// deslocamento relevante, mantém o clique para recolher/expandir.
    /// Soltar numa miniatura encontra automaticamente a seção daquela miniatura.
    fn soltar_secao(&mut self, destino_slide: usize, sobre_cabecalho: bool) {
        let Some(origem) = self.arrastando_secao.take() else { return };
        let arrastou = self.inicio_arrasto_secao.take()
            .zip(self.cursor_arrasto)
            .is_some_and(|(a, b)| (b.x - a.x).abs() > 6.0 || (b.y - a.y).abs() > 6.0);
        self.cursor_arrasto = None;

        let secoes = self.sessao.secoes();
        let Some(destino) = secoes.iter()
            .find(|s| s.inicio <= destino_slide && destino_slide < s.fim)
            .map(|s| s.inicio) else { return };

        if origem == destino {
            if sobre_cabecalho && destino_slide == origem && !arrastou {
                if !self.secoes_recolhidas.remove(&origem) {
                    self.secoes_recolhidas.insert(origem);
                }
            }
            return;
        }

        // Guardar o estado de recolhimento por ID (posição muda no arrasto).
        let ids_recolhidas: std::collections::HashSet<u64> = secoes.iter()
            .filter(|s| self.secoes_recolhidas.contains(&s.inicio))
            .map(|s| self.sessao.apresentacao().slides[s.inicio].id)
            .collect();
        match self.sessao.secao_mover_para(origem, destino) {
            Ok(_) => {
                let inicio = self.sessao.atual();
                self.slides_sel = [inicio].into();
                self.secoes_recolhidas = self.sessao.secoes().into_iter()
                    .filter(|s| ids_recolhidas.contains(&self.sessao.apresentacao().slides[s.inicio].id))
                    .map(|s| s.inicio)
                    .collect();
                self.status = "Seção movida com todos os seus slides (Ctrl+Z para desfazer)".into();
            }
            Err(e) => self.status = e,
        }
    }

    /// Os slides sobre os quais uma ação de slide age: os marcados (se são vários) ou o atual.
    fn alvos_de_slides(&self) -> Vec<usize> {
        let total = self.sessao.apresentacao().slides.len();
        let marcados: Vec<usize> = self.slides_sel.iter().copied().filter(|&i| i < total).collect();
        if self.foco_miniaturas && !marcados.is_empty()
            || marcados.len() > 1 && marcados.contains(&self.sessao.atual())
        {
            marcados
        } else {
            vec![self.sessao.atual()]
        }
    }

    /// O quadro do slide `i` sem dicas (o das miniaturas), refeito só quando o slide mudou.
    fn quadro_em_cache(&self, i: usize) -> Option<Quadro> {
        let id = self.sessao.apresentacao().slides.get(i)?.id;
        let mut cache = self.cache_quadros.borrow_mut();
        let conhecida = cache.get(&id).map(|q| q.versao);
        match self.sessao.quadro(i, conhecida, false) {
            Ok(Some(novo)) => {
                cache.insert(id, novo.clone());
                Some(novo)
            }
            Ok(None) => cache.get(&id).cloned(),
            Err(_) => None,
        }
    }

    /// A animação do primeiro elemento selecionado.
    fn animacao_da_selecao(&self) -> Option<featherslide_core::modelo::Animacao> {
        let id = *self.sessao.estado().selecao.first()?;
        self.sessao.slide().elementos.iter().find(|e| e.id == id).map(|e| e.animacao.clone())
    }

    /// Muda um campo da animação dos elementos selecionados.
    fn mudar_animacao(&mut self, f: impl FnOnce(&mut featherslide_core::modelo::Animacao)) {
        let mut a = self.animacao_da_selecao().unwrap_or_default();
        f(&mut a);
        // Escolher um efeito numa animação "Nenhuma" já a liga; os outros campos não mudam o tipo.
        let _ = self.sessao.animacao_definir(a);
    }

    /// A cor (preenchimento ou contorno) do primeiro elemento selecionado; vazia = sem cor.
    fn cor_do_alvo(&self, alvo: crate::faixa::AlvoCor) -> String {
        self.sessao
            .estado()
            .selecao
            .first()
            .and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == *id))
            .map(|e| match alvo {
                crate::faixa::AlvoCor::Preenchimento => e.preenchimento.clone(),
                crate::faixa::AlvoCor::Contorno => e.contorno.clone(),
                crate::faixa::AlvoCor::Texto => e.estilo.cor.clone(),
                crate::faixa::AlvoCor::Realce => e.estilo.realce.clone(),
                crate::faixa::AlvoCor::Fundo => e.fundo_cor.clone(),
            })
            .unwrap_or_default()
    }

    fn formatacao_da_cor(alvo: crate::faixa::AlvoCor, hex: String) -> Formatacao {
        match alvo {
            crate::faixa::AlvoCor::Preenchimento => Formatacao::Preenchimento(hex),
            crate::faixa::AlvoCor::Contorno => Formatacao::Contorno(hex),
            crate::faixa::AlvoCor::Texto => Formatacao::CorDoTexto(hex),
            crate::faixa::AlvoCor::Realce => Formatacao::Realce(hex),
            crate::faixa::AlvoCor::Fundo => Formatacao::FundoCor(hex),
        }
    }

    /// Acrescenta uma marca (manter/remover) do watershed na imagem selecionada, a partir de um
    /// retângulo em pontos do slide. A marcação continua ligada para o próximo traço.
    fn adicionar_marca(&mut self, manter: bool, x: f32, y: f32, w: f32, h: f32) {
        let alvo = self.sessao.estado().selecao.first().copied()
            .and_then(|id| self.sessao.quadro(self.sessao.atual(), None, true).ok().flatten().and_then(|q| q.elementos.into_iter().find(|e| e.id == id)));
        match alvo.and_then(|el| crate::desenho::retangulo_na_imagem(&el, x, y, w, h)) {
            Some(r) => {
                let _ = self.sessao.imagem_marca_adicionar(manter, r);
                let (m, r2) = self.sessao.imagem_marcas_contagem();
                self.status = format!("Marcas: {m} para manter, {r2} para remover");
            }
            None => self.status = "Marque dentro da imagem selecionada".into(),
        }
    }

    /// Aplica a cor que o quadrado e as barras do seletor descrevem.
    fn aplicar_hsva(&mut self, alvo: crate::faixa::AlvoCor) {
        let [h, s, v, a] = self.hsva_cor;
        let [r, g, b] = crate::seletor_cor::hsv_para_rgb(h, s, v);
        let hex = crate::seletor_cor::hex_de([r, g, b, (a * 255.0).round() as u8]);
        self.texto_cor = hex.clone();
        let _ = self.sessao.formatar_previa(Self::formatacao_da_cor(alvo, hex));
    }

    /// A cor do ponto (x, y) do slide: imagem, preenchimento de forma ou o fundo do slide.
    fn cor_no_slide(&self, x: f32, y: f32) -> Option<String> {
        let quadro = self.sessao.quadro(self.sessao.atual(), None, true).ok().flatten()?;
        for el in quadro.elementos.iter().rev() {
            if x < el.x || y < el.y || x > el.x + el.largura || y > el.y + el.altura {
                continue;
            }
            if el.tipo == "imagem" {
                if let Some(c) = crate::desenho::cor_no_ponto(el, x, y) {
                    return Some(c);
                }
            } else if !el.preenchimento.is_empty() {
                return Some(el.preenchimento.clone());
            }
        }
        Some(quadro.fundo.clone())
    }

    fn aplicar_cor(&mut self, alvo: crate::faixa::AlvoCor, hex: String) {
        self.hsva_cor = crate::seletor_cor::rgba_para_hsva(crate::seletor_cor::rgba_de(&hex), self.hsva_cor[0]);
        self.texto_cor = hex.clone();
        let _ = self.sessao.formatar_previa(Self::formatacao_da_cor(alvo, hex));
    }

    fn contexto_faixa(&self) -> crate::faixa_abas::Contexto {
        let estado = self.sessao.estado();
        let formato_vertical = estado.formato.as_ref().map(|f| f.vertical.clone()).unwrap_or_default();
        let formato_alinhamento = estado.formato.as_ref().map(|f| f.alinhamento.clone()).unwrap_or_default();
        let tipo_sel = estado
            .selecao
            .first()
            .and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == *id))
            .map(|e| e.tipo.nome().to_string())
            .unwrap_or_default();
        let selecionado = estado.selecao.first().and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == *id));
        let ef = self.sessao.imagem_efeitos();
        crate::faixa_abas::Contexto {
            wordart_texto: selecionado.map(|e| e.texto.clone()).unwrap_or_default(),
            wordart_modo: selecionado.map(|e| featherslide_core::wordart::Transformacao::ler(&e.dados)).unwrap_or_default(),
            wordart_intensidade: selecionado.and_then(|e| e.ajustes.first()).copied().unwrap_or(0.5),
            wordart_sombra: selecionado.is_some_and(|e| !e.sombra.cor.is_empty()),
            fonte: selecionado.map(|e| e.estilo.fonte.clone()).unwrap_or_default(),
            tamanho_fonte: selecionado.map_or(54.0, |e| e.estilo.tamanho),
            cor_texto: selecionado.map(|e| e.estilo.cor.clone()).unwrap_or_default(),
            tem_selecao: !estado.selecao.is_empty(),
            tipo_sel,
            tema: self.sessao.tema_atual().to_string(),
            proporcao: self.sessao.apresentacao().proporcao,
            fundo: self.sessao.slide().fundo.clone(),
            transicao: self.sessao.slide().transicao.clone(),
            animacao: self.animacao_da_selecao(),
            slide_oculto: self.sessao.slide().oculto,
            repetir: self.repetir,
            mostrar_painel: self.config.painel_slides,
            mostrar_notas: self.config.painel_notas,
            grade: self.grade,
            guias: self.guias,
            regua: self.regua,
            ortografia_automatica: self.config.ortografia_automatica,
            gramatica_automatica: self.config.gramatica_automatica,
            classificacao: self.classificacao,
            zoom: self.zoom,
            canto: self.canto_sel,
            carrossel_temas: self.carrossel_temas,
            n_selecao: estado.selecao.len(),
            painel_selecao: self.painel_selecao,
            vertical: formato_vertical,
            alinhamento: formato_alinhamento,
            modo_recorte: self.ajustando_recorte.is_some(),
            fundo_removido: ef.as_ref().map_or(0.0, |e| e.removido),
            fundo_cor: ef.as_ref().map(|e| e.cor.clone()).unwrap_or_default(),
            pegando_cor: self.pegando_cor,
            fundo_suave: ef.as_ref().map_or(0.5, |e| e.suave),
            fundo_modo: ef.as_ref().map_or(0, |e| e.modo),
            fundo_refino: ef.as_ref().is_some_and(|e| e.refino),
            fundo_borda: ef.as_ref().map_or(0.0, |e| e.borda),
            fundo_pena: ef.as_ref().map_or(0.0, |e| e.pena),
            marcando: self.marcando,
            marcas: self.sessao.imagem_marcas_contagem(),
            visual_imagem: selecionado.map(|e| e.visual_imagem.clone()).unwrap_or_default(),
            margens_imagem: selecionado.map_or([0.0;4], |e| [
                e.ajustes.get(4).copied().unwrap_or(0.0),
                e.ajustes.get(5).copied().unwrap_or(0.0),
                e.ajustes.get(6).copied().unwrap_or(0.0),
                e.ajustes.get(7).copied().unwrap_or(0.0),
            ]),
            imagem_sombra: selecionado.is_some_and(|e| !e.sombra.cor.is_empty()),
            imagem_ajuste_selecionado: self.imagem_ajuste_selecionado,
            imagem_efeito_selecionado: self.imagem_efeito_selecionado,
            imagem_borda_selecionada: self.imagem_borda_selecionada,
            imagem_fundo_detalhes: self.imagem_fundo_detalhes,
            imagem_alt_expandido: self.imagem_alt_expandido,
            imagem_alt_texto: selecionado.map_or(String::new(), |e| {
                if self.imagem_alt_rascunho_id == Some(e.id) {
                    self.imagem_alt_rascunho.clone()
                } else {
                    e.texto_alternativo.clone()
                }
            }),
            imagem_opacidade_texto: selecionado.map_or(String::new(), |e| {
                self.imagem_opacidade_rascunho.as_ref()
                    .filter(|(id,_)| *id == e.id)
                    .map(|(_,t)| t.clone())
                    .unwrap_or_else(|| format!("{:.0}", e.opacidade * 100.0))
            }),
            indice_texto: match selecionado {
                Some(e) if self.editando_id == Some(e.id) && cursor::intervalo(&self.texto_edicao, self.ancora_edicao, self.cursor_edicao).is_none() => self.nivel_da_digitacao(),
                Some(e) => featherslide_core::texto_formato::nivel(e, if self.editando_id == Some(e.id) { self.ancora_edicao.unwrap_or(self.cursor_edicao).min(self.cursor_edicao) } else { 0 }),
                None => 0,
            },
            sublinhado_duplo: estado.formato.as_ref().is_some_and(|f| f.sublinhado_duplo),
            tachado_duplo: estado.formato.as_ref().is_some_and(|f| f.tachado_duplo),
            menu_fonte: self.menu_fonte,
            preenchimento: estado.selecao.first().and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == *id)).map(|e| e.preenchimento.clone()).unwrap_or_default(),
            contorno: estado.selecao.first().and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == *id)).map(|e| e.contorno.clone()).unwrap_or_default(),
            espessura: estado.selecao.first().and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == *id)).map_or(1.0, |e| e.espessura),
        }
    }

    fn salvar_copia(&mut self, mut destino: PathBuf) -> bool {
        destino.set_extension("fsdz");
        if self.caminho_arquivo.as_ref().is_some_and(|original| {
            *original == destino || original.canonicalize().ok().zip(destino.canonicalize().ok()).is_some_and(|(a, b)| a == b)
        }) {
            self.status = "Escolha outro destino para salvar a cópia.".into();
            return false;
        }
        match featherslide_io::documentos::salvar(self.sessao.apresentacao(), &destino) {
            Ok(_) => { self.status = format!("Cópia salva: {}", destino.display()); true }
            Err(e) => { self.status = format!("Não consegui salvar a cópia: {e}"); false }
        }
    }

    fn salvar_automaticamente(&mut self, agora: std::time::Instant) {
        let estado = self.sessao.estado();
        if self.arquivos.leitura || !self.config.auto_salvar || !estado.sujo || self.auto_versao == Some(estado.versao)
            || agora.saturating_duration_since(self.auto_inicio).as_secs() < self.config.intervalo.max(1) {
            return;
        }
        let salvo = if self.caminho_arquivo.is_some() {
            self.gravar_no_caminho()
        } else {
            self.atualizar_imagens_de_gatilhos();
            let resultado = self.rascunho.parent().ok_or_else(|| "pasta de rascunhos inválida".to_string())
                .and_then(|p| std::fs::create_dir_all(p).map_err(|e| e.to_string()))
                .and_then(|_| fsdz::salvar(self.sessao.apresentacao(), &self.rascunho));
            match resultado {
                Ok(_) => { self.status = format!("Rascunho automático salvo: {}", self.rascunho.display()); true }
                Err(e) => { self.status = format!("Não consegui salvar automaticamente: {e}"); false }
            }
        };
        self.auto_inicio = agora;
        if salvo {
            self.auto_versao = Some(self.sessao.estado().versao);
            registro::registrar("[salvar] salvamento automático concluído");
        }
    }

    /// Grava no caminho atual; `true` se gravou.
    pub(crate) fn gravar_no_caminho(&mut self) -> bool {
        if self.arquivos.leitura { self.status = "Arquivo somente para leitura; use Salvar cópia".into(); return false; }
        self.atualizar_imagens_de_gatilhos();
        let Some(mut path) = self.caminho_arquivo.clone() else { return false };
        // Nunca grava `.fsdz` por cima de um `.pptx`/`.odp` (corrompia o original): vai para um `.fsdz` ao lado.
        if path.extension().is_some_and(|e| matches!(e.to_string_lossy().to_lowercase().as_str(), "pptx" | "odp")) {
            path.set_extension("fsdz");
        }
        match featherslide_io::documentos::salvar(self.sessao.apresentacao(), &path) {
            Ok(_) => {
                self.caminho_arquivo = Some(path.clone());
                self.nome_arquivo = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                self.sessao.marcar_gravado();
                let _ = std::fs::remove_file(&self.rascunho);
                self.status = "Salvo".into();
                if self.arquivos.painel { self.atualizar_arquivos(); }
                true
            }
            Err(e) => {
                self.status = format!("Não consegui salvar: {e}");
                false
            }
        }
    }

    pub(crate) fn tratar_config(&mut self, m: configuracoes::Msg) -> Task<Mensagem> {
        use configuracoes::Msg;
        match m {
            Msg::Secao(s) => {
                self.config_ui.secao = s;
                self.config_ui.confirmar_limpar = false;
                if s == configuracoes::Secao::Ortografia {
                    self.config_ui.atualizar_ortografia();
                }
            }
            Msg::OrtAuto(b) => self.config.ortografia_automatica = b,
            Msg::Autocorrecao(b) => self.config.autocorrecao = b,
            Msg::Capitalizacao(b) => self.config.capitalizacao = b,
            Msg::Ferramenta(modo) => {
                let tarefa = self.tratar(Mensagem::Faixa(FaixaMensagem::FerramentasTexto(modo)));
                let fechar = if self.aberta(Janela::Config) { self.alternar(Janela::Config) } else { Task::none() };
                return Task::batch([tarefa, fechar]);
            }
            Msg::ConfigurarSlide => {
                self.configuracao_slide = Some(crate::configuracao_slide::Estado::novo(&self.sessao));
                return if self.aberta(Janela::Config) { self.alternar(Janela::Config) } else { Task::none() };
            }
            Msg::GramAuto(b) => self.config.gramatica_automatica = b,
            Msg::OrtIdioma(codigo, usar) => {
                let lista = &mut self.config.idiomas_ortografia;
                if usar && !lista.contains(&codigo) {
                    lista.push(codigo);
                } else if !usar && lista.len() > 1 {
                    lista.retain(|c| *c != codigo);
                } else if !usar {
                    self.config_ui.aviso = "Deixe pelo menos um idioma ligado.".into();
                    return Task::none();
                }
                self.config_ui.aviso.clear();
                self.config.gravar();
                self.recarregar_corretor();
            }
            Msg::OrtPrincipal(codigo) => {
                self.config.idiomas_ortografia.retain(|c| *c != codigo);
                self.config.idiomas_ortografia.insert(0, codigo);
                self.config.gravar();
                self.recarregar_corretor();
            }
            Msg::OrtAdicionar => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new().set_title("Escolha o dicionário (.dic)").add_filter("Dicionário Hunspell (*.dic)", &["dic"]).pick_file().await.map(|f| f.path().to_path_buf())
                    },
                    Mensagem::IdiomaEscolhido,
                );
            }
            Msg::OrtLimpar => self.config_ui.confirmar_limpar = true,
            Msg::OrtLimparNao => self.config_ui.confirmar_limpar = false,
            Msg::OrtLimparSim => {
                self.config_ui.confirmar_limpar = false;
                self.config_ui.aviso = match featherslide_ortografia::limpar_dicionario_pessoal() {
                    Ok(()) => "Dicionário pessoal esvaziado.".into(),
                    Err(e) => format!("Não consegui esvaziar: {e}"),
                };
                self.config_ui.atualizar_ortografia();
                self.recarregar_corretor();
            }
            Msg::AutoSalvar(b) => self.config.auto_salvar = b,
            Msg::Intervalo(s) => self.config.intervalo = s,
            Msg::Escuro(b) => {
                self.config.tema_auto = false;
                self.config.escuro = b;
                definir_tema(b, self.config.acento);
            }
            Msg::TemaAuto => {
                self.config.tema_auto = true;
                definir_tema(escuro_resolvido(true, self.config.escuro), self.config.acento);
            }
            Msg::Acento(i) => {
                self.config.acento = i.min(feather_ui::tema::ACENTOS.len()-1);
                self.config.marca_personalizada = Some(true);
                definir_tema(escuro_resolvido(self.config.tema_auto, self.config.escuro), self.config.acento);
                feather_ui::marca::definir_marca(feather_ui::marca::Produto::FeatherSlide,true);
            }
            Msg::LogoPadrao => {
                self.config.marca_personalizada = Some(false);
                feather_ui::marca::definir_marca(feather_ui::marca::Produto::FeatherSlide,false);
            }
            Msg::GravarAtalho(id) => {
                self.config_ui.gravando = Some(id);
                self.config_ui.aviso.clear();
                return Task::none();
            }
            Msg::RestaurarAtalhos => {
                self.config.atalhos.clear();
                self.config_ui.gravando = None;
                self.config_ui.aviso.clear();
            }
            Msg::PainelSlides(b) => self.config.painel_slides = b,
            Msg::PainelNotas(b) => self.config.painel_notas = b,
            Msg::Grade(b) => self.grade = b,
            Msg::BarraPropria(b) => self.config.barra_propria = b,
            Msg::DoisMonitores(b) => self.config.dois_monitores = b,
            Msg::Guias(b) => self.guias = b,
            Msg::Repetir(b) => self.repetir = b,
            Msg::Proporcao(p) => {
                let _ = self.sessao.proporcao_definir(p);
            }
            Msg::NomeModelo(n) => {
                self.config_ui.nome_modelo = n;
                return Task::none();
            }
            Msg::SalvarModelo => {
                let nome = if self.config_ui.nome_modelo.trim().is_empty() { self.sessao.apresentacao().titulo.clone() } else { self.config_ui.nome_modelo.clone() };
                let nome = if nome.trim().is_empty() { self.nome_arquivo.trim_end_matches(".fsdz").to_string() } else { nome };
                match crate::modelos_usuario::criar(&nome, self.sessao.apresentacao()) {
                    Ok(_) => {
                        self.config_ui.aviso = format!("Modelo \"{nome}\" criado.");
                        self.config_ui.nome_modelo.clear();
                        self.status = "Modelo criado".into();
                    }
                    Err(e) => self.config_ui.aviso = format!("Erro ao criar o modelo: {e}"),
                }
                self.meus_modelos = crate::modelos_usuario::listar();
                return Task::none();
            }
            Msg::ImportarModelo => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("Apresentações (*.fsdz, *.pptx, *.odp)", &["fsdz", "pptx", "odp"])
                            .pick_file()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    Mensagem::ModeloImportarEscolhido,
                );
            }
            Msg::UsarModelo(i) => {
                if let Some(m) = self.meus_modelos.get(i) {
                    let caminho = m.caminho.clone();
                    return self.tratar(Mensagem::NovoDeMeuModelo(caminho));
                }
                return Task::none();
            }
            Msg::ExportarModelo(i) => {
                if let Some(m) = self.meus_modelos.get(i) {
                    let origem = m.caminho.clone();
                    let nome = format!("{}.fsdz", m.nome);
                    return Task::perform(
                        async move {
                            let escolhido = rfd::AsyncFileDialog::new().set_file_name(&nome).add_filter("FeatherSlide (*.fsdz)", &["fsdz"]).save_file().await.map(|f| f.path().to_path_buf());
                            (origem, escolhido)
                        },
                        |(o, d)| Mensagem::ModeloExportarEscolhido(o, d),
                    );
                }
                return Task::none();
            }
            Msg::RemoverModelo(i) => {
                if let Some(m) = self.meus_modelos.get(i) {
                    match crate::modelos_usuario::remover(&m.caminho) {
                        Ok(()) => self.config_ui.aviso = format!("Modelo \"{}\" removido.", m.nome),
                        Err(e) => self.config_ui.aviso = e,
                    }
                }
                self.meus_modelos = crate::modelos_usuario::listar();
                return Task::none();
            }
            Msg::SalvarAgora => return self.salvar_ou_dialogo(),
            Msg::SalvarComo => return self.abrir_dialogo_salvar(),
            Msg::SalvarCopia => return self.abrir_dialogo_copia(),
        }
        self.config.gravar();
        Task::none()
    }

    /// Grava o PDF (um slide por página; os ocultos ficam de fora). Devolve o número de páginas.
    fn exportar_pdf(&self, destino: &std::path::Path, so_atual: bool) -> Result<usize, String> {
        let ap = self.sessao.apresentacao();
        let mut quadros: Vec<_> = (0..ap.slides.len()).filter(|&i| if so_atual { i == self.sessao.atual() } else { !ap.slides[i].oculto }).filter_map(|i| self.sessao.quadro(i, None, false).ok().flatten()).collect();
        if quadros.is_empty() {
            return Err("não há slides visíveis para exportar".into());
        }
        // Efeitos externos da foto são camadas PNG temporárias na saída PDF.
        // O arquivo .fsdz nunca é substituído por pixels processados.
        let camada_raster = |base: &featherslide_core::quadro::ElementoDesenhado,
            rgba: image::RgbaImage, x: f32, y: f32, largura: f32, altura: f32|
            -> Option<featherslide_core::quadro::ElementoDesenhado>
        {
            let mut memoria = std::io::Cursor::new(Vec::new());
            image::DynamicImage::ImageRgba8(rgba)
                .write_to(&mut memoria, image::ImageFormat::Png).ok()?;
            let mut camada = base.clone();
            camada.imagem = featherslide_io::base64::para_data_url(
                "image/png", &memoria.into_inner()).into();
            camada.x = x;
            camada.y = y;
            camada.largura = largura;
            camada.altura = altura;
            camada.sombra = None;
            camada.visual_imagem = Default::default();
            camada.ajustes.clear();
            camada.recorte = "retangulo";
            camada.contorno.clear();
            camada.fundo_removido = 0.0;
            Some(camada)
        };
        for quadro in &mut quadros {
            let mut preparados = Vec::with_capacity(quadro.elementos.len());
            for mut elemento in std::mem::take(&mut quadro.elementos) {
                if elemento.tipo == "wordart" {
                    elemento.primitivas = crate::texto_artistico::wordart(&elemento);
                    elemento.linhas.clear();
                    // WordArt já tem a sombra nos contornos vetoriais.
                    elemento.sombra = None;
                }
                if elemento.tipo == "imagem" &&
                    (elemento.sombra.as_ref().is_some_and(|s| !s.cor.is_empty())
                      || elemento.visual_imagem.reflexo > 0.001
                      || elemento.visual_imagem.brilho_externo > 0.001)
                {
                    if let Some(pixels) = crate::desenho::decodificar(&elemento) {
                        if elemento.visual_imagem.brilho_externo > 0.001 {
                            let (halo, margem) = featherslide_io::imagem_compartilhada::gerar_brilho(
                                &pixels, elemento.visual_imagem.brilho_externo);
                            let mx = margem as f32/pixels.width().max(1) as f32;
                            let my = margem as f32/pixels.height().max(1) as f32;
                            if let Some(camada) = camada_raster(&elemento, halo,
                                elemento.x-elemento.largura*mx,
                                elemento.y-elemento.altura*my,
                                elemento.largura*(1.0+2.0*mx),
                                elemento.altura*(1.0+2.0*my))
                            {
                                preparados.push(camada);
                            }
                        }
                        if let Some(sombra) = elemento.sombra.as_ref().filter(|s| !s.cor.is_empty()) {
                            let escala = pixels.width() as f32/elemento.largura.max(1.0);
                            let resultado = featherslide_io::imagem_compartilhada::gerar_sombra(
                                &pixels, &sombra.cor, sombra.desfoque*escala);
                            if let Some(camada) = camada_raster(&elemento, resultado,
                                elemento.x+sombra.dx, elemento.y+sombra.dy,
                                elemento.largura, elemento.altura)
                            {
                                preparados.push(camada);
                            }
                        }
                        if elemento.visual_imagem.reflexo > 0.001 {
                            let resultado = featherslide_io::imagem_compartilhada::gerar_reflexo(
                                &pixels, elemento.visual_imagem.reflexo);
                            if let Some(camada) = camada_raster(&elemento, resultado,
                                elemento.x, elemento.y+elemento.altura+3.0,
                                elemento.largura, elemento.altura/3.0)
                            {
                                preparados.push(camada);
                            }
                        }
                    }
                }
                preparados.push(elemento);
            }
            quadro.elementos = preparados;
        }
        let imagem = |e: &featherslide_core::quadro::ElementoDesenhado| crate::desenho::decodificar(e).map(|i| (i.width(), i.height(), i.into_raw()));
        let bytes = featherslide_io::pdf::exportar(&ap.titulo, &quadros, &imagem);
        std::fs::write(destino, bytes).map_err(|e| e.to_string())?;
        Ok(quadros.len())
    }

    fn salvar_ou_dialogo(&mut self) -> Task<Mensagem> {
        if self.caminho_arquivo.is_some() {
            self.gravar_no_caminho();
            Task::none()
        } else {
            self.abrir_dialogo_salvar()
        }
    }

    fn abrir_dialogo_abrir(&self) -> Task<Mensagem> {
        Task::perform(
            async {
                let mut dialogo = rfd::AsyncFileDialog::new().add_filter("Apresentações (*.fsdz, *.pptx, *.odp)", &["fsdz", "pptx", "odp"]);
                if let Some(pasta) = crate::configuracao::pasta_das_apresentacoes() {
                    dialogo = dialogo.set_directory(pasta);
                }
                dialogo
                    .pick_file()
                    .await
                    .map(|f| f.path().to_path_buf())
            },
            Mensagem::ArquivoEscolhidoParaAbrir,
        )
    }

    fn abrir_dialogo_copia(&self) -> Task<Mensagem> {
        let nome = format!("{} - cópia.fsdz", std::path::Path::new(&self.nome_arquivo).file_stem().unwrap_or_default().to_string_lossy());
        Task::perform(async move {
            let mut dialogo = rfd::AsyncFileDialog::new().set_title("Salvar cópia")
                .set_file_name(&nome).add_filter("FeatherSlide (*.fsdz)", &["fsdz"]);
            if let Some(pasta) = crate::configuracao::pasta_das_apresentacoes() { dialogo = dialogo.set_directory(pasta); }
            dialogo.save_file().await.map(|f| f.path().to_path_buf())
        }, Mensagem::CopiaEscolhida)
    }

    fn abrir_dialogo_salvar(&self) -> Task<Mensagem> {
        let nome_sugerido = self.nome_arquivo.clone();
        Task::perform(
            async move {
                let mut dialogo = rfd::AsyncFileDialog::new().set_file_name(&nome_sugerido).add_filter("FeatherSlide (*.fsdz)", &["fsdz"]);
                if let Some(pasta) = crate::configuracao::pasta_das_apresentacoes() {
                    dialogo = dialogo.set_directory(pasta);
                }
                dialogo
                    .save_file()
                    .await
                    .map(|f| f.path().to_path_buf())
            },
            Mensagem::ArquivoEscolhidoParaSalvar,
        )
    }

    /// O nível (x² = 1, x₂ = -1) do que se digitar no cursor: o ligado no botão, se o
    /// cursor não saiu do lugar; senão, o da letra logo antes.
    fn nivel_da_digitacao(&self) -> i8 {
        if let Some((pos, nivel)) = self.indice_digitacao {
            if pos == self.cursor_edicao {
                return nivel;
            }
        }
        self.editando_id
            .and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == id))
            .map_or(0, |e| featherslide_core::texto_formato::nivel_antes(e, self.cursor_edicao))
    }

    /// Aumentar/Diminuir recuo (nível da lista).
    fn recuar_texto(&mut self, mais: bool) -> Task<Mensagem> {
        // Em edição: as linhas do trecho (ou a do cursor); senão, a caixa inteira.
        let trecho = self.editando_id.map(|id| {
            let (a, b) = cursor::intervalo(&self.texto_edicao, self.ancora_edicao, self.cursor_edicao)
                .unwrap_or((self.cursor_edicao, self.cursor_edicao));
            (id, a, b)
        });
        let mapa = trecho.map(|(_, a, b)| featherslide_core::texto_formato::recuar(&self.texto_edicao, a, b, mais));
        match self.sessao.texto_recuo(trecho, mais) {
            Err(e) => self.status = e,
            Ok(_) => if let Some((novo, mapa)) = mapa {
                use featherslide_core::texto_formato::traduzir;
                self.cursor_edicao = traduzir(&mapa, self.cursor_edicao);
                self.ancora_edicao = self.ancora_edicao.map(|a| traduzir(&mapa, a));
                self.texto_edicao = novo;
            },
        }
        Task::none()
    }

    fn tratar_faixa(&mut self, msg: FaixaMensagem) -> Task<Mensagem> {
        // A faixa original possui Copiar/Recortar/Colar de elementos. Quando a pessoa
        // está trabalhando nas miniaturas, esses ícones também operam sobre SLIDES.
        if self.foco_miniaturas {
            match msg {
                FaixaMensagem::Copiar => return self.tratar_faixa(FaixaMensagem::SlidesCopiar),
                FaixaMensagem::Recortar => return self.tratar_faixa(FaixaMensagem::SlidesRecortar),
                FaixaMensagem::Colar if self.slides_copiados.is_some() => {
                    return self.tratar_faixa(FaixaMensagem::SlidesColar(featherslide_core::EstiloColagem::ManterOrigem));
                }
                _ => {}
            }
        }
        match msg {
            FaixaMensagem::FerramentasTexto(modo) => {
                self.abrir_ferramentas_texto(modo);
                if modo == crate::ferramentas_texto::Modo::Dicionario && self.corretor.is_none() && !self.corretor_carregando {
                    self.corretor_carregando = true;
                    return Self::carregar_corretor(self.config.idiomas_ortografia.clone());
                }
            }
            FaixaMensagem::ColarSemFormatacao => return iced::clipboard::read().map(Mensagem::ColarTextoSimples),
            FaixaMensagem::ColarOriginal => {
                if self.editando_id.is_some() {
                    self.status = "Para preservar o estilo original, cole como elemento fora da edição de texto.".into();
                } else { return iced::clipboard::read().map(Mensagem::ColarOriginalDoSistema); }
            }
            FaixaMensagem::Pincel => match self.sessao.pincel_capturar() {
                Ok(p) => {
                    self.pincel = Some(p); self.conector_modo = None; self.desenho_livre = false;
                    self.editando_id = None;
                    self.status = "Clique no elemento de destino para aplicar a formatação. Esc cancela.".into();
                }, Err(e) => self.status = e,
            },
            FaixaMensagem::AbrirAjuda => return self.alternar(Janela::Ajuda),
            FaixaMensagem::AbrirMonitor => return self.alternar(Janela::Monitor),
            FaixaMensagem::AbrirConfig => return self.alternar(Janela::Config),
            FaixaMensagem::ImagemAbrirAjustes => {
                if let Some(el) = self.sessao.estado().selecao.first()
                    .and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == *id))
                    .filter(|e| e.tipo == Tipo::Imagem)
                {
                    self.imagem_alt_rascunho = el.texto_alternativo.clone();
                    self.pagina_imagem = crate::imagem_painel::Pagina::Cor;
                    self.painel_imagem = true;
                } else {
                    self.status = "Selecione uma imagem para abrir os ajustes.".into();
                }
            }
            FaixaMensagem::ImagemFecharAjustes => self.painel_imagem = false,
            FaixaMensagem::ImagemAjusteSelecionado(campo) => self.imagem_ajuste_selecionado = campo,
            FaixaMensagem::ImagemEfeitoSelecionado(campo) => self.imagem_efeito_selecionado = campo,
            FaixaMensagem::ImagemBordaSelecionada(borda) => self.imagem_borda_selecionada = borda,
            FaixaMensagem::ImagemFundoDetalhes(aberto) => self.imagem_fundo_detalhes = aberto,
            FaixaMensagem::ImagemAltExpandir(aberto) => self.imagem_alt_expandido = aberto,
            FaixaMensagem::ImagemOpacidadeTexto(texto) => {
                if let Some(id) = self.sessao.estado().selecao.first().copied() {
                    if texto.len() <= 10 { self.imagem_opacidade_rascunho = Some((id, texto)); }
                }
            }
            FaixaMensagem::ImagemOpacidadeAplicar => {
                if let Some((id, texto)) = self.imagem_opacidade_rascunho.clone() {
                    if self.sessao.estado().selecao.contains(&id) {
                        let normalizado = texto.trim().trim_end_matches('%').replace(',', ".");
                        match normalizado.parse::<f32>() {
                            Ok(valor) if valor.is_finite() && (0.0..=100.0).contains(&valor) => {
                                match self.sessao.imagem_opacidade(valor / 100.0) {
                                    Ok(_) => self.imagem_opacidade_rascunho = None,
                                    Err(e) => self.status = e,
                                }
                            }
                            _ => self.status = "Opacidade inválida: informe de 0 a 100%.".into(),
                        }
                    }
                }
            },
            FaixaMensagem::ImagemPagina(pagina) => self.pagina_imagem = pagina,
            FaixaMensagem::ImagemCortarMargem(borda, valor) => {
                if let Err(e) = self.sessao.imagem_cortar_margem(borda, valor) {
                    self.status = e;
                }
            }
            FaixaMensagem::ImagemVisual(campo, valor) => {
                let resultado = match campo {
                    "corte_esquerda" => self.sessao.imagem_cortar_margem("esquerda", valor),
                    "corte_topo" => self.sessao.imagem_cortar_margem("topo", valor),
                    "corte_direita" => self.sessao.imagem_cortar_margem("direita", valor),
                    "corte_base" => self.sessao.imagem_cortar_margem("base", valor),
                    _ => self.sessao.imagem_visual_definir(campo, valor),
                };
                if let Err(e) = resultado {
                    self.status = e;
                }
            }
            FaixaMensagem::ImagemResetarVisual => {
                if let Err(e) = self.sessao.imagem_efeitos_resetar() {
                    self.status = e;
                }
            }
            FaixaMensagem::ImagemTextoAltAlterado(texto) => {
                self.imagem_alt_rascunho_id = self.sessao.estado().selecao.first().copied();
                self.imagem_alt_rascunho = texto.chars().take(4000).collect();
            }
            FaixaMensagem::ImagemTextoAltAplicar => {
                if let Some(id) = self.sessao.estado().selecao.first().copied() {
                    let alternativa = if self.imagem_alt_rascunho_id == Some(id) {
                        self.imagem_alt_rascunho.clone()
                    } else {
                        self.sessao.slide().elementos.iter().find(|e| e.id == id)
                            .map(|e| e.texto_alternativo.clone()).unwrap_or_default()
                    };
                    match self.sessao.imagem_texto_alternativo(&alternativa) {
                        Ok(_) => { self.imagem_alt_rascunho_id = None; self.status = "Texto alternativo atualizado".into(); },
                        Err(e) => self.status = e,
                    }
                }
            }
            FaixaMensagem::ImagemSombra(ligado) => {
                match self.sessao.imagem_sombra(ligado) {
                    Ok(_) => self.status = if ligado { "Sombra aplicada à imagem".into() } else { "Sombra removida".into() },
                    Err(e) => self.status = e,
                }
            }
            FaixaMensagem::ImagemSubstituir => {
                let Some(&id) = self.sessao.estado().selecao.first() else {
                    self.status = "Selecione uma imagem.".into();
                    return Task::none();
                };
                return Task::perform(async move {
                    let caminho = rfd::AsyncFileDialog::new()
                        .add_filter("Imagens", &["png","jpg","jpeg","gif","webp","bmp"])
                        .pick_file().await.map(|f| f.path().to_path_buf());
                    let Some(caminho) = caminho else { return (id,None) };
                    let img = tokio::task::spawn_blocking(move || {
                        let bytes = std::fs::read(&caminho).ok()?;
                        if bytes.len() > 64*1024*1024 { return None; }
                        let tipo = match caminho.extension()?.to_string_lossy().to_lowercase().as_str() {
                            "jpeg"|"jpg" => "image/jpeg",
                            "gif" => "image/gif", "bmp" => "image/bmp",
                            "webp" => "image/webp", _ => "image/png",
                        };
                        image::load_from_memory(&bytes).ok()?;
                        Some(featherslide_io::base64::para_data_url(tipo,&bytes))
                    }).await.ok().flatten();
                    (id,img)
                }, |(id,img)| Mensagem::ImagemArquivoSubstituida(id,img));
            }
            FaixaMensagem::ImagemReduzir(lado) => {
                let Some(&id) = self.sessao.estado().selecao.first() else {
                    self.status = "Selecione uma imagem.".into(); return Task::none();
                };
                let Some(fonte) = self.sessao.slide().elementos.iter().find(|e| e.id==id && e.tipo==Tipo::Imagem)
                    .map(|e| e.imagem.to_string()) else { return Task::none(); };
                self.status = format!("Reduzindo imagem para até {lado} px…");
                return Task::perform(async move {
                    let resultado = tokio::task::spawn_blocking(move || crate::imagem_arquivo::recomprimir(&fonte, Some(lado), 90))
                        .await.unwrap_or_else(|e| Err(e.to_string()));
                    (id,resultado)
                }, |(id,r)| Mensagem::ImagemRecomprimida(id,r));
            }
            FaixaMensagem::ImagemComprimir(qualidade) => {
                let Some(&id) = self.sessao.estado().selecao.first() else {
                    self.status = "Selecione uma imagem.".into(); return Task::none();
                };
                let Some(fonte) = self.sessao.slide().elementos.iter().find(|e| e.id==id && e.tipo==Tipo::Imagem)
                    .map(|e| e.imagem.to_string()) else { return Task::none(); };
                self.status = format!("Compactando imagem (JPEG {qualidade})…");
                return Task::perform(async move {
                    let resultado = tokio::task::spawn_blocking(move || crate::imagem_arquivo::recomprimir(&fonte, None, qualidade))
                        .await.unwrap_or_else(|e| Err(e.to_string()));
                    (id,resultado)
                }, |(id,r)| Mensagem::ImagemRecomprimida(id,r));
            }
            FaixaMensagem::ImagemOpacidade(v) => {
                self.imagem_opacidade_rascunho = None;
                let _ = self.sessao.imagem_opacidade(v);
            }
            FaixaMensagem::ImagemCorDoFundo(c) => {
                self.pegando_cor = false;
                let _ = self.sessao.imagem_cor_do_fundo(&c);
            }
            FaixaMensagem::ImagemSuavidadeFundo(v) => {
                let _ = self.sessao.imagem_suavidade_fundo(v);
            }
            FaixaMensagem::ImagemModoFundo(m) => {
                let _ = self.sessao.imagem_modo_fundo(m);
            }
            FaixaMensagem::ImagemMarcar(manter) => {
                self.pegando_cor = false;
                self.marcando = if self.marcando == Some(manter) { None } else { Some(manter) };
                if self.marcando.is_some() {
                    self.status = if manter { "Arraste sobre o que deve ser MANTIDO (parte do objeto)".into() } else { "Arraste sobre o que deve ser REMOVIDO (parte do fundo)".into() };
                }
            }
            FaixaMensagem::ImagemLimparMarcas => {
                self.marcando = None;
                let _ = self.sessao.imagem_marcas_limpar();
            }
            FaixaMensagem::ImagemAjusteBorda(borda, pena) => {
                let _ = self.sessao.imagem_ajuste_borda(borda, pena);
            }
            FaixaMensagem::ImagemRefinarBorda(v) => {
                let _ = self.sessao.imagem_refinar_borda(v);
            }
            FaixaMensagem::ImagemPegarCor => {
                self.marcando = None;
                self.pegando_cor = !self.pegando_cor;
                if self.pegando_cor {
                    self.status = "Conta-gotas: clique no slide, sobre o fundo da imagem, para escolher a cor a remover".into();
                }
            }
            FaixaMensagem::ImagemRemoverFundo(v) => {
                let _ = self.sessao.imagem_remover_fundo(v);
            }
            FaixaMensagem::TrocarAba(aba) => {
                self.aba_faixa = aba;
                if aba==AbaDaFaixa::Imagem && self.sessao.estado().selecao.first()
                    .and_then(|id|self.sessao.slide().elementos.iter().find(|e|e.id==*id))
                    .is_some_and(|e|featherslide_core::estilo_forma::aceita(e.tipo)) {
                    return self.tratar_faixa(FaixaMensagem::FormatoForma);
                }
            }
            FaixaMensagem::Novo => {
                self.trocar_sessao(Sessao::default());
                self.aplicar_dicas();
                self.caminho_arquivo = None;
                self.nome_arquivo = "Apresentação sem título".into();
                self.status = "Novo documento".into();
            }
            FaixaMensagem::Abrir => {
                return self.abrir_dialogo_abrir();
            }
            FaixaMensagem::Salvar => {
                return self.salvar_ou_dialogo();
            }
            FaixaMensagem::Propriedades => {
                self.propriedades = Some(propriedades::Estado::nova(self.sessao.apresentacao()));
            }
            FaixaMensagem::Imprimir => {
                if self.impressao.is_none() {
                    self.impressao = Some(crate::imprimir::Impressao::nova());
                }
            }
            FaixaMensagem::ExportarPdf => {
                self.pdf_so_atual = false;
                return self.dialogo_salvar_pdf();
            }
            FaixaMensagem::SalvarCopia => return self.abrir_dialogo_copia(),
            FaixaMensagem::SalvarComo => {
                return self.abrir_dialogo_salvar();
            }
            FaixaMensagem::Desfazer => {
                let _ = self.sessao.desfazer();
            }
            FaixaMensagem::Refazer => {
                let _ = self.sessao.refazer();
            }
            FaixaMensagem::SlideNovo => {
                let _ = self.sessao.slide_novo("titulo_conteudo");
                self.slides_sel = [self.sessao.atual()].into();
            }
            FaixaMensagem::SlideInserirAntes => {
                let i = self.sessao.atual();
                match self.sessao.slide_inserir_em(i, "titulo_conteudo") {
                    Ok(_) => { self.slides_sel = [i].into(); self.foco_miniaturas = true; },
                    Err(e) => self.status = e,
                }
            }
            FaixaMensagem::SlideInserirDepois => {
                let i = self.sessao.atual() + 1;
                match self.sessao.slide_inserir_em(i, "titulo_conteudo") {
                    Ok(_) => { self.slides_sel = [i].into(); self.foco_miniaturas = true; },
                    Err(e) => self.status = e,
                }
            }
            FaixaMensagem::SlidesCopiar => {
                match self.sessao.slides_copiar(&self.alvos_de_slides()) {
                    Ok(dados) => { self.status = format!("{} slides copiados", dados.quantidade()); self.slides_copiados = Some(dados); },
                    Err(e) => self.status = e,
                }
            }
            FaixaMensagem::SlidesRecortar => {
                match self.sessao.slides_recortar(&self.alvos_de_slides()) {
                    Ok((dados, _)) => {
                        self.status = format!("{} slides recortados", dados.quantidade());
                        self.slides_copiados = Some(dados);
                        self.slides_sel = [self.sessao.atual()].into();
                    },
                    Err(e) => self.status = e,
                }
            }
            FaixaMensagem::SlidesColar(estilo) => {
                match self.slides_copiados.clone() {
                    Some(dados) => {
                        let pos = self.sessao.atual() + 1;
                        match self.sessao.slides_colar(pos, &dados, estilo) {
                            Ok(_) => {
                                self.slides_sel = (pos..pos + dados.quantidade()).collect();
                                self.status = format!("{} slides colados", dados.quantidade());
                            },
                            Err(e) => self.status = e,
                        }
                    }
                    None => self.status = "Primeiro copie ou recorte um slide na barra de miniaturas.".into(),
                }
            }
            FaixaMensagem::SlidesSelecionarTodos => {
                self.slides_sel = (0..self.sessao.apresentacao().slides.len()).collect();
                self.foco_miniaturas = true;
                self.status = format!("{} slides marcados", self.slides_sel.len());
            }
            FaixaMensagem::SlidesRestaurarLayout => {
                match self.sessao.slides_restaurar_layout(&self.alvos_de_slides()) {
                    Ok(_) => self.status = "Layout original dos slides restaurado".into(),
                    Err(e) => self.status = e,
                }
            }
            FaixaMensagem::SlidesImportar => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Reutilizar slides de outra apresentação")
                            .add_filter("Apresentações", &["fsdz", "pptx", "odp"])
                            .pick_file().await.map(|f| f.path().to_path_buf())
                    },
                    Mensagem::ReutilizarArquivoEscolhido,
                );
            }
            FaixaMensagem::SlidesImportarTodos => return self.tratar(Mensagem::Arquivos(crate::arquivos::Acao::Importar)),
            FaixaMensagem::SlideExportarPdf => {
                self.pdf_so_atual = true;
                return self.dialogo_salvar_pdf();
            }
            FaixaMensagem::IrParaNumero => {
                self.ir_numero = Some((self.sessao.atual() + 1).to_string());
            }
            FaixaMensagem::SlideDuplicar => {
                let alvos = self.alvos_de_slides();
                match self.sessao.slides_copiar(&alvos) {
                    Ok(dados) => {
                        let pos = alvos.last().copied().unwrap_or(self.sessao.atual()) + 1;
                        match self.sessao.slides_colar(pos, &dados, featherslide_core::EstiloColagem::ManterOrigem) {
                            Ok(_) => self.slides_sel = (pos..pos + dados.quantidade()).collect(),
                            Err(e) => self.status = e,
                        }
                    }
                    Err(e) => self.status = e,
                }
            }
            FaixaMensagem::SlideExcluir => {
                match self.sessao.slides_excluir(&self.alvos_de_slides()) {
                    Ok(_) => self.slides_sel = [self.sessao.atual()].into(),
                    Err(e) => self.status = e,
                }
            }
            FaixaMensagem::Colar => {
                return Task::perform(
                    async {
                        tokio::task::spawn_blocking(|| {
                            let img = arboard::Clipboard::new().ok()?.get_image().ok()?;
                            let (w, h) = (img.width as u32, img.height as u32);
                            let rgba = image::RgbaImage::from_raw(w, h, img.bytes.into_owned())?;
                            let mut png = std::io::Cursor::new(Vec::new());
                            rgba.write_to(&mut png, image::ImageFormat::Png).ok()?;
                            Some((base64::para_data_url("image/png", &png.into_inner()), w as f32, h as f32))
                        })
                        .await
                        .ok()
                        .flatten()
                    },
                    Mensagem::ColarImagem,
                );
            }
            FaixaMensagem::Copiar => {
                let texto = self.sessao.copiar();
                self.status = "Copiado".into();
                return iced::clipboard::write(texto);
            }
            FaixaMensagem::Recortar => {
                if let Ok((texto, _)) = self.sessao.recortar() {
                    return iced::clipboard::write(texto);
                }
            }
            FaixaMensagem::FonteEscolhida(nome) => {
                let _ = self.sessao.formatar(Formatacao::Fonte(nome));
            }
            FaixaMensagem::TamanhoTexto(t) => {
                self.tamanho_txt = t.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',').collect();
                self.tamanho_editando = true;
            }
            FaixaMensagem::TamanhoAplicar => {
                if let Ok(n) = self.tamanho_txt.replace(',', ".").parse::<f32>() {
                    if (1.0..=400.0).contains(&n) {
                        let _ = self.sessao.formatar(Formatacao::Tamanho(n));
                    }
                }
                self.tamanho_editando = false;
            }
            FaixaMensagem::Duplicar => {
                let _ = self.sessao.selecao_duplicar();
            }
            FaixaMensagem::Excluir => {
                let _ = self.sessao.selecao_excluir();
            }
            FaixaMensagem::SelecionarTudo => {
                if self.foco_miniaturas {
                    self.slides_sel = (0..self.sessao.apresentacao().slides.len()).collect();
                } else {
                    self.sessao.selecionar_tudo();
                }
            }
            FaixaMensagem::IndiceTexto(nivel) => {
                if let Some(id) = self.editando_id {
                    if let Some((a,b)) = cursor::intervalo(&self.texto_edicao,self.ancora_edicao,self.cursor_edicao) {
                        if let Err(e) = self.sessao.texto_indice(id,a,b,nivel) { self.status=e; }
                    } else {
                        // Sem trecho: liga (ou desliga) para o que for digitado a seguir.
                        let atual = self.nivel_da_digitacao();
                        self.indice_digitacao = Some((self.cursor_edicao, if atual == nivel { 0 } else { nivel }));
                    }
                } else {
                    let ids=self.sessao.estado().selecao.clone();
                    for id in ids {if let Some(e)=self.sessao.slide().elementos.iter().find(|e|e.id==id){let fim=e.texto.len();if fim>0 {let _=self.sessao.texto_indice(id,0,fim,nivel);}}}
                }
            },
            FaixaMensagem::EspacamentoLetras(v) => {
                self.menu_fonte = None;
                if let Err(e)=self.sessao.texto_espacamento(v){self.status=e;}
            },
            FaixaMensagem::Negrito => {
                let _ = self.sessao.formatar(Formatacao::Negrito);
            }
            FaixaMensagem::Italico => {
                let _ = self.sessao.formatar(Formatacao::Italico);
            }
            FaixaMensagem::Sublinhado => {
                let _ = self.sessao.formatar(Formatacao::Sublinhado);
            }
            FaixaMensagem::Tachado => {
                let _ = self.sessao.formatar(Formatacao::Tachado);
            }
            FaixaMensagem::AumentarFonte => {
                let _ = self.sessao.formatar(Formatacao::TamanhoPasso(1));
            }
            FaixaMensagem::DiminuirFonte => {
                let _ = self.sessao.formatar(Formatacao::TamanhoPasso(-1));
            }
            FaixaMensagem::LimparFormatacao => {
                let _ = self.sessao.formatar(Formatacao::Limpar);
            }
            FaixaMensagem::CorDoTexto(c) => {
                let _ = self.sessao.formatar(Formatacao::CorDoTexto(c));
            }
            FaixaMensagem::AbrirMenuFonte(m) => {
                self.menu_fonte = if self.menu_fonte == Some(m) { None } else { Some(m) };
            }
            FaixaMensagem::FecharMenuFonte => self.menu_fonte = None,
            FaixaMensagem::SublinhadoDuplo => {
                self.menu_fonte = None;
                let _ = self.sessao.formatar(Formatacao::SublinhadoDuplo);
            }
            FaixaMensagem::TachadoDuplo => {
                self.menu_fonte = None;
                let _ = self.sessao.formatar(Formatacao::TachadoDuplo);
            }
            FaixaMensagem::Caixa(caixa) => {
                self.menu_fonte = None;
                // Em edição com trecho marcado, só o trecho; senão, o texto inteiro.
                let trecho = self.editando_id.and_then(|id| {
                    cursor::intervalo(&self.texto_edicao, self.ancora_edicao, self.cursor_edicao).map(|(a, b)| (id, a, b))
                });
                let antes = self.texto_edicao.len();
                match self.sessao.texto_caixa(trecho, caixa) {
                    Err(e) => self.status = e,
                    Ok(_) => if let Some(id) = self.editando_id {
                        if let Some(e) = self.sessao.slide().elementos.iter().find(|e| e.id == id) {
                            // "ß" → "SS" muda o tamanho: o fim da seleção acompanha.
                            let delta = e.texto.len() as isize - antes as isize;
                            self.texto_edicao = e.texto.clone();
                            match trecho {
                                Some((_, a, b)) => {
                                    let b = b.saturating_add_signed(delta);
                                    let (c, an) = if self.cursor_edicao <= a { (a, b) } else { (b, a) };
                                    self.cursor_edicao = c;
                                    self.ancora_edicao = Some(an);
                                }
                                None => self.cursor_edicao = cursor::limitar(&self.texto_edicao, self.cursor_edicao),
                            }
                        }
                    },
                }
            }
            FaixaMensagem::Marcadores => {
                let _ = self.sessao.lista_alternar("marcadores");
            }
            FaixaMensagem::Numeracao => {
                let _ = self.sessao.lista_alternar("numerada");
            }
            FaixaMensagem::DiminuirRecuo => return self.recuar_texto(false),
            FaixaMensagem::AumentarRecuo => return self.recuar_texto(true),
            FaixaMensagem::Alinhar(a) => {
                let _ = self.sessao.formatar(Formatacao::Alinhar(a));
            }
            FaixaMensagem::AlinharVertical(v) => {
                let _ = self.sessao.formatar(Formatacao::Vertical(v));
            }
            FaixaMensagem::GirarSelecao(delta) => {
                let alvos: Vec<(u64, f32)> = self.sessao.slide().elementos.iter().filter(|e| self.sessao.estado().selecao.contains(&e.id)).map(|e| (e.id, e.rotacao)).collect();
                for (id, atual) in alvos {
                    let _ = self.sessao.elemento_rotacao(id, (atual + delta).rem_euclid(360.0));
                }
            }
            FaixaMensagem::AnguloSetor { fim, delta } => {
                let atual = self.sessao.estado().selecao.first().and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == *id)).map(|e| (e.ajustes.first().copied().unwrap_or(0.0), e.ajustes.get(1).copied().unwrap_or(270.0)));
                if let Some((de, ate)) = atual {
                    let (de, ate) = if fim { (de, (ate + delta).rem_euclid(360.0)) } else { ((de + delta).rem_euclid(360.0), ate) };
                    let _ = self.sessao.selecao_ajustes(vec![de, ate]);
                }
            }
            FaixaMensagem::Inserir(tipo) => {
                self.galeria_figuras = None;
                let _ = self.sessao.elemento_inserir(tipo, None, None);
            }
            FaixaMensagem::InserirGaleria(id) => {
                self.galeria_figuras=None;
                if let Err(e)=self.sessao.galeria_inserir(id){self.status=e;}
            }
            FaixaMensagem::InserirConector(estilo) => {
                self.galeria_figuras=None;
                self.desenho_livre=false;self.editando_id=None;
                let sel=self.sessao.estado().selecao;
                if sel.len()==2 {
                    self.conector_modo=None;self.conector_origem=None;
                    if let Err(e)=self.sessao.conector_inserir(estilo){self.status=e;}
                } else {
                    self.conector_modo=Some(estilo);
                    self.conector_origem=if sel.len()==1{Some(sel[0])}else{None};
                    self.status=if self.conector_origem.is_some(){"Clique na forma de destino. Esc cancela."}else{"Clique na forma de origem e depois no destino. Esc cancela."}.into();
                }
            }
            FaixaMensagem::DesenhoLivre => {
                self.conector_modo=None;self.conector_origem=None;
                self.galeria_figuras=None;self.editando_id=None;self.desenho_livre=true;
                self.status="Arraste no slide para desenhar. Esc cancela.".into();
            }
            FaixaMensagem::InserirTabela(l, c) => {
                let _ = self.sessao.tabela_inserir(l, c);
            }
            FaixaMensagem::InserirIcone(id) => {
                self.galeria_figuras = None;
                if let Err(e) = self.sessao.icone_inserir(&id) {
                    self.status = e;
                }
            }
            FaixaMensagem::InserirGrafico(tipo) => {
                self.galeria_figuras = None;
                let _ = self.sessao.grafico_inserir(&tipo);
            }
            FaixaMensagem::DiagramaInserir(categoria) => {
                self.galeria_figuras = None;
                match self.sessao.diagrama_inserir(categoria) {
                    Ok(_) => {
                        self.status = format!("Diagrama {} inserido — formas e textos editáveis", categoria.rotulo());
                        return self.tratar_faixa(FaixaMensagem::DiagramaEditar);
                    },
                    Err(e) => self.status = e,
                }
            }
            FaixaMensagem::DiagramaEditar => match self.sessao.diagrama_textos_selecionados() {
                Ok(edicao) => {
                    self.editando_id = None; self.editando_celula = None;
                    let selecionado = edicao.nos.first().map(|n| n.id);
                    self.editor_diagrama = Some(crate::editor_diagrama::Estado { edicao, selecionado, aviso: String::new() });
                    return iced::widget::text_input::focus(iced::widget::text_input::Id::new("rotulo_diagrama_0"));
                }, Err(e) => self.status = e,
            },
            FaixaMensagem::InserirWordArt => {
                if let Err(e) = self.sessao.wordart_inserir() { self.status = e; }
                self.aba_faixa = AbaDaFaixa::Imagem;
                self.editando_id = self.sessao.estado().selecao.first().copied();
                self.texto_edicao = "Seu texto".into();
                self.cursor_edicao = self.texto_edicao.len();
                self.ancora_edicao = Some(0);
            }
            FaixaMensagem::WordArtTexto(t) => {
                if let Some(id) = self.sessao.estado().selecao.first().copied() {
                    let _ = self.sessao.texto_definir(id, &t);
                    self.editando_id = None;
                    self.texto_edicao = t;
                    self.ancora_edicao = None;
                }
            }
            FaixaMensagem::WordArtEstilo(i) => { let _ = self.sessao.wordart_estilo(i); }
            FaixaMensagem::WordArtTransformacao(m, a) => { let _ = self.sessao.wordart_transformacao(m, a); }
            FaixaMensagem::WordArtSombra(b) => { let _ = self.sessao.wordart_sombra(b); }
            FaixaMensagem::InserirDataHora => {
                let data = chrono_like_hoje();
                let _ = self.sessao.elemento_inserir(Tipo::Texto, Some([50.0, 500.0, 200.0, 36.0]), None);
                if let Some(&id) = self.sessao.estado().selecao.first() {
                    let _ = self.sessao.texto_definir(id, &data);
                    let _ = self.sessao.formatar(Formatacao::Tamanho(14.0));
                    let _ = self.sessao.formatar(Formatacao::CorDoTexto("#5f6368".into()));
                }
            }
            FaixaMensagem::InserirNumeroSlide => {
                let _ = self.sessao.elemento_inserir(Tipo::Texto, Some([880.0, 500.0, 60.0, 36.0]), None);
                if let Some(&id) = self.sessao.estado().selecao.first() {
                    let _ = self.sessao.texto_definir(id, &format!("{}", self.sessao.atual() + 1));
                    let _ = self.sessao.formatar(Formatacao::Tamanho(14.0));
                    let _ = self.sessao.formatar(Formatacao::CorDoTexto("#5f6368".into()));
                }
            }
            FaixaMensagem::IconeSubstituir(id) => {
                self.galeria_figuras = None;
                if let Err(e) = self.sessao.icone_substituir(&id) { self.status = e; }
            }
            FaixaMensagem::EscalarSelecao(fator) => {
                if let Some(e) = self.sessao.estado().selecao.first().and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == *id)) {
                    let (id, r) = (e.id, [e.x, e.y, e.largura * fator, e.altura * fator]);
                    if let Err(e) = self.sessao.selecao_redimensionar(id, r) { self.status = e; }
                }
            }
            FaixaMensagem::InserirEquacao(e) => {
                let _ = self.sessao.elemento_inserir(Tipo::Texto, Some([180.0, 230.0, 600.0, 80.0]), None);
                if let Some(&id) = self.sessao.estado().selecao.first() {
                    let _ = self.sessao.texto_definir(id, &e);
                    let _ = self.sessao.formatar(Formatacao::Tamanho(32.0));
                    let _ = self.sessao.formatar(Formatacao::Negrito);
                }
            }
            FaixaMensagem::IrParaInicio => return self.tratar(Mensagem::IrParaInicio),
            FaixaMensagem::AbrirGaleria(novo) => self.galeria_layouts = if self.galeria_layouts == Some(novo) { None } else { Some(novo) },
            FaixaMensagem::FecharGaleria => self.galeria_layouts = None,
            FaixaMensagem::EscolherLayout(layout) => {
                if self.galeria_layouts.take() == Some(true) {
                    if let Err(e) = self.sessao.slide_novo(layout) { self.status = e; }
                } else {
                    let alvos = self.alvos_de_slides();
                    if let Err(e) = self.sessao.slides_layout(&alvos, layout) { self.status = e; }
                }
            }
            FaixaMensagem::AbrirConfigModelos => {
                self.config_ui.secao = configuracoes::Secao::Modelos;
                if !self.aberta(Janela::Config) {
                    return self.alternar(Janela::Config);
                }
            }
            FaixaMensagem::AlinharEntreSi(como) => {
                if let Err(e) = self.sessao.alinhar_entre_si(como) {
                    self.status = e;
                }
            }
            FaixaMensagem::Distribuir(eixo) => {
                if let Err(e) = self.sessao.distribuir(eixo) {
                    self.status = e;
                }
            }
            FaixaMensagem::PainelSelecao => self.painel_selecao = !self.painel_selecao,
            FaixaMensagem::AgruparOuDesagrupar => {
                let sel = self.sessao.estado().selecao;
                let ja_agrupado = self.sessao.slide().elementos.iter().any(|e| e.grupo != 0 && sel.contains(&e.id));
                let r = if ja_agrupado { self.sessao.desagrupar() } else { self.sessao.agrupar() };
                match r {
                    Ok(_) => self.status = if ja_agrupado { "Desagrupado".into() } else { "Agrupado".into() },
                    Err(e) => self.status = e,
                }
            }
            FaixaMensagem::Agrupar => {
                if let Err(e) = self.sessao.agrupar() {
                    self.status = e;
                }
            }
            FaixaMensagem::Desagrupar => {
                if let Err(e) = self.sessao.desagrupar() {
                    self.status = e;
                }
            }
            FaixaMensagem::MesclarFormas(op) => {
                self.editando_id = None;
                match self.sessao.formas_booleanas(op) {
                    Ok(_) => self.status = format!("Operação {:?} concluída no motor Rust", op),
                    Err(erro) => self.status = erro,
                }
            }
            FaixaMensagem::OcultarItem(id, oculto) => {
                if let Err(e) = self.sessao.elemento_ocultar(id, oculto) {
                    self.status = e;
                }
            }
            FaixaMensagem::SelecionarItem(id, somar) => {
                self.editando_id = None;
                if somar {
                    self.sessao.alternar_selecao(id);
                } else {
                    self.sessao.selecionar_por_id(id);
                }
            }
            FaixaMensagem::CantoSel(c) => self.canto_sel = c,
            FaixaMensagem::Arredondar(delta) => {
                let primeiro = self.sessao.estado().selecao.first().and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == *id)).map(|e| (e.tipo, e.ajustes.clone()));
                if let Some((tipo, aj)) = primeiro {
                    let n = match tipo {
                        Tipo::Retangulo | Tipo::RetanguloArredondado => 4,
                        Tipo::Triangulo => 3,
                        Tipo::Seta => 7,
                        _ => 0,
                    };
                    if n > 0 {
                        let mut raios: Vec<f32> = if aj.len() == n {
                            aj
                        } else if tipo == Tipo::RetanguloArredondado {
                            let (cima, baixo) = (aj.first().copied().unwrap_or(0.16), aj.get(1).copied().unwrap_or_else(|| aj.first().copied().unwrap_or(0.16)));
                            vec![cima, cima, baixo, baixo]
                        } else {
                            vec![0.0; n]
                        };
                        let alvo = if n == 7 { None } else { self.canto_sel.filter(|&c| c < n) };
                        for (i, r) in raios.iter_mut().enumerate() {
                            if alvo.map_or(true, |c| c == i) {
                                *r = ((*r + delta) * 100.0).round() / 100.0;
                                *r = r.clamp(0.0, 0.5);
                            }
                        }
                        match self.sessao.selecao_ajustes(raios) {
                            Ok(_) => registro::registrar(format!("[forma] arredondar {delta:+} — ponta {:?}", self.canto_sel)),
                            Err(e) => self.status = e,
                        }
                    }
                }
            }
            FaixaMensagem::AbrirFiguras(q) => {
                self.galeria_figuras = if self.galeria_figuras == Some(q) { None } else { Some(q) };
                if self.galeria_figuras == Some(crate::galeria_figuras::Figuras::Icones) {
                    return iced::widget::text_input::focus(iced::widget::text_input::Id::new(crate::galeria_figuras::ID_BUSCA_ICONES));
                }
            }
            FaixaMensagem::BuscaIcones(t) => self.busca_icones = t,
            FaixaMensagem::FecharFiguras => self.galeria_figuras = None,
            FaixaMensagem::RecorteForma(tipo) => {
                self.galeria_figuras = None;
                match self.sessao.elemento_recorte(tipo) {
                    Ok(_) => {
                        self.status = format!("Recorte da imagem: {}", tipo.nome());
                        registro::registrar(format!("[imagem] recorte alterado para {}", tipo.nome()));
                    }
                    Err(e) => self.status = e,
                }
            }
            FaixaMensagem::ImagemAjustarRecorte { dx, dy, zoom } => {
                let _ = self.sessao.imagem_ajustar_recorte(dx, dy, zoom);
            }
            FaixaMensagem::ImagemResetarRecorte => {
                let _ = self.sessao.imagem_resetar_recorte();
                self.status = "Enquadramento da imagem centralizado".into();
            }
            FaixaMensagem::AlternarModoRecorte => {
                if self.ajustando_recorte.is_some() {
                    self.ajustando_recorte = None;
                    self.status = "Modo de enquadramento finalizado".into();
                } else {
                    let sel = self.sessao.estado().selecao.first().copied();
                    if let Some(id) = sel {
                        if let Some(el) = self.sessao.slide().elementos.iter().find(|e| e.id == id) {
                            if el.tipo == featherslide_core::modelo::Tipo::Imagem {
                                self.ajustando_recorte = Some(id);
                                self.status = "Modo de enquadramento ativo: arraste a foto, use a roda do mouse para zoom ou as setas do teclado".into();
                            } else {
                                self.status = "Selecione uma imagem para ajustar o enquadramento".into();
                            }
                        }
                    } else {
                        self.status = "Selecione uma imagem para ajustar o enquadramento".into();
                    }
                }
            }
            FaixaMensagem::CarrosselTemas(d) => self.carrossel_temas = (self.carrossel_temas as i32 + d).max(0) as usize,
            FaixaMensagem::SecaoDosMarcados => return self.tratar(Mensagem::Miniatura(MiniaturaMensagem::Lote(crate::miniaturas::Lote::NovaSecao))),
            FaixaMensagem::MarcarTodosSlides => self.slides_sel = (0..self.sessao.apresentacao().slides.len()).collect(),
            FaixaMensagem::DesmarcarSlides => self.slides_sel = [self.sessao.atual()].into(),
            FaixaMensagem::AbrirSimbolos => self.painel_simbolos = !self.painel_simbolos,
            FaixaMensagem::FecharSimbolos => self.painel_simbolos = false,
            FaixaMensagem::InserirSimbolo(s) => {
                self.painel_simbolos = false;
                let _ = self.sessao.elemento_inserir(Tipo::Texto, Some([300.0, 250.0, 100.0, 80.0]), None);
                if let Some(&id) = self.sessao.estado().selecao.first() {
                    let _ = self.sessao.texto_definir(id, &s);
                    let _ = self.sessao.formatar(Formatacao::Tamanho(48.0));
                    let _ = self.sessao.formatar(Formatacao::CorDoTexto("#1a73e8".into()));
                }
            }
            FaixaMensagem::TabelaEstrutura(acao) => {
                if let Some(&id) = self.sessao.estado().selecao.first() {
                    if let Err(erro) = featherslide_tabelas::slide_editar_estrutura(&mut self.sessao, id, acao) { self.status = erro; }
                }
            }
            FaixaMensagem::EditarDadosGrafico => {
                if let Some(&id) = self.sessao.estado().selecao.first() {
                    if self.sessao.grafico_dados(id).is_some() {
                        self.dados_id = Some(id);
                        if !self.aberta(Janela::Dados) {
                            return self.alternar(Janela::Dados);
                        }
                    }
                }
            }
            FaixaMensagem::TemaAplicar(id) => {
                let _ = self.sessao.tema_aplicar(&id);
            }
            FaixaMensagem::OrtografiaAbrir => {
                self.ferramentas_texto = None;
                if self.ortografia.is_some() {
                    return Task::none();
                }
                self.ortografia = Some(crate::ortografia::Ortografia::nova());
                if self.corretor.is_some() {
                    return self.ortografia_iniciar();
                }
                // Ainda carregando (ou nunca pedido): o painel abre em "Carregando…" e acorda quando chegar.
                if !self.corretor_carregando {
                    self.corretor_carregando = true;
                    return Self::carregar_corretor(self.config.idiomas_ortografia.clone());
                }
            }
            FaixaMensagem::OrtografiaTrocar(nova) => {
                if let (Some(c), Some(e)) = (self.corretor.as_ref(), self.erro_do_menu.take()) {
                    if let Err(msg) = c.trocar(&mut self.sessao, &e, &nova) {
                        self.status = msg;
                    }
                }
            }
            FaixaMensagem::OrtografiaIgnorarTudo => {
                if let (Some(c), Some(e)) = (self.corretor.as_mut(), self.erro_do_menu.take()) {
                    c.ignorar(&e.palavra);
                }
            }
            FaixaMensagem::OrtografiaAdicionar => {
                if let (Some(c), Some(e)) = (self.corretor.as_mut(), self.erro_do_menu.take()) {
                    if let Err(msg) = c.adicionar_ao_dicionario(&e.palavra) {
                        self.status = format!("Não consegui gravar o dicionário pessoal: {msg}");
                    }
                }
            }
            FaixaMensagem::GramaticaAuto => {
                self.config.gramatica_automatica = !self.config.gramatica_automatica;
                self.config.gravar();
            }
            FaixaMensagem::GramaticaIgnorar => {
                if let (Some(c), Some(chave)) = (self.corretor.as_mut(), self.chave_gramatica_do_menu.take()) {
                    c.ignorar_gramatica(&chave);
                }
            }
            FaixaMensagem::OrtografiaAuto => {
                self.config.ortografia_automatica = !self.config.ortografia_automatica;
                self.config.gravar();
            }
            FaixaMensagem::ComentariosExcluirTodos => {
                let _ = self.sessao.comentarios_excluir_todos(true);
                self.status = "Comentários excluídos".into();
            }
            FaixaMensagem::LinkAbrir => {
                if self.sessao.estado().selecao.is_empty() {
                    self.status = "Selecione uma forma, imagem ou caixa de texto para ligar.".into();
                } else {
                    let atual = self.sessao.estado().selecao.first().and_then(|&id| self.sessao.link_do_elemento(self.sessao.atual(), id)).unwrap_or_default();
                    self.revisao = Some(Revisao::Link(atual));
                    return iced::widget::text_input::focus(iced::widget::text_input::Id::new(crate::revisao::ID_CAMPO));
                }
            }
            FaixaMensagem::ComentarioAbrir => {
                self.revisao = Some(Revisao::Comentario(String::new()));
                return iced::widget::text_input::focus(iced::widget::text_input::Id::new(crate::revisao::ID_CAMPO));
            }
            FaixaMensagem::SecaoNova => {
                let nome = self.sessao.secao_nome_sugerido();
                match self.sessao.secao_adicionar(&nome) {
                    Ok(_) => {
                        // Criar outra seção no mesmo cabeçalho insere slide em branco;
                        // abrir a edição na seção realmente criada.
                        let inicio = self.sessao.atual();
                        self.slides_sel = [inicio].into();
                        self.secoes_recolhidas.clear();
                        self.renomeando_secao = Some((inicio, nome));
                        self.status = "Seção criada. Digite o nome e pressione Enter.".into();
                        return iced::widget::text_input::focus(
                            iced::widget::text_input::Id::new(ID_RENOMEAR)
                        );
                    }
                    Err(e) => self.status = e,
                }
            },
            FaixaMensagem::SecaoRenomear(inicio) => {
                let nome = self.sessao.secoes().into_iter().find(|s| s.inicio == inicio).map(|s| s.nome).unwrap_or_default();
                self.renomeando_secao = Some((inicio, nome));
                return iced::widget::text_input::focus(iced::widget::text_input::Id::new(ID_RENOMEAR));
            }
            FaixaMensagem::SecaoMover(inicio, direcao) => {
                match self.sessao.secao_mover(inicio, direcao) {
                    Ok(_) => {
                        self.slides_sel = [self.sessao.atual()].into();
                        self.secoes_recolhidas.clear();
                        self.status = "Seção movida com todos os seus slides".into();
                    }
                    Err(e) => self.status = e,
                }
            }
            FaixaMensagem::SlidesParaSecao(inicio) => {
                let alvos = self.alvos_de_slides();
                match self.sessao.slides_para_secao(&alvos, inicio) {
                    Ok(_) => {
                        let primeiro = self.sessao.atual();
                        self.slides_sel = (primeiro..primeiro + alvos.len()).collect();
                        self.secoes_recolhidas.clear();
                        self.status = format!("{} slides transferidos para a seção", alvos.len());
                    }
                    Err(e) => self.status = e,
                }
            }
            FaixaMensagem::SecaoRemover(inicio, com_slides) => {
                if let Err(e) = self.sessao.secao_remover(inicio, com_slides) {
                    self.status = e;
                }
                self.secoes_recolhidas.clear();
            }
            FaixaMensagem::SecoesRemoverTodas => {
                let _ = self.sessao.secoes_remover_todas();
                self.secoes_recolhidas.clear();
            }
            FaixaMensagem::SecoesRecolherTodas => {
                self.secoes_recolhidas = self.sessao.secoes().into_iter().map(|s| s.inicio).collect();
            }
            FaixaMensagem::SecoesExpandirTodas => self.secoes_recolhidas.clear(),
            FaixaMensagem::SecoesAlternar(i) => {
                if !self.secoes_recolhidas.remove(&i) {
                    self.secoes_recolhidas.insert(i);
                }
            }
            FaixaMensagem::SecaoDoSlideAtual(acao) => {
                // Renomear/remover a seção que contém o slide atual.
                let atual = self.sessao.atual();
                if let Some(s) = self.sessao.secoes().into_iter().find(|s| s.inicio <= atual && atual < s.fim) {
                    let msg = if acao == "renomear" { FaixaMensagem::SecaoRenomear(s.inicio) } else { FaixaMensagem::SecaoRemover(s.inicio, false) };
                    return self.tratar_faixa(msg);
                }
                self.status = "Este slide não está em nenhuma seção.".into();
            }
            FaixaMensagem::Organizar(o) => {
                let _ = self.sessao.organizar(o);
            }
            FaixaMensagem::AlinharNoSlide(como) => {
                let _ = self.sessao.alinhar_no_slide(como);
            }
            FaixaMensagem::SlideMover(delta) => {
                let de = self.sessao.atual();
                let para = (de as i32 + delta).max(0) as usize;
                let _ = self.sessao.slide_mover(de, para);
            }
            FaixaMensagem::FundoLiberar => match self.sessao.fundo_liberar(false) {
                Ok(_) => self.status = "Formas do fundo soltas: agora dá para selecionar e mover".into(),
                Err(e) => self.status = e,
            },
            FaixaMensagem::SelecaoParaFundo => {
                let _ = self.sessao.selecao_para_fundo();
            }
            FaixaMensagem::FundoTodos(_cor) => {
                let _ = self.sessao.fundo_aplicar_todos();
            }
            FaixaMensagem::PreviaEncerrar => {}, // Sair do botão não interrompe o efeito em curso.
            FaixaMensagem::PreviaTransicao(tipo) => {
                self.previa = None;
                if tipo != featherslide_core::modelo::TipoTransicao::Nenhuma && !self.em_apresentacao {
                    if let Some(q) = self.quadro_em_cache(self.sessao.atual()) {
                        let mut de = self.sessao.atual().checked_sub(1).and_then(|i| self.quadro_em_cache(i)).unwrap_or_else(|| q.clone());
                        if self.sessao.atual() == 0 { de.elementos.clear(); }
                        let mut tr = self.sessao.slide().transicao.clone();
                        tr.tipo = tipo;
                        self.previa = Some(crate::previa::Previa::nova(q, crate::previa::Efeito::Transicao(de, tr)));
                    }
                }
            }
            FaixaMensagem::PreviaAnimacao(tipo) => {
                self.previa = None;
                let ids = self.sessao.estado().selecao;
                if tipo != featherslide_core::modelo::TipoAnimacao::Nenhuma && !ids.is_empty() && !self.em_apresentacao {
                    if let Some(q) = self.quadro_em_cache(self.sessao.atual()) {
                        let mut a = self.animacao_da_selecao().unwrap_or_default();
                        a.tipo = tipo;
                        a.atraso = 0.0;
                        self.previa = Some(crate::previa::Previa::nova(q, crate::previa::Efeito::Animacao(ids, a)));
                    }
                }
            }
            FaixaMensagem::TransicaoTipo(t) => {
                self.previa = None;
                let mut tr = self.sessao.slide().transicao.clone();
                tr.tipo = t;
                let _ = self.sessao.transicao_definir(tr, false);
                return self.tratar_faixa(FaixaMensagem::PreviaTransicao(t));
            }
            FaixaMensagem::TransicaoDirecao(d) => {
                let mut tr = self.sessao.slide().transicao.clone();
                tr.direcao = d;
                let _ = self.sessao.transicao_definir(tr, false);
            }
            FaixaMensagem::TransicaoDuracao(delta) => {
                let mut tr = self.sessao.slide().transicao.clone();
                tr.duracao = ((tr.duracao + delta) * 10.0).round() / 10.0;
                tr.duracao = tr.duracao.clamp(0.1, 10.0);
                let _ = self.sessao.transicao_definir(tr, false);
            }
            FaixaMensagem::TransicaoApos(delta) => {
                let mut tr = self.sessao.slide().transicao.clone();
                tr.apos = (tr.apos + delta).clamp(0.0, 120.0);
                let _ = self.sessao.transicao_definir(tr, false);
            }
            FaixaMensagem::TransicaoTodos => {
                let tr = self.sessao.slide().transicao.clone();
                let _ = self.sessao.transicao_definir(tr, true);
                self.status = "Transição aplicada a todos os slides".into();
            }
            FaixaMensagem::AnimTipo(t) => {
                self.previa = None;
                self.mudar_animacao(|a| a.tipo = t);
                return self.tratar_faixa(FaixaMensagem::PreviaAnimacao(t));
            },
            FaixaMensagem::AnimInicio(i) => self.mudar_animacao(|a| a.inicio = i),
            FaixaMensagem::AnimDirecao(d) => self.mudar_animacao(|a| a.direcao = d),
            FaixaMensagem::AnimDuracao(delta) => self.mudar_animacao(|a| a.duracao = (((a.duracao + delta) * 10.0).round() / 10.0).clamp(0.1, 20.0)),
            FaixaMensagem::AnimAtraso(delta) => self.mudar_animacao(|a| a.atraso = (((a.atraso + delta) * 10.0).round() / 10.0).clamp(0.0, 30.0)),
            FaixaMensagem::AnimOrdem(delta) => {
                if let Some(&id) = self.sessao.estado().selecao.first() {
                    let _ = self.sessao.animacao_ordem(id, delta);
                }
            }
            FaixaMensagem::AbrirPainelAnimacao => return self.alternar(Janela::Animacoes),
            FaixaMensagem::OcultarSlide => {
                if let Err(e) = self.sessao.slides_alternar_ocultacao(&self.alvos_de_slides()) {
                    self.status = e;
                }
            }
            FaixaMensagem::Repetir => self.repetir = !self.repetir,
            FaixaMensagem::MostrarPainel => {
                self.config.painel_slides = !self.config.painel_slides;
                self.config.gravar();
            }
            FaixaMensagem::MostrarNotas => {
                self.config.painel_notas = !self.config.painel_notas;
                self.config.gravar();
            }
            FaixaMensagem::Grade => self.grade = !self.grade,
            FaixaMensagem::Guias => self.guias = !self.guias,
            FaixaMensagem::Regua => self.regua = !self.regua,
            FaixaMensagem::Classificacao => self.classificacao = true,
            FaixaMensagem::ExibirNormal => self.classificacao = false,
            FaixaMensagem::FormatoForma => {
                self.formato_forma = self.sessao.estado().selecao.first()
                    .and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == *id))
                    .filter(|e| featherslide_core::estilo_forma::aceita(e.tipo))
                    .map(crate::formato_forma::Estado::novo);
            },
            FaixaMensagem::Preenchimento(cor) => {
                let _ = self.sessao.formatar(Formatacao::Preenchimento(cor));
            }
            FaixaMensagem::AbrirSeletorCor(alvo) => {
                self.conta_gotas = None;
                self.sessao.confirmar_previa();
                if self.seletor_cor == Some(alvo) {
                    self.seletor_cor = None;
                } else {
                    self.seletor_cor = Some(alvo);
                    self.texto_cor = self.cor_do_alvo(alvo);
                    self.hsva_cor = crate::seletor_cor::rgba_para_hsva(crate::seletor_cor::rgba_de(&self.texto_cor), 0.0);
                }
            }
            FaixaMensagem::FecharSeletorCor => {
                self.seletor_cor = None;
                self.conta_gotas = None;
                self.sessao.confirmar_previa();
            }
            FaixaMensagem::SeletorSV(alvo, s, v) => {
                self.hsva_cor[1] = s;
                self.hsva_cor[2] = v;
                self.aplicar_hsva(alvo);
            }
            FaixaMensagem::SeletorMatiz(alvo, h) => {
                self.hsva_cor[0] = h;
                self.aplicar_hsva(alvo);
            }
            FaixaMensagem::SeletorAlfa(alvo, a) => {
                self.hsva_cor[3] = a;
                self.aplicar_hsva(alvo);
            }
            FaixaMensagem::ConteGotas(alvo) => {
                self.conta_gotas = if self.conta_gotas == Some(alvo) { None } else { Some(alvo) };
                if self.conta_gotas.is_some() {
                    self.status = "Conta-gotas: clique no slide para pegar a cor".into();
                }
            }
            FaixaMensagem::TextoCanal(alvo, canal, texto) => {
                if let Ok(v) = texto.trim().parse::<u8>() {
                    let mut c = crate::seletor_cor::rgba_de(&self.cor_do_alvo(alvo));
                    c[canal.min(3)] = v;
                    self.aplicar_cor(alvo, crate::seletor_cor::hex_de(c));
                }
            }
            FaixaMensagem::CanalCor(alvo, canal, valor) => {
                let mut c = crate::seletor_cor::rgba_de(&self.cor_do_alvo(alvo));
                c[canal.min(3)] = valor;
                self.aplicar_cor(alvo, crate::seletor_cor::hex_de(c));
            }
            FaixaMensagem::CorEscolhida(alvo, hex) => self.aplicar_cor(alvo, hex),
            FaixaMensagem::TextoCor(alvo, texto) => {
                self.texto_cor = texto.clone();
                if crate::seletor_cor::hex_valido(&texto) {
                    let hex = format!("#{}", texto.trim().trim_start_matches('#').to_lowercase());
                    self.hsva_cor = crate::seletor_cor::rgba_para_hsva(crate::seletor_cor::rgba_de(&hex), self.hsva_cor[0]);
                    let _ = self.sessao.formatar_previa(Self::formatacao_da_cor(alvo, hex));
                }
            }
            FaixaMensagem::EspessuraContorno(w) => {
                let _ = self.sessao.formatar(Formatacao::Espessura(w));
            }
            FaixaMensagem::Contorno(cor) => {
                let _ = self.sessao.formatar(Formatacao::Contorno(cor));
            }
            FaixaMensagem::ConfiguracaoSlide => { self.configuracao_slide = Some(crate::configuracao_slide::Estado::novo(&self.sessao)); }
            FaixaMensagem::Proporcao(p) => {
                let _ = self.sessao.proporcao_definir(p);
            }
            FaixaMensagem::FundoSlide(cor) => {
                let _ = self.sessao.slide_fundo(&cor, false);
            }
            FaixaMensagem::PersonalizadasAbrir => {
                self.personalizadas = Some(personalizadas::Estado::default());
            }
            FaixaMensagem::Apresentar => {
                self.roteiro_personalizado = None;
                return self.comecar_apresentacao(true);
            }
            FaixaMensagem::ApresentarDoAtual => {
                self.roteiro_personalizado = None;
                return self.comecar_apresentacao(false);
            }
            FaixaMensagem::ApresentarComNotas => {
                self.roteiro_personalizado = None;
                self.modo_apresentador = true;
                return self.comecar_apresentacao(false);
            }
            FaixaMensagem::ZoomMais => {
                self.zoom = (self.zoom + 0.1).min(4.0);
            }
            FaixaMensagem::ZoomMenos => {
                self.zoom = (self.zoom - 0.1).max(0.2);
                if self.zoom <= 1.0 {
                    self.pan = (0.0, 0.0);
                }
            }
            FaixaMensagem::ZoomReset => {
                self.zoom = 1.0;
                self.pan = (0.0, 0.0);
            }
            FaixaMensagem::InserirImagem => {
                return Task::perform(
                    async {
                        let caminho = rfd::AsyncFileDialog::new()
                            .add_filter("Imagens", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
                            .pick_file()
                            .await
                            .map(|f| f.path().to_path_buf())?;
                        tokio::task::spawn_blocking(move || {
                            let bytes = std::fs::read(&caminho).ok()?;
                            let ext = caminho.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                            let tipo = match ext.as_str() {
                                "jpg" | "jpeg" => "image/jpeg",
                                "gif" => "image/gif",
                                "webp" => "image/webp",
                                "bmp" => "image/bmp",
                                _ => "image/png",
                            };
                            // O tamanho natural, para a imagem nascer na proporção certa.
                            let (w, h) = image::load_from_memory(&bytes).map(|i| (i.width() as f32, i.height() as f32)).unwrap_or((4.0, 3.0));
                            Some((base64::para_data_url(tipo, &bytes), w, h))
                        })
                        .await
                        .ok()
                        .flatten()
                    },
                    Mensagem::ImagemLida,
                );
            }
            FaixaMensagem::InserirSvg | FaixaMensagem::ConverterSvg | FaixaMensagem::ExportarSvg | FaixaMensagem::ExportarGraficoSvg => return self.tratar_vetores(msg),
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Mensagem> {
        Subscription::batch([
            iced::event::listen_with(|ev, estado, id| match ev {
                // Tecla que um campo de texto já usou (digitando na pesquisa, renomeando seção) não vira atalho.
                Event::Keyboard(_) if estado == iced::event::Status::Captured => None,
                Event::Keyboard(_) => Some(Mensagem::TecladoJanela(ev, id)),
                Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => Some(Mensagem::SoltouMouse),
                Event::Mouse(iced::mouse::Event::CursorMoved { position }) => Some(Mensagem::CursorMoveu(position)),
                Event::Window(window::Event::FileDropped(p)) => Some(Mensagem::ArquivoEscolhidoParaAbrir(Some(p))),
                _ => None,
            }),
            window::close_requests().map(Mensagem::FecharJanela),
            window::close_events().map(Mensagem::JanelaFechada),
            window::resize_events().map(|(id, tam)| Mensagem::JanelaRedimensionada(id, tam)),
            // O tique de 1 s só existe quando alguém usa: salvamento automático ou o Monitor aberto.
            if self.arquivos.ativo || !self.arquivos.abertos.is_empty() || self.config.auto_salvar || self.aberta(Janela::Monitor) || self.modo_apresentador {
                iced::time::every(std::time::Duration::from_secs(1)).map(|_| Mensagem::Tique)
            } else {
                Subscription::none()
            },
            if self.passo_captura < 2 && std::env::var_os("FEATHERSLIDE_CAPTURAR").is_some() {
                iced::time::every(std::time::Duration::from_millis(1500)).map(|_| Mensagem::CapturarTela)
            } else { Subscription::none() },
            if self.previa.is_some() {
                iced::time::every(std::time::Duration::from_millis(16)).map(|_| Mensagem::QuadroPrevia)
            } else { Subscription::none() },
            if self.em_apresentacao {
                iced::time::every(std::time::Duration::from_millis(30)).map(|_| Mensagem::QuadroApresentacao)
            } else {
                Subscription::none()
            },
        ])
    }

    pub fn view(&self, janela: window::Id) -> Element<'_, Mensagem> {
        if self.tipo_de(janela) == Some(Janela::Plateia)
            || (self.tipo_de(janela).is_none() && self.em_apresentacao)
        {
            return self.view_conteudo(janela);
        }
        feather_ui::cromo::moldura_janela(self.view_conteudo(janela))
    }

    fn view_conteudo(&self, janela: window::Id) -> Element<'_, Mensagem> {
        match self.tipo_de(janela) {
            Some(Janela::Monitor) => return self.com_barra_de_janela(janela, self.monitor.view().map(Mensagem::MonitorMsg)),
            Some(Janela::Dados) => return self.com_barra_de_janela(janela, dados_janela::view(&self.sessao, self.dados_id).map(Mensagem::DadosMsg)),
            Some(Janela::Animacoes) => return self.com_barra_de_janela(janela, painel_animacao::view(&self.sessao).map(Mensagem::AnimacaoMsg)),
            Some(Janela::Plateia) => return self.view_apresentacao(false),
            Some(Janela::Ajuda) => return self.com_barra_de_janela(janela, self.ajuda.view().map(Mensagem::AjudaMsg)),
            Some(Janela::Config) => {
                let proporcao = self.sessao.apresentacao().proporcao;
                let arquivo = self.caminho_arquivo.as_ref().map(|_| self.nome_arquivo.as_str());
                let tela = self.config_ui.view(&self.config, proporcao, arquivo, configuracoes::Extras { grade: self.grade, guias: self.guias, repetir: self.repetir, dois_monitores: self.config.dois_monitores, monitores: self.n_monitores }, &self.meus_modelos).map(Mensagem::ConfigMsg);
                return self.com_barra_de_janela(janela, tela);
            }
            None => {}
        }
        let inicio = std::time::Instant::now();
        let tela = self.view_editor();
        let ms = inicio.elapsed().as_secs_f64() * 1000.0;
        if ACAO_PENDENTE.swap(false, std::sync::atomic::Ordering::Relaxed) || ms >= 30.0 {
            registro::registrar(format!("[quadro] montar a tela — {ms:.1} ms"));
        }
        let tela = if self.painel_simbolos { crate::simbolos::sobrepor(tela, self.tamanho_janela.width) } else { tela };
        let tela = match &self.reutilizacao {
            Some(e) => reutilizar_slides::sobrepor(tela, e, self.tamanho_janela.width),
            None => tela,
        };
        let tela = match &self.personalizadas {
            Some(p) => personalizadas::sobrepor(tela, p, &self.sessao, &self.slides_sel, self.tamanho_janela.width),
            None => tela,
        };
        let tela = match &self.ir_numero {
            Some(numero) => {
                let fundo = iced::widget::mouse_area(Space::new(Length::Fill, Length::Fill))
                    .on_press(Mensagem::NumeroSlideFechar);
                let painel = container(column![
                    text("Ir para o slide").size(16),
                    text(format!("Digite um número entre 1 e {}.", self.sessao.apresentacao().slides.len())).size(12),
                    text_input("Número do slide", numero)
                        .on_input(Mensagem::NumeroSlideTexto)
                        .on_submit(Mensagem::NumeroSlideConfirmar)
                        .size(14).padding([7, 10]),
                    row![
                        button(text("Cancelar")).on_press(Mensagem::NumeroSlideFechar).style(estilo_botao_secundario),
                        button(text("Ir")).on_press(Mensagem::NumeroSlideConfirmar).style(estilo_botao_primario),
                    ].spacing(9),
                ].spacing(12))
                    .width(Length::Fixed(320.0))
                    .padding(20)
                    .style(|_| container::Style {
                        background: Some(Background::Color(COR_PAINEL())),
                        border: Border { color: COR_BORDA(), width: 1.0, radius: Radius::from(8.0) },
                        ..Default::default()
                    });
                let esquerda = ((self.tamanho_janela.width - 320.0) / 2.0).max(8.0);
                let sobreposto = container(painel).padding(iced::Padding { top: 140.0, left: esquerda, right: 0.0, bottom: 0.0 });
                iced::widget::stack![tela, fundo, sobreposto].into()
            },
            None => tela,
        };
        let tela = if self.painel_imagem {
            if let Some(e) = self.sessao.estado().selecao.first()
                .and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id==*id && e.tipo==Tipo::Imagem))
            {
                let margens = [
                    e.ajustes.get(4).copied().unwrap_or(0.0),
                    e.ajustes.get(5).copied().unwrap_or(0.0),
                    e.ajustes.get(6).copied().unwrap_or(0.0),
                    e.ajustes.get(7).copied().unwrap_or(0.0),
                ];
                crate::imagem_painel::sobrepor(tela, &e.visual_imagem,
                    &self.imagem_alt_rascunho, !e.sombra.cor.is_empty(),
                    margens, self.tamanho_janela.width,
                    self.tamanho_janela.height, self.pagina_imagem)
            } else { tela }
        } else { tela };
        let tela = match &self.configuracao_slide { Some(e) => crate::configuracao_slide::sobrepor(tela, e), None => tela };
        let tela = match &self.ferramentas_texto { Some(e) => crate::ferramentas_texto::sobrepor(tela, e, &self.config), None => tela };
        let tela = match &self.propriedades {
            Some(p) => propriedades::sobrepor(tela, p, self.sessao.estatisticas(), self.tamanho_janela.width),
            None => tela,
        };
        let tela = match &self.impressao {
            Some(i) => crate::imprimir::sobrepor(tela, i, self.sessao.apresentacao().slides.iter().any(|s| s.oculto), self.tamanho_janela.width),
            None => tela,
        };
        let tela = match &self.ortografia {
            Some(o) => crate::ortografia::sobrepor(tela, o, self.tamanho_janela.width),
            None => tela,
        };
        let tela = match &self.revisao {
            Some(r) => {
                let tem_link = self.sessao.estado().selecao.first().is_some_and(|&id| self.sessao.link_do_elemento(self.sessao.atual(), id).is_some());
                crate::revisao::sobrepor(tela, r, &self.sessao.slide().comentarios, tem_link, self.tamanho_janela.width)
            }
            None => tela,
        };
        let tela = match self.menu_fonte {
            Some(m) => {
                let esp = self.sessao.estado().selecao.first()
                    .and_then(|id| self.sessao.slide().elementos.iter().find(|e| e.id == *id))
                    .map_or(0.0, |e| e.espacamento_letras);
                let f = self.sessao.estado().formato;
                let duplos = f.as_ref().map_or((false, false), |f| (f.sublinhado_duplo, f.tachado_duplo));
                crate::menu_fonte::sobrepor(tela, m, esp, duplos, self.tamanho_janela.width)
            }
            None => tela,
        };
        let tela = match self.seletor_cor {
            Some(alvo) => crate::seletor_cor::sobrepor(tela, alvo, &self.cor_do_alvo(alvo), self.hsva_cor, &self.texto_cor, self.conta_gotas == Some(alvo), self.tamanho_janela.width),
            None => tela,
        };
        let tela = match self.galeria_figuras {
            Some(q) => crate::galeria_figuras::sobrepor(tela, q, &self.busca_icones, self.tamanho_janela.width),
            None => tela,
        };
        let tela = match self.galeria_layouts {
            Some(novo) => crate::galeria_layouts::sobrepor(tela, novo, &self.sessao, self.tamanho_janela.width),
            None => tela,
        };
        let tela = match &self.menu {
            Some(m) => self.com_menu(tela, m),
            None => tela,
        };
        if let (Some(idx), Some(pos)) = (self.arrastando_slide, self.cursor_arrasto) {
            self.com_arrasto_slide(tela, idx, pos)
        } else if let (Some(inicio), Some(primeiro), Some(pos)) =
            (self.arrastando_secao, self.inicio_arrasto_secao, self.cursor_arrasto)
        {
            // A prévia só aparece depois de arrastar mais de 6px; um clique
            // simples no cabeçalho continua recolhendo/expandindo a seção.
            if (pos.x - primeiro.x).abs() > 6.0 || (pos.y - primeiro.y).abs() > 6.0 {
                self.com_arrasto_secao(tela, inicio, pos)
            } else {
                tela
            }
        } else {
            tela
        }
    }

    /// Classificação de slides: todos os slides em grade; clique escolhe, clique no atual volta ao editor.
    fn view_classificacao(&self) -> Element<'_, Mensagem> {
        let (mut cartoes, atual): (Vec<Element<'_, Mensagem>>, usize) = (Vec::new(), self.sessao.atual());
        for (i, _) in self.sessao.apresentacao().slides.iter().enumerate() {
            let quadro = self.quadro_em_cache(i);
            let miniatura = Element::from(canvas(Miniatura::nova(quadro, None, self.sessao.apresentacao().slides[i].oculto)).width(Length::Fixed(224.0)).height(Length::Fixed(126.0))).map(Mensagem::Miniatura);
            let oculto = self.sessao.apresentacao().slides[i].oculto;
            let cartao = button(
                column![
                    miniatura,
                    text(format!("{}{}", i + 1, if oculto { "  (oculto)" } else { "" })).size(12).color(COR_TEXTO_MUTED()),
                ]
                .spacing(5)
                .align_x(Alignment::Center),
            )
            .padding(8)
            .width(Length::Fixed(244.0))
            .height(Length::Fixed(172.0))
            .style(move |_, _| button::Style {
                background: Some(Background::Color(COR_PAINEL())),
                text_color: COR_TEXTO(),
                border: Border { color: if i == atual { COR_AZUL_ACENTO() } else { COR_BORDA_SUAVE() }, width: if i == atual { 2.5 } else { 1.0 }, radius: Radius::from(8.0) },
                shadow: iced::Shadow::default(),
            })
            .on_press(Mensagem::ClassificacaoClique(i));
            cartoes.push(cartao.into());
        }
        let largura_util = self.tamanho_janela.width - if self.config.painel_slides { 190.0 } else { 0.0 } - 70.0;
        let por_linha = ((largura_util / 262.0).floor() as usize).max(1);
        let mut grade = column![].spacing(18);
        let mut restante = cartoes.into_iter().peekable();
        while restante.peek().is_some() {
            let mut linha = row![].spacing(18);
            for cartao in restante.by_ref().take(por_linha) {
                linha = linha.push(cartao);
            }
            grade = grade.push(linha);
        }
        container(column![
            container(row![icone_classificacao(20.0, "#5f6368"),text("Todos os slides").size(19),text(format!("{} slides",self.sessao.apresentacao().slides.len())).size(12).color(COR_TEXTO_MUTED()),Space::with_width(Length::Fill),button(text("Voltar ao editor").size(12)).padding([6,12]).style(estilo_botao_secundario).on_press(Mensagem::Faixa(FaixaMensagem::ExibirNormal))].spacing(10).align_y(Alignment::Center)).padding([14,20]).style(estilo_container_topo),
            crate::tema::rolagem(container(grade).padding(20)).height(Length::Fill),
        ]).width(Length::Fill).height(Length::Fill).style(|_| container::Style {
            background: Some(Background::Color(COR_MESA())),
            ..container::Style::default()
        }).into()
    }

    /// A tela com o menu do botão direito por cima; clicar fora dele (ou com o direito) fecha.
    fn com_menu<'a>(&'a self, base: Element<'a, Mensagem>, m: &MenuCtx) -> Element<'a, Mensagem> {
        let (largura, altura) = (self.tamanho_janela.width, self.tamanho_janela.height);
        let x = m.x.min(largura - menu_contexto::LARGURA - 8.0).max(4.0);
        let y = m.y.min(altura - menu_contexto::altura(&m.tipo) - 8.0).max(4.0);
        let fundo = iced::widget::mouse_area(Space::new(Length::Fill, Length::Fill))
            .on_press(Mensagem::FecharMenu)
            .on_right_press(Mensagem::FecharMenu);
        let menu = container(menu_contexto::view(&m.tipo).map(Mensagem::MenuAcao)).padding(iced::Padding { top: y, left: x, right: 0.0, bottom: 0.0 });
        iced::widget::stack![base, fundo, menu].into()
    }

    /// Desenha a prévia flutuante da miniatura do slide acompanhando o cursor enquanto ele é arrastado.
    fn com_arrasto_slide<'a>(&'a self, base: Element<'a, Mensagem>, slide_idx: usize, cursor: Point) -> Element<'a, Mensagem> {
        let (largura, altura) = (self.tamanho_janela.width, self.tamanho_janela.height);
        let larg_card = 136.0;
        let alt_card = 76.5;

        // Posicionado ligeiramente à frente do cursor (14px) para não cobrir o ponteiro do mouse
        // e permitir que o evento de soltar chegue com precisão aos elementos por baixo.
        let x = (cursor.x + 14.0).min(largura - larg_card - 8.0).max(4.0);
        let y = (cursor.y + 14.0).min(altura - alt_card - 28.0).max(4.0);

        let quadro = self.quadro_em_cache(slide_idx);
        let oculto = self.sessao.apresentacao().slides.get(slide_idx).map_or(false, |s| s.oculto);

        let preview = container(
            Element::from(
                canvas::Canvas::new(Miniatura::nova(quadro, None, oculto))
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .map(|_: MiniaturaMensagem| Mensagem::LimparArraste),
        )
        .width(Length::Fixed(larg_card))
        .height(Length::Fixed(alt_card))
        .style(|_| container::Style {
            background: Some(Background::Color(Color::WHITE)),
            border: Border {
                color: COR_AZUL_ACENTO(),
                width: 2.0,
                radius: Radius::from(5.0),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.40),
                offset: iced::Vector::new(3.0, 5.0),
                blur_radius: 10.0,
            },
            ..Default::default()
        });

        let etiqueta = container(
            text(format!("Slide {}", slide_idx + 1))
                .size(10)
                .color(Color::WHITE),
        )
        .padding([2, 6])
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgba8(26, 115, 232, 0.95))),
            border: Border {
                radius: Radius::from(3.0),
                ..Default::default()
            },
            ..Default::default()
        });

        let cartao = column![preview, Space::with_height(3), etiqueta];
        let flutuante = container(cartao).padding(iced::Padding {
            top: y,
            left: x,
            right: 0.0,
            bottom: 0.0,
        });

        iced::widget::stack![base, flutuante].into()
    }

    /// Prévia flutuante da seção arrastada, deslocada do ponteiro para
    /// permitir soltar sobre o cabeçalho/miniatura sem capturar o evento.
    fn com_arrasto_secao<'a>(&'a self, base: Element<'a, Mensagem>, inicio: usize, cursor: Point) -> Element<'a, Mensagem> {
        let secoes = self.sessao.secoes();
        let Some(secao) = secoes.iter().find(|s| s.inicio == inicio) else { return base };
        let largura = 172.0;
        let altura = 38.0;
        let x = (cursor.x + 15.0).min(self.tamanho_janela.width - largura - 8.0).max(4.0);
        let y = (cursor.y + 15.0).min(self.tamanho_janela.height - altura - 26.0).max(4.0);
        let cartao = container(
            column![
                text(secao.nome.clone()).size(12).color(COR_TEXTO()),
                text(format!("{} slide(s)", secao.fim - secao.inicio)).size(10).color(COR_TEXTO_MUTED()),
            ].spacing(2)
        )
        .padding([5, 9])
        .width(Length::Fixed(largura))
        .style(|_| container::Style {
            background: Some(Background::Color(COR_PAINEL())),
            border: Border { color: COR_AZUL_ACENTO(), width: 2.0, radius: Radius::from(5.0) },
            shadow: iced::Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.25),
                offset: iced::Vector::new(2.0, 3.0),
                blur_radius: 7.0,
            },
            ..Default::default()
        });
        let flutuante = container(cartao).padding(iced::Padding {
            top: y, left: x, right: 0.0, bottom: 0.0,
        });
        iced::widget::stack![base, flutuante].into()
    }

    /// Gera a imagem PNG do slide em base64 (data URL) para uso como visão/gatilho.
    fn gerar_visao_slide(&self, slide_idx: usize) -> Option<String> {
        let q = self.quadro_em_cache(slide_idx).or_else(|| self.sessao.quadro(slide_idx, None, false).ok().flatten())?;
        let fonte = featherslide_mcp::fonte::carregar();
        let bytes = featherslide_mcp::renderizar::renderizar_png(&q, 480, false, &fonte).ok()?;
        Some(featherslide_io::base64::para_data_url("image/png", &bytes))
    }

    /// Com o corretor pronto, faz a verificação e mostra o primeiro erro.
    fn ortografia_iniciar(&mut self) -> Task<Mensagem> {
        let (Some(o), Some(c)) = (self.ortografia.as_mut(), self.corretor.as_ref()) else { return Task::none() };
        o.carregando = false;
        o.atualizar(c, self.sessao.apresentacao());
        self.ortografia_mostrar();
        iced::widget::text_input::focus(iced::widget::text_input::Id::new(crate::ortografia::ID_CAMPO))
    }

    /// Leva a tela ao slide e ao elemento do erro atual.
    fn ortografia_mostrar(&mut self) {
        let alvo = self.ortografia.as_ref().and_then(|o| o.atual().map(|e| (e.slide, e.elemento, e.campo)));
        if let Some((slide, elemento, campo)) = alvo {
            let _ = self.sessao.slide_ir(slide);
            self.editando_id = None; self.editando_celula = None;
            if campo == featherslide_ortografia::CampoErro::Notas { self.config.painel_notas = true; }
            else { let _ = self.sessao.selecionar_por_id(elemento); }
        }
    }

    fn tratar_ortografia(&mut self, m: crate::ortografia::OrtMsg) -> Task<Mensagem> {
        use crate::ortografia::OrtMsg as O;
        let (Some(o), Some(c)) = (self.ortografia.as_mut(), self.corretor.as_mut()) else {
            if matches!(m, O::Fechar) {
                self.ortografia = None;
            }
            return Task::none();
        };
        match m {
            O::Texto(t) => o.correcao = t,
            O::Sugestao(s) => o.correcao = s,
            O::Fechar => {
                self.ortografia = None;
                return Task::none();
            }
            O::Alterar => {
                if let Some(e) = o.atual().cloned() {
                    match c.corrigir(&mut self.sessao, &e, &o.correcao) {
                        Ok(_) => o.atualizar(c, self.sessao.apresentacao()),
                        Err(msg) => self.status = msg,
                    }
                }
            }
            O::Ignorar => {
                if let Some(e) = o.atual().cloned() {
                    o.ignorados_aqui.insert((e.elemento, e.campo, e.palavra));
                    o.atualizar(c, self.sessao.apresentacao());
                }
            }
            O::IgnorarTodas => {
                if let Some(e) = o.atual().cloned() {
                    c.ignorar(&e.palavra);
                    o.atualizar(c, self.sessao.apresentacao());
                }
            }
            O::Adicionar => {
                if let Some(e) = o.atual().cloned() {
                    if let Err(msg) = c.adicionar_ao_dicionario(&e.palavra) {
                        self.status = format!("Não consegui gravar o dicionário pessoal: {msg}");
                    }
                    o.atualizar(c, self.sessao.apresentacao());
                }
            }
        }
        self.ortografia_mostrar();
        iced::widget::text_input::focus(iced::widget::text_input::Id::new(crate::ortografia::ID_CAMPO))
    }

    /// O diálogo de "salvar como" do PDF; a escolha volta como `PdfEscolhido`.
    fn dialogo_salvar_pdf(&self) -> Task<Mensagem> {
        let sugerido = std::path::Path::new(&self.nome_arquivo).with_extension("pdf").to_string_lossy().to_string();
        Task::perform(
            async move {
                let mut dialogo = rfd::AsyncFileDialog::new().set_file_name(&sugerido).add_filter("PDF (*.pdf)", &["pdf"]);
                if let Some(pasta) = crate::configuracao::pasta_das_apresentacoes() {
                    dialogo = dialogo.set_directory(pasta);
                }
                dialogo.save_file().await.map(|f| f.path().to_path_buf())
            },
            Mensagem::PdfEscolhido,
        )
    }

    /// Joga fora o corretor atual; o próximo ciclo carrega de novo com os idiomas da configuração.
    fn recarregar_corretor(&mut self) {
        self.corretor = None;
        self.corretor_carregando = false;
        self.ortografia = None;
    }

    fn tratar_impressao(&mut self, m: crate::imprimir::ImprMsg) -> Task<Mensagem> {
        use crate::imprimir::ImprMsg as I;
        let Some(i) = self.impressao.as_mut() else { return Task::none() };
        match m {
            I::Impressora(n) => {
                i.escolhida = n;
                i.pdf = false;
            }
            I::Pdf => i.pdf = true,
            I::Copias(d) => i.copias = (i.copias as i32 + d).clamp(1, 99) as u32,
            I::SoAtual(b) => i.so_atual = b,
            I::Cancelar => self.impressao = None,
            I::Confirmar if i.pdf => {
                // "Salvar como PDF": o mesmo diálogo de Exportar, valendo só para o que foi escolhido.
                self.pdf_so_atual = i.so_atual;
                self.impressao = None;
                return self.dialogo_salvar_pdf();
            }
            I::Confirmar => {
                i.enviando = true;
                let (so_atual, copias, impressora) = (i.so_atual, i.copias, i.impressora().map(String::from));
                let arquivo = std::env::temp_dir().join(format!("featherslide-impressao-{}.pdf", std::process::id()));
                if let Err(e) = self.exportar_pdf(&arquivo, so_atual) {
                    self.impressao = None;
                    self.status = format!("Erro ao imprimir: {e}");
                    return Task::none();
                }
                return Task::perform(
                    async move {
                        let r = crate::imprimir::enviar(&arquivo, impressora.as_deref(), copias);
                        // O CUPS já copiou o arquivo para a fila; no Windows o leitor ainda precisa dele.
                        if !cfg!(target_os = "windows") {
                            let _ = std::fs::remove_file(&arquivo);
                        }
                        r
                    },
                    Mensagem::ImpressaoResultado,
                );
            }
        }
        Task::none()
    }

    fn tratar_revisao(&mut self, m: crate::revisao::RevisaoMsg) -> Task<Mensagem> {
        use crate::revisao::RevisaoMsg as R;
        match m {
            R::Texto(t) => match self.revisao.as_mut() {
                Some(Revisao::Link(x)) | Some(Revisao::Comentario(x)) => *x = t,
                None => {}
            },
            R::Fechar => self.revisao = None,
            R::Confirmar => match self.revisao.clone() {
                Some(Revisao::Link(url)) => match self.sessao.link_definir(&url) {
                    Ok(_) => self.revisao = None,
                    Err(e) => self.status = e,
                },
                Some(Revisao::Comentario(texto)) => {
                    let autor = nome_do_usuario();
                    match self.sessao.comentario_adicionar(&autor, &texto) {
                        Ok(_) => self.revisao = Some(Revisao::Comentario(String::new())),
                        Err(e) => self.status = e,
                    }
                }
                None => {}
            },
            R::RemoverLink => {
                let _ = self.sessao.link_remover();
                self.revisao = None;
            }
            R::Resolver(i, resolvido) => {
                let _ = self.sessao.comentario_resolver(i, resolvido);
            }
            R::Excluir(i) => {
                let _ = self.sessao.comentario_excluir(i);
            }
            R::ExcluirTodos => {
                let _ = self.sessao.comentarios_excluir_todos(false);
            }
        }
        Task::none()
    }

    /// Atualiza as imagens de todos os gatilhos com a visão atual do slide de destino antes de salvar.
    fn atualizar_imagens_de_gatilhos(&mut self) {
        let total_slides = self.sessao.apresentacao().slides.len();
        let mut mudancas = Vec::new();
        let fonte = featherslide_mcp::fonte::carregar();

        for (s_idx, s) in self.sessao.apresentacao().slides.iter().enumerate() {
            for el in &s.elementos {
                if el.destino > 0 && (el.destino as usize) <= total_slides {
                    let dest_idx = el.destino as usize - 1;
                    if let Some(q) = self.quadro_em_cache(dest_idx).or_else(|| self.sessao.quadro(dest_idx, None, false).ok().flatten()) {
                        if let Ok(bytes) = featherslide_mcp::renderizar::renderizar_png(&q, 480, false, &fonte) {
                            let url = featherslide_io::base64::para_data_url("image/png", &bytes);
                            mudancas.push((s_idx, el.id, url));
                        }
                    }
                }
            }
        }

        for (s_idx, el_id, url) in mudancas {
            if let Some(s) = self.sessao.apresentacao_mut().slides.get_mut(s_idx) {
                if let Some(e) = s.elementos.iter_mut().find(|e| e.id == el_id) {
                    e.imagem = url.into();
                    e.tipo = Tipo::Imagem;
                    e.texto.clear();
                    if e.contorno.is_empty() {
                        e.contorno = "#dadce0".into();
                        e.espessura = 1.0;
                    }
                }
            }
        }
    }

    fn view_editor(&self) -> Element<'_, Mensagem> {
        if self.arquivos.painel { return self.view_arquivos(); }
        // 1. Modo Apresentação em Tela Cheia
        if self.em_apresentacao {
            return if self.modo_apresentador { self.view_apresentador() } else { self.view_apresentacao(true) };
        }

        // 2. Tela de Início / Boas-Vindas (Imagem 1)
        if self.em_tela_inicio {
            return self.view_tela_inicio();
        }

        // 3. Editor Principal com a Faixa e Palco (Imagem 2)
        let estado = self.sessao.estado();
        let tem_sel = !estado.selecao.is_empty();
        let formato = estado.formato;
        let negrito_on = formato.as_ref().map_or(false, |f| f.negrito);
        let italico_on = formato.as_ref().map_or(false, |f| f.italico);
        let sublinhado_on = formato.as_ref().map_or(false, |f| f.sublinhado);
        let tachado_on = formato.as_ref().map_or(false, |f| f.tachado);

        // Faixa Superior (Ribbon com abas organizadas)
        let contexto = self.contexto_faixa();
        let faixa: Element<'_, Mensagem> = if self.faixa_recolhida {
            container(
                button(text("Faixa de opções (Ctrl+F1 para expandir)").size(12))
                    .padding([5, 12])
                    .style(estilo_botao_secundario)
                    .on_press(Mensagem::AlternarFaixa),
            ).width(Length::Fill).into()
        } else {
            view_faixa(
            self.aba_faixa,
            estado.pode_desfazer,
            estado.pode_refazer,
            tem_sel,
            negrito_on,
            italico_on,
            sublinhado_on,
            tachado_on,
            formato.as_ref().map_or("", |f| f.fonte.as_str()),
            &if self.tamanho_editando {
                self.tamanho_txt.clone()
            } else {
                formato.as_ref().map_or(String::new(), |f| format!("{}", f.tamanho))
            },
            &contexto,
        )
            .map(Mensagem::Faixa)
        };

        // Painel Lateral de Miniaturas dos Slides
        let slides_info: Vec<InfoMiniatura> = self
            .sessao
            .apresentacao()
            .slides
            .iter()
            .enumerate()
            .map(|(i, s)| {
                InfoMiniatura {
                    indice: i,
                    quadro: self.quadro_em_cache(i),
                    oculto: s.oculto,
                }
            })
            .collect();
        let secoes = self.sessao.secoes();
        let miniaturas = view_miniaturas(&slides_info, self.sessao.atual(), &self.slides_sel, self.arrastando_slide, self.arrastando_secao, &secoes, &self.secoes_recolhidas, self.renomeando_secao.as_ref()).map(Mensagem::Miniatura);

        // Palco Central (Canvas do slide)
        let quadro = self.sessao.quadro(self.sessao.atual(), None, true).ok().flatten();
        let canvas_palco: Element<'_, Mensagem> = Element::from(canvas(PalcoPrograma {
            controle_gradiente: self.formato_forma.as_ref().and_then(|e|e.controle_gradiente()),
            quadro,
            selecao: estado.selecao.clone(),
            editando: self.editando_id,
            texto_edicao: if self.editando_id.is_some() { self.texto_edicao.clone() } else { String::new() },
            cursor_edicao: self.cursor_edicao,
            selecao_texto: if self.editando_id.is_some() { cursor::intervalo(&self.texto_edicao, self.ancora_edicao, self.cursor_edicao) } else { None },
            grade: self.grade,
            guias: self.guias,
            regua: self.regua,
            ondulados: self.ondulados_atuais().iter().map(|o| [o.x, o.y, o.largura, o.tamanho, if o.tipo == featherslide_ortografia::TipoOndulado::Gramatica { 1.0 } else { 0.0 }]).collect(),
            celula: self.editando_celula.and_then(|(id, r, c)| self.sessao.tabela_celula_retangulo(id, r, c)),
            caixas: self.sessao.slide().elementos.iter().filter(|e| e.papel != featherslide_core::modelo::Papel::Fundo && !e.oculto).map(|e| [e.x.min(e.x + e.largura), e.y.min(e.y + e.altura), e.largura.abs(), e.altura.abs()]).collect(),
            zoom: self.zoom,
            arrastando_slide: self.arrastando_slide,
            pan: self.pan,
            ids_caixas: self.sessao.slide().elementos.iter().filter(|e| e.papel != featherslide_core::modelo::Papel::Fundo && !e.oculto).map(|e| e.id).collect(),
            ajustando_recorte: self.ajustando_recorte,
            pegando_cor: self.pegando_cor || self.marcando.is_some(),
            desenho_livre: self.desenho_livre,
            conectando: self.conector_modo.is_some(),
        })
        .width(Length::Fill)
        .height(Length::Fill))
        .map(Mensagem::Palco);
        let canvas_palco = match self.previa.as_ref().and_then(|p| p.programa()) {
            Some(mut programa) => {
                programa.escala_relativa = 0.95 * self.zoom;
                programa.pan = self.pan;
                Element::from(canvas(crate::previa::CanvasPrevia(programa)).width(Length::Fill).height(Length::Fill))
                    .map(|()| Mensagem::QuadroPrevia)
            },
            None => canvas_palco,
        };
        // A imagem desenhada no canvas não é aparada pelo canvas: o contêiner apara, para o
        // slide não passar por cima da faixa de opções ao dar zoom ou arrastar.
        let canvas_palco: Element<'_, Mensagem> = container(canvas_palco).width(Length::Fill).height(Length::Fill).clip(true).into();

        // Painel de Anotações do apresentador
        let notas_do_slide = &self.sessao.slide().notas;
        let painel_notas = container(
            row![
                container(icone_comentario(15.0, "#5f6368")).padding(iced::Padding { top: 2.0, ..Default::default() }),
                text_input("Clique para adicionar anotações do apresentador", notas_do_slide)
                    .on_input(Mensagem::NotasMudou)
                    .padding([2, 4])
                    .size(13)
                    .width(Length::Fill)
                    .style(|_, _| iced::widget::text_input::Style {
                        background: Background::Color(Color::TRANSPARENT),
                        border: Border::default(),
                        icon: COR_TEXTO_MUTED(),
                        placeholder: COR_TEXTO_MUTED(),
                        value: COR_TEXTO(),
                        selection: Color { a: 0.3, ..COR_AZUL_ACENTO() },
                    }),
            ]
            .spacing(10)
            .padding([12, 20]),
        )
        .height(Length::Fixed(96.0))
        .width(Length::Fill)
        .style(estilo_painel_notas);

        // As anotações ficam só embaixo do slide, não embaixo do painel de miniaturas.
        let mut area_central = if self.classificacao {
            column![self.view_classificacao()]
        } else {
            column![canvas_palco]
        };
        if self.config.painel_notas && !self.classificacao {
            area_central = area_central.push(painel_notas);
        }

        let mut corpo_meio = if self.config.painel_slides {
            row![miniaturas, area_central]
        } else {
            row![area_central]
        };
        if let Some(e) = &self.formato_forma {
            corpo_meio = corpo_meio.push(crate::formato_forma::view(e).map(Mensagem::FormatoForma));
        } else if let Some(e) = &self.editor_diagrama {
            corpo_meio = corpo_meio.push(crate::editor_diagrama::view(e));
        } else if self.painel_selecao {
            corpo_meio = corpo_meio.push(self.view_painel_selecao());
        }
        let corpo_meio = corpo_meio
        .width(Length::Fill)
        .height(Length::Fill);

        // Barra de Status e Rodapé
        let rodape = container(
            row![
                text(format!(
                    "Slide {} de {}",
                    self.sessao.atual() + 1,
                    self.sessao.apresentacao().slides.len()
                ))
                .size(11)
                .color(COR_TEXTO_MUTED()),
                Space::with_width(Length::Fill),
                text(&self.status).size(11).color(COR_TEXTO_MUTED()),
                Space::with_width(Length::Fill),
                button(row![icone_classificacao(14.0, "#5f6368"), text("Todos os slides").size(11)].spacing(5).align_y(Alignment::Center))
                    .padding([3,8]).style(estilo_botao_pequeno(self.classificacao))
                    .on_press(Mensagem::Faixa(if self.classificacao { FaixaMensagem::ExibirNormal } else { FaixaMensagem::Classificacao })),
                button(icone_menos(12.0, "#5f6368"))
                    .padding([2, 6])
                    .style(estilo_botao_pequeno(false))
                    .on_press(Mensagem::Zoom(-0.1)),
                text(format!("{}%", (self.zoom * 100.0) as u32))
                    .size(11)
                    .color(COR_TEXTO_MUTED()),
                button(icone_mais(12.0, "#5f6368"))
                    .padding([2, 6])
                    .style(estilo_botao_pequeno(false))
                    .on_press(Mensagem::Zoom(0.1)),

                Space::with_width(12),
            ]
            .spacing(6)
            .padding([2, 12])
            .align_y(Alignment::Center),
        )
        .style(estilo_rodape);

        column![self.view_barra_titulo(), self.barra_documentos(), faixa, corpo_meio, rodape].into()
    }

    /// Wrapper de compatibilidade: os botões e a geometria vivem no feather-ui.
    fn controles_janela(id: window::Id) -> Element<'static, Mensagem> {
        feather_ui::cromo::controles_janela(feather_ui::cromo::AcoesJanela {
            arrastar: Mensagem::ArrastarJanela(id),
            minimizar: Mensagem::Minimizar(id),
            maximizar: Mensagem::AlternarMaximizar(id),
            fechar: Mensagem::FecharJanela(id),
        })
    }

    /// A mesma barra própria do template, também nas janelas secundárias.
    fn com_barra_de_janela<'a>(&'a self, id: window::Id, conteudo: Element<'a, Mensagem>) -> Element<'a, Mensagem> {
        if !self.config.barra_propria { return conteudo; }
        feather_ui::cromo::janela_com_barra(
            &self.titulo(id), conteudo,
            feather_ui::cromo::AcoesJanela {
                arrastar: Mensagem::ArrastarJanela(id),
                minimizar: Mensagem::Minimizar(id),
                maximizar: Mensagem::AlternarMaximizar(id),
                fechar: Mensagem::FecharJanela(id),
            }
        )
    }

    /// A barra da tela de início quando a janela usa a barra própria: nome do app, arrastável, e os botões do sistema.
    pub(crate) fn barra_da_janela_simples(&self) -> Option<Element<'_, Mensagem>> {
        if !self.config.barra_propria {
            return None;
        }
        let nome = container(
            row![
                feather_ui::marca::icone_atual(26.),
                text("FeatherSlide").size(13),
                button(text("Arquivos")).on_press(Mensagem::Arquivos(crate::arquivos::Acao::Painel)).style(estilo_botao_secundario),
                Space::with_width(Length::Fill),
                button(row![icone_config(14.0, "#5f6368"), text("Configurações").size(12.5)].spacing(6).align_y(Alignment::Center)).padding([5, 12]).style(estilo_botao_secundario).on_press(Mensagem::Configuracoes),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ).padding([0, 14]).align_y(Alignment::Center).width(Length::Fill).height(Length::Fixed(BARRA_ALTURA)).style(estilo_container_topo);
        let id = self.janelas.principal?;
        Some(row![iced::widget::mouse_area(nome).on_press(Mensagem::ArrastarJanela(id)), Self::controles_janela(id)].align_y(Alignment::Center).into())
    }

    fn resultado_da_pesquisa(&self, total: usize, atual: usize) -> String {
        if self.pesquisa.is_empty() {
            String::new()
        } else if total == 0 {
            "Nada encontrado".into()
        } else {
            format!("{atual} de {total}")
        }
    }

    /// A barra de título: salvar, desfazer, refazer, nome do arquivo, pesquisa e Apresentar.
    fn view_barra_titulo(&self) -> Element<'_, Mensagem> {
        let estado = self.sessao.estado();
        let tira = |icone: iced::widget::svg::Svg<'static, iced::Theme>, msg: Option<FaixaMensagem>, dica: &str| -> Element<'_, Mensagem> {
            feather_ui::cromo::acao_icone(icone, dica, msg.map(Mensagem::Faixa))
        };
        let emblema = feather_ui::cromo::emblema();
        let ativo = if estado.sujo { "#5f6368" } else { "#9aa0a6" };
        let titulo = row![
            text(self.nome_arquivo.clone()).size(13),
            text(if estado.sujo { "  • não salvo" } else { "  salvo" }).size(11.5).color(COR_TEXTO_MUTED()),
        ]
        .align_y(Alignment::Center);
        let pesquisa = row![
            iced::widget::text_input("Pesquisar nos slides", &self.pesquisa)
                .on_input(Mensagem::Pesquisa)
                .on_submit(Mensagem::PesquisaProximo)
                .size(12.5)
                .padding([6, 14])
                .width(Length::Fixed(320.0))
                .style(|_, status| iced::widget::text_input::Style {
                    background: Background::Color(COR_PAINEL()),
                    border: Border { color: if matches!(status, iced::widget::text_input::Status::Focused) { COR_AZUL_ACENTO() } else { COR_BORDA() }, width: 1.0, radius: Radius::from(16.0) },
                    icon: COR_TEXTO_MUTED(),
                    placeholder: COR_TEXTO_MUTED(),
                    value: COR_TEXTO(),
                    selection: Color { a: 0.3, ..COR_AZUL_ACENTO() },
                }),
            text(self.pesquisa_resultado.clone()).size(11.5).color(COR_TEXTO_MUTED()),
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        let apresentar = iced::widget::tooltip(
            button(icone_apresentar(18.0, "#ffffff")).padding([7, 12]).style(estilo_botao_primario).on_press(Mensagem::Faixa(FaixaMensagem::Apresentar)),
            container(text("Apresentar do começo (F5)").size(12)).padding([4, 8]).style(estilo_dica),
            iced::widget::tooltip::Position::Bottom,
        );
        let barra = container(
            row![
                emblema,
                button(text("Arquivos").size(12.5)).padding([7, 10]).style(estilo_botao_pequeno(false)).on_press(Mensagem::Arquivos(crate::arquivos::Acao::Painel)),
                Space::with_width(6),
                tira(icone_salvar(19.0, ativo), Some(FaixaMensagem::Salvar), "Salvar (Ctrl+S)"),
                tira(icone_pdf(19.0, ativo), Some(FaixaMensagem::ExportarPdf), "Exportar como PDF (Ctrl+Shift+E)"),
                tira(icone_imprimir(19.0, ativo), Some(FaixaMensagem::Imprimir), "Imprimir (Ctrl+P)"),
                tira(icone_dados(19.0, ativo), Some(FaixaMensagem::Propriedades), "Propriedades da apresentação"),
                tira(icone_desfazer(19.0, if estado.pode_desfazer { "#5f6368" } else { "#bdc1c6" }), estado.pode_desfazer.then_some(FaixaMensagem::Desfazer), "Voltar (Ctrl+Z)"),
                tira(icone_refazer(19.0, if estado.pode_refazer { "#5f6368" } else { "#bdc1c6" }), estado.pode_refazer.then_some(FaixaMensagem::Refazer), "Avançar (Ctrl+Y)"),
                Space::with_width(8),
                apresentar,
                container(Space::with_width(1)).height(Length::Fixed(20.0)).style(estilo_linha),
                Space::with_width(10),
                titulo,
                Space::with_width(Length::Fill),
                pesquisa,
                Space::with_width(Length::Fill),
            ]
            .spacing(2)
            .align_y(Alignment::Center),
        )
        .padding([0, 12])
        .width(Length::Fill)
        .height(Length::Fixed(BARRA_ALTURA))
        .align_y(Alignment::Center)
        .style(estilo_container_topo);
        let fio = container(Space::with_height(1)).width(Length::Fill).style(estilo_linha);
        if !self.config.barra_propria {
            return column![barra, fio].into();
        }
        // Barra própria: arrastar move a janela, duplo clique maximiza, e os três botões do sistema ficam à direita.
        let Some(id) = self.janelas.principal else { return column![barra, fio].into() };
        let arrastavel = iced::widget::mouse_area(barra).on_press(Mensagem::ArrastarJanela(id));
        column![row![arrastavel, Self::controles_janela(id)].align_y(Alignment::Center), fio].into()
    }

    /// O painel de seleção: os itens do slide, de cima para baixo, com olho para esconder.
    fn view_painel_selecao(&self) -> Element<'_, Mensagem> {
        let itens = self.sessao.lista_de_elementos();
        let nome_do_tipo = |t: &str| match t {
            "texto" => "Caixa de texto",
            "retangulo" => "Retângulo",
            "retangulo_arredondado" => "Retângulo arredondado",
            "elipse" => "Elipse",
            "triangulo" => "Triângulo",
            "seta" => "Seta",
            "linha" => "Linha",
            "imagem" => "Imagem",
            "setor" => "Círculo parcial",
            "tabela" => "Tabela",
            "grafico" => "Gráfico",
            outro => outro,
        }
        .to_string();
        let mut lista = column![].spacing(2);
        if itens.is_empty() {
            lista = lista.push(text("Este slide não tem itens.").size(12).color(COR_TEXTO_MUTED()));
        }
        for it in itens {
            let rotulo = if it.rotulo == it.tipo { nome_do_tipo(it.tipo) } else { it.rotulo.clone() };
            let cor = if it.oculto { COR_TEXTO_MUTED() } else { COR_TEXTO() };
            let olho = button(if it.oculto { icone_ocultar(14.0, "#5f6368") } else { icone_olho(14.0, "#5f6368") })
                .padding([4, 6])
                .style(estilo_botao_pequeno(false))
                .on_press(Mensagem::Faixa(FaixaMensagem::OcultarItem(it.id, !it.oculto)));
            let grupo = if it.grupo != 0 { text(format!("grupo {}", it.grupo)).size(10.5).color(COR_AZUL_ACENTO()) } else { text("").size(10.5) };
            let linha = row![
                text(rotulo).size(12.5).color(cor).width(Length::Fill),
                grupo,
            ]
            .spacing(6)
            .align_y(Alignment::Center);
            let corpo = button(linha)
                .width(Length::Fill)
                .padding([5, 8])
                .style(estilo_botao_pequeno(it.selecionado))
                .on_press(Mensagem::Faixa(FaixaMensagem::SelecionarItem(it.id, self.modificadores.control() || self.modificadores.shift())));
            lista = lista.push(row![olho, corpo].spacing(2).align_y(Alignment::Center));
        }
        let acoes = row![
            button(row![icone_agrupar(13.0, "#5f6368"), text("Agrupar").size(11.5)].spacing(5).align_y(Alignment::Center)).padding([3, 8]).style(estilo_botao_pequeno(false)).on_press_maybe((self.sessao.estado().selecao.len() >= 2).then_some(Mensagem::Faixa(FaixaMensagem::Agrupar))),
            button(row![icone_desagrupar(13.0, "#5f6368"), text("Desagrupar").size(11.5)].spacing(5).align_y(Alignment::Center)).padding([3, 8]).style(estilo_botao_pequeno(false)).on_press_maybe((!self.sessao.estado().selecao.is_empty()).then_some(Mensagem::Faixa(FaixaMensagem::Desagrupar))),
        ]
        .spacing(4);
        container(column![text("Painel de seleção").size(13.5), acoes, crate::tema::rolagem(lista).height(Length::Fill)].spacing(8).padding(12))
            .width(Length::Fixed(240.0))
            .height(Length::Fill)
            .style(estilo_painel_miniaturas)
            .into()
    }

    /// A apresentação em tela cheia: o slide, e embaixo uma barra de controles discreta.
    fn view_apresentacao(&self, com_barra: bool) -> Element<'_, Mensagem> {
        let Some(quadro) = self.sessao.quadro(self.sessao.atual(), None, false).ok().flatten() else {
            return Space::new(Length::Fill, Length::Fill).into();
        };
        let transicao = self.repro.transicao.as_ref().map(|t| (t.de.clone(), t.tr.clone(), self.repro.progresso().unwrap_or(1.0)));
        let tela = if self.repro.preta {
            Some(Color::BLACK)
        } else if self.repro.branca {
            Some(Color::WHITE)
        } else {
            None
        };
        let gatilhos: Vec<([f32; 4], usize)> = self
            .sessao
            .slide()
            .elementos
            .iter()
            .filter(|e| e.destino > 0)
            .filter_map(|e| Some(([e.x, e.y, e.largura, e.altura], self.sessao.gatilho_do_elemento(self.sessao.atual(), e.id)?)))
            .collect();
        let links: Vec<([f32; 4], u64)> = self
            .sessao
            .slide()
            .elementos
            .iter()
            .filter(|e| !e.link.is_empty() && !e.oculto)
            .map(|e| ([e.x, e.y, e.largura, e.altura], e.id))
            .collect();
        let slide: Element<'_, Mensagem> = Element::from(
            canvas(Programa { quadro, fundo_externo: Color::BLACK, escala_relativa: 1.0, pan: (0.0, 0.0), efeitos: self.efeitos_atuais(), transicao, tela, gatilhos, links }).width(Length::Fill).height(Length::Fill),
        )
        .map(Mensagem::ApresentacaoAcao);

        let botao = |conteudo: Element<'static, Mensagem>, msg: Mensagem| {
            button(conteudo).padding([6, 10]).style(estilo_controle_apresentacao).on_press(msg)
        };
        let barra = container(
            row![
                botao(icone_seta_esq(15.0, "#ffffff").into(), Mensagem::ApresentacaoAnterior),
                text(if let Some(roteiro) = &self.roteiro_personalizado {
                    let id = self.sessao.slide().id;
                    let n = roteiro.iter().position(|&x| x == id).map_or(0, |p| p + 1);
                    format!("{} / {} (personalizada)", n, roteiro.len())
                } else {
                    format!("{} / {}", self.sessao.atual() + 1, self.sessao.apresentacao().slides.len())
                }).size(13).color(Color::WHITE),
                botao(icone_seta_dir(15.0, "#ffffff").into(), Mensagem::ApresentacaoProximo),
                botao(icone_cancelar(15.0, "#ffffff").into(), Mensagem::ApresentacaoSair),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding([4, 12])
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgba8(20, 20, 20, 0.45))),
            border: Border { radius: Radius::from(20.0), ..Border::default() },
            ..container::Style::default()
        });
        if !com_barra {
            return slide;
        }
        iced::widget::stack![
            slide,
            container(barra).width(Length::Fill).height(Length::Fill).align_x(Alignment::Center).align_y(iced::alignment::Vertical::Bottom).padding(18),
        ]
        .into()
    }

    /// A tela do apresentador (monitor principal, com dois monitores): o slide atual, o próximo,
    /// as anotações, o cronômetro e os controles. A plateia vê só o slide, no outro monitor.
    fn view_apresentador(&self) -> Element<'_, Mensagem> {
        let atual = self.sessao.atual();
        let total = self.sessao.apresentacao().slides.len();
        let proximo = self.visivel_a_partir_de(atual, 1);
        let miniatura_proximo: Element<'_, Mensagem> = match proximo.and_then(|i| self.quadro_em_cache(i)) {
            Some(q) => Element::from(canvas(Miniatura::nova(Some(q), None, false)).width(Length::Fill).height(Length::Fixed(190.0))).map(|_: MiniaturaMensagem| Mensagem::Tique),
            None => container(text("Fim da apresentação").size(14).color(COR_TEXTO_MUTED())).width(Length::Fill).height(Length::Fixed(190.0)).align_x(Alignment::Center).align_y(iced::alignment::Vertical::Center).into(),
        };
        let decorrido = self.cronometro.decorrido().as_secs();
        let cronometro = format!("{:02}:{:02}", decorrido / 60, decorrido % 60);
        let notas = self.sessao.slide().notas.clone();
        let notas: Element<'_, Mensagem> = if notas.trim().is_empty() {
            text("Sem anotações neste slide.").size(14).color(COR_TEXTO_MUTED()).into()
        } else {
            crate::tema::rolagem(text(notas).size(17)).height(Length::Fill).into()
        };
        let botao = |rotulo: &str, msg: Mensagem| button(text(rotulo.to_string()).size(13).align_x(Alignment::Center)).padding([8, 14]).style(estilo_botao_secundario).on_press(msg);
        let controles = row![
            botao("Anterior", Mensagem::ApresentacaoAnterior),
            botao("Próximo", Mensagem::ApresentacaoProximo),
            button(text("Encerrar").size(13)).padding([8, 14]).style(estilo_botao_primario).on_press(Mensagem::ApresentacaoSair),
        ]
        .spacing(8);
        let lateral = column![
            text(cronometro).size(40),
            row![botao(if self.cronometro.pausado() { "Retomar tempo" } else { "Pausar tempo" }, Mensagem::CronometroAlternar), botao("Zerar", Mensagem::CronometroReiniciar)].spacing(6),
            text("Tela da plateia").size(12).color(COR_TEXTO_MUTED()),
            iced::widget::pick_list(std::iter::once("Janela".to_string()).chain(self.monitores_apresentacao.iter().map(|m| m.nome.clone())).collect::<Vec<_>>(), Some(self.monitor_plateia.clone()), Mensagem::ApresentadorMonitor).text_size(13),
            row![botao("Tela preta", Mensagem::TelaApresentacao(1)), botao("Tela branca", Mensagem::TelaApresentacao(2))].spacing(6),
            text(format!("Slide {} de {}", atual + 1, total)).size(13).color(COR_TEXTO_MUTED()),
            Space::with_height(6),
            text("Próximo").size(12.5).color(COR_TEXTO_MUTED()),
            miniatura_proximo,
            Space::with_height(8),
            text("Anotações").size(12.5).color(COR_TEXTO_MUTED()),
            notas,
            controles,
        ]
        .spacing(6)
        .width(Length::Fixed(380.0))
        .height(Length::Fill);
        container(row![container(self.view_apresentacao(false)).width(Length::Fill).height(Length::Fill), lateral].spacing(16))
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style { background: Some(Background::Color(Color::from_rgb8(0x14, 0x14, 0x18))), text_color: Some(Color::WHITE), ..container::Style::default() })
            .into()
    }

    fn view_tela_inicio(&self) -> Element<'_, Mensagem> {
        let topo = container(
            row![
                button(text(if self.arquivos.recuperacoes.is_empty() { "Arquivos".to_string() } else { format!("Arquivos · {} recuperáveis", self.arquivos.recuperacoes.len()) })).padding([6, 14]).style(estilo_botao_secundario).on_press(Mensagem::Arquivos(crate::arquivos::Acao::Painel)),
                Space::with_width(Length::Fill),
                button(row![icone_config(14.0, "#5f6368"), text("Configurações").size(12.5)].spacing(6).align_y(Alignment::Center))
                    .padding([6, 14])
                    .style(estilo_botao_secundario)
                    .on_press(Mensagem::Configuracoes),
            ]
            .padding([10, 16])
            .align_y(Alignment::Center),
        );

        let icone_f = container(text("F").size(26).color(Color::WHITE))
            .width(Length::Fixed(52.0))
            .height(Length::Fixed(52.0))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(|_theme| container::Style {
                background: Some(Background::Color(COR_AZUL_ACENTO())),
                border: Border { radius: Radius::from(8.0), ..Border::default() },
                ..container::Style::default()
            });

        let cabecalho_app = row![
            icone_f,
            column![
                text("FeatherSlide").size(26),
                text("Apresentações que ficam na sua máquina.").size(13).color(COR_TEXTO_MUTED()),
            ]
            .spacing(2),
        ]
        .spacing(14)
        .align_y(Alignment::Center);

        let mut botoes_acao = row![
            button(row![icone_mais(16.0, "#ffffff"), text("Nova apresentação").size(13.5)].spacing(8).align_y(Alignment::Center))
                .padding([9, 18])
                .style(estilo_botao_primario)
                .on_press(Mensagem::NovaApresentacao),
            button(text("Abrir…").size(13.5))
                .padding([9, 18])
                .style(estilo_botao_secundario)
                .on_press(Mensagem::AbrirArquivo),
        ]
        .spacing(12);
        if self.pode_continuar {
            botoes_acao = botoes_acao.push(button(row![icone_voltar(15.0, "#5f6368"), text("Continuar editando").size(13.5)].spacing(8).align_y(Alignment::Center)).padding([9, 18]).style(estilo_botao_secundario).on_press(Mensagem::ContinuarEditando));
        }

        let divisoria = container(Space::with_height(1)).width(Length::Fill).style(|_theme| container::Style {
            background: Some(Background::Color(COR_BORDA_SUAVE())),
            ..container::Style::default()
        });

        let mut corpo = column![cabecalho_app, Space::with_height(6), botoes_acao, Space::with_height(10), divisoria].spacing(10);

        // Modelos em carrossel: os da casa e depois os do usuário, com setas para os lados.
        let estilo_cartao = |_: &iced::Theme, status: button::Status| button::Style {
            background: Some(Background::Color(if matches!(status, button::Status::Hovered) { COR_HOVER() } else { Color::TRANSPARENT })),
            text_color: COR_TEXTO(),
            border: Border { color: if matches!(status, button::Status::Hovered) { COR_AZUL_ACENTO() } else { COR_BORDA_SUAVE() }, width: 1.0, radius: Radius::from(8.0) },
            shadow: iced::Shadow::default(),
        };
        let cartao_modelo = |quadro: Option<Quadro>, nome: String, descricao: String, msg: Mensagem| -> Element<'_, Mensagem> {
            let miniatura = container(Element::from(canvas(Miniatura::nova(quadro, None, false)).width(Length::Fixed(200.0)).height(Length::Fixed(112.0))).map(|_: MiniaturaMensagem| Mensagem::Tique))
                .style(|_| container::Style { border: Border { color: COR_BORDA_SUAVE(), width: 1.0, radius: Radius::from(4.0) }, ..container::Style::default() });
            button(column![miniatura, text(nome).size(13), text(descricao).size(11).color(COR_TEXTO_MUTED()).width(Length::Fixed(200.0))].spacing(4))
                .padding(8)
                .width(Length::Fixed(218.0))
                .style(estilo_cartao)
                .on_press(msg)
                .into()
        };
        let mut todos: Vec<Element<'_, Mensagem>> = Vec::new();
        for (m, quadro) in &self.modelos {
            todos.push(cartao_modelo(quadro.clone(), m.nome.to_string(), m.descricao.to_string(), Mensagem::NovoDoModelo(m.id)));
        }
        for m in &self.meus_modelos {
            todos.push(cartao_modelo(m.quadro.clone(), m.nome.clone(), "Modelo seu".into(), Mensagem::NovoDeMeuModelo(m.caminho.clone())));
        }
        let total = todos.len();
        let visiveis = (((self.tamanho_janela.width - 160.0) / 234.0).floor() as usize).clamp(1, total.max(1));
        let inicio = self.carrossel.min(total.saturating_sub(visiveis));
        let seta = |icone: iced::widget::svg::Svg<'static, iced::Theme>, d: i32, ativa: bool| {
            button(container(icone).width(Length::Fill).height(Length::Fill).align_x(Alignment::Center).align_y(Alignment::Center))
                .width(Length::Fixed(38.0))
                .height(Length::Fixed(112.0))
                .style(estilo_botao_secundario)
                .on_press_maybe(ativa.then_some(Mensagem::Carrossel(d)))
        };
        let corpo_modelos: Element<'_, Mensagem> = if self.ver_todos_modelos {
            // Todos os modelos em grade, para ver de uma vez.
            let mut grade = column![].spacing(14);
            let mut restantes = todos.into_iter().peekable();
            while restantes.peek().is_some() {
                let mut linha = row![].spacing(16);
                for c in restantes.by_ref().take((visiveis + 1).max(1)) {
                    linha = linha.push(c);
                }
                grade = grade.push(linha);
            }
            grade.into()
        } else {
            let mut faixa = row![].spacing(16).align_y(Alignment::Center);
            faixa = faixa.push(seta(icone_seta_esq(18.0, "#5f6368"), -1, inicio > 0));
            for c in todos.into_iter().skip(inicio).take(visiveis) {
                faixa = faixa.push(c);
            }
            faixa = faixa.push(seta(icone_seta_dir(18.0, "#5f6368"), 1, inicio + visiveis < total));
            faixa.into()
        };
        let cabecalho = row![
            text("Comece por um modelo").size(13.5),
            Space::with_width(Length::Fill),
            text(if self.ver_todos_modelos { format!("{total} modelos") } else { format!("{}–{} de {}", inicio + 1, (inicio + visiveis).min(total), total) }).size(11.5).color(COR_TEXTO_MUTED()),
            button(text(if self.ver_todos_modelos { "Mostrar em carrossel" } else { "Ver todos" }).size(12)).padding([4, 10]).style(estilo_botao_secundario).on_press(Mensagem::VerTodosModelos),
            button(text("Importar modelo…").size(12)).padding([4, 10]).style(estilo_botao_secundario).on_press(Mensagem::Faixa(FaixaMensagem::AbrirConfigModelos)),
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        let modelos_col = column![cabecalho, corpo_modelos].spacing(10);

        let mut rolagem = column![modelos_col, Space::with_height(8)].spacing(10);
        if !self.recentes.is_empty() {
            rolagem = rolagem.push(text("Apresentações recentes").size(13.5));
            let mut cartoes: Vec<Element<'_, Mensagem>> = Vec::new();
            for r in &self.recentes {
                let miniatura = container(Element::from(canvas(Miniatura::nova(r.quadro.clone(), None, false)).width(Length::Fixed(200.0)).height(Length::Fixed(112.0))).map(|_: MiniaturaMensagem| Mensagem::Tique))
                    .style(|_| container::Style {
                        border: Border { color: COR_BORDA_SUAVE(), width: 1.0, radius: Radius::from(4.0) },
                        ..container::Style::default()
                    });
                let cartao = button(
                    column![
                        miniatura,
                        text(r.nome.clone()).size(12.5).width(Length::Fixed(200.0)),
                        text(r.data.clone()).size(11).color(COR_TEXTO_MUTED()),
                    ]
                    .spacing(5),
                )
                .padding(8)
                .width(Length::Fixed(218.0))
                .height(Length::Fixed(176.0))
                .style(|_theme, status| button::Style {
                    background: Some(Background::Color(COR_PAINEL())),
                    text_color: COR_TEXTO(),
                    border: Border {
                        color: if matches!(status, button::Status::Hovered) { COR_AZUL_ACENTO() } else { COR_BORDA_SUAVE() },
                        width: 1.0,
                        radius: Radius::from(8.0),
                    },
                    shadow: iced::Shadow { color: Color::from_rgba8(0, 0, 0, 0.10), offset: iced::Vector::new(0.0, 1.0), blur_radius: 4.0 },
                })
                .on_press(Mensagem::ArquivoEscolhidoParaAbrir(Some(r.caminho.clone())));
                cartoes.push(cartao.into());
            }
            // Linhas montadas aqui (o `wrap` do Iced deixava os cartões esticarem até o fim da janela).
            let por_linha = (((self.tamanho_janela.width - 88.0) / 234.0).floor() as usize).max(1);
            let mut linhas = column![].spacing(16);
            let mut restante = cartoes.into_iter().peekable();
            while restante.peek().is_some() {
                let mut linha = row![].spacing(16);
                for cartao in restante.by_ref().take(por_linha) {
                    linha = linha.push(cartao);
                }
                linhas = linhas.push(linha);
            }
            rolagem = rolagem.push(linhas);
        }
        corpo = corpo.push(crate::tema::rolagem(rolagem).height(Length::Fill));

        let barra_janela: Element<'_, Mensagem> = self.barra_da_janela_simples().unwrap_or_else(|| Space::with_height(0).into());
        let topo: Element<'_, Mensagem> = if self.config.barra_propria { Space::with_height(0).into() } else { topo.into() };
        container(column![barra_janela, topo, container(corpo).padding([20, 44]).width(Length::Fill).height(Length::Fill)])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(COR_FUNDO_JANELA())),
                ..container::Style::default()
            })
            .into()
    }
}

/// A data e a hora de agora, no fuso do sistema (`dd/mm/aaaa HH:MM`).
fn chrono_like_hoje() -> String {
    std::process::Command::new("date")
        .arg("+%d/%m/%Y %H:%M")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
#[test]
fn ui011_alinhamento_horizontal_slide_tem_estado_e_undo() {
    use feather_ui::alinhamento::Horizontal;

    let mut app = FeatherSlideApp::default();
    app.sessao.elemento_inserir(Tipo::Texto, None, None).unwrap();
    let id = app.sessao.slide().elementos.last().unwrap().id;
    app.sessao.selecionar_por_id(id);
    let inicial = app.sessao.estado().formato.unwrap().alinhamento;
    assert_eq!(Horizontal::de_identificador(&inicial), Some(Horizontal::Esquerda));

    let _ = app.update(Mensagem::Faixa(FaixaMensagem::Alinhar(
        featherslide_core::modelo::Alinhamento::Centro,
    )));
    let contexto = app.contexto_faixa();
    assert_eq!(Horizontal::de_identificador(&contexto.alinhamento), Some(Horizontal::Centro));
    let _controle = feather_ui::alinhamento::seletor_horizontal(
        Horizontal::de_identificador(&contexto.alinhamento),
        true, true,
        |op| FaixaMensagem::Alinhar(match op {
            Horizontal::Esquerda => featherslide_core::modelo::Alinhamento::Esquerda,
            Horizontal::Centro => featherslide_core::modelo::Alinhamento::Centro,
            Horizontal::Direita => featherslide_core::modelo::Alinhamento::Direita,
            Horizontal::Justificado => featherslide_core::modelo::Alinhamento::Justificado,
        }),
    );

    let _ = app.update(Mensagem::Faixa(FaixaMensagem::Alinhar(
        featherslide_core::modelo::Alinhamento::Justificado,
    )));
    assert_eq!(Horizontal::de_identificador(&app.contexto_faixa().alinhamento),
        Some(Horizontal::Justificado));
    // O alinhamento continua passando pelo modelo e pelo histórico nativo.
    let _ = app.sessao.desfazer();
    assert_eq!(Horizontal::de_identificador(&app.contexto_faixa().alinhamento),
        Some(Horizontal::Centro));
}

#[cfg(test)]
#[test]
fn app004_ribbon_migrada_preserva_aba_atalhos_e_documento() {
    use feather_ui::atalhos::{acao_global, AcaoGlobal};
    use iced::keyboard::{key::Named, Key, Modifiers};

    let mut app = FeatherSlideApp::default();
    let documento_antes = app.sessao.apresentacao().clone();
    // Construir a Ribbon executa os novos componentes Iced compartilhados:
    // abas, grupos, botões compactos e tooltips, sem alterar a apresentação.
    for aba in [
        AbaDaFaixa::Inicio, AbaDaFaixa::Inserir,
        AbaDaFaixa::Design, AbaDaFaixa::Exibir,
    ] {
        let contexto = app.contexto_faixa();
        let _ribbon = view_faixa(
            aba, false, false, false, false, false, false, false,
            "Liberation Sans", "12", &contexto,
        );
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::TrocarAba(aba)));
        assert_eq!(app.aba_faixa, aba);
    }

    // Botões visuais e atalhos distintos continuam encaminhando à mesma
    // semântica global; Shift+F1 e Ctrl+F12 não invocam funções por engano.
    assert_eq!(acao_global(&Key::Named(Named::F1), Modifiers::default()), Some(AcaoGlobal::Ajuda));
    assert_eq!(acao_global(&Key::Named(Named::F12), Modifiers::default()), Some(AcaoGlobal::Monitor));
    assert_eq!(acao_global(&Key::Named(Named::F1), Modifiers::CTRL), Some(AcaoGlobal::AlternarFaixa));
    assert_eq!(acao_global(&Key::Named(Named::F1), Modifiers::SHIFT), None);
    assert_eq!(acao_global(&Key::Named(Named::F12), Modifiers::CTRL), None);

    assert!(!app.faixa_recolhida);
    let _ = app.update(Mensagem::AlternarFaixa);
    assert!(app.faixa_recolhida);
    let _ = app.update(Mensagem::AlternarFaixa);
    assert!(!app.faixa_recolhida);
    assert_eq!(app.sessao.apresentacao(), &documento_antes,
        "UI, abas e atalhos não devem criar revisões no documento");
}

#[cfg(test)]
#[test]
fn alternar_faixa_nao_modifica_apresentacao() {
    let mut app = FeatherSlideApp::default();
    let dados = app.sessao.apresentacao().clone();
    assert!(!app.faixa_recolhida);
    let _ = app.update(Mensagem::AlternarFaixa);
    assert!(app.faixa_recolhida);
    let _ = app.update(Mensagem::AlternarFaixa);
    assert!(!app.faixa_recolhida);
    assert_eq!(app.sessao.apresentacao(), &dados);
}

#[cfg(test)]
#[test]
fn ui010_arrasto_de_miniatura_atualiza_cursor_e_limpa_estado_ao_soltar() {
    let mut app = FeatherSlideApp::default();
    app.sessao.slide_novo("em_branco").unwrap();
    let documento = app.sessao.apresentacao().clone();
    let _ = app.update(Mensagem::Miniatura(MiniaturaMensagem::Pressionou(0)));
    assert!(app.foco_miniaturas);
    assert_eq!(app.arrastando_slide, Some(0));
    assert!(app.arrastando_secao.is_none());
    let cursor = Point::new(180.0, 120.0);
    let _ = app.update(Mensagem::CursorMoveu(cursor));
    assert_eq!(app.cursor_arrasto, Some(cursor));
    let _ = app.update(Mensagem::LimparArraste);
    assert!(app.arrastando_slide.is_none());
    assert!(app.arrastando_secao.is_none());
    assert!(app.inicio_arrasto_secao.is_none());
    assert!(app.cursor_arrasto.is_none());
    assert_eq!(app.sessao.apresentacao(), &documento);
}

#[cfg(test)]
#[test]
fn ui010_arrasto_de_secao_cancelado_nao_reordena_slides() {
    let mut app = FeatherSlideApp::default();
    app.sessao.slide_novo("em_branco").unwrap();
    let ids: Vec<_> = app.sessao.apresentacao().slides.iter().map(|s| s.id).collect();
    let _ = app.update(Mensagem::Miniatura(MiniaturaMensagem::PressionouSecao(
        0, Point::new(30.0, 50.0),
    )));
    assert_eq!(app.arrastando_secao, Some(0));
    let _ = app.update(Mensagem::CursorMoveu(Point::new(70.0, 140.0)));
    assert!(app.cursor_arrasto.is_some());
    let _ = app.update(Mensagem::LimparArraste);
    let depois: Vec<_> = app.sessao.apresentacao().slides.iter().map(|s| s.id).collect();
    assert_eq!(ids, depois);
    assert!(app.arrastando_secao.is_none() && app.cursor_arrasto.is_none());
}

#[cfg(test)]
mod testes_do_app {
    #[test]
    fn conector_pede_origem_e_destino_quando_nao_ha_selecao() {
        let mut app=FeatherSlideApp::default();app.sessao.slide_novo("em_branco").unwrap();
        for r in [[10.0,10.0,100.0,100.0],[300.0,200.0,100.0,100.0]] {app.sessao.elemento_inserir(Tipo::Retangulo,Some(r),None).unwrap();}
        app.sessao.selecionar_no_ponto(-100.0,-100.0,false);
        let _=app.update(Mensagem::Faixa(FaixaMensagem::InserirConector("reto")));
        assert_eq!(app.conector_modo,Some("reto"));assert!(app.conector_origem.is_none());
        let _=app.update(Mensagem::Palco(PalcoMensagem::Clicou{x:50.0,y:50.0,somar:false,duplo:false}));
        assert!(app.conector_origem.is_some());assert_eq!(app.sessao.slide().elementos.len(),2);
        let _=app.update(Mensagem::Palco(PalcoMensagem::Clicou{x:350.0,y:250.0,somar:false,duplo:false}));
        assert!(app.conector_modo.is_none());assert_eq!(app.sessao.slide().elementos.len(),3);
        assert!(app.sessao.slide().elementos.last().unwrap().dados.contains("#CON "));
        app.sessao.desfazer().unwrap();assert_eq!(app.sessao.slide().elementos.len(),2);
        app.sessao.selecionar_no_ponto(-100.0,-100.0,false);
        let _=app.update(Mensagem::Faixa(FaixaMensagem::InserirConector("curvo")));
        tecla(&mut app,keyboard::Key::Named(keyboard::key::Named::Escape),None,false);
        assert!(app.conector_modo.is_none());assert_eq!(app.sessao.slide().elementos.len(),2);
    }
    #[test]
    fn galerias_e_desenho_livre_pela_interface() {
        let mut app=FeatherSlideApp::default();
        let _=app.update(Mensagem::Faixa(FaixaMensagem::InserirGaleria("fala")));
        assert!(app.sessao.slide().elementos.last().unwrap().dados.starts_with("SVG1\n"));
        let _=app.update(Mensagem::Faixa(FaixaMensagem::DesenhoLivre));assert!(app.desenho_livre);
        let _=app.update(Mensagem::Palco(PalcoMensagem::DesenhoLivre(vec![[10.0,20.0],[50.0,60.0]])));
        assert!(!app.desenho_livre);assert!(app.sessao.slide().elementos.last().unwrap().preenchimento.is_empty());
        let _=app.update(Mensagem::Faixa(FaixaMensagem::DesenhoLivre));
        let antes=app.sessao.slide().elementos.len();let _=app.update(Mensagem::Palco(PalcoMensagem::CancelarDesenho));
        assert!(!app.desenho_livre);assert_eq!(app.sessao.slide().elementos.len(),antes);
    }
    #[test]
    fn configuracao_slide_pela_faixa_aplica_valida_e_cancela_rascunho() {
        use crate::configuracao_slide::Msg as C;
        let mut app = FeatherSlideApp::default();
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::ConfiguracaoSlide));
        let _ = app.update(Mensagem::ConfiguracaoSlide(C::Texto("largura", "400".into())));
        let _ = app.update(Mensagem::ConfiguracaoSlide(C::Texto("altura", "700".into())));
        assert_eq!(app.sessao.apresentacao().tamanho(), (960.0, 540.0));
        let _ = app.update(Mensagem::ConfiguracaoSlide(C::AplicarTamanho));
        assert_eq!(app.sessao.apresentacao().tamanho(), (400.0, 700.0));
        let _ = app.update(Mensagem::ConfiguracaoSlide(C::Tipo("gradiente".into())));
        let _ = app.update(Mensagem::ConfiguracaoSlide(C::Texto("opacidade", "37,5".into())));
        let _ = app.update(Mensagem::ConfiguracaoSlide(C::AplicarFundo));
        assert_eq!(app.sessao.slide().fundo_config.opacidade, 0.375);
        let antes = app.sessao.apresentacao().clone();
        let _ = app.update(Mensagem::ConfiguracaoSlide(C::Texto("largura", "NaN".into())));
        let _ = app.update(Mensagem::ConfiguracaoSlide(C::AplicarTamanho));
        assert_eq!(app.sessao.apresentacao(), &antes);
        assert!(app.configuracao_slide.as_ref().unwrap().aviso.contains("dimensão"));
        let _ = app.update(Mensagem::ConfiguracaoSlide(C::Fechar));
        assert!(app.configuracao_slide.is_none());
    }

    use super::*;

    #[test]
    fn falha_ao_salvar_preserva_alteracoes_e_caminho_da_importacao() {
        let mut app = FeatherSlideApp::default();
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::Inserir(Tipo::Retangulo)));
        let pasta = std::env::temp_dir().join(format!("ftslide-gui-salvar-{}", std::process::id()));
        std::fs::create_dir_all(pasta.join("original.fsdz")).unwrap();
        let original = pasta.join("original.pptx");
        std::fs::write(&original, b"importacao preservada").unwrap();
        app.caminho_arquivo = Some(original.clone());
        assert!(!app.gravar_no_caminho());
        assert!(app.sessao.estado().sujo);
        assert_eq!(app.caminho_arquivo.as_ref(), Some(&original));
        assert!(app.status.starts_with("Não consegui salvar:"));
        assert_eq!(std::fs::read(&original).unwrap(), b"importacao preservada");
        assert_eq!(std::fs::read_dir(&pasta).unwrap().count(), 2);
        std::fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn arredondar_muda_os_ajustes_da_forma_selecionada() {
        let mut app = FeatherSlideApp::default();
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::Inserir(Tipo::Retangulo)));
        assert_eq!(app.sessao.estado().selecao.len(), 1);
        assert_eq!(app.contexto_faixa().tipo_sel, "retangulo");
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::Arredondar(0.08)));
        let id = app.sessao.estado().selecao[0];
        let e = app.sessao.slide().elementos.iter().find(|e| e.id == id).unwrap();
        assert_eq!(e.ajustes, vec![0.08; 4]);
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::CantoSel(Some(2))));
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::Arredondar(0.08)));
        let e = app.sessao.slide().elementos.iter().find(|e| e.id == id).unwrap();
        assert_eq!(e.ajustes, vec![0.08, 0.08, 0.16, 0.08]);
    }

    #[test]
    fn ortografia_acha_troca_e_conclui() {
        use crate::ortografia::OrtMsg as O;
        let mut app = FeatherSlideApp::default();
        let id = app.sessao.apresentacao().slides[0].elementos[0].id;
        app.sessao.texto_definir(id, "casa bnita").unwrap();
        app.corretor = Some(featherslide_ortografia::Corretor::de_texto("SET UTF-8\n", "3\ncasa\nbonita\nxx\n").unwrap());
        app.ortografia = Some(crate::ortografia::Ortografia::nova());
        let _ = app.ortografia_iniciar();
        assert_eq!(app.ortografia.as_ref().unwrap().erros.len(), 1);
        let _ = app.update(Mensagem::Ortografia(O::Texto("bonita".into())));
        let _ = app.update(Mensagem::Ortografia(O::Alterar));
        assert!(app.ortografia.as_ref().unwrap().erros.is_empty());
        assert_eq!(app.sessao.slide().elementos[0].texto, "casa bonita");
        let _ = app.update(Mensagem::Ortografia(O::Fechar));
        assert!(app.ortografia.is_none() && app.corretor.is_some());
    }

    #[test]
    fn botao_direito_na_palavra_errada_abre_correcoes_e_troca() {
        use crate::menu_contexto::TipoMenu;
        let mut app = FeatherSlideApp::default();
        app.config.ortografia_automatica = true;
        let id = app.sessao.apresentacao().slides[0].elementos[0].id;
        app.sessao.texto_definir(id, "casa bnita").unwrap();
        app.corretor = Some(featherslide_ortografia::Corretor::de_texto("SET UTF-8\n", "3\ncasa\nbonita\nxx\n").unwrap());
        let ondulados = app.ondulados_atuais();
        assert_eq!(ondulados.len(), 1);
        let w = &ondulados[0];
        // Clique num ponto sem erro: menu comum. Sobre a palavra: menu de correção.
        let _ = app.update(Mensagem::Palco(PalcoMensagem::MenuContexto { x: 1.0, y: 1.0, janela: Point::new(5.0, 5.0) }));
        assert!(app.erro_do_menu.is_none());
        let _ = app.update(Mensagem::Palco(PalcoMensagem::MenuContexto { x: w.x + 1.0, y: w.y - 2.0, janela: Point::new(5.0, 5.0) }));
        assert!(matches!(app.menu.as_ref().map(|m| &m.tipo), Some(TipoMenu::Ortografia { .. })));
        assert_eq!(app.erro_do_menu.as_ref().map(|e| e.palavra.as_str()), Some("bnita"));
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::OrtografiaTrocar("bonita".into())));
        assert_eq!(app.sessao.slide().elementos[0].texto, "casa bonita");
        assert!(app.ondulados_atuais().is_empty());
        // Desligado, não há ondulação.
        app.sessao.texto_definir(id, "casa bnita").unwrap();
        app.config.ortografia_automatica = false; // direto, sem `OrtografiaAuto`: ele grava o config.json do usuário
        assert!(app.ondulados_atuais().is_empty());
    }

    #[test]
    fn salvar_copia_preserva_original_caminho_e_alteracoes() {
        let mut app = FeatherSlideApp::default();
        let pasta = std::env::temp_dir().join(format!("fs-copia-{}", std::process::id()));
        std::fs::create_dir_all(&pasta).unwrap();
        let original = pasta.join("original.fsdz");
        app.caminho_arquivo = Some(original.clone());
        assert!(app.gravar_no_caminho());
        let antes = std::fs::read(&original).unwrap();
        app.sessao.wordart_inserir().unwrap();
        let nome = app.nome_arquivo.clone();
        assert!(app.salvar_copia(pasta.join("copia.pptx")));
        let copia = fsdz::ler(&std::fs::read(pasta.join("copia.fsdz")).unwrap()).unwrap();
        assert_eq!(copia.slides[0].elementos.len(), app.sessao.slide().elementos.len());
        assert_eq!(std::fs::read(&original).unwrap(), antes);
        assert_eq!(app.caminho_arquivo, Some(original.clone()));
        assert_eq!(app.nome_arquivo, nome);
        assert!(app.sessao.estado().sujo);
        assert!(!app.salvar_copia(original));
        assert!(!app.salvar_copia(pasta.join("inexistente/copia.fsdz")));
        assert!(app.sessao.estado().sujo);
        std::fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn automatico_respeita_tempo_real_e_salva_rascunho_sem_trocar_documento() {
        let mut app = FeatherSlideApp::default();
        app.config.auto_salvar = true;
        app.config.intervalo = 5;
        app.rascunho = std::env::temp_dir().join(format!("fs-auto-{}.fsdz", std::process::id()));
        app.sessao.wordart_inserir().unwrap();
        let inicio = std::time::Instant::now();
        app.auto_inicio = inicio;
        for _ in 0..20 { app.salvar_automaticamente(inicio); }
        assert!(app.auto_versao.is_none());
        app.salvar_automaticamente(inicio + std::time::Duration::from_secs(5));
        let bytes = std::fs::read(&app.rascunho).unwrap();
        assert!(fsdz::ler(&bytes).is_ok());
        assert!(app.caminho_arquivo.is_none());
        assert!(app.sessao.estado().sujo);
        let versao = app.auto_versao;
        app.salvar_automaticamente(inicio + std::time::Duration::from_secs(20));
        assert_eq!(app.auto_versao, versao);
        app.config.auto_salvar = false;
        app.sessao.wordart_inserir().unwrap();
        app.salvar_automaticamente(inicio + std::time::Duration::from_secs(30));
        assert_eq!(std::fs::read(&app.rascunho).unwrap(), bytes);
        std::fs::remove_file(&app.rascunho).unwrap();
    }

    #[test]
    fn automatico_grava_arquivo_nomeado_e_preserva_sujo_em_falha() {
        let mut app = FeatherSlideApp::default();
        app.config.auto_salvar = true;
        app.config.intervalo = 1;
        let pasta = std::env::temp_dir().join(format!("fs-auto-nomeado-{}", std::process::id()));
        std::fs::create_dir_all(&pasta).unwrap();
        app.caminho_arquivo = Some(pasta.join("teste.fsdz"));
        app.sessao.wordart_inserir().unwrap();
        let agora = app.auto_inicio + std::time::Duration::from_secs(2);
        app.salvar_automaticamente(agora);
        assert!(!app.sessao.estado().sujo);
        app.sessao.wordart_inserir().unwrap();
        app.caminho_arquivo = Some(pasta.join("ausente/teste.fsdz"));
        app.salvar_automaticamente(agora + std::time::Duration::from_secs(2));
        assert!(app.sessao.estado().sujo);
        assert!(app.status.contains("Não consegui"));
        std::fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn pdf_wordart_preserva_transformacao_sem_texto_duplicado() {
        let mut app = FeatherSlideApp::default();
        app.sessao.wordart_inserir().unwrap();
        let id = app.sessao.estado().selecao[0];
        app.sessao.texto_definir(id, "WORDARTPDF").unwrap();
        let destino = std::env::temp_dir().join(format!("fs-wordart-{}.pdf", std::process::id()));
        app.exportar_pdf(&destino, true).unwrap();
        let reto = std::fs::read(&destino).unwrap();
        app.sessao.wordart_transformacao(featherslide_core::wordart::Transformacao::Arco, 1.0).unwrap();
        app.exportar_pdf(&destino, true).unwrap();
        let arco = std::fs::read(&destino).unwrap();
        assert_ne!(reto, arco, "a transformação precisa alterar os contornos exportados");
        assert!(!String::from_utf8_lossy(&arco).contains("WORDARTPDF"), "não deve exportar uma segunda cópia em texto comum");
        std::fs::remove_file(destino).unwrap();
    }

    #[test]
    fn ctrl_p_abre_a_caixa_de_impressao_e_limita_as_copias() {
        use crate::imprimir::ImprMsg as I;
        let mut app = FeatherSlideApp::default();
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::Imprimir));
        assert!(app.impressao.is_some());
        for _ in 0..200 {
            let _ = app.update(Mensagem::Imprimir(I::Copias(1)));
        }
        assert_eq!(app.impressao.as_ref().unwrap().copias, 99);
        for _ in 0..200 {
            let _ = app.update(Mensagem::Imprimir(I::Copias(-1)));
        }
        assert_eq!(app.impressao.as_ref().unwrap().copias, 1);
        // Sem impressora no sistema, "Salvar como PDF" já vem marcado; escolher uma impressora o desmarca.
        let sem_impressora = app.impressao.as_ref().unwrap().impressoras.is_empty();
        assert_eq!(app.impressao.as_ref().unwrap().pdf, sem_impressora);
        let _ = app.update(Mensagem::Imprimir(I::Pdf));
        assert!(app.impressao.as_ref().unwrap().pdf && app.impressao.as_ref().unwrap().impressora().is_none());
        let _ = app.update(Mensagem::Imprimir(I::SoAtual(true)));
        assert!(app.impressao.as_ref().unwrap().so_atual);
        let _ = app.update(Mensagem::Imprimir(I::Cancelar));
        assert!(app.impressao.is_none());
        // O atalho padrão é Ctrl+P.
        assert_eq!(app.config.comando_de("Ctrl+P"), Some("imprimir"));
        // O PDF de um slide só tem uma página.
        let destino = std::env::temp_dir().join(format!("fs-teste-{}.pdf", std::process::id()));
        assert_eq!(app.exportar_pdf(&destino, true), Ok(1));
        let _ = std::fs::remove_file(destino);
    }

    /// Aperta uma tecla (com o texto que ela digita, se for letra) como o teclado faria.
    fn tecla(app: &mut FeatherSlideApp, key: iced::keyboard::Key, texto: Option<&str>, ctrl: bool) {
        let modifiers = if ctrl { iced::keyboard::Modifiers::CTRL } else { iced::keyboard::Modifiers::default() };
        let _ = app.update(Mensagem::Teclado(iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            modified_key: key.clone(),
            key,
            physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::KeyA),
            location: iced::keyboard::Location::Standard,
            modifiers,
            text: texto.map(Into::into),
        })));
    }

    #[test]
    fn cursor_volta_no_texto_e_escreve_no_meio() {
        use iced::keyboard::key::Named::{ArrowLeft, ArrowUp, Backspace, Delete, End, Home};
        use iced::keyboard::Key::Named as N;
        let mut app = FeatherSlideApp::default();
        let id = app.sessao.apresentacao().slides[0].elementos[0].id;
        app.sessao.texto_definir(id, "casa bonita").unwrap();
        app.editando_id = Some(id);
        app.texto_edicao = "casa bonita".into();
        app.cursor_edicao = app.texto_edicao.len();
        // Voltar 7 letras e escrever "grande " no meio.
        for _ in 0..7 {
            tecla(&mut app, N(ArrowLeft), None, false);
        }
        assert_eq!(app.cursor_edicao, "casa".len() - 0 + 0);
        for ch in "grande".chars() {
            tecla(&mut app, iced::keyboard::Key::Character(ch.to_string().into()), Some(&ch.to_string()), false);
        }
        assert_eq!(app.sessao.slide().elementos[0].texto, "casagrande bonita");
        // Backspace apaga antes do cursor, não no fim.
        tecla(&mut app, N(Backspace), None, false);
        assert_eq!(app.sessao.slide().elementos[0].texto, "casagrand bonita");
        // Delete apaga depois.
        tecla(&mut app, N(Delete), None, false);
        assert_eq!(app.sessao.slide().elementos[0].texto, "casagrandbonita");
        // Home e End; Ctrl+Seta salta palavra (aqui só há uma, então vai ao começo).
        tecla(&mut app, N(Home), None, false);
        assert_eq!(app.cursor_edicao, 0);
        tecla(&mut app, N(End), None, false);
        assert_eq!(app.cursor_edicao, app.texto_edicao.len());
        tecla(&mut app, N(ArrowLeft), None, true);
        assert_eq!(app.cursor_edicao, 0);
        // Seta para cima numa linha só vai ao começo.
        app.cursor_edicao = 5;
        tecla(&mut app, N(ArrowUp), None, false);
        assert_eq!(app.cursor_edicao, 0);
        assert_eq!(app.editando_id, Some(id));
    }

    #[test]
    fn link_e_comentario_pela_interface() {
        use crate::revisao::RevisaoMsg as R;
        let mut app = FeatherSlideApp::default();
        let _ = app.sessao.slide_novo("em_branco");
        // Sem seleção, o Link não abre.
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::LinkAbrir));
        assert!(app.revisao.is_none());
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::Inserir(Tipo::Retangulo)));
        let id = app.sessao.estado().selecao[0];
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::LinkAbrir));
        let _ = app.update(Mensagem::Revisao(R::Texto("exemplo.com".into())));
        let _ = app.update(Mensagem::Revisao(R::Confirmar));
        assert!(app.revisao.is_none());
        assert_eq!(app.sessao.link_do_elemento(app.sessao.atual(), id).as_deref(), Some("https://exemplo.com"));
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::LinkAbrir));
        let _ = app.update(Mensagem::Revisao(R::RemoverLink));
        assert!(app.sessao.link_do_elemento(app.sessao.atual(), id).is_none());
        // Comentário: a caixa continua aberta e limpa depois de comentar.
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::ComentarioAbrir));
        let _ = app.update(Mensagem::Revisao(R::Texto("rever".into())));
        let _ = app.update(Mensagem::Revisao(R::Confirmar));
        assert_eq!(app.sessao.slide().comentarios.len(), 1);
        let _ = app.update(Mensagem::Revisao(R::Resolver(0, true)));
        assert!(app.sessao.slide().comentarios[0].resolvido);
        let _ = app.update(Mensagem::Revisao(R::Excluir(0)));
        assert!(app.sessao.slide().comentarios.is_empty());
    }

    #[test]
    fn ctrl_g_agrupa_e_desagrupa() {
        let mut app = FeatherSlideApp::default();
        let _ = app.sessao.slide_novo("em_branco");
        for _ in 0..2 {
            let _ = app.update(Mensagem::Faixa(FaixaMensagem::Inserir(Tipo::Retangulo)));
        }
        app.sessao.selecionar_tudo();
        let grupos = |a: &FeatherSlideApp| a.sessao.slide().elementos.iter().filter(|e| e.grupo != 0).count();
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::AgruparOuDesagrupar));
        assert_eq!(grupos(&app), 2);
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::AgruparOuDesagrupar));
        assert_eq!(grupos(&app), 0);
    }

    #[test]
    fn faixa_de_imagem_nao_pode_disparar_panic_do_scrollable_iced() {
        let mut app = FeatherSlideApp::default();
        app.sessao.slide_novo("em_branco").unwrap();
        app.sessao.elemento_inserir(
            Tipo::Imagem,
            Some([10.0, 10.0, 240.0, 140.0]),
            Some("data:image/png;base64,teste".into()),
        ).unwrap();

        // Reproduz o caminho que disparava "scrollable content must not fill its
        // vertical scrolling axis" ao abrir Formato da Imagem no Iced 0.13.
        for detalhes in [false, true] {
            app.imagem_fundo_detalhes = detalhes;
            let contexto = app.contexto_faixa();
            assert_eq!(contexto.tipo_sel, "imagem");
            let _faixa = view_faixa(
                AbaDaFaixa::Imagem,
                false, false, true,
                false, false, false, false,
                "Arial", "12", &contexto,
            );
        }
    }

    #[test]
    fn opacidade_digitada_e_grupos_de_imagem_na_faixa() {
        let mut app = FeatherSlideApp::default();
        app.sessao.slide_novo("em_branco").unwrap();
        app.sessao.elemento_inserir(
            Tipo::Imagem, Some([90.0, 40.0, 220.0, 140.0]),
            Some("data:image/png;base64,teste".into())
        ).unwrap();
        let id = app.sessao.estado().selecao[0];
        let antes = app.sessao.slide().elementos.iter().find(|e| e.id == id).unwrap().opacidade;
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::ImagemOpacidadeTexto("37".into())));
        assert_eq!(app.sessao.slide().elementos.iter().find(|e| e.id == id).unwrap().opacidade, antes,
            "digitar não cria várias revisões até confirmar");
        assert_eq!(app.contexto_faixa().imagem_opacidade_texto, "37");
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::ImagemOpacidadeAplicar));
        assert!((app.sessao.slide().elementos.iter().find(|e| e.id == id).unwrap().opacidade - 0.37).abs() < 0.001);
        assert_eq!(app.contexto_faixa().imagem_opacidade_texto, "37");
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::ImagemOpacidadeTexto("180".into())));
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::ImagemOpacidadeAplicar));
        assert!((app.sessao.slide().elementos.iter().find(|e| e.id == id).unwrap().opacidade - 0.37).abs() < 0.001,
            "valores acima de 100 são rejeitados");
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::ImagemFundoDetalhes(true)));
        assert!(app.contexto_faixa().imagem_fundo_detalhes);
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::ImagemAjusteSelecionado("Saturação")));
        assert_eq!(app.contexto_faixa().imagem_ajuste_selecionado, "Saturação");
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::ImagemVisual("saturacao", 1.25)));
        assert!((app.contexto_faixa().visual_imagem.saturacao-1.25).abs() < 0.001);
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::ImagemBordaSelecionada("Direita")));
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::ImagemCortarMargem("direita",0.15)));
        assert!((app.contexto_faixa().margens_imagem[2]-0.15).abs() < 0.001);
    }

    #[test]
    fn enquadramento_de_recorte_da_imagem_via_faixa_e_palco() {
        let mut app = FeatherSlideApp::default();
        let _ = app.sessao.slide_novo("em_branco");
        let _ = app.sessao.elemento_inserir(Tipo::Imagem, Some([100.0, 100.0, 200.0, 200.0]), Some("data:image/png;base64,teste".into()));
        let id = app.sessao.estado().selecao[0];

        // Corta como elipse
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::RecorteForma(Tipo::Elipse)));
        let el = app.sessao.slide().elementos.iter().find(|e| e.id == id).unwrap();
        assert_eq!(el.recorte, Tipo::Elipse);

        // Ativa modo de enquadramento
        assert_eq!(app.ajustando_recorte, None);
        assert!(!app.contexto_faixa().modo_recorte);
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::AlternarModoRecorte));
        assert_eq!(app.ajustando_recorte, Some(id));
        assert!(app.contexto_faixa().modo_recorte);

        // Ajusta via faixa (botões de deslocamento / zoom)
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::ImagemAjustarRecorte { dx: 0.1, dy: -0.1, zoom: 0.2 }));
        let el = app.sessao.slide().elementos.iter().find(|e| e.id == id).unwrap();
        assert_eq!(el.ajustes[0], 0.1);
        assert_eq!(el.ajustes[1], -0.1);
        assert_eq!(el.ajustes[2], 1.2);

        // Ajusta via palco (arrasto e zoom pelo mouse)
        let _ = app.update(Mensagem::Palco(PalcoMensagem::ArrastouRecorte { id, dx: 20.0, dy: 10.0 }));
        let el = app.sessao.slide().elementos.iter().find(|e| e.id == id).unwrap();
        assert!((el.ajustes[0] - 0.2).abs() < 1e-4);
        assert!((el.ajustes[1] - (-0.05)).abs() < 1e-4);

        let _ = app.update(Mensagem::Palco(PalcoMensagem::ZoomRecorte { id, delta: 0.3 }));
        let el = app.sessao.slide().elementos.iter().find(|e| e.id == id).unwrap();
        assert!((el.ajustes[2] - 1.5).abs() < 1e-4);

        // Teclado Escape sai do modo
        let _ = app.update(Mensagem::Teclado(iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            modified_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::Escape),
            location: iced::keyboard::Location::Standard,
            modifiers: iced::keyboard::Modifiers::default(),
            text: None,
        })));
        assert_eq!(app.ajustando_recorte, None);

        // Resetar recorte
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::ImagemResetarRecorte));
        let el = app.sessao.slide().elementos.iter().find(|e| e.id == id).unwrap();
        assert_eq!(el.ajustes[0], 0.0);
        assert_eq!(el.ajustes[1], 0.0);
        assert_eq!(el.ajustes[2], 1.0);
    }
}


/// O login do sistema (`USER` no Linux e no macOS, `USERNAME` no Windows); vazio se não houver.
fn nome_do_usuario() -> String {
    std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_default()
}

/// Abre o endereço no navegador ou no cliente de e-mail do sistema.
fn abrir_endereco(url: &str) -> std::io::Result<std::process::Child> {
    #[cfg(target_os = "windows")]
    return std::process::Command::new("rundll32").arg("url.dll,FileProtocolHandler").arg(url).spawn();
    #[cfg(target_os = "macos")]
    return std::process::Command::new("open").arg(url).spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    return std::process::Command::new("xdg-open").arg(url).spawn();
}

#[cfg(test)]
#[test]
fn passar_mouse_em_efeitos_nao_edita_o_documento() {
    use featherslide_core::modelo::{TipoAnimacao, TipoTransicao};
    let mut app = FeatherSlideApp::default();
    let id = app.sessao.slide().elementos[0].id;
    app.sessao.selecionar_por_id(id);
    let antes = app.sessao.apresentacao().clone();
    let selecao = app.sessao.estado().selecao;
    let _ = app.tratar_faixa(FaixaMensagem::PreviaTransicao(TipoTransicao::Deslizar));
    assert!(app.previa.is_some());
    let _ = app.tratar_faixa(FaixaMensagem::PreviaAnimacao(TipoAnimacao::Zoom));
    assert!(app.previa.is_some());
    assert_eq!(app.sessao.apresentacao(), &antes);
    assert_eq!(app.sessao.estado().selecao, selecao);
    let _ = app.tratar_faixa(FaixaMensagem::PreviaEncerrar);
    assert!(app.previa.is_some());
    assert_eq!(app.sessao.apresentacao(), &antes);
}

#[cfg(test)]
#[test]
fn clicar_no_efeito_aplica_e_reproduz_ate_o_fim() {
    use featherslide_core::modelo::{TipoAnimacao, TipoTransicao};
    let mut app = FeatherSlideApp::default();
    let id = app.sessao.slide().elementos[0].id;
    app.sessao.selecionar_por_id(id);
    let _ = app.tratar_faixa(FaixaMensagem::TransicaoTipo(TipoTransicao::Zoom));
    assert_eq!(app.sessao.slide().transicao.tipo, TipoTransicao::Zoom);
    assert!(app.previa.as_ref().unwrap().programa().is_some());
    let _ = app.tratar_faixa(FaixaMensagem::AnimTipo(TipoAnimacao::Pulsar));
    assert_eq!(app.sessao.slide().elementos[0].animacao.tipo, TipoAnimacao::Pulsar);
    let _ = app.tratar_faixa(FaixaMensagem::PreviaEncerrar);
    assert!(app.previa.is_some());
}

#[cfg(test)]
#[test]
fn arrastar_cabecalho_move_secao_com_slides_para_destino_distante() {
    let mut app = FeatherSlideApp::default();
    for _ in 0..4 { app.sessao.slide_novo("em_branco").unwrap(); }
    for (inicio, nome) in [(0, "A"), (2, "B"), (4, "C")] {
        app.sessao.slide_ir(inicio).unwrap();
        app.sessao.secao_adicionar(nome).unwrap();
    }
    let ids: Vec<_> = app.sessao.apresentacao().slides.iter().map(|sl| sl.id).collect();
    app.secoes_recolhidas.insert(0);
    let _ = app.update(Mensagem::Miniatura(MiniaturaMensagem::PressionouSecao(0, Point::new(22.0, 48.0))));
    let _ = app.update(Mensagem::CursorMoveu(Point::new(28.0, 172.0)));
    let _ = app.update(Mensagem::Miniatura(MiniaturaMensagem::SoltouSecao(4)));
    assert_eq!(app.sessao.secoes().iter().map(|s| s.nome.as_str()).collect::<Vec<_>>(), vec!["B", "C", "A"]);
    assert_eq!(app.sessao.apresentacao().slides.iter().map(|sl| sl.id).collect::<Vec<_>>(),
        vec![ids[2], ids[3], ids[4], ids[0], ids[1]]);
    assert_eq!(app.secoes_recolhidas, [3].into());
    assert!(app.arrastando_secao.is_none() && app.cursor_arrasto.is_none());
}

#[cfg(test)]
#[test]
fn clicar_ou_arrastar_no_proprio_cabecalho_nao_move_secao() {
    let mut app = FeatherSlideApp::default();
    app.sessao.secao_adicionar("Uma").unwrap();
    let _ = app.update(Mensagem::Miniatura(MiniaturaMensagem::PressionouSecao(0, Point::new(20.0, 30.0))));
    let _ = app.update(Mensagem::Miniatura(MiniaturaMensagem::SoltouSecao(0)));
    assert!(app.secoes_recolhidas.contains(&0));
    let _ = app.update(Mensagem::Miniatura(MiniaturaMensagem::PressionouSecao(0, Point::new(20.0, 30.0))));
    let _ = app.update(Mensagem::CursorMoveu(Point::new(40.0, 70.0)));
    let _ = app.update(Mensagem::Miniatura(MiniaturaMensagem::SoltouSecao(0)));
    assert!(app.secoes_recolhidas.contains(&0), "arrastar e soltar no mesmo grupo não recolhe/expande");
    assert_eq!(app.sessao.secoes().len(), 1);
    let _ = app.update(Mensagem::Miniatura(MiniaturaMensagem::PressionouSecao(0, Point::new(20.0, 30.0))));
    let _ = app.update(Mensagem::LimparArraste);
    assert!(app.arrastando_secao.is_none());
}

#[cfg(test)]
#[test]
fn galeria_de_diagramas_insere_grupo_no_slide_sem_rasterizar() {
    use featherslide_core::diagramas::Categoria;
    let mut app = FeatherSlideApp::default();
    let qtde = app.sessao.slide().elementos.len();
    let _ = app.update(Mensagem::Faixa(FaixaMensagem::AbrirFiguras(
        crate::galeria_figuras::Figuras::Diagramas
    )));
    assert_eq!(app.galeria_figuras, Some(crate::galeria_figuras::Figuras::Diagramas));
    let _ = app.update(Mensagem::Faixa(FaixaMensagem::DiagramaInserir(Categoria::Venn)));
    assert!(app.galeria_figuras.is_none());
    assert!(app.sessao.slide().elementos.len() > qtde);
    assert!(app.sessao.slide().elementos[qtde..].iter().all(|e| e.grupo > 0));
    app.sessao.desfazer().unwrap();
    assert_eq!(app.sessao.slide().elementos.len(), qtde);
}

#[cfg(test)]
mod testes_personalizacao {
    use super::*;
    #[test]
    fn indice_aplica_apenas_ao_trecho_em_edicao() {
        let mut app = FeatherSlideApp::default();
        app.sessao
            .elemento_inserir(Tipo::Retangulo, None, None)
            .unwrap();
        let id = app.sessao.slide().elementos.last().unwrap().id;
        app.sessao.texto_definir(id, "x2").unwrap();
        app.editando_id = Some(id);
        app.texto_edicao = "x2".into();
        app.ancora_edicao = Some(1);
        app.cursor_edicao = 2;
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::IndiceTexto(1)));
        let e = app.sessao.slide().elementos.last().unwrap();
        assert_eq!(featherslide_core::texto_formato::nivel(e, 0), 0);
        assert_eq!(featherslide_core::texto_formato::nivel(e, 1), 1);
    }
    #[test]
    fn caixa_no_trecho_em_edicao_acompanha_a_selecao() {
        let mut app = FeatherSlideApp::default();
        app.sessao.elemento_inserir(Tipo::Retangulo, None, None).unwrap();
        let id = app.sessao.slide().elementos.last().unwrap().id;
        app.sessao.texto_definir(id, "rua straße").unwrap();
        app.editando_id = Some(id);
        app.texto_edicao = "rua straße".into();
        app.ancora_edicao = Some(4);
        app.cursor_edicao = "rua straße".len();
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::Caixa(featherslide_core::texto_formato::Caixa::Maiusculas)));
        assert_eq!(app.sessao.slide().elementos.last().unwrap().texto, "rua STRASSE");
        assert_eq!(app.texto_edicao, "rua STRASSE");
        assert_eq!((app.ancora_edicao, app.cursor_edicao), (Some(4), "rua STRASSE".len()));
    }
    #[test]
    fn recuo_muda_linhas_e_nao_a_fonte() {
        let mut app = FeatherSlideApp::default();
        app.sessao.elemento_inserir(Tipo::Retangulo, None, None).unwrap();
        let id = app.sessao.slide().elementos.last().unwrap().id;
        app.sessao.texto_definir(id, "• um\n• dois").unwrap();
        app.sessao.selecionar_por_id(id);
        let tamanho = app.sessao.estado().formato.unwrap().tamanho;
        app.editando_id = Some(id);
        app.texto_edicao = "• um\n• dois".into();
        app.ancora_edicao = None;
        app.cursor_edicao = app.texto_edicao.len();
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::AumentarRecuo));
        assert_eq!(app.sessao.slide().elementos.last().unwrap().texto, "• um\n    • dois");
        assert_eq!(app.texto_edicao, "• um\n    • dois");
        assert_eq!(app.cursor_edicao, app.texto_edicao.len());
        assert_eq!(app.sessao.estado().formato.unwrap().tamanho, tamanho);
        app.editando_id = None;
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::DiminuirRecuo));
        assert_eq!(app.sessao.slide().elementos.last().unwrap().texto, "• um\n• dois");
    }
    #[test]
    fn traco_duplo_liga_e_desliga() {
        let mut app = FeatherSlideApp::default();
        app.sessao.elemento_inserir(Tipo::Texto, None, None).unwrap();
        let id = app.sessao.slide().elementos.last().unwrap().id;
        app.sessao.selecionar_por_id(id);
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::SublinhadoDuplo));
        assert!(app.contexto_faixa().sublinhado_duplo);
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::Sublinhado));
        let f = app.sessao.estado().formato.unwrap();
        assert!(!f.sublinhado && !f.sublinhado_duplo);
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::TachadoDuplo));
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::TachadoDuplo));
        assert!(!app.contexto_faixa().tachado_duplo);
    }
    #[test]
    fn indice_ligado_antes_de_digitar() {
        let mut app = FeatherSlideApp::default();
        app.sessao.elemento_inserir(Tipo::Retangulo, None, None).unwrap();
        let id = app.sessao.slide().elementos.last().unwrap().id;
        app.sessao.texto_definir(id, "x").unwrap();
        app.sessao.selecionar_por_id(id);
        app.editando_id = Some(id);
        app.texto_edicao = "x".into();
        app.ancora_edicao = None;
        app.cursor_edicao = 1;
        let digitar = |app: &mut FeatherSlideApp, t: &str| {
            let _ = app.update(Mensagem::Teclado(iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(t.into()),
                modified_key: keyboard::Key::Character(t.into()),
                physical_key: keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified),
                location: keyboard::Location::Standard,
                modifiers: keyboard::Modifiers::empty(),
                text: Some(t.into()),
            })));
        };
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::IndiceTexto(1)));
        assert_eq!(app.contexto_faixa().indice_texto, 1);
        digitar(&mut app, "2");
        digitar(&mut app, "3");
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::IndiceTexto(1)));
        digitar(&mut app, "y");
        let e = app.sessao.slide().elementos.last().unwrap();
        assert_eq!(e.texto, "x23y");
        let n = |b| featherslide_core::texto_formato::nivel(e, b);
        assert_eq!((n(0), n(1), n(2), n(3)), (0, 1, 1, 0));
        assert_eq!(e.indices_texto.len(), 1);
    }
    #[test]
    fn personalizacao_cancelar_e_aplicar() {
        let mut app = FeatherSlideApp::default();
        app.sessao
            .elemento_inserir(Tipo::Retangulo, None, None)
            .unwrap();
        let antes = app.sessao.apresentacao().clone();
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::FormatoForma));
        assert!(app.formato_forma.is_some());
        let _ = app.update(Mensagem::FormatoForma(crate::formato_forma::Msg::Tipo(
            "Linear".into(),
        )));
        let _ = app.update(Mensagem::FormatoForma(crate::formato_forma::Msg::Fechar));
        assert_eq!(*app.sessao.apresentacao(), antes);
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::FormatoForma));
        let _ = app.update(Mensagem::FormatoForma(crate::formato_forma::Msg::Tipo(
            "Radial".into(),
        )));
        let _ = app.update(Mensagem::FormatoForma(crate::formato_forma::Msg::Aplicar));
        assert!(app
            .sessao
            .slide()
            .elementos
            .last()
            .unwrap()
            .gradiente
            .is_some());
        app.sessao.desfazer().unwrap();
        assert_eq!(app.sessao.apresentacao().slides, antes.slides);
    }
    #[test]
    fn todos_os_slides_abre_slide_clicado() {
        let mut app = FeatherSlideApp::default();
        app.sessao.slide_novo("em_branco").unwrap();
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::Classificacao));
        assert!(app.classificacao);
        let _ = app.update(Mensagem::ClassificacaoClique(0));
        assert!(!app.classificacao);
        assert_eq!(app.sessao.atual(), 0);
    }
}

#[cfg(test)]
#[test]
fn painel_forma_acompanha_nova_selecao(){
    let mut app=FeatherSlideApp::default();
    app.sessao.elemento_inserir(Tipo::Retangulo,None,None).unwrap();
    let primeiro=app.sessao.slide().elementos.last().unwrap().id;
    app.sessao.elemento_inserir(Tipo::Elipse,None,None).unwrap();
    app.sessao.formatar(Formatacao::Preenchimento("#ff8800".into())).unwrap();
    let segundo=app.sessao.slide().elementos.last().unwrap().id;
    app.sessao.selecionar_por_id(primeiro);
    let _=app.update(Mensagem::Faixa(FaixaMensagem::FormatoForma));
    let _=app.update(Mensagem::Faixa(FaixaMensagem::SelecionarItem(segundo,false)));
    assert_eq!(app.formato_forma.as_ref().unwrap().cor,"#ff8800");
    assert!(app.formato_forma.as_ref().unwrap().cor_aberta.is_none());
}

#[cfg(test)]
#[test]
fn forma_lateral_atualiza_ao_vivo_e_cancelar_restaura(){
    let mut app=FeatherSlideApp::default();app.sessao.elemento_inserir(Tipo::Retangulo,None,None).unwrap();let antes=app.sessao.apresentacao().clone();
    let _=app.update(Mensagem::Faixa(FaixaMensagem::TrocarAba(AbaDaFaixa::Imagem)));assert!(app.formato_forma.is_some());
    let _=app.update(Mensagem::FormatoForma(crate::formato_forma::Msg::Tipo("Linear".into())));assert!(app.sessao.slide().elementos.last().unwrap().gradiente.is_some());
    let _=app.update(Mensagem::Palco(PalcoMensagem::GradientePonto{indice:0,posicao:25.0}));assert_eq!(app.sessao.slide().elementos.last().unwrap().gradiente.as_ref().unwrap().pontos[0].posicao,0.25);
    let _=app.update(Mensagem::Palco(PalcoMensagem::GradienteAngulo(60.0)));assert_eq!(app.sessao.slide().elementos.last().unwrap().gradiente.as_ref().unwrap().angulo,60.0);
    let _=app.update(Mensagem::FormatoForma(crate::formato_forma::Msg::Fechar));assert_eq!(*app.sessao.apresentacao(),antes);
}
