//! FTS-0103: escolher quais slides reutilizar de um arquivo já salvo.
//! Abre .fsdz, .pptx ou .odp sem alterar o documento até o usuário confirmar.

use std::collections::BTreeSet;
use featherslide_core::modelo::{Apresentacao, Papel};
use featherslide_core::EstiloColagem;
use pelicano::*;
use pelicano::column;
use crate::app::Mensagem;
use crate::tema::*;

#[derive(Debug, Clone)]
pub struct Estado {
    pub apresentacao: Apresentacao,
    pub caminho: std::path::PathBuf,
    pub selecionados: BTreeSet<usize>,
}

impl Estado {
    pub fn novo(apresentacao: Apresentacao, caminho: std::path::PathBuf) -> Self {
        let selecionados = (0..apresentacao.slides.len()).collect();
        Self { apresentacao, caminho, selecionados }
    }

    pub fn titulo(&self, indice: usize) -> String {
        self.apresentacao.slides.get(indice).and_then(|s| {
            s.elementos.iter().find(|e| e.papel == Papel::Titulo && !e.texto.trim().is_empty())
                .or_else(|| s.elementos.iter().find(|e| !e.texto.trim().is_empty()))
                .map(|e| e.texto.lines().next().unwrap_or_default().to_string())
        }).filter(|s| !s.trim().is_empty()).unwrap_or_else(|| "Sem título".into())
    }
}

pub fn sobrepor<'a>(base: Element<'a, Mensagem>, estado: &'a Estado, largura_janela: f32) -> Element<'a, Mensagem> {
    let fundo = mouse_area(Space::new(Length::FILL, Length::FILL))
        .on_press(Mensagem::ReutilizarFechar);
    let nome = estado.caminho.file_name().unwrap_or_default().to_string_lossy();
    let mut lista = column![].spacing(4);
    for (i, slide) in estado.apresentacao.slides.iter().enumerate() {
        let marcado = estado.selecionados.contains(&i);
        let titulo = estado.titulo(i);
        let rotulo = format!("{}   {}. {}{}", if marcado { "☑" } else { "☐" },
            i + 1, titulo.chars().take(55).collect::<String>(),
            if slide.oculto { "   [oculto]" } else { "" });
        lista = lista.push(
            button(text(rotulo).size(12))
                .padding([7, 11]).width(Length::FILL)
                .style(estilo_botao_pequeno(marcado))
                .on_press(Mensagem::ReutilizarAlternar(i))
        );
    }

    let botoes = row![
        button(text("Selecionar todos").size(12)).padding([6, 10])
            .style(estilo_botao_secundario).on_press(Mensagem::ReutilizarTodos),
        Space::new(Length::FILL, Length::Shrink),
        button(text("Cancelar").size(12)).padding([6, 10])
            .style(estilo_botao_secundario).on_press(Mensagem::ReutilizarFechar),
        button(text("Manter original").size(12)).padding([6, 10])
            .style(estilo_botao_primario)
            .on_press_maybe((!estado.selecionados.is_empty())
                .then_some(Mensagem::ReutilizarConfirmar(EstiloColagem::ManterOrigem))),
        button(text("Adaptar tema").size(12)).padding([6, 10])
            .style(estilo_botao_secundario)
            .on_press_maybe((!estado.selecionados.is_empty())
                .then_some(Mensagem::ReutilizarConfirmar(EstiloColagem::AdaptarAoTema))),
    ].spacing(7).align(Align::Center);

    let painel = container(column![
        text("Reutilizar slides").size(18),
        text(format!("Arquivo: {} · {} de {} slides selecionados",
            nome, estado.selecionados.len(), estado.apresentacao.slides.len()))
            .size(12).color(COR_TEXTO_MUTED()),
        scrollable(lista).scrollbar_width(10).scrollbar_track(COR_TRILHO_ROLAGEM()).scrollbar_color(COR_POLEGAR_ROLAGEM()).height(Length::Fixed(425.0)),
        botoes,
    ].spacing(12))
        .width(Length::Fixed(650.0))
        .padding(16)
        .style(|_| Style {
            background: Some(Background::Color(COR_PAINEL())),
            text_color: Some(COR_TEXTO()),
            border: Border { color: COR_BORDA(), width: 1.0, radius: [8.0; 4] },
            shadow: Some(Shadow { color: Color::rgb8(0, 0, 0).alpha(0.25),
                offset: Point::new(0.0,6.0), blur: 18.0 }),
            ..Default::default()
        });
    let x = ((largura_janela - 650.0) / 2.0).max(8.0);
    let pop = container(painel).padding(Padding {
        top: 85.0, left: x, right: 0.0, bottom: 0.0
    });
    stack![base, fundo, pop].into()
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn reuso_seleciona_todos_mas_permite_filtro() {
        let mut s = featherslide_core::sessao::Sessao::default();
        s.slide_novo("titulo").unwrap();
        let mut e = Estado::novo(s.apresentacao().clone(), "apresentacao.fsdz".into());
        assert_eq!(e.selecionados.len(), 2);
        assert_eq!(e.titulo(0), "Sem título");
        e.selecionados.remove(&0);
        assert_eq!(e.selecionados.into_iter().collect::<Vec<_>>(), vec![1]);
    }
}
