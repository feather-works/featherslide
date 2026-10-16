//! Modelos de diagramas vetoriais do FeatherSlide (categorias 9.1).
//! Não são SmartArt OOXML: os nós são formas nativas e textos editáveis,
//! os conectores são linhas nativas, e o conjunto pode ser desagrupado.
//! Uma única edição insere todos os elementos e pode ser desfeita de uma vez.

use crate::modelo::{Elemento, Tipo};
use crate::sessao::{Mudanca, Sessao};

/// As categorias da galeria (do `feather-shapes`).
pub use feather_shapes::diagramas::Categoria;

/// Formas-protótipo sem ID: úteis também para desenhar a miniatura da galeria.
/// O modelo vem do `feather-shapes` no quadro de 720×360 e é escalado aqui.
/// A ordem é a ordem de desenho (conectores ao fundo, nós à frente).
pub fn modelo(categoria: Categoria, x: f32, y: f32, largura: f32, altura: f32) -> Vec<Elemento> {
    let mut elementos: Vec<Elemento> = feather_shapes::diagramas::modelo_base(categoria)
        .iter()
        .map(crate::formas_comuns::elemento_do_item)
        .collect();
    let fx = largura / 720.0;
    let fy = altura / 360.0;
    for e in &mut elementos {
        e.x = x + e.x * fx;
        e.y = y + e.y * fy;
        e.largura *= fx;
        e.altura *= fy;
        e.espessura *= fx.min(fy);
        e.estilo.tamanho *= fx.min(fy);
    }
    elementos
}

impl Sessao {
    /// Insere um modelo em grupo único, sem rasterizar, e registra um único Ctrl+Z.
    pub fn diagrama_inserir(&mut self, categoria: Categoria) -> Result<Mudanca, String> {
        self.editar(move |s| {
            let (sw, sh) = s.ap.tamanho();
            let escala = (sw * 0.77 / 720.0).min(sh * 0.70 / 360.0).min(1.0);
            let (w, h) = (720.0 * escala, 360.0 * escala);
            let (x, y) = ((sw - w) / 2.0, (sh - h) / 2.0);
            let grupo = s.slide().elementos.iter().map(|e| e.grupo).max().unwrap_or(0)
                .checked_add(1).ok_or("Limite de grupos no slide atingido.")?;
            let mut id_selecionado = None;
            for mut el in modelo(categoria, x, y, w, h) {
                el.id = s.ap.novo_id();
                el.grupo = grupo;
                if id_selecionado.is_none() && el.tipo != Tipo::Linha {
                    id_selecionado = Some(el.id);
                }
                s.slide_mut().elementos.push(el);
            }
            s.selecao = id_selecionado.into_iter().collect();
            Ok((vec![s.atual], false))
        })
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn todas_as_categorias_geram_diagramas_vetoriais_na_area_do_slide() {
        for cat in Categoria::TODAS {
            let elementos = modelo(cat, 120.0, 90.0, 720.0, 360.0);
            assert!(elementos.len() >= 3, "{:?} sem elementos suficientes", cat);
            assert!(elementos.iter().any(|e| !e.texto.is_empty()), "{cat:?} sem textos editáveis");
            assert!(elementos.iter().all(|e| e.x >= 119.5 && e.y >= 89.5
                && e.x + e.largura >= 119.0 && e.x + e.largura <= 841.0
                && e.y + e.altura >= 89.0 && e.y + e.altura <= 451.0), "{cat:?} fora do slide");
            assert!(elementos.iter().all(|e| e.tipo != Tipo::Imagem), "nunca criar imagem sem bytes");
        }
    }

    #[test]
    fn inserir_quatorze_categorias_preserva_ids_grupos_e_desfazer() {
        let mut s = Sessao::default();
        let iniciais = s.slide().elementos.clone();
        let proximo = s.apresentacao().proximo_id;
        for cat in Categoria::TODAS {
            s.diagrama_inserir(cat).unwrap();
        }
        let todos = &s.slide().elementos;
        assert!(todos.len() > iniciais.len() + 60);
        let novos = &todos[iniciais.len()..];
        let ids: std::collections::HashSet<_> = novos.iter().map(|e| e.id).collect();
        assert_eq!(ids.len(), novos.len());
        assert!(ids.iter().all(|id| *id >= proximo));
        let grupos: std::collections::HashSet<_> = novos.iter().map(|e| e.grupo).collect();
        assert_eq!(grupos.len(), Categoria::TODAS.len());
        s.desfazer().unwrap();
        let grupos_depois: std::collections::HashSet<_> = s.slide().elementos[iniciais.len()..].iter().map(|e| e.grupo).collect();
        assert_eq!(grupos_depois.len(), 13);
        for _ in 0..13 { s.desfazer().unwrap(); }
        assert_eq!(s.slide().elementos, iniciais);
        s.refazer().unwrap();
        assert!(s.slide().elementos.len() > iniciais.len());
    }

    #[test]
    fn proporcao_quatro_por_tres_tambem_cabe() {
        use crate::modelo::Proporcao;
        let mut s = Sessao::default();
        s.ap.proporcao = Proporcao::Padrao;
        s.diagrama_inserir(Categoria::Organogramas).unwrap();
        let (w,h) = s.ap.tamanho();
        assert!(s.slide().elementos.iter().filter(|e| e.grupo > 0)
            .all(|e| e.x >= 0.0 && e.y >= 0.0 && e.x + e.largura <= w && e.y + e.altura <= h));
    }
}
