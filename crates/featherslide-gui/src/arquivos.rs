//! Gestão de apresentações abertas, rascunhos, histórico e locais do usuário.
use crate::app::{FeatherSlideApp, Mensagem};
use crate::icones::*;
use crate::tema::*;
use featherslide_core::sessao::Sessao;
use featherslide_io::documentos::{historico, ler, listar, pesquisar, salvar};
use featherslide_io::fsdz;
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length, Task};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub struct Documento {
    pub sessao: Sessao,
    pub caminho: Option<PathBuf>,
    pub nome: String,
    pub rascunho: PathBuf,
    pub leitura: bool,
    pub auto_versao: Option<u64>,
    pub auto_inicio: Instant,
}

#[derive(Clone, Debug)]
pub enum Acao {
    Painel,
    Voltar,
    EmBranco,
    Abrir(bool),
    Escolhido(Option<PathBuf>, bool),
    Trocar(usize),
    Fechar,
    FecharTodas,
    Reverter,
    Duplicar,
    Renomear,
    RenomearEscolhido(Option<PathBuf>),
    Excluir,
    Confirmar,
    Cancelar,
    Sair,
    Busca(String),
    Pasta,
    PastaEscolhida(Option<PathBuf>),
    Favorito,
    Local(PathBuf),
    Recuperar(PathBuf),
    Versao(PathBuf),
    Importar,
    ImportarEscolhido(Option<PathBuf>),
}

pub struct Estado {
    pub abertos: Vec<Documento>,
    pub ativo: bool,
    pub leitura: bool,
    pub painel: bool,
    pub busca: String,
    pub pasta: PathBuf,
    pub lista: Vec<PathBuf>,
    pub recuperacoes: Vec<PathBuf>,
    nomes_recuperacoes: std::collections::HashMap<PathBuf, String>,
    pub versoes: Vec<PathBuf>,
    pub pendente: Option<Acao>,
    pub checkpoint: Instant,
}
impl Default for Estado {
    fn default() -> Self {
        Self {
            abertos: Vec::new(),
            ativo: true,
            leitura: false,
            painel: false,
            busca: String::new(),
            pasta: crate::configuracao::pasta_das_apresentacoes()
                .unwrap_or_else(|| PathBuf::from(".")),
            lista: Vec::new(),
            recuperacoes: Vec::new(),
            nomes_recuperacoes: Default::default(),
            versoes: Vec::new(),
            pendente: None,
            checkpoint: Instant::now(),
        }
    }
}

fn id() -> String {
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{:020}-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        std::process::id(),
        N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}
/// Rascunhos nunca mudam o arquivo original nem o estado de edição.
pub fn checkpoint(sessao: &Sessao, path: &Path) -> Result<(), String> {
    if !sessao.estado().sujo {
        return Ok(());
    }
    std::fs::create_dir_all(path.parent().ok_or("Pasta do rascunho inválida")?)
        .map_err(|e| e.to_string())?;
    fsdz::salvar(sessao.apresentacao(), path).map(|_| ())
}

impl FeatherSlideApp {
    /// Abre todos os arquivos recebidos pelo executável, preservando as sessões.
    pub(crate) fn abrir_argumentos(&mut self, args: impl IntoIterator<Item = std::ffi::OsString>) {
        for argumento in args {
            if let Err(e) = self.abrir_documento(PathBuf::from(argumento), false) {
                self.status = e;
            }
        }
    }

