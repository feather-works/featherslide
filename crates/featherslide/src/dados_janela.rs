//! A janela "Dados do gráfico": o tipo, o título e a grade de valores (categorias nas linhas,
//! séries nas colunas). Cada tecla vai ao motor e o gráfico no slide se refaz na hora.

use featherslide_core::sessao::Sessao;
use pelicano::*;
use pelicano::column;

use crate::tema::*;

const TIPOS: [&str; 4] = ["Colunas", "Barras", "Linhas", "Pizza"];

#[derive(Clone, Debug)]
pub enum Msg {
    Tipo(String),
    Titulo(String),
    Celula(usize, usize, String),
    LinhaMais,
    LinhaMenos,
    SerieMais,
    SerieMenos,
}

fn rotulo_do_tipo(tipo: &str) -> &'static str {
    match tipo {
        "barras" => "Barras",
        "linhas" => "Linhas",
        "pizza" => "Pizza",
        _ => "Colunas",
    }
}

pub fn atualizar(sessao: &mut Sessao, id: u64, msg: Msg) {
    let Some(mut g) = sessao.grafico_dados(id) else {
        return;
    };
    match msg {
        Msg::Tipo(t) => g.tipo = t.to_lowercase(),
        Msg::Titulo(t) => g.titulo = t,
        Msg::Celula(r, c, valor) => {
            if let Some(cel) = g.grade.get_mut(r).and_then(|l| l.get_mut(c)) {
                *cel = valor;
            }
        }
        Msg::LinhaMais => {
            let n = g.grade.len();
            let mut nova = vec![String::new(); g.grade[0].len()];
            nova[0] = format!("Categoria {n}");
            g.grade.push(nova);
        }
        Msg::LinhaMenos if g.grade.len() > 2 => {
            g.grade.pop();
        }
        Msg::SerieMais if g.grade[0].len() < 7 => {
            let n = g.grade[0].len();
            for (i, l) in g.grade.iter_mut().enumerate() {
                l.push(if i == 0 {
                    format!("Série {n}")
                } else {
                    "0".into()
                });
            }
        }
        Msg::SerieMenos if g.grade[0].len() > 2 => {
            for l in g.grade.iter_mut() {
                l.pop();
            }
        }
        _ => return,
    }
    let _ = sessao.grafico_definir(id, g);
}

