//! Pintura de um quadro do motor num `Frame` do Iced. Serve o palco, a apresentação e as
//! miniaturas: o que é decisão (linhas, recorte, geometria) já vem pronto do motor.

use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use featherslide_core::dados::Primitiva;
use featherslide_core::modelo::{Animacao, Direcao, TipoAnimacao};
use featherslide_core::quadro::{ElementoDesenhado, Quadro};
use iced::border::Radius;
use iced::widget::canvas::{self, Frame, Path, Stroke, Text};
use iced::widget::image::Handle;
use iced::{Color, Point, Rectangle, Size, Vector};

use crate::palco::{cor_hex, OPACIDADE};

const LADO_MAXIMO_IMAGEM: u32 = 1024;
const SEGMENTOS: usize = 240;

thread_local! {
    /// Imagem em modo de enquadramento: a imagem inteira aparece esmaecida atrás da moldura.
    pub static ENQUADRANDO: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
    static COMPLETAS: RefCell<HashMap<u64, Option<(Handle, f32)>>> = RefCell::new(HashMap::new());
    /// Imagens já decodificadas (e recortadas pela forma), por elemento e conteúdo.
    static IMAGENS: RefCell<HashMap<u64, Option<Handle>>> = RefCell::new(HashMap::new());
    static SOMBRAS_IMAGEM: RefCell<HashMap<u64, Handle>> = RefCell::new(HashMap::new());
    static REFLEXOS_IMAGEM: RefCell<HashMap<u64, Handle>> = RefCell::new(HashMap::new());
    static SOMBRAS_FORMA: RefCell<HashMap<String, (Handle,f32)>> = RefCell::new(HashMap::new());
    static FORMAS: RefCell<HashMap<String, Handle>> = RefCell::new(HashMap::new());
    static HALOS_IMAGEM: RefCell<HashMap<u64, (Handle, f32, f32)>> = RefCell::new(HashMap::new());
    /// Pedaços já aparados de imagens que passam da borda do palco, por imagem e retângulo.
    static ULTIMA_CHAVE: RefCell<HashMap<u64, u64>> = RefCell::new(HashMap::new());
    static CORTES: RefCell<HashMap<(u64, u32, u32, u32, u32), Handle>> = RefCell::new(HashMap::new());
    /// A área visível (em coordenadas do canvas) que o desenho não pode ultrapassar: o Iced
    /// não apara imagens pelo limite do canvas, então elas são cortadas aqui.
    pub static AREA_VISIVEL: std::cell::Cell<Option<Rectangle>> = const { std::cell::Cell::new(None) };
}

fn chave(el: &ElementoDesenhado) -> u64 {
    let mut h = DefaultHasher::new();
    el.id.hash(&mut h);
    el.imagem.len().hash(&mut h);
    el.imagem.get(..96.min(el.imagem.len())).hash(&mut h);
    el.imagem.get(el.imagem.len().saturating_sub(96)..).hash(&mut h);
    el.recorte.hash(&mut h);
    el.fundo_removido.to_bits().hash(&mut h);
    el.fundo_cor.hash(&mut h);
    el.fundo_suave.to_bits().hash(&mut h);
    el.fundo_modo.hash(&mut h);
    el.fundo_refino.hash(&mut h);
    el.fundo_marcas.hash(&mut h);
    el.fundo_borda.to_bits().hash(&mut h);
    el.fundo_pena.to_bits().hash(&mut h);
    el.visual_imagem.codificar().hash(&mut h);
    if let Some(sh) = &el.sombra {
        sh.cor.hash(&mut h);
        sh.desfoque.to_bits().hash(&mut h);
        sh.dx.to_bits().hash(&mut h);
        sh.dy.to_bits().hash(&mut h);
    }
    for a in &el.ajustes {
        a.to_bits().hash(&mut h);
    }
    // O recorte depende da proporção da caixa (elipse vs círculo).
    ((el.largura / el.altura.max(1.0) * 100.0) as i64).hash(&mut h);
    h.finish()
}

/// Projeta sombra suave, com cache separado das imagens normais.
fn sombra_da_imagem(el: &ElementoDesenhado) -> Option<Handle> {
    let sombra = el.sombra.as_ref().filter(|s| !s.cor.is_empty())?;
    let k = chave(el);
    SOMBRAS_IMAGEM.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.len() > 80 { cache.clear(); }
        if !cache.contains_key(&k) {
            let origem = decodificar(el)?;
            let escala = origem.width() as f32 / el.largura.max(1.0);
            let img = crate::imagem_processamento::gerar_sombra(&origem, &sombra.cor, sombra.desfoque*escala);
            let handle = Handle::from_rgba(img.width(), img.height(), img.into_raw());
            cache.insert(k,handle);
        }
        cache.get(&k).cloned()
    })
}

fn reflexo_da_imagem(el: &ElementoDesenhado) -> Option<Handle> {
    if el.visual_imagem.reflexo <= 0.001 { return None; }
    REFLEXOS_IMAGEM.with(|reflexos| {
        let mut cache = reflexos.borrow_mut();
        if cache.len() > 70 { cache.clear(); }
        let key = chave(el);
        if !cache.contains_key(&key) {
            let foto = decodificar(el)?;
            let reflexo = crate::imagem_processamento::gerar_reflexo(
                &foto, el.visual_imagem.reflexo);
            cache.insert(key, Handle::from_rgba(reflexo.width(), reflexo.height(), reflexo.into_raw()));
        }
        cache.get(&key).cloned()
    })
}

fn halo_da_imagem(el: &ElementoDesenhado) -> Option<(Handle, f32, f32)> {
    if el.visual_imagem.brilho_externo <= 0.001 { return None; }
    HALOS_IMAGEM.with(|halos| {
        let mut cache = halos.borrow_mut();
        if cache.len() > 70 { cache.clear(); }
        let key = chave(el);
        if !cache.contains_key(&key) {
            let foto = decodificar(el)?;
            let (halo, margem) = crate::imagem_processamento::gerar_brilho(
                &foto, el.visual_imagem.brilho_externo);
            let fx = margem as f32/foto.width().max(1) as f32;
            let fy = margem as f32/foto.height().max(1) as f32;
            cache.insert(key, (Handle::from_rgba(halo.width(), halo.height(),
                halo.into_raw()),fx,fy));
        }
        cache.get(&key).cloned()
    })
}

/// A imagem do elemento, com o recorte (elipse, cantos…) aplicado como transparência.
fn imagem_do_elemento(el: &ElementoDesenhado) -> Option<(u64, Handle)> {
    let k = chave(el);
    IMAGENS
        .with(|c| {
            let mut c = c.borrow_mut();
            // Cada ajuste (tolerância, zoom…) gera uma entrada nova de vários MB: a versão
            // anterior do mesmo elemento é descartada.
            ULTIMA_CHAVE.with(|u| {
                if let Some(antiga) = u.borrow_mut().insert(el.id, k) {
                    if antiga != k {
                        c.remove(&antiga);
                    }
                }
            });
            if c.len() > 300 {
                c.clear();
            }
            c.entry(k).or_insert_with(|| decodificar(el).map(|img| Handle::from_rgba(img.width(), img.height(), img.into_raw()))).clone()
        })
        .map(|h| (k, h))
}

/// O pedaço da imagem que cabe na área visível: `None` se está tudo fora; senão o retângulo
/// (em pontos do canvas) e a imagem aparada (ou a inteira, se já cabe).
fn aparar_na_area(k: u64, h: Handle, r: Rectangle) -> Option<(Rectangle, Handle)> {
    let Some(a) = AREA_VISIVEL.with(|c| c.get()) else { return Some((r, h)) };
    aparar_na_regiao(k, h, r, a)
}

/// Recorta os pixels de uma foto na máscara da animação. Em Iced 0.13,
/// Canvas::Frame::with_clip não recorta confiavelmente imagens raster,
/// embora funcione com formas vetoriais. Usar o Handle já decodificado.
fn aparar_na_regiao(k: u64, h: Handle, r: Rectangle, a: Rectangle) -> Option<(Rectangle, Handle)> {
    let (x0, y0) = (r.x.max(a.x), r.y.max(a.y));
    let (x1, y1) = ((r.x + r.width).min(a.x + a.width), (r.y + r.height).min(a.y + a.height));
    if x1 - x0 < 0.5 || y1 - y0 < 0.5 {
        return None;
    }
    if x0 <= r.x && y0 <= r.y && x1 >= r.x + r.width && y1 >= r.y + r.height {
        return Some((r, h));
    }
    // O pedaço é copiado direto dos pixels do próprio `Handle` (sem guardar outra cópia da imagem).
    let Handle::Rgba { width: lw, height: lh, pixels, .. } = &h else { return Some((r, h)) };
    let (lw, lh) = (*lw, *lh);
    let (iw, ih) = (lw as f32, lh as f32);
    let px = (((x0 - r.x) / r.width) * iw).floor().clamp(0.0, iw - 1.0) as u32;
    let py = (((y0 - r.y) / r.height) * ih).floor().clamp(0.0, ih - 1.0) as u32;
    let pw = ((((x1 - r.x) / r.width) * iw).ceil() as u32).saturating_sub(px).clamp(1, lw - px);
    let ph = ((((y1 - r.y) / r.height) * ih).ceil() as u32).saturating_sub(py).clamp(1, lh - py);
    // O retângulo final é o do pedaço em pixels, para não esticar a imagem.
    let destino = Rectangle::new(
        Point::new(r.x + px as f32 / iw * r.width, r.y + py as f32 / ih * r.height),
        Size::new(pw as f32 / iw * r.width, ph as f32 / ih * r.height),
    );
    let pedaco = CORTES.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 48 {
            c.clear();
        }
        c.entry((k, px, py, pw, ph))
            .or_insert_with(|| {
                let mut recorte = Vec::with_capacity((pw * ph * 4) as usize);
                for linha in py..py + ph {
                    let i = ((linha * lw + px) * 4) as usize;
                    recorte.extend_from_slice(&pixels[i..i + (pw * 4) as usize]);
                }
                Handle::from_rgba(pw, ph, recorte)
            })
            .clone()
    });
    Some((destino, pedaco))
}

/// A imagem inteira, sem recorte nem ajuste (só a memória da tela; o arquivo não muda), e sua proporção.
fn imagem_completa(el: &ElementoDesenhado) -> Option<(Handle, f32)> {
    let mut h = DefaultHasher::new();
    el.id.hash(&mut h);
    el.imagem.len().hash(&mut h);
    el.imagem.get(..96.min(el.imagem.len())).hash(&mut h);
    let k = h.finish();
    COMPLETAS.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 4 && !c.contains_key(&k) {
            c.clear();
        }
        c.entry(k)
            .or_insert_with(|| {
                let (_, bytes) = featherslide_io::base64::de_data_url(&el.imagem)?;
                let rgba = featherslide_io::imagem_compartilhada::decodificar_raster(
                    &bytes, LADO_MAXIMO_IMAGEM,
                )?;
                let prop = rgba.width() as f32 / rgba.height().max(1) as f32;
                Some((Handle::from_rgba(rgba.width(), rgba.height(), rgba.into_raw()), prop))
            })
            .clone()
    })
}

/// Desenha a imagem inteira, esmaecida, na posição que o enquadramento (pan/zoom) lhe dá.
fn pintar_fantasma(f: &mut Frame, el: &ElementoDesenhado, ex: f32, ey: f32, ew: f32, eh: f32) {
    let Some((h, prop_img)) = imagem_completa(el) else { return };
    let pan_x = el.ajustes.first().copied().unwrap_or(0.0);
    let pan_y = el.ajustes.get(1).copied().unwrap_or(0.0);
    let zoom = el.ajustes.get(2).copied().unwrap_or(1.0).clamp(0.1, 10.0);
    let prop_el = el.largura / el.altura.max(1.0);
    let (scale_u, scale_v) = if prop_img > prop_el { (prop_el / prop_img, 1.0) } else { (1.0, prop_img / prop_el) };
    let (w, hh) = (ew * zoom / scale_u, eh * zoom / scale_v);
    let (cx, cy) = (ex + ew * (0.5 + pan_x), ey + eh * (0.5 + pan_y));
    f.draw_image(
        Rectangle::new(Point::new(cx - w / 2.0, cy - hh / 2.0), Size::new(w, hh)),
        iced::advanced::image::Image::new(h).opacity(0.35_f32),
    );
}

