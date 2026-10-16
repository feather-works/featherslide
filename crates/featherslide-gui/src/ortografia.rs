//! O painel de Revisão → Ortografia: mostra a palavra errada, as sugestões e as ações. Achar
//! os erros e trocar a palavra é do motor (`featherslide-ortografia`); aqui só se desenha.

use std::collections::HashSet;

use featherslide_ortografia::{Corretor, Erro};
use iced::widget::{button, column, container, mouse_area, row, text, text_input, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Vector};

use crate::app::Mensagem;
use crate::tema::*;

pub const ID_CAMPO: &str = "campo_ortografia";

#[derive(Debug, Clone)]
pub enum OrtMsg {
    Texto(String),
    Sugestao(String),
    Alterar,
    Ignorar,
    IgnorarTodas,
    Adicionar,
    Fechar,
}

/// O que o painel guarda enquanto está aberto.
pub struct Ortografia {
    pub carregando: bool,
    pub aviso: String,
    pub erros: Vec<Erro>,
    pub indice: usize,
    pub correcao: String,
    pub sugestoes: Vec<String>,
    /// (elemento, palavra) que a pessoa mandou ignorar só ali.
    pub ignorados_aqui: HashSet<(u64, featherslide_ortografia::CampoErro, String)>,
}

impl Ortografia {
    pub fn nova() -> Self {
        Self { carregando: true, aviso: String::new(), erros: Vec::new(), indice: 0, correcao: String::new(), sugestoes: Vec::new(), ignorados_aqui: HashSet::new() }
    }

    pub fn atual(&self) -> Option<&Erro> {
        self.erros.get(self.indice)
    }

    /// Refaz a lista de erros (menos os ignorados) e volta ao primeiro; atualiza as sugestões.
    pub fn atualizar(&mut self, c: &Corretor, ap: &featherslide_core::modelo::Apresentacao) {
        self.erros = c.verificar(ap).into_iter().filter(|e| !self.ignorados_aqui.contains(&(e.elemento, e.campo, e.palavra.clone()))).collect();
        self.indice = 0;
        self.preparar(c);
    }

    /// Sugestões e campo de correção do erro atual.
    pub fn preparar(&mut self, c: &Corretor) {
        let Some(e) = self.erros.get(self.indice) else {
            self.sugestoes.clear();
            self.correcao.clear();
            return;
        };
        self.sugestoes = c.sugestoes(&e.palavra);
        self.correcao = self.sugestoes.first().cloned().unwrap_or_else(|| e.palavra.clone());
    }
}

fn botao<'a>(rotulo: &str, msg: OrtMsg, primario: bool) -> Element<'a, OrtMsg> {
    let b = button(text(rotulo.to_string()).size(12.5)).padding([5, 10]).on_press(msg);
    if primario { b.style(estilo_botao_primario).into() } else { b.style(estilo_botao_secundario).into() }
}

/// O painel por cima do editor, com um fundo que não fecha ao clicar fora (a verificação é um trabalho longo).
pub fn sobrepor<'a>(base: Element<'a, Mensagem>, o: &'a Ortografia, largura_janela: f32) -> Element<'a, Mensagem> {
    let fundo = mouse_area(Space::new(Length::Fill, Length::Fill));
    let largura = 400.0;
    let esquerda = (largura_janela - largura - 16.0).max(8.0);
    let painel = container(caixa(o).map(Mensagem::Ortografia)).padding(iced::Padding { top: 130.0, left: esquerda, right: 0.0, bottom: 0.0 });
    iced::widget::stack![base, fundo, painel].into()
}

fn caixa<'a>(o: &'a Ortografia) -> Element<'a, OrtMsg> {
    let corpo: Element<'a, OrtMsg> = if o.carregando {
        text("Carregando o dicionário…").size(13).into()
    } else if !o.aviso.is_empty() {
        column![text(o.aviso.as_str()).size(12.5), botao("Fechar", OrtMsg::Fechar, false)].spacing(10).into()
    } else if let Some(e) = o.atual() {
        let mut sugestoes = column![].spacing(3);
        for s in o.sugestoes.iter().take(6) {
            sugestoes = sugestoes.push(button(text(s.as_str()).size(12.5)).width(Length::Fill).padding([3, 8]).style(estilo_botao_pequeno(false)).on_press(OrtMsg::Sugestao(s.clone())));
        }
        if o.sugestoes.is_empty() {
            sugestoes = sugestoes.push(text("Sem sugestões.").size(12).color(COR_TEXTO_MUTED()));
        }
        column![
            text(format!("Erro {} de {} — slide {} — {}", o.indice + 1, o.erros.len(), e.slide + 1,
                match e.campo { featherslide_ortografia::CampoErro::Texto => "texto".into(), featherslide_ortografia::CampoErro::Notas => "notas".into(), featherslide_ortografia::CampoErro::Celula(l,c) => format!("célula {}, {}", l+1,c+1) })).size(11).color(COR_TEXTO_MUTED()),
            text(format!("Não está no dicionário: «{}»", e.palavra)).size(13.5),
            text_input("Correção", &o.correcao).id(text_input::Id::new(ID_CAMPO)).on_input(OrtMsg::Texto).on_submit(OrtMsg::Alterar).size(13).padding([6, 8]),
            sugestoes,
            row![botao("Alterar", OrtMsg::Alterar, true), botao("Ignorar", OrtMsg::Ignorar, false), botao("Ignorar tudo", OrtMsg::IgnorarTodas, false)].spacing(6),
            row![botao("Adicionar ao dicionário", OrtMsg::Adicionar, false), Space::new(Length::Fill, Length::Shrink), botao("Fechar", OrtMsg::Fechar, false)].spacing(6).align_y(Alignment::Center),
        ]
        .spacing(8)
        .into()
    } else {
        column![text("Verificação ortográfica concluída.").size(13.5), text("Nenhum erro encontrado.").size(12).color(COR_TEXTO_MUTED()), botao("Fechar", OrtMsg::Fechar, true)].spacing(8).into()
    };
    container(column![text("Ortografia").size(14), corpo].spacing(10))
        .width(Length::Fixed(400.0))
        .padding(14)
        .style(|_| container::Style {
            background: Some(Background::Color(COR_PAINEL())),
            text_color: Some(COR_TEXTO()),
            border: Border { color: COR_BORDA(), width: 1.0, radius: 8.0.into() },
            shadow: Shadow { color: Color::from_rgba8(0, 0, 0, 0.3), offset: Vector::new(0.0, 6.0), blur_radius: 18.0 },
        })
        .into()
}
