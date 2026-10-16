//! `cargo run --release --example palavra -- palavra...`: o dicionário real aceita? (RECEITA.md §6)
fn main() {
    let c = featherslide_ortografia::Corretor::carregar(&[]).expect("dicionário");
    for p in std::env::args().skip(1) {
        println!("{p}: {}", if c.palavra_ok(&p) { "aceita" } else { "ERRO" });
    }
}
