//! Renderização de um slide em imagem PNG para o servidor MCP (`slide_ver`).
//! Utiliza `tiny-skia` para formas geométricas e `ab_glyph` para textos, em Rust 100% puro.

use featherslide_core::quadro::{ElementoDesenhado, Quadro};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

use crate::fonte::{self, Fonte, CENTRO_MEIO, ESQUERDA_BASE};

/// Decodifica uma cor hexadecimal (`#rrggbb` ou `#rrggbbaa`) para `tiny_skia::Color` e `[u8; 3]`.
pub fn cor_hex(s: &str, padrao: [u8; 4]) -> (Color, [u8; 3]) {
    let limpo = s.trim_start_matches('#');
    let mut rgba = padrao;
    if limpo.len() == 6 {
        if let Ok(v) = u32::from_str_radix(limpo, 16) {
            rgba[0] = ((v >> 16) & 0xff) as u8;
            rgba[1] = ((v >> 8) & 0xff) as u8;
            rgba[2] = (v & 0xff) as u8;
            rgba[3] = 255;
        }
    } else if limpo.len() == 8 {
        if let Ok(v) = u32::from_str_radix(limpo, 16) {
            rgba[0] = ((v >> 24) & 0xff) as u8;
            rgba[1] = ((v >> 16) & 0xff) as u8;
            rgba[2] = ((v >> 8) & 0xff) as u8;
            rgba[3] = (v & 0xff) as u8;
        }
    }
    let a = f32::from(rgba[3]) / 255.0;
    let cor = Color::from_rgba(f32::from(rgba[0]) / 255.0, f32::from(rgba[1]) / 255.0, f32::from(rgba[2]) / 255.0, a).unwrap_or(Color::WHITE);
    (cor, [rgba[0], rgba[1], rgba[2]])
}

fn desenhar_elipse(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32) {
    let rx = (w / 2.0).abs();
    let ry = (h / 2.0).abs();
    let cx = x + rx;
    let cy = y + ry;
    let k = 0.552_284_8;
    let ox = rx * k;
    let oy = ry * k;

    pb.move_to(cx - rx, cy);
    pb.cubic_to(cx - rx, cy - oy, cx - ox, cy - ry, cx, cy - ry);
    pb.cubic_to(cx + ox, cy - ry, cx + rx, cy - oy, cx + rx, cy);
    pb.cubic_to(cx + rx, cy + oy, cx + ox, cy + ry, cx, cy + ry);
    pb.cubic_to(cx - ox, cy + ry, cx - rx, cy + oy, cx - rx, cy);
    pb.close();
}

