//! Preferências e acesso às ferramentas locais de revisão e apresentação.
//! As preferências ficam em `configuracao.rs`; o que é da apresentação (proporção) é do motor.

use crate::localidade::tr;
use featherslide_core::modelo::Proporcao;
use pelicano::*;
use pelicano::column;

use crate::configuracao::{Configuracao, ATALHOS};
use crate::tema::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::FeatherSlideApp;

    #[test]
    fn preferencias_de_revisao_persistem_sem_alterar_o_documento() {
        let mut app = FeatherSlideApp::default();
        let antes = app.sessao.apresentacao().clone();
        let _ = app.tratar_config(Msg::Autocorrecao(true));
        let _ = app.tratar_config(Msg::Capitalizacao(true));
        let json = serde_json::to_string(&app.config).unwrap();
        let config: Configuracao = serde_json::from_str(&json).unwrap();
        assert!(config.autocorrecao && config.capitalizacao);
        assert_eq!(app.sessao.apresentacao(), &antes);
    }

    #[test]
    fn gerenciar_abre_a_ferramenta_e_configurar_slide_abre_rascunho() {
        let mut app = FeatherSlideApp::default();
        let antes = app.sessao.apresentacao().clone();
        let _ = app.tratar_config(Msg::Ferramenta(Modo::Autocorrecao));
        assert_eq!(
            app.ferramentas_texto.as_ref().unwrap().modo,
            Modo::Autocorrecao
        );
        let _ = app.tratar_config(Msg::ConfigurarSlide);
        assert!(app.configuracao_slide.is_some());
        assert_eq!(app.sessao.apresentacao(), &antes);
    }
}
use crate::{ferramentas_texto::Modo, localidade::texto};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Secao {
    Aparencia,
    Salvamento,
    Exibicao,
    Modelos,
    Apresentacao,
    Ortografia,
    Revisao,
    Integracoes,
    Atalhos,
    Sobre,
}

#[derive(Clone, Debug)]
pub enum Msg {
    Secao(Secao),
    Escuro(bool),
    TemaAuto,
    Acento(usize),
    LogoPadrao,
    AutoSalvar(bool),
    Intervalo(u64),
    PainelSlides(bool),
    PainelNotas(bool),
    Grade(bool),
    BarraPropria(bool),
    Idioma(&'static str),
    DoisMonitores(bool),
    Guias(bool),
    Repetir(bool),
    Proporcao(Proporcao),
    SalvarAgora,
    SalvarComo,
    SalvarCopia,
    GravarAtalho(&'static str),
    RestaurarAtalhos,
    NomeModelo(String),
    SalvarModelo,
    ImportarModelo,
    UsarModelo(usize),
    ExportarModelo(usize),
    RemoverModelo(usize),
    OrtAuto(bool),
    Autocorrecao(bool),
    Capitalizacao(bool),
    Ferramenta(Modo),
    ConfigurarSlide,
    GramAuto(bool),
    OrtIdioma(String, bool),
    OrtPrincipal(String),
    OrtAdicionar,
    OrtLimpar,
    OrtLimparSim,
    OrtLimparNao,
}

/// O que a configuração mostra e que mora no estado do aplicativo (não no arquivo de configuração).
#[derive(Clone, Copy, Default)]
pub struct Extras {
    pub grade: bool,
    pub guias: bool,
    pub repetir: bool,
    pub dois_monitores: bool,
    /// Quantos monitores foram achados ao abrir o programa.
    pub monitores: usize,
}

pub struct Estado {
    pub secao: Secao,
    /// O atalho esperando a próxima combinação de teclas.
    pub gravando: Option<&'static str>,
    pub aviso: String,
    /// O nome digitado para o modelo que vai ser criado.
    pub nome_modelo: String,
    /// Os dicionários achados no disco (códigos) e quantas palavras há no pessoal.
    pub idiomas: Vec<String>,
    pub n_pessoais: usize,
    pub confirmar_limpar: bool,
}

impl Default for Estado {
    fn default() -> Self {
        Self {
            secao: Secao::Aparencia,
            gravando: None,
            aviso: String::new(),
            nome_modelo: String::new(),
            idiomas: Vec::new(),
            n_pessoais: 0,
            confirmar_limpar: false,
        }
    }
}

fn linha<'a>(rotulo: &str, dica: &str, controle: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    feather_ui::configuracoes::linha(crate::localidade::tr_se_houver(rotulo), crate::localidade::tr_se_houver(dica), controle)
}

fn opcao<'a>(rotulo: &str, ativa: bool, msg: Msg) -> Element<'a, Msg> {
    button(text(crate::localidade::tr_se_houver(rotulo).to_string()).size(12.5))
        .padding([6, 13])
        .style(estilo_botao_pequeno(ativa))
        .on_press(msg)
        .into()
}

fn amostra(cor: &str, ativa: bool, msg: Msg) -> Element<'static, Msg> {
    let c = {
        let v = u32::from_str_radix(cor.trim_start_matches('#'), 16).unwrap_or(0);
        Color::rgb8((v >> 16) as u8, (v >> 8) as u8, v as u8)
    };
    button(Space::new(Length::Fixed(18.0), Length::Fixed(18.0)))
        .padding(3)
        .style(move |_, _| Style {
            background: Some(Background::Color(c)),
            border: Border {
                color: if ativa {
                    COR_TEXTO()
                } else {
                    Color::TRANSPARENT
                },
                width: 2.0,
                radius: [20.0; 4],
            },
            ..Default::default()
        })
        .on_press(msg)
        .into()
}

