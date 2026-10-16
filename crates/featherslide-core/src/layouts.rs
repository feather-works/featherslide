//! Os layouts de slide, como no PowerPoint e no Google Apresentações: cada um cria as
//! caixas vazias (com a dica "Clique para adicionar título"…) nas posições certas para
//! o tamanho do slide. As posições são frações do slide, para valer no 16:9 e no 4:3.

use crate::modelo::{Alinhamento, Elemento, EstiloDoTexto, Papel, Tipo, Vertical};

/// Os layouts, na ordem do menu.
pub const LAYOUTS: [&str; 14] = [
    "titulo",
    "titulo_conteudo",
    "secao",
    "dois_conteudos",
    "comparacao",
    "tres_colunas",
    "quatro_quadrantes",
    "titulo_lateral",
    "numeros",
    "linha_do_tempo",
    "citacao",
    "somente_conteudo",
    "somente_titulo",
    "em_branco",
];

/// Uma caixa do layout: papel e retângulo em frações do slide (x, y, largura, altura).
struct Caixa {
    papel: Papel,
    rect: [f32; 4],
}

const fn caixa(papel: Papel, rect: [f32; 4]) -> Caixa {
    Caixa { papel, rect }
}

/// As caixas de cada layout. `None` se o nome não existe.
fn caixas(layout: &str) -> Option<Vec<Caixa>> {
    Some(match layout {
        "titulo" => vec![caixa(Papel::Titulo, [0.08, 0.27, 0.84, 0.25]), caixa(Papel::Subtitulo, [0.08, 0.55, 0.84, 0.13])],
        "titulo_conteudo" => vec![caixa(Papel::Titulo, [0.06, 0.06, 0.88, 0.16]), caixa(Papel::Corpo, [0.06, 0.26, 0.88, 0.66])],
        "secao" => vec![caixa(Papel::Titulo, [0.08, 0.36, 0.84, 0.18]), caixa(Papel::Subtitulo, [0.08, 0.56, 0.84, 0.1])],
        "dois_conteudos" => vec![
            caixa(Papel::Titulo, [0.06, 0.06, 0.88, 0.16]),
            caixa(Papel::Corpo, [0.06, 0.26, 0.43, 0.66]),
            caixa(Papel::Corpo, [0.51, 0.26, 0.43, 0.66]),
        ],
        "comparacao" => vec![
            caixa(Papel::Titulo, [0.06, 0.06, 0.88, 0.16]),
            caixa(Papel::Subtitulo, [0.06, 0.26, 0.43, 0.09]),
            caixa(Papel::Corpo, [0.06, 0.36, 0.43, 0.56]),
            caixa(Papel::Subtitulo, [0.51, 0.26, 0.43, 0.09]),
            caixa(Papel::Corpo, [0.51, 0.36, 0.43, 0.56]),
        ],
        "tres_colunas" => vec![
            caixa(Papel::Titulo, [0.06, 0.06, 0.88, 0.16]),
            caixa(Papel::Corpo, [0.06, 0.26, 0.28, 0.66]),
            caixa(Papel::Corpo, [0.36, 0.26, 0.28, 0.66]),
            caixa(Papel::Corpo, [0.66, 0.26, 0.28, 0.66]),
        ],
        "quatro_quadrantes" => vec![
            caixa(Papel::Titulo, [0.06, 0.06, 0.88, 0.16]),
            caixa(Papel::Corpo, [0.06, 0.26, 0.43, 0.31]),
            caixa(Papel::Corpo, [0.51, 0.26, 0.43, 0.31]),
            caixa(Papel::Corpo, [0.06, 0.61, 0.43, 0.31]),
            caixa(Papel::Corpo, [0.51, 0.61, 0.43, 0.31]),
        ],
        "titulo_lateral" => vec![caixa(Papel::Titulo, [0.06, 0.1, 0.32, 0.8]), caixa(Papel::Corpo, [0.44, 0.1, 0.5, 0.8])],
        "numeros" => vec![
            caixa(Papel::Titulo, [0.06, 0.06, 0.88, 0.16]),
            caixa(Papel::Corpo, [0.06, 0.3, 0.26, 0.2]),
            caixa(Papel::Subtitulo, [0.06, 0.52, 0.26, 0.16]),
            caixa(Papel::Corpo, [0.37, 0.3, 0.26, 0.2]),
            caixa(Papel::Subtitulo, [0.37, 0.52, 0.26, 0.16]),
            caixa(Papel::Corpo, [0.68, 0.3, 0.26, 0.2]),
            caixa(Papel::Subtitulo, [0.68, 0.52, 0.26, 0.16]),
        ],
        "linha_do_tempo" => vec![
            caixa(Papel::Titulo, [0.06, 0.06, 0.88, 0.16]),
            caixa(Papel::Subtitulo, [0.06, 0.3, 0.2, 0.09]),
            caixa(Papel::Corpo, [0.06, 0.42, 0.2, 0.46]),
            caixa(Papel::Subtitulo, [0.285, 0.3, 0.2, 0.09]),
            caixa(Papel::Corpo, [0.285, 0.42, 0.2, 0.46]),
            caixa(Papel::Subtitulo, [0.51, 0.3, 0.2, 0.09]),
            caixa(Papel::Corpo, [0.51, 0.42, 0.2, 0.46]),
            caixa(Papel::Subtitulo, [0.735, 0.3, 0.2, 0.09]),
            caixa(Papel::Corpo, [0.735, 0.42, 0.2, 0.46]),
        ],
        "citacao" => vec![caixa(Papel::Corpo, [0.12, 0.24, 0.76, 0.38]), caixa(Papel::Subtitulo, [0.12, 0.66, 0.76, 0.1])],
        "somente_conteudo" => vec![caixa(Papel::Corpo, [0.06, 0.08, 0.88, 0.84])],
        "somente_titulo" => vec![caixa(Papel::Titulo, [0.06, 0.06, 0.88, 0.16])],
        "em_branco" => vec![],
        _ => return None,
    })
}