fn pintar_forma(pix: &mut Pixmap, elem: &ElementoDesenhado, escala: f32, tr: Transform) {
    if elem.primitivas.iter().any(|p| matches!(p, featherslide_core::dados::Primitiva::Vetor { .. })) {
        pintar_vetores(pix, elem, escala, tr);
        return;
    }
    let w = elem.largura * escala;
    let h = elem.altura * escala;
    if w <= 0.0 && h <= 0.0 {
        return;
    }

    if elem.tipo == "imagem" && !elem.imagem.is_empty() {
        if let Some((_, bytes)) = featherslide_io::base64::de_data_url(&elem.imagem) {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let mut rgba = img.to_rgba8();
                let (iw, ih) = (rgba.width(), rgba.height());
                if !elem.recorte.is_empty() && elem.recorte != "retangulo" {
                    let (rw, rh) = (iw as f32, ih as f32);
                    let suave = 1.5 / rw.min(rh);
                    for (px, py, p) in rgba.enumerate_pixels_mut() {
                        let u = (px as f32 + 0.5) / rw;
                        let v = (py as f32 + 0.5) / rh;
                        let dentro = match elem.recorte {
                            "elipse" => {
                                let (dx, dy) = (u * 2.0 - 1.0, v * 2.0 - 1.0);
                                let r = (dx * dx + dy * dy).sqrt();
                                (0.5 - (r - 1.0) * 0.5 / suave).clamp(0.0, 1.0)
                            }
                            "retangulo_arredondado" => {
                                let r = 0.16 * rw.min(rh);
                                let (cx, cy) = (u * rw - rw / 2.0, v * rh - rh / 2.0);
                                let (qx, qy) = (cx.abs() - (rw / 2.0 - r), cy.abs() - (rh / 2.0 - r));
                                let d = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - r;
                                (0.5 - d / (rw.min(rh) * suave)).clamp(0.0, 1.0)
                            }
                            "triangulo" => {
                                let d = 2.0 * (u - 0.5).abs() - v;
                                (0.5 - d * 0.5 / suave).clamp(0.0, 1.0)
                            }
                            _ => 1.0,
                        };
                        p.0[3] = (p.0[3] as f32 * dentro) as u8;
                    }
                }
                if let Some(mut img_pix) = Pixmap::new(iw, ih) {
                    img_pix.data_mut().copy_from_slice(&rgba);
                    let sx = w / iw as f32;
                    let sy = h / ih as f32;
                    pix.draw_pixmap(0, 0, img_pix.as_ref(), &tiny_skia::PixmapPaint::default(), tr.pre_scale(sx, sy), None);
                }
            }
        }
        if !elem.contorno.is_empty() && elem.espessura > 0.0 {
            let mut pb = PathBuilder::new();
            if elem.recorte == "elipse" {
                desenhar_elipse(&mut pb, 0.0, 0.0, w, h);
            } else if let Some(r) = Rect::from_xywh(0.0, 0.0, w, h) {
                pb.push_rect(r);
            }
            if let Some(caminho) = pb.finish() {
                let (cor, _) = cor_hex(&elem.contorno, [0, 0, 0, 255]);
                let mut paint = Paint::default();
                paint.set_color(cor);
                let stroke = Stroke { width: elem.espessura * escala, ..Stroke::default() };
                pix.stroke_path(&caminho, &paint, &stroke, tr, None);
            }
        }
        return;
    }

    // Formas básicas nativas: o quadro leva cor/contorno nas primitivas, não na caixa.
    // Reaproveita exatamente os mesmos caminhos de Iced/PDF (inclusive furos).
    if let Some(tipo) = featherslide_core::modelo::Tipo::pelo_nome(elem.tipo) {
        if let Some(d) = featherslide_core::novas_formas::desenho(tipo, 0.0, 0.0, w, h) {
            let cores = elem.primitivas.iter().find_map(|p| match p {
                featherslide_core::dados::Primitiva::Caminho { preenchimento, contorno, espessura, .. } =>
                    Some((preenchimento.as_str(), contorno.as_str(), *espessura)),
                featherslide_core::dados::Primitiva::Poli { cor, espessura, .. } =>
                    Some(("", cor.as_str(), *espessura)),
                _ => None,
            });
            let (preenchimento, contorno, espessura) = cores.unwrap_or(("", "", 0.0));
            if !d.contornos.is_empty() {
                let mut pb = PathBuilder::new();
                for pontos in &d.contornos {
                    if let Some([x0, y0]) = pontos.first().copied() {
                        pb.move_to(x0, y0);
                        for [px, py] in &pontos[1..] { pb.line_to(*px, *py); }
                        pb.close();
                    }
                }
                if let Some(path) = pb.finish() {
                    if !preenchimento.is_empty() {
                        let (cor, _) = cor_hex(preenchimento, [66, 133, 244, 255]);
                        let mut tinta = Paint::default();
                        tinta.set_color(cor);
                        pix.fill_path(&path, &tinta, FillRule::Winding, tr, None);
                    }
                    if !contorno.is_empty() && espessura > 0.0 {
                        let (cor, _) = cor_hex(contorno, [0, 0, 0, 255]);
                        let mut tinta = Paint::default();
                        tinta.set_color(cor);
                        let stroke = Stroke { width: espessura * escala, ..Stroke::default() };
                        pix.stroke_path(&path, &tinta, &stroke, tr, None);
                    }
                }
            }
            if !contorno.is_empty() && espessura > 0.0 {
                let (cor, _) = cor_hex(contorno, [0, 0, 0, 255]);
                let mut tinta = Paint::default();
                tinta.set_color(cor);
                let stroke = Stroke { width: espessura * escala, ..Stroke::default() };
                for pontos in d.tracos {
                    let mut pb = PathBuilder::new();
                    if let Some([x0,y0]) = pontos.first().copied() {
                        pb.move_to(x0, y0);
                        for [px,py] in &pontos[1..] { pb.line_to(*px, *py); }
                        if let Some(path) = pb.finish() {
                            pix.stroke_path(&path, &tinta, &stroke, tr, None);
                        }
                    }
                }
            }
            return;
        }
    }
    let mut pb = PathBuilder::new();
    if let Some(pontos) = featherslide_core::formas::pontos(&elem.tipo, 0.0, 0.0, w, h) {
        for (i, [x, y]) in pontos.into_iter().enumerate() {
            if i == 0 { pb.move_to(x, y); } else { pb.line_to(x, y); }
        }
        pb.close();
    } else if elem.tipo == "elipse" {
        desenhar_elipse(&mut pb, 0.0, 0.0, w, h);
    } else if elem.tipo == "linha" {
        pb.move_to(0.0, 0.0);
        pb.line_to(w, h);
    } else if let Some(r) = Rect::from_xywh(0.0, 0.0, w, h) {
        pb.push_rect(r);
    }
    let Some(caminho) = pb.finish() else { return };

    if !elem.preenchimento.is_empty() && elem.tipo != "linha" {
        let (cor, _) = cor_hex(&elem.preenchimento, [255, 255, 255, 255]);
        let mut paint = Paint::default();
        paint.set_color(cor);
        pix.fill_path(&caminho, &paint, FillRule::Winding, tr, None);
    }

    if !elem.contorno.is_empty() && elem.espessura > 0.0 {
        let (cor, _) = cor_hex(&elem.contorno, [0, 0, 0, 255]);
        let mut paint = Paint::default();
        paint.set_color(cor);
        let stroke = Stroke { width: elem.espessura * escala, ..Stroke::default() };
        pix.stroke_path(&caminho, &paint, &stroke, tr, None);
    }
}

