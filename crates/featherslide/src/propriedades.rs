//! Painel de propriedades do documento: UI Iced, sem regra de negócio.
//! A edição usa um rascunho e um único comando de confirmação no motor Rust.

use featherslide_core::metadados::{data_utc, Estatisticas, Metadados};
use featherslide_core::modelo::Apresentacao;
use pelicano::*;
use pelicano::column;

use crate::app::Mensagem;
use crate::tema::*;

#[derive(Debug, Clone)]
pub enum Msg {
    Fechar,
    Salvar,
    Texto(&'static str, String),
    ChaveNova(bool, String),
    ValorNovo(bool, String),
    Adicionar(bool),
    Remover(bool, String),
}

#[derive(Debug, Clone)]
pub struct Estado {
    pub titulo: String,
    pub autor: String,
    pub dados: Metadados,
    pub nova_chave_personalizada: String,
    pub novo_valor_personalizada: String,
    pub nova_chave_estendida: String,
    pub novo_valor_estendida: String,
    pub aviso: String,
}

impl Estado {
    pub fn nova(ap: &Apresentacao) -> Self {
        Self {
            titulo: ap.titulo.clone(), autor: ap.autor.clone(), dados: ap.metadados.clone(),
            nova_chave_personalizada: String::new(), novo_valor_personalizada: String::new(),
            nova_chave_estendida: String::new(), novo_valor_estendida: String::new(),
            aviso: String::new(),
        }
    }

    /// Apenas altera o rascunho; cancelar não toca no documento.
    pub fn tratar(&mut self, mensagem: Msg) {
        match mensagem {
            Msg::Texto(campo, valor) => match campo {
                "titulo" => self.titulo = valor,
                "autor" => self.autor = valor,
                "assunto" => self.dados.assunto = valor,
                "empresa" => self.dados.empresa = valor,
                "gerente" => self.dados.gerente = valor,
                "categoria" => self.dados.categoria = valor,
                "palavras_chave" => self.dados.palavras_chave = valor,
                "comentarios" => self.dados.comentarios = valor,
                "idioma" => self.dados.idioma = valor,
                _ => {},
            },
            Msg::ChaveNova(estendido, valor) => {
                if estendido { self.nova_chave_estendida = valor; }
                else { self.nova_chave_personalizada = valor; }
            }
            Msg::ValorNovo(estendido, valor) => {
                if estendido { self.novo_valor_estendida = valor; }
                else { self.novo_valor_personalizada = valor; }
            }
            Msg::Adicionar(estendido) => {
                let (chave, valor, mapa) = if estendido {
                    (&mut self.nova_chave_estendida, &mut self.novo_valor_estendida,
                        &mut self.dados.estendidos)
                } else {
                    (&mut self.nova_chave_personalizada, &mut self.novo_valor_personalizada,
                        &mut self.dados.propriedades_personalizadas)
                };
                let nome = chave.trim();
                if nome.is_empty() || nome.len() > 128 {
                    self.aviso = "A chave precisa ter entre 1 e 128 bytes.".into();
                } else if mapa.contains_key(nome) {
                    self.aviso = "Já existe uma propriedade com essa chave.".into();
                } else if mapa.len() >= 100 || valor.len() > 16_384 {
                    self.aviso = "Limite de propriedades ou de valor excedido.".into();
                } else {
                    mapa.insert(nome.into(), std::mem::take(valor));
                    chave.clear();
                    self.aviso.clear();
                }
            }
            Msg::Remover(estendido, chave) => {
                if estendido { self.dados.estendidos.remove(&chave); }
                else { self.dados.propriedades_personalizadas.remove(&chave); }
            }
            Msg::Fechar | Msg::Salvar => {},
        }
    }
}

fn campo<'a>(rotulo: &'static str, id: &'static str, valor: &'a str) -> Element<'a, Msg> {
    row![
        text(rotulo).size(12).width(Length::Fixed(138.0)).color(COR_TEXTO()),
        text_input(rotulo, valor).on_input(move |s| Msg::Texto(id, s))
            .size(12).padding([6, 9]).width(Length::FILL),
    ].spacing(8).align(Align::Center).into()
}

fn informacao(rotulo: &str, valor: &str) -> Element<'static, Msg> {
    row![
        text(rotulo.to_string()).size(12).width(Length::Fixed(155.0)).color(COR_TEXTO_MUTED()),
        text(valor.to_string()).size(12).color(COR_TEXTO()),
    ].spacing(8).into()
}

fn painel_mapa<'a>(
    titulo: &'static str, mapa: &'a std::collections::BTreeMap<String, String>,
    chave_nova: &'a str, valor_novo: &'a str, estendido: bool,
) -> Element<'a, Msg> {
    let mut corpo = column![text(titulo).size(14)].spacing(7);
    for (chave, valor) in mapa {
        corpo = corpo.push(
            row![
                text(chave.as_str()).size(12).width(Length::Fixed(185.0)),
                text(valor.as_str()).size(12).width(Length::FILL),
                button(text("×").size(15)).padding([2, 8])
                    .style(estilo_botao_secundario)
                    .on_press(Msg::Remover(estendido, chave.clone())),
            ].spacing(5).align(Align::Center)
        );
    }
    corpo = corpo.push(
        row![
            text_input("Chave", chave_nova)
                .on_input(move |s| Msg::ChaveNova(estendido, s))
                .size(12).padding([5, 8]).width(Length::Fixed(165.0)),
            text_input("Valor", valor_novo)
                .on_input(move |s| Msg::ValorNovo(estendido, s))
                .size(12).padding([5, 8]).width(Length::FILL),
            button(text("+").size(14)).padding([5, 11])
                .style(estilo_botao_secundario).on_press(Msg::Adicionar(estendido)),
        ].spacing(7).align(Align::Center)
    );
    corpo.into()
}