pub(crate) fn decodificar(el: &ElementoDesenhado) -> Option<image::RgbaImage> {
    let (_, bytes) = featherslide_io::base64::de_data_url(&el.imagem)?;
    // Mesmo raster reduzido e validado utilizado no Canvas e na saída PDF.
    let mut orig_rgba = featherslide_io::imagem_compartilhada::decodificar_raster(
        &bytes, LADO_MAXIMO_IMAGEM,
    )?;
    let tem_fundo = el.fundo_removido > 0.001;
    if tem_fundo {
        let guia = if el.fundo_refino || el.fundo_borda != 0.0 || el.fundo_pena > 0.0 { Some(orig_rgba.clone()) } else { None };
        let cor = cor_hex(&el.fundo_cor);
        match el.fundo_modo {
            1 => remover_fundo_da_borda(&mut orig_rgba, el.fundo_removido, el.fundo_suave, cor),
            2 => remover_fundo_lab(&mut orig_rgba, el.fundo_removido, el.fundo_suave, cor),
            3 => remover_fundo_watershed(&mut orig_rgba, el.fundo_removido, el.fundo_suave, cor, &el.fundo_marcas),
            4 => remover_fundo_por_cor(&mut orig_rgba, el.fundo_removido, el.fundo_suave, cor, &el.fundo_marcas),
            5 => {
                if let Some(c) = cor {
                    let alvo = [(c.r*255.0).round() as u8, (c.g*255.0).round() as u8, (c.b*255.0).round() as u8];
                    for pixel in orig_rgba.pixels_mut() {
                        if pixel.0[0..3] == alvo { pixel.0[3] = 0; }
                    }
                }
            }
            _ => remover_fundo(&mut orig_rgba, el.fundo_removido, el.fundo_suave, cor),
        }
        if let Some(guia) = guia {
            if el.fundo_refino {
                refinar_alfa(&mut orig_rgba, &guia);
            }
            ajustar_borda(&mut orig_rgba, &guia, el.fundo_borda, el.fundo_pena);
        }
    }
    crate::imagem_processamento::cores(&mut orig_rgba, &el.visual_imagem);
    let (iw, ih) = (orig_rgba.width() as f32, orig_rgba.height() as f32);

    let pan_x = el.ajustes.first().copied().unwrap_or(0.0);
    let pan_y = el.ajustes.get(1).copied().unwrap_or(0.0);
    let zoom = el.ajustes.get(2).copied().unwrap_or(1.0).clamp(0.1, 10.0);
    let margens = [
        el.ajustes.get(4).copied().unwrap_or(0.0).clamp(0.0,0.45),
        el.ajustes.get(5).copied().unwrap_or(0.0).clamp(0.0,0.45),
        el.ajustes.get(6).copied().unwrap_or(0.0).clamp(0.0,0.45),
        el.ajustes.get(7).copied().unwrap_or(0.0).clamp(0.0,0.45),
    ];
    let tem_ajuste = pan_x.abs() > 0.001 || pan_y.abs() > 0.001
        || (zoom - 1.0).abs() > 0.001 || el.visual_imagem.encaixe != 0
        || margens.iter().any(|v| *v > 0.0001);
    let tem_recorte = el.recorte != "retangulo" && !el.recorte.is_empty();

    if !tem_ajuste && !tem_recorte {
        crate::imagem_processamento::efeitos(&mut orig_rgba, &el.visual_imagem);
        return Some(orig_rgba);
    }

    let prop_el = el.largura / el.altura.max(1.0);
    let max_lado = orig_rgba.width().max(orig_rgba.height()).min(LADO_MAXIMO_IMAGEM);
    let (target_w, target_h) = if prop_el >= 1.0 {
        (max_lado, (max_lado as f32 / prop_el).round().max(1.0) as u32)
    } else {
        ((max_lado as f32 * prop_el).round().max(1.0) as u32, max_lado)
    };

    let mut dest_rgba = image::RgbaImage::new(target_w.max(1), target_h.max(1));
    let (dw, dh) = (dest_rgba.width() as f32, dest_rgba.height() as f32);
    let suave = 1.5 / dw.min(dh);

    let corte_w = (1.0-margens[0]-margens[2]).max(0.1);
    let corte_h = (1.0-margens[1]-margens[3]).max(0.1);
    let prop_img = (iw*corte_w)/(ih*corte_h).max(1.0);
    let (scale_u, scale_v) = if el.visual_imagem.encaixe == 1 {
        // Ajustar (fit): a imagem inteira cabe na forma; sobras tornam-se transparentes.
        if prop_img > prop_el { (1.0, prop_img / prop_el) }
        else { (prop_el / prop_img.max(0.0001), 1.0) }
    } else if prop_img > prop_el {
        (prop_el / prop_img, 1.0)
    } else {
        (1.0, prop_img / prop_el)
    };

    for (px, py, p) in dest_rgba.enumerate_pixels_mut() {
        let u = (px as f32 + 0.5) / dw;
        let v = (py as f32 + 0.5) / dh;
        let dentro = if tem_recorte {
            dentro_do_recorte(el, u, v, prop_el, suave)
        } else {
            1.0
        };
        if dentro <= 0.001 {
            *p = image::Rgba([0, 0, 0, 0]);
            continue;
        }

        let src_u = margens[0] + corte_w*(0.5 + (u - 0.5 - pan_x) * scale_u / zoom);
        let src_v = margens[1] + corte_h*(0.5 + (v - 0.5 - pan_y) * scale_v / zoom);

        if src_u >= 0.0 && src_u < 1.0 && src_v >= 0.0 && src_v < 1.0 {
            let sx = (src_u * iw).floor().clamp(0.0, iw - 1.0) as u32;
            let sy = (src_v * ih).floor().clamp(0.0, ih - 1.0) as u32;
            let mut cor = *orig_rgba.get_pixel(sx, sy);
            cor.0[3] = (cor.0[3] as f32 * dentro) as u8;
            *p = cor;
        } else {
            *p = image::Rgba([0, 0, 0, 0]);
        }
    }
    crate::imagem_processamento::efeitos(&mut dest_rgba, &el.visual_imagem);
    Some(dest_rgba)
}

/// A cor mais comum (`#rrggbb`) da parte da imagem que fica dentro do retângulo (x, y, w, h) do
/// slide: como no PowerPoint, o usuário seleciona uma região e a cor "padrão" dela é a escolhida.
/// Conta as cores em baldes de 16 níveis por canal e devolve a média do balde mais cheio.
pub fn cor_dominante_na_area(el: &ElementoDesenhado, x: f32, y: f32, w: f32, h: f32) -> Option<String> {
    let (_, bytes) = featherslide_io::base64::de_data_url(&el.imagem)?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
    // Uma região só de um clique (sem arrasto) vira um quadradinho em volta do ponto.
    let (x, y, w, h) = if w < 3.0 && h < 3.0 { (x - 3.0, y - 3.0, 6.0, 6.0) } else { (x, y, w, h) };
    let passos = 48;
    let mut baldes: HashMap<(u8, u8, u8), (u32, [u32; 3])> = HashMap::new();
    for i in 0..passos {
        for j in 0..passos {
            let (px, py) = (x + w * (i as f32 + 0.5) / passos as f32, y + h * (j as f32 + 0.5) / passos as f32);
            if let Some(c) = pixel_no_ponto(el, &img, px, py) {
                let e = baldes.entry((c[0] >> 4, c[1] >> 4, c[2] >> 4)).or_insert((0, [0; 3]));
                e.0 += 1;
                for k in 0..3 {
                    e.1[k] += c[k] as u32;
                }
            }
        }
    }
    let (n, soma) = baldes.values().max_by_key(|(n, _)| *n)?;
    Some(format!("#{:02x}{:02x}{:02x}", soma[0] / n, soma[1] / n, soma[2] / n))
}

/// O ponto (x, y) do slide em coordenadas da imagem (0 a 1), desfazendo o pan e o zoom do enquadramento.
fn ponto_na_imagem(el: &ElementoDesenhado, iw: f32, ih: f32, x: f32, y: f32) -> (f32, f32) {
    let (u, v) = ((x - el.x) / el.largura.max(1.0), (y - el.y) / el.altura.max(1.0));
    let pan_x = el.ajustes.first().copied().unwrap_or(0.0);
    let pan_y = el.ajustes.get(1).copied().unwrap_or(0.0);
    let zoom = el.ajustes.get(2).copied().unwrap_or(1.0).clamp(0.1, 10.0);
    let (prop_el, prop_img) = (el.largura / el.altura.max(1.0), iw / ih.max(1.0));
    let (scale_u, scale_v) = if prop_img > prop_el { (prop_el / prop_img, 1.0) } else { (1.0, prop_img / prop_el) };
    (0.5 + (u - 0.5 - pan_x) * scale_u / zoom, 0.5 + (v - 0.5 - pan_y) * scale_v / zoom)
}

/// O retângulo (x, y, w, h) do slide em coordenadas da imagem [x0, y0, x1, y1] (0 a 1), cortado
/// pelos limites da imagem; `None` se ele não toca a imagem.
pub fn retangulo_na_imagem(el: &ElementoDesenhado, x: f32, y: f32, w: f32, h: f32) -> Option<[f32; 4]> {
    let (_, bytes) = featherslide_io::base64::de_data_url(&el.imagem)?;
    let (iw, ih) = image::ImageReader::new(std::io::Cursor::new(&bytes)).with_guessed_format().ok()?.into_dimensions().ok()?;
    let (a, b) = ponto_na_imagem(el, iw as f32, ih as f32, x, y);
    let (c, d) = ponto_na_imagem(el, iw as f32, ih as f32, x + w, y + h);
    let (x0, x1, y0, y1) = (a.min(c).clamp(0.0, 1.0), a.max(c).clamp(0.0, 1.0), b.min(d).clamp(0.0, 1.0), b.max(d).clamp(0.0, 1.0));
    (x1 - x0 > 0.0005 && y1 - y0 > 0.0005).then_some([x0, y0, x1, y1])
}

/// A cor do pixel da imagem que aparece no ponto (x, y) do slide (desfaz pan e zoom do enquadramento).
fn pixel_no_ponto(el: &ElementoDesenhado, img: &image::RgbaImage, x: f32, y: f32) -> Option<[u8; 3]> {
    let (iw, ih) = (img.width() as f32, img.height() as f32);
    let (u, v) = ((x - el.x) / el.largura.max(1.0), (y - el.y) / el.altura.max(1.0));
    if !(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v) {
        return None;
    }
    let pan_x = el.ajustes.first().copied().unwrap_or(0.0);
    let pan_y = el.ajustes.get(1).copied().unwrap_or(0.0);
    let zoom = el.ajustes.get(2).copied().unwrap_or(1.0).clamp(0.1, 10.0);
    let (prop_el, prop_img) = (el.largura / el.altura.max(1.0), iw / ih.max(1.0));
    let (scale_u, scale_v) = if prop_img > prop_el { (prop_el / prop_img, 1.0) } else { (1.0, prop_img / prop_el) };
    let su = 0.5 + (u - 0.5 - pan_x) * scale_u / zoom;
    let sv = 0.5 + (v - 0.5 - pan_y) * scale_v / zoom;
    if !(0.0..1.0).contains(&su) || !(0.0..1.0).contains(&sv) {
        return None;
    }
    let p = img.get_pixel((su * iw) as u32, (sv * ih) as u32);
    Some([p.0[0], p.0[1], p.0[2]])
}

/// A cor (`#rrggbb`) da imagem no ponto (x, y) do slide, para o conta-gotas do "remover fundo".
/// Desfaz a mesma conta de enquadramento (pan/zoom) que `decodificar` faz; ignora a rotação.
pub fn cor_no_ponto(el: &ElementoDesenhado, x: f32, y: f32) -> Option<String> {
    let (_, bytes) = featherslide_io::base64::de_data_url(&el.imagem)?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let (iw, ih) = (img.width() as f32, img.height() as f32);
    let (u, v) = ((x - el.x) / el.largura.max(1.0), (y - el.y) / el.altura.max(1.0));
    if !(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v) {
        return None;
    }
    let pan_x = el.ajustes.first().copied().unwrap_or(0.0);
    let pan_y = el.ajustes.get(1).copied().unwrap_or(0.0);
    let zoom = el.ajustes.get(2).copied().unwrap_or(1.0).clamp(0.1, 10.0);
    let (prop_el, prop_img) = (el.largura / el.altura.max(1.0), iw / ih.max(1.0));
    let (scale_u, scale_v) = if prop_img > prop_el { (prop_el / prop_img, 1.0) } else { (1.0, prop_img / prop_el) };
    let su = 0.5 + (u - 0.5 - pan_x) * scale_u / zoom;
    let sv = 0.5 + (v - 0.5 - pan_y) * scale_v / zoom;
    if !(0.0..1.0).contains(&su) || !(0.0..1.0).contains(&sv) {
        return None;
    }
    let p = img.get_pixel((su * iw) as u32, (sv * ih) as u32);
    Some(format!("#{:02x}{:02x}{:02x}", p.0[0], p.0[1], p.0[2]))
}

/// "Cor para transparência": o que tem a cor do fundo (a dada, ou a dos cantos) fica
/// transparente; o que é essa cor misturada com outra fica semitransparente, na proporção da
/// mistura, e recupera a cor pura (sem halo). A tolerância faz o que está quase na cor do fundo
/// sumir de vez. Trabalha só na cópia da tela; os bytes da imagem no arquivo continuam inteiros.
fn remover_fundo(img: &mut image::RgbaImage, tolerancia: f32, suavidade: f32, cor: Option<Color>) {
    let (w, h) = (img.width(), img.height());
    if w == 0 || h == 0 {
        return;
    }
    let mut r = [0f32; 3];
    if let Some(c) = cor {
        r = [c.r * 255.0, c.g * 255.0, c.b * 255.0];
    } else {
        for (x, y) in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)] {
            let c = img.get_pixel(x, y);
            for i in 0..3 {
                r[i] += c.0[i] as f32 / 4.0;
            }
        }
    }
    // Abaixo de `piso` de mistura o pixel some inteiro.
    let piso = tolerancia.clamp(0.0, 1.0) * 0.6;
    // Largura do degradê acima do piso: pequena = corte duro, grande = borda suave.
    let largura = (0.02 + suavidade.clamp(0.0, 1.0) * 0.6).min(1.0 - piso).max(0.02);
    for p in img.pixels_mut() {
        // Quanto da cor do pixel NÃO é fundo: o menor alfa que, sobre o fundo, reproduz o pixel.
        let mut a = 0f32;
        for i in 0..3 {
            let v = p.0[i] as f32;
            let ai = if v > r[i] {
                (v - r[i]) / (255.0 - r[i]).max(1.0)
            } else if v < r[i] {
                (r[i] - v) / r[i].max(1.0)
            } else {
                0.0
            };
            a = a.max(ai);
        }
        let a = a.clamp(0.0, 1.0);
        let t = ((a - piso) / largura).clamp(0.0, 1.0);
        let alfa = t * t * (3.0 - 2.0 * t);
        if alfa <= 0.001 {
            p.0[3] = 0;
            continue;
        }
        if a < 0.999 {
            for i in 0..3 {
                p.0[i] = (r[i] + (p.0[i] as f32 - r[i]) / a).clamp(0.0, 255.0) as u8;
            }
        }
        p.0[3] = (p.0[3] as f32 * alfa) as u8;
    }
    sangrar_cor(img);
}

