//! Imprimir (Ctrl+P): escolhe a impressora, as cópias e o que imprimir; o PDF do motor
//! (`featherslide-io::pdf`) vai para o CUPS (`lp`) no Linux e no macOS. No Windows não há `lp`:
//! o PDF abre no leitor do sistema, de onde se imprime. Tudo local, nada sai da máquina.

use std::path::Path;
use std::process::Command;

use iced::widget::{button, column, container, mouse_area, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Vector};

use crate::app::Mensagem;
use crate::tema::*;

#[derive(Debug, Clone)]
pub enum ImprMsg {
    Impressora(usize),
    /// "Salvar como PDF": em vez da impressora, grava um arquivo.
    Pdf,
    Copias(i32),
    SoAtual(bool),
    Confirmar,
    Cancelar,
}

pub struct Impressao {
    pub impressoras: Vec<String>,
    pub escolhida: usize,
    /// Salvar como PDF no lugar de imprimir (sempre disponível; já vem marcado quando não há impressora).
    pub pdf: bool,
    pub copias: u32,
    pub so_atual: bool,
    pub enviando: bool,
}

impl Impressao {
    pub fn nova() -> Self {
        let (impressoras, escolhida) = listar();
        let pdf = impressoras.is_empty();
        Self { impressoras, escolhida, pdf, copias: 1, so_atual: false, enviando: false }
    }

    /// A impressora escolhida; `None` se for "Salvar como PDF".
    pub fn impressora(&self) -> Option<&str> {
        if self.pdf {
            return None;
        }
        self.impressoras.get(self.escolhida).map(String::as_str)
    }
}

/// As impressoras do sistema e o índice da padrão. Vazio onde não há CUPS (Windows).
fn listar() -> (Vec<String>, usize) {
    if cfg!(target_os = "windows") {
        return (Vec::new(), 0);
    }
    let saida = |args: &[&str]| Command::new("lpstat").args(args).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
    let nomes: Vec<String> = saida(&["-e"]).lines().map(str::trim).filter(|l| !l.is_empty()).map(String::from).collect();
    let padrao = saida(&["-d"]).rsplit(':').next().map(|s| s.trim().to_string()).unwrap_or_default();
    let indice = nomes.iter().position(|n| *n == padrao).unwrap_or(0);
    (nomes, indice)
}

/// Manda o PDF para a impressora. Devolve a mensagem para a barra de status.
pub fn enviar(arquivo: &Path, impressora: Option<&str>, copias: u32) -> Result<String, String> {
    if cfg!(target_os = "windows") {
        Command::new("rundll32").arg("url.dll,FileProtocolHandler").arg(arquivo).spawn().map_err(|e| e.to_string())?;
        return Ok("PDF aberto no leitor do sistema: imprima por lá (Ctrl+P).".into());
    }
    let impressora = impressora.ok_or("Nenhuma impressora encontrada. Dá para exportar como PDF e imprimir de outro lugar.")?;
    let saida = Command::new("lp").arg("-d").arg(impressora).arg("-n").arg(copias.to_string()).args(["-o", "fit-to-page"]).arg(arquivo).output().map_err(|e| format!("Não consegui chamar o lp: {e}"))?;
    if saida.status.success() {
        Ok(format!("Enviado para a impressora {impressora}."))
    } else {
        Err(format!("A impressora recusou: {}", String::from_utf8_lossy(&saida.stderr).trim()))
    }
}

fn botao<'a>(rotulo: &str, msg: Option<ImprMsg>, primario: bool) -> Element<'a, ImprMsg> {
    let b = button(text(rotulo.to_string()).size(12.5)).padding([5, 12]).on_press_maybe(msg);
    if primario { b.style(estilo_botao_primario).into() } else { b.style(estilo_botao_secundario).into() }
}

fn opcao<'a>(rotulo: &str, ativa: bool, msg: ImprMsg) -> Element<'a, ImprMsg> {
    button(text(rotulo.to_string()).size(12.5)).padding([5, 11]).style(estilo_botao_pequeno(ativa)).on_press(msg).into()
}

/// A caixa por cima do editor.
pub fn sobrepor<'a>(base: Element<'a, Mensagem>, i: &'a Impressao, tem_oculto: bool, largura_janela: f32) -> Element<'a, Mensagem> {
    let fundo = mouse_area(Space::new(Length::Fill, Length::Fill));
    let largura = 380.0;
    let mut impressoras = column![].spacing(3);
    impressoras = impressoras.push(opcao("Salvar como PDF", i.pdf, ImprMsg::Pdf));
    for (n, nome) in i.impressoras.iter().enumerate() {
        impressoras = impressoras.push(opcao(nome, !i.pdf && n == i.escolhida, ImprMsg::Impressora(n)));
    }
    if i.impressoras.is_empty() {
        impressoras = impressoras.push(text("Nenhuma impressora encontrada neste computador.").size(11.5).color(COR_TEXTO_MUTED()));
    }
    let pode_enviar = !i.enviando;
    let nota = if tem_oculto && !i.so_atual { "Slides ocultos não são impressos." } else { "" };
    let corpo = column![
        text("Imprimir").size(14),
        text("Impressora").size(11).color(COR_TEXTO_MUTED()),
        impressoras,
        text("O que imprimir").size(11).color(COR_TEXTO_MUTED()),
        row![opcao("Todos os slides", !i.so_atual, ImprMsg::SoAtual(false)), opcao("Só o slide atual", i.so_atual, ImprMsg::SoAtual(true))].spacing(4),
        text(nota).size(11).color(COR_TEXTO_MUTED()),
        row![
            text(if i.pdf { "" } else { "Cópias" }).size(12.5),
            botao("-", (!i.pdf).then_some(ImprMsg::Copias(-1)), false),
            text(if i.pdf { String::new() } else { i.copias.to_string() }).size(13),
            botao("+", (!i.pdf).then_some(ImprMsg::Copias(1)), false),
            Space::new(Length::Fill, Length::Shrink),
            botao("Cancelar", Some(ImprMsg::Cancelar), false),
            botao(if i.enviando { "Enviando…" } else if i.pdf { "Salvar PDF…" } else { "Imprimir" }, pode_enviar.then_some(ImprMsg::Confirmar), true),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(8);
    let caixa = container(corpo)
        .width(Length::Fixed(largura))
        .padding(14)
        .style(|_| container::Style {
            background: Some(Background::Color(COR_PAINEL())),
            text_color: Some(COR_TEXTO()),
            border: Border { color: COR_BORDA(), width: 1.0, radius: 8.0.into() },
            shadow: Shadow { color: Color::from_rgba8(0, 0, 0, 0.3), offset: Vector::new(0.0, 6.0), blur_radius: 18.0 },
        });
    let esquerda = ((largura_janela - largura) / 2.0).max(8.0);
    let painel = container(Element::from(caixa).map(Mensagem::Imprimir)).padding(iced::Padding { top: 150.0, left: esquerda, right: 0.0, bottom: 0.0 });
    iced::widget::stack![base, fundo, painel].into()
}
