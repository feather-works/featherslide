//! O pacote ZIP de um `.pptx` ou `.odp`: ler uma parte, ler uma parte como XML (guardando a
//! árvore, porque o mesmo layout e o mesmo mestre servem muitos slides) e, no `.pptx`, as
//! ligações (`_rels`) de cada parte com os caminhos já resolvidos.

use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::rc::Rc;

use crate::xml::{self, No};

/// Teto de uma parte descompactada: um ZIP de poucos kB pode se expandir em GB (ZIP-bomba).
pub(crate) const TETO_PARTE: u64 = 512 * 1024 * 1024;

/// Lê uma parte do ZIP até o [`TETO_PARTE`]; passou disso, erro em vez de encher a memória.
pub(crate) fn ler_com_teto(f: impl Read, nome: &str) -> Result<Vec<u8>, String> {
    ler_ate(f, nome, TETO_PARTE)
}

fn ler_ate(f: impl Read, nome: &str, teto: u64) -> Result<Vec<u8>, String> {
    let mut v = Vec::new();
    f.take(teto + 1).read_to_end(&mut v).map_err(|e| format!("Não consegui ler {nome}: {e}"))?;
    if v.len() as u64 > teto {
        return Err(format!("{nome} passa de {} MB descompactado; o arquivo está corrompido ou é malicioso.", teto >> 20));
    }
    Ok(v)
}

pub struct Pacote<'a> {
    zip: zip::ZipArchive<Cursor<&'a [u8]>>,
    arvores: HashMap<String, Rc<No>>,
    /// Nome do formato, para as mensagens de erro (".pptx", ".odp").
    formato: &'static str,
}

impl<'a> Pacote<'a> {
    /// Abre o ZIP. Erro em português se não for um.
    pub fn abrir(bytes: &'a [u8], formato: &'static str) -> Result<Self, String> {
        let zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| format!("Este arquivo não é um {formato} (não é um ZIP)."))?;
        Ok(Self { zip, arvores: HashMap::new(), formato })
    }

    /// A parte existe?
    pub fn tem(&self, nome: &str) -> bool {
        self.zip.index_for_name(nome).is_some()
    }

    /// Os bytes de uma parte.
    pub fn parte(&mut self, nome: &str) -> Result<Vec<u8>, String> {
        let formato = self.formato;
        let f = self.zip.by_name(nome).map_err(|_| format!("O {formato} está incompleto: falta {nome}."))?;
        ler_com_teto(f, nome)
    }

    /// Uma parte como árvore XML (lida uma vez só).
    pub fn xml(&mut self, nome: &str) -> Result<Rc<No>, String> {
        if let Some(a) = self.arvores.get(nome) {
            return Ok(a.clone());
        }
        let bytes = self.parte(nome)?;
        let arvore = Rc::new(xml::ler(&bytes, nome)?);
        self.arvores.insert(nome.to_string(), arvore.clone());
        Ok(arvore)
    }

    /// As ligações de uma parte do `.pptx` (`Id` → caminho da parte ligada, já resolvido).
    /// Sem o arquivo de ligações, nenhuma.
    pub fn ligacoes(&mut self, parte: &str) -> HashMap<String, String> {
        let (pasta, arquivo) = parte.rsplit_once('/').unwrap_or(("", parte));
        let nome = format!("{pasta}/_rels/{arquivo}.rels");
        let Ok(rels) = self.xml(&nome) else { return HashMap::new() };
        rels.filhos("Relationship")
            .filter(|r| r.attr("TargetMode") != Some("External"))
            .filter_map(|r| Some((r.attr("Id")?.to_string(), resolver(pasta, r.attr("Target")?))))
            .collect()
    }
}

/// `../slideLayouts/slideLayout1.xml` visto de `ppt/slides` vira `ppt/slideLayouts/slideLayout1.xml`.
pub fn resolver(pasta: &str, alvo: &str) -> String {
    if let Some(absoluto) = alvo.strip_prefix('/') {
        return absoluto.to_string();
    }
    let mut partes: Vec<&str> = pasta.split('/').filter(|p| !p.is_empty()).collect();
    for p in alvo.split('/') {
        match p {
            ".." => {
                partes.pop();
            }
            "." | "" => {}
            _ => partes.push(p),
        }
    }
    partes.join("/")
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn parte_acima_do_teto_da_erro() {
        assert!(super::ler_ate(&[0u8; 11][..], "bomba", 10).is_err());
        assert_eq!(super::ler_ate(&b"ok"[..], "pequena", 10).unwrap(), b"ok");
    }

    #[test]
    fn caminho_relativo_resolvido() {
        assert_eq!(resolver("ppt/slides", "../slideLayouts/slideLayout1.xml"), "ppt/slideLayouts/slideLayout1.xml");
        assert_eq!(resolver("ppt", "slides/slide1.xml"), "ppt/slides/slide1.xml");
        assert_eq!(resolver("ppt/slides", "/ppt/media/a.png"), "ppt/media/a.png");
    }
}
