//! Recursos locais de revisão. As substituições exigem confirmação; as regras são opcionais.
use crate::localidade::texto as tr;
use crate::{app::Mensagem, configuracao::Configuracao, tema::*};
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Modo {
    Dicionario,
    Sinonimos,
    Autocorrecao,
    Idioma,
    Traducao,
    Contagem,
}
impl Modo {
    pub fn titulo(self) -> &'static str {
        match self {
            Self::Dicionario => tr("dicionario"),
            Self::Sinonimos => tr("sinonimos"),
            Self::Autocorrecao => tr("autocorrecao"),
            Self::Idioma => tr("idioma"),
            Self::Traducao => tr("traducao"),
            Self::Contagem => tr("contagem"),
        }
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    Fechar,
    Palavra(String),
    Valor(String),
    Origem(String),
    Destino(String),
    Buscar,
    Salvar,
    Remover,
    Aplicar(String),
    Auto(bool),
    Capital(bool),
    Idioma(String),
    Instalar,
    Adicionar,
}
pub struct Estado {
    pub modo: Modo,
    pub palavra: String,
    pub valor: String,
    pub origem: String,
    pub destino: String,
    pub resultados: Vec<String>,
    pub aviso: String,
    /// O trecho de origem e seu local; mudanças posteriores invalidam a aplicação.
    pub alvo: Option<(u64, usize, usize, String)>,
}
impl Estado {
    pub fn novo(modo: Modo) -> Self {
        Self {
            modo,
            palavra: String::new(),
            valor: String::new(),
            origem: "pt_BR".into(),
            destino: "en_US".into(),
            resultados: vec![],
            aviso: String::new(),
            alvo: None,
        }
    }
    pub fn chave(&self) -> String {
        if self.modo == Modo::Traducao {
            format!(
                "{}|{}|{}",
                self.origem.trim(),
                self.destino.trim(),
                self.palavra.trim()
            )
        } else {
            self.palavra.trim().to_lowercase()
        }
    }
}

/// Corrige a palavra anterior ao delimitador; índices sempre em limites UTF-8.
pub fn corrigir(texto: &mut String, cursor: &mut usize, c: &Configuracao) {
    if *cursor > texto.len() || !texto.is_char_boundary(*cursor) {
        return;
    }
    if c.autocorrecao {
        if let Some((fim, ultimo)) = texto[..*cursor].char_indices().next_back() {
            if !ultimo.is_alphanumeric() && ultimo != '\'' && ultimo != '-' {
                let inicio = texto[..fim]
                    .char_indices()
                    .rev()
                    .find(|(_, ch)| !ch.is_alphabetic() && *ch != '\'')
                    .map_or(0, |(i, ch)| i + ch.len_utf8());
                let palavra = &texto[inicio..fim];
                if let Some(nova) = c.correcoes.get(&palavra.to_lowercase()) {
                    let nova = if palavra.chars().all(|ch| ch.is_uppercase()) {
                        nova.to_uppercase()
                    } else if palavra.chars().next().is_some_and(char::is_uppercase) {
                        maiuscula(nova)
                    } else {
                        nova.clone()
                    };
                    let antigo = fim - inicio;
                    texto.replace_range(inicio..fim, &nova);
                    *cursor = *cursor - antigo + nova.len();
                }
            }
        }
    }
    if c.capitalizacao {
        if let Some((i, ch)) = texto[..*cursor].char_indices().next_back() {
            let antes = texto[..i].trim_end();
            if ch.is_lowercase()
                && (antes.is_empty()
                    || antes.ends_with(['.', '!', '?', '\n'])
                    || texto[..i].ends_with('\n'))
            {
                let nova = ch.to_uppercase().to_string();
                texto.replace_range(i..*cursor, &nova);
                *cursor = i + nova.len();
            }
        }
    }
}
fn maiuscula(s: &str) -> String {
    let mut cs = s.chars();
    cs.next().map_or_else(String::new, |ch| {
        ch.to_uppercase().collect::<String>() + cs.as_str()
    })
}

