//! Ponte única do FeatherSlide para o motor featheroffice-image.
//! Compartilhada pelo Canvas Iced e pela preparação de camadas raster PDF;
//! os bytes e a serialização .fsdz permanecem intocados.
use featherslide_core::imagem::VisualImagem;
use feather_image::{AjustesCor, ImagemRgba, aplicar_cores_rgba, realcar_detalhes_rgba};
use image::{imageops, RgbaImage};
use std::io::Cursor;

/// Abertura do raster de prévia e PDF limitada antes de criar o buffer RGBA.
/// Bytes originais de .fsdz não são modificados por esta política.
pub const LIMITE_BYTES_RASTER: usize = 64 * 1024 * 1024;
pub const LIMITE_PIXELS_RASTER: u64 = 40_000_000;

/// Decodifica o raster com tamanho defensivo e preserva o resize Triangle do Slide.
/// O limite é aplicado antes da descompressão, inclusive para imagens hostis.
pub fn decodificar_raster(bytes: &[u8], max_lado: u32) -> Option<RgbaImage> {
    if bytes.is_empty() || bytes.len() > LIMITE_BYTES_RASTER || max_lado == 0 {
        return None;
    }
    let leitor = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format().ok()?;
    let (largura, altura) = leitor.into_dimensions().ok()?;
    if largura == 0 || altura == 0
        || u64::from(largura) * u64::from(altura) > LIMITE_PIXELS_RASTER {
        return None;
    }
    let mut imagem = image::load_from_memory(bytes).ok()?;
    if imagem.width().max(imagem.height()) > max_lado {
        imagem = imagem.resize(max_lado, max_lado, imageops::FilterType::Triangle);
    }
    Some(imagem.to_rgba8())
}


/// Aplica correções de cor e os filtros artísticos sobre os pixels originais.
pub fn cores(img: &mut RgbaImage, fx: &VisualImagem) {
    if fx.inverter_horizontal { imageops::flip_horizontal_in_place(img); }
    if fx.inverter_vertical { imageops::flip_vertical_in_place(img); }
    let ajustes = AjustesCor {
        brilho: fx.brilho,
        contraste: fx.contraste,
        saturacao: fx.saturacao,
        temperatura: fx.temperatura,
        matiz: fx.matiz,
        filtro: fx.filtro,
    };
    // Zero-copy: o próprio buffer RGBA do Iced/PDF é processado pelo motor neutro.
    let resultado = aplicar_cores_rgba(img.as_mut(), ajustes);
    debug_assert!(resultado.is_ok(), "VisualImagem deveria fornecer ajustes válidos");
    if fx.nitidez > 0.001 {
        // O blur gaussiano atual da GUI é mantido para preservar a aparência.
        // A máscara unsharp é compartilhada, sem materializar outro RGBAImage.
        let blurred = imageops::blur(img, 1.35);
        let resultado = realcar_detalhes_rgba(img.as_mut(), blurred.as_raw(), fx.nitidez);
        debug_assert!(resultado.is_ok(), "nitidez do VisualImagem deve ser válida");
    }
}

/// Deformação neutra do motor Image; modelo de imagem e layout permanecem no Slide.
pub fn efeitos(img: &mut RgbaImage, fx: &VisualImagem) {
    if img.width() == 0 || img.height() == 0 || fx.perspectiva.abs() <= 0.001 {
        return;
    }
    // Move o buffer em vez de clonar a matriz RGBA completa.
    // Falhas do motor restauram os pixels originais sem perda.
    let original = std::mem::replace(img, RgbaImage::new(0, 0));
    let origem = ImagemRgba::de_image(original).expect("RGBA do Slide válido");
    *img = match origem.aplicar_perspectiva(fx.perspectiva.clamp(-1.0, 1.0)) {
        Ok(saida) => saida.em_image(),
        Err(_) => origem.em_image(),
    };
}

/// Camada de reflexo em separado, a ser composta abaixo do retângulo do Slide.
pub fn gerar_reflexo(origem: &RgbaImage, intensidade: f32) -> RgbaImage {
    let (w, h) = origem.dimensions();
    if w == 0 || h == 0 {
        return RgbaImage::new(w.max(1), (h / 3).max(1));
    }
    let entrada = ImagemRgba::de_image(origem.clone()).expect("RGBA válido");
    entrada.gerar_reflexo(intensidade.clamp(0.0, 1.0))
        .expect("intensidade e dimensões válidas")
        .em_image()
}

/// Halo dourado, mantendo as margens visuais legadas e os canais RGBA.
pub fn gerar_brilho(origem: &RgbaImage, intensidade: f32) -> (RgbaImage, u32) {
    let (w, h) = origem.dimensions();
    if w == 0 || h == 0 {
        let margem = 4;
        return (RgbaImage::new(w.saturating_add(2 * margem), h.saturating_add(2 * margem)), margem);
    }
    let entrada = ImagemRgba::de_image(origem.clone()).expect("RGBA válido");
    let (camada, margem) = entrada.gerar_brilho_externo(intensidade.clamp(0.0, 1.0))
        .expect("intensidade e dimensões válidas");
    (camada.em_image(), margem)
}

