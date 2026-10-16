//! `Medido`: um widget que só embrulha outro e anota, a cada desenho, onde ele ficou na janela.
//! As galerias (símbolos, formas, layouts…) usam isso para abrir logo embaixo do botão que as abriu,
//! sem depender de números fixos.

use std::cell::RefCell;
use std::collections::HashMap;

use iced::advanced::widget::{tree, Operation, Tree};
use iced::advanced::{layout, mouse, overlay, renderer, Clipboard, Layout, Shell, Widget};
use iced::{event, Element, Event, Length, Rectangle, Size, Vector};

thread_local! {
    static POSICOES: RefCell<HashMap<&'static str, Rectangle>> = RefCell::new(HashMap::new());
}

/// Onde o widget `nome` foi desenhado pela última vez (pixels da janela).
pub fn posicao(nome: &'static str) -> Option<Rectangle> {
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

pub struct Medido<'a, M, T, R> {
    nome: &'static str,
    conteudo: Element<'a, M, T, R>,
}

pub fn medido<'a, M, T, R>(nome: &'static str, conteudo: impl Into<Element<'a, M, T, R>>) -> Medido<'a, M, T, R> {
    Medido { nome, conteudo: conteudo.into() }
}

impl<'a, M, T, R> Widget<M, T, R> for Medido<'a, M, T, R>
where
    R: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn state(&self) -> tree::State {
        tree::State::None
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.conteudo)]
    }

    fn diff(&self, arvore: &mut Tree) {
        arvore.diff_children(std::slice::from_ref(&self.conteudo));
    }

    fn size(&self) -> Size<Length> {
        self.conteudo.as_widget().size()
    }

    fn layout(&self, arvore: &mut Tree, renderer: &R, limites: &layout::Limits) -> layout::Node {
        self.conteudo.as_widget().layout(&mut arvore.children[0], renderer, limites)
    }

    fn operate(&self, arvore: &mut Tree, layout: Layout<'_>, renderer: &R, operacao: &mut dyn Operation) {
        self.conteudo.as_widget().operate(&mut arvore.children[0], layout, renderer, operacao)
    }

    fn on_event(
        &mut self,
        arvore: &mut Tree,
        evento: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &R,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.conteudo.as_widget_mut().on_event(&mut arvore.children[0], evento, layout, cursor, renderer, clipboard, shell, viewport)
    }

    fn mouse_interaction(&self, arvore: &Tree, layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle, renderer: &R) -> mouse::Interaction {
        self.conteudo.as_widget().mouse_interaction(&arvore.children[0], layout, cursor, viewport, renderer)
    }

    fn draw(&self, arvore: &Tree, renderer: &mut R, tema: &T, estilo: &renderer::Style, layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle) {
        POSICOES.with(|p| p.borrow_mut().insert(self.nome, layout.bounds()));
        self.conteudo.as_widget().draw(&arvore.children[0], renderer, tema, estilo, layout, cursor, viewport)
    }

    fn overlay<'b>(&'b mut self, arvore: &'b mut Tree, layout: Layout<'_>, renderer: &R, translation: Vector) -> Option<overlay::Element<'b, M, T, R>> {
        self.conteudo.as_widget_mut().overlay(&mut arvore.children[0], layout, renderer, translation)
    }
}

impl<'a, M: 'a, T: 'a, R: renderer::Renderer + 'a> From<Medido<'a, M, T, R>> for Element<'a, M, T, R> {
    fn from(m: Medido<'a, M, T, R>) -> Self {
        Element::new(m)
    }
}

/// Atalho: o `Element` comum, marcado com `nome` (para `ancora` achar onde ele ficou).
pub fn marcar<'a, M: 'a>(nome: &'static str, elemento: impl Into<Element<'a, M>>) -> Element<'a, M> {
    Element::new(medido(nome, elemento.into()))
}