/// O nome do idioma pelo código do dicionário (`pt_BR` → "Português (Brasil)").
fn nome_do_idioma(codigo: &str) -> String {
    let nome = match codigo {
        "pt_BR" => tr("Português (Brasil)"),
        "pt_PT" => tr("Português (Portugal)"),
        "en_US" => tr("Inglês (Estados Unidos)"),
        "en_GB" => tr("Inglês (Reino Unido)"),
        "es_ES" => tr("Espanhol (Espanha)"),
        "fr_FR" => tr("Francês"),
        "de_DE" => tr("Alemão"),
        "it_IT" => tr("Italiano"),
        _ => return codigo.to_string(),
    };
    format!("{nome} · {codigo}")
}

impl Estado {
    /// Relê do disco os dicionários que existem e o tamanho do dicionário pessoal.
    pub fn atualizar_ortografia(&mut self) {
        self.idiomas = featherslide_ortografia::idiomas_disponiveis()
            .into_iter()
            .map(|i| i.codigo)
            .collect();
        self.n_pessoais = featherslide_ortografia::palavras_pessoais();
    }

    fn ortografia<'a>(&'a self, c: &Configuracao) -> Element<'a, Msg> {
        let usados: Vec<&str> = c.idiomas_ortografia.iter().map(String::as_str).collect();
        let mut col = column![linha(
            tr("Sublinhar erros ao digitar"),
            tr("Ondulação vermelha sob a palavra errada; o botão direito mostra as correções."),
            toggler("", c.ortografia_automatica).on_toggle(Msg::OrtAuto)
        )]
        .spacing(10);
        col = col.push(linha(tr("Sublinhar gramática (azul)"), tr("Crase, concordância de gênero, vírgula antes de «mas», palavra repetida e outros erros comuns. Não é um corretor completo: só regras de alta confiança."), toggler("", c.gramatica_automatica).on_toggle(Msg::GramAuto)));
        col = col.push(
            text(tr("Idiomas do corretor (o principal dá as sugestões)"))
                .size(12)
                .color(COR_TEXTO_MUTED()),
        );
        if self.idiomas.is_empty() {
            col = col.push(text(tr("Nenhum dicionário achado. Instale o pacote hunspell do seu idioma ou use \"Adicionar idioma\".")).size(12.5));
        }
        for codigo in &self.idiomas {
            let usa = usados.contains(&codigo.as_str());
            let principal = usados.first() == Some(&codigo.as_str());
            let acao: Element<'a, Msg> = if principal {
                text(tr("Principal")).size(12).color(COR_AZUL_ACENTO()).into()
            } else if usa {
                opcao(tr("Tornar principal"), false, Msg::OrtPrincipal(codigo.clone()))
            } else {
                Space::new(Length::Shrink, Length::Shrink).into()
            };
            let cod = codigo.clone();
            col = col.push(linha(
                &nome_do_idioma(codigo),
                if usa { tr("Em uso") } else { tr("Desligado") },
                row![
                    acao,
                    toggler("", usa).on_toggle(move |b| Msg::OrtIdioma(cod.clone(), b))
                ]
                .spacing(14)
                .align(Align::Center),
            ));
        }
        col = col.push(linha(
            tr("Adicionar idioma"),
            tr("Escolha um dicionário Hunspell (.dic, com o .aff de mesmo nome na mesma pasta)."),
            opcao(tr("Adicionar idioma…"), false, Msg::OrtAdicionar),
        ));
        let limpar: Element<'a, Msg> = if self.confirmar_limpar {
            row![
                opcao(tr("Sim, apagar"), true, Msg::OrtLimparSim),
                opcao(tr("Cancelar"), false, Msg::OrtLimparNao)
            ]
            .spacing(6)
            .into()
        } else {
            opcao(tr("Esvaziar…"), false, Msg::OrtLimpar)
        };
        if !self.aviso.is_empty() {
            col = col.push(text(self.aviso.clone()).size(12.5).color(COR_AZUL_ACENTO()));
        }
        col = col.push(linha(tr("Dicionário pessoal"), &crate::localidade::trf("{} palavra(s) que você adicionou. Esvaziar faz elas voltarem a ser marcadas como erro.", &[&self.n_pessoais]), limpar));
        col.into()
    }

    pub fn view<'a>(
        &'a self,
        c: &Configuracao,
        proporcao: Proporcao,
        arquivo: Option<&str>,
        extras: Extras,
        meus: &[crate::modelos_usuario::MeuModelo],
    ) -> Element<'a, Msg> {
        let menu = feather_ui::configuracoes::menu(tr("Configurações"), [
            (tr("Aparência").to_owned(), self.secao == Secao::Aparencia, Msg::Secao(Secao::Aparencia)),
            (tr("Salvamento").to_owned(), self.secao == Secao::Salvamento, Msg::Secao(Secao::Salvamento)),
            (tr("Exibição").to_owned(), self.secao == Secao::Exibicao, Msg::Secao(Secao::Exibicao)),
            (tr("Modelos").to_owned(), self.secao == Secao::Modelos, Msg::Secao(Secao::Modelos)),
            (tr("Apresentação").to_owned(), self.secao == Secao::Apresentacao, Msg::Secao(Secao::Apresentacao)),
            (tr("Ortografia").to_owned(), self.secao == Secao::Ortografia, Msg::Secao(Secao::Ortografia)),
            (texto("config_revisao").to_owned(), self.secao == Secao::Revisao, Msg::Secao(Secao::Revisao)),
            (texto("config_integracoes").to_owned(), self.secao == Secao::Integracoes, Msg::Secao(Secao::Integracoes)),
            (tr("Atalhos").to_owned(), self.secao == Secao::Atalhos, Msg::Secao(Secao::Atalhos)),
            (tr("Sobre").to_owned(), self.secao == Secao::Sobre, Msg::Secao(Secao::Sobre)),
        ]);

        let (titulo, corpo): (&str, Element<'a, Msg>) = match self.secao {
            Secao::Aparencia => (tr("Aparência"), self.aparencia(c)),
            Secao::Salvamento => (tr("Salvamento"), self.salvamento(c, arquivo)),
            Secao::Exibicao => (
                tr("Exibição"),
                column![
                    linha(tr("Painel de slides"), tr("Miniaturas à esquerda para navegar e reordenar."), toggler("", c.painel_slides).on_toggle(Msg::PainelSlides)),
                    linha(tr("Anotações do apresentador"), tr("A caixa de anotações embaixo do slide."), toggler("", c.painel_notas).on_toggle(Msg::PainelNotas)),
                    linha(tr("Grade"), tr("Linhas de grade sobre o slide, para alinhar."), toggler("", extras.grade).on_toggle(Msg::Grade)),
                    linha(tr("Guias do centro"), tr("Linhas no meio do slide, na horizontal e na vertical."), toggler("", extras.guias).on_toggle(Msg::Guias)),
                ]
                .spacing(10)
                .into(),
            ),
            Secao::Apresentacao => (
                tr("Apresentação"),
                column![
                    linha(texto("config_slide"), texto("config_slide_dica"), opcao(texto("config_abrir"), false, Msg::ConfigurarSlide)),
                    linha(
                        tr("Tamanho do slide"),
                        tr("Proporção dos slides desta apresentação."),
                        row![
                            opcao(tr("Widescreen 16:9"), proporcao == Proporcao::Larga, Msg::Proporcao(Proporcao::Larga)),
                            opcao(tr("Padrão 4:3"), proporcao == Proporcao::Padrao, Msg::Proporcao(Proporcao::Padrao)),
                        ]
                        .spacing(4)
                    ),
                    linha(
                        tr("Apresentar em dois monitores"),
                        &crate::localidade::trf(
                            "Com dois monitores, a plateia vê o slide no segundo e você vê o próximo slide, as anotações e o cronômetro no principal. {}",
                            &[&if extras.monitores >= 2 { crate::localidade::trf("{} monitores encontrados.", &[&extras.monitores]) } else { tr("Nenhum segundo monitor encontrado agora (a apresentação usa a tela inteira do principal).").to_string() }]
                        ),
                        toggler("", extras.dois_monitores).on_toggle(Msg::DoisMonitores)
                    ),
                    linha(tr("Repetir até Esc"), tr("Ao chegar ao último slide, volta ao primeiro em vez de terminar."), toggler("", extras.repetir).on_toggle(Msg::Repetir)),
                ]
                .spacing(10)
                .into(),
            ),
            Secao::Modelos => (tr("Modelos"), self.modelos(meus)),
            Secao::Ortografia => (tr("Ortografia"), self.ortografia(c)),
            Secao::Revisao => (texto("config_revisao"), column![
                linha(texto("autocorrecao"), texto("config_auto_dica"), toggler("", c.autocorrecao).on_toggle(Msg::Autocorrecao)),
                linha(texto("capital"), texto("config_capital_dica"), toggler("", c.capitalizacao).on_toggle(Msg::Capitalizacao)),
                linha(texto("config_regras"), &format!("{} {}", c.correcoes.len(), texto("config_registros")), opcao(texto("config_gerenciar"), false, Msg::Ferramenta(Modo::Autocorrecao))),
                linha(texto("dicionario"), texto("config_dicionario_dica"), opcao(texto("config_gerenciar"), false, Msg::Ferramenta(Modo::Dicionario))),
                linha(texto("sinonimos"), &format!("{} {}", c.sinonimos.len(), texto("config_registros")), opcao(texto("config_gerenciar"), false, Msg::Ferramenta(Modo::Sinonimos))),
                linha(texto("config_glossario"), texto("config_glossario_dica"), opcao(texto("config_gerenciar"), false, Msg::Ferramenta(Modo::Traducao))),
            ].spacing(10).into()),
            Secao::Integracoes => (texto("config_integracoes"), column![
                text(texto("config_mcp_titulo")).size(17),
                text(texto("config_mcp_dica")).size(13),
                text("cargo run --release --manifest-path featherslide/crates/featherslide-mcp/Cargo.toml").size(12),
                text(texto("config_mcp_estado")).size(13).color(COR_TEXTO_MUTED()),
            ].spacing(12).into()),
            Secao::Atalhos => (tr("Atalhos de teclado"), self.atalhos(c)),
            Secao::Sobre => (
                tr("Sobre"),
                column![
                    text("FeatherSlide").size(22),
                    text(crate::localidade::trf("Versão {} · A suíte FeatherWorks", &[&env!("CARGO_PKG_VERSION")])).size(12.5).color(COR_TEXTO_MUTED()),
                    Space::new(Length::Shrink, 8),
                    text(tr("Programa de apresentações da família FeatherWorks. Motor 100% Rust; esta interface nativa não usa WebKit.")).size(13.5),
                    text(tr("• Privacidade total: sem telemetria, sem contas, sem nuvem.")).size(13.5),
                    text(tr("• Formato .fsdz: contêiner aberto (ZIP com colunas Arrow). Abre também .pptx e .odp.")).size(13.5),
                    text(tr("• Ícones da interface e da galeria: Tabler Icons 3.49.0 (licença MIT, © Paweł Kuna) — https://github.com/tabler/tabler-icons.")).size(13.5),
                    text(tr("• Dicionários de ortografia: Hunspell/VERO, instalados no sistema.")).size(13.5),
                    Space::new(Length::Shrink, 8),
                    feather_ui::creditos::colaboradores(),
                    Space::new(Length::Shrink, 8),
                    text(tr("Licença dos ícones Tabler")).size(15),
                    text(featherslide_core::icones::LICENCA_TABLER).size(11),
                ]
                .spacing(8)
                .into(),
            ),
        };

        feather_ui::configuracoes::tela(menu, titulo, corpo, None)
    }

    fn aparencia<'a>(&self, c: &Configuracao) -> Element<'a, Msg> {
        let cores = ACENTOS.iter().enumerate().fold(
            row![].spacing(8).align(Align::Center),
            |r, (i, (_, claro, escuro_))| {
                r.push(amostra(
                    if escuro_resolvido(c.tema_auto, c.escuro) { escuro_ } else { claro },
                    c.acento == i,
                    Msg::Acento(i),
                ))
            },
        );
        let idiomas = crate::localidade::IDIOMAS.iter().fold(row![].spacing(4), |r, (codigo, nome)| {
            // O nome de cada idioma fica na própria língua, sem tradução.
            r.push(button(text(*nome).size(12.5)).padding([6, 13])
                .style(estilo_botao_pequeno(crate::localidade::atual() == *codigo)).on_press(Msg::Idioma(codigo)))
        });
        column![
            linha(tr("Idioma da interface"), tr("Menus, faixa, mensagens e Ajuda. Troca na hora."), idiomas),
            linha(
                tr("Tema"),
                tr("Automático acompanha o sistema na abertura; não muda as cores dos slides."),
                row![
                    opcao(tr("Automático"), c.tema_auto, Msg::TemaAuto),
                    opcao(tr("Claro"), !c.tema_auto && !c.escuro, Msg::Escuro(false)),
                    opcao(tr("Escuro"), !c.tema_auto && c.escuro, Msg::Escuro(true)),
                ].spacing(4)
            ),
            linha(tr("Cor de destaque"), tr("Abas, botões ativos e seleção."), cores),
            linha(tr("Logotipo"), tr("Amarelo de fábrica ou cor escolhida"),
                opcao(tr("Restaurar amarelo padrão"), false, Msg::LogoPadrao)),
            linha(
                tr("Barra de título integrada"),
                tr("Salvar, voltar, pesquisa e Apresentar ficam na própria barra da janela, com minimizar, maximizar e fechar. Vale na próxima vez que abrir o programa. Desligada, a janela volta a usar a barra do sistema (que permite redimensionar pelas bordas)."),
                toggler("", c.barra_propria).on_toggle(Msg::BarraPropria)
            ),
        ]
        .spacing(10)
        .into()
    }

    fn salvamento<'a>(&self, c: &Configuracao, arquivo: Option<&str>) -> Element<'a, Msg> {
        let botoes = row![
            button(text(tr("Salvar agora")).size(12.5))
                .padding([6, 14])
                .style(estilo_botao_primario)
                .on_press(Msg::SalvarAgora),
            button(text(tr("Salvar como…")).size(12.5))
                .padding([6, 14])
                .style(estilo_botao_secundario)
                .on_press(Msg::SalvarComo),
            button(text(tr("Salvar cópia…")).size(12.5))
                .padding([6, 14])
                .style(estilo_botao_secundario)
                .on_press(Msg::SalvarCopia),
        ]
        .spacing(8);
        let intervalos = [
            (2u64, "2 s"),
            (5, "5 s"),
            (15, "15 s"),
            (30, "30 s"),
            (60, "1 min"),
        ]
        .into_iter()
        .fold(row![].spacing(4), |r, (v, rot)| {
            r.push(opcao(rot, c.intervalo == v, Msg::Intervalo(v)))
        });
        let mut col = column![
            linha(
                tr("Salvar"),
                &arquivo.map(|a| crate::localidade::trf("Arquivo atual: {}", &[&a])).unwrap_or_else(|| tr("Apresentação ainda não gravada no disco.").into()),
                botoes
            ),
            linha(
                tr("Salvamento automático"),
                tr("Grava após inatividade. Sem arquivo definido, mantém um rascunho automático em .fsdz."),
                toggler("", c.auto_salvar).on_toggle(Msg::AutoSalvar)
            ),
        ]
        .spacing(10);
        if c.auto_salvar {
            col = col.push(linha(
                tr("Gravar após inatividade"),
                tr("Tempo desde a última alteração."),
                intervalos,
            ));
        }
        col.push(text(tr("O FeatherSlide roda inteiro na sua máquina: sem telemetria, sem contas e sem envio de dados.")).size(12).color(COR_TEXTO_MUTED())).into()
    }

    fn modelos<'a>(&'a self, meus: &[crate::modelos_usuario::MeuModelo]) -> Element<'a, Msg> {
        let pequeno = |rotulo: &str, msg: Msg| {
            button(text(rotulo.to_string()).size(12.5))
                .padding([5, 11])
                .style(estilo_botao_secundario)
                .on_press(msg)
        };
        let criar = row![
            text_input(tr("Nome do modelo"), &self.nome_modelo)
                .on_input(Msg::NomeModelo)
                .on_submit(Msg::SalvarModelo)
                .padding([6, 8])
                .width(Length::Fixed(220.0)),
            button(text(tr("Salvar a apresentação atual como modelo")).size(12.5))
                .padding([6, 12])
                .style(estilo_botao_primario)
                .on_press(Msg::SalvarModelo),
        ]
        .spacing(8)
        .align(Align::Center);
        let mut col = column![
            linha(tr("Criar modelo"), tr("Guarda esta apresentação (slides, tema e visual) para começar outras a partir dela."), criar),
            linha(tr("Importar modelo"), tr("Traz um modelo de um arquivo .fsdz (ou .pptx / .odp, convertido)."), pequeno(tr("Importar…"), Msg::ImportarModelo)),
        ]
        .spacing(10);
        if !self.aviso.is_empty() {
            col = col.push(text(self.aviso.clone()).size(12.5).color(COR_AZUL_ACENTO()));
        }
        if meus.is_empty() {
            col = col.push(
                text(tr("Você ainda não tem modelos próprios."))
                    .size(12.5)
                    .color(COR_TEXTO_MUTED()),
            );
        }
        for (i, m) in meus.iter().enumerate() {
            col = col.push(linha(
                &m.nome,
                &m.caminho.display().to_string(),
                row![
                    pequeno(tr("Usar"), Msg::UsarModelo(i)),
                    pequeno(tr("Exportar…"), Msg::ExportarModelo(i)),
                    pequeno(tr("Remover"), Msg::RemoverModelo(i))
                ]
                .spacing(6),
            ));
        }
        col.into()
    }

    fn atalhos<'a>(&self, c: &Configuracao) -> Element<'a, Msg> {
        let mut col = column![row![
            text(tr("Clique na combinação e aperte as novas teclas. Esc cancela."))
                .size(12.5)
                .color(COR_TEXTO_MUTED())
                .width(Length::FILL),
            button(text(tr("Restaurar padrões")).size(12.5))
                .padding([6, 12])
                .style(estilo_botao_secundario)
                .on_press(Msg::RestaurarAtalhos),
        ]
        .align(Align::Center)]
        .spacing(10);
        if !self.aviso.is_empty() {
            col = col.push(
                text(self.aviso.clone())
                    .size(12.5)
                    .color(Color::rgb8(0xd9, 0x30, 0x25)),
            );
        }
        for (id, rotulo, padrao) in ATALHOS {
            let gravando = self.gravando == Some(id);
            let atual = if gravando {
                tr("Pressione as teclas…").to_string()
            } else {
                c.atalho(id)
            };
            let tecla = button(
                text(atual)
                    .size(12.5)
                    .align_x(Align::Center),
            )
            .width(Length::Fixed(190.0))
            .padding([6, 10])
            .style(move |t, s| {
                let mut e = estilo_botao_secundario(t, s);
                if gravando {
                    e.border.color = COR_AZUL_ACENTO();
                    e.border.width = 2.0;
                }
                e
            })
            .on_press(Msg::GravarAtalho(id));
            col = col.push(linha(rotulo, &crate::localidade::trf("Padrão: {}", &[&padrao]), tecla));
        }
        col.into()
    }
}