/// Mostra um painel rolável; o rascunho somente é salvo quando "Aplicar" é clicado.
pub fn sobrepor<'a>(
    base: Element<'a, Mensagem>,
    estado: &'a Estado,
    estat: Estatisticas,
    largura_janela: f32,
) -> Element<'a, Mensagem> {
    let fundo = mouse_area(Space::new(Length::FILL, Length::FILL))
        .on_press(Mensagem::Propriedades(Msg::Fechar));
    let dados = &estado.dados;
    let mut campos = column![
        text("Propriedades da apresentação").size(18),
        text("Geral").size(14),
        campo("Título", "titulo", &estado.titulo),
        campo("Assunto", "assunto", &dados.assunto),
        campo("Autor", "autor", &estado.autor),
        campo("Empresa", "empresa", &dados.empresa),
        campo("Gerente", "gerente", &dados.gerente),
        campo("Categoria", "categoria", &dados.categoria),
        campo("Palavras-chave", "palavras_chave", &dados.palavras_chave),
        campo("Comentários", "comentarios", &dados.comentarios),
        campo("Idioma", "idioma", &dados.idioma),
        text("Identificação e histórico (gerados pelo FeatherSlide)").size(14),
        informacao("Identificador", &dados.identificador),
        informacao("Criação", &data_utc(dados.criado_em_utc)),
        informacao("Modificação", &data_utc(dados.modificado_em_utc)),
        informacao("Último autor", &dados.ultimo_autor),
        informacao("Revisões", &dados.revisoes.to_string()),
        informacao("Tempo de edição", &format!("{} minutos (estimativa)", dados.tempo_edicao_segundos / 60)),
        painel_mapa("Propriedades personalizadas", &dados.propriedades_personalizadas,
            &estado.nova_chave_personalizada, &estado.novo_valor_personalizada, false),
        painel_mapa("Metadados estendidos", &dados.estendidos,
            &estado.nova_chave_estendida, &estado.novo_valor_estendida, true),
        text("Estatísticas do documento (calculadas em tempo real)").size(14),
        informacao("Slides / ocultos", &format!("{} / {}", estat.slides, estat.slides_ocultos)),
        informacao("Elementos / formas", &format!("{} / {}", estat.elementos, estat.formas)),
        informacao("Caixas de texto", &estat.caixas_texto.to_string()),
        informacao("Imagens / gráficos", &format!("{} / {}", estat.imagens, estat.graficos)),
        informacao("Tabelas", &estat.tabelas.to_string()),
        informacao("Animações", &estat.animacoes.to_string()),
        informacao("Notas / seções", &format!("{} / {}", estat.notas, estat.secoes)),
        informacao("Comentários / resolvidos", &format!("{} / {}", estat.comentarios, estat.comentarios_resolvidos)),
        informacao("Palavras / caracteres", &format!("{} / {}", estat.palavras, estat.caracteres)),
        informacao("Fontes incorporadas", &estat.fontes_embutidas.to_string()),
    ].spacing(10);

    if !estado.aviso.is_empty() {
        campos = campos.push(text(&estado.aviso).size(12).color(COR_TEXTO_MUTED()));
    }

    let botoes = row![
        Space::new(Length::FILL, Length::Shrink),
        button(text("Cancelar").size(12.5)).padding([7, 14])
            .style(estilo_botao_secundario).on_press(Msg::Fechar),
        button(text("Aplicar").size(12.5)).padding([7, 14])
            .style(estilo_botao_primario).on_press(Msg::Salvar),
    ].spacing(8).align(Align::Center);

    let painel: Element<'a, Msg> = container(column![
        scrollable(campos).scrollbar_width(10).scrollbar_track(COR_TRILHO_ROLAGEM()).scrollbar_color(COR_POLEGAR_ROLAGEM()).height(Length::Fixed(550.0)),
        botoes,
    ].spacing(12))
        .width(Length::Fixed(700.0))
        .padding(18)
        .style(|_| Style {
            background: Some(Background::Color(COR_PAINEL())),
            text_color: Some(COR_TEXTO()),
            border: Border { color: COR_BORDA(), width: 1.0, radius: [8.0; 4] },
            shadow: Some(Shadow { color: Color::rgb8(0, 0, 0).alpha(0.3),
                offset: Point::new(0.0, 6.0), blur: 18.0 }),
            ..Default::default()
        }).into();
    let esquerda = ((largura_janela - 700.0) / 2.0).max(8.0);
    let pop = container(painel.map(Mensagem::Propriedades))
        .padding(Padding { top: 48.0, left: esquerda, right: 0.0, bottom: 0.0 });
    stack![base, fundo, pop].into()
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn cancelar_nao_modifica_documento() {
        let original = featherslide_core::sessao::Sessao::default();
        let mut estado = Estado::nova(original.apresentacao());
        estado.tratar(Msg::Texto("titulo", "Novo título".into()));
        estado.tratar(Msg::ChaveNova(false, "Cliente".into()));
        estado.tratar(Msg::ValorNovo(false, "Empresa X".into()));
        estado.tratar(Msg::Adicionar(false));
        assert_eq!(estado.dados.propriedades_personalizadas["Cliente"], "Empresa X");
        assert_ne!(estado.titulo, original.apresentacao().titulo);
        assert!(!original.apresentacao().metadados.propriedades_personalizadas.contains_key("Cliente"));
    }
}