fn pintar_vetores(pix: &mut Pixmap, elem: &ElementoDesenhado, escala: f32, tr: Transform) {
    use featherslide_core::{dados::Primitiva, vetores::Segmento};
    let p = |p: &[f32;2]| [(p[0]-elem.x)*escala,(p[1]-elem.y)*escala];
    for primitiva in &elem.primitivas {
        let Primitiva::Vetor { segmentos,par,preenchimento,contorno,espessura } = primitiva else { continue };
        let mut b=PathBuilder::new();
        for s in segmentos { match s {
            Segmento::M(a) => { let a=p(a); b.move_to(a[0],a[1]); },
            Segmento::L(a) => { let a=p(a); b.line_to(a[0],a[1]); },
            Segmento::Q(a,c) => { let a=p(a); let c=p(c); b.quad_to(a[0],a[1],c[0],c[1]); },
            Segmento::C(a,c,d) => { let a=p(a); let c=p(c); let d=p(d); b.cubic_to(a[0],a[1],c[0],c[1],d[0],d[1]); },
            Segmento::Z => b.close(),
        }}
        let Some(caminho)=b.finish() else {continue};
        if !preenchimento.is_empty() {
            let mut tinta=Paint::default(); tinta.set_color(cor_hex(preenchimento,[0,0,0,255]).0);
            pix.fill_path(&caminho,&tinta,if *par {FillRule::EvenOdd} else {FillRule::Winding},tr,None);
        }
        if !contorno.is_empty() && *espessura>0.0 {
            let mut tinta=Paint::default(); tinta.set_color(cor_hex(contorno,[0,0,0,255]).0);
            pix.stroke_path(&caminho,&tinta,&Stroke{width:espessura*escala,..Default::default()},tr,None);
        }
    }
}