/// Só o fundo ligado à borda: um flood fill (BFS) a partir das quatro bordas, passando só por
/// pixels parecidos com a cor do fundo, diz ONDE agir. Nessa região (mais 2 pixels, onde mora o
/// halo de antialiasing) vale o cor para alfa, com degradê; fora dela a imagem fica intacta, então
/// o que tem a cor do fundo mas está cercado pelo desenho (jaleco, cartaz) é preservado.
fn remover_fundo_da_borda(img: &mut image::RgbaImage, tolerancia: f32, suavidade: f32, cor: Option<Color>) {
    let (w, h) = (img.width() as usize, img.height() as usize);
    if w == 0 || h == 0 {
        return;
    }
    let mut r = [0f32; 3];
    if let Some(c) = cor {
        r = [c.r * 255.0, c.g * 255.0, c.b * 255.0];
    } else {
        for (x, y) in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)] {
            let c = img.get_pixel(x as u32, y as u32);
            for i in 0..3 {
                r[i] += c.0[i] as f32 / 4.0;
            }
        }
    }
    // O fill passa um pouco além da tolerância para alcançar os pixels do halo.
    let alcance = tolerancia.clamp(0.0, 1.0) * 220.0 + 60.0;
    let perto = |p: &image::Rgba<u8>| ((0..3).map(|i| (p.0[i] as f32 - r[i]).powi(2)).sum::<f32>()).sqrt() <= alcance;

    let mut regiao = vec![false; w * h];
    let mut fila = std::collections::VecDeque::new();
    let semear = |i: usize, regiao: &mut Vec<bool>, fila: &mut std::collections::VecDeque<usize>| {
        if !regiao[i] && perto(img.get_pixel((i % w) as u32, (i / w) as u32)) {
            regiao[i] = true;
            fila.push_back(i);
        }
    };
    for x in 0..w {
        semear(x, &mut regiao, &mut fila);
        semear((h - 1) * w + x, &mut regiao, &mut fila);
    }
    for y in 0..h {
        semear(y * w, &mut regiao, &mut fila);
        semear(y * w + w - 1, &mut regiao, &mut fila);
    }
    while let Some(i) = fila.pop_front() {
        let (x, y) = (i % w, i / w);
        if x > 0 { semear(i - 1, &mut regiao, &mut fila); }
        if x + 1 < w { semear(i + 1, &mut regiao, &mut fila); }
        if y > 0 { semear(i - w, &mut regiao, &mut fila); }
        if y + 1 < h { semear(i + w, &mut regiao, &mut fila); }
    }
    // Aumenta a região em 2 pixels para pegar o halo que encosta nela.
    for _ in 0..2 {
        let antes = regiao.clone();
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if !antes[i] && ((x > 0 && antes[i - 1]) || (x + 1 < w && antes[i + 1]) || (y > 0 && antes[i - w]) || (y + 1 < h && antes[i + w])) {
                    regiao[i] = true;
                }
            }
        }
    }
    // O cor para alfa roda numa cópia; só a região leva o resultado.
    let mut tratada = img.clone();
    remover_fundo(&mut tratada, tolerancia, suavidade, cor);
    for (i, (p, t)) in img.pixels_mut().zip(tratada.pixels()).enumerate() {
        if regiao[i] {
            *p = *t;
        }
    }
}

// ------------------------------------------------------------------ LAB, watershed e filtro guiado

/// sRGB (0..255) → CIELAB (L 0..100, a e b ≈ −128..127), iluminante D65.
fn rgb_para_lab(c: [u8; 3]) -> [f32; 3] {
    let lin = |v: u8| {
        let v = v as f32 / 255.0;
        if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    let (r, g, b) = (lin(c[0]), lin(c[1]), lin(c[2]));
    let x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047;
    let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883;
    let f = |t: f32| if t > 0.008856 { t.cbrt() } else { 7.787 * t + 16.0 / 116.0 };
    let (fx, fy, fz) = (f(x), f(y), f(z));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// A cor de referência do fundo: a dada, ou a média dos quatro cantos.
fn cor_de_referencia(img: &image::RgbaImage, cor: Option<Color>) -> [u8; 3] {
    if let Some(c) = cor {
        return [(c.r * 255.0).round() as u8, (c.g * 255.0).round() as u8, (c.b * 255.0).round() as u8];
    }
    let (w, h) = (img.width(), img.height());
    let mut soma = [0u32; 3];
    for (x, y) in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)] {
        let c = img.get_pixel(x, y);
        for i in 0..3 {
            soma[i] += c.0[i] as u32;
        }
    }
    [(soma[0] / 4) as u8, (soma[1] / 4) as u8, (soma[2] / 4) as u8]
}

/// Ignorar o brilho: a distância é medida em CIELAB quase só nos canais a e b (a cor, não a
/// luminosidade), então sombras e variações de luz no fundo não deixam resíduo. A luminosidade
/// entra com peso pequeno para que um traço preto não sumisse num fundo acinzentado.
fn remover_fundo_lab(img: &mut image::RgbaImage, tolerancia: f32, suavidade: f32, cor: Option<Color>) {
    if img.width() == 0 || img.height() == 0 {
        return;
    }
    let ref_lab = rgb_para_lab(cor_de_referencia(img, cor));
    let limite = tolerancia.clamp(0.0, 1.0) * 60.0;
    let largura = 4.0 + suavidade.clamp(0.0, 1.0) * 40.0;
    for p in img.pixels_mut() {
        let lab = rgb_para_lab([p.0[0], p.0[1], p.0[2]]);
        let d = ((lab[1] - ref_lab[1]).powi(2) + (lab[2] - ref_lab[2]).powi(2) + (0.3 * (lab[0] - ref_lab[0])).powi(2)).sqrt();
        let t = ((d - limite) / largura).clamp(0.0, 1.0);
        let alfa = t * t * (3.0 - 2.0 * t);
        p.0[3] = (p.0[3] as f32 * alfa) as u8;
    }
    sangrar_cor(img);
}

/// Watershed por marcadores: o fundo certo (ligado à borda e muito parecido com a cor do fundo) e
/// o desenho certo (bem diferente dela) são as sementes; o resto é inundado na ordem do gradiente
/// (as bacias se encontram no cume do contorno). Depois a máscara ganha um degradê curto e a cor
/// do fundo misturada na borda é descontada.
fn remover_fundo_watershed(img: &mut image::RgbaImage, tolerancia: f32, suavidade: f32, cor: Option<Color>, marcas: &str) {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    let (w, h) = (img.width() as usize, img.height() as usize);
    if w < 3 || h < 3 {
        return;
    }
    let r = cor_de_referencia(img, cor);
    let rf = [r[0] as f32, r[1] as f32, r[2] as f32];
    let dist = |p: &image::Rgba<u8>| ((0..3).map(|i| (p.0[i] as f32 - rf[i]).powi(2)).sum::<f32>()).sqrt();
    let limite = tolerancia.clamp(0.0, 1.0) * 220.0;
    let (seguro_fundo, seguro_frente) = (limite * 0.6 + 8.0, limite + 70.0);

    // Gradiente (Sobel na distância ao fundo): é o "relevo" que a inundação sobe.
    let d: Vec<f32> = img.pixels().map(dist).collect();
    let mut grad = vec![0f32; w * h];
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            let gx = d[i - w + 1] + 2.0 * d[i + 1] + d[i + w + 1] - d[i - w - 1] - 2.0 * d[i - 1] - d[i + w - 1];
            let gy = d[i + w - 1] + 2.0 * d[i + w] + d[i + w + 1] - d[i - w - 1] - 2.0 * d[i - w] - d[i - w + 1];
            grad[i] = (gx * gx + gy * gy).sqrt();
        }
    }

    // Rótulos: 0 = livre, 1 = fundo, 2 = desenho. Fundo certo: ligado à borda por pixels bem parecidos.
    let mut rotulo = vec![0u8; w * h];
    let mut fila = std::collections::VecDeque::new();
    for x in 0..w {
        for i in [x, (h - 1) * w + x] {
            if rotulo[i] == 0 && d[i] <= seguro_fundo {
                rotulo[i] = 1;
                fila.push_back(i);
            }
        }
    }
    for y in 0..h {
        for i in [y * w, y * w + w - 1] {
            if rotulo[i] == 0 && d[i] <= seguro_fundo {
                rotulo[i] = 1;
                fila.push_back(i);
            }
        }
    }
    while let Some(i) = fila.pop_front() {
        let (x, y) = (i % w, i / w);
        let vizinhos = [(x > 0).then(|| i - 1), (x + 1 < w).then(|| i + 1), (y > 0).then(|| i - w), (y + 1 < h).then(|| i + w)];
        for v in vizinhos.into_iter().flatten() {
            if rotulo[v] == 0 && d[v] <= seguro_fundo {
                rotulo[v] = 1;
                fila.push_back(v);
            }
        }
    }
    for i in 0..w * h {
        if rotulo[i] == 0 && d[i] >= seguro_frente {
            rotulo[i] = 2;
        }
    }

    // As marcas do usuário mandam: o que ele marcou para manter é desenho, o que marcou para
    // remover é fundo, não importa a cor.
    for m in marcas.split(';').filter(|m| m.len() > 1) {
        let n: Vec<f32> = m[1..].split(',').filter_map(|v| v.parse().ok()).collect();
        if n.len() != 4 {
            continue;
        }
        let rot = if m.starts_with('+') { 2 } else { 1 };
        let (x0, x1) = ((n[0].min(n[2]) * w as f32).floor() as usize, ((n[0].max(n[2]) * w as f32).ceil() as usize).min(w));
        let (y0, y1) = ((n[1].min(n[3]) * h as f32).floor() as usize, ((n[1].max(n[3]) * h as f32).ceil() as usize).min(h));
        for y in y0..y1 {
            for x in x0..x1 {
                rotulo[y * w + x] = rot;
            }
        }
    }

    // Inundação por prioridade (menor gradiente primeiro), a partir de todas as sementes.
    let chave = |g: f32| (g * 16.0) as u32;
    let mut heap: BinaryHeap<Reverse<(u32, usize)>> = BinaryHeap::new();
    for i in 0..w * h {
        if rotulo[i] != 0 {
            heap.push(Reverse((chave(grad[i]), i)));
        }
    }
    while let Some(Reverse((_, i))) = heap.pop() {
        let (x, y) = (i % w, i / w);
        let vizinhos = [(x > 0).then(|| i - 1), (x + 1 < w).then(|| i + 1), (y > 0).then(|| i - w), (y + 1 < h).then(|| i + w)];
        for v in vizinhos.into_iter().flatten() {
            if rotulo[v] == 0 {
                rotulo[v] = rotulo[i];
                heap.push(Reverse((chave(grad[v]), v)));
            }
        }
    }

    // Máscara binária → alfa com um degradê curto (média 3×3, repetida conforme a suavidade) e
    // desconto da cor do fundo nos pixels de borda.
    let mut alfa: Vec<f32> = rotulo.iter().map(|&l| if l == 1 { 0.0 } else { 1.0 }).collect();
    for _ in 0..(1 + (suavidade.clamp(0.0, 1.0) * 3.0).round() as usize) {
        let c = alfa.clone();
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let i = y * w + x;
                alfa[i] = (c[i] * 4.0 + c[i - 1] + c[i + 1] + c[i - w] + c[i + w]) / 8.0;
            }
        }
    }
    for (i, p) in img.pixels_mut().enumerate() {
        let a = alfa[i];
        if a <= 0.004 {
            p.0[3] = 0;
            continue;
        }
        if a < 0.996 {
            let a_cor = a.max(0.25);
            for c in 0..3 {
                p.0[c] = (rf[c] + (p.0[c] as f32 - rf[c]) / a_cor).clamp(0.0, 255.0) as u8;
            }
        }
        p.0[3] = (p.0[3] as f32 * a).round() as u8;
    }
    sangrar_cor(img);
}

/// As marcas do usuário (`+x0,y0,x1,y1;-...`) como (manter?, retângulo em 0 a 1).
fn ler_marcas(marcas: &str) -> Vec<(bool, [f32; 4])> {
    marcas
        .split(';')
        .filter(|m| m.len() > 1)
        .filter_map(|m| {
            let n: Vec<f32> = m[1..].split(',').filter_map(|v| v.parse().ok()).collect();
            (n.len() == 4).then(|| (m.starts_with('+'), [n[0].min(n[2]), n[1].min(n[3]), n[0].max(n[2]), n[1].max(n[3])]))
        })
        .collect()
}

/// Os pixels (índices) de um retângulo em coordenadas 0 a 1 de uma imagem `w × h`.
fn pixels_do_retangulo(r: [f32; 4], w: usize, h: usize) -> impl Iterator<Item = usize> {
    let (x0, x1) = ((r[0] * w as f32).floor() as usize, ((r[2] * w as f32).ceil() as usize).min(w));
    let (y0, y1) = ((r[1] * h as f32).floor() as usize, ((r[3] * h as f32).ceil() as usize).min(h));
    (y0..y1).flat_map(move |y| (x0..x1).map(move |x| y * w + x))
}

