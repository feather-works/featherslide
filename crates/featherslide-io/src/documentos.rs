//! Arquivos de apresentação e versões preservadas ao salvar.
use crate::{arquivo::gravar_atomico, fsdz, odp, pptx};
use featherslide_core::modelo::Apresentacao;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Abre um arquivo nativo, PPTX ou ODP pelo caminho.
pub fn ler(path: &Path) -> Result<Apresentacao, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("Não consegui abrir {}: {e}", path.display()))?;
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "pptx" => pptx::ler(&bytes),
        "odp" => odp::ler(&bytes),
        _ => fsdz::ler(&bytes),
    }
}
fn aceito(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase()
            .as_str(),
        "fsdz" | "pptx" | "odp"
    )
}
/// Filtra os nomes dos arquivos sem diferenciar maiúsculas e minúsculas.
pub fn pesquisar(lista: &[PathBuf], consulta: &str) -> Vec<PathBuf> {
    let consulta = consulta.to_lowercase();
    lista
        .iter()
        .filter(|p| {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
                .contains(&consulta)
        })
        .cloned()
        .collect()
}

/// Lista apresentações de um diretório em ordem de nome.
pub fn listar(pasta: &Path) -> Vec<PathBuf> {
    let mut lista: Vec<_> = std::fs::read_dir(pasta)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && aceito(p))
        .collect();
    lista.sort();
    lista
}
/// Diretório de versões associado ao nome do arquivo, na mesma pasta.
pub fn pasta_historico(path: &Path) -> PathBuf {
    let mut nome = std::ffi::OsString::from(".");
    nome.push(path.file_name().unwrap_or_default());
    nome.push(".versoes");
    path.with_file_name(nome)
}
fn id() -> String {
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{:020}-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        std::process::id(),
        N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}
/// Preserva a versão anterior antes da substituição atômica do arquivo atual.
pub fn salvar(ap: &Apresentacao, path: &Path) -> Result<usize, String> {
    let bytes = fsdz::gravar(ap)?;
    if path.exists() {
        let anteriores = std::fs::read(path).map_err(|e| e.to_string())?;
        if anteriores != bytes {
            fsdz::ler(&anteriores).map_err(|_| {
                "O destino não é uma apresentação .fsdz; escolha outro arquivo.".to_string()
            })?;
            let pasta = pasta_historico(path);
            std::fs::create_dir_all(&pasta).map_err(|e| e.to_string())?;
            gravar_atomico(&pasta.join(format!("{}.fsdz", id())), &anteriores)?;
        }
    }
    gravar_atomico(path, &bytes)?;
    Ok(bytes.len())
}
/// Lista versões anteriores, da mais recente para a mais antiga.
pub fn historico(path: &Path) -> Vec<PathBuf> {
    let mut lista = listar(&pasta_historico(path));
    lista.reverse();
    lista
}
