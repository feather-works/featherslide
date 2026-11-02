//! Compressão/reescalonamento explícitos, separados dos efeitos visuais do documento.
//! Evita perder o arquivo original até o usuário confirmar. Ctrl+Z restaura os bytes.
use std::io::Cursor;

/// Retorna data:image/ com bytes recompostos para a mesma imagem.
/// Com alfa mantém PNG; imagens opacas viram JPEG com qualidade solicitada.
pub fn recomprimir(origem: &str, lado: Option<u32>, qualidade: u8) -> Result<String, String> {
    let (mime, bytes) = featherslide_io::base64::de_data_url(origem)
        .ok_or("Imagem inválida: não é uma URL data:image.")?;
    if !mime.starts_with("image/") || bytes.len() > 64 * 1024 * 1024 {
        return Err("Formato inválido ou imagem maior que 64 MB.".into());
    }
    let mut imagem = image::load_from_memory(&bytes).map_err(|e| format!("Não consegui abrir a imagem: {e}"))?;
    let (w,h) = (imagem.width(), imagem.height());
    if w == 0 || h == 0 || u64::from(w)*u64::from(h) > 80_000_000 {
        return Err("A imagem excede o limite seguro de pixels.".into());
    }
    if let Some(limite) = lado {
        if limite < 32 || limite > 8192 { return Err("Resolução solicitada inválida.".into()); }
        if w.max(h) > limite {
            imagem = imagem.resize(limite, limite, image::imageops::FilterType::Lanczos3);
        }
    }
    let rgba = imagem.to_rgba8();
    let alfa = rgba.pixels().any(|p| p.0[3] != 255);
    let mut out = Vec::new();
    let tipo = if alfa {
        image::DynamicImage::ImageRgba8(rgba).write_to(&mut Cursor::new(&mut out),
            image::ImageFormat::Png).map_err(|e| format!("Erro ao recomprimir PNG: {e}"))?;
        "image/png"
    } else {
        let rgb = imagem.to_rgb8();
        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, qualidade.clamp(20,100));
        enc.encode_image(&image::DynamicImage::ImageRgb8(rgb))
            .map_err(|e| format!("Erro ao recomprimir JPEG: {e}"))?;
        "image/jpeg"
    };
    Ok(featherslide_io::base64::para_data_url(tipo, &out))
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn reducao_real_e_jpeg_com_qualidade_limitada() {
        let im = image::RgbaImage::from_pixel(100,80,image::Rgba([120,210,40,255]));
        let mut buf=Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(im).write_to(&mut buf,image::ImageFormat::Png).unwrap();
        let dado=featherslide_io::base64::para_data_url("image/png",&buf.into_inner());
        let comprimida=recomprimir(&dado,Some(40),70).unwrap();
        let (_,v)=featherslide_io::base64::de_data_url(&comprimida).unwrap();
        let foto=image::load_from_memory(&v).unwrap();
        assert_eq!((foto.width(), foto.height()), (40, 32));
        assert!(comprimida.starts_with("data:image/jpeg;"));
    }
    #[test]
    fn mantem_transparencia_por_png() {
        let im = image::RgbaImage::from_pixel(50,20,image::Rgba([255,0,0,55]));
        let mut buf=Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(im).write_to(&mut buf,image::ImageFormat::Png).unwrap();
        let dado=featherslide_io::base64::para_data_url("image/png",&buf.into_inner());
        let out=recomprimir(&dado,None,60).unwrap();
        assert!(out.starts_with("data:image/png;"));
    }
}
