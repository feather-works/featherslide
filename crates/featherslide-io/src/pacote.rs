//! O pacote ZIP de um `.pptx` ou `.odp`: ler uma parte, ler uma parte como XML (guardando a
//! árvore, porque o mesmo layout e o mesmo mestre servem muitos slides) e, no `.pptx`, as
//! ligações (`_rels`) de cada parte com os caminhos já resolvidos.

use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::rc::Rc;

use crate::xml::{self, No};

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
        let mut f = self.zip.by_name(nome).map_err(|_| format!("O {formato} está incompleto: falta {nome}."))?;
        let mut v = Vec::new();
        f.read_to_end(&mut v).map_err(|e| format!("Não consegui ler {nome}: {e}"))?;
        Ok(v)
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
    fn caminho_relativo_resolvido() {
        assert_eq!(resolver("ppt/slides", "../slideLayouts/slideLayout1.xml"), "ppt/slideLayouts/slideLayout1.xml");
        assert_eq!(resolver("ppt", "slides/slide1.xml"), "ppt/slides/slide1.xml");
        assert_eq!(resolver("ppt/slides", "/ppt/media/a.png"), "ppt/media/a.png");
    }
}
