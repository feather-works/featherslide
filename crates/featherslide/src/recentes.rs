//! As apresentações recentes da página inicial: os caminhos ficam na configuração; o que se
//! mostra (miniatura do primeiro slide, nome e data) é lido em segundo plano, porque abrir um
//! `.pptx` grande leva tempo.

use std::path::{Path, PathBuf};
use std::process::Command;

use featherslide_core::modelo::Apresentacao;
use featherslide_core::quadro::{Dicas, Quadro};
use featherslide_core::sessao::Sessao;
use featherslide_io::{fsdz, odp, pptx};

use crate::configuracao::{pasta_das_apresentacoes, Configuracao};

const MAXIMO: usize = 12;

#[derive(Clone, Debug)]
pub struct Recente {
    pub caminho: PathBuf,
    pub nome: String,
    /// "Modificado hoje 12:04" ou "Modificado em 07/10/2026 12:04".
    pub data: String,
    pub quadro: Option<Quadro>,
}

fn aceito(p: &Path) -> bool {
    matches!(p.extension().and_then(|e| e.to_str()).map(str::to_lowercase).as_deref(), Some("fsdz" | "pptx" | "odp"))
}

/// Os caminhos a mostrar: os recentes gravados (que ainda existem) e, se faltar, os arquivos
/// da pasta `Documentos/FeatherSlide`, do mais novo para o mais velho.
pub fn caminhos(config: &Configuracao) -> Vec<PathBuf> {
    let mut lista: Vec<PathBuf> = config.recentes.iter().map(PathBuf::from).filter(|p| p.is_file()).collect();
    if lista.len() < MAXIMO {
        if let Some(pasta) = pasta_das_apresentacoes() {
            let mut achados: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(pasta)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_file() && aceito(p))
                .filter_map(|p| Some((std::fs::metadata(&p).ok()?.modified().ok()?, p)))
                .collect();
            achados.sort_by(|a, b| b.0.cmp(&a.0));
            for (_, p) in achados {
                if !lista.contains(&p) && lista.len() < MAXIMO {
                    lista.push(p);
                }
            }
        }
    }
    lista.truncate(MAXIMO);
    lista
}

/// Lê o arquivo (`.fsdz`, `.pptx` ou `.odp`).
pub fn ler_arquivo(p: &Path) -> Result<Apresentacao, String> {
    let bytes = std::fs::read(p).map_err(|e| e.to_string())?;
    match p.extension().and_then(|e| e.to_str()).map(str::to_lowercase).as_deref() {
        Some("pptx") => pptx::ler(&bytes),
        Some("odp") => odp::ler(&bytes),
        _ => fsdz::ler(&bytes),
    }
}

fn data_local(segundos: u64, formato: &str) -> Option<String> {
    let saida = Command::new("date").arg("-d").arg(format!("@{segundos}")).arg(format!("+{formato}")).output().ok()?;
    saida.status.success().then(|| String::from_utf8_lossy(&saida.stdout).trim().to_string())
}

fn descrever_data(p: &Path) -> String {
    let Some(segundos) = std::fs::metadata(p).ok().and_then(|m| m.modified().ok()).and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()) else {
        return String::new();
    };
    let agora = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (dia, hora) = (data_local(segundos, "%d/%m/%Y"), data_local(segundos, "%H:%M"));
    match (dia, hora) {
        (Some(d), Some(h)) if Some(&d) == data_local(agora, "%d/%m/%Y").as_ref() => format!("Modificado hoje {h}"),
        (Some(d), Some(h)) => format!("Modificado em {d} {h}"),
        _ => String::new(),
    }
}

/// Carrega nome, data e a miniatura do primeiro slide de cada arquivo (bloqueia: rode em segundo plano).
pub fn carregar(caminhos: Vec<PathBuf>) -> Vec<Recente> {
    caminhos
        .into_iter()
        .map(|caminho| {
            let quadro = ler_arquivo(&caminho).ok().and_then(|ap| {
                let mut s = Sessao::nova(ap);
                s.definir_dicas(Dicas::default());
                s.quadro(0, None, false).ok().flatten()
            });
            Recente {
                nome: caminho.file_name().unwrap_or_default().to_string_lossy().to_string(),
                data: descrever_data(&caminho),
                caminho,
                quadro,
            }
        })
        .collect()
}
