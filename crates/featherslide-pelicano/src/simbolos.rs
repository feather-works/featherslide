//! A paleta de símbolos (Inserir → Símbolo): uma grade por categoria, no lugar do menu suspenso
//! simples. Desenhada com a DejaVu Sans, que tem setas, matemática e letras gregas completas.

use pelicano::{button, column, container, mouse_area, row, scrollable, stack, text, Space};
use pelicano::{Align, Border, Color, Element, Font, Length, Point, Shadow, Status, Style, Theme};

use crate::faixa::FaixaMensagem;
use crate::tema::*;

const FONTE: Font = Font::with_name("DejaVu Sans");
const POR_LINHA: usize = 10;

const CATEGORIAS: [(&str, &[&str]); 7] = [
    ("Comuns", &["©", "®", "™", "°", "±", "×", "÷", "§", "¶", "•", "…", "—", "–", "«", "»", "†", "‡", "№", "′", "″"]),
    ("Matemática", &["≤", "≥", "≠", "≈", "≡", "∞", "√", "∑", "∏", "∫", "∂", "∆", "∇", "∈", "∉", "⊂", "⊃", "∪", "∩", "∧", "∨", "¬", "∀", "∃", "∅", "½", "¼", "¾", "²", "³"]),
    ("Setas", &["→", "←", "↑", "↓", "↔", "↕", "⇒", "⇐", "⇑", "⇓", "⇔", "↗", "↘", "↙", "↖", "↩", "↪", "➜", "➔", "➤"]),
    ("Gregas", &["α", "β", "γ", "δ", "ε", "θ", "λ", "μ", "π", "ρ", "σ", "τ", "φ", "ω", "Δ", "Θ", "Λ", "Σ", "Φ", "Ω"]),
    ("Moedas", &["€", "£", "¥", "¢", "$", "₹", "₽", "₩", "₿", "₪"]),
    ("Marcas", &["✓", "✔", "✗", "✘", "★", "☆", "♥", "♦", "♣", "♠", "♪", "♫", "☀", "☁", "☂", "☎", "✉", "✂", "⚠", "☐", "☑", "☒", "●", "○", "■", "□", "▲", "▼", "◆", "◇"]),
    ("Sobrescritos", &["⁰", "¹", "⁴", "⁵", "⁶", "⁷", "⁸", "⁹", "₀", "₁", "₂", "₃", "₄", "₅", "₆", "₇", "₈", "₉"]),
];

/// A paleta, com um fundo que fecha ao clicar fora.
pub fn sobrepor<'a>(base: Element<'a, crate::app::Mensagem>, largura_janela: f32) -> Element<'a, crate::app::Mensagem> {
    use crate::app::Mensagem;
    let fundo = mouse_area(Space::new(Length::FILL, Length::FILL)).on_press(Mensagem::Faixa(FaixaMensagem::FecharSimbolos));
    let (esquerda, topo) = crate::medido::ancora("simbolo", 420.0, largura_janela, (330.0, 128.0));
    let painel = container(paleta().map(Mensagem::Faixa)).padding(pelicano::Padding { top: topo, left: esquerda, right: 0.0, bottom: 0.0 });
    stack![base, fundo, painel].into()
}

fn paleta<'a>() -> Element<'a, FaixaMensagem> {
    let mut corpo = column![].spacing(10).padding(12);
    for (nome, simbolos) in CATEGORIAS {
        corpo = corpo.push(text(nome).size(11).color(COR_TEXTO_MUTED()));
        for linha in simbolos.chunks(POR_LINHA) {
            let mut r = row![].spacing(3);
            for s in linha {
                r = r.push(
                    button(text(*s).size(18).font(FONTE).width(Length::FILL).align_x(Align::Center))
                        .width(Length::Fixed(36.0))
                        .height(Length::Fixed(34.0))
                        .padding(0)
                        .style(estilo_celula_simbolo)
                        .on_press(FaixaMensagem::InserirSimbolo((*s).to_string())),
                );
            }
            corpo = corpo.push(r);
        }
    }
    container(scrollable(corpo).scrollbar_width(10).scrollbar_track(COR_TRILHO_ROLAGEM()).scrollbar_color(COR_POLEGAR_ROLAGEM()).height(Length::Fixed(430.0)))
        .width(Length::Fixed(420.0))
        .style(|_| Style {
            background: Some(COR_PAINEL().into()),
            text_color: Some(COR_TEXTO()),
            border: Border::rounded(8.0).color(COR_BORDA(), 1.0),
            shadow: Some(Shadow { color: Color::BLACK.alpha(0.3), offset: Point::new(0.0, 6.0), blur: 18.0 }),
            ..Style::default()
        })
        .into()
}

fn estilo_celula_simbolo(_: &Theme, status: Status) -> Style {
    let sobre = matches!(status, Status::Hovered | Status::Pressed);
    Style {
        background: Some((if sobre { COR_AZUL_ACENTO() } else { COR_HOVER() }).into()),
        text_color: Some(if sobre { Color::WHITE } else { COR_TEXTO() }),
        border: Border::rounded(5.0),
        ..Style::default()
    }
}