/// As peças de tabela, gráfico e ícone (em pontos do slide), na escala da imagem.
fn pintar_primitivas(pix: &mut Pixmap, fonte: &Fonte, primitivas: &[featherslide_core::dados::Primitiva], escala: f32) {
    use featherslide_core::dados::Primitiva;
    let tinta = |cor: &str| {
        let mut t = Paint::default();
        t.set_color(cor_hex(cor, [0, 0, 0, 255]).0);
        t.anti_alias = true;
        t
    };
    let encher_e_tracar = |pix: &mut Pixmap, caminho: &tiny_skia::Path, preenchimento: &str, contorno: &str, espessura: f32, regra: FillRule| {
        if !preenchimento.is_empty() {
            pix.fill_path(caminho, &tinta(preenchimento), regra, Transform::identity(), None);
        }
        if !contorno.is_empty() && espessura > 0.0 {
            pix.stroke_path(caminho, &tinta(contorno), &Stroke { width: espessura * escala, ..Default::default() }, Transform::identity(), None);
        }
    };
    let poligono = |pontos: &[[f32; 2]], fechar: bool| {
        let mut b = PathBuilder::new();
        for (i, q) in pontos.iter().enumerate() {
            if i == 0 { b.move_to(q[0] * escala, q[1] * escala) } else { b.line_to(q[0] * escala, q[1] * escala) }
        }
        if fechar {
            b.close();
        }
        b.finish()
    };
    for p in primitivas {
        match p {
            Primitiva::Ret { x, y, w, h, preenchimento, contorno, espessura } => {
                if let Some(r) = Rect::from_xywh(x * escala, y * escala, w.max(0.01) * escala, h.max(0.01) * escala) {
                    encher_e_tracar(pix, &PathBuilder::from_rect(r), preenchimento, contorno, *espessura, FillRule::Winding);
                }
            }
            Primitiva::Linha { x1, y1, x2, y2, cor, espessura } => {
                if let Some(c) = poligono(&[[*x1, *y1], [*x2, *y2]], false) {
                    encher_e_tracar(pix, &c, "", cor, *espessura, FillRule::Winding);
                }
            }
            Primitiva::Poli { pontos, cor, espessura } => {
                if let Some(c) = poligono(pontos, false) {
                    encher_e_tracar(pix, &c, "", cor, *espessura, FillRule::Winding);
                }
            }
            Primitiva::Poligono { pontos, preenchimento } => {
                if let Some(c) = poligono(pontos, true) {
                    encher_e_tracar(pix, &c, preenchimento, "", 0.0, FillRule::Winding);
                }
            }
            Primitiva::Caminho { contornos, preenchimento, contorno, espessura } => {
                let mut b = PathBuilder::new();
                for anel in contornos.iter().filter(|a| a.len() > 2) {
                    for (i, q) in anel.iter().enumerate() {
                        if i == 0 { b.move_to(q[0] * escala, q[1] * escala) } else { b.line_to(q[0] * escala, q[1] * escala) }
                    }
                    b.close();
                }
                if let Some(c) = b.finish() {
                    encher_e_tracar(pix, &c, preenchimento, contorno, *espessura, FillRule::EvenOdd);
                }
            }
            Primitiva::Setor { cx, cy, r, de, ate, preenchimento, contorno, espessura } => {
                // Graus no sentido horário a partir das 3 horas; volta inteira = círculo.
                let varre = (ate - de).rem_euclid(360.0);
                let inteiro = varre > 359.0 || varre == 0.0;
                let passos = ((if inteiro { 360.0 } else { varre }) / 4.0).ceil().max(2.0) as usize;
                let mut pontos = Vec::with_capacity(passos + 2);
                if !inteiro {
                    pontos.push([*cx, *cy]);
                }
                for i in 0..=passos {
                    let g = (de + (if inteiro { 360.0 } else { varre }) * i as f32 / passos as f32).to_radians();
                    pontos.push([cx + r * g.cos(), cy + r * g.sin()]);
                }
                if let Some(c) = poligono(&pontos, true) {
                    encher_e_tracar(pix, &c, preenchimento, contorno, *espessura, FillRule::Winding);
                }
            }
            Primitiva::Texto { x, y, tamanho, cor, alinh, texto, .. } => {
                let h = match *alinh { "c" => 1, "d" => 2, _ => 0 };
                fonte::escrever(pix, fonte, texto, x * escala, y * escala, tamanho * escala, 0.0, cor_hex(cor, [0, 0, 0, 255]).1, fonte::Ancora { h, v: 2 }, false);
            }
            Primitiva::Vetor { .. } => {}
        }
    }
}

