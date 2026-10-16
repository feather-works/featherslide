//! Texto artístico: conteúdo continua editável; a interface resolve os contornos da fonte.
use crate::modelo::{Alinhamento, Elemento, EstiloDoTexto, Sombra, Tipo, Vertical};
use crate::sessao::{Mudanca, Sessao};

/// A transformação e a deformação dos contornos vêm do `feather-shapes`.
pub use feather_shapes::wordart::{transformar, Transformacao};

/// Copia o estilo pronto `indice` da galeria comum (`feather_shapes::wordart::ESTILOS`) para o
/// elemento; fora do intervalo vira o padrão (índice 0).
pub fn aplicar_estilo(e: &mut Elemento, indice: usize) {
    let estilo = feather_shapes::wordart::estilo(indice);
    e.estilo.cor = estilo.preenchimento.into();
    e.contorno = estilo.contorno.into();
    e.espessura = estilo.espessura;
    e.sombra = match estilo.sombra {
        Some(s) => Sombra { cor: s.cor.into(), dx: s.dx, dy: s.dy, desfoque: 0.0 },
        None => Sombra { cor: String::new(), dx: 3.0, dy: 3.0, desfoque: 0.0 },
    };
}

impl Sessao {
    /// Um único passo de desfazer inclui inserção, texto e estilo inicial.
    pub fn wordart_inserir(&mut self) -> Result<Mudanca, String> {
        self.editar(|s| {
            let (w, h) = s.ap.tamanho();
            let mut e = Elemento {
                id: s.ap.novo_id(), tipo: Tipo::WordArt, x: (w - 480.0) / 2.0, y: (h - 120.0) / 2.0,
                largura: 480.0, altura: 120.0, texto: "Seu texto".into(), dados: "reto".into(), ajustes: vec![0.5],
                estilo: EstiloDoTexto { tamanho: 54.0, negrito: true, alinhamento: Alinhamento::Centro, vertical: Vertical::Meio, ..Default::default() },
                ..Default::default()
            };
            aplicar_estilo(&mut e, 0);
            s.selecao = vec![e.id];
            s.slide_mut().elementos.push(e);
            Ok((vec![s.atual], false))
        })
    }

    pub fn wordart_estilo(&mut self, indice: usize) -> Result<Mudanca, String> {
        self.editar_wordart(|e| aplicar_estilo(e, indice))
    }

    pub fn wordart_transformacao(&mut self, modo: Transformacao, intensidade: f32) -> Result<Mudanca, String> {
        if !intensidade.is_finite() { return Err("A intensidade precisa ser um número finito.".into()); }
        self.editar_wordart(|e| { e.dados = modo.nome().into(); e.ajustes = vec![intensidade.clamp(-1.0, 1.0)]; })
    }

    pub fn wordart_sombra(&mut self, ligada: bool) -> Result<Mudanca, String> {
        self.editar_wordart(|e| e.sombra = if ligada {
            let p = feather_shapes::wordart::SOMBRA_PADRAO;
            Sombra { cor: p.cor.into(), dx: p.dx, dy: p.dy, desfoque: 0.0 }
        } else { Sombra::default() })
    }

    fn editar_wordart(&mut self, f: impl Fn(&mut Elemento)) -> Result<Mudanca, String> {
        let ids = self.selecao.clone();
        if !self.slide().elementos.iter().any(|e| ids.contains(&e.id) && e.tipo == Tipo::WordArt) {
            return Err("Selecione um WordArt.".into());
        }
        self.editar(|s| {
            for e in &mut s.slide_mut().elementos {
                if ids.contains(&e.id) && e.tipo == Tipo::WordArt { f(e); }
            }
            Ok((vec![s.atual], false))
        })
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn texto_estilo_transformacao_e_desfazer() {
        let mut s = Sessao::default();
        let antes = s.slide().elementos.len();
        s.wordart_inserir().unwrap();
        let id = s.estado().selecao[0];
        s.texto_definir(id, "Título artístico").unwrap();
        s.wordart_estilo(2).unwrap();
        s.wordart_transformacao(Transformacao::Arco, 0.7).unwrap();
        let e = s.slide().elementos.last().unwrap();
        assert_eq!(e.texto, "Título artístico");
        assert_eq!(e.dados, "arco");
        assert!(!e.sombra.cor.is_empty());
        s.desfazer().unwrap();
        assert_eq!(s.slide().elementos.last().unwrap().dados, "reto");
        s.refazer().unwrap();
        assert_eq!(s.slide().elementos.last().unwrap().dados, "arco");
        for _ in 0..4 { s.desfazer().unwrap(); }
        assert_eq!(s.slide().elementos.len(), antes);
    }
}
