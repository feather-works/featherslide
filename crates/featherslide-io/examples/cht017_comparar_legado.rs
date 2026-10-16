//! CHT-017: exportação lado a lado do mesmo gráfico pelo renderer legado e
//! pelo motor Charts. Saída: 12 arquivos (4 tipos × 3 exportadores).
use featherslide_core::{
    dados::{self, Grafico as GraficoLegado},
    modelo::{Elemento, Slide, Tipo},
    quadro::{self, Dicas},
};
use featherslide_io::{grafico_compartilhado::converter_elemento, pdf};
use std::{env, error::Error, fs, path::PathBuf};

fn grafico(tipo: &str) -> Elemento {
    let grade = vec![
        vec!["".into(), "Real".into(), "Meta".into()],
        vec!["Jan".into(), "10".into(), "5".into()],
        vec!["Fev".into(), "20".into(), "12".into()],
        vec!["Mar".into(), "70".into(), "32".into()],
    ];
    let g = GraficoLegado {
        tipo: tipo.into(), titulo: "Vendas 2026".into(), grade,
    };
    Elemento {
        id: 731, tipo: Tipo::Grafico, x: 40.0, y: 70.0,
        largura: 640.0, altura: 360.0,
        dados: dados::gravar_grafico(&g), ..Default::default()
    }
}
fn slide(e: Elemento) -> Slide {
    Slide {
        id: 1, layout: String::new(), fundo: "#ffffff".into(),
        fundo_config: Default::default(), ocultar_mestre: false,
        elementos: vec![e], notas: String::new(), oculto: false,
        transicao: Default::default(), secao: String::new(),
        comentarios: Vec::new(),
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let destino = PathBuf::from(env::args().nth(1)
        .unwrap_or_else(|| "/tmp/featheroffice-cht017".into()));
    fs::create_dir_all(&destino)?;
    for tipo in ["colunas", "barras", "linhas", "pizza"] {
        let elemento = grafico(tipo);
        let original = elemento.dados.clone();
        let adaptado = converter_elemento(&elemento)
            .map_err(|e| format!("conversao {tipo}: {e:?}"))?;
        let quadro = quadro::quadro(&slide(elemento.clone()), (960., 540.),
            1, &Dicas::default(), false);
        let antigo_pdf = pdf::exportar("Vendas 2026", &[quadro], &|_| None);
        let novo_pdf = adaptado.exportar_pdf()
            .map_err(|e| format!("pdf {tipo}: {e:?}"))?;
        let novo_svg = adaptado.exportar_svg()
            .map_err(|e| format!("svg {tipo}: {e:?}"))?;
        assert!(antigo_pdf.starts_with(b"%PDF-"));
        assert!(novo_pdf.starts_with(b"%PDF-"));
        assert!(novo_svg.contains("<path"));
        assert_eq!(original, elemento.dados);
        fs::write(destino.join(format!("{tipo}-legado.pdf")), antigo_pdf)?;
        fs::write(destino.join(format!("{tipo}-comum.pdf")), novo_pdf)?;
        fs::write(destino.join(format!("{tipo}-comum.svg")), novo_svg)?;
        let maximo = adaptado.grafico.series.iter()
            .flat_map(|s| s.valores.iter().copied()).fold(0.0_f64, f64::max);
        let soma: f64 = adaptado.grafico.series.iter().flat_map(|s| &s.valores).sum();
        println!("OK {tipo}: categorias={}, series={}, total={soma}, maximo={maximo}",
            adaptado.grafico.categorias.len(), adaptado.grafico.series.len());
    }
    println!("CHT-017: 12 arquivos (legado.pdf, comum.pdf, comum.svg) em {}",
        destino.display());
    Ok(())
}
