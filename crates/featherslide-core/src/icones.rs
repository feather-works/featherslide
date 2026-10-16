//! A biblioteca de ícones (Inserir → Ícones): os ícones de contorno do **Tabler Icons**
//! (https://github.com/tabler/tabler-icons, licença MIT), 5 mil e tantos, embutidos no
//! `feather-shapes` (dados e aviso de licença em `dados/` de lá). Um ícone é um elemento `Tipo::Icone` com o nome em
//! `dados`; a cor vem do `preenchimento`/`contorno` e a espessura do traço, do `espessura`.
//! Sai como primitivas (traços redondos) que a tela e o PDF só pintam: sem arquivo de imagem,
//! sem rede, e recolorir e redimensionar não perdem qualidade.
//!
//! Para atualizar o conjunto: `tools/gerar_tabler.py` do `feather-shapes`.

use crate::dados::Primitiva;
use crate::modelo::{Elemento, Tipo};
use crate::sessao::{Mudanca, Sessao};

/// O catálogo e a busca vêm do `feather-shapes`.
pub use feather_shapes::icones::{buscar, buscar_por_id, categorias, da_categoria, existe, todos, Icone, LICENCA_TABLER};

/// O desenho do ícone `e` (nome em `dados`) dentro da caixa do elemento, ajustado ao menor lado e
/// centrado, em pontos do slide. A cor é a do contorno (ou do preenchimento) e a espessura do
/// traço é a do contorno: 1 = o traço original do Tabler (2 em 24).
pub fn primitivas(e: &Elemento) -> Vec<Primitiva> {
    let Some(ic) = buscar_por_id(&e.dados) else { return Vec::new() };
    let cor = [&e.contorno, &e.preenchimento].into_iter().find(|c| !c.is_empty()).cloned().unwrap_or_else(|| "#202124".into());
    let (linhas, escala) = feather_shapes::icones::tracos(ic, e.x, e.y, e.largura, e.altura);
    let espessura = 2.0 * escala * if e.espessura > 0.0 { e.espessura } else { 1.0 };
    linhas
        .into_iter()
        .map(|pontos| Primitiva::Poli { pontos, cor: cor.clone(), espessura })
        .collect()
}

impl Sessao {
    /// Troca o desenho dos ícones selecionados, preservando tamanho, cor e posição.
    pub fn icone_substituir(&mut self, id: &str) -> Result<Mudanca, String> {
        let nome = buscar_por_id(id).ok_or_else(|| format!("O ícone \"{id}\" não existe."))?.id;
        if !self.slide().elementos.iter().any(|e| self.selecao.contains(&e.id) && e.tipo == Tipo::Icone) {
            return Err("Selecione um ícone primeiro.".into());
        }
        self.editar(|s| {
            for e in s.selecionados_mut().filter(|e| e.tipo == Tipo::Icone) { e.dados = nome.to_string(); }
            Ok((vec![s.atual], false))
        })
    }

    /// Insere o ícone `id` no meio do slide atual, na cor padrão, e o seleciona. A cor se troca
    /// depois pelo Preenchimento (ou Contorno), a espessura pelo Contorno e o tamanho pelas alças.
    pub fn icone_inserir(&mut self, id: &str) -> Result<Mudanca, String> {
        let nome = buscar_por_id(id).ok_or_else(|| format!("O ícone \"{id}\" não existe."))?.id;
        self.editar(|s| {
            let (sl, sa) = s.ap.tamanho();
            let lado = 96.0_f32;
            let novo = s.ap.novo_id();
            let el = Elemento {
                id: novo,
                tipo: Tipo::Icone,
                x: ((sl - lado) / 2.0).round(),
                y: ((sa - lado) / 2.0).round(),
                largura: lado,
                altura: lado,
                preenchimento: "#1a73e8".into(),
                contorno: "#1a73e8".into(),
                espessura: 1.0,
                dados: nome.to_string(),
                ..Elemento::default()
            };
            s.slide_mut().elementos.push(el);
            s.selecao = vec![novo];
            Ok((vec![s.atual], false))
        })
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn todos_desenham_dentro_do_quadrado() {
        assert!(todos().len() > 5000);
        for ic in todos() {
            let e = Elemento { tipo: Tipo::Icone, dados: ic.id.into(), largura: 24.0, altura: 24.0, ..Elemento::default() };
            let p = primitivas(&e);
            assert!(!p.is_empty(), "{} sem desenho", ic.id);
            for prim in p {
                let Primitiva::Poli { pontos, .. } = prim else { panic!("só traços") };
                for [x, y] in pontos {
                    assert!(x.is_finite() && y.is_finite() && (-1.5..=25.5).contains(&x) && (-1.5..=25.5).contains(&y), "{} sai do quadrado: {x},{y}", ic.id);
                }
            }
        }
    }

    #[test]
    fn inserir_recolorir_e_desfazer() {
        let mut s = Sessao::default();
        assert!(s.icone_inserir("nao_existe").is_err());
        s.icone_inserir("heart").unwrap();
        let e = s.slide().elementos.last().unwrap().clone();
        assert_eq!((e.tipo, e.dados.as_str()), (Tipo::Icone, "heart"));
        let q = s.quadro(0, None, false).unwrap().unwrap();
        let d = q.elementos.iter().find(|d| d.id == e.id).unwrap();
        assert!(!d.primitivas.is_empty());
        assert!(d.preenchimento.is_empty() && d.contorno.is_empty(), "a caixa do ícone não pode ter fundo nem moldura");
        // Pintar: tanto o Preenchimento quanto o Contorno mudam a cor do ícone, e a espessura vem do Contorno.
        s.formatar(crate::Formatacao::Preenchimento("#ff0000".into())).unwrap();
        let cor = |s: &Sessao| match s.quadro(0, None, false).unwrap().unwrap().elementos.iter().find(|d| d.id == e.id).unwrap().primitivas[0].clone() {
            Primitiva::Poli { cor, espessura, .. } => (cor, espessura),
            _ => panic!(),
        };
        assert_eq!(cor(&s).0, "#ff0000");
        s.formatar(crate::Formatacao::Contorno("#00aa00".into())).unwrap();
        assert_eq!(cor(&s).0, "#00aa00");
        let fina = cor(&s).1;
        s.formatar(crate::Formatacao::Espessura(2.0)).unwrap();
        assert!((cor(&s).1 - 2.0 * fina).abs() < 1e-4, "dobrar a espessura dobra o traço");
        s.desfazer().unwrap();
        s.desfazer().unwrap();
        s.desfazer().unwrap();
        s.desfazer().unwrap();
        assert!(s.slide().elementos.iter().all(|x| x.tipo != Tipo::Icone));
    }
}
