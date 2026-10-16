//! IMG-013: regressões da ponte Image usada pelo Canvas e pelo PDF do Slide.
use featherslide_core::imagem::VisualImagem;
use featherslide_io::imagem_compartilhada::{
    cores, decodificar_raster, efeitos, LIMITE_BYTES_RASTER,
};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;

fn origem() -> RgbaImage {
    RgbaImage::from_fn(9, 6, |x, y| {
        Rgba([
            ((x * 17 + y * 11) % 256) as u8,
            (x * 23) as u8,
            (y * 31) as u8,
            [0, 128, 255][((x + y) % 3) as usize],
        ])
    })
}

fn codificar_png(img: &RgbaImage) -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(img.clone())
        .write_to(&mut bytes, ImageFormat::Png)
        .expect("escrita PNG do corpus");
    bytes.into_inner()
}

#[test]
fn decodificacao_valida_preserva_alfa_rgb_e_dimensoes() {
    let original = origem();
    let bytes = codificar_png(&original);
    assert_eq!(decodificar_raster(&bytes, 1024), Some(original.clone()));
    assert_eq!(decodificar_raster(&bytes, 0), None);
    assert_eq!(decodificar_raster(b"PNG falso", 1024), None);
    // Evitar alocação proporcional ao conteúdo hostil é parte do contrato.
    assert!(LIMITE_BYTES_RASTER > bytes.len());
}

#[test]
fn reducao_equivale_ao_triangle_legado() {
    let original = origem();
    let bytes = codificar_png(&original);
    let esperado = DynamicImage::ImageRgba8(original)
        .resize(4, 4, image::imageops::FilterType::Triangle)
        .to_rgba8();
    let novo = decodificar_raster(&bytes, 4).unwrap();
    assert_eq!(novo, esperado);
    assert!(novo.width() <= 4 && novo.height() <= 4);
}

#[test]
fn perspectiva_sem_clone_equivale_ao_motor_e_nao_muda_origem() {
    let original = origem();
    let cfg = VisualImagem {
        perspectiva: 0.65,
        ..Default::default()
    };
    let compartilhada = feather_image::ImagemRgba::de_image(original.clone()).unwrap();
    let esperado = compartilhada.aplicar_perspectiva(cfg.perspectiva)
        .unwrap().em_image();
    let mut aplicada = original.clone();
    efeitos(&mut aplicada, &cfg);
    assert_eq!(aplicada, esperado);
    assert_eq!(original, origem(), "origem armazenada permanece intacta");
}

#[test]
fn operacoes_invalidas_nao_descartam_buffer() {
    let mut imagem = origem();
    let original = imagem.clone();
    let cfg = VisualImagem {
        perspectiva: f32::NAN,
        ..Default::default()
    };
    efeitos(&mut imagem, &cfg);
    assert_eq!(imagem, original);
}

#[test]
fn canvas_e_pdf_compartilham_pixels_e_alfa() {
    let bytes = codificar_png(&origem());
    let cfg = VisualImagem {
        filtro: 2,
        brilho: 0.16,
        perspectiva: 0.21,
        nitidez: 0.4,
        ..Default::default()
    };
    let mut canvas = decodificar_raster(&bytes, 1024).unwrap();
    let mut pdf = decodificar_raster(&bytes, 1024).unwrap();
    cores(&mut canvas, &cfg);
    efeitos(&mut canvas, &cfg);
    cores(&mut pdf, &cfg);
    efeitos(&mut pdf, &cfg);
    assert_eq!(canvas, pdf);
    assert!(pdf.pixels().any(|pixel| pixel.0[3] == 0));
    assert!(pdf.pixels().any(|pixel| pixel.0[3] == 128));
}
