//! Os modelos do usuário: apresentações `.fsdz` guardadas numa pasta própria. Criar = gravar a
//! apresentação atual ali; importar = trazer um `.fsdz` (ou `.pptx`/`.odp`, convertido) de fora;
//! exportar = copiar para onde a pessoa quiser. Tudo local, sem conta nem nuvem.

use std::path::{Path, PathBuf};

use featherslide_core::modelo::Apresentacao;
use featherslide_core::quadro::Quadro;
use featherslide_core::sessao::Sessao;
use featherslide_io::{fsdz, odp, pptx};

#[derive(Clone, Debug)]
pub struct MeuModelo {
    pub nome: String,
    pub caminho: PathBuf,
    /// O primeiro slide, para o cartão do carrossel.
    pub quadro: Option<Quadro>,
}

/// `~/.local/share/featherslide/modelos` (ou `XDG_DATA_HOME`).
pub fn pasta() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(base.join("featherslide").join("modelos"))
}

/// O nome vira nome de arquivo: letras, números, espaço, hífen e sublinhado.
fn nome_de_arquivo(nome: &str) -> String {
    let limpo: String = nome.trim().chars().map(|c| if c.is_alphanumeric() || matches!(c, ' ' | '-' | '_') { c } else { '_' }).collect();
    let limpo = limpo.trim().chars().take(60).collect::<String>();
    if limpo.is_empty() { "Modelo".into() } else { limpo }
}

/// Um caminho livre na pasta: `Nome.fsdz`, `Nome 2.fsdz`…
fn caminho_livre(nome: &str) -> Result<PathBuf, String> {
    let pasta = pasta().ok_or("Não achei a pasta de dados do usuário.")?;
    std::fs::create_dir_all(&pasta).map_err(|e| format!("Não consegui criar a pasta de modelos: {e}"))?;
    let base = nome_de_arquivo(nome);
    let mut n = 1;
    loop {
        let candidato = pasta.join(if n == 1 { format!("{base}.fsdz") } else { format!("{base} {n}.fsdz") });
        if !candidato.exists() {
            return Ok(candidato);
        }
        n += 1;
    }
}

pub fn abrir(caminho: &Path) -> Result<Apresentacao, String> {
    let bytes = std::fs::read(caminho).map_err(|e| format!("Não consegui ler o modelo: {e}"))?;
    fsdz::ler(&bytes)
}

pub fn listar() -> Vec<MeuModelo> {
    let Some(pasta) = pasta() else { return Vec::new() };
    let Ok(entradas) = std::fs::read_dir(pasta) else { return Vec::new() };
    let mut v: Vec<MeuModelo> = entradas
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("fsdz")))
        .map(|caminho| {
            let nome = caminho.file_stem().unwrap_or_default().to_string_lossy().to_string();
            let quadro = abrir(&caminho).ok().and_then(|ap| Sessao::nova(ap).quadro(0, None, false).ok().flatten());
            MeuModelo { nome, caminho, quadro }
        })
        .collect();
    v.sort_by(|a, b| a.nome.to_lowercase().cmp(&b.nome.to_lowercase()));
    v
}

/// Grava a apresentação como modelo `nome`.
pub fn criar(nome: &str, ap: &Apresentacao) -> Result<PathBuf, String> {
    let caminho = caminho_livre(nome)?;
    let bytes = fsdz::gravar(ap)?;
    std::fs::write(&caminho, bytes).map_err(|e| format!("Não consegui gravar o modelo: {e}"))?;
    Ok(caminho)
}

/// Traz um arquivo de fora para a pasta de modelos (`.pptx`/`.odp` viram `.fsdz`).
pub fn importar(origem: &Path) -> Result<PathBuf, String> {
    let bytes = std::fs::read(origem).map_err(|e| format!("Não consegui ler o arquivo: {e}"))?;
    let ext = origem.extension().unwrap_or_default().to_string_lossy().to_lowercase();
    let ap = match ext.as_str() {
        "pptx" => pptx::ler(&bytes).or_else(|e| fsdz::ler(&bytes).map_err(|_| e))?,
        "odp" => odp::ler(&bytes)?,
        _ => fsdz::ler(&bytes)?,
    };
    let nome = origem.file_stem().unwrap_or_default().to_string_lossy().to_string();
    criar(&nome, &ap)
}

pub fn exportar(modelo: &Path, destino: &Path) -> Result<(), String> {
    let mut destino = destino.to_path_buf();
    if destino.extension().map_or(true, |e| !e.eq_ignore_ascii_case("fsdz")) {
        destino.set_extension("fsdz");
    }
    std::fs::copy(modelo, &destino).map(|_| ()).map_err(|e| format!("Não consegui exportar o modelo: {e}"))
}

pub fn remover(modelo: &Path) -> Result<(), String> {
    std::fs::remove_file(modelo).map_err(|e| format!("Não consegui remover o modelo: {e}"))
}
