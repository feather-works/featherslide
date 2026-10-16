//! `cargo run --example ler_pptx -- arquivo.pptx`: o que o leitor entendeu de cada slide,
//! e quanto tempo levou. Para conferir com arquivo real (RECEITA.md §6).

fn main() {
    let caminho = std::env::args().nth(1).expect("passe o caminho do .pptx");
    let bytes = std::fs::read(&caminho).expect("não consegui ler o arquivo");
    let inicio = std::time::Instant::now();
    let ap = featherslide_io::pptx::ler(&bytes).expect("não consegui ler o .pptx");
    println!("[abrir] {} slides em {:.1} ms ({:?})", ap.slides.len(), inicio.elapsed().as_secs_f64() * 1000.0, ap.proporcao);
    println!("[fontes] {} embutidas: {:?}", ap.fontes.len(), ap.fontes.iter().map(|f| format!("{} (reg={}, neg={}, ita={})", f.nome, f.regular.is_some(), f.negrito.is_some(), f.italico.is_some())).collect::<Vec<_>>());
    for (i, s) in ap.slides.iter().enumerate() {
        println!("— slide {} layout={} fundo={} oculto={} notas={:?}", i + 1, s.layout, s.fundo, s.oculto, s.notas.chars().take(40).collect::<String>());
        for e in &s.elementos {
            let t: String = e.texto.replace('\n', " ⏎ ").chars().take(70).collect();
            println!(
                "   {:?}/{:?} [{:.0},{:.0} {:.0}×{:.0}] fill={} ln={} {}pt {} {}{} cor={} {:?}",
                e.tipo, e.papel, e.x, e.y, e.largura, e.altura, e.preenchimento, e.contorno, e.estilo.tamanho, e.estilo.fonte,
                if e.estilo.negrito { "N" } else { "" }, if e.estilo.italico { "I" } else { "" }, e.estilo.cor, t
            );
        }
    }
}