/// O estilo do texto de cada papel, no layout dado.
fn estilo_do_papel(papel: Papel, layout: &str) -> EstiloDoTexto {
    let base = EstiloDoTexto::default();
    match (papel, layout) {
        (Papel::Titulo, "titulo" | "secao") => EstiloDoTexto { tamanho: 40.0, negrito: true, alinhamento: Alinhamento::Centro, vertical: Vertical::Base, ..base },
        (Papel::Titulo, "titulo_lateral") => EstiloDoTexto { tamanho: 36.0, negrito: true, vertical: Vertical::Meio, ..base },
        (Papel::Titulo, _) => EstiloDoTexto { tamanho: 30.0, negrito: true, vertical: Vertical::Meio, ..base },
        (Papel::Subtitulo, "comparacao") => EstiloDoTexto { tamanho: 22.0, negrito: true, vertical: Vertical::Base, ..base },
        (Papel::Subtitulo, "linha_do_tempo") => EstiloDoTexto { tamanho: 20.0, negrito: true, alinhamento: Alinhamento::Centro, vertical: Vertical::Base, ..base },
        (Papel::Corpo, "citacao") => EstiloDoTexto { tamanho: 34.0, italico: true, alinhamento: Alinhamento::Centro, vertical: Vertical::Meio, ..base },
        (Papel::Corpo, "numeros") => EstiloDoTexto { tamanho: 56.0, negrito: true, alinhamento: Alinhamento::Centro, vertical: Vertical::Base, ..base },
        (Papel::Subtitulo, _) => EstiloDoTexto { tamanho: 20.0, cor: "#5f6368".into(), alinhamento: Alinhamento::Centro, ..base },
        _ => base,
    }
}

/// Os elementos que o layout cria num slide de `largura` × `altura`, com ids de `novo_id`.
pub fn elementos_do_layout(layout: &str, largura: f32, altura: f32, novo_id: &mut dyn FnMut() -> u64) -> Option<Vec<Elemento>> {
    let lista = caixas(layout)?;
    Some(
        lista
            .into_iter()
            .map(|c| Elemento {
                id: novo_id(),
                tipo: Tipo::Texto,
                papel: c.papel,
                x: c.rect[0] * largura,
                y: c.rect[1] * altura,
                largura: c.rect[2] * largura,
                altura: c.rect[3] * altura,
                estilo: estilo_do_papel(c.papel, layout),
                ..Elemento::default()
            })
            .collect(),
    )
}
