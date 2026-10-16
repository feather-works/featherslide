//! CHT-014: converte gráficos de Elemento::dados sem alterar o .fsdz.
//!
//! Exemplo de validação: gera 4 arquivos SVG + 4 PDF usando Charts comum.
use featherslide_core::{dados, modelo::{Elemento, Tipo}};
use featherslide_io::grafico_compartilhado::converter_elemento;
use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let destino = PathBuf::from(
        env::args().nth(1).unwrap_or_else(|| "/tmp/featheroffice-cht014".into())
    );
    fs::create_dir_all(&destino)?;
    for tipo in ["colunas", "barras", "linhas", "pizza"] {
        let dados_originais = dados::gravar_grafico(&dados::grafico_exemplo(tipo));
        let elemento = Elemento {
            id: 700,
            tipo: Tipo::Grafico,
            x: 72.0,
            y: 80.0,
            largura: 640.0,
            altura: 400.0,
            rotacao: 0.0,
            dados: dados_originais.clone(),
            ..Default::default()
        };
        let comum = converter_elemento(&elemento)
            .map_err(|erro| format!("adaptador {tipo}: {erro:?}"))?;
        assert_eq!(comum.id, elemento.id);
        assert_eq!((comum.x, comum.y), (elemento.x, elemento.y));
        assert_eq!(comum.grafico.categorias.len(), dados::grafico_exemplo(tipo).grade.len() - 1);
        assert_eq!(elemento.dados, dados_originais);
        assert_eq!(dados::gravar_grafico(&dados::ler_grafico(&elemento)), dados_originais);

        let svg = comum.exportar_svg().map_err(|erro| format!("SVG {tipo}: {erro:?}"))?;
        let pdf = comum.exportar_pdf().map_err(|erro| format!("PDF {tipo}: {erro:?}"))?;
        assert!(svg.starts_with("<svg"));
        assert!(pdf.starts_with(b"%PDF-1.4"));
        let caminho_svg = destino.join(format!("{tipo}.svg"));
        let caminho_pdf = destino.join(format!("{tipo}.pdf"));
        fs::write(&caminho_svg, svg)?;
        fs::write(&caminho_pdf, pdf)?;
        println!("OK {tipo}: {} + {} | .fsdz intacto",
            caminho_svg.display(), caminho_pdf.display());
    }
    println!("CHT-014: 8 arquivos vetoriais em {}", destino.display());
    Ok(())
}
