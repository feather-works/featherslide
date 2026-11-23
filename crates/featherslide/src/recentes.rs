//! As apresentações recentes da página inicial: os caminhos ficam na configuração; o que se
//! mostra (miniatura do primeiro slide, nome e data) é lido em segundo plano, porque abrir um
//! `.pptx` grande leva tempo.

use std::path::{Path, PathBuf};

use featherslide_core::quadro::{Dicas, Quadro};
use featherslide_core::sessao::Sessao;

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

fn descrever_data(p: &Path) -> String {
    let Some(segundos) = std::fs::metadata(p).ok().and_then(|m| m.modified().ok()).and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()) else {
        return String::new();
    };
    let agora = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (d, h) = crate::data_local::formatar(segundos as i64);
    if d == crate::data_local::formatar(agora as i64).0 {
        crate::localidade::trf("Modificado hoje {}", &[&h])
    } else {
        crate::localidade::trf("Modificado em {} {}", &[&d, &h])
    }
}

/// Carrega nome, data e a miniatura do primeiro slide de cada arquivo (bloqueia: rode em segundo plano).
pub fn carregar(caminhos: Vec<PathBuf>) -> Vec<Recente> {
    caminhos
        .into_iter()
        .map(|caminho| {
            let quadro = featherslide_io::documentos::ler(&caminho).ok().and_then(|ap| {
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