/// Agrupa cores (CIELAB) em até `k` tons médios (k-médias, 8 rodadas, começo espalhado pela luminosidade).
fn agrupar_cores(amostras: &[[f32; 3]], k: usize) -> Vec<[f32; 3]> {
    if amostras.is_empty() {
        return Vec::new();
    }
    let mut ordem: Vec<usize> = (0..amostras.len()).collect();
    ordem.sort_by(|&a, &b| amostras[a][0].total_cmp(&amostras[b][0]));
    let k = k.min(amostras.len());
    let mut centros: Vec<[f32; 3]> = (0..k).map(|i| amostras[ordem[(i * 2 + 1) * amostras.len() / (2 * k)]]).collect();
    for _ in 0..8 {
        let mut soma = vec![[0f32; 4]; k];
        for a in amostras {
            let (j, _) = centros.iter().enumerate().map(|(j, c)| (j, (0..3).map(|i| (a[i] - c[i]).powi(2)).sum::<f32>())).min_by(|x, y| x.1.total_cmp(&y.1)).unwrap_or((0, 0.0));
            for i in 0..3 {
                soma[j][i] += a[i];
            }
            soma[j][3] += 1.0;
        }
        for j in 0..k {
            if soma[j][3] > 0.0 {
                centros[j] = [soma[j][0] / soma[j][3], soma[j][1] / soma[j][3], soma[j][2] / soma[j][3]];
            }
        }
    }
    centros
}

/// Seleção por cor das marcas: as cores do que você marcou para manter e para remover (mais a
/// moldura da imagem e a cor do fundo) viram poucos tons; cada pixel vai para o lado cujos tons
/// mais se parecem com a cor dele. Depois ficam só as regiões ligadas ao que foi marcado (ilhas
/// soltas somem), e a borda recebe um degradê curto e perde a cor do fundo misturada.
fn remover_fundo_por_cor(img: &mut image::RgbaImage, tolerancia: f32, suavidade: f32, cor: Option<Color>, marcas: &str) {
    let (w, h) = (img.width() as usize, img.height() as usize);
    if w < 3 || h < 3 {
        return;
    }
    let ref_rgb = cor_de_referencia(img, cor);
    let rf = [ref_rgb[0] as f32, ref_rgb[1] as f32, ref_rgb[2] as f32];
    let marcas = ler_marcas(marcas);
    let lab: Vec<[f32; 3]> = img.pixels().map(|p| rgb_para_lab([p.0[0], p.0[1], p.0[2]])).collect();

    // Amostras de cada lado (no máximo ~4000 por lado, em passo regular).
    let amostrar = |indices: Vec<usize>| -> Vec<[f32; 3]> {
        let passo = (indices.len() / 4000).max(1);
        indices.into_iter().step_by(passo).map(|i| lab[i]).collect()
    };
    let mut manter: Vec<usize> = marcas.iter().filter(|m| m.0).flat_map(|m| pixels_do_retangulo(m.1, w, h)).collect();
    let mut remover: Vec<usize> = marcas.iter().filter(|m| !m.0).flat_map(|m| pixels_do_retangulo(m.1, w, h)).collect();
    // Moldura da imagem (fundo quase certo) e, sem marca de manter, o que destoa bem do fundo.
    for x in 0..w {
        remover.push(x);
        remover.push((h - 1) * w + x);
    }
    for y in 0..h {
        remover.push(y * w);
        remover.push(y * w + w - 1);
    }
    let limite = tolerancia.clamp(0.0, 1.0) * 220.0;
    let dist_ref = |p: &image::Rgba<u8>| ((0..3).map(|i| (p.0[i] as f32 - rf[i]).powi(2)).sum::<f32>()).sqrt();
    if manter.is_empty() {
        manter = img.pixels().enumerate().filter(|(_, p)| dist_ref(p) > limite + 90.0).map(|(i, _)| i).collect();
    }
    let (c_manter, c_remover) = (agrupar_cores(&amostrar(manter), 4), agrupar_cores(&amostrar(remover), 4));
    if c_manter.is_empty() || c_remover.is_empty() {
        return;
    }
    let mais_perto = |c: &[f32; 3], centros: &[[f32; 3]]| centros.iter().map(|k| (0..3).map(|i| (c[i] - k[i]).powi(2)).sum::<f32>().sqrt()).fold(f32::MAX, f32::min);

    // Probabilidade de ser objeto: quanto mais perto dos tons de manter que dos de remover.
    let limiar = 0.35 + tolerancia.clamp(0.0, 1.0) * 0.6;
    let faixa = 0.04 + suavidade.clamp(0.0, 1.0) * 0.4;
    let mut alfa = vec![0f32; w * h];
    for i in 0..w * h {
        let (dm, dr) = (mais_perto(&lab[i], &c_manter), mais_perto(&lab[i], &c_remover));
        let p = if dm + dr < 1e-3 { 0.5 } else { dr / (dm + dr) };
        alfa[i] = ((p - limiar) / faixa + 0.5).clamp(0.0, 1.0);
    }
    // As marcas mandam sobre a cor.
    for (manter_, r) in &marcas {
        for i in pixels_do_retangulo(*r, w, h) {
            alfa[i] = if *manter_ { 1.0 } else { 0.0 };
        }
    }

    // Fica só o que se liga ao que foi marcado para manter (ou, sem marca, a maior região).
    let forte: Vec<bool> = alfa.iter().map(|&a| a > 0.5).collect();
    let mut dentro = vec![false; w * h];
    let mut fila = std::collections::VecDeque::new();
    let sementes: Vec<usize> = marcas.iter().filter(|m| m.0).flat_map(|m| pixels_do_retangulo(m.1, w, h)).filter(|&i| forte[i]).collect();
    let mut percorrer = |inicio: Vec<usize>, dentro: &mut Vec<bool>| {
        for i in inicio {
            if !dentro[i] {
                dentro[i] = true;
                fila.push_back(i);
            }
        }
        while let Some(i) = fila.pop_front() {
            let (x, y) = (i % w, i / w);
            for v in [(x > 0).then(|| i - 1), (x + 1 < w).then(|| i + 1), (y > 0).then(|| i - w), (y + 1 < h).then(|| i + w)].into_iter().flatten() {
                if forte[v] && !dentro[v] {
                    dentro[v] = true;
                    fila.push_back(v);
                }
            }
        }
    };
    if sementes.is_empty() {
        // A maior região conexa de "objeto": cada região é percorrida uma vez, guardando só os membros.
        let (mut visto, mut melhor) = (vec![false; w * h], Vec::<usize>::new());
        for i in 0..w * h {
            if !forte[i] || visto[i] {
                continue;
            }
            let mut membros = vec![i];
            visto[i] = true;
            let mut k = 0;
            while k < membros.len() {
                let j = membros[k];
                k += 1;
                let (x, y) = (j % w, j / w);
                for v in [(x > 0).then(|| j - 1), (x + 1 < w).then(|| j + 1), (y > 0).then(|| j - w), (y + 1 < h).then(|| j + w)].into_iter().flatten() {
                    if forte[v] && !visto[v] {
                        visto[v] = true;
                        membros.push(v);
                    }
                }
            }
            if membros.len() > melhor.len() {
                melhor = membros;
            }
        }
        for j in melhor {
            dentro[j] = true;
        }
    } else {
        percorrer(sementes, &mut dentro);
    }
    for i in 0..w * h {
        if !dentro[i] {
            alfa[i] = 0.0;
        }
    }

    // Degradê curto na borda e desconto da cor do fundo misturada.
    let c0 = alfa.clone();
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            alfa[i] = (c0[i] * 4.0 + c0[i - 1] + c0[i + 1] + c0[i - w] + c0[i + w]) / 8.0;
        }
    }
    for (i, p) in img.pixels_mut().enumerate() {
        let a = alfa[i];
        if a <= 0.004 {
            p.0[3] = 0;
            continue;
        }
        if a < 0.996 {
            let a_cor = a.max(0.25);
            for c in 0..3 {
                p.0[c] = (rf[c] + (p.0[c] as f32 - rf[c]) / a_cor).clamp(0.0, 255.0) as u8;
            }
        }
        p.0[3] = (p.0[3] as f32 * a).round() as u8;
    }
    sangrar_cor(img);
}

/// Expande (`borda` > 0) ou contrai (`borda` < 0) a parte visível em pixels e suaviza (`pena`) a
/// borda com uma média de raio `pena`. Onde a expansão revela pixels que estavam transparentes
/// vale a cor original da imagem.
fn ajustar_borda(img: &mut image::RgbaImage, original: &image::RgbaImage, borda: f32, pena: f32) {
    let (w, h) = (img.width() as usize, img.height() as usize);
    if w < 3 || h < 3 || (borda == 0.0 && pena <= 0.0) {
        return;
    }
    let mut alfa: Vec<f32> = img.pixels().map(|p| p.0[3] as f32 / 255.0).collect();
    let passos = borda.abs().round() as usize;
    for _ in 0..passos {
        let c = alfa.clone();
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let mut v = c[i];
                for n in [(x > 0).then(|| i - 1), (x + 1 < w).then(|| i + 1), (y > 0).then(|| i - w), (y + 1 < h).then(|| i + w)].into_iter().flatten() {
                    v = if borda > 0.0 { v.max(c[n]) } else { v.min(c[n]) };
                }
                alfa[i] = v;
            }
        }
    }
    if pena >= 1.0 {
        alfa = media_caixa(&alfa, w, h, pena.round() as usize);
    }
    for (i, p) in img.pixels_mut().enumerate() {
        if p.0[3] == 0 && alfa[i] > 0.0 {
            // Pixel revelado pela expansão: a cor vem da imagem original.
            let o = original.get_pixel((i % w) as u32, (i / w) as u32);
            p.0[0] = o.0[0];
            p.0[1] = o.0[1];
            p.0[2] = o.0[2];
        }
        p.0[3] = (alfa[i] * 255.0).round() as u8;
    }
}

/// Filtro de média (caixa) de raio `r` sobre um mapa `w × h`, em tempo linear (soma acumulada).
fn media_caixa(v: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    // Passada horizontal.
    let mut tmp = vec![0f32; w * h];
    for y in 0..h {
        let linha = &v[y * w..(y + 1) * w];
        let mut soma = vec![0f32; w + 1];
        for x in 0..w {
            soma[x + 1] = soma[x] + linha[x];
        }
        for x in 0..w {
            let (a, b) = (x.saturating_sub(r), (x + r + 1).min(w));
            tmp[y * w + x] = (soma[b] - soma[a]) / (b - a) as f32;
        }
    }
    // Passada vertical.
    let mut saida = vec![0f32; w * h];
    for x in 0..w {
        let mut soma = vec![0f32; h + 1];
        for y in 0..h {
            soma[y + 1] = soma[y] + tmp[y * w + x];
        }
        for y in 0..h {
            let (a, b) = (y.saturating_sub(r), (y + r + 1).min(h));
            saida[y * w + x] = (soma[b] - soma[a]) / (b - a) as f32;
        }
    }
    saida
}

/// Refino da borda por filtro guiado (He et al.): a transparência fica uma função linear local
/// da luminosidade da imagem original (a guia), então o degradê segue o traço em vez de
/// borrar. Tempo linear no número de pixels.
fn refinar_alfa(img: &mut image::RgbaImage, guia: &image::RgbaImage) {
    let (w, h) = (img.width() as usize, img.height() as usize);
    if w < 3 || h < 3 {
        return;
    }
    let i_g: Vec<f32> = guia.pixels().map(|p| (0.299 * p.0[0] as f32 + 0.587 * p.0[1] as f32 + 0.114 * p.0[2] as f32) / 255.0).collect();
    let p_a: Vec<f32> = img.pixels().map(|p| p.0[3] as f32 / 255.0).collect();
    let (r, eps) = (3, 1e-3);
    let mi = media_caixa(&i_g, w, h, r);
    let mp = media_caixa(&p_a, w, h, r);
    let ip: Vec<f32> = i_g.iter().zip(&p_a).map(|(a, b)| a * b).collect();
    let ii: Vec<f32> = i_g.iter().map(|a| a * a).collect();
    let mip = media_caixa(&ip, w, h, r);
    let mii = media_caixa(&ii, w, h, r);
    let (mut a, mut b) = (vec![0f32; w * h], vec![0f32; w * h]);
    for k in 0..w * h {
        let var = mii[k] - mi[k] * mi[k];
        let cov = mip[k] - mi[k] * mp[k];
        a[k] = cov / (var + eps);
        b[k] = mp[k] - a[k] * mi[k];
    }
    let (ma, mb) = (media_caixa(&a, w, h, r), media_caixa(&b, w, h, r));
    for (k, p) in img.pixels_mut().enumerate() {
        // Só mexe onde a transparência não é total nem nula na vizinhança (a borda).
        let q = (ma[k] * i_g[k] + mb[k]).clamp(0.0, 1.0);
        let original = p_a[k];
        if mp[k] > 0.004 && mp[k] < 0.996 {
            p.0[3] = (q * 255.0).round() as u8;
        } else {
            p.0[3] = (original * 255.0).round() as u8;
        }
    }
}