    pub(crate) fn guardar_documento(&mut self) {
        if !self.arquivos.ativo {
            return;
        }
        self.sessao.confirmar_previa();
        self.arquivos.abertos.push(Documento {
            sessao: std::mem::take(&mut self.sessao),
            caminho: self.caminho_arquivo.take(),
            nome: std::mem::take(&mut self.nome_arquivo),
            rascunho: self.rascunho.clone(),
            leitura: self.arquivos.leitura,
            auto_versao: self.auto_versao,
            auto_inicio: self.auto_inicio,
        });
        self.arquivos.ativo = false;
    }
    fn ativar_documento(&mut self, doc: Documento) {
        self.sessao = doc.sessao;
        self.caminho_arquivo = doc.caminho;
        self.nome_arquivo = doc.nome;
        self.rascunho = doc.rascunho;
        self.arquivos.leitura = doc.leitura;
        self.auto_versao = doc.auto_versao;
        self.auto_inicio = doc.auto_inicio;
        self.arquivos.ativo = true;
        self.em_tela_inicio = false;
        self.arquivos.painel = false;
        self.limpar_interface_documento();
    }
    pub(crate) fn limpar_interface_documento(&mut self) {
        self.editor_diagrama = None;
        self.ferramentas_texto = None;
        self.pincel = None;
        self.configuracao_slide = None;
        self.cache_quadros.borrow_mut().clear();
        self.editando_id = None;
        self.editando_celula = None;
        self.dados_id = None;
        self.slides_sel.clear();
        self.secoes_recolhidas.clear();
        self.pan = (0.0, 0.0);
        self.previa = None;
        self.menu = None;
        self.revisao = None;
        self.propriedades = None;
        self.reutilizacao = None;
        self.painel_imagem = false;
        self.imagem_alt_rascunho.clear();
        self.ir_numero = None;
        self.foco_miniaturas = false;
        self.ortografia = None;
        self.seletor_cor = None;
        self.galeria_layouts = None;
        self.impressao = None;
        self.texto_edicao.clear();
        self.ancora_edicao = None;
        self.painel_selecao = false;
    }
    pub(crate) fn abrir_documento(&mut self, path: PathBuf, leitura: bool) -> Result<(), String> {
        if self
            .caminho_arquivo
            .as_ref()
            .is_some_and(|p| mesmo_arquivo(p, &path))
            && self.arquivos.ativo
        {
            self.arquivos.leitura |= leitura;
            self.arquivos.painel = false;
            self.em_tela_inicio = false;
            return Ok(());
        }
        if let Some(i) = self
            .arquivos
            .abertos
            .iter()
            .position(|d| d.caminho.as_ref().is_some_and(|p| mesmo_arquivo(p, &path)))
        {
            self.trocar_documento(i);
            self.arquivos.leitura |= leitura;
            return Ok(());
        }
        let ap = ler(&path)?;
        self.trocar_sessao(Sessao::nova(ap));
        self.caminho_arquivo = Some(path.clone());
        self.nome_arquivo = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into();
        self.arquivos.leitura =
            leitura || std::fs::metadata(&path).is_ok_and(|m| m.permissions().readonly());
        self.arquivos.painel = false;
        self.em_tela_inicio = false;
        self.pode_continuar = false;
        self.aplicar_dicas();
        self.config.registrar_recente(&path);
        self.status = if self.arquivos.leitura {
            "Apresentação aberta somente para leitura"
        } else {
            "Apresentação aberta"
        }
        .into();
        Ok(())
    }
    pub(crate) fn trocar_documento(&mut self, i: usize) {
        if i >= self.arquivos.abertos.len() {
            return;
        }
        let doc = self.arquivos.abertos.remove(i);
        self.guardar_documento();
        self.ativar_documento(doc);
    }
    pub(crate) fn salvar_documentos_em_segundo_plano(&mut self, agora: Instant) {
        if !self.config.auto_salvar {
            return;
        }
        for doc in &mut self.arquivos.abertos {
            let estado = doc.sessao.estado();
            if doc.leitura
                || !estado.sujo
                || doc.auto_versao == Some(estado.versao)
                || agora.saturating_duration_since(doc.auto_inicio).as_secs()
                    < self.config.intervalo.max(1)
            {
                continue;
            }
            let resultado = if let Some(origem) = &doc.caminho {
                let mut destino = origem.clone();
                if destino.extension().is_some_and(|e| {
                    e.eq_ignore_ascii_case("pptx") || e.eq_ignore_ascii_case("odp")
                }) {
                    destino.set_extension("fsdz");
                }
                salvar(doc.sessao.apresentacao(), &destino).map(|_| {
                    doc.caminho = Some(destino.clone());
                    doc.nome = destino
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into();
                    doc.sessao.marcar_gravado();
                    let _ = std::fs::remove_file(&doc.rascunho);
                })
            } else {
                checkpoint(&doc.sessao, &doc.rascunho)
            };
            doc.auto_inicio = agora;
            match resultado {
                Ok(()) => doc.auto_versao = Some(doc.sessao.estado().versao),
                Err(e) => self.status = format!("Não consegui salvar {}: {e}", doc.nome),
            }
        }
    }

