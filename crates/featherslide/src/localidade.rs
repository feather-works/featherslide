//! Textos da interface em pt-BR, en-US e es-ES.
//!
//! O português é a fonte. [`tr`] recebe o texto em português ("Nova apresentação"); em inglês ou
//! espanhol procura esse português em `dados/en-US.json` / `dados/es-ES.json` (um mapa
//! português → tradução) e, se faltar, mostra o português. Os textos com chave (`ferramentas_texto`,
//! a Ajuda F1) ficam em `dados/pt-BR.json` e passam pelo mesmo `tr`. O idioma é global e a tela troca
//! na hora: as `view` chamam `tr` a cada quadro.
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};

/// Os idiomas da interface; o código é o que fica gravado na configuração.
pub const IDIOMAS: [(&str, &str); 3] = [("pt-BR", "Português (Brasil)"), ("en-US", "English"), ("es-ES", "Español")];

static LINGUA: AtomicU8 = AtomicU8::new(0);

// Nos testes, o idioma vale só na thread que o pediu: os outros testes, em paralelo, seguem em português.
#[cfg(test)]
thread_local! {
    static NA_THREAD: std::cell::Cell<Option<u8>> = const { std::cell::Cell::new(None) };
}

fn lingua() -> u8 {
    #[cfg(test)]
    if let Some(l) = NA_THREAD.with(std::cell::Cell::get) {
        return l;
    }
    LINGUA.load(Ordering::Relaxed)
}

/// Troca o idioma só nesta thread (testes).
#[cfg(test)]
pub fn definir_na_thread(codigo: &str) {
    NA_THREAD.with(|c| c.set(Some(indice(codigo))));
    feather_ui::idioma::definir_tradutor(|pt| match lingua() {
        0 => None,
        l => dicionario(l).get(pt).map(String::as_str),
    });
}

fn indice(codigo: &str) -> u8 {
    if codigo.starts_with("en") {
        1
    } else if codigo.starts_with("es") {
        2
    } else {
        0
    }
}

/// O idioma da configuração ou, se vazio, o mais próximo do sistema ("en_GB.UTF-8" → "en-US").
pub fn escolhido(config: &str) -> &'static str {
    if !config.is_empty() {
        return IDIOMAS[usize::from(indice(config))].0;
    }
    let sistema = ["LC_ALL", "LC_MESSAGES", "LANG"].iter().find_map(|v| std::env::var(v).ok().filter(|s| !s.is_empty()));
    IDIOMAS[usize::from(indice(sistema.as_deref().unwrap_or("pt")))].0
}

/// Troca o idioma da interface (na abertura e nas Configurações).
pub fn definir(codigo: &str) {
    LINGUA.store(indice(codigo), Ordering::Relaxed);
    // A Ajuda, o F12 e os componentes do feather-ui usam o mesmo dicionário.
    feather_ui::idioma::definir_tradutor(|pt| match lingua() {
        0 => None,
        l => dicionario(l).get(pt).map(String::as_str),
    });
}

/// O código do idioma atual.
pub fn atual() -> &'static str {
    IDIOMAS[usize::from(lingua())].0
}

fn pt_br() -> &'static Value {
    static PT: OnceLock<Value> = OnceLock::new();
    PT.get_or_init(|| serde_json::from_str(include_str!("../dados/pt-BR.json")).expect("Localidade pt-BR inválida"))
}

/// Português → idioma `l` (1 = inglês, 2 = espanhol).
fn dicionario(l: u8) -> &'static HashMap<String, String> {
    static EN: OnceLock<HashMap<String, String>> = OnceLock::new();
    static ES: OnceLock<HashMap<String, String>> = OnceLock::new();
    let ler = |json: &str| -> HashMap<String, String> {
        let mapa: HashMap<String, String> = serde_json::from_str(json).expect("Localidade inválida");
        mapa.into_iter().filter(|(k, _)| !k.starts_with("//")).collect()
    };
    if l == 1 {
        EN.get_or_init(|| ler(include_str!("../dados/en-US.json")))
    } else {
        ES.get_or_init(|| ler(include_str!("../dados/es-ES.json")))
    }
}

fn ja_traduzido(l: u8, texto: &str) -> bool {
    static EN: OnceLock<std::collections::HashSet<&'static str>> = OnceLock::new();
    static ES: OnceLock<std::collections::HashSet<&'static str>> = OnceLock::new();
    let conjunto = if l == 1 { &EN } else { &ES };
    conjunto.get_or_init(|| dicionario(l).values().map(String::as_str).collect()).contains(texto)
}

/// O texto em português `pt` no idioma da interface.
pub fn tr(pt: &str) -> &str {
    match lingua() {
        0 => pt,
        l => match dicionario(l).get(pt) {
            Some(t) => t,
            None => {
                // Texto que já chegou traduzido (um ajudante que traduz o rótulo recebido) passa direto.
                if !ja_traduzido(l, pt) {
                    anotar_falta(pt);
                }
                pt
            }
        },
    }
}

/// Como [`tr`], sem anotar falta: para ajudantes que recebem tanto o português quanto um texto
/// já montado e traduzido ("Padrão: Ctrl+B").
pub fn tr_se_houver(texto: &str) -> &str {
    match lingua() {
        0 => texto,
        l => dicionario(l).get(texto).map_or(texto, String::as_str),
    }
}

/// Como [`tr`], com os `{}` do modelo trocados pelos valores, na ordem.
pub fn trf(modelo: &str, valores: &[&dyn std::fmt::Display]) -> String {
    let mut saida = String::new();
    let mut resto = tr(modelo);
    for v in valores {
        let Some(i) = resto.find("{}") else { break };
        saida.push_str(&resto[..i]);
        saida.push_str(&v.to_string());
        resto = &resto[i + 2..];
    }
    saida.push_str(resto);
    saida
}