/// Os pixels transparentes levam a cor dos vizinhos visíveis. Sem isso o branco do fundo, que
/// continua guardado neles, se mistura na borda quando a imagem é reduzida/filtrada na tela
/// (a causa da "borda branca").
fn sangrar_cor(img: &mut image::RgbaImage) {
    let (w, h) = (img.width() as i64, img.height() as i64);
    for _ in 0..4 {
        let copia = img.clone();
        for y in 0..h {
            for x in 0..w {
                if copia.get_pixel(x as u32, y as u32).0[3] > 0 {
                    continue;
                }
                let (mut soma, mut n) = ([0u32; 3], 0u32);
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= w || ny >= h {
                        continue;
                    }
                    let q = copia.get_pixel(nx as u32, ny as u32);
                    if q.0[3] > 0 {
                        for i in 0..3 {
                            soma[i] += q.0[i] as u32;
                        }
                        n += 1;
                    }
                }
                if n > 0 {
                    let p = img.get_pixel_mut(x as u32, y as u32);
                    for i in 0..3 {
                        p.0[i] = (soma[i] / n) as u8;
                    }
                }
            }
        }
    }
}

/// Cobertura (0..1) do ponto (u, v) ∈ [0,1]² pela forma de recorte.
fn dentro_do_recorte(el: &ElementoDesenhado, u: f32, v: f32, proporcao: f32, suave: f32) -> f32 {
    let borda = |dist: f32| (0.5 - dist / suave).clamp(0.0, 1.0);
    match el.recorte {
        "elipse" => {
            let (dx, dy) = (u * 2.0 - 1.0, v * 2.0 - 1.0);
            // Distância aproximada até a borda, em unidades normalizadas.
            let r = (dx * dx + dy * dy).sqrt();
            borda((r - 1.0) * 0.5)
        }
        "setor" => {
            let (dx, dy) = ((u * 2.0 - 1.0), (v * 2.0 - 1.0));
            let r = (dx * dx + dy * dy).sqrt();
            let (de, ate) = (el.ajustes.first().copied().unwrap_or(0.0), el.ajustes.get(1).copied().unwrap_or(270.0));
            let mut ang = dy.atan2(dx).to_degrees();
            if ang < 0.0 {
                ang += 360.0;
            }
            let ate = if ate <= de { ate + 360.0 } else { ate };
            let dentro_ang = (ang >= de && ang <= ate) || (ang + 360.0 >= de && ang + 360.0 <= ate);
            if dentro_ang { borda((r - 1.0) * 0.5) } else { 0.0 }
        }
        "retangulo_arredondado" => {
            let raio_norm = if el.tipo == "imagem" {
                el.ajustes.get(3).copied().unwrap_or(0.16).min(0.5)
            } else {
                el.ajustes.first().copied().unwrap_or(0.16).min(0.5)
            };
            // Em unidades da menor dimensão.
            let (w, h) = if proporcao >= 1.0 { (proporcao, 1.0) } else { (1.0, 1.0 / proporcao) };
            let r = raio_norm * w.min(h);
            let (cx, cy) = (u * w - w / 2.0, v * h - h / 2.0);
            let (qx, qy) = (cx.abs() - (w / 2.0 - r), cy.abs() - (h / 2.0 - r));
            let d = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - r;
            borda(d / w.min(h))
        }
        "triangulo" => {
            // Dentro se v ≥ 2·|u − 0.5| (ápice no topo, base embaixo).
            let d = 2.0 * (u - 0.5).abs() - v;
            borda(d * 0.5)
        }
        "seta" => {
            let haste = u <= 0.6 && v >= 0.3 && v <= 0.7;
            let ponta = u >= 0.6 && (v - 0.5).abs() <= (1.0 - u) * 1.25;
            if haste || ponta { 1.0 } else { 0.0 }
        }
        _ => 1.0,
    }
}

/// As oito alças de um retângulo na tela: NO, N, NE, E, SE, S, SO, O.
pub fn posicoes_alcas(x: f32, y: f32, w: f32, h: f32) -> [Point; 8] {
    let (cx, cy) = (x + w / 2.0, y + h / 2.0);
    [
        Point::new(x, y),
        Point::new(cx, y),
        Point::new(x + w, y),
        Point::new(x + w, cy),
        Point::new(x + w, y + h),
        Point::new(cx, y + h),
        Point::new(x, y + h),
        Point::new(x, cy),
    ]
}

/// As oito alças de um retângulo girado `rotacao` graus (horário) em torno do centro dele.
pub fn posicoes_alcas_giradas(x: f32, y: f32, w: f32, h: f32, rotacao: f32) -> [Point; 8] {
    let (cx, cy) = (x + w / 2.0, y + h / 2.0);
    let (sen, cos) = rotacao.to_radians().sin_cos();
    posicoes_alcas(x, y, w, h).map(|p| {
        let (dx, dy) = (p.x - cx, p.y - cy);
        Point::new(cx + dx * cos - dy * sen, cy + dx * sen + dy * cos)
    })
}

thread_local! {
    static FAMILIAS: RefCell<HashMap<String, &'static str>> = RefCell::new(HashMap::new());
}

/// A fonte do elemento. O Iced pede o nome com vida `'static`: cada nome é guardado uma vez.
pub(crate) fn fonte_do_elemento(el: &ElementoDesenhado) -> iced::Font {
    let familia = if el.fonte.trim().is_empty() {
        iced::font::Family::SansSerif
    } else {
        let nome = FAMILIAS.with(|f| *f.borrow_mut().entry(el.fonte.clone()).or_insert_with(|| Box::leak(el.fonte.clone().into_boxed_str())));
        iced::font::Family::Name(nome)
    };
    iced::Font {
        family: familia,
        weight: if el.negrito { iced::font::Weight::Bold } else { iced::font::Weight::Normal },
        style: if el.italico { iced::font::Style::Italic } else { iced::font::Style::Normal },
        ..iced::Font::DEFAULT
    }
}

/// Pinta as primitivas de uma tabela ou gráfico. Elas foram calculadas para o retângulo `base`;
/// se o elemento está agora noutro (puxando uma alça), o desenho é esticado junto.
fn pintar_primitivas(f: &mut Frame, el: &ElementoDesenhado, ox: f32, oy: f32, escala: f32, dx: f32, dy: f32) {
    let [bx, by, bw, bh] = el.base;
    let sx = if bw.abs() > 0.01 { el.largura / bw } else { 1.0 };
    let sy = if bh.abs() > 0.01 { el.altura / bh } else { 1.0 };
    let px = |x: f32| ox + (el.x + dx + (x - bx) * sx) * escala;
    let py = |y: f32| oy + (el.y + dy + (y - by) * sy) * escala;
    let cor = |c: &str| cor_hex(c);
    // Caminhos vetoriais e compostos usam a mesma geometria neutra do
    // exportador SVG FeatherWorks. Mantemos o caminho Iced anterior como
    // fallback para estilos sem equivalência fiel no backend novo.
    let viewport_drawing = feather_draw::Viewport::de_escala_logica(
        feather_draw::Unidade::Ponto,
        escala,
        feather_draw::Ponto::novo(0.0, 0.0),
        feather_draw::Ponto::novo(ox, oy),
        1.0,
    ).ok();
    for p in &el.primitivas {
        if matches!(p, Primitiva::Vetor { .. } | Primitiva::Caminho { .. }) {
            if let (Some(vp), Ok(figura)) =
                (viewport_drawing, featherslide_io::drawing::converter_primitiva(p, false))
            {
                let deslocamento = feather_draw::Transformacao {
                    a: sx, b: 0.0, c: 0.0, d: sy,
                    tx: el.x + dx - bx * sx,
                    ty: el.y + dy - by * sy,
                };
                let figura = feather_draw::transformar(&figura, deslocamento);
                let opacidade = OPACIDADE.with(|o| o.get()).clamp(0.0, 1.0);
                if crate::drawing_iced::desenhar_figura_com_opacidade(
                    f, &figura, vp, opacidade,
                ).is_ok() {
                    continue;
                }
            }
        }
        match p {
            Primitiva::Ret { x, y, w, h, preenchimento, contorno, espessura } => {
                let (ponto, tam) = (Point::new(px(*x), py(*y)), Size::new(w * sx * escala, h * sy * escala));
                if let Some(c) = cor(preenchimento) {
                    f.fill_rectangle(ponto, tam, c);
                }
                if let Some(c) = cor(contorno) {
                    f.stroke(&Path::rectangle(ponto, tam), Stroke::default().with_color(c).with_width((espessura * escala).max(0.5)));
                }
            }
            Primitiva::Linha { x1, y1, x2, y2, cor: c, espessura } => {
                if let Some(c) = cor(c) {
                    f.stroke(&Path::line(Point::new(px(*x1), py(*y1)), Point::new(px(*x2), py(*y2))), Stroke::default().with_color(c).with_width((espessura * escala).max(0.5)));
                }
            }
            Primitiva::Setor { cx, cy, r, de, ate, preenchimento, contorno, espessura } => {
                let (centro_x, centro_y) = (px(*cx), py(*cy));
                let (rx, ry) = (r * sx.min(sy) * escala, r * sx.min(sy) * escala);
                let ate = if ate <= de { ate + 360.0 } else { *ate };
                let caminho = Path::new(|b| {
                    b.move_to(Point::new(centro_x, centro_y));
                    for i in 0..=SEGMENTOS {
                        b.line_to(ponto_na_elipse(centro_x, centro_y, rx, ry, de + (ate - de) * i as f32 / SEGMENTOS as f32));
                    }
                    b.close();
                });
                if let Some(c) = cor(preenchimento) {
                    f.fill(&caminho, c);
                }
                if let Some(c) = cor(contorno) {
                    f.stroke(&caminho, Stroke::default().with_color(c).with_width((espessura * escala).max(0.5)));
                }
            }
            Primitiva::Poli { pontos, cor: c, espessura } => {
                if let (Some(c), true) = (cor(c), pontos.len() > 1) {
                    let caminho = Path::new(|b| {
                        b.move_to(Point::new(px(pontos[0][0]), py(pontos[0][1])));
                        for q in &pontos[1..] {
                            b.line_to(Point::new(px(q[0]), py(q[1])));
                        }
                    });
                    f.stroke(&caminho, Stroke::default().with_color(c).with_width((espessura * escala).max(0.5)).with_line_cap(canvas::LineCap::Round).with_line_join(canvas::LineJoin::Round));
                }
            }
            Primitiva::Poligono { pontos, preenchimento } => {
                if let (Some(c), true) = (cor(preenchimento), pontos.len() > 2) {
                    let caminho = Path::new(|b| {
                        b.move_to(Point::new(px(pontos[0][0]), py(pontos[0][1])));
                        for q in &pontos[1..] {
                            b.line_to(Point::new(px(q[0]), py(q[1])));
                        }
                        b.close();
                    });
                    f.fill(&caminho, c);
                }
            }
            Primitiva::Vetor { segmentos, par, preenchimento, contorno, espessura } => {
                use featherslide_core::vetores::Segmento;
                let ponto = |p: &[f32;2]| Point::new(px(p[0]),py(p[1]));
                let caminho = Path::new(|b| for s in segmentos {
                    match s {
                        Segmento::M(p) => b.move_to(ponto(p)), Segmento::L(p) => b.line_to(ponto(p)),
                        Segmento::Q(a,p) => b.quadratic_curve_to(ponto(a),ponto(p)),
                        Segmento::C(a,c,p) => b.bezier_curve_to(ponto(a),ponto(c),ponto(p)),
                        Segmento::Z => b.close(),
                    }
                });
                if let Some(c) = cor(preenchimento) {
                    f.fill(&caminho, iced::widget::canvas::Fill { style: c.into(),
                        rule: if *par { iced::widget::canvas::fill::Rule::EvenOdd } else { iced::widget::canvas::fill::Rule::NonZero } });
                }
                if let Some(c) = cor(contorno) { f.stroke(&caminho, Stroke::default().with_color(c).with_width(espessura * escala)); }
            }
            Primitiva::Caminho { contornos, preenchimento, contorno, espessura } => {
                let caminho = Path::new(|b| {
                    for pontos in contornos.iter().filter(|p| p.len() > 2) {
                        b.move_to(Point::new(px(pontos[0][0]), py(pontos[0][1])));
                        for q in &pontos[1..] { b.line_to(Point::new(px(q[0]), py(q[1]))); }
                        b.close();
                    }
                });
                if let Some(c) = cor(preenchimento) { f.fill(&caminho, c); }
                if let Some(c) = cor(contorno) { f.stroke(&caminho, Stroke::default().with_color(c).with_width(espessura * escala)); }
            }
            Primitiva::Texto { x, y, tamanho, cor: c, negrito, alinh, texto } => {
                let Some(c) = cor(c) else { continue };
                f.fill_text(Text {
                    content: texto.clone(),
                    position: Point::new(px(*x), py(*y)),
                    color: c,
                    size: (tamanho * escala).max(1.0).into(),
                    font: iced::Font { weight: if *negrito { iced::font::Weight::Bold } else { iced::font::Weight::Normal }, ..iced::Font::DEFAULT },
                    horizontal_alignment: match *alinh {
                        "c" => iced::alignment::Horizontal::Center,
                        "d" => iced::alignment::Horizontal::Right,
                        _ => iced::alignment::Horizontal::Left,
                    },
                    vertical_alignment: iced::alignment::Vertical::Center,
                    ..Text::default()
                });
            }
        }
    }
}

fn ponto_na_elipse(cx: f32, cy: f32, rx: f32, ry: f32, graus: f32) -> Point {
    let a = graus.to_radians();
    Point::new(cx + rx * a.cos(), cy + ry * a.sin())
}

