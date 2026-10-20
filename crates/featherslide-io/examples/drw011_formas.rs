//! Smoke de migração real FeatherSlide → FeatherWorks Drawing.
//! cargo run --manifest-path featherslide/crates/featherslide-io/Cargo.toml --example drw011_formas -- /tmp/drw011-formas.svg
use featherslide_core::modelo::{Elemento, Tipo};
use featherslide_io::drawing::{cena_dos_elementos, exportar_svg};
use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let destino = PathBuf::from(
        env::args().nth(1).unwrap_or_else(|| "/tmp/drw011-formas.svg".into()),
    );
    let retangulo = Elemento {
        id: 71, tipo: Tipo::Retangulo,
        x: 40.0, y: 60.0, largura: 180.0, altura: 100.0,
        rotacao: 20.0,
        preenchimento: "#3478cfb0".into(), contorno: "#153060".into(),
        espessura: 3.0, ..Elemento::default()
    };
    let anel = Elemento {
        id: 72, tipo: Tipo::Anel,
        x: 270.0, y: 80.0, largura: 160.0, altura: 160.0,
        preenchimento: "#e58b45".into(), contorno: "#44271a".into(),
        espessura: 2.0, ..Elemento::default()
    };
    let mut icone = Elemento {
        id: 73, tipo: Tipo::Icone,
        x: 475.0, y: 75.0, largura: 120.0, altura: 120.0,
        preenchimento: "#1c8c65".into(), contorno: "#1c8c65".into(),
        espessura: 3.0, ..Elemento::default()
    };
    icone.dados = "home".into();
    let entrada = [retangulo, anel, icone];
    let base = entrada.to_vec();
    let mapeamento = cena_dos_elementos(&entrada, [960.0, 540.0])?;
    assert_eq!(entrada.to_vec(), base, "o modelo original não pode mudar");
    assert_eq!(mapeamento.ids.first(), Some(&71));
    assert_eq!(mapeamento.ids.last(), Some(&73));
    let svg = exportar_svg(&entrada, [960.0, 540.0])?;
    assert!(svg.contains("viewBox=\"0 0 960 540\""));
    fs::write(&destino, svg.as_bytes())?;
    println!("SVG exportado: {} ({} bytes)", destino.display(), svg.len());
    println!("IDs do modelo preservados: {:?}", mapeamento.ids);
    Ok(())
}