/// Sombra por máscara alfa; a cor HEX permanece interpretada pelo consumidor.
pub fn gerar_sombra(origem: &RgbaImage, cor: &str, raio: f32) -> RgbaImage {
    let hex = cor.trim_start_matches('#');
    let numero = u32::from_str_radix(hex, 16).unwrap_or(0x00000080);
    let canais = if hex.len() == 8 {
        [
            ((numero >> 24) & 255) as u8,
            ((numero >> 16) & 255) as u8,
            ((numero >> 8) & 255) as u8,
            (numero & 255) as u8,
        ]
    } else {
        [
            ((numero >> 16) & 255) as u8,
            ((numero >> 8) & 255) as u8,
            (numero & 255) as u8,
            128,
        ]
    };
    if origem.width() == 0 || origem.height() == 0 {
        return origem.clone();
    }
    let entrada = ImagemRgba::de_image(origem.clone()).expect("RGBA válido");
    entrada.gerar_sombra(canais, if raio.is_finite() { raio.clamp(0.0, 28.0) } else { 0.0 })
        .expect("sigma e dimensões válidos")
        .em_image()
}


#[cfg(test)]
mod testes {
    use super::*;
    use image::Rgba;

    fn amostra() -> RgbaImage {
        RgbaImage::from_fn(24, 12, |x, y| Rgba([
            ((x * 13 + y * 9) % 256) as u8,
            ((x * 5 + y * 19) % 256) as u8,
            ((x * 31 + y * 7) % 256) as u8,
            match (x + y) % 3 { 0 => 0, 1 => 128, _ => 255 },
        ]))
    }

    #[test]
    fn filtro_compartilhado_e_deterministico_preserva_alfa_original() {
        let origem = amostra();
        let mut processada = origem.clone();
        let cfg = VisualImagem {
            brilho: 0.2, contraste: -0.1, saturacao: 1.4,
            temperatura: 0.3, matiz: 18.0, nitidez: 0.4,
            filtro: 2, inverter_horizontal: true,
            ..VisualImagem::default()
        };
        cores(&mut processada, &cfg);
        assert_eq!(origem, amostra());
        let mut antes: Vec<u8> = origem.pixels().map(|p| p.0[3]).collect();
        let mut depois: Vec<u8> = processada.pixels().map(|p| p.0[3]).collect();
        antes.sort_unstable();
        depois.sort_unstable();
        assert_eq!(antes, depois);
        let mut segunda = origem.clone();
        cores(&mut segunda, &cfg);
        assert_eq!(processada, segunda);
    }

    #[test]
    fn entrada_da_gui_e_do_pdf_passam_pelo_mesmo_motor() {
        let cfg = VisualImagem { filtro: 1, nitidez: 0.4, perspectiva: 0.2, ..Default::default() };
        let mut gui = amostra();
        let mut pdf = amostra();
        cores(&mut gui, &cfg);
        cores(&mut pdf, &cfg);
        efeitos(&mut gui, &cfg);
        efeitos(&mut pdf, &cfg);
        assert_eq!(gui, pdf);
        assert!(gui.pixels().any(|p| p.0[3] == 0));
        assert!(gui.pixels().any(|p| p.0[3] == 128));
    }

    #[test]
    fn camadas_para_pdf_preservam_pixeis_e_estabilidade() {
        let origem = amostra();
        let sombra = gerar_sombra(&origem, "#16212c80", 1.5);
        assert_eq!(sombra, gerar_sombra(&origem, "#16212c80", 1.5));
        let (brilho, margem) = gerar_brilho(&origem, 0.6);
        let (novo, nova_margem) = gerar_brilho(&origem, 0.6);
        assert_eq!((brilho, margem), (novo, nova_margem));
        assert_eq!(gerar_reflexo(&origem, 0.65), gerar_reflexo(&origem, 0.65));
        assert_eq!(origem, amostra());
    }
    #[test]
    fn exportador_pdf_recebe_rgba_preparado_no_motor_compartilhado() {
        use featherslide_core::{
            modelo::{Elemento, Slide, Tipo},
            quadro::{quadro, Dicas},
        };

        let origem = amostra();
        let mut processada = origem.clone();
        let cfg = VisualImagem { filtro: 1, brilho: 0.1, ..Default::default() };
        cores(&mut processada, &cfg);
        let slide = Slide {
            id: 1,
            layout: String::new(),
            fundo: "#ffffff".into(),
            fundo_config: Default::default(),
            ocultar_mestre: false,
            elementos: vec![Elemento {
                id: 2,
                tipo: Tipo::Imagem,
                x: 15.0,
                y: 20.0,
                largura: 100.0,
                altura: 60.0,
                ..Elemento::default()
            }],
            notas: String::new(),
            oculto: false,
            transicao: Default::default(),
            secao: String::new(),
            comentarios: Vec::new(),
        };
        let pagina = quadro(&slide, (960.0, 540.0), 1, &Dicas::default(), false);
        let callback = |_: &_| {
            Some((
                processada.width(),
                processada.height(),
                processada.as_raw().clone(),
            ))
        };
        let bytes = crate::pdf::exportar("Imagem compartilhada", &[pagina.clone()], &callback);
        assert!(bytes.starts_with(b"%PDF-1.5"));
        assert!(bytes.windows(6).any(|v| v == b"/SMask"));
        assert_eq!(bytes, crate::pdf::exportar("Imagem compartilhada", &[pagina], &callback));
        assert_eq!(origem, amostra());
    }
}
