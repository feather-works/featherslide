//! `cargo run --example icones_pdf -- saida.pdf`: uma página com todos os ícones, para conferir o desenho (RECEITA.md §6).

use featherslide_core::icones::buscar;
use featherslide_core::sessao::Sessao;

fn main() {
    let saida = std::env::args().nth(1).expect("passe o .pdf de saída");
    let mut s = Sessao::default();
    s.slide_novo("em_branco").unwrap();
    for (i, ic) in buscar("", 64).iter().enumerate() {
        s.icone_inserir(ic.id).unwrap();
        let e = s.estado().selecao[0];
        let (x, y) = (30.0 + (i % 8) as f32 * 112.0, 20.0 + (i / 8) as f32 * 62.0);
        s.elemento_geometria(e, [x, y, 56.0, 56.0]).unwrap();
    }
    let q = s.quadro(1, None, false).unwrap().unwrap();
    std::fs::write(&saida, featherslide_io::pdf::exportar("Ícones", &[q], &|_| None)).unwrap();
}
