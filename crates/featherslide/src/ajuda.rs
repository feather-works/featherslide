//! A Ajuda (F1): "Como usar" e "Atalhos", com busca e navegação anterior/próximo. Os textos
//! vêm da seção `ajuda` do arquivo do idioma da interface (`dados/<idioma>.json`).

use std::sync::OnceLock;
use crate::localidade::{self, tr};

pub use feather_ui::ajuda::Evento as Msg;
use feather_ui::ajuda::{Catalogo, Linha, Secao, Topico};
use pelicano::Element;
use serde_json::Value;

/// O texto em `caminho` (`ajuda.rotulos.titulo`) no idioma da interface, ou o próprio caminho.
pub fn t(caminho: &str) -> String {
    localidade::no_caminho(caminho).and_then(Value::as_str).unwrap_or(caminho).to_string()
}

const TOPICOS: [&str; 8] = [
    "comecar",
    "slides",
    "elementos",
    "texto",
    "formas",
    "apresentar",
    "arquivos",
    "configuracoes",
];

const GRUPOS: [(&str, &[(&str, &str)]); 5] = [
    (
        "slides",
        &[("novoSlide", "Ctrl+M"), ("irSlide", "PgUp / PgDn")],
    ),
    (
        "edicao",
        &[
            ("desfazer", "Ctrl+Z"),
            ("refazer", "Ctrl+Y / Ctrl+Shift+Z"),
            ("selTudo", "Ctrl+A"),
            ("area", "Ctrl+C / Ctrl+X / Ctrl+V"),
            ("duplicar", "Ctrl+D"),
            ("excluir", "Delete / Backspace"),
            ("mover", "Setas do teclado"),
            ("moverMais", "Shift + setas"),
            ("editar", "Enter"),
            ("esc", "Esc"),
        ],
    ),
    (
        "formato",
        &[
            ("negrito", "Ctrl+B"),
            ("italico", "Ctrl+I"),
            ("sublinhado", "Ctrl+U"),
            ("alinhar", "Ctrl+L / Ctrl+E / Ctrl+R"),
        ],
    ),
    (
        "arquivo",
        &[
            ("novo", "Ctrl+N"),
            ("abrir", "Ctrl+O"),
            ("salvar", "Ctrl+S"),
            ("salvarComo", "Ctrl+Shift+S"),
        ],
    ),
    (
        "janela",
        &[
            ("apresentar", "F5"),
            ("apresentarAtual", "Shift+F5"),
            ("zoom", "Ctrl++ / Ctrl+-"),
            ("zoomAjustar", "Ctrl+0"),
            ("faixa", "Ctrl+F1"),
            ("ajuda", "F1"),
            ("monitor", "F12"),
        ],
    ),
];

fn corpo_do_topico(id: &str) -> Vec<String> {
    localidade::no_caminho(&format!("ajuda.comoUsar.{id}.corpo"))
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

/// O mesmo componente visual do F1 do FeatherSheet, com conteúdo do Slide (um por idioma).
pub fn modelo() -> &'static Catalogo {
    static CATALOGOS: [OnceLock<Catalogo>; 3] = [OnceLock::new(), OnceLock::new(), OnceLock::new()];
    let i = localidade::IDIOMAS.iter().position(|(c, _)| *c == localidade::atual()).unwrap_or(0);
    CATALOGOS[i].get_or_init(|| {
        let uso = TOPICOS
            .iter()
            .map(|id| Topico {
                id: (*id).into(),
                titulo: t(&format!("ajuda.comoUsar.{id}.titulo")),
                linhas: corpo_do_topico(id)
                    .into_iter()
                    .map(|l| match l.strip_prefix("• ") {
                        Some(item) => Linha::Marcador(item.to_owned()),
                        None => Linha::Texto(l),
                    })
                    .collect(),
            })
            .collect();
        let atalhos = GRUPOS
            .iter()
            .map(|(id, atalhos)| Topico {
                id: (*id).into(),
                titulo: t(&format!("ajuda.rotulos.g_{id}")),
                linhas: atalhos
                    .iter()
                    .map(|(id, tecla)| Linha::Atalho {
                        descricao: t(&format!("ajuda.teclas.{id}")),
                        // Só as teclas descritas em palavras ("Setas do teclado") se traduzem; "Ctrl+Z" fica.
                        tecla: if tecla.to_lowercase().contains("seta") { tr(tecla) } else { tecla }.into(),
                    })
                    .collect(),
            })
            .collect();
        Catalogo {
            titulo: t("ajuda.rotulos.titulo"),
            busca_rotulo: t("ajuda.rotulos.busca"),
            nada_encontrado: t("ajuda.rotulos.semResultado"),
            anterior: t("ajuda.rotulos.anterior"),
            proximo: t("ajuda.rotulos.proximo"),
            nesta_secao: t("ajuda.rotulos.nestaSecao"),
            secoes: vec![
                Secao {
                    id: "uso".into(),
                    titulo: t("ajuda.rotulos.uso"),
                    introducao: t("ajuda.introducoes.uso"),
                    topicos: uso,
                },
                Secao {
                    id: "atalhos".into(),
                    titulo: t("ajuda.rotulos.atalhos"),
                    introducao: t("ajuda.introducoes.atalhos"),
                    topicos: atalhos,
                },
            ],
        }
    })
}
#[derive(Default)]
pub struct Estado {
    pub navegacao: feather_ui::ajuda::Estado,
}
impl Estado {
    pub fn update(&mut self, m: Msg) {
        self.navegacao.atualizar(m, modelo());
    }
    pub fn view(&self) -> Element<'_, Msg> {
        modelo().view(&self.navegacao)
    }
}
#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn catalogo_preserva_topicos_e_atalhos_do_slide() {
        let c = modelo();
        assert_eq!(c.secoes.len(), 2);
        assert_eq!(c.secoes[0].topicos.len(), TOPICOS.len());
        assert_eq!(c.secoes[1].topicos.len(), GRUPOS.len());
        assert!(c
            .secoes
            .iter()
            .flat_map(|s| &s.topicos)
            .all(|t| !t.titulo.starts_with("ajuda.") && !t.linhas.is_empty()));
        assert!(c.quantidade_encontrada("Ctrl+Shift+S") > 0);
        assert!(c.quantidade_encontrada("apresentacao") > 0);
    }
    #[test]
    fn busca_e_navegacao_compartilhadas() {
        let mut e = Estado::default();
        e.update(Msg::IrTopico(0, 2));
        assert_eq!(e.navegacao.topico, Some(2));
        e.update(Msg::Proximo);
        assert_eq!(e.navegacao.topico, Some(3));
        e.update(Msg::Buscar("formas".into()));
        assert!(modelo().quantidade_encontrada(&e.navegacao.busca) > 0);
        e.update(Msg::IrSecao(1));
        assert!(e.navegacao.busca.is_empty());
        let _: Element<'_, Msg> = e.view();
    }
}