impl crate::app::FeatherSlideApp {
    pub(crate) fn abrir_ferramentas_texto(&mut self, modo: Modo) {
        let mut e = Estado::novo(modo);
        self.ortografia = None;
        if let Some(id) = self
            .editando_id
            .or_else(|| self.sessao.estado().selecao.first().copied())
        {
            if let Some(elemento) = self
                .sessao
                .slide()
                .elementos
                .iter()
                .find(|e| e.id == id)
                .filter(|e| e.tipo.aceita_texto())
            {
                let texto = &elemento.texto;
                let (a, b) = if self.editando_id == Some(id) {
                    featherslide_core::cursor::intervalo(
                        texto,
                        self.ancora_edicao,
                        self.cursor_edicao,
                    )
                    .unwrap_or((0, texto.len()))
                } else {
                    (0, texto.len())
                };
                if let Some(trecho) = texto.get(a..b) {
                    e.palavra = trecho.to_string();
                    e.alvo = Some((id, a, b, trecho.to_string()));
                }
            }
        }
        if modo == Modo::Contagem {
            let mut palavras = 0;
            let mut caracteres = 0;
            let mut sem_espacos = 0;
            for slide in &self.sessao.apresentacao().slides {
                for elemento in &slide.elementos {
                    if elemento.papel == featherslide_core::modelo::Papel::Fundo {
                        continue;
                    }
                    contar(
                        &elemento.texto,
                        &mut palavras,
                        &mut caracteres,
                        &mut sem_espacos,
                    );
                    if elemento.tipo == featherslide_core::modelo::Tipo::Tabela {
                        for linha in featherslide_core::dados::ler_tabela(elemento) {
                            for celula in linha {
                                contar(&celula, &mut palavras, &mut caracteres, &mut sem_espacos);
                            }
                        }
                    }
                    if elemento.tipo == featherslide_core::modelo::Tipo::Grafico {
                        let grafico = featherslide_core::dados::ler_grafico(elemento);
                        contar(
                            &grafico.titulo,
                            &mut palavras,
                            &mut caracteres,
                            &mut sem_espacos,
                        );
                        for (i, linha) in grafico.grade.iter().enumerate() {
                            for (j, celula) in linha.iter().enumerate() {
                                if i == 0 || j == 0 {
                                    contar(
                                        celula,
                                        &mut palavras,
                                        &mut caracteres,
                                        &mut sem_espacos,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            e.aviso = format!("Texto dos slides (inclui tabelas e rótulos de gráficos; exclui fundos, notas e comentários): {palavras} palavras; {caracteres} caracteres; {sem_espacos} sem espaços.\nTrecho selecionado: {} palavras; {} caracteres.", e.palavra.split_whitespace().count(), e.palavra.chars().count());
        }
        self.ferramentas_texto = Some(e);
    }

    pub(crate) fn tratar_ferramentas_texto(&mut self, msg: Msg) -> iced::Task<Mensagem> {
        if matches!(msg, Msg::Fechar) {
            self.ferramentas_texto = None;
            return iced::Task::none();
        }
        if let Msg::Idioma(codigo) = msg {
            return self.tratar_config(crate::configuracoes::Msg::OrtPrincipal(codigo));
        }
        if matches!(msg, Msg::Instalar) {
            return self.tratar_config(crate::configuracoes::Msg::OrtAdicionar);
        }
        let Some(e) = self.ferramentas_texto.as_mut() else {
            return iced::Task::none();
        };
        match msg {
            Msg::Palavra(s) => {
                e.palavra = s;
                e.resultados.clear();
                e.aviso.clear();
            }
            Msg::Valor(s) => e.valor = s,
            Msg::Origem(s) => {
                e.origem = s;
                e.resultados.clear();
            }
            Msg::Destino(s) => {
                e.destino = s;
                e.resultados.clear();
            }
            Msg::Auto(b) => {
                self.config.autocorrecao = b;
                self.config.gravar();
            }
            Msg::Capital(b) => {
                self.config.capitalizacao = b;
                self.config.gravar();
            }
            Msg::Buscar => {
                e.resultados.clear();
                let chave = e.chave();
                match e.modo {
                    Modo::Dicionario => {
                        e.aviso = if let Some(c) = &self.corretor {
                            if c.palavra_ok(e.palavra.trim()) {
                                tr("reconhecida").into()
                            } else {
                                e.resultados = c.sugestoes(e.palavra.trim());
                                tr("nao_reconhecida").into()
                            }
                        } else {
                            tr("indisponivel").into()
                        };
                    }
                    _ => {
                        let mapa = match e.modo {
                            Modo::Sinonimos => &self.config.sinonimos,
                            Modo::Traducao => &self.config.traducoes,
                            _ => &self.config.correcoes,
                        };
                        if let Some(v) = mapa.get(&chave) {
                            e.valor = v.clone();
                            e.resultados = if e.modo == Modo::Sinonimos {
                                v.split(';')
                                    .map(str::trim)
                                    .filter(|s| !s.is_empty())
                                    .map(str::to_owned)
                                    .collect()
                            } else {
                                vec![v.clone()]
                            };
                            e.aviso = tr("confirmar").into();
                        } else {
                            e.aviso = tr("sem_resultado").into();
                        }
                    }
                }
            }
            Msg::Salvar | Msg::Remover => {
                let chave = e.chave();
                if e.palavra.trim().is_empty()
                    || (matches!(msg, Msg::Salvar) && e.valor.trim().is_empty())
                {
                    e.aviso = tr("informe").into();
                } else if e.modo == Modo::Autocorrecao
                    && !e.palavra.chars().all(|c| c.is_alphabetic() || c == '\'')
                {
                    e.aviso = tr("uma_palavra").into();
                } else {
                    let mapa = match e.modo {
                        Modo::Sinonimos => &mut self.config.sinonimos,
                        Modo::Traducao => &mut self.config.traducoes,
                        _ => &mut self.config.correcoes,
                    };
                    if matches!(msg, Msg::Remover) {
                        mapa.remove(&chave);
                    } else {
                        mapa.insert(chave, e.valor.trim().to_owned());
                    }
                    self.config.gravar();
                    e.resultados.clear();
                    e.aviso = tr("atualizado").into();
                }
            }
            Msg::Adicionar => {
                e.aviso = match self.corretor.as_mut() {
                    Some(c)
                        if !e.palavra.trim().is_empty()
                            && !e.palavra.trim().contains(char::is_whitespace) =>
                    {
                        match c.adicionar_ao_dicionario(e.palavra.trim()) {
                            Ok(_) => tr("adicionada").into(),
                            Err(err) => err,
                        }
                    }
                    _ => tr("aguarde").into(),
                };
            }
            Msg::Aplicar(nova) => {
                if let Some((id, a, b, original)) = &e.alvo {
                    let texto = self
                        .sessao
                        .slide()
                        .elementos
                        .iter()
                        .find(|e| e.id == *id)
                        .map(|e| e.texto.clone());
                    if let Some(mut texto) =
                        texto.filter(|t| t.get(*a..*b) == Some(original.as_str()))
                    {
                        texto.replace_range(*a..*b, &nova);
                        match self.sessao.texto_definir(*id, &texto) {
                            Ok(_) => {
                                if self.editando_id == Some(*id) {
                                    self.texto_edicao = texto;
                                    self.cursor_edicao = *a + nova.len();
                                    self.ancora_edicao = None;
                                }
                                e.alvo = Some((*id, *a, *a + nova.len(), nova));
                                e.aviso = tr("aplicada").into();
                            }
                            Err(err) => e.aviso = err,
                        }
                    } else {
                        e.aviso = tr("texto_mudou").into();
                    }
                } else {
                    e.aviso = tr("selecione").into();
                }
            }
            _ => {}
        }
        iced::Task::none()
    }
}
fn contar(t: &str, palavras: &mut usize, caracteres: &mut usize, sem_espacos: &mut usize) {
    *palavras += t.split_whitespace().count();
    *caracteres += t.chars().count();
    *sem_espacos += t.chars().filter(|c| !c.is_whitespace()).count();
}

fn botao(label: &str, msg: Msg) -> Element<'static, Msg> {
    button(text(label.to_owned()).size(12))
        .padding([5, 8])
        .style(estilo_botao_secundario)
        .on_press(msg)
        .into()
}
pub fn sobrepor<'a>(
    base: Element<'a, Mensagem>,
    e: &'a Estado,
    c: &'a Configuracao,
) -> Element<'a, Mensagem> {
    let mut conteudo = column![row![
        text(e.modo.titulo()).size(18),
        Space::new(Length::Fill, 1),
        botao(tr("fechar"), Msg::Fechar)
    ]
    .spacing(8)]
    .spacing(10);
    match e.modo {
        Modo::Contagem => {}
        Modo::Idioma => {
            conteudo = conteudo.push(text(tr("sobre_idioma")).size(12));
            for idioma in featherslide_ortografia::idiomas_disponiveis() {
                let rotulo = format!(
                    "{}{}",
                    idioma.codigo,
                    if c.idiomas_ortografia.first() == Some(&idioma.codigo) {
                        " (principal)"
                    } else {
                        ""
                    }
                );
                conteudo = conteudo.push(botao(&rotulo, Msg::Idioma(idioma.codigo)));
            }
            conteudo = conteudo.push(botao(tr("instalar"), Msg::Instalar));
        }
        _ => {
            if e.modo == Modo::Autocorrecao {
                conteudo = conteudo
                    .push(checkbox(tr("auto"), c.autocorrecao).on_toggle(Msg::Auto))
                    .push(checkbox(tr("capital"), c.capitalizacao).on_toggle(Msg::Capital));
            }
            if e.modo == Modo::Traducao {
                conteudo = conteudo.push(text(tr("sobre_traducao")).size(12)).push(
                    row![
                        text_input(tr("origem"), &e.origem).on_input(Msg::Origem),
                        text_input(tr("destino"), &e.destino).on_input(Msg::Destino)
                    ]
                    .spacing(8),
                );
            }
            if e.modo == Modo::Sinonimos {
                conteudo = conteudo.push(text(tr("sobre_sinonimos")).size(12));
            }
            conteudo = conteudo.push(
                text_input(tr("palavra"), &e.palavra)
                    .on_input(Msg::Palavra)
                    .on_submit(Msg::Buscar),
            );
            if e.modo == Modo::Dicionario {
                conteudo = conteudo.push(row![
                    botao(tr("consultar"), Msg::Buscar),
                    botao(tr("adicionar"), Msg::Adicionar)
                ]);
            } else {
                conteudo = conteudo
                    .push(text_input(tr("valor"), &e.valor).on_input(Msg::Valor))
                    .push(
                        row![
                            botao(tr("consultar"), Msg::Buscar),
                            botao(tr("salvar"), Msg::Salvar),
                            botao(tr("excluir"), Msg::Remover)
                        ]
                        .spacing(6),
                    );
            }
            for resultado in &e.resultados {
                conteudo = conteudo.push(botao(resultado, Msg::Aplicar(resultado.clone())));
            }
        }
    }
    conteudo = conteudo.push(text(&e.aviso).size(12));
    let painel: Element<'a, Msg> = container(scrollable(conteudo))
        .padding(18)
        .width(480)
        .max_height(550)
        .style(estilo_painel_miniaturas)
        .into();
    let camada = container(painel.map(Mensagem::FerramentasTexto))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    iced::widget::stack![
        base,
        iced::widget::mouse_area(Space::new(Length::Fill, Length::Fill)),
        camada
    ]
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn digitacao_corrige_em_um_passo_desfazivel() {
        let mut app = crate::app::FeatherSlideApp::default();
        app.em_tela_inicio = false;
        app.config.autocorrecao = true;
        app.sessao.colar_sem_formatacao("voce").unwrap();
        app.editando_id = Some(app.sessao.estado().selecao[0]);
        app.texto_edicao = "voce".into();
        app.cursor_edicao = app.texto_edicao.len();
        let _ = app.update(Mensagem::Teclado(iced::Event::Keyboard(
            iced::keyboard::Event::KeyPressed {
                key: iced::keyboard::Key::Character(" ".into()),
                modified_key: iced::keyboard::Key::Character(" ".into()),
                physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::Space),
                location: iced::keyboard::Location::Standard,
                modifiers: iced::keyboard::Modifiers::default(),
                text: Some(" ".into()),
            },
        )));
        assert_eq!(app.texto_edicao, "você ");
        app.sessao.desfazer().unwrap();
        let id = app.editando_id.unwrap();
        assert_eq!(
            app.sessao
                .slide()
                .elementos
                .iter()
                .find(|e| e.id == id)
                .unwrap()
                .texto,
            "voce"
        );
    }
    #[test]
    fn pincel_pela_faixa_aplica_no_proximo_clique() {
        let mut app = crate::app::FeatherSlideApp::default();
        app.sessao.slide_novo("em_branco").unwrap();
        app.sessao.colar_sem_formatacao("origem").unwrap();
        app.sessao
            .formatar(featherslide_core::Formatacao::Negrito)
            .unwrap();
        let _ = app.update(Mensagem::Faixa(crate::faixa::FaixaMensagem::Pincel));
        app.sessao.colar_sem_formatacao("destino").unwrap();
        let id = app.sessao.estado().selecao[0];
        app.sessao
            .elemento_geometria(id, [100., 100., 200., 100.])
            .unwrap();
        let _ = app.update(Mensagem::Palco(crate::palco::PalcoMensagem::Clicou {
            x: 150.,
            y: 150.,
            somar: false,
            duplo: false,
        }));
        assert!(app.pincel.is_none());
        let destino = app
            .sessao
            .slide()
            .elementos
            .iter()
            .find(|e| e.id == id)
            .unwrap();
        assert_eq!(destino.texto, "destino");
        assert!(destino.estilo.negrito);
    }
    #[test]
    fn contagem_inclui_tabela_e_exclui_notas() {
        let mut app = crate::app::FeatherSlideApp::default();
        app.sessao.colar_sem_formatacao("ação importante").unwrap();
        app.sessao
            .notas_definir("notas não entram na contagem")
            .unwrap();
        app.sessao.tabela_inserir(1, 1).unwrap();
        let id = app.sessao.estado().selecao[0];
        app.sessao.tabela_celula_definir(id, 0, 0, "fim").unwrap();
        app.abrir_ferramentas_texto(Modo::Contagem);
        assert!(app
            .ferramentas_texto
            .as_ref()
            .unwrap()
            .aviso
            .contains("3 palavras"));
    }
    #[test]
    fn sinonimos_aplicam_so_na_selecao_e_desfazem() {
        let mut app = crate::app::FeatherSlideApp::default();
        app.sessao
            .colar_sem_formatacao("ação importante fim")
            .unwrap();
        let id = app.sessao.estado().selecao[0];
        app.editando_id = Some(id);
        app.texto_edicao = "ação importante fim".into();
        app.ancora_edicao = Some("ação ".len());
        app.cursor_edicao = "ação importante".len();
        app.abrir_ferramentas_texto(Modo::Sinonimos);
        let _ = app.tratar_ferramentas_texto(Msg::Buscar);
        assert!(app
            .ferramentas_texto
            .as_ref()
            .unwrap()
            .resultados
            .contains(&"relevante".into()));
        let _ = app.tratar_ferramentas_texto(Msg::Aplicar("relevante".into()));
        assert_eq!(app.texto_edicao, "ação relevante fim");
        app.sessao.desfazer().unwrap();
        assert_eq!(
            app.sessao
                .slide()
                .elementos
                .iter()
                .find(|e| e.id == id)
                .unwrap()
                .texto,
            "ação importante fim"
        );
    }
    #[test]
    fn mudanca_do_texto_invalida_sugestao_antiga() {
        let mut app = crate::app::FeatherSlideApp::default();
        app.sessao.colar_sem_formatacao("importante").unwrap();
        let id = app.sessao.estado().selecao[0];
        app.abrir_ferramentas_texto(Modo::Sinonimos);
        app.sessao.texto_definir(id, "novo conteúdo").unwrap();
        let _ = app.tratar_ferramentas_texto(Msg::Aplicar("relevante".into()));
        assert_eq!(
            app.sessao
                .slide()
                .elementos
                .iter()
                .find(|e| e.id == id)
                .unwrap()
                .texto,
            "novo conteúdo"
        );
    }
    #[test]
    fn glossario_respeita_a_direcao_e_nao_inventa_traducao() {
        let mut app = crate::app::FeatherSlideApp::default();
        app.ferramentas_texto = Some(Estado::novo(Modo::Traducao));
        let _ = app.tratar_ferramentas_texto(Msg::Palavra("bom dia".into()));
        let _ = app.tratar_ferramentas_texto(Msg::Valor("good morning".into()));
        let _ = app.tratar_ferramentas_texto(Msg::Salvar);
        let _ = app.tratar_ferramentas_texto(Msg::Buscar);
        assert_eq!(
            app.ferramentas_texto.as_ref().unwrap().resultados,
            vec!["good morning"]
        );
        let _ = app.tratar_ferramentas_texto(Msg::Destino("es_ES".into()));
        let _ = app.tratar_ferramentas_texto(Msg::Buscar);
        assert!(app
            .ferramentas_texto
            .as_ref()
            .unwrap()
            .resultados
            .is_empty());
    }
    #[test]
    fn configuracao_antiga_carrega_e_preserva_regras_novas() {
        let c: Configuracao = serde_json::from_str("{\"escuro\":false}").unwrap();
        assert!(!c.autocorrecao && !c.capitalizacao);
        let recuperada: Configuracao =
            serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert_eq!(recuperada.correcoes, c.correcoes);
        assert_eq!(recuperada.sinonimos, c.sinonimos);
    }
    #[test]
    fn regras_respeitam_utf8_e_preservam_sufixo() {
        let c = Configuracao {
            autocorrecao: true,
            ..Configuracao::default()
        };
        let mut s = "ação nao fim".to_string();
        let mut cursor = "ação nao ".len();
        corrigir(&mut s, &mut cursor, &c);
        assert_eq!(s, "ação não fim");
        assert_eq!(cursor, "ação não ".len());
    }
    #[test]
    fn capitalizacao_e_regras_sao_opcionais() {
        let mut s = "voce ".to_string();
        let mut i = s.len();
        corrigir(&mut s, &mut i, &Configuracao::default());
        assert_eq!(s, "voce ");
        let c = Configuracao {
            capitalizacao: true,
            ..Configuracao::default()
        };
        let mut s = "Olá! é".to_string();
        let mut i = s.len();
        corrigir(&mut s, &mut i, &c);
        assert_eq!(s, "Olá! É");
        assert_eq!(i, s.len());
    }
}
