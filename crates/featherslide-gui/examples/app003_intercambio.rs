//! APP-003: prova executável de Charts + Tables ligados ao Slide sem mudar .fsdz.
//! A partir da raiz: cargo run --manifest-path featherslide/crates/featherslide-gui/Cargo.toml --example app003_intercambio -j 2 -- /tmp/app003
use featherslide_core::{
    dados,
    modelo::{Elemento, Tipo},
    sessao::Sessao,
};
use featherslide_io::{fsdz, grafico_compartilhado::converter_elemento};
use featherslide_tabelas::{comum_para_slide, slide_para_comum};
use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let destino = PathBuf::from(env::args().nth(1).unwrap_or_else(|| "/tmp/app003".into()));
    fs::create_dir_all(&destino)?;
    let grade = vec![
        vec!["Produto".to_string(), "Valor".to_string()],
        vec!["Café".to_string(), "42".to_string()],
    ];
    let tabela = Elemento {
        id: 91,
        tipo: Tipo::Tabela,
        x: 20.0,
        y: 30.0,
        largura: 320.0,
        altura: 160.0,
        dados: dados::gravar_tabela(&grade),
        ..Elemento::default()
    };
    let comum = slide_para_comum(&tabela)
        .map_err(|e| format!("Tabela → motor compartilhado: {e:?}"))?;
    assert_eq!(comum.dimensoes(), (2, 2));
    let reconvertida = comum_para_slide(&comum, &tabela)
        .map_err(|e| format!("Tabela → Slide: {e:?}"))?;
    assert_eq!(reconvertida, tabela, "a tabela não deve perder atributos");

    let grafico = Elemento {
        id: 92,
        tipo: Tipo::Grafico,
        x: 380.0,
        y: 30.0,
        largura: 500.0,
        altura: 330.0,
        dados: dados::gravar_grafico(&dados::grafico_exemplo("colunas")),
        ..Elemento::default()
    };
    let convertido = converter_elemento(&grafico)
        .map_err(|e| format!("Gráfico → Charts: {e:?}"))?;
    assert_eq!(convertido.id, grafico.id);
    let svg = convertido.exportar_svg()
        .map_err(|e| format!("Charts → SVG: {e:?}"))?;
    fs::write(destino.join("grafico.svg"), &svg)?;

    let mut apresentacao = Sessao::default().apresentacao().clone();
    let slide = apresentacao.slides.get_mut(0)
        .ok_or("Apresentação inicial sem slide")?;
    slide.elementos.push(tabela.clone());
    slide.elementos.push(grafico.clone());
    apresentacao.proximo_id = apresentacao.proximo_id.max(93);
    let bytes = fsdz::gravar(&apresentacao)?;
    fs::write(destino.join("app003.fsdz"), &bytes)?;
    let restaurada = fsdz::ler(&bytes)?;
    let elementos = &restaurada.slides[0].elementos;
    for original in [&tabela, &grafico] {
        let lido = elementos.iter().find(|e| e.id == original.id)
            .ok_or("Elemento perdido no FSDZ")?;
        assert_eq!(lido.dados, original.dados, "dados nativos preservados");
        assert_eq!(lido.tipo, original.tipo, "tipo preservado");
    }
    let relido = elementos.iter().find(|e| e.id == grafico.id)
        .ok_or("Gráfico não retornou do FSDZ")?;
    let relido_svg = converter_elemento(relido)
        .map_err(|e| format!("Charts após FSDZ: {e:?}"))?
        .exportar_svg()
        .map_err(|e| format!("SVG após FSDZ: {e:?}"))?;
    assert_eq!(relido_svg, svg, "SVG antes e depois do FSDZ");

    eprintln!("APP-003: tabela 2×2, gráfico SVG e round-trip FSDZ preservados");
    eprintln!("SVG: {}", destino.join("grafico.svg").display());
    eprintln!("FSDZ: {}", destino.join("app003.fsdz").display());
    Ok(())
}
