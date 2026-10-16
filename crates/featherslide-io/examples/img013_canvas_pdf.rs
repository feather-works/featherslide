//! IMG-013 — exemplo do raster comum usado no Canvas e na exportação PDF.
//! cargo run --manifest-path featherslide/crates/featherslide-io/Cargo.toml --example img013_canvas_pdf -j 2 -- /tmp/img013
use featherslide_core::{
    imagem::VisualImagem,
    modelo::{Elemento, Slide, Tipo},
    quadro::{quadro, Dicas},
};
use featherslide_io::{imagem_compartilhada, pdf};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::{env, error::Error, fs, io::Cursor, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let destino = PathBuf::from(env::args().nth(1).unwrap_or_else(|| "/tmp/img013".into()));
    fs::create_dir_all(&destino)?;

    let original = RgbaImage::from_fn(96, 72, |x, y| Rgba([
        ((x * 2 + y * 3) % 256) as u8,
        ((y * 3) % 256) as u8,
        180,
        [0, 128, 255][((x + y) % 3) as usize],
    ]));
    let mut png = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(original)
        .write_to(&mut png, ImageFormat::Png)?;
    fs::write(destino.join("original.png"), png.get_ref())?;

    // A mesma API é chamada por desenho.rs no FeatherSlide Iced e,
    // através de desenho::decodificar, pelo callback da exportação PDF.
    let mut processada = imagem_compartilhada::decodificar_raster(png.get_ref(), 96)
        .ok_or("PNG de exemplo rejeitado pelo decodificador")?;
    let visual = VisualImagem {
        brilho: 0.1,
        filtro: 2,
        nitidez: 0.3,
        perspectiva: 0.2,
        ..Default::default()
    };
    imagem_compartilhada::cores(&mut processada, &visual);
    imagem_compartilhada::efeitos(&mut processada, &visual);
    processada.save(destino.join("previsualizacao.png"))?;

    let slide = Slide {
        id: 1,
        layout: String::new(),
        fundo: "#ffffff".into(),
        fundo_config: Default::default(),
        ocultar_mestre: false,
        elementos: vec![Elemento {
            id: 2,
            tipo: Tipo::Imagem,
            x: 50.0,
            y: 50.0,
            largura: 360.0,
            altura: 270.0,
            visual_imagem: visual,
            ..Elemento::default()
        }],
        notas: String::new(),
        oculto: false,
        transicao: Default::default(),
        secao: String::new(),
        comentarios: Vec::new(),
    };
    let pagina = quadro(&slide, (960.0, 540.0), 1, &Dicas::default(), false);
    let callback = |_: &_| Some((
        processada.width(),
        processada.height(),
        processada.as_raw().clone(),
    ));
    let pdf_bytes = pdf::exportar("IMG-013 — Canvas/PDF", &[pagina], &callback);
    if !pdf_bytes.starts_with(b"%PDF-1.5") || !pdf_bytes.windows(6).any(|x| x == b"/SMask") {
        return Err("PDF não manteve máscara de transparência".into());
    }
    fs::write(destino.join("previsualizacao.pdf"), pdf_bytes)?;

    eprintln!("IMG-013 — gerado: {}", destino.display());
    eprintln!("original.png, previsualizacao.png e previsualizacao.pdf (com /SMask)");
    Ok(())
}
