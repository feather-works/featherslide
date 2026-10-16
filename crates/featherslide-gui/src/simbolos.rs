//! A paleta de símbolos (Inserir → Símbolo): uma grade por categoria, no lugar do menu suspenso
//! simples. Desenhada com a DejaVu Sans, que tem setas, matemática e letras gregas completas.

use iced::widget::{button, column, container, mouse_area, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Font, Length, Shadow, Vector};

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
    let fundo = mouse_area(Space::new(Length::Fill, Length::Fill)).on_press(Mensagem::Faixa(FaixaMensagem::FecharSimbolos));
    let (esquerda, topo) = crate::medido::ancora("simbolo", 420.0, largura_janela, (330.0, 128.0));
    let painel = container(paleta().map(Mensagem::Faixa)).padding(iced::Padding { top: topo, left: esquerda, right: 0.0, bottom: 0.0 });
    iced::widget::stack![base, fundo, painel].into()
}

fn paleta<'a>() -> Element<'a, FaixaMensagem> {
    let mut corpo = column![].spacing(10).padding(12);
    for (nome, simbolos) in CATEGORIAS {
        corpo = corpo.push(text(nome).size(11).color(COR_TEXTO_MUTED()));
        for linha in simbolos.chunks(POR_LINHA) {
            let mut r = row![].spacing(3);
            for s in linha {
                r = r.push(
                    button(text(*s).shaping(iced::widget::text::Shaping::Advanced).size(18).font(FONTE).width(Length::Fill).align_x(Alignment::Center))
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
    container(scrollable(corpo).height(Length::Fixed(430.0)))
        .width(Length::Fixed(420.0))
        .style(|_| container::Style {
            background: Some(Background::Color(COR_PAINEL())),
            text_color: Some(COR_TEXTO()),
            border: Border { color: COR_BORDA(), width: 1.0, radius: 8.0.into() },
            shadow: Shadow { color: Color::from_rgba8(0, 0, 0, 0.3), offset: Vector::new(0.0, 6.0), blur_radius: 18.0 },
        })
        .into()
}

fn estilo_celula_simbolo(_: &iced::Theme, status: button::Status) -> button::Style {
    let sobre = matches!(status, button::Status::Hovered | button::Status::Pressed);
    button::Style {
        background: Some(Background::Color(if sobre { COR_AZUL_ACENTO() } else { COR_HOVER() })),
        text_color: if sobre { Color::WHITE } else { COR_TEXTO() },
        border: Border { radius: 5.0.into(), ..Border::default() },
        shadow: Shadow::default(),
    }
}
