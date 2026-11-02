//! Painel de rótulos ao lado do slide; aplicar registra uma única edição.
use crate::{app::Mensagem, localidade::texto as tr, tema::*};
use featherslide_core::editor_diagrama::{Arranjo, EdicaoDiagrama};
use pelicano::column;
use pelicano::*;

#[derive(Clone, Debug)]
pub enum Msg {
    Selecionar(usize),
    Texto(usize, String),
    Adicionar(Option<usize>, bool),
    Excluir(usize),
    Mover(usize, isize),
    Arranjo(Arranjo),
    Nivel(usize, bool),
    Aplicar,
    Fechar,
}
pub struct Estado {
    pub selecionado: Option<u64>,
    pub edicao: EdicaoDiagrama,
    pub aviso: String,
}
pub fn view(e: &Estado) -> Element<'_, Mensagem> {
    let mut campos = column![].spacing(3);
    for (i, no) in e.edicao.nos.iter().enumerate() {
        let campo = text_input(tr("diagrama_rotulo"), &no.texto)
            .id(format!("rotulo_diagrama_{i}"))
            .on_input(move |s| Msg::Texto(i, s))
            .padding([7, 8])
            .size(13);
        let mut acoes = row![
            button(text(tr("diagrama_irmao")).size(10))
                .on_press(Msg::Adicionar(Some(i), false))
                .style(estilo_botao_secundario),
            button(crate::icones::icone_subir(13.0, "#5f6368"))
                .on_press(Msg::Mover(i, -1))
                .style(estilo_botao_secundario),
            button(crate::icones::icone_descer(13.0, "#5f6368"))
                .on_press(Msg::Mover(i, 1))
                .style(estilo_botao_secundario),
            button(text(tr("diagrama_excluir")).size(10))
                .on_press_maybe((e.edicao.nos.len() > 1).then_some(Msg::Excluir(i)))
                .style(estilo_botao_secundario),
        ]
        .spacing(3);
        if e.edicao.arranjo == Arranjo::Organograma {
            acoes = acoes.push(
                button(text(tr("diagrama_filho")).size(10))
                    .on_press(Msg::Adicionar(Some(i), true))
                    .style(estilo_botao_secundario),
            );
        }
        let mut ajustes = row![].spacing(4);
        if e.edicao.arranjo == Arranjo::Organograma {
            ajustes = ajustes
                .push(
                    button(text(tr("diagrama_promover")).size(10))
                        .on_press(Msg::Nivel(i, false))
                        .style(estilo_botao_secundario),
                )
                .push(
                    button(text(tr("diagrama_rebaixar")).size(10))
                        .on_press(Msg::Nivel(i, true))
                        .style(estilo_botao_secundario),
                );
        }
        let ativo = e.selecionado == Some(no.id);
        let selecionar = button(text(format!("{}", i + 1)).size(11))
            .on_press(Msg::Selecionar(i))
            .padding([8, 6])
            .style(estilo_botao_pequeno(ativo));
        let mut item = column![row![selecionar, campo].spacing(5)].spacing(6);
        if ativo {
            item = item.push(acoes).push(ajustes);
        }
        campos = campos.push(
            row![
                Space::new((no.linha.min(6) * 20) as f32, 0),
                container(item)
                    .padding(6)
                    .width(Length::FILL)
                    .style(move |_| Style {
                        background: ativo.then_some(COR_AZUL_FUNDO().into()),
                        border: Border::rounded(5.0),
                        ..Default::default()
                    })
            ]
            .spacing(0),
        );
    }
    let mut layouts = column![].spacing(4);
    for pares in [
        [
            (Arranjo::Lista, "diagrama_lista"),
            (Arranjo::Processo, "diagrama_processo"),
        ],
        [
            (Arranjo::Ciclo, "diagrama_ciclo"),
            (Arranjo::Organograma, "diagrama_arvore"),
        ],
    ] {
        let mut linha = row![].spacing(5);
        for (arranjo, chave) in pares {
            linha = linha.push(
                button(text(tr(chave)).size(11))
                    .width(Length::FILL)
                    .padding([7, 8])
                    .on_press(Msg::Arranjo(arranjo))
                    .style(estilo_botao_pequeno(e.edicao.arranjo == arranjo)),
            );
        }
        layouts = layouts.push(linha);
    }
    let conteudo = column![
        text(tr("diagrama_titulo")).size(17),
        text(tr("diagrama_dica")).size(12),
        layouts,
        button(text(tr("diagrama_adicionar")).size(12))
            .style(estilo_botao_secundario)
            .on_press(Msg::Adicionar(None, false)),
        scrollable(campos).scrollbar_width(10).scrollbar_track(COR_TRILHO_ROLAGEM()).scrollbar_color(COR_POLEGAR_ROLAGEM()).height(Length::FILL),
        text(&e.aviso).size(12),
        row![
            button(text(tr("diagrama_cancelar")).size(12))
                .style(estilo_botao_secundario)
                .on_press(Msg::Fechar),
            button(text(tr("diagrama_aplicar")).size(12))
                .style(estilo_botao_primario)
                .on_press(Msg::Aplicar),
        ]
        .spacing(8),
    ]
    .spacing(12);
    let painel: Element<'_, Msg> = container(conteudo)
        .padding(16)
        .width(340)
        .height(Length::FILL)
        .style(estilo_painel_miniaturas)
        .into();
    painel.map(Mensagem::EditorDiagrama)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{app::FeatherSlideApp, faixa::FaixaMensagem};
    use featherslide_core::diagramas::Categoria;
    #[test]
    fn selecao_acompanha_item_movido_adicionado_e_excluido() {
        let mut app = FeatherSlideApp::default();
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::DiagramaInserir(
            Categoria::Organogramas,
        )));
        let _ = app.update(Mensagem::EditorDiagrama(Msg::Selecionar(2)));
        let id = app.editor_diagrama.as_ref().unwrap().selecionado;
        let _ = app.update(Mensagem::EditorDiagrama(Msg::Mover(2, -1)));
        assert_eq!(app.editor_diagrama.as_ref().unwrap().selecionado, id);
        let _ = app.update(Mensagem::EditorDiagrama(Msg::Adicionar(Some(1), true)));
        let e = app.editor_diagrama.as_ref().unwrap();
        let i = e
            .edicao
            .nos
            .iter()
            .position(|n| Some(n.id) == e.selecionado)
            .unwrap();
        assert_eq!(e.edicao.nos[i].texto, tr("diagrama_novo"));
        let _ = app.update(Mensagem::EditorDiagrama(Msg::Excluir(i)));
        let e = app.editor_diagrama.as_ref().unwrap();
        assert!(e.edicao.nos.iter().any(|n| Some(n.id) == e.selecionado));
    }
    #[test]
    fn adicionar_filho_pelo_painel_preserva_o_nivel_ao_aplicar() {
        let mut app = FeatherSlideApp::default();
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::DiagramaInserir(
            Categoria::Hierarquias,
        )));
        let e = &app.editor_diagrama.as_ref().unwrap().edicao;
        assert_eq!(
            e.nos
                .iter()
                .map(|n| (n.texto.as_str(), n.linha))
                .collect::<Vec<_>>(),
            [
                ("Nível 1", 0),
                ("Ramo A", 1),
                ("Item 1", 2),
                ("Item 2", 2),
                ("Ramo B", 1),
                ("Ramo C", 1),
                ("Item 3", 2),
                ("Item 4", 2)
            ]
        );
        let _ = app.update(Mensagem::EditorDiagrama(Msg::Adicionar(Some(4), true)));
        let e = &app.editor_diagrama.as_ref().unwrap().edicao;
        assert_eq!(e.nos[5].texto, tr("diagrama_novo"));
        assert_eq!(e.nos[5].linha, 2);
        let _ = app.update(Mensagem::EditorDiagrama(Msg::Texto(5, "Compras".into())));
        let _ = app.update(Mensagem::EditorDiagrama(Msg::Aplicar));
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::DiagramaEditar));
        let e = &app.editor_diagrama.as_ref().unwrap().edicao;
        assert_eq!(e.nos[5].texto, "Compras");
        assert_eq!(e.nos[5].linha, 2);
    }
    #[test]
    fn inserir_abre_painel_aplicar_desfaz_e_reabrir_carrega_textos() {
        let mut app = FeatherSlideApp::default();
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::DiagramaInserir(
            Categoria::Organogramas,
        )));
        let painel = app.editor_diagrama.as_ref().unwrap();
        assert_eq!(
            painel
                .edicao
                .nos
                .iter()
                .map(|n| n.texto.as_str())
                .collect::<Vec<_>>(),
            ["Diretoria", "Produto", "Engenharia", "Operações"]
        );
        let antes = app.sessao.apresentacao().slides.clone();
        let _ = app.update(Mensagem::EditorDiagrama(Msg::Texto(0, "Diretora".into())));
        let _ = app.update(Mensagem::EditorDiagrama(Msg::Texto(2, "Pesquisa".into())));
        assert_eq!(
            app.sessao.apresentacao().slides,
            antes,
            "Digitar apenas altera o rascunho."
        );
        let _ = app.update(Mensagem::EditorDiagrama(Msg::Aplicar));
        assert!(app.editor_diagrama.is_none());
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::DiagramaEditar));
        let painel = app.editor_diagrama.as_ref().unwrap();
        assert_eq!(painel.edicao.nos[0].texto, "Diretora");
        assert_eq!(painel.edicao.nos[2].texto, "Pesquisa");
        let _ = app.update(Mensagem::EditorDiagrama(Msg::Fechar));
        app.sessao.desfazer().unwrap();
        assert_eq!(app.sessao.apresentacao().slides, antes);
    }
    #[test]
    fn cancelar_preserva_o_documento() {
        let mut app = FeatherSlideApp::default();
        let _ = app.update(Mensagem::Faixa(FaixaMensagem::DiagramaInserir(
            Categoria::Organogramas,
        )));
        let antes = app.sessao.apresentacao().clone();
        let _ = app.update(Mensagem::EditorDiagrama(Msg::Texto(0, "Diretora".into())));
        let _ = app.update(Mensagem::EditorDiagrama(Msg::Fechar));
        assert!(app.editor_diagrama.is_none());
        assert_eq!(app.sessao.apresentacao(), &antes);
    }
}
