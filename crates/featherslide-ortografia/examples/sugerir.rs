//! `cargo run --release --example sugerir -- palavra...`: as sugestões do dicionário real e quanto levou (RECEITA.md §6).
fn main() {
    let c = featherslide_ortografia::Corretor::carregar(&[]).expect("dicionário");
    for p in std::env::args().skip(1) {
        let t = std::time::Instant::now();
        let s = c.sugestoes(&p);
        println!("{p} → {s:?} ({:.0} ms)", t.elapsed().as_secs_f64() * 1000.0);
    }
}
