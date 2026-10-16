//! Geometria SVG normalizada na caixa do elemento, preservando curvas e subcaminhos abertos.
use crate::{
    dados::Primitiva,
    modelo::{Elemento, Tipo},
    sessao::{Mudanca, Sessao},
};

/// Um comando do caminho em coordenadas do slide (do `feather-shapes`).
pub use feather_shapes::caminho::Segmento;

pub fn eh_svg(e: &Elemento) -> bool {
    e.tipo == Tipo::Caminho && feather_shapes::caminho::eh_svg1(&e.dados)
}

/// A codificação em texto usa apenas números finitos e comandos normalizados.
pub fn decodificar(e: &Elemento) -> Result<(bool, Vec<Segmento>), String> {
    feather_shapes::caminho::decodificar_svg1(&e.dados, [e.x, e.y, e.largura, e.altura])
}

pub fn primitivas(e: &Elemento) -> Vec<Primitiva> {
    let Ok((par, segmentos)) = decodificar(e) else {
        return Vec::new();
    };
    vec![Primitiva::Vetor {
        segmentos,
        par,
        preenchimento: e.preenchimento.clone(),
        contorno: e.contorno.clone(),
        espessura: e.espessura,
    }]
}

impl Sessao {
    /// Insere o desenho como um grupo. A conversão libera suas peças para edição individual.
    pub fn svg_inserir(
        &mut self,
        elementos: Vec<Elemento>,
        tamanho: [f32; 2],
    ) -> Result<Mudanca, String> {
        if elementos.is_empty()
            || elementos.len() > 10_000
            || tamanho.iter().any(|v| !v.is_finite() || *v <= 0.0)
        {
            return Err("SVG sem geometria válida.".into());
        }
        for e in &elementos {
            decodificar(e)?;
        }
        self.editar(move |s| {
            let (w, h) = s.ap.tamanho();
            let escala = (w * 0.7 / tamanho[0]).min(h * 0.7 / tamanho[1]).min(1.0);
            let origem = [
                (w - tamanho[0] * escala) / 2.0,
                (h - tamanho[1] * escala) / 2.0,
            ];
            let grupo = s
                .slide()
                .elementos
                .iter()
                .map(|e| e.grupo)
                .max()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or("Limite de grupos atingido.")?;
            let mut ids = Vec::new();
            for mut e in elementos {
                e.id = s.ap.novo_id();
                e.grupo = grupo;
                e.x = origem[0] + e.x * escala;
                e.y = origem[1] + e.y * escala;
                e.largura *= escala;
                e.altura *= escala;
                e.espessura *= escala;
                ids.push(e.id);
                s.slide_mut().elementos.push(e);
            }
            s.selecao = ids;
            Ok((vec![s.atual], false))
        })
    }

    pub fn svg_converter(&mut self) -> Result<Mudanca, String> {
        if !self
            .slide()
            .elementos
            .iter()
            .any(|e| self.selecao.contains(&e.id) && eh_svg(e))
        {
            return Err("Selecione um SVG importado.".into());
        }
        self.editar(|s| {
            for e in s.selecionados_mut().filter(|e| eh_svg(e)) {
                e.grupo = 0;
            }
            Ok((vec![s.atual], false))
        })
    }
}
