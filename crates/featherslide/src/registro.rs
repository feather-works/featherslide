//! Caixa de entrada do Console do Monitor (F12): qualquer parte do aplicativo (inclusive outra
//! thread, o pânico) deixa uma linha `[origem] texto — N ms` aqui, e o `App` a entrega ao
//! `feather_ui::monitor` no próximo tique (`recolher`).

use std::sync::Mutex;

/// Teto enquanto ninguém recolhe (o tique recolhe a cada segundo).
const TETO: usize = 2000;

static PENDENTES: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn registrar(texto: impl Into<String>) {
    if let Ok(mut l) = PENDENTES.lock() {
        if l.len() >= TETO {
            l.remove(0);
        }
        l.push(texto.into());
    }
}

/// As linhas registradas desde a última chamada.
pub fn recolher() -> Vec<String> {
    PENDENTES.lock().map(|mut l| std::mem::take(&mut *l)).unwrap_or_default()
}

#[cfg(test)]
mod testes {
    #[test]
    fn recolher_entrega_uma_vez() {
        super::registrar("[teste] a");
        assert!(super::recolher().iter().any(|l| l == "[teste] a"));
        assert!(!super::recolher().iter().any(|l| l == "[teste] a"));
    }
}
