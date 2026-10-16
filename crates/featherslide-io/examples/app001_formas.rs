//! Smoke APP-001: as quatro formas nativas usando o mesmo Drawing da GUI.
//! cargo run --manifest-path featherslide/crates/featherslide-io/Cargo.toml --example app001_formas -j 2 -- /tmp/app001-formas.svg
use featherslide_io::drawing::converter_forma_basica;
use feather_draw::{exportar_svg, Cena};
use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let destino = PathBuf::from(
        env::args().nth(1).unwrap_or_else(|| "/tmp/app001-formas.svg".into()),
    );
    let mut cena = Cena::nova(960.0, 540.0);
    for (tipo, area, preenchimento, contorno) in [
        ("retangulo", [40.0, 40.0, 150.0, 85.0], "#00669980", "#103355"),
        ("elipse", [240.0, 40.0, 150.0, 85.0], "#cc6600", "#552200"),
        ("triangulo", [440.0, 40.0, 150.0, 85.0], "#339955", "#116633"),
        ("linha", [640.0, 40.0, 150.0, 85.0], "", "#992277"),
    ] {
        let figura = converter_forma_basica(
            tipo, area, preenchimento, contorno, 2.5,
        )?.ok_or_else(|| format!("tipo não migrado: {tipo}"))?;
        cena.adicionar(figura);
    }
    let svg = exportar_svg(&cena).map_err(|e| format!("Erro ao exportar SVG: {e:?}"))?;
    if !svg.contains(r#"viewBox="0 0 960 540""#) || cena.figuras.len() != 4 {
        return Err("exportação das formas nativas incompleta".into());
    }
    fs::write(&destino, svg.as_bytes())?;
    println!(
        "APP-001: {} figuras, {} bytes em {}",
        cena.figuras.len(),
        svg.len(),
        destino.display(),
    );
    Ok(())
}
