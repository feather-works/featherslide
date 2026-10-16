//! IO-009: FSdz ZIP/Arrow -> ler -> regravar -> reler.
//! cargo run --manifest-path featherslide/crates/featherslide-io/Cargo.toml --example io009_roundtrip -- /tmp/io009.fsdz
use featherslide_core::{modelo::Tipo, sessao::Sessao};
use featherslide_io::fsdz;

fn executar(destino: &std::path::Path) -> Result<(), String> {
    let mut sessao = Sessao::default();
    sessao.apresentacao_mut().titulo = "Apresentação ⚡ UTF-8".into();
    sessao.apresentacao_mut().autor = "Equipe FeatherOffice".into();
    let id = sessao.apresentacao().slides[0].elementos[0].id;
    sessao.texto_definir(id, "Relatório de energia: ação e café")?;
    sessao.slide_novo("em_branco")?;
    sessao.elemento_inserir(Tipo::Retangulo, Some([12.0, 24.0, 160.0, 95.0]), None)?;
    sessao.slide_fundo("#fef7e0", false)?;
    let original = sessao.apresentacao().clone();

    let bytes = fsdz::gravar(&original)?;
    if !bytes.starts_with(b"PK\x03\x04") {
        return Err("FSdz não começa com cabeçalho ZIP".into());
    }
    let recuperada = fsdz::ler(&bytes)?;
    if recuperada != original {
        return Err("FSdz alterou campos no primeiro round-trip".into());
    }
    let novamente = fsdz::gravar(&recuperada)?;
    let segunda = fsdz::ler(&novamente)?;
    if segunda != original {
        return Err("FSdz alterou campos ao regravar o arquivo".into());
    }
    if fsdz::ler(b"lixo, nao zip").is_ok() {
        return Err("FSdz aceitou bytes inválidos".into());
    }
    std::fs::write(destino, &novamente).map_err(|e| e.to_string())?;
    eprintln!("IO-009 fsdz: {} bytes, {} slides, round-trip semantico OK", novamente.len(), segunda.slides.len());
    Ok(())
}

fn main() {
    let destino = std::env::args().nth(1).unwrap_or_else(|| "/tmp/io009.fsdz".into());
    if let Err(e) = executar(std::path::Path::new(&destino)) {
        eprintln!("ERRO IO-009 fsdz: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn io009_fsdz_arquivo_e_roundtrip_real() {
        let arquivo = std::env::temp_dir().join(format!("io009-{}-slide.fsdz", std::process::id()));
        executar(&arquivo).unwrap();
        let bytes = std::fs::read(&arquivo).unwrap();
        assert!(fsdz::ler(&bytes).is_ok());
        std::fs::remove_file(arquivo).unwrap();
    }
}