/// Um polígono com cada vértice arredondado: `raios` em fração do menor lado (0 = ponta reta).
fn poligono_arredondado(pontos: &[Point], raios: &[f32], menor: f32) -> Path {
    let n = pontos.len();
    let unit = |de: Point, para: Point| {
        let (dx, dy) = (para.x - de.x, para.y - de.y);
        let d = (dx * dx + dy * dy).sqrt().max(0.0001);
        (dx / d, dy / d, d)
    };
    // Cada vértice vira (ponto antes da curva, o vértice, ponto depois), ou só o vértice se o raio é 0.
    let cantos: Vec<(Point, Point, Point, f32)> = (0..n)
        .map(|i| {
            let (ant, v, prox) = (pontos[(i + n - 1) % n], pontos[i], pontos[(i + 1) % n]);
            let (ax, ay, da) = unit(v, ant);
            let (bx, by, db) = unit(v, prox);
            let r = (raios.get(i).copied().unwrap_or(0.0).clamp(0.0, 0.5) * menor).min(da / 2.0).min(db / 2.0);
            (Point::new(v.x + ax * r, v.y + ay * r), v, Point::new(v.x + bx * r, v.y + by * r), r)
        })
        .collect();
    Path::new(|b| {
        for (i, (antes, v, depois, r)) in cantos.iter().enumerate() {
            if *r < 0.5 {
                if i == 0 { b.move_to(*v) } else { b.line_to(*v) }
            } else {
                if i == 0 { b.move_to(*antes) } else { b.line_to(*antes) }
                b.quadratic_curve_to(*v, *depois);
            }
        }
        b.close();
    })
}

pub(crate) fn caminho_da_forma(tipo: &str, ex: f32, ey: f32, ew: f32, eh: f32, aj: &[f32], eh_imagem: bool) -> Path {
    if let Some(pontos) = featherslide_core::formas::pontos(tipo, ex, ey, ew, eh) {
        return Path::new(|b| {
            for (i, [x, y]) in pontos.into_iter().enumerate() {
                if i == 0 { b.move_to(Point::new(x, y)); } else { b.line_to(Point::new(x, y)); }
            }
            b.close();
        });
    }
    let menor = ew.abs().min(eh.abs());
    match tipo {
        "retangulo" if aj.len() == 4 && !eh_imagem => poligono_arredondado(&[Point::new(ex, ey), Point::new(ex + ew, ey), Point::new(ex + ew, ey + eh), Point::new(ex, ey + eh)], aj, menor),
        "retangulo_arredondado" if aj.len() == 4 && !eh_imagem => poligono_arredondado(&[Point::new(ex, ey), Point::new(ex + ew, ey), Point::new(ex + ew, ey + eh), Point::new(ex, ey + eh)], aj, menor),
        "triangulo" if aj.len() == 3 => poligono_arredondado(&[Point::new(ex + ew / 2.0, ey), Point::new(ex + ew, ey + eh), Point::new(ex, ey + eh)], aj, menor),
        "seta" if aj.len() == 7 => {
            let ponta = (ew * 0.4).min(eh);
            poligono_arredondado(
                &[
                    Point::new(ex, ey + eh * 0.3),
                    Point::new(ex + ew - ponta, ey + eh * 0.3),
                    Point::new(ex + ew - ponta, ey),
                    Point::new(ex + ew, ey + eh / 2.0),
                    Point::new(ex + ew - ponta, ey + eh),
                    Point::new(ex + ew - ponta, ey + eh * 0.7),
                    Point::new(ex, ey + eh * 0.7),
                ],
                aj,
                menor,
            )
        }
        "elipse" => Path::new(|b| {
            let (cx, cy, rx, ry) = (ex + ew / 2.0, ey + eh / 2.0, (ew / 2.0).abs(), (eh / 2.0).abs());
            b.move_to(ponto_na_elipse(cx, cy, rx, ry, 0.0));
            for i in 1..=SEGMENTOS {
                b.line_to(ponto_na_elipse(cx, cy, rx, ry, 360.0 * i as f32 / SEGMENTOS as f32));
            }
            b.close();
        }),
        "setor" => Path::new(|b| {
            let (cx, cy, rx, ry) = (ex + ew / 2.0, ey + eh / 2.0, (ew / 2.0).abs(), (eh / 2.0).abs());
            let de = aj.first().copied().unwrap_or(0.0);
            let ate = aj.get(1).copied().unwrap_or(270.0);
            let ate = if ate <= de { ate + 360.0 } else { ate };
            b.move_to(Point::new(cx, cy));
            for i in 0..=SEGMENTOS {
                b.line_to(ponto_na_elipse(cx, cy, rx, ry, de + (ate - de) * i as f32 / SEGMENTOS as f32));
            }
            b.close();
        }),
        "retangulo_arredondado" => {
            let menor = ew.abs().min(eh.abs());
            let cima = if eh_imagem { aj.get(3).copied().unwrap_or(0.16) } else { aj.first().copied().unwrap_or(0.16) };
            let baixo = if eh_imagem { cima } else { aj.get(1).copied().unwrap_or(cima) };
            let rc = (menor / 2.0).min(menor * cima);
            let rb = (menor / 2.0).min(menor * baixo);
            Path::rounded_rectangle(
                Point::new(ex, ey),
                Size::new(ew, eh),
                Radius { top_left: rc, top_right: rc, bottom_right: rb, bottom_left: rb },
            )
        }
        "triangulo" => Path::new(|b| {
            b.move_to(Point::new(ex + ew / 2.0, ey));
            b.line_to(Point::new(ex + ew, ey + eh));
            b.line_to(Point::new(ex, ey + eh));
            b.close();
        }),
        "seta" => Path::new(|b| {
            let ponta = (ew * 0.4).min(eh);
            b.move_to(Point::new(ex, ey + eh * 0.3));
            b.line_to(Point::new(ex + ew - ponta, ey + eh * 0.3));
            b.line_to(Point::new(ex + ew - ponta, ey));
            b.line_to(Point::new(ex + ew, ey + eh / 2.0));
            b.line_to(Point::new(ex + ew - ponta, ey + eh));
            b.line_to(Point::new(ex + ew - ponta, ey + eh * 0.7));
            b.line_to(Point::new(ex, ey + eh * 0.7));
            b.close();
        }),
        "linha" => Path::line(Point::new(ex, ey), Point::new(ex + ew, ey + eh)),
        _ => Path::rectangle(Point::new(ex, ey), Size::new(ew, eh)),
    }
}

/// O andamento da animação de cada elemento que a tem: 0 (ainda não entrou) a 1 (terminou).
pub type Efeitos = HashMap<u64, (f32, Animacao)>;

/// Máscara de revelação independente do tipo de objeto (vetor, texto ou foto).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mascara {
    Nenhuma,
    Surgir,
    Dividir,
    Apagar,
    Forma,
    Barras,
}

/// Um retângulo visível e o deslocamento adicional do conteúdo naquele pedaço.
type FaixaVisivel = (Rectangle, Vector);

/// Como um efeito de entrada deforma/revela o elemento num instante.
struct Transformacao {
    /// Deslocamento em pixels.
    dx: f32,
    dy: f32,
    sx: f32,
    sy: f32,
    /// O ponto fixo da escala e da rotação, em pixels.
    ancora: Point,
    opacidade: f32,
    rotacao: f32,
    mascara: Mascara,
    progresso: f32,
    direcao: Direcao,
}

/// Retângulos temporários de revelação. Com a máscara desligada ou ao final
/// do efeito, renderiza o elemento uma única vez, evitando cortes nas bordas.
fn faixas_visiveis(t: &Transformacao, ex: f32, ey: f32, ew: f32, eh: f32) -> Option<Vec<FaixaVisivel>> {
    if t.mascara == Mascara::Nenhuma || t.progresso >= 0.9999 {
        return None;
    }
    let (x, y, w, h) = (ex.min(ex+ew), ey.min(ey+eh), ew.abs().max(1.0), eh.abs().max(1.0));
    let p = t.progresso.clamp(0.0, 1.0);
    let suave = 1.0 - (1.0 - p).powi(3);
    let mut resultado = Vec::new();
    let mut adicionar = |x: f32, y: f32, w: f32, h: f32, dx: f32, dy: f32| {
        if w > 0.01 && h > 0.01 {
            resultado.push((Rectangle::new(Point::new(x,y), Size::new(w,h)), Vector::new(dx,dy)));
        }
    };
    match t.mascara {
        Mascara::Nenhuma => unreachable!("tratada acima"),
        Mascara::Forma => {
            let (cw, ch) = (w * suave, h * suave);
            adicionar(x+(w-cw)/2.0, y+(h-ch)/2.0, cw, ch, 0.0, 0.0);
        }
        Mascara::Dividir => {
            // Metades deslizam de encontro ao lugar original, aparadas no seu lado.
            adicionar(x, y, w, h/2.0, 0.0, -(1.0-suave)*h/2.0);
            adicionar(x, y+h/2.0, w, h/2.0, 0.0, (1.0-suave)*h/2.0);
        }
        Mascara::Surgir | Mascara::Apagar => {
            let f = if t.mascara == Mascara::Surgir { suave } else { p };
            match t.direcao {
                Direcao::Esquerda => adicionar(x, y, w*f, h, 0.0, 0.0),
                Direcao::Direita => adicionar(x+w*(1.0-f), y, w*f, h, 0.0, 0.0),
                Direcao::Cima => adicionar(x, y, w, h*f, 0.0, 0.0),
                Direcao::Baixo => adicionar(x, y+h*(1.0-f), w, h*f, 0.0, 0.0),
            }
        }
        Mascara::Barras => {
            // Sete faixas em sequência, não um fade único.
            let vertical = matches!(t.direcao, Direcao::Esquerda | Direcao::Direita);
            for i in 0..7 {
                let fracao = (p*7.0 - i as f32).clamp(0.0, 1.0);
                if fracao <= 0.0 { continue; }
                let j = if matches!(t.direcao, Direcao::Direita | Direcao::Baixo) { 6-i } else { i };
                if vertical {
                    let x0 = x + j as f32 * w / 7.0;
                    let x1 = x + (j+1) as f32 * w / 7.0;
                    adicionar(x0, y, x1-x0, h*fracao, 0.0, 0.0);
                } else {
                    let y0 = y + j as f32 * h / 7.0;
                    let y1 = y + (j+1) as f32 * h / 7.0;
                    adicionar(x, y0, w*fracao, y1-y0, 0.0, 0.0);
                }
            }
        }
    }
    Some(resultado)
}

/// `None`: o elemento ainda não entrou (não se vê).
fn efeito_do_elemento(p: f32, a: &Animacao, el: &ElementoDesenhado, ex: f32, ey: f32, ew: f32, eh: f32, quadro: &Quadro, escala: f32) -> Option<Transformacao> {
    let centro = Point::new(ex + ew / 2.0, ey + eh / 2.0);
    let normal = Transformacao {
        dx: 0.0, dy: 0.0, sx: 1.0, sy: 1.0, ancora: centro,
        opacidade: 1.0, rotacao: 0.0, mascara: Mascara::Nenhuma,
        progresso: p.clamp(0.0, 1.0), direcao: a.direcao,
    };
    let e = 1.0 - (1.0 - p.clamp(0.0, 1.0)).powi(3);
    match a.tipo {
        TipoAnimacao::Nenhuma => Some(normal),
        // Ênfase: sempre visível, dá um "pulso" durante o passo.
        TipoAnimacao::Pulsar => {
            let s = 1.0 + 0.10 * (std::f32::consts::PI * p.clamp(0.0, 1.0)).sin();
            Some(Transformacao { sx: s, sy: s, ..normal })
        }
        _ if p <= 0.0 => None,
        TipoAnimacao::Aparecer => Some(normal),
        TipoAnimacao::Esmaecer => Some(Transformacao { opacidade: p.clamp(0.0, 1.0), ..normal }),
        TipoAnimacao::Zoom => {
            let s = 0.3 + 0.7 * e;
            Some(Transformacao { sx: s, sy: s, opacidade: e, ..normal })
        }
        TipoAnimacao::Voar => {
            let (dx, dy) = match a.direcao {
                Direcao::Esquerda => (-(el.x + el.largura) * (1.0 - e), 0.0),
                Direcao::Direita => ((quadro.largura - el.x) * (1.0 - e), 0.0),
                Direcao::Cima => (0.0, -(el.y + el.altura) * (1.0 - e)),
                Direcao::Baixo => (0.0, (quadro.altura - el.y) * (1.0 - e)),
            };
            Some(Transformacao { dx: dx * escala, dy: dy * escala, ..normal })
        }
        TipoAnimacao::Revelar => {
            // Cresce a partir da borda de onde vem.
            let (sx, sy, ancora) = match a.direcao {
                Direcao::Esquerda => (e, 1.0, Point::new(ex, centro.y)),
                Direcao::Direita => (e, 1.0, Point::new(ex + ew, centro.y)),
                Direcao::Cima => (1.0, e, Point::new(centro.x, ey)),
                Direcao::Baixo => (1.0, e, Point::new(centro.x, ey + eh)),
            };
            Some(Transformacao { sx: sx.max(0.001), sy: sy.max(0.001), ancora, ..normal })
        }
        TipoAnimacao::Surgir => Some(Transformacao {
            mascara: Mascara::Surgir, opacidade: 0.25 + 0.75 * e, ..normal
        }),
        TipoAnimacao::Flutuar | TipoAnimacao::Deslizar => {
            let distancia = if a.tipo == TipoAnimacao::Flutuar { 65.0 } else { 125.0 };
            let v = distancia * (1.0 - e) * escala;
            let (dx,dy) = match a.direcao {
                Direcao::Esquerda => (-v,0.0), Direcao::Direita => (v,0.0),
                Direcao::Cima => (0.0,-v), Direcao::Baixo => (0.0,v),
            };
            Some(Transformacao {
                dx, dy, opacidade: if a.tipo == TipoAnimacao::Flutuar { e } else { 1.0 },
                ..normal
            })
        }
        TipoAnimacao::Dividir => Some(Transformacao { mascara: Mascara::Dividir, ..normal }),
        TipoAnimacao::Apagar => Some(Transformacao { mascara: Mascara::Apagar, ..normal }),
        TipoAnimacao::Forma => Some(Transformacao { mascara: Mascara::Forma, ..normal }),
        TipoAnimacao::Barras => Some(Transformacao { mascara: Mascara::Barras, ..normal }),
        TipoAnimacao::Roda => Some(Transformacao {
            rotacao: (1.0-e) * std::f32::consts::TAU,
            opacidade: e, sx: 0.35 + 0.65*e, sy: 0.35 + 0.65*e, ..normal
        }),
        TipoAnimacao::Girar => Some(Transformacao {
            rotacao: -(1.0-e)*std::f32::consts::FRAC_PI_2,
            opacidade: e, ..normal
        }),
        TipoAnimacao::Rebote => {
            let t = e - 1.0;
            let voltar = 1.0 + 2.70158*t.powi(3) + 1.70158*t.powi(2);
            Some(Transformacao {
                dy: (1.0-voltar)*-100.0*escala,
                sx: 0.85 + 0.15*voltar,
                sy: 0.85 + 0.15*voltar,
                ..normal
            })
        }
        TipoAnimacao::Expandir => Some(Transformacao {
            sx: (0.05 + 0.95*e).max(0.001), sy: 0.75 + 0.25*e,
            opacidade: e, ..normal
        }),
        TipoAnimacao::Contrair => Some(Transformacao {
            sx: 1.65 - 0.65*e, sy: 1.65 - 0.65*e, opacidade: e, ..normal
        }),
    }
}