    pub(crate) fn checkpoints(&mut self) -> Result<(), String> {
        if self.arquivos.ativo && !self.arquivos.leitura {
            checkpoint(&self.sessao, &self.rascunho)?;
        }
        for doc in &self.arquivos.abertos {
            if !doc.leitura {
                checkpoint(&doc.sessao, &doc.rascunho)?;
            }
        }
        self.arquivos.checkpoint = Instant::now();
        Ok(())
    }
    pub(crate) fn atualizar_arquivos(&mut self) {
        self.arquivos.lista = listar(&self.arquivos.pasta);
        let pasta = crate::configuracao::novo_rascunho()
            .parent()
            .unwrap()
            .to_path_buf();
        let ativos: Vec<_> = self
            .arquivos
            .abertos
            .iter()
            .map(|d| &d.rascunho)
            .chain(self.arquivos.ativo.then_some(&self.rascunho))
            .collect();
        self.arquivos.recuperacoes = listar(&pasta)
            .into_iter()
            .filter(|p| !ativos.contains(&p))
            .collect();
        self.arquivos.nomes_recuperacoes = self
            .arquivos
            .recuperacoes
            .iter()
            .map(|path| {
                let nome = ler(path)
                    .map(|ap| {
                        if ap.titulo.trim().is_empty() {
                            format!("Apresentação · {} slides", ap.slides.len())
                        } else {
                            ap.titulo
                        }
                    })
                    .unwrap_or_else(|_| "Rascunho recuperável".to_string());
                (path.clone(), nome)
            })
            .collect();
        self.arquivos.versoes = self
            .caminho_arquivo
            .as_deref()
            .map(historico)
            .unwrap_or_default();
        if let Some(p) = &self.caminho_arquivo {
            if p.exists() {
                self.arquivos.versoes.insert(0, p.clone());
            }
        }
    }
    fn fechar_documento(&mut self) -> Result<(), String> {
        if self.arquivos.ativo && !self.arquivos.leitura {
            checkpoint(&self.sessao, &self.rascunho)?;
        }
        self.arquivos.ativo = false;
        self.arquivos.painel = false;
        if let Some(doc) = self.arquivos.abertos.pop() {
            self.ativar_documento(doc);
        } else {
            self.sessao = Sessao::default();
            self.caminho_arquivo = None;
            self.nome_arquivo = "Apresentação sem título".into();
            self.em_tela_inicio = true;
            self.pode_continuar = false;
            self.limpar_interface_documento();
        }
        Ok(())
    }
    pub(crate) fn tratar_arquivos(&mut self, acao: Acao) -> Task<Mensagem> {
        use Acao::*;
        if self.arquivos.pendente.is_some() && !matches!(acao, Confirmar | Cancelar) {
            return Task::none();
        }
        let precisa_confirmar = match &acao {
            Fechar | Reverter => self.arquivos.ativo && self.sessao.estado().sujo,
            FecharTodas | Sair => {
                (self.arquivos.ativo && self.sessao.estado().sujo)
                    || self.arquivos.abertos.iter().any(|d| d.sessao.estado().sujo)
            }
            Excluir => self.caminho_arquivo.is_some(),
            _ => false,
        };
        if precisa_confirmar {
            self.arquivos.pendente = Some(acao);
            self.arquivos.painel = true;
            return Task::none();
        }
        self.executar_arquivo(acao)
    }
    fn executar_arquivo(&mut self, acao: Acao) -> Task<Mensagem> {
        use Acao::*;
        let resultado: Result<(), String> = match acao {
            Painel => {
                self.atualizar_arquivos();
                self.arquivos.painel = true;
                Ok(())
            }
            Voltar => {
                self.arquivos.painel = false;
                Ok(())
            }
            EmBranco => {
                let mut nova = Sessao::default();
                let _ = nova.slide_layout("em_branco");
                self.trocar_sessao(nova);
                self.caminho_arquivo = None;
                self.nome_arquivo = "Apresentação em branco".into();
                self.arquivos.painel = false;
                self.em_tela_inicio = false;
                Ok(())
            }
            Cancelar => {
                self.arquivos.pendente = None;
                Ok(())
            }
            Confirmar => {
                if let Some(a) = self.arquivos.pendente.take() {
                    return self.executar_arquivo(a);
                }
                Ok(())
            }
            Abrir(leitura) => {
                return Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .add_filter("Apresentações", &["fsdz", "pptx", "odp"])
                            .pick_file()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    move |p| Mensagem::Arquivos(Escolhido(p, leitura)),
                )
            }
            Escolhido(Some(p), leitura) => self.abrir_documento(p, leitura),
            Escolhido(None, _)
            | PastaEscolhida(None)
            | RenomearEscolhido(None)
            | ImportarEscolhido(None) => Ok(()),
            Trocar(i) => {
                self.trocar_documento(i);
                Ok(())
            }
            Fechar => self.fechar_documento(),
            FecharTodas | Sair => {
                if let Err(e) = self.checkpoints() {
                    self.status = e;
                    return Task::none();
                }
                if matches!(acao, Sair) {
                    return iced::exit();
                }
                self.arquivos.abertos.clear();
                self.fechar_documento()
            }
            Reverter => {
                match self
                    .caminho_arquivo
                    .clone()
                    .ok_or("Salve a apresentação antes de reverter".to_string())
                    .and_then(|p| ler(&p))
                {
                    Ok(ap) => {
                        if let Err(e) = checkpoint(&self.sessao, &self.rascunho) {
                            self.status = e;
                            return Task::none();
                        }
                        self.sessao = Sessao::nova(ap);
                        self.aplicar_dicas();
                        self.limpar_interface_documento();
                        self.auto_versao = None;
                        self.status = "Versão salva restaurada; alterações anteriores preservadas no rascunho".into();
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            Duplicar => {
                let mut ap = self.sessao.apresentacao().clone();
                // Uma apresentação duplicada é outro documento: novo ID, criação e revisão.
                ap.metadados.identificador = featherslide_core::metadados::novo_identificador();
                ap.metadados.criado_em_utc = featherslide_core::metadados::agora_utc();
                ap.metadados.modificado_em_utc = ap.metadados.criado_em_utc;
                ap.metadados.revisoes = 0;
                ap.metadados.tempo_edicao_segundos = 0;
                ap.metadados.ultimo_autor.clone_from(&ap.autor);
                let nome = format!("{} - cópia", self.nome_arquivo);
                self.trocar_sessao(Sessao::nova(ap));
                self.nome_arquivo = nome;
                self.caminho_arquivo = None;
                self.sessao.marcar_alterado();
                self.arquivos.painel = false;
                self.em_tela_inicio = false;
                Ok(())
            }
            Renomear => {
                if self.arquivos.leitura {
                    Err("Arquivo aberto somente para leitura".into())
                } else {
                    return Task::perform(
                        async {
                            rfd::AsyncFileDialog::new()
                                .set_title("Renomear apresentação")
                                .add_filter("FeatherSlide", &["fsdz"])
                                .save_file()
                                .await
                                .map(|f| f.path().to_path_buf())
                        },
                        |p| Mensagem::Arquivos(RenomearEscolhido(p)),
                    );
                }
            }
            RenomearEscolhido(Some(p)) => self.renomear_documento(p),
            Excluir => self.excluir_documento(),
            Busca(q) => {
                self.arquivos.busca = q;
                Ok(())
            }
            Pasta => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .pick_folder()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    |p| Mensagem::Arquivos(PastaEscolhida(p)),
                )
            }
            PastaEscolhida(Some(p)) | Local(p) => {
                self.arquivos.pasta = p;
                self.atualizar_arquivos();
                Ok(())
            }
            Favorito => {
                let p = self.arquivos.pasta.to_string_lossy().to_string();
                if self.config.favoritos.contains(&p) {
                    self.config.favoritos.retain(|s| s != &p);
                } else {
                    self.config.favoritos.push(p);
                }
                self.config.gravar();
                Ok(())
            }
            Recuperar(p) | Versao(p) => match ler(&p) {
                Ok(ap) => {
                    self.trocar_sessao(Sessao::nova(ap));
                    self.caminho_arquivo = None;
                    self.nome_arquivo = "Apresentação recuperada".into();
                    self.sessao.marcar_alterado();
                    self.em_tela_inicio = false;
                    self.arquivos.painel = false;
                    self.status = "Recuperada como nova apresentação; use Salvar como".into();
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Importar => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Importar slides")
                            .add_filter("Apresentações", &["fsdz", "pptx", "odp"])
                            .pick_file()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    |p| Mensagem::Arquivos(ImportarEscolhido(p)),
                )
            }
            ImportarEscolhido(Some(p)) => {
                ler(&p).and_then(|ap| self.sessao.apresentacao_importar(&ap).map(|_| ()))
            }
        };
        if let Err(e) = resultado {
            self.status = e;
        }
        Task::none()
    }
    fn renomear_documento(&mut self, mut novo: PathBuf) -> Result<(), String> {
        if self.arquivos.leitura {
            return Err("Arquivo aberto somente para leitura".into());
        }
        let origem = self
            .caminho_arquivo
            .clone()
            .ok_or("Salve a apresentação antes de renomear")?;
        novo.set_extension(origem.extension().unwrap_or_default());
        if novo.exists() {
            return Err("O destino já existe; escolha outro nome".into());
        }
        // Renomeação no mesmo diretório mantém arquivo e histórico juntos, sem cópia parcial.
        if origem.parent().map(|p| p.canonicalize().ok())
            != novo.parent().map(|p| p.canonicalize().ok())
        {
            return Err("Escolha um nome na mesma pasta; use Salvar cópia para outra pasta".into());
        }
        std::fs::rename(&origem, &novo).map_err(|e| e.to_string())?;
        if featherslide_io::documentos::pasta_historico(&origem).exists() {
            if let Err(e) = std::fs::rename(
                featherslide_io::documentos::pasta_historico(&origem),
                featherslide_io::documentos::pasta_historico(&novo),
            ) {
                let _ = std::fs::rename(&novo, &origem);
                return Err(e.to_string());
            }
        }
        self.config.recentes.retain(|p| Path::new(p) != origem);
        self.config.registrar_recente(&novo);
        self.caminho_arquivo = Some(novo.clone());
        self.nome_arquivo = novo
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into();
        self.status = "Apresentação renomeada".into();
        self.atualizar_arquivos();
        Ok(())
    }
    fn excluir_documento(&mut self) -> Result<(), String> {
        if self.arquivos.leitura {
            return Err("Arquivo aberto somente para leitura".into());
        }
        let origem = self
            .caminho_arquivo
            .clone()
            .ok_or("A apresentação ainda não tem arquivo")?;
        // Preserva o arquivo em lixeira própria e oferece recuperação pelo painel.
        let pasta = self
            .rascunho
            .parent()
            .ok_or("Pasta de recuperação inválida")?;
        std::fs::create_dir_all(pasta).map_err(|e| e.to_string())?;
        let recuperacao = pasta.join(format!("excluida-{}.fsdz", id()));
        fsdz::salvar(&ler(&origem)?, &recuperacao)?;
        checkpoint(&self.sessao, &self.rascunho)?;
        std::fs::remove_file(&origem).map_err(|e| e.to_string())?;
        self.config.recentes.retain(|p| Path::new(p) != origem);
        self.config.gravar();
        self.caminho_arquivo = None;
        self.nome_arquivo = "Apresentação excluída — cópia aberta".into();
        self.status = "Arquivo excluído; cópia disponível em Recuperação".into();
        self.atualizar_arquivos();
        Ok(())
    }
    pub(crate) fn barra_documentos(&self) -> Element<'_, Mensagem> {
        if self.arquivos.abertos.is_empty() {
            return iced::widget::Space::with_height(0).into();
        }
        let rotulo = |nome: &str, leitura: bool, sujo: bool| {
            row![
                icone_apresentar(15.0, "#5f6368"),
                text(format!(
                    "{nome}{}{}",
                    if leitura { " · somente leitura" } else { "" },
                    if sujo { " •" } else { "" }
                ))
                .size(12)
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
        };
        let mut linha = row![].spacing(4).align_y(iced::Alignment::Center);
        if self.arquivos.ativo {
            linha = linha.push(
                container(rotulo(
                    &self.nome_arquivo,
                    self.arquivos.leitura,
                    self.sessao.estado().sujo,
                ))
                .padding([7, 12])
                .style(|_| container::Style {
                    background: Some(COR_AZUL_FUNDO().into()),
                    text_color: Some(COR_AZUL_ACENTO()),
                    border: iced::Border {
                        color: COR_AZUL_ACENTO(),
                        width: 1.0,
                        radius: 5.0.into(),
                    },
                    ..Default::default()
                }),
            );
        }
        for (i, doc) in self.arquivos.abertos.iter().enumerate() {
            linha = linha.push(
                button(rotulo(&doc.nome, doc.leitura, doc.sessao.estado().sujo))
                    .padding([7, 12])
                    .on_press(Mensagem::Arquivos(Acao::Trocar(i)))
                    .style(estilo_botao_pequeno(false)),
            );
        }
        container(
            iced::widget::scrollable(linha).direction(
                iced::widget::scrollable::Direction::Horizontal(
                    iced::widget::scrollable::Scrollbar::default()
                        .width(3)
                        .scroller_width(3),
                ),
            ),
        )
        .height(Length::Fixed(44.0))
        .padding([4, 10])
        .style(estilo_container_topo)
        .into()
    }
    pub(crate) fn view_arquivos(&self) -> Element<'_, Mensagem> {
        use iced::widget::{horizontal_rule, Space};
        use iced::Alignment;
        let barra = self
            .barra_da_janela_simples()
            .unwrap_or_else(|| Space::with_height(0).into());
        let acao = |rotulo: &str, acao: Acao| {
            button(text(rotulo.to_owned()).size(13))
                .padding([9, 14])
                .on_press(Mensagem::Arquivos(acao))
                .style(estilo_botao_secundario)
        };
        if let Some(pendente) = &self.arquivos.pendente {
            let aviso = if matches!(pendente, Acao::Excluir) {
                "Excluir o arquivo? Uma cópia será mantida em Recuperação."
            } else {
                "Há alterações não salvas. Ao continuar, elas serão preservadas em Recuperação."
            };
            let cartao = container(
                column![
                    text("Confirmar ação").size(24),
                    text(aviso).size(15),
                    row![
                        Space::with_width(Length::Fill),
                        acao("Cancelar", Acao::Cancelar),
                        acao("Continuar", Acao::Confirmar).style(estilo_botao_primario)
                    ]
                    .spacing(10),
                    text(&self.status).size(12).color(COR_TEXTO_MUTED()),
                ]
                .spacing(20),
            )
            .padding(28)
            .max_width(540)
            .style(estilo_cartao_arquivos);
            return column![
                barra,
                container(cartao)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .style(estilo_fundo_arquivos)
            ]
            .into();
        }
        let menu = |nome: &str, icone, mensagem| {
            button(
                row![icone, text(nome.to_owned()).size(13)]
                    .spacing(12)
                    .align_y(Alignment::Center),
            )
            .padding([9, 12])
            .width(Length::Fill)
            .on_press(mensagem)
            .style(estilo_botao_pequeno(false))
        };
        let cinza = "#5f6368";
        let mut lateral = column![
            menu(
                "Voltar",
                icone_voltar(18.0, cinza),
                Mensagem::Arquivos(Acao::Voltar)
            ),
            horizontal_rule(1),
            container(text("ARQUIVOS").size(11).color(COR_TEXTO_MUTED())).padding([8, 12]),
            menu(
                "Nova apresentação",
                icone_mais(18.0, cinza),
                Mensagem::NovaApresentacao
            ),
            menu(
                "Em branco",
                icone_novo_slide(18.0, cinza),
                Mensagem::Arquivos(Acao::EmBranco)
            ),
            menu(
                "Abrir…",
                icone_pasta(18.0, cinza),
                Mensagem::Arquivos(Acao::Abrir(false))
            ),
            menu(
                "Abrir somente leitura…",
                icone_apresentar(18.0, cinza),
                Mensagem::Arquivos(Acao::Abrir(true))
            ),
        ]
        .spacing(3);
        if self.arquivos.ativo {
            lateral = lateral.push(horizontal_rule(1)).push(
                container(text("APRESENTAÇÃO ATUAL").size(11).color(COR_TEXTO_MUTED()))
                    .padding([8, 12]),
            );
            for (nome, icone, mensagem) in [
                (
                    "Salvar",
                    icone_salvar(18.0, cinza),
                    Mensagem::Faixa(crate::faixa::FaixaMensagem::Salvar),
                ),
                (
                    "Salvar como…",
                    icone_lapis(18.0, cinza),
                    Mensagem::Faixa(crate::faixa::FaixaMensagem::SalvarComo),
                ),
                (
                    "Salvar cópia…",
                    icone_copiar(18.0, cinza),
                    Mensagem::Faixa(crate::faixa::FaixaMensagem::SalvarCopia),
                ),
                (
                    "Duplicar",
                    icone_duplicar(18.0, cinza),
                    Mensagem::Arquivos(Acao::Duplicar),
                ),
                (
                    "Importar slides…",
                    icone_novo_slide(18.0, cinza),
                    Mensagem::Arquivos(Acao::Importar),
                ),
                (
                    "Renomear…",
                    icone_lapis(18.0, cinza),
                    Mensagem::Arquivos(Acao::Renomear),
                ),
                (
                    "Reverter para salva",
                    icone_desfazer(18.0, cinza),
                    Mensagem::Arquivos(Acao::Reverter),
                ),
                (
                    "Excluir…",
                    icone_excluir(18.0, cinza),
                    Mensagem::Arquivos(Acao::Excluir),
                ),
                (
                    "Fechar",
                    icone_fechar_janela(18.0, cinza),
                    Mensagem::Arquivos(Acao::Fechar),
                ),
                (
                    "Fechar todas",
                    icone_fechar_janela(18.0, cinza),
                    Mensagem::Arquivos(Acao::FecharTodas),
                ),
            ] {
                lateral = lateral.push(menu(nome, icone, mensagem));
            }
        }
        let mut conteudo = column![row![
            column![
                text("Arquivos").size(30),
                text("Suas apresentações, em um só lugar.")
                    .size(14)
                    .color(COR_TEXTO_MUTED())
            ]
            .spacing(6),
            Space::with_width(Length::Fill),
            acao("Abrir arquivo…", Acao::Abrir(false)).style(estilo_botao_primario)
        ]
        .align_y(Alignment::Center)
        .spacing(16),]
        .spacing(24);
        if self.arquivos.ativo {
            let situacao = if self.arquivos.leitura {
                "Somente leitura"
            } else if self.sessao.estado().sujo {
                "Alterações não salvas"
            } else {
                "Salvo"
            };
            conteudo = conteudo.push(
                container(
                    row![
                        icone_apresentar(24.0, cinza),
                        column![
                            text("EM EDIÇÃO").size(10).color(COR_TEXTO_MUTED()),
                            text(&self.nome_arquivo).size(15)
                        ]
                        .spacing(4),
                        Space::with_width(Length::Fill),
                        text(situacao).size(12).color(COR_TEXTO_MUTED()),
                    ]
                    .spacing(16)
                    .align_y(Alignment::Center),
                )
                .padding(18)
                .width(Length::Fill)
                .style(estilo_cartao_arquivos),
            );
        }
        let favorito = self
            .config
            .favoritos
            .contains(&self.arquivos.pasta.to_string_lossy().to_string());
        let mut lista = column![
            row![
                icone_pasta(22.0, cinza),
                column![
                    text("Neste computador").size(18),
                    text(self.arquivos.pasta.to_string_lossy().to_string())
                        .size(12)
                        .color(COR_TEXTO_MUTED())
                ]
                .spacing(4),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                acao("Escolher pasta…", Acao::Pasta),
                acao(
                    if favorito {
                        "Remover favorito"
                    } else {
                        "Adicionar favorito"
                    },
                    Acao::Favorito
                ),
            ]
            .spacing(8),
            text_input("Pesquisar nesta pasta", &self.arquivos.busca)
                .on_input(|q| Mensagem::Arquivos(Acao::Busca(q)))
                .padding([11, 14])
                .size(14),
        ]
        .spacing(14);
        if !self.config.favoritos.is_empty() {
            let mut locais =
                column![text("Favoritos").size(12).color(COR_TEXTO_MUTED())].spacing(6);
            for local in &self.config.favoritos {
                locais = locais.push(
                    acao(local, Acao::Local(PathBuf::from(local)))
                        .style(estilo_botao_pequeno(false)),
                );
            }
            lista = lista.push(locais);
        }
        let arquivos = pesquisar(&self.arquivos.lista, &self.arquivos.busca);
        lista = lista.push(row![
            text(format!("{} apresentações", arquivos.len()))
                .size(12)
                .color(COR_TEXTO_MUTED()),
            Space::with_width(Length::Fill),
            text("FORMATO").size(10).color(COR_TEXTO_MUTED())
        ]);
        if arquivos.is_empty() {
            lista = lista.push(
                container(
                    column![
                        text(if self.arquivos.busca.is_empty() {
                            "Nenhuma apresentação nesta pasta"
                        } else {
                            "Nenhum arquivo encontrado"
                        })
                        .size(16),
                        text("Escolha outra pasta ou ajuste a pesquisa.")
                            .size(13)
                            .color(COR_TEXTO_MUTED()),
                    ]
                    .spacing(8),
                )
                .padding([24, 12]),
            );
        }
        for path in arquivos {
            let nome = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let formato = path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_uppercase();
            lista = lista.push(horizontal_rule(1)).push(
                button(
                    row![
                        container(icone_apresentar(22.0, cinza))
                            .padding(10)
                            .style(estilo_selo_arquivos),
                        text(nome).size(14).width(Length::Fill),
                        text(formato).size(11).color(COR_TEXTO_MUTED()),
                    ]
                    .spacing(14)
                    .align_y(Alignment::Center),
                )
                .on_press(Mensagem::Arquivos(Acao::Escolhido(
                    Some(path.clone()),
                    false,
                )))
                .padding([8, 4])
                .width(Length::Fill)
                .style(estilo_botao_pequeno(false)),
            );
        }
        conteudo = conteudo.push(
            container(lista)
                .padding(22)
                .width(Length::Fill)
                .style(estilo_cartao_arquivos),
        );
        if !self.arquivos.recuperacoes.is_empty() {
            let mut recuperacao = column![
                row![icone_relogio(22.0, cinza), text("Recuperação").size(18)]
                    .spacing(10)
                    .align_y(Alignment::Center),
                text("Retome um rascunho ou abra uma cópia de um arquivo excluído.")
                    .size(13)
                    .color(COR_TEXTO_MUTED()),
            ]
            .spacing(12);
            for path in &self.arquivos.recuperacoes {
                let nome = self
                    .arquivos
                    .nomes_recuperacoes
                    .get(path)
                    .map(String::as_str)
                    .unwrap_or("Rascunho recuperável");
                recuperacao = recuperacao.push(
                    acao(&rotulo_versao(path, nome), Acao::Recuperar(path.clone()))
                        .width(Length::Fill)
                        .style(estilo_botao_pequeno(false)),
                );
            }
            conteudo = conteudo.push(
                container(recuperacao)
                    .padding(22)
                    .width(Length::Fill)
                    .style(estilo_cartao_arquivos),
            );
        }
        if !self.arquivos.versoes.is_empty() {
            let mut versoes = column![
                text("Histórico de versões").size(18),
                text("Abra uma versão salva como cópia.")
                    .size(13)
                    .color(COR_TEXTO_MUTED())
            ]
            .spacing(12);
            for (i, path) in self.arquivos.versoes.iter().enumerate() {
                versoes = versoes.push(
                    acao(
                        &rotulo_versao(
                            path,
                            if i == 0 {
                                "Versão salva atual"
                            } else {
                                "Versão anterior"
                            },
                        ),
                        Acao::Versao(path.clone()),
                    )
                    .width(Length::Fill)
                    .style(estilo_botao_pequeno(false)),
                );
            }
            conteudo = conteudo.push(
                container(versoes)
                    .padding(22)
                    .width(Length::Fill)
                    .style(estilo_cartao_arquivos),
            );
        }
        let lateral = container(crate::tema::rolagem(lateral))
            .padding([18, 12])
            .width(230)
            .height(Length::Fill)
            .style(estilo_container_topo);
        let principal = container(crate::tema::rolagem(
            container(conteudo).padding(30).max_width(1100),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(estilo_fundo_arquivos);
        let rodape = container(text(&self.status).size(12).color(COR_TEXTO_MUTED()))
            .padding([8, 18])
            .width(Length::Fill)
            .style(estilo_container_topo);
        column![barra, row![lateral, principal].height(Length::Fill), rodape].into()
    }
}

fn estilo_fundo_arquivos(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(COR_FUNDO_JANELA().into()),
        ..Default::default()
    }
}

