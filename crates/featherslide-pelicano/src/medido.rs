//! `Medido`: um widget que só embrulha outro e anota, a cada desenho, onde ele ficou na janela.
//! As galerias (símbolos, formas, layouts…) usam isso para abrir logo embaixo do botão que as abriu,
//! sem depender de números fixos.

use std::cell::RefCell;
use std::collections::HashMap;

use pelicano::{Captured, DrawCx, Element, Event, EventCx, LayoutCx, Length, Limits, Node, Point, Rect, Widget};

thread_local! {
    static POSICOES: RefCell<HashMap<&'static str, Rect>> = RefCell::new(HashMap::new());
}

/// Onde o widget `nome` foi desenhado pela última vez (pixels da janela).
pub fn posicao(nome: &'static str) -> Option<Rect> {
    POSICOES.with(|p| p.borrow().get(nome).copied())
}

/// Onde abrir um painel de `largura` embaixo do widget `nome`: (esquerda, topo), sem sair da janela.
/// Sem medida ainda, usa `padrao`.
pub fn ancora(nome: &'static str, largura: f32, largura_janela: f32, padrao: (f32, f32)) -> (f32, f32) {
    match posicao(nome) {
        Some(r) => ((r.x).min(largura_janela - largura - 8.0).max(4.0), r.y + r.height + 4.0),
        None => padrao,
    }
}

pub struct Medido<'a, M> {
    nome: &'static str,
    conteudo: Element<'a, M>,
}

pub fn medido<'a, M: 'a>(nome: &'static str, conteudo: impl Into<Element<'a, M>>) -> Medido<'a, M> {
    Medido { nome, conteudo: conteudo.into() }
}

impl<M> Widget<M> for Medido<'_, M> {
    fn size(&self) -> (Length, Length) {
        self.conteudo.widget().size()
    }

    fn layout(&self, cx: &mut LayoutCx, limites: Limits) -> Node {
        let n = cx.child(0, &self.conteudo, limites);
        Node { rect: n.rect, key: cx.key(), children: vec![n], ..Default::default() }
    }

    fn draw(&self, node: &Node, cx: &mut DrawCx) {
        POSICOES.with(|p| p.borrow_mut().insert(self.nome, node.rect));
        self.conteudo.widget().draw(&node.children[0], cx)
    }

    fn event(&self, node: &Node, evento: &Event, cx: &mut EventCx<M>) -> Captured {
        self.conteudo.widget().event(&node.children[0], evento, cx)
    }

    fn opaque_at(&self, node: &Node, p: Point) -> bool {
        self.conteudo.widget().opaque_at(&node.children[0], p)
    }
}

impl<'a, M: 'a> From<Medido<'a, M>> for Element<'a, M> {
    fn from(m: Medido<'a, M>) -> Self {
        Element::new(m)
    }
}

/// Atalho: o `Element` comum, marcado com `nome` (para `ancora` achar onde ele ficou).
pub fn marcar<'a, M: 'a>(nome: &'static str, elemento: impl Into<Element<'a, M>>) -> Element<'a, M> {
    Element::new(medido(nome, elemento.into()))
}
