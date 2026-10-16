use featherslide_core::{
    dados::Primitiva,
    sessao::Sessao,
    vetores::{self, Segmento},
    Formatacao,
};
use featherslide_io::{fsdz, pdf, svg};

const DESENHO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
<g transform="translate(10 5)"><path d="M0 0 C20 0 20 40 40 40" fill="none" stroke="#123456"/>
<path d="M50 0H90V40H50Z M60 10H80V30H60Z" fill-rule="evenodd" fill="#ff0000" fill-opacity="0.5"/></g></svg>"##;

#[test]
fn curvas_abertas_furos_e_transformacoes() {
    let d = svg::importar(DESENHO).unwrap();
    assert_eq!(d.tamanho, [200.0, 100.0]);
    assert_eq!(d.elementos.len(), 2);
    assert_eq!(d.elementos[0].x, 10.0);
    assert_eq!(d.elementos[0].y, 5.0);
    let (par, s) = vetores::decodificar(&d.elementos[0]).unwrap();
    assert!(!par);
    assert!(s.iter().any(|s| matches!(s, Segmento::C(..))));
    assert!(!s.iter().any(|s| matches!(s, Segmento::Z)));
    assert!(vetores::decodificar(&d.elementos[1]).unwrap().0);
    assert_eq!(d.elementos[1].preenchimento, "#ff000080");
}

#[test]
fn editar_converter_desfazer_redimensionar_e_persistir() {
    let d = svg::importar(DESENHO).unwrap();
    let mut s = Sessao::default();
    let antes = s.slide().elementos.len();
    s.svg_inserir(d.elementos, d.tamanho).unwrap();
    let ids = s.estado().selecao;
    assert_eq!(ids.len(), 2);
    assert!(s
        .slide()
        .elementos
        .iter()
        .filter(|e| ids.contains(&e.id))
        .all(|e| e.grupo > 0));
    s.svg_converter().unwrap();
    assert!(s
        .slide()
        .elementos
        .iter()
        .filter(|e| ids.contains(&e.id))
        .all(|e| e.grupo == 0));
    s.desfazer().unwrap();
    s.refazer().unwrap();
    s.formatar(Formatacao::Preenchimento("#00ff00".into()))
        .unwrap();
    s.formatar(Formatacao::Contorno("#0000ff".into())).unwrap();
    let e = s
        .slide()
        .elementos
        .iter()
        .find(|e| ids.contains(&e.id))
        .unwrap();
    let dados = e.dados.clone();
    let mut maior = e.clone();
    maior.largura *= 100.0;
    maior.altura *= 100.0;
    let original = vetores::decodificar(e).unwrap().1;
    let ampliado = vetores::decodificar(&maior).unwrap().1;
    assert_eq!(dados, maior.dados);
    assert_eq!(original.len(), ampliado.len());
    let Segmento::C(_, _, p) = original[1] else {
        panic!("curva perdida")
    };
    let Segmento::C(_, _, q) = ampliado[1] else {
        panic!("curva perdida")
    };
    assert!((q[0] - maior.x - (p[0] - e.x) * 100.0).abs() < 0.01);
    let ap = fsdz::ler(&fsdz::gravar(s.apresentacao()).unwrap()).unwrap();
    let reaberto = ap.slides[s.atual()]
        .elementos
        .iter()
        .find(|e| e.dados == dados)
        .unwrap();
    assert_eq!(reaberto.preenchimento, "#00ff00");
    assert_eq!(reaberto.contorno, "#0000ff");
    assert_eq!(vetores::decodificar(reaberto).unwrap().1, original);
    while s.desfazer().is_some() {}
    assert_eq!(s.slide().elementos.len(), antes);
}

#[test]
fn exportacao_svg_roundtrip_e_pdf_com_bezier_e_regra_par() {
    let d = svg::importar(DESENHO).unwrap();
    let texto = svg::exportar(&d.elementos, d.tamanho).unwrap();
    assert!(texto.contains("evenodd"));
    assert!(texto.contains("C"));
    let volta = svg::importar(texto.as_bytes()).unwrap();
    assert_eq!(d.elementos.len(), volta.elementos.len());
    for (a, b) in d.elementos.iter().zip(&volta.elementos) {
        assert_eq!(a.preenchimento, b.preenchimento);
        assert_eq!(a.contorno, b.contorno);
        assert_eq!(
            vetores::decodificar(a).unwrap(),
            vetores::decodificar(b).unwrap()
        );
    }
    let mut s = Sessao::default();
    s.svg_inserir(d.elementos, d.tamanho).unwrap();
    let q = s.quadro(s.atual(), None, false).unwrap().unwrap();
    assert!(q.elementos.iter().any(|e| e
        .primitivas
        .iter()
        .any(|p| matches!(p, Primitiva::Vetor { .. }))));
    let bytes = pdf::exportar("SVG", &[q], &|_| None);
    use std::io::Read;
    let inicio = bytes.windows(7).position(|w| w == b"stream\n").unwrap() + 7;
    let mut conteudo = String::new();
    flate2::read::ZlibDecoder::new(&bytes[inicio..])
        .read_to_string(&mut conteudo)
        .unwrap();
    assert!(conteudo.contains("f*\n"));
    assert!(conteudo.contains(" c\n"));
}

#[test]
fn recusa_recursos_sem_conversao_e_dtd() {
    for s in [
        "<svg><text>texto</text></svg>",
        "<svg><image href='/etc/passwd'/></svg>",
        "<!DOCTYPE svg><svg/>",
        "<svg><linearGradient/></svg>",
    ] {
        assert!(svg::importar(s.as_bytes()).is_err());
    }
    assert!(svg::importar(b"<svg xmlns='http://www.w3.org/2000/svg' width='20' height='20'><rect width='10' height='10'/></svg>").is_ok());
}