fn estilo_cartao_arquivos(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(COR_PAINEL().into()),
        border: iced::Border {
            color: COR_BORDA_SUAVE(),
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

fn estilo_selo_arquivos(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(COR_AZUL_FUNDO().into()),
        border: iced::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn rotulo_versao(path: &Path, prefixo: &str) -> String {
    let tempo = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map_or(0, |d| d.as_secs());
    let data = if tempo < 60 {
        "agora".to_string()
    } else if tempo < 3600 {
        format!("há {} min", tempo / 60)
    } else if tempo < 86400 {
        format!("há {} h", tempo / 3600)
    } else {
        format!("há {} dias", tempo / 86400)
    };
    format!("{prefixo} · {data}")
}

fn mesmo_arquivo(a: &Path, b: &Path) -> bool {
    a == b
        || a.canonicalize()
            .ok()
            .zip(b.canonicalize().ok())
            .is_some_and(|(a, b)| a == b)
}

#[cfg(test)]
mod testes {
    use super::*;
    struct Pasta(PathBuf);
    impl Pasta {
        fn nova() -> Self {
            let p = std::env::temp_dir().join(format!("fs-arquivos-{}", id()));
            std::fs::create_dir_all(&p).unwrap();
            Self(p)
        }
        fn arquivo(&self, nome: &str) -> PathBuf {
            self.0.join(nome)
        }
    }
    impl Drop for Pasta {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn app() -> FeatherSlideApp {
        let mut app = FeatherSlideApp::default();
        app.config = crate::configuracao::Configuracao::default();
        app.config.ortografia_automatica = false;
        app
    }
    #[test]
    fn duas_apresentacoes_preservam_edicoes_desfazer_e_caminho_ao_alternar() {
        let pasta = Pasta::nova();
        let mut app = app();
        app.rascunho = pasta.arquivo("a-rascunho.fsdz");
        app.nome_arquivo = "A".into();
        app.sessao.wordart_inserir().unwrap();
        let elementos = app.sessao.slide().elementos.len();
        let _ = app.update(Mensagem::NovaApresentacao);
        app.rascunho = pasta.arquivo("b-rascunho.fsdz");
        app.nome_arquivo = "B".into();
        app.sessao.slide_novo("em_branco").unwrap();
        app.trocar_documento(0);
        assert_eq!(app.nome_arquivo, "A");
        assert_eq!(app.sessao.slide().elementos.len(), elementos);
        assert!(app.sessao.desfazer().is_some());
        assert_eq!(app.sessao.slide().elementos.len(), elementos - 1);
        app.trocar_documento(0);
        assert_eq!(app.nome_arquivo, "B");
        assert_eq!(app.sessao.apresentacao().slides.len(), 2);
    }
    #[test]
    fn somente_leitura_impede_edicao_e_sobrescrita_mas_permite_copia() {
        let pasta = Pasta::nova();
        let original = pasta.arquivo("original.fsdz");
        fsdz::salvar(Sessao::default().apresentacao(), &original).unwrap();
        let antes = std::fs::read(&original).unwrap();
        let mut app = app();
        app.arquivos.ativo = false;
        app.abrir_documento(original.clone(), true).unwrap();
        let ap = app.sessao.apresentacao().clone();
        let _ = app.update(Mensagem::Faixa(crate::faixa::FaixaMensagem::InserirWordArt));
        assert_eq!(app.sessao.apresentacao(), &ap);
        assert!(!app.sessao.estado().sujo);
        let _ = app.update(Mensagem::Faixa(crate::faixa::FaixaMensagem::Salvar));
        let _ = app.update(Mensagem::ArquivoEscolhidoParaSalvar(Some(original.clone())));
        assert_eq!(std::fs::read(&original).unwrap(), antes);
        let copia = pasta.arquivo("copia.fsdz");
        let _ = app.update(Mensagem::CopiaEscolhida(Some(copia.clone())));
        assert!(ler(&copia).is_ok());
        assert_eq!(app.caminho_arquivo.as_ref(), Some(&original));
    }
    #[test]
    fn historico_preserva_versoes_e_reverter_nao_perde_edicao_pendente() {
        let pasta = Pasta::nova();
        let destino = pasta.arquivo("ap.fsdz");
        let mut app = app();
        app.caminho_arquivo = Some(destino.clone());
        app.rascunho = pasta.arquivo("rascunho.fsdz");
        assert!(app.gravar_no_caminho());
        let ap = app.sessao.apresentacao().clone();
        app.sessao.wordart_inserir().unwrap();
        assert!(app.gravar_no_caminho());
        let versoes = historico(&destino);
        assert_eq!(versoes.len(), 1);
        assert_eq!(ler(&versoes[0]).unwrap(), ap);
        app.sessao.wordart_inserir().unwrap();
        let pendente = app.sessao.apresentacao().clone();
        let _ = app.tratar_arquivos(Acao::Reverter);
        assert!(app.arquivos.pendente.is_some());
        let _ = app.tratar_arquivos(Acao::Confirmar);
        assert_eq!(ler(&app.rascunho).unwrap(), pendente);
        assert_eq!(app.sessao.apresentacao(), &ler(&destino).unwrap());
        assert!(!app.sessao.estado().sujo);
    }
    #[test]
    fn fechar_todas_preserva_rascunhos_e_recupera_como_copia_editavel() {
        let pasta = Pasta::nova();
        let mut app = app();
        app.rascunho = pasta.arquivo("primeira.fsdz");
        app.sessao.wordart_inserir().unwrap();
        let primeira = app.sessao.apresentacao().clone();
        let _ = app.update(Mensagem::NovaApresentacao);
        app.rascunho = pasta.arquivo("segunda.fsdz");
        app.sessao.slide_novo("em_branco").unwrap();
        let _ = app.tratar_arquivos(Acao::FecharTodas);
        assert!(app.arquivos.pendente.is_some());
        let _ = app.tratar_arquivos(Acao::Cancelar);
        assert!(app.arquivos.ativo);
        let _ = app.tratar_arquivos(Acao::FecharTodas);
        let _ = app.tratar_arquivos(Acao::Confirmar);
        assert!(!app.arquivos.ativo);
        assert!(app.arquivos.abertos.is_empty());
        assert_eq!(ler(&pasta.arquivo("primeira.fsdz")).unwrap(), primeira);
        assert!(ler(&pasta.arquivo("segunda.fsdz")).is_ok());
        let _ = app.tratar_arquivos(Acao::Recuperar(pasta.arquivo("primeira.fsdz")));
        assert_eq!(app.sessao.apresentacao(), &primeira);
        assert!(app.sessao.estado().sujo);
        assert!(app.caminho_arquivo.is_none());
    }
    #[test]
    fn renomear_move_historico_e_excluir_mantem_recuperacao() {
        let pasta = Pasta::nova();
        let mut app = app();
        app.rascunho = pasta.arquivo("recuperacao/rascunho.fsdz");
        app.caminho_arquivo = Some(pasta.arquivo("original.fsdz"));
        assert!(app.gravar_no_caminho());
        app.sessao.wordart_inserir().unwrap();
        assert!(app.gravar_no_caminho());
        let novo = pasta.arquivo("novo.fsdz");
        app.renomear_documento(novo.clone()).unwrap();
        assert!(!pasta.arquivo("original.fsdz").exists());
        assert_eq!(historico(&novo).len(), 1);
        let _ = app.tratar_arquivos(Acao::Excluir);
        assert!(novo.exists());
        let _ = app.tratar_arquivos(Acao::Cancelar);
        assert!(novo.exists());
        let _ = app.tratar_arquivos(Acao::Excluir);
        let _ = app.tratar_arquivos(Acao::Confirmar);
        assert!(!novo.exists());
        assert!(app.caminho_arquivo.is_none());
        assert_eq!(listar(&pasta.arquivo("recuperacao")).len(), 1);
    }
    #[test]
    fn duplicar_separa_documentos_e_auto_salva_sessao_em_segundo_plano() {
        let pasta = Pasta::nova();
        let mut app = app();
        app.caminho_arquivo = Some(pasta.arquivo("original.fsdz"));
        assert!(app.gravar_no_caminho());
        app.sessao.wordart_inserir().unwrap();
        let antes = app.sessao.apresentacao().clone();
        let _ = app.tratar_arquivos(Acao::Duplicar);
        app.rascunho = pasta.arquivo("rascunho-copia.fsdz");
        let mut copiado = app.sessao.apresentacao().clone();
        assert_ne!(copiado.metadados.identificador, antes.metadados.identificador);
        copiado.metadados = antes.metadados.clone();
        assert_eq!(copiado, antes, "a duplicação preserva conteúdo e propriedades editáveis");
        assert!(app.caminho_arquivo.is_none());
        app.sessao.wordart_inserir().unwrap();
        assert_eq!(app.arquivos.abertos[0].sessao.apresentacao(), &antes);
        app.config.auto_salvar = true;
        let agora = app.arquivos.abertos[0].auto_inicio + std::time::Duration::from_secs(60);
        app.salvar_documentos_em_segundo_plano(agora);
        assert_eq!(ler(&pasta.arquivo("original.fsdz")).unwrap(), antes);
        assert!(!app.arquivos.abertos[0].sessao.estado().sujo);
    }
    #[test]
    fn favoritos_sobrevivem_as_preferencias_e_pesquisa_ignora_outros_formatos() {
        let pasta = Pasta::nova();
        let mut app = app();
        fsdz::salvar(app.sessao.apresentacao(), &pasta.arquivo("Reunião.fsdz")).unwrap();
        std::fs::write(pasta.arquivo("notas.txt"), "texto").unwrap();
        let lista = listar(&pasta.0);
        assert_eq!(pesquisar(&lista, "REUNIÃO").len(), 1);
        assert!(pesquisar(&lista, "notas").is_empty());
        let _ = app.tratar_arquivos(Acao::Local(pasta.0.clone()));
        let _ = app.tratar_arquivos(Acao::Favorito);
        let json = serde_json::to_string(&app.config).unwrap();
        let config: crate::configuracao::Configuracao = serde_json::from_str(&json).unwrap();
        assert_eq!(
            config.favoritos,
            vec![pasta.0.to_string_lossy().to_string()]
        );
        let _ = app.tratar_arquivos(Acao::Favorito);
        assert!(app.config.favoritos.is_empty());
        let antiga: crate::configuracao::Configuracao = serde_json::from_str("{}").unwrap();
        assert!(antiga.favoritos.is_empty());
    }

    #[test]
    fn linha_de_comando_abre_varios_arquivos_com_espacos_sem_descartar_sessoes() {
        let pasta = Pasta::nova();
        let mut app = app();
        app.arquivos.ativo = false;
        let a = pasta.arquivo("primeira apresentação.fsdz");
        let b = pasta.arquivo("segunda apresentação.fsdz");
        fsdz::salvar(app.sessao.apresentacao(), &a).unwrap();
        fsdz::salvar(app.sessao.apresentacao(), &b).unwrap();
        app.abrir_argumentos([a.as_os_str().to_owned(), b.as_os_str().to_owned()]);
        assert_eq!(app.caminho_arquivo, Some(b));
        assert_eq!(app.arquivos.abertos.len(), 1);
        assert_eq!(app.arquivos.abertos[0].caminho.as_ref(), Some(&a));
    }
    #[test]
    fn checkpoint_apos_falha_do_processo_reabre_em_nova_sessao_sem_sobrescrever_original() {
        let pasta = Pasta::nova();
        let rascunho = pasta.arquivo("rascunho.fsdz");
        let ap = {
            let mut anterior = app();
            anterior.rascunho = rascunho.clone();
            anterior.sessao.wordart_inserir().unwrap();
            anterior.checkpoints().unwrap();
            anterior.sessao.apresentacao().clone()
        };
        let mut reiniciado = app();
        reiniciado.arquivos.ativo = false;
        let _ = reiniciado.tratar_arquivos(Acao::Recuperar(rascunho));
        assert_eq!(reiniciado.sessao.apresentacao(), &ap);
        assert!(reiniciado.caminho_arquivo.is_none());
        assert!(reiniciado.sessao.estado().sujo);
    }

    #[test]
    fn falha_ao_abrir_preserva_sessao_e_arquivo_ja_aberto_nao_duplica() {
        let pasta = Pasta::nova();
        let mut app = app();
        app.arquivos.ativo = false;
        let arquivo = pasta.arquivo("ap.fsdz");
        fsdz::salvar(app.sessao.apresentacao(), &arquivo).unwrap();
        app.abrir_documento(arquivo.clone(), false).unwrap();
        app.sessao.wordart_inserir().unwrap();
        let antes = app.sessao.apresentacao().clone();
        assert!(app
            .abrir_documento(pasta.arquivo("ausente.fsdz"), false)
            .is_err());
        assert_eq!(app.sessao.apresentacao(), &antes);
        app.abrir_documento(arquivo, false).unwrap();
        assert!(app.arquivos.abertos.is_empty());
    }
}
