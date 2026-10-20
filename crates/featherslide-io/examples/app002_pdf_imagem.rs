//! APP-002 — validação da mesma imagem processada no PNG e no PDF.
//! cargo run --manifest-path featherslide/crates/featherslide-io/Cargo.toml --example app002_pdf_imagem -- /tmp/app002
use featherslide_core::{
    imagem::VisualImagem,
    modelo::{Elemento, Slide, Tipo},
    quadro::{quadro, Dicas},
};
use featherslide_io::{imagem_compartilhada, pdf};
use image::{Rgba, RgbaImage};
use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let destino = PathBuf::from(env::args().nth(1).unwrap_or_else(|| "/tmp/app002".into()));
    fs::create_dir_all(&destino)?;
    let origem = RgbaImage::from_fn(64, 48, |x, y| Rgba([
        (x * 3 + y * 5) as u8,
        (x * 2 + y * 4) as u8,
        130,
        [0, 128, 255][((x + y) % 3) as usize],
    ]));
    let mut visivel = origem.clone();
    let visual = VisualImagem {
        brilho: 0.15, contraste: 0.1, saturacao: 1.2,
        filtro: 2, nitidez: 0.3, ..Default::default()
    };
    imagem_compartilhada::cores(&mut visivel, &visual);
    imagem_compartilhada::efeitos(&mut visivel, &visual);

    let png = destino.join("imagem.png");
    visivel.save(&png)?;

    let slide = Slide {
        id: 1,
        layout: String::new(),
        fundo: "#ffffff".into(),
        fundo_config: Default::default(),
        ocultar_mestre: false,
        elementos: vec![Elemento {
            id: 2, tipo: Tipo::Imagem,
            x: 45.0, y: 35.0, largura: 320.0, altura: 240.0,
            visual_imagem: visual,
            ..Elemento::default()
        }],
        notas: String::new(),
        oculto: false,
        transicao: Default::default(),
        secao: String::new(),
        comentarios: Vec::new(),
    };
    let quadro = quadro(&slide, (960.0, 540.0), 1, &Dicas::default(), false);
    let callback = |_: &_| Some((visivel.width(), visivel.height(), visivel.as_raw().clone()));
    let dados = pdf::exportar("APP-002 FeatherWorks Image", &[quadro], &callback);
    assert!(dados.starts_with(b"%PDF-1.5"));
    assert!(dados.windows(6).any(|x| x == b"/SMask"), "PDF deve manter alfa");
    let pdf = destino.join("imagem.pdf");
    fs::write(&pdf, &dados)?;
    assert_eq!(origem.get_pixel(0, 0).0[3], 0, "origem imutável");
    println!("PNG: {}", png.display());
    println!("PDF: {} ({} bytes, com SMask)", pdf.display(), dados.len());
    Ok(())
}
