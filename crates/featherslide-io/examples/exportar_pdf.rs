//! `cargo run --example exportar_pdf -- entrada.(pptx|fsdz) saida.pdf`: exporta e mede (RECEITA.md §6).

use featherslide_core::sessao::Sessao;

fn main() {
    let mut args = std::env::args().skip(1);
    let (entrada, saida) = (args.next().expect("passe o arquivo de entrada"), args.next().expect("passe o .pdf de saída"));
    let bytes = std::fs::read(&entrada).expect("não consegui ler a entrada");
    let ap = if entrada.ends_with(".fsdz") { featherslide_io::fsdz::ler(&bytes) } else { featherslide_io::pptx::ler(&bytes) }.expect("não consegui abrir");
    let s = Sessao::nova(ap);
    let inicio = std::time::Instant::now();
    let quadros: Vec<_> = (0..s.apresentacao().slides.len()).filter(|&i| !s.apresentacao().slides[i].oculto).filter_map(|i| s.quadro(i, None, false).ok().flatten()).collect();
    let imagem = |e: &featherslide_core::quadro::ElementoDesenhado| {
        let (_, b) = featherslide_io::base64::de_data_url(&e.imagem)?;
        let img = image::load_from_memory(&b).ok()?.to_rgba8();
        Some((img.width(), img.height(), img.into_raw()))
    };
    let pdf = featherslide_io::pdf::exportar(&s.apresentacao().titulo, &quadros, &imagem);
    std::fs::write(&saida, &pdf).expect("não consegui gravar");
    println!("[pdf] {} páginas, {} KB em {:.0} ms", quadros.len(), pdf.len() / 1024, inicio.elapsed().as_secs_f64() * 1000.0);
}
