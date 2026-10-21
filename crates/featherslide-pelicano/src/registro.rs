//! O registro de métricas e mensagens do aplicativo: o que o F12 (Monitor) mostra no Console e
//! nas Métricas. Linhas no formato `[origem] texto — N ms`, como as do FeatherSheet.

use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

const TETO: usize = 2000;

#[derive(Clone, Debug)]
pub struct Linha {
    /// Segundos desde que o aplicativo abriu.
    pub quando: f32,
    pub texto: String,
}

static LOG: Mutex<VecDeque<Linha>> = Mutex::new(VecDeque::new());
static INICIO: OnceLock<Instant> = OnceLock::new();

pub fn iniciar() {
    INICIO.get_or_init(Instant::now);
}

pub fn registrar(texto: impl Into<String>) {
    let quando = INICIO.get_or_init(Instant::now).elapsed().as_secs_f32();
    if let Ok(mut l) = LOG.lock() {
        if l.len() >= TETO {
            l.pop_front();
        }
        l.push_back(Linha { quando, texto: texto.into() });
    }
}

pub fn copia() -> Vec<Linha> {
    LOG.lock().map(|l| l.iter().cloned().collect()).unwrap_or_default()
}

pub fn limpar() {
    if let Ok(mut l) = LOG.lock() {
        l.clear();
    }
}

/// Uma linha de métrica lida: origem entre colchetes, o resto e o primeiro tempo em ms.
pub struct Metrica {
    pub origem: String,
    pub texto: String,
    pub ms: Option<f64>,
}

pub fn ler_metrica(linha: &str) -> Option<Metrica> {
    let resto = linha.strip_prefix('[')?;
    let (origem, texto) = resto.split_once(']')?;
    if origem.is_empty() || origem.contains(char::is_whitespace) && origem.trim().is_empty() {
        return None;
    }
    let texto = texto.trim().to_string();
    let ms = texto.split_whitespace().collect::<Vec<_>>().windows(2).find_map(|p| {
        if p[1].trim_end_matches(|c: char| !c.is_alphanumeric()) == "ms" {
            p[0].replace(',', ".").parse::<f64>().ok()
        } else {
            None
        }
    });
    Some(Metrica { origem: origem.to_string(), texto, ms })
}