/// Pinta o fundo e todos os elementos do quadro com a origem (`ox`, `oy`) e `escala`.
/// `selecao` leva contorno e alças (só no palco); `deslocamento` (em pontos do slide) é a
/// prévia do arrasto, aplicada aos elementos selecionados.
pub fn pintar_quadro(
    frame: &mut Frame,
    quadro: &Quadro,
    ox: f32,
    oy: f32,
    escala: f32,
    selecao: &[u64],
    deslocamento: (f32, f32),
    com_texto_minimo: f32,
    efeitos: Option<&Efeitos>,
) {
    let cor_fundo = cor_hex(&quadro.fundo).unwrap_or(Color::WHITE);
    frame.fill_rectangle(Point::new(ox, oy), Size::new(quadro.largura * escala, quadro.altura * escala), cor_fundo);

    for el in &quadro.elementos {
        let opacidade_antes = OPACIDADE.with(|o| o.get());
        let mut transf: Option<Transformacao> = None;
        if let Some((p, a)) = efeitos.and_then(|e| e.get(&el.id)) {
            let (x, y, w, h) = (ox + el.x * escala, oy + el.y * escala, el.largura * escala, el.altura * escala);
            match efeito_do_elemento(*p, a, el, x, y, w, h, quadro, escala) {
                Some(t) => {
                    OPACIDADE.with(|o| o.set(opacidade_antes * t.opacidade));
                    transf = Some(t);
                }
                None => continue,
            }
        }
        let mover = selecao.contains(&el.id);
        let (dx, dy) = if mover { deslocamento } else { (0.0, 0.0) };
        let ex = ox + (el.x + dx) * escala;
        let ey = oy + (el.y + dy) * escala;
        let ew = el.largura * escala;
        let eh = el.altura * escala;
        let espessura = (el.espessura * escala).max(0.5);
        let preenchimento = cor_hex(&el.preenchimento);
        let contorno = cor_hex(&el.contorno);

        let pintar_elemento = |destino: &mut Frame, desloc_extra: Vector, clip_imagem: Option<Rectangle>| {
        destino.with_save(|f| {
            if let Some(t) = &transf {
                f.translate(Vector::new(t.dx + desloc_extra.x, t.dy + desloc_extra.y));
                if t.rotacao.abs() > 0.0001 || (t.sx - 1.0).abs() > 0.0001 || (t.sy - 1.0).abs() > 0.0001 {
                    f.translate(Vector::new(t.ancora.x, t.ancora.y));
                    if t.rotacao.abs() > 0.0001 { f.rotate(t.rotacao); }
                    f.scale_nonuniform(Vector::new(t.sx, t.sy));
                    f.translate(Vector::new(-t.ancora.x, -t.ancora.y));
                }
            }
            if el.rotacao != 0.0 {
                let (cx, cy) = (ex + ew / 2.0, ey + eh / 2.0);
                f.translate(Vector::new(cx, cy));
                f.rotate(el.rotacao.to_radians());
                f.translate(Vector::new(-cx, -cy));
            }

            if el.tipo == "wordart" {
                let mut artistico = el.clone();
                artistico.primitivas = crate::texto_artistico::wordart(el);
                pintar_primitivas(f, &artistico, ox, oy, escala, dx, dy);
                return;
            }
            if el.tipo == "imagem" {
                // As sombras e reflexos decorativos não pertencem aos pixels
                // da foto; ocultá-los enquanto a máscara está se abrindo.
                if clip_imagem.is_none() {
                if let Some((handle, mx, my)) = halo_da_imagem(el) {
                    f.draw_image(Rectangle::new(
                        Point::new(ex-ew*mx, ey-eh*my),
                        Size::new(ew*(1.0+2.0*mx),eh*(1.0+2.0*my))),
                        iced::advanced::image::Image::new(handle)
                            .opacity(OPACIDADE.with(|o| o.get())*el.opacidade));
                }
                if let Some(reflexo) = reflexo_da_imagem(el) {
                    f.draw_image(Rectangle::new(
                        Point::new(ex,ey+eh+3.0*escala),Size::new(ew,eh/3.0)),
                        iced::advanced::image::Image::new(reflexo)
                            .opacity(OPACIDADE.with(|o| o.get())*el.opacidade));
                }
                if let (Some(sombra), Some(handle)) = (el.sombra.as_ref(), sombra_da_imagem(el)) {
                    f.draw_image(
                        Rectangle::new(
                            Point::new(ex + sombra.dx*escala, ey + sombra.dy*escala),
                            Size::new(ew,eh),
                        ), iced::advanced::image::Image::new(handle)
                            .opacity(OPACIDADE.with(|o| o.get()) * el.opacidade)
                    );
                }
                }
                if ENQUADRANDO.with(|e| e.get()) == Some(el.id) && clip_imagem.is_none() {
                    pintar_fantasma(f, el, ex, ey, ew, eh);
                }
                if let Some((k, h)) = imagem_do_elemento(el) {
                    let opacidade = OPACIDADE.with(|o| o.get()) * el.opacidade;
                    let caixa = Rectangle::new(Point::new(ex, ey), Size::new(ew, eh));
                    // Só a imagem reta e sem efeito de animação é aparada (a girada segue inteira).
                    let pedaco = if let Some(limite) = clip_imagem {
                        aparar_na_regiao(k, h, caixa, limite)
                    } else if el.rotacao == 0.0 && transf.is_none() {
                        aparar_na_area(k, h, caixa)
                    } else {
                        Some((caixa, h))
                    };
                    if let Some((dest, h)) = pedaco {
                        f.draw_image(dest, iced::advanced::image::Image::new(h).opacity(opacidade));
                    }
                }
                if let Some(c) = contorno {
                    let p = caminho_da_forma(if el.recorte.is_empty() { "retangulo" } else { el.recorte }, ex, ey, ew, eh, &el.ajustes, true);
                    f.stroke(&p, Stroke::default().with_color(c).with_width(espessura));
                }
            } else if el.gradiente.is_some() || el.opacidade != 1.0 || el.sombra.is_some() {
                let chave = format!("{}:{}:{}:{:?}:{:?}:{}:{}:{}", el.tipo, el.largura, el.altura, el.ajustes, el.gradiente, el.preenchimento, el.contorno, el.espessura) + &format!(":{}",el.opacidade);
                if let Some(sombra)=&el.sombra {
                    let sombra_chave=format!("{chave}:{sombra:?}");
                    let camada=SOMBRAS_FORMA.with(|cache| {
                        let mut cache=cache.borrow_mut();
                        if let Some(h)=cache.get(&sombra_chave){return Some(h.clone());}
                        let ((w,h,px),margem)=featherslide_io::pdf::raster_sombra(el)?;
                        let h=(Handle::from_rgba(w,h,px),margem);
                        if cache.len()>=8{cache.clear();}cache.insert(sombra_chave,h.clone());Some(h)
                    });
                    if let Some((h,m))=camada {
                        f.draw_image(Rectangle::new(Point::new(ex+(sombra.dx-m)*escala,ey+(sombra.dy-m)*escala),Size::new(ew+2.0*m*escala,eh+2.0*m*escala)),iced::advanced::image::Image::new(h).opacity(OPACIDADE.with(|o|o.get())));
                    }
                }
                let handle = FORMAS.with(|cache| {
                    let mut cache = cache.borrow_mut();
                    if let Some(h) = cache.get(&chave) { return Some(h.clone()); }
                    let (w,h,px) = featherslide_io::pdf::raster_forma(el)?;
                    let handle = Handle::from_rgba(w,h,px);
                    if cache.len() >= 8 { cache.clear(); }
                    cache.insert(chave,handle.clone()); Some(handle)
                });
                if let Some(handle) = handle {
                    let m=featherslide_io::pdf::margem_forma(el)*escala;
                    f.draw_image(Rectangle::new(Point::new(ex-m,ey-m),Size::new(ew+2.0*m,eh+2.0*m)),iced::advanced::image::Image::new(handle).opacity(OPACIDADE.with(|o|o.get())));
                }
            } else if !el.preenchimento.is_empty() || !el.contorno.is_empty() {
                // A geometria nativa simples passa pelo mesmo motor Drawing
                // da exportação SVG. O Canvas pai já aplica rotação/animações.
                // Em formas especiais o caminho antigo continua preservado.
                let migrou = crate::drawing_iced::desenhar_forma_basica(
                    f, el, (ox, oy), escala, (dx, dy),
                    OPACIDADE.with(|o| o.get()).clamp(0.0, 1.0),
                );
                if !migrou {
                    let p = caminho_da_forma(el.tipo, ex, ey, ew, eh, &el.ajustes, false);
                    if el.tipo != "linha" {
                        if let Some(c) = preenchimento {
                            f.fill(&p, c);
                        }
                    }
                    let cor_linha = if el.tipo == "linha" { contorno.or(preenchimento) } else { contorno };
                    if let Some(c) = cor_linha {
                        f.stroke(&p, Stroke::default().with_color(c).with_width(espessura));
                    }
                }
            }

            if !el.primitivas.is_empty() && el.gradiente.is_none() && el.opacidade == 1.0 && el.sombra.is_none() {
                pintar_primitivas(f, el, ox, oy, escala, dx, dy);
            }

            let cor_texto = if el.dica {
                Color::from_rgb8(0x9a, 0xa0, 0xa6)
            } else {
                cor_hex(&el.cor).unwrap_or(Color::from_rgb8(0x20, 0x21, 0x24))
            };
            let tam = (el.tamanho * escala).max(com_texto_minimo);
            for linha in &el.linhas {
                if linha.texto.is_empty() {
                    continue;
                }
                if let Some(fundo) = cor_hex(&el.realce) {
                    f.fill_rectangle(
                        Point::new(ox + (linha.x + dx) * escala, oy + (linha.y + dy - el.tamanho * 0.88) * escala),
                        Size::new(linha.largura * escala, el.tamanho * 1.2 * escala),
                        fundo,
                    );
                }
                if !linha.glifos.is_empty() {
                    for g in &linha.glifos {
                        f.fill_text(Text { content:g.texto.clone(), position:Point::new(ox+(g.x+dx)*escala,oy+(g.y+dy-g.tamanho*0.8)*escala), color:cor_texto, size:(g.tamanho*escala).max(com_texto_minimo).into(), font:fonte_do_elemento(el), shaping:iced::widget::text::Shaping::Advanced, ..Text::default() });
                    }
                } else {
                f.fill_text(Text {
                    content: linha.texto.clone(),
                    position: Point::new(ox + (linha.x + dx) * escala, oy + (linha.y + dy - el.tamanho * 0.8) * escala),
                    color: cor_texto,
                    size: tam.into(),
                    font: fonte_do_elemento(el),
                    // Símbolos matemáticos podem vir de outra fonte instalada; Basic não faz fallback.
                    shaping: if linha.texto.chars().any(|c| c as u32 > 255) { iced::widget::text::Shaping::Advanced } else { iced::widget::text::Shaping::Basic },
                    ..Text::default()
                });
                }
                let (lx, ly) = (ox + (linha.x + dx) * escala, oy + (linha.y + dy) * escala);
                let grossura = (el.tamanho * escala / 16.0).max(1.0);
                // Traço duplo: dois traços separados por uma espessura.
                let tracos = |base: f32, duplo: bool| if duplo { vec![base - grossura, base + grossura] } else { vec![base] };
                if el.sublinhado {
                    for y in tracos(ly + el.tamanho * escala * 0.1 + if el.sublinhado_duplo { grossura } else { 0.0 }, el.sublinhado_duplo) {
                        f.fill_rectangle(Point::new(lx, y), Size::new(linha.largura * escala, grossura), cor_texto);
                    }
                }
                if el.tachado {
                    for y in tracos(ly - el.tamanho * escala * 0.28, el.tachado_duplo) {
                        f.fill_rectangle(Point::new(lx, y), Size::new(linha.largura * escala, grossura), cor_texto);
                    }
                }
            }
        });
        };
        // O recorte é por elemento, não por slide. O frame recortado
        // usa coordenadas locais: desfazemos o deslocamento da origem.
        if let Some(faixas) = transf.as_ref().and_then(|t| faixas_visiveis(t, ex, ey, ew, eh)) {
            for (area, desloc_extra) in faixas {
                frame.with_clip(area, |parte| {
                    parte.translate(Vector::new(-area.x, -area.y));
                    // O recorte em pixels usa coordenadas do elemento ANTES
                    // da translação da metade animada (Dividir).
                    let limite = Rectangle::new(
                        Point::new(area.x - desloc_extra.x, area.y - desloc_extra.y),
                        area.size(),
                    );
                    pintar_elemento(parte, desloc_extra, Some(limite));
                });
            }
        } else {
            pintar_elemento(frame, Vector::new(0.0, 0.0), None);
        }

        if mover {
            let cor_sel = Color::from_rgb8(0x1a, 0x73, 0xe8);
            f_contorno_selecao(frame, ex, ey, ew, eh, el.rotacao, cor_sel);
        }
        OPACIDADE.with(|o| o.set(opacidade_antes));
    }
}

