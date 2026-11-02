//! A GUI do Slide consome o pipeline Image do adaptador IO.
//! Sem filtros duplicados no Iced: o PDF usa os mesmos bytes e efeitos.
//! As regressões abaixo preservam referências legadas até validação visual.
pub use featherslide_io::imagem_compartilhada::{
    cores, efeitos, gerar_brilho, gerar_reflexo, gerar_sombra,
};

#[cfg(test)]
use featherslide_core::imagem::VisualImagem;
#[cfg(test)]
use image::{imageops, Rgba, RgbaImage};

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn regressao_filtros_rgb_do_slide_compartilhados() {
        // Valores de referência gerados pelo pipeline anterior do Slide.
        let referencias = [
            (1u8, [79u8, 79u8, 79u8]),
            (2u8, [111, 99, 77]),
            (3u8, [135, 221, 55]),
            (4u8, [128, 64, 191]),
            (5u8, [120, 34, 55]),
        ];
        for (filtro, esperado) in referencias {
            let mut im = RgbaImage::from_pixel(1, 1, Rgba([120, 34, 200, 128]));
            let mut fx = VisualImagem::default();
            fx.filtro = filtro;
            cores(&mut im, &fx);
            let p = im.get_pixel(0, 0).0;
            for canal in 0..3 {
                assert!(p[canal].abs_diff(esperado[canal]) <= 2,
                    "filtro {filtro}, canal {canal}: {} != {}", p[canal], esperado[canal]);
            }
            assert_eq!(p[3], 128, "alfa não pode mudar");
        }
    }

    #[test]
    fn espelhamento_brilho_filtro_e_nitidez_preservam_alfa() {
        let mut im = RgbaImage::new(2, 1);
        im.put_pixel(0,0,Rgba([255,0,0,255]));
        im.put_pixel(1,0,Rgba([0,0,255,128]));
        let mut cfg = VisualImagem::default();
        cfg.inverter_horizontal = true;
        cfg.filtro = 1;
        cores(&mut im, &cfg);
        assert_eq!(im.get_pixel(0,0).0[3],128);
        assert_eq!(im.get_pixel(1,0).0[3],255);
        assert_eq!(im.get_pixel(0,0).0[0],im.get_pixel(0,0).0[2]);
        cfg.perspectiva=0.2;
        efeitos(&mut im,&cfg);
        assert_eq!(im.dimensions(),(2,1));
    }
    // Referência congelada do commit anterior à extração 6ea4a8d.
    // Mantida somente no teste para detectar regressões do consumidor.