fn desenhar_rotulo(pix: &mut Pixmap, fonte: &Fonte, id: u64, x: f32, y: f32) {
    let texto_id = format!("#{id}");
    let larg_badge = (texto_id.len() as f32 * 7.0 + 8.0).max(18.0);
    let mut pb = PathBuilder::new();
    if let Some(r) = Rect::from_xywh(x, y, larg_badge, 14.0) {
        pb.push_rect(r);
    }
    if let Some(caminho) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(26, 115, 232, 220));
        pix.fill_path(&caminho, &paint, FillRule::Winding, Transform::identity(), None);
    }
    fonte::escrever(pix, fonte, &texto_id, x + larg_badge / 2.0, y + 7.0, 10.0, 0.0, [255, 255, 255], CENTRO_MEIO, false);
}

fn desenhar_linhas(pix: &mut Pixmap, fonte: &Fonte, elem: &ElementoDesenhado, escala: f32) {
    let (_, rgb) = cor_hex(&elem.cor, [0, 0, 0, 255]);
    for linha in &elem.linhas {
        let lx = linha.x * escala;
        let ly = linha.y * escala;
        let tam = elem.tamanho * escala;
        fonte::escrever(pix, fonte, &linha.texto, lx, ly, tam, 0.0, rgb, ESQUERDA_BASE, false);
    }
}

/// Renderiza o `Quadro` em imagem PNG com a largura dada em pixels e retorna os bytes do PNG.
pub fn renderizar_png(quadro: &Quadro, largura_px: u32, rotulos: bool, fonte: &Fonte) -> Result<Vec<u8>, String> {
    let largura_px = largura_px.clamp(320, 3840);
    let escala = largura_px as f32 / quadro.largura.max(1.0);
    let altura_px = ((quadro.altura * escala).round() as u32).max(180);

    let mut pix = Pixmap::new(largura_px, altura_px).ok_or_else(|| "Não foi possível alocar o Pixmap".to_string())?;

    let (cor_fundo, _) = cor_hex(&quadro.fundo, [255, 255, 255, 255]);
    pix.fill(cor_fundo);

    for elem in &quadro.elementos {
        let ex = elem.x * escala;
        let ey = elem.y * escala;
        let tr = Transform::from_translate(ex, ey);
        if elem.primitivas.iter().any(|p| !matches!(p, featherslide_core::dados::Primitiva::Vetor { .. })) {
            // Tabela, gráfico e ícone: o quadro já traz as peças prontas; a caixa não se pinta.
            pintar_primitivas(&mut pix, fonte, &elem.primitivas, escala);
        } else {
            pintar_forma(&mut pix, elem, escala, tr);
        }
        desenhar_linhas(&mut pix, fonte, elem, escala);
        if rotulos && !elem.dica {
            desenhar_rotulo(&mut pix, fonte, elem.id, ex, ey);
        }
    }

    pix.encode_png().map_err(|e| format!("Falha ao codificar PNG: {e}"))
}

#[cfg(test)]
mod testes_primitivas {
    use super::*;

    #[test]
    fn grafico_e_tabela_aparecem_na_previa() {
        let mut s = featherslide_core::sessao::Sessao::default();
        s.slide_novo("em_branco").unwrap();
        s.grafico_inserir("rosca").unwrap();
        let q = s.quadro(1, None, false).unwrap().unwrap();
        let png = renderizar_png(&q, 640, false, crate::fonte::da_sessao()).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgb8();
        // O azul da primeira fatia (#1a73e8) tem de aparecer em algum lugar.
        assert!(img.pixels().any(|p| p.0[2] > 200 && p.0[0] < 60 && p.0[1] > 90 && p.0[1] < 140), "a rosca não foi desenhada");
    }
}