fn f_contorno_selecao(frame: &mut Frame, ex: f32, ey: f32, ew: f32, eh: f32, rotacao: f32, cor: Color) {
    frame.with_save(|f| {
        if rotacao != 0.0 {
            let (cx, cy) = (ex + ew / 2.0, ey + eh / 2.0);
            f.translate(Vector::new(cx, cy));
            f.rotate(rotacao.to_radians());
            f.translate(Vector::new(-cx, -cy));
        }
        f.stroke(
            &Path::rectangle(Point::new(ex - 1.0, ey - 1.0), Size::new(ew + 2.0, eh + 2.0)),
            Stroke::default().with_color(cor).with_width(1.5),
        );
    });
    // As alças ficam fora da transformação: quadrados retos, só a posição acompanha o giro.
    let r = 4.0;
    for p in posicoes_alcas_giradas(ex, ey, ew, eh, rotacao) {
        let canto = Point::new(p.x - r, p.y - r);
        frame.fill_rectangle(canto, Size::new(r * 2.0, r * 2.0), Color::WHITE);
        frame.stroke(&Path::rectangle(canto, Size::new(r * 2.0, r * 2.0)), Stroke::default().with_color(cor).with_width(1.0));
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn curvas(p: &Path) -> usize {
        p.raw().iter().filter(|e| matches!(e, iced::widget::canvas::path::lyon_path::Event::Quadratic { .. })).count()
    }

    #[test]
    fn retangulo_com_quatro_raios_tem_curvas_so_onde_ha_raio() {
        assert_eq!(curvas(&caminho_da_forma("retangulo", 0.0, 0.0, 200.0, 140.0, &[], false)), 0);
        assert_eq!(curvas(&caminho_da_forma("retangulo", 0.0, 0.0, 200.0, 140.0, &[0.2, 0.0, 0.0, 0.0], false)), 1);
        assert_eq!(curvas(&caminho_da_forma("retangulo", 0.0, 0.0, 200.0, 140.0, &[0.2, 0.2, 0.2, 0.2], false)), 4);
    }
}

#[cfg(test)]
mod testes_fundo {
    use super::*;

    /// Fundo branco com um quadrado preto e um miolo branco dentro dele (um "buraco" cercado).
    fn imagem() -> image::RgbaImage {
        let mut img = image::RgbaImage::from_pixel(40, 40, image::Rgba([255, 255, 255, 255]));
        for y in 10..30 {
            for x in 10..30 {
                img.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
            }
        }
        for y in 17..23 {
            for x in 17..23 {
                img.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
            }
        }
        img
    }

    fn alfa(img: &image::RgbaImage, x: u32, y: u32) -> u8 {
        img.get_pixel(x, y).0[3]
    }

    #[test]
    fn cor_para_alfa_tira_o_fundo_e_o_miolo() {
        let mut img = imagem();
        remover_fundo(&mut img, 0.25, 0.5, None);
        assert_eq!(alfa(&img, 2, 2), 0);
        assert_eq!(alfa(&img, 12, 12), 255, "o quadrado preto fica");
        assert_eq!(alfa(&img, 20, 20), 0, "o miolo branco também some (é a cor do fundo)");
    }

    #[test]
    fn fundo_ligado_a_borda_preserva_o_miolo() {
        let mut img = imagem();
        remover_fundo_da_borda(&mut img, 0.25, 0.5, None);
        assert_eq!(alfa(&img, 2, 2), 0);
        assert_eq!(alfa(&img, 12, 12), 255);
        assert_eq!(alfa(&img, 20, 20), 255, "o miolo cercado fica");
    }

    #[test]
    fn watershed_separa_fundo_e_desenho() {
        let mut img = imagem();
        remover_fundo_watershed(&mut img, 0.25, 0.5, None, "");
        assert_eq!(alfa(&img, 2, 2), 0);
        assert_eq!(alfa(&img, 14, 14), 255, "bem dentro do desenho fica opaco"); assert_eq!(alfa(&img, 20, 20), 255, "o miolo cercado fica");
    }

    #[test]
    fn watershed_com_marca_mantem_o_objeto_quase_igual_ao_fundo() {
        // Fundo cinza claro e um "chapéu" branco com borda de sombra: sem marca vira fundo.
        let mut img = image::RgbaImage::from_pixel(40, 40, image::Rgba([240, 240, 240, 255]));
        for y in 10..30 {
            for x in 10..30 {
                img.put_pixel(x, y, image::Rgba([252, 252, 252, 255]));
            }
        }
        for y in 10..30 {
            for x in [10, 11, 28, 29] {
                img.put_pixel(x, y, image::Rgba([225, 225, 225, 255]));
            }
        }
        let mut sem = img.clone();
        remover_fundo_watershed(&mut sem, 0.25, 0.3, None, "");
        let mut com = img.clone();
        remover_fundo_watershed(&mut com, 0.25, 0.3, None, "+0.35,0.35,0.65,0.65");
        assert_eq!(alfa(&sem, 20, 20), 0, "sem marca, o objeto branco some junto com o fundo");
        assert_eq!(alfa(&com, 20, 20), 255, "com a marca, ele fica");
        assert_eq!(alfa(&com, 2, 2), 0);
    }

    #[test]
    fn selecao_por_cor_das_marcas_separa_objeto_de_fundo() {
        // Fundo azul, objeto laranja; uma ilha laranja solta (sem marca) deve sumir.
        let mut img = image::RgbaImage::from_pixel(60, 40, image::Rgba([20, 40, 120, 255]));
        for y in 10..30 {
            for x in 10..30 {
                img.put_pixel(x, y, image::Rgba([230, 140, 40, 255]));
            }
        }
        for y in 2..6 {
            for x in 50..56 {
                img.put_pixel(x, y, image::Rgba([230, 140, 40, 255]));
            }
        }
        remover_fundo_por_cor(&mut img, 0.25, 0.3, None, "+0.35,0.35,0.45,0.65");
        assert_eq!(alfa(&img, 20, 20), 255, "o objeto marcado fica");
        assert_eq!(alfa(&img, 3, 3), 0, "o fundo some");
        assert_eq!(alfa(&img, 52, 4), 0, "a ilha solta, sem ligação com a marca, some");
    }

    #[test]
    fn ajuste_de_borda_expande_e_contrai() {
        let original = imagem();
        let mut base = imagem();
        remover_fundo(&mut base, 0.25, 0.5, None);
        let (mut mais, mut menos) = (base.clone(), base.clone());
        ajustar_borda(&mut mais, &original, 3.0, 0.0);
        ajustar_borda(&mut menos, &original, -3.0, 0.0);
        assert_eq!(alfa(&base, 8, 20), 0);
        assert_eq!(alfa(&mais, 8, 20), 255, "expandir: o objeto ganha pixels ao redor");
        assert_eq!(alfa(&menos, 11, 20), 0, "contrair: o objeto perde a borda");
        assert_eq!(alfa(&menos, 14, 14), 255, "mas o miolo continua");
    }

    #[test]
    fn lab_tira_fundo_colorido_mesmo_com_sombra() {
        // Fundo verde com uma faixa mais escura (sombra) e um quadrado vermelho.
        let mut img = image::RgbaImage::from_pixel(30, 30, image::Rgba([0, 177, 64, 255]));
        for y in 0..30 {
            for x in 0..10 {
                img.put_pixel(x, y, image::Rgba([0, 120, 44, 255]));
            }
        }
        for y in 10..20 {
            for x in 15..25 {
                img.put_pixel(x, y, image::Rgba([200, 30, 30, 255]));
            }
        }
        remover_fundo_lab(&mut img, 0.5, 0.3, Some(Color::from_rgb8(0, 177, 64)));
        assert_eq!(alfa(&img, 25, 2), 0);
        assert_eq!(alfa(&img, 3, 15), 0, "a sombra do fundo também some");
        assert_eq!(alfa(&img, 20, 15), 255, "o vermelho fica");
    }

    #[test]
    fn filtro_guiado_mantem_o_que_e_opaco_ou_transparente() {
        let guia = imagem();
        let mut img = imagem();
        remover_fundo(&mut img, 0.25, 0.5, None);
        refinar_alfa(&mut img, &guia);
        assert_eq!(alfa(&img, 1, 1), 0);
        assert_eq!(alfa(&img, 12, 12), 255);
    }
}

#[cfg(test)]
mod testes_animacoes_entrada {
    use super::*;
    use featherslide_core::modelo::{Animacao, TipoAnimacao};
    use featherslide_core::sessao::Sessao;

    #[test]
    fn novos_efeitos_tem_estado_inicial_e_final_consistentes() {
        let s = Sessao::default();
        let quadro = s.quadro(0, None, false).unwrap().unwrap();
        let el = &quadro.elementos[0];
        for tipo in TipoAnimacao::TODOS {
            if matches!(tipo, TipoAnimacao::Nenhuma | TipoAnimacao::Pulsar) { continue; }
            let a = Animacao { tipo, ..Animacao::default() };
            let inicio = efeito_do_elemento(0.0, &a, el, el.x, el.y, el.largura, el.altura, &quadro, 1.0);
            assert!(inicio.is_none(), "{tipo:?} deveria começar invisível");
            let meio = efeito_do_elemento(0.5, &a, el, el.x, el.y, el.largura, el.altura, &quadro, 1.0)
                .unwrap();
            assert!(meio.dx.is_finite() && meio.dy.is_finite() && meio.rotacao.is_finite(), "{tipo:?}");
            assert!(meio.sx > 0.0 && meio.sy > 0.0 && meio.opacidade >= 0.0, "{tipo:?}");
            let fim = efeito_do_elemento(1.0, &a, el, el.x, el.y, el.largura, el.altura, &quadro, 1.0)
                .unwrap();
            assert!((fim.dx).abs() < 0.001 && fim.dy.abs() < 0.001, "{tipo:?} não termina no lugar");
            assert!((fim.sx-1.0).abs() < 0.001 && (fim.sy-1.0).abs() < 0.001, "{tipo:?} não termina no tamanho");
            assert!((fim.opacidade-1.0).abs() < 0.001, "{tipo:?} não termina opaco");
            assert!(faixas_visiveis(&fim, 0.0, 0.0, 200.0, 100.0).is_none(),
                "{tipo:?} não pode recortar após a animação");
        }
    }


    #[test]
    fn recorte_raster_espelha_area_da_mascara_sem_esticar_a_foto() {
        use iced::widget::image::Handle;
        let p = vec![255_u8; 100 * 80 * 4];
        let foto = Handle::from_rgba(100, 80, p);
        let caixa = Rectangle::new(Point::new(20.0, 30.0), Size::new(200.0, 160.0));
        let mascara = Rectangle::new(Point::new(70.0, 70.0), Size::new(90.0, 75.0));
        let (ret, cropped) = aparar_na_regiao(999999, foto, caixa, mascara).unwrap();
        assert!(ret.x >= mascara.x - 0.1 && ret.y >= mascara.y - 0.1);
        assert!(ret.x + ret.width <= mascara.x + mascara.width + 2.0);
        assert!(ret.y + ret.height <= mascara.y + mascara.height + 2.0);
        if let Handle::Rgba { width, height, .. } = cropped {
            assert!(width < 100 && height < 80);
        } else {
            panic!("recorte não produziu pixels RGBA");
        }
    }

    #[test]
    fn mascaras_produzem_revelacoes_distintas() {
        let s = Sessao::default();
        let q = s.quadro(0, None, false).unwrap().unwrap();
        let el = &q.elementos[0];
        for tipo in [TipoAnimacao::Surgir, TipoAnimacao::Dividir, TipoAnimacao::Apagar,
            TipoAnimacao::Forma, TipoAnimacao::Barras]
        {
            let a = Animacao { tipo, ..Animacao::default() };
            let t = efeito_do_elemento(0.5, &a, el, el.x, el.y, el.largura, el.altura, &q, 1.0).unwrap();
            let regioes = faixas_visiveis(&t, 100.0, 80.0, 180.0, 120.0).unwrap();
            assert!(!regioes.is_empty(), "{tipo:?} sem máscara");
            assert!(regioes.iter().all(|(r, _)| r.x >= 99.9 && r.y >= 79.9
                && r.x+r.width <= 280.1 && r.y+r.height <= 200.1), "{tipo:?} ultrapassou a caixa");
            let areas: f32 = regioes.iter().map(|(r, _)| r.width*r.height).sum();
            assert!(areas > 0.0 && areas <= 180.0*120.0 + 0.1);
        }
    }
}
