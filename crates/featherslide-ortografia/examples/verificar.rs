//! `cargo run --release --example verificar -- arquivo.(fsdz|pptx)`: os erros achados e quanto levou (RECEITA.md §6).
fn main() {
    let caminho = std::env::args().nth(1).expect("passe o arquivo");
    let bytes = std::fs::read(&caminho).expect("não consegui ler");
    let ap = if caminho.ends_with(".fsdz") { featherslide_io::fsdz::ler(&bytes) } else { featherslide_io::pptx::ler(&bytes) }.expect("não consegui abrir");
    let t = std::time::Instant::now();
    let c = featherslide_ortografia::Corretor::carregar(&[]).expect("dicionário");
    println!("[ortografia] dicionários carregados em {:.0} ms", t.elapsed().as_secs_f64() * 1000.0);
    let t = std::time::Instant::now();
    let erros = c.verificar(&ap);
    println!("[ortografia] {} erros em {:.0} ms", erros.len(), t.elapsed().as_secs_f64() * 1000.0);
    for e in erros.iter().take(25) {
        println!("  slide {} «{}» → {:?}", e.slide + 1, e.palavra, c.sugestoes(&e.palavra));
    }
}