fn cores_legadas(img: &mut RgbaImage, fx: &VisualImagem) {
    if fx.inverter_horizontal { imageops::flip_horizontal_in_place(img); }
    if fx.inverter_vertical { imageops::flip_vertical_in_place(img); }
    let cor = fx.brilho.abs() > 0.0001 || fx.contraste.abs() > 0.0001
        || (fx.saturacao-1.0).abs() > 0.0001 || fx.temperatura.abs() > 0.0001
        || fx.matiz.abs() > 0.0001 || fx.filtro != 0;
    if cor {
        let (sin, cos) = fx.matiz.to_radians().sin_cos();
        for p in img.pixels_mut() {
            let [r,g,b,a] = p.0;
            if a == 0 { continue; }
            let (mut r,mut g,mut b) = (f32::from(r)/255.0, f32::from(g)/255.0, f32::from(b)/255.0);
            // Mistura por luminância e rotação YIQ da matiz.
            let luma = r*0.299 + g*0.587 + b*0.114;
            let i = r*0.596 - g*0.275 - b*0.321;
            let q = r*0.212 - g*0.523 + b*0.311;
            let ii = i*cos - q*sin;
            let qq = i*sin + q*cos;
            r = luma + 0.956*ii + 0.621*qq;
            g = luma - 0.272*ii - 0.647*qq;
            b = luma - 1.106*ii + 1.703*qq;
            let lum = r*0.299 + g*0.587 + b*0.114;
            r = lum + (r-lum)*fx.saturacao;
            g = lum + (g-lum)*fx.saturacao;
            b = lum + (b-lum)*fx.saturacao;
            // Contraste em torno de cinza médio, brilho e temperatura em unidades normalizadas.
            let contrast = (1.0 + fx.contraste).max(0.0);
            let aplicar = |c: f32, t: f32| {
                ((c-0.5)*contrast + 0.5 + fx.brilho*0.45 + t).clamp(0.0, 1.0)
            };
            r = aplicar(r, fx.temperatura*0.17);
            g = aplicar(g, fx.temperatura*0.025);
            b = aplicar(b, -fx.temperatura*0.17);
            match fx.filtro {
                1 => {
                    let y = r*0.299+g*0.587+b*0.114;
                    r=y; g=y; b=y;
                },
                2 => {
                    let (or,og,ob)=(r,g,b);
                    r=(or*0.393+og*0.769+ob*0.189).min(1.0);
                    g=(or*0.349+og*0.686+ob*0.168).min(1.0);
                    b=(or*0.272+og*0.534+ob*0.131).min(1.0);
                },
                3 => { r=1.0-r; g=1.0-g; b=1.0-b; },
                4 => {
                    r=(r*4.0).round()/4.0; g=(g*4.0).round()/4.0; b=(b*4.0).round()/4.0;
                },
                5 => {
                    if r>0.5 { r=1.0-r; }
                    if g>0.5 { g=1.0-g; }
                    if b>0.5 { b=1.0-b; }
                },
                _ => {},
            }
            *p = Rgba([
                (r.clamp(0.0,1.0)*255.0).round() as u8,
                (g.clamp(0.0,1.0)*255.0).round() as u8,
                (b.clamp(0.0,1.0)*255.0).round() as u8,
                a,
            ]);
        }
    }
    if fx.nitidez > 0.001 {
        // Unsharp-mask: evita mutar a transparência/recorte.
        let blurred = imageops::blur(img, 1.35);
        for (x,y,p) in img.enumerate_pixels_mut() {
            let original = *p;
            let suavizado = blurred.get_pixel(x,y);
            for i in 0..3 {
                let valor = original.0[i] as f32 +
                    (original.0[i] as f32 - suavizado.0[i] as f32)*fx.nitidez*2.2;
                p.0[i] = valor.clamp(0.0,255.0).round() as u8;
            }
        }
    }
}


    #[test]
    fn imagem_completa_equivale_ao_pipeline_anterior_do_slide() {
        let origem = RgbaImage::from_fn(64,48, |x,y| Rgba([
            ((x*19+y*7)%256) as u8, ((x*3+y*23)%256) as u8,
            if x < 32 { 220 } else { 20 }, [0,128,255][(x/16%3) as usize]
        ]));
        let mut casos: Vec<(&str, VisualImagem)> = vec![("original", VisualImagem::default())];
        for (filtro,nome) in [(1,"cinza"),(2,"sepia"),(3,"negativo"),(4,"posterizar"),(5,"solarizar")] {
            casos.push((nome,VisualImagem { filtro, ..VisualImagem::default() }));
        }
        for (nome,cfg) in [
            ("saturacao",VisualImagem { saturacao:1.6,..VisualImagem::default() }),
            ("temperatura",VisualImagem { temperatura:0.65,..VisualImagem::default() }),
            ("matiz",VisualImagem { matiz:70.,..VisualImagem::default() }),
            ("nitidez",VisualImagem { nitidez:0.7,..VisualImagem::default() }),
            ("composto",VisualImagem { filtro:2,brilho:0.2,contraste:0.15,saturacao:1.3,temperatura:0.4,matiz:42.,nitidez:0.4,inverter_horizontal:true,..VisualImagem::default() }),
        ] { casos.push((nome,cfg)); }
        let pasta = std::env::var_os("FEATHEROFFICE_VALIDACAO_VISUAL_DIR").map(std::path::PathBuf::from);
        if let Some(ref p) = pasta { std::fs::create_dir_all(p).unwrap(); }
        for (nome,cfg) in casos {
            let mut esperado = origem.clone();
            let mut atual = origem.clone();
            cores_legadas(&mut esperado,&cfg);
            cores(&mut atual,&cfg);
            assert_eq!(atual,esperado,"regressão visual em {nome}");
            if let Some(ref p) = pasta {
                esperado.save(p.join(format!("{nome}-anterior.png"))).unwrap();
                atual.save(p.join(format!("{nome}-atual.png"))).unwrap();
            }
        }
    }

    #[test]
    fn regressao_img008_slide_perspectiva_reflexo_sombra_e_brilho() {
        let origem = RgbaImage::from_fn(64, 48, |x, y| Rgba([
            (x * 3) as u8, (y * 5) as u8, 130, [0, 128, 255][(x % 3) as usize],
        ]));
        let mut atual = origem.clone();
        let fx = VisualImagem { perspectiva: 0.65, ..Default::default() };
        efeitos(&mut atual, &fx);
        let mut legado = origem.clone();
        let (w, h) = legado.dimensions();
        for (x, y, p) in legado.enumerate_pixels_mut() {
            let altura = y as f32 / h as f32;
            let proporcao = (1.0 - fx.perspectiva * 0.65 * (1.0 - altura)).max(0.2);
            let u = ((x as f32 + 0.5) / w as f32 - 0.5) / proporcao + 0.5;
            *p = if (0.0..1.0).contains(&u) {
                *origem.get_pixel((u * w as f32).floor().clamp(0.0, (w - 1) as f32) as u32, y)
            } else {
                Rgba([0, 0, 0, 0])
            };
        }
        assert_eq!(atual, legado, "deformação de perspectiva deve preservar raster legado");

        let reflexo = gerar_reflexo(&origem, 0.7);
        assert_eq!(reflexo.dimensions(), (64, 16));
        for y in 0..16 {
            for x in 0..64 {
                let mut esperado = *origem.get_pixel(x, 47 - y);
                let t = y as f32 / 16.0;
                esperado.0[3] = (f32::from(esperado.0[3]) * 0.7 * (1.0 - t).powi(2) * 0.7).round() as u8;
                assert_eq!(*reflexo.get_pixel(x, y), esperado);
            }
        }

        let mut cor = origem.clone();
        for p in cor.pixels_mut() {
            p.0[..3].copy_from_slice(&[22, 33, 44]);
        }
        let mut esperado_sombra = imageops::blur(&cor, 1.5);
        for p in esperado_sombra.pixels_mut() {
            p.0[..3].copy_from_slice(&[22, 33, 44]);
            p.0[3] = (f32::from(p.0[3]) * (128.0 / 255.0)).round() as u8;
        }
        assert_eq!(gerar_sombra(&origem, "#16212c80", 1.5), esperado_sombra);

        let (brilho, margem) = gerar_brilho(&origem, 0.8);
        assert!(margem >= 4);
        assert_eq!(brilho.dimensions(), (64 + 2 * margem, 48 + 2 * margem));
        assert!(brilho.pixels().any(|p| p.0[3] > 0));
        assert_eq!(origem.get_pixel(0, 0).0, [0, 0, 130, 0]);
    }

}