pub fn view<'a>(sessao: &Sessao, id: Option<u64>) -> Element<'a, Msg> {
    let Some(g) = id.and_then(|i| sessao.grafico_dados(i)) else {
        return container(
            text("Selecione um gráfico no slide e dê dois cliques nele.")
                .size(14)
                .color(COR_TEXTO_MUTED()),
        )
        .padding(30)
        .into();
    };
    let tipo = pick_list(&TIPOS[..], Some(rotulo_do_tipo(&g.tipo)), |t| {
        Msg::Tipo(t.to_string())
    })
    .size(12)
    .padding([7, 10])
    .width(125)
    .style(crate::faixa_abas::estilo_menu_da_faixa).handle_color(COR_AZUL_ACENTO());
    let topo = container(
        column![
            row![
                feather_ui::icones::icone("chart-bar", 22.0),
                text("Dados do gráfico").size(19),
                Space::new(Length::FILL, Length::Shrink),
                text(format!(
                    "{} categorias · {} séries",
                    g.grade.len().saturating_sub(1),
                    g.grade[0].len().saturating_sub(1)
                ))
                .size(11)
                .color(COR_TEXTO_MUTED())
            ]
            .spacing(10)
            .align(Align::Center),
            row![
                column![
                    text("Tipo de gráfico").size(11).color(COR_TEXTO_MUTED()),
                    tipo
                ]
                .spacing(5),
                column![
                    text("Título").size(11).color(COR_TEXTO_MUTED()),
                    text_input("Título do gráfico", g.titulo.clone())
                        .on_input(Msg::Titulo)
                        .size(13)
                        .padding([7, 10])
                        .width(Length::FILL)
                        .style(estilo_entrada_suspensa)
                ]
                .spacing(5)
                .width(Length::FILL)
            ]
            .spacing(16)
            .align(Align::End),
        ]
        .spacing(18),
    )
    .padding(18)
    .width(Length::FILL)
    .style(estilo_painel_miniaturas);
    let mut grade = column![].spacing(0);
    for (r, linha) in g.grade.iter().enumerate() {
        let mut celulas = row![container(
            text(if r == 0 { "#".into() } else { r.to_string() })
                .size(11)
                .color(COR_TEXTO_MUTED())
        )
        .width(34)
        .padding([9, 8])]
        .spacing(4);
        for (c, valor) in linha.iter().enumerate() {
            if r == 0 && c == 0 {
                celulas = celulas.push(
                    container(text("Categoria").size(12))
                        .width(170)
                        .padding([9, 8]),
                );
            } else {
                celulas = celulas.push(
                    text_input(
                        if r == 0 {
                            "Série"
                        } else if c == 0 {
                            "Categoria"
                        } else {
                            "0"
                        },
                        valor.clone(),
                    )
                    .on_input(move |s| Msg::Celula(r, c, s))
                    .padding([7, 9])
                    .size(12)
                    .width(if c == 0 { 170 } else { 115 })
                    .style(estilo_entrada_suspensa),
                );
            }
        }
        grade = grade.push(
            container(celulas.align(Align::Center))
                .padding([5, 8])
                .style(move |_| Style {
                    background: Some(Background::Color(if r == 0 {
                        COR_AZUL_FUNDO()
                    } else if r % 2 == 0 {
                        COR_FUNDO_JANELA()
                    } else {
                        COR_PAINEL()
                    })),
                    ..Default::default()
                }),
        );
    }
    let tabela = container(
        scrollable(grade).scrollbar_width(10).scrollbar_track(COR_TRILHO_ROLAGEM()).scrollbar_color(COR_POLEGAR_ROLAGEM())
            .direction(Direction::Both)
            .height(Length::FILL),
    )
    .padding(8)
    .width(Length::FILL)
    .height(Length::FILL)
    .style(estilo_painel_miniaturas);
    let btn = |icone: &str, rotulo: &str, m: Msg, habilitado: bool| {
        button(
            row![
                if icone == "minus" {
                    crate::icones::icone_menos(14.0, "#5f6368")
                } else {
                    feather_ui::icones::icone(icone, 14.0)
                },
                text(rotulo.to_owned()).size(12)
            ]
            .spacing(6)
            .align(Align::Center),
        )
        .padding([7, 10])
        .style(estilo_botao_secundario)
        .on_press_maybe(habilitado.then_some(m))
    };
    let acoes = row![
        btn("plus", "Categoria", Msg::LinhaMais, true),
        btn("minus", "Categoria", Msg::LinhaMenos, g.grade.len() > 2),
        Space::new(10, Length::Shrink),
        btn("plus", "Série", Msg::SerieMais, g.grade[0].len() < 7),
        btn("minus", "Série", Msg::SerieMenos, g.grade[0].len() > 2)
    ]
    .spacing(6);
    let dica = if g.tipo == "pizza" {
        "A pizza usa a primeira série. Alterações aparecem no slide automaticamente."
    } else {
        "Use vírgula ou ponto decimal. Alterações aparecem no slide automaticamente."
    };
    container(
        column![
            topo,
            tabela,
            acoes,
            text(dica).size(11).color(COR_TEXTO_MUTED())
        ]
        .spacing(14),
    )
    .padding(16)
    .width(Length::FILL)
    .height(Length::FILL)
    .style(|_| Style {
        background: Some(Background::Color(COR_FUNDO_JANELA())),
        ..Default::default()
    })
    .into()
}
