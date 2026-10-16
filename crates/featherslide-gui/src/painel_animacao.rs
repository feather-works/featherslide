//! O Painel de animação: a lista das animações do slide atual na ordem em que acontecem, com
//! o passo de cada uma (cada clique abre um passo). Clique numa linha para selecionar o
//! elemento no slide; as setas mudam a ordem; o ✕ tira a animação.

use featherslide_core::modelo::{Animacao, InicioAnimacao, Tipo};
use featherslide_core::sessao::Sessao;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::icones::*;
use crate::tema::*;

#[derive(Clone, Debug)]
pub enum Msg {
    Selecionar(u64),
    Mover(u64, i32),
    Remover(u64),
}

pub fn atualizar(sessao: &mut Sessao, m: Msg) {
    match m {
        Msg::Selecionar(id) => {
            sessao.selecionar_por_id(id);
        }
        Msg::Mover(id, delta) => {
            sessao.selecionar_por_id(id);
            let _ = sessao.animacao_ordem(id, delta);
        }
        Msg::Remover(id) => {
            sessao.selecionar_por_id(id);
            let _ = sessao.animacao_definir(Animacao::default());
        }
    }
}

fn rotulo_do_elemento(sessao: &Sessao, id: u64) -> String {
    let Some(e) = sessao.slide().elementos.iter().find(|e| e.id == id) else { return format!("Elemento {id}") };
    let nome = match e.tipo {
        Tipo::Texto => "Texto",
        Tipo::Imagem => "Imagem",
        Tipo::Tabela => "Tabela",
        Tipo::Grafico => "Gráfico",
        Tipo::Linha => "Linha",
        _ => "Forma",
    };
    let trecho: String = e.texto.lines().next().unwrap_or("").chars().take(28).collect();
    if trecho.is_empty() { nome.to_string() } else { format!("{nome}: {trecho}") }
}

pub fn view<'a>(sessao: &Sessao) -> Element<'a, Msg> {
    let passos = sessao.passos_de_animacao(sessao.atual());
    let selecao = sessao.estado().selecao;
    let mut lista = column![].spacing(4);
    let mut n = 0;
    for (i, passo) in passos.iter().enumerate() {
        lista = lista.push(text(format!("Clique {}", i + 1)).size(11.5).color(COR_TEXTO_MUTED()));
        for (id, _) in &passo.elementos {
            n += 1;
            let Some(a) = sessao.animacoes_do_slide(sessao.atual()).into_iter().find(|(i, _)| i == id).map(|(_, a)| a) else { continue };
            let marcado = selecao.contains(id);
            let inicio = match a.inicio {
                InicioAnimacao::AoClicar => "Ao clicar",
                InicioAnimacao::ComAnterior => "Com a anterior",
                InicioAnimacao::AposAnterior => "Após a anterior",
            };
            let id = *id;
            let linha = row![
                button(
                    row![
                        text(format!("{n}")).size(12).color(COR_TEXTO_MUTED()).width(Length::Fixed(18.0)),
                        column![
                            text(rotulo_do_elemento(sessao, id)).size(13),
                            text(format!("{} · {inicio} · {:.1} s", a.tipo.nome(), a.duracao)).size(11).color(COR_TEXTO_MUTED()),
                        ]
                        .spacing(1),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .padding([6, 10])
                .style(estilo_botao_pequeno(marcado))
                .on_press(Msg::Selecionar(id)),
                button(icone_subir(13.0, "#5f6368")).padding([6, 7]).style(estilo_botao_pequeno(false)).on_press(Msg::Mover(id, -1)),
                button(icone_descer(13.0, "#5f6368")).padding([6, 7]).style(estilo_botao_pequeno(false)).on_press(Msg::Mover(id, 1)),
                button(icone_cancelar(13.0, "#5f6368")).padding([6, 7]).style(estilo_botao_pequeno(false)).on_press(Msg::Remover(id)),
            ]
            .spacing(2)
            .align_y(Alignment::Center);
            lista = lista.push(linha);
        }
    }
    let corpo: Element<'a, Msg> = if n == 0 {
        column![
            text("Este slide ainda não tem animações.").size(14),
            text("Selecione um elemento no slide e escolha um efeito na aba Animações.").size(12.5).color(COR_TEXTO_MUTED()),
        ]
        .spacing(6)
        .into()
    } else {
        crate::tema::rolagem(lista).height(Length::Fill).into()
    };
    container(column![text(format!("Slide {}", sessao.atual() + 1)).size(17), Space::with_height(4), corpo].spacing(8)).padding(18).into()
}
