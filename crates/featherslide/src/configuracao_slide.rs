//! Painel de configuração do slide, com rascunhos locais.
use crate::app::Mensagem;
use featherslide_core::configuracao_slide::{Ajuste, Fundo};
use featherslide_core::sessao::Sessao;
use pelicano::column;
use pelicano::*;

#[derive(Clone, Debug)]
pub enum Msg {
    Fechar,
    Texto(&'static str, String),
    Ajuste(String),
    Tipo(String),
    Retrato(bool),
    Mestre(bool),
    Todos(bool),
    Imagem,
    AplicarTamanho,
    AplicarFundo,
}
#[derive(Clone, Debug)]
pub struct Estado {
    pub largura: String,
    pub altura: String,
    pub ajuste: String,
    pub cor: String,
    pub fundo: Fundo,
    pub opacidade: String,
    pub mestre: bool,
    pub todos: bool,
    pub aviso: String,
}
impl Estado {
    pub fn novo(s: &Sessao) -> Self {
        let (w, h) = s.apresentacao().tamanho();
        let slide = s.slide();
        Self {
            largura: w.to_string(),
            altura: h.to_string(),
            ajuste: "Ajustar".into(),
            cor: if slide.fundo.is_empty() {
                "#ffffff".into()
            } else {
                slide.fundo.clone()
            },
            fundo: slide.fundo_config.clone(),
            opacidade: (slide.fundo_config.opacidade * 100.0).to_string(),
            mestre: slide.ocultar_mestre,
            todos: false,
            aviso: String::new(),
        }
    }
    pub fn tratar(&mut self, msg: Msg) {
        match msg {
            Msg::Texto(campo, valor) => match campo {
                "largura" => self.largura = valor,
                "altura" => self.altura = valor,
                "cor" => self.cor = valor,
                "cor2" => self.fundo.cor2 = valor,
                "opacidade" => self.opacidade = valor,
                _ => {}
            },
            Msg::Ajuste(v) => self.ajuste = v,
            Msg::Tipo(v) => self.fundo.tipo = v,
            Msg::Mestre(v) => self.mestre = v,
            Msg::Todos(v) => self.todos = v,
            Msg::Retrato(retrato) => {
                if let (Ok(w), Ok(h)) = (numero(&self.largura), numero(&self.altura)) {
                    if (retrato && w > h) || (!retrato && h > w) {
                        std::mem::swap(&mut self.largura, &mut self.altura);
                    }
                }
            }
            _ => {}
        }
    }
    pub fn tamanho(&self) -> Result<(f32, f32, Ajuste), String> {
        let ajuste = match self.ajuste.as_str() {
            "Esticar" => Ajuste::Esticar,
            "Maximizar" => Ajuste::Maximizar,
            _ => Ajuste::Ajustar,
        };
        Ok((numero(&self.largura)?, numero(&self.altura)?, ajuste))
    }
    pub fn fundo(&self) -> Result<Fundo, String> {
        let mut f = self.fundo.clone();
        f.opacidade = numero(&self.opacidade)? / 100.0;
        Ok(f)
    }
}
fn numero(t: &str) -> Result<f32, String> {
    t.trim()
        .replace(',', ".")
        .parse()
        .map_err(|_| "Digite um número válido.".into())
}
fn enviar(m: Msg) -> Mensagem {
    Mensagem::ConfiguracaoSlide(m)
}
fn campo<'a>(titulo: &'a str, chave: &'static str, valor: &'a str) -> Element<'a, Mensagem> {
    column![
        text(titulo),
        text_input("", valor).on_input(move |v| enviar(Msg::Texto(chave, v)))
    ]
    .spacing(4)
    .into()
}
fn corpo(e: &Estado) -> Element<'_, Mensagem> {
    const TIPOS: [&str; 5] = ["Cor sólida", "Imagem", "Gradiente", "Textura", "Padrão"];
    const AJUSTES: [&str; 3] = ["Ajustar", "Maximizar", "Esticar"];
    const CODIGOS: [&str; 5] = ["solido", "imagem", "gradiente", "textura", "padrao"];
    let selecionado = TIPOS[CODIGOS.iter().position(|t| *t == e.fundo.tipo).unwrap_or(0)];
    column![
        text("Configuração do slide").size(22),
        text("Tamanho em pontos (72 pt = 1 polegada)"),
        row![
            campo("Largura", "largura", &e.largura),
            campo("Altura", "altura", &e.altura)
        ]
        .spacing(12),
        row![
            button("Paisagem").on_press(enviar(Msg::Retrato(false))),
            button("Retrato").on_press(enviar(Msg::Retrato(true)))
        ]
        .spacing(8),
        text("Ao mudar o tamanho: Ajustar preserva proporções; Maximizar pode cortar conteúdo."),
        pick_list(
            &AJUSTES,
            Some(e.ajuste.as_str()),
            |v: &str| enviar(Msg::Ajuste(v.to_string()))
        ),
        button("Aplicar tamanho à apresentação").on_press(enviar(Msg::AplicarTamanho)),
        text("Fundo").size(18),
        pick_list(
            &TIPOS,
            Some(selecionado),
            |v: &str| enviar(Msg::Tipo(
                CODIGOS[TIPOS.iter().position(|t| *t == v).unwrap_or(0)].to_string()
            ))
        ),
        row![
            campo("Cor #RRGGBB", "cor", &e.cor),
            campo("Segunda cor #RRGGBB", "cor2", &e.fundo.cor2)
        ]
        .spacing(12),
        text("Gradiente horizontal • Textura pontilhada • Padrão xadrez"),
        button(if e.fundo.imagem.is_empty() {
            "Escolher imagem local…"
        } else {
            "Substituir imagem local…"
        })
        .on_press(enviar(Msg::Imagem)),
        campo("Opacidade do fundo (0–100%)", "opacidade", &e.opacidade),
        checkbox("Ocultar elementos herdados do mestre/layout", e.mestre)
            .on_toggle(|v| enviar(Msg::Mestre(v))),
        checkbox("Aplicar fundo a todos os slides", e.todos).on_toggle(|v| enviar(Msg::Todos(v))),
        row![
            button("Aplicar fundo").on_press(enviar(Msg::AplicarFundo)),
            button("Fechar").on_press(enviar(Msg::Fechar))
        ]
        .spacing(8),
        text(&e.aviso),
    ]
    .spacing(10)
    .into()
}
pub fn sobrepor<'a>(base: Element<'a, Mensagem>, e: &'a Estado) -> Element<'a, Mensagem> {
    let overlay = container(
        container(scrollable(corpo(e)).scrollbar_width(10).scrollbar_track(crate::tema::COR_TRILHO_ROLAGEM()).scrollbar_color(crate::tema::COR_POLEGAR_ROLAGEM()))
            .padding(20)
            .width(Length::FILL)
            .max_width(620.0)
            .height(Length::FILL)
            .max_height(680)
            .style(crate::tema::estilo_painel_faixa),
    )
    .width(Length::FILL)
    .height(Length::FILL)
    .center();
    let fundo = mouse_area(Space::new(Length::FILL, Length::FILL)).on_press(enviar(Msg::Fechar));
    stack![base, fundo, overlay].into()
}
