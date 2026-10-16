use featherslide_core::modelo::Tipo;
use featherslide_core::sessao::Sessao;
use featherslide_io::pdf;

fn conta(pdf: &[u8], agulha: &str) -> usize {
    String::from_utf8_lossy(pdf).matches(agulha).count()
}

#[test]
fn tabela_exporta_espessura_e_oculta_bordas_com_zero() {
    use featherslide_core::Formatacao;
    let mut s = Sessao::default();
    s.slide_novo("em_branco").unwrap();
    s.tabela_inserir(2, 3).unwrap();
    s.formatar(Formatacao::Espessura(3.5)).unwrap();
    let exportar = |s: &Sessao| {
        use std::io::Read;
        let q = s.quadro(1, None, false).unwrap().unwrap();
        let bytes = pdf::exportar("Tabela", &[q], &|_| None);
        let inicio = bytes.windows(7).position(|w| w == b"stream\n").unwrap() + 7;
        let mut conteudo = Vec::new();
        flate2::read::ZlibDecoder::new(&bytes[inicio..]).read_to_end(&mut conteudo).unwrap();
        conteudo
    };
    assert_eq!(conta(&exportar(&s), "RG 3.5 w"), 6);
    s.formatar(Formatacao::Espessura(0.0)).unwrap();
    assert_eq!(conta(&exportar(&s), " RG "), 0);
}

#[test]
fn pdf_tem_uma_pagina_por_slide_e_offsets_certos() {
    let mut s = Sessao::default();
    s.slide_novo("titulo_conteudo").unwrap();
    s.elemento_inserir(Tipo::Elipse, Some([10.0, 20.0, 80.0, 60.0]), None).unwrap();
    s.tabela_inserir(2, 2).unwrap();
    s.grafico_inserir("pizza").unwrap();
    let quadros: Vec<_> = (0..2).map(|i| s.quadro(i, None, false).unwrap().unwrap()).collect();
    let bytes = pdf::exportar("Título com ação", &quadros, &|_| None);
    assert!(bytes.starts_with(b"%PDF-1.5"));
    assert!(bytes.ends_with(b"%%EOF\n"));
    assert_eq!(conta(&bytes, "/Type /Page "), 2);
    assert!(conta(&bytes, "/MediaBox [0 0 960 540]") == 2);
    // Cada entrada da tabela xref aponta para "N 0 obj".
    let texto = String::from_utf8_lossy(&bytes).to_string();
    let xref = texto.rfind("xref\n").unwrap();
    for (n, linha) in texto[xref..].lines().skip(3).take_while(|l| l.ends_with(" n ")).enumerate() {
        let pos: usize = linha[..10].parse().unwrap();
        assert!(bytes[pos..].starts_with(format!("{} 0 obj", n + 1).as_bytes()), "objeto {} fora do lugar", n + 1);
    }
}

#[test]
fn configuracao_slide_exporta_pagina_retrato_e_fundo_gradiente() {
    use featherslide_core::configuracao_slide::{Ajuste, Fundo};
    let mut s = Sessao::default();
    s.slide_novo("em_branco").unwrap();
    s.tamanho_slide_definir(400.0, 700.0, Ajuste::Ajustar).unwrap();
    s.fundo_configurar("#000000", Fundo { tipo: "gradiente".into(), ..Fundo::default() }, false, true).unwrap();
    let q = s.quadro(s.atual(), None, false).unwrap().unwrap();
    assert_eq!(q.elementos.len(), 128);
    let pdf = pdf::exportar("Retrato", &[q], &|_| None);
    assert_eq!(conta(&pdf, "/MediaBox [0 0 400 700]"), 1);
}

#[test]
fn gradiente_respeita_direcao_radial_recorte_e_transparencia() {
    use featherslide_core::estilo_forma::{Aparencia, Gradiente, TipoGradiente};
    use featherslide_core::modelo::Sombra;
    let mut s = Sessao::default();
    s.slide_novo("em_branco").unwrap();
    s.elemento_inserir(Tipo::Elipse, Some([20.0, 20.0, 100.0, 100.0]), None)
        .unwrap();
    let mut g = Gradiente::default();
    g.pontos[0].cor = "#ff0000".into();
    g.pontos[1].cor = "#0000ff".into();
    let a = |g| Aparencia {
        preenchimento: "#ff0000".into(),
        gradiente: Some(g),
        contorno: String::new(),
        espessura: 0.0,
        opacidade: 0.5,
        sombra: Sombra::default(),
    };
    s.forma_aparencia(a(g.clone())).unwrap();
    let q = s.quadro(s.atual(), None, false).unwrap().unwrap();
    let e = q.elementos.last().unwrap();
    let (w, h, px) = pdf::raster_forma(e).unwrap();
    let p = |x: u32, y: u32| &px[((y * w + x) * 4) as usize..((y * w + x) * 4 + 4) as usize];
    assert_eq!(p(0, 0)[3], 0);
    assert!(p(w / 4, h / 2)[0] > p(w * 3 / 4, h / 2)[0]);
    assert!((126..=129).contains(&p(w / 2, h / 2)[3]));
    g.angulo = 180.0;
    s.forma_aparencia(a(g.clone())).unwrap();
    let q = s.quadro(s.atual(), None, false).unwrap().unwrap();
    let (_, _, px) = pdf::raster_forma(q.elementos.last().unwrap()).unwrap();
    assert!(px[((h / 2 * w + w / 4) * 4) as usize] < px[((h / 2 * w + w * 3 / 4) * 4) as usize]);
    g.tipo = TipoGradiente::Radial;
    s.forma_aparencia(a(g)).unwrap();
    let q = s.quadro(s.atual(), None, false).unwrap().unwrap();
    let (w, h, px) = pdf::raster_forma(q.elementos.last().unwrap()).unwrap();
    assert!(px[((h / 2 * w + w / 2) * 4) as usize] > px[((h / 2 * w + w * 9 / 10) * 4) as usize]);
    let bytes = pdf::exportar("Gradiente", &[q], &|_| None);
    assert!(conta(&bytes, "/Subtype /Image") >= 2);
    assert!(conta(&bytes, "/SMask ") >= 1);
}
