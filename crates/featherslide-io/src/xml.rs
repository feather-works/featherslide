//! Um XML lido em árvore, pequeno de propósito: o `.pptx` e o `.odp` precisam de herança
//! (o slide pergunta ao layout, o layout ao mestre) e de procurar nós por caminho, o que
//! num leitor por eventos vira uma bagunça de estados. Os arquivos de um slide têm poucos
//! kB, então a árvore cabe folgada. Os nomes ficam com o prefixo como estão no arquivo
//! (`a:off`, `draw:frame`): os dois formatos usam sempre os mesmos prefixos.

use quick_xml::events::{BytesRef, BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

/// Um elemento: nome, atributos, filhos e o texto que está direto dentro dele. Os pedaços de
/// texto também entram em `filhos`, na ordem do arquivo, como nós de nome [`TEXTO`] — o ODF
/// mistura texto e marcas no mesmo parágrafo (`Olá <text:span>ODP</text:span> mundo`).
/// O nome dos nós que são só um pedaço de texto.
pub const TEXTO: &str = "#texto";

#[derive(Clone, Debug, Default)]
pub struct No {
    pub nome: String,
    pub attrs: Vec<(String, String)>,
    pub filhos: Vec<No>,
    pub texto: String,
}

impl No {
    /// O valor do atributo `nome`.
    pub fn attr(&self, nome: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| k == nome).map(|(_, v)| v.as_str())
    }

    /// O atributo como número.
    pub fn num(&self, nome: &str) -> Option<f64> {
        self.attr(nome).and_then(|v| v.trim().parse().ok())
    }

    /// O primeiro filho com esse nome.
    pub fn filho(&self, nome: &str) -> Option<&No> {
        self.filhos.iter().find(|f| f.nome == nome)
    }

    /// Todos os filhos com esse nome.
    pub fn filhos<'a>(&'a self, nome: &'a str) -> impl Iterator<Item = &'a No> + 'a {
        self.filhos.iter().filter(move |f| f.nome == nome)
    }

    /// Desce pelo caminho de nomes (`["p:spPr", "a:xfrm", "a:off"]`).
    pub fn caminho(&self, nomes: &[&str]) -> Option<&No> {
        nomes.iter().try_fold(self, |no, n| no.filho(n))
    }

    /// O primeiro descendente (em qualquer profundidade) com esse nome.
    pub fn busca(&self, nome: &str) -> Option<&No> {
        self.filhos.iter().find_map(|f| if f.nome == nome { Some(f) } else { f.busca(nome) })
    }

    /// Todos os descendentes com esse nome, na ordem do arquivo.
    pub fn todos<'a>(&'a self, nome: &str, saida: &mut Vec<&'a No>) {
        for f in &self.filhos {
            if f.nome == nome {
                saida.push(f);
            }
            f.todos(nome, saida);
        }
    }
}

fn referencia(r: &BytesRef) -> String {
    match r.resolve_char_ref() {
        Ok(Some(c)) => c.to_string(),
        _ => match r.as_ref() {
            "amp" => "&".into(),
            "lt" => "<".into(),
            "gt" => ">".into(),
            "quot" => "\"".into(),
            "apos" => "'".into(),
            _ => String::new(),
        },
    }
}

fn abrir(e: &BytesStart) -> No {
    let attrs = e
        .attributes()
        .flatten()
        .map(|a| (a.key.as_ref().to_string(), a.normalized_value(XmlVersion::Implicit1_0).map(|v| v.into_owned()).unwrap_or_default()))
        .collect();
    No { nome: e.name().as_ref().to_string(), attrs, filhos: Vec::new(), texto: String::new() }
}

/// Junta texto ao nó: no `texto` dele e como pedaço na ordem dos filhos (emendando pedaços vizinhos).
fn por_texto(no: &mut No, t: &str) {
    if t.is_empty() {
        return;
    }
    no.texto.push_str(t);
    match no.filhos.last_mut() {
        Some(f) if f.nome == TEXTO => f.texto.push_str(t),
        _ => no.filhos.push(No { nome: TEXTO.into(), texto: t.to_string(), ..No::default() }),
    }
}

/// Lê o XML inteiro em árvore. Erro em português, nunca pânico.
pub fn ler(bytes: &[u8], nome: &str) -> Result<No, String> {
    let mut leitor = Reader::from_reader(bytes);
    let mut pilha: Vec<No> = vec![No::default()];
    loop {
        let ev = leitor.read_event().map_err(|e| format!("{nome}: XML ilegível ({e})"))?;
        match ev {
            Event::Start(e) => pilha.push(abrir(&e)),
            Event::Empty(e) => {
                let no = abrir(&e);
                pilha.last_mut().expect("a raiz nunca sai").filhos.push(no);
            }
            Event::End(_) => {
                if pilha.len() > 1 {
                    let no = pilha.pop().expect("conferido acima");
                    pilha.last_mut().expect("a raiz nunca sai").filhos.push(no);
                }
            }
            Event::Text(t) => por_texto(pilha.last_mut().expect("a raiz nunca sai"), &t.xml10_content()),
            Event::GeneralRef(r) => por_texto(pilha.last_mut().expect("a raiz nunca sai"), &referencia(&r)),
            Event::CData(c) => por_texto(pilha.last_mut().expect("a raiz nunca sai"), c.as_ref()),
            Event::Eof => break,
            _ => {}
        }
    }
    while pilha.len() > 1 {
        let no = pilha.pop().expect("conferido acima");
        pilha.last_mut().expect("a raiz nunca sai").filhos.push(no);
    }
    // Fora do elemento raiz só sobra espaço em branco — o Google Apresentações grava um `\r`
    // logo depois do `<?xml …?>`, que não pode passar por raiz.
    let raiz = pilha.pop().unwrap_or_default();
    raiz.filhos.into_iter().find(|f| f.nome != TEXTO).ok_or_else(|| format!("{nome}: XML vazio"))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn le_atributos_filhos_e_texto_com_entidades() {
        let no = ler(br#"<a:p x="1"><a:r><a:t>caf&#233; &amp; ch&aacute;</a:t></a:r><a:br/></a:p>"#, "t").unwrap();
        assert_eq!(no.nome, "a:p");
        assert_eq!(no.num("x"), Some(1.0));
        assert_eq!(no.caminho(&["a:r", "a:t"]).unwrap().texto, "café & ch");
        assert!(no.filho("a:br").is_some());
    }

    #[test]
    fn espaco_antes_da_raiz_nao_vira_raiz() {
        let no = ler(b"<?xml version=\"1.0\"?>\r\n<p:presentation><p:sldIdLst/></p:presentation>", "t").unwrap();
        assert_eq!(no.nome, "p:presentation");
        assert!(no.filho("p:sldIdLst").is_some());
        assert!(ler(b"<?xml version=\"1.0\"?>\r\n", "so espaco").is_err());
    }

    #[test]
    fn xml_quebrado_da_erro_nao_panico() {
        assert!(ler(b"", "vazio").is_err());
        let _ = ler(b"<a><b></a>", "torto");
    }
}