/// Uma lista fixa de opções traduzida (as listas suspensas pedem `&'static [&'static str]`).
/// Cada lista é traduzida uma vez por idioma; a posição de cada opção não muda.
pub fn lista(pt: &'static [&'static str]) -> &'static [&'static str] {
    let l = lingua();
    if l == 0 {
        return pt;
    }
    type Cache = Mutex<Vec<((usize, u8), &'static [&'static str])>>;
    static CACHE: OnceLock<Cache> = OnceLock::new();
    let chave = (pt.as_ptr() as usize, l);
    let mut cache = CACHE.get_or_init(Default::default).lock().unwrap_or_else(|e| e.into_inner());
    if let Some((_, lista)) = cache.iter().find(|(k, _)| *k == chave) {
        return lista;
    }
    let traduzida: &'static [&'static str] = Box::leak(pt.iter().map(|s| tr(s)).collect::<Vec<_>>().into_boxed_slice());
    cache.push((chave, traduzida));
    traduzida
}

/// A opção em português de `pt` que aparece como `mostrada` (o caminho de volta de [`lista`]).
pub fn de_volta(pt: &'static [&'static str], mostrada: &str) -> &'static str {
    pt.iter().copied().find(|o| tr(o) == mostrada).unwrap_or(pt[0])
}

/// O texto em português no `caminho` de `pt-BR.json` (`ajuda.rotulos.titulo`), ou `None`.
pub fn pt_em(caminho: &str) -> Option<&'static str> {
    let mut v = pt_br();
    for parte in caminho.split('.') {
        v = v.get(parte)?;
    }
    v.as_str()
}

/// Texto de `ferramentas_texto` no idioma da interface.
pub fn texto(chave: &str) -> &'static str {
    tr(pt_em(&format!("ferramentas_texto.{chave}")).unwrap_or("Texto indisponível"))
}

/// Textos pedidos sem tradução (só nos testes: o teste de guarda desenha as telas e confere).
#[cfg(test)]
static FALTAS: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn anotar_falta(_pt: &str) {
    // "1280 px", "E = mc²", "JPEG 85" não pedem tradução; palavra de verdade pede.
    #[cfg(test)]
    if _pt.chars().collect::<Vec<_>>().windows(3).any(|w| w.iter().all(|c| c.is_lowercase())) {
        FALTAS.lock().unwrap_or_else(|e| e.into_inner()).push(_pt.to_string());
    }
}

#[cfg(test)]
pub fn tirar_faltas() -> Vec<String> {
    let mut v = std::mem::take(&mut *FALTAS.lock().unwrap_or_else(|e| e.into_inner()));
    v.sort();
    v.dedup();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn textos_do_pt(v: &Value, saida: &mut Vec<String>) {
        match v {
            Value::String(s) => saida.push(s.clone()),
            Value::Object(m) => m.values().for_each(|x| textos_do_pt(x, saida)),
            Value::Array(a) => a.iter().for_each(|x| textos_do_pt(x, saida)),
            _ => {}
        }
    }

    /// Todo literal passado a `tr("…")`/`trf("…", …)` no código e todo texto de `pt-BR.json` existe
    /// em inglês e em espanhol, com os mesmos `{}`; os dois dicionários têm as mesmas chaves.
    #[test]
    fn todo_texto_tem_traducao_em_ingles_e_espanhol() {
        let mut pts = Vec::new();
        textos_do_pt(pt_br(), &mut pts);
        let literal = regex_lite();
        for (_, fonte) in fontes() {
            fonte.lines().filter(|l| !l.trim_start().starts_with("//")).for_each(|linha| pts.extend(literal(linha)));
        }
        let (en, es) = (dicionario(1), dicionario(2));
        let mut faltam = Vec::new();
        for pt in pts.iter().chain(en.keys()).chain(es.keys()) {
            for (nome, d) in [("en-US", en), ("es-ES", es)] {
                match d.get(pt.as_str()) {
                    None => faltam.push(format!("{nome}: {pt:?}")),
                    Some(t) if t.matches("{}").count() != pt.matches("{}").count() => {
                        faltam.push(format!("{nome} (marcadores): {pt:?}"))
                    }
                    _ => {}
                }
            }
        }
        faltam.sort();
        faltam.dedup();
        assert!(faltam.is_empty(), "{} texto(s) sem tradução:\n{}", faltam.len(), faltam.join("\n"));
    }

    /// Os literais de `tr("…")`, `trf("…"` e `lista(&[…])` numa linha (aspas escapadas incluídas).
    fn regex_lite() -> impl Fn(&str) -> Vec<String> {
        |linha: &str| {
            let mut achados = Vec::new();
            for marca in ["tr(\"", "trf(\""] {
                let mut resto = linha;
                while let Some(i) = resto.find(marca) {
                    let antes = resto[..i].chars().last();
                    resto = &resto[i + marca.len()..];
                    if antes.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                        continue;
                    }
                    let mut s = String::new();
                    let mut cs = resto.chars();
                    while let Some(c) = cs.next() {
                        match c {
                            '\\' => {
                                if let Some(n) = cs.next() {
                                    s.push(match n { 'n' => '\n', 't' => '\t', o => o });
                                }
                            }
                            '"' => break,
                            o => s.push(o),
                        }
                    }
                    achados.push(s);
                }
            }
            achados
        }
    }

    fn fontes() -> Vec<(String, String)> {
        let pasta = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
        std::fs::read_dir(pasta)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "rs"))
            .map(|e| (e.file_name().to_string_lossy().into_owned(), std::fs::read_to_string(e.path()).unwrap()))
            .collect()
    }
}
