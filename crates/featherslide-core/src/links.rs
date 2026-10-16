//! Links e comentários: o hiperlink de um elemento (abre um endereço na apresentação) e as
//! notas de revisão presas a um slide. O motor só guarda e valida; abrir o endereço é da casca.

use crate::modelo::Comentario;
use crate::sessao::{Mudanca, Sessao};

/// Normaliza o endereço digitado: sem espaços nas pontas; "a@b.com" vira `mailto:`; sem
/// esquema (`exemplo.com`) vira `https://`. Vazio é erro. Só `http`, `https` e `mailto` passam.
pub fn endereco_valido(bruto: &str) -> Result<String, String> {
    let t = bruto.trim();
    if t.is_empty() {
        return Err("Digite o endereço do link.".into());
    }
    if t.chars().any(char::is_whitespace) {
        return Err("O endereço não pode ter espaços.".into());
    }
    let minusculo = t.to_lowercase();
    if minusculo.starts_with("http://") || minusculo.starts_with("https://") || minusculo.starts_with("mailto:") {
        return Ok(t.to_string());
    }
    if let Some((esquema, _)) = t.split_once(':') {
        if esquema.chars().all(|c| c.is_ascii_alphabetic()) && !esquema.is_empty() {
            return Err(format!("Esquema \"{esquema}:\" não é permitido; use http, https ou mailto."));
        }
    }
    if t.contains('@') && !t.contains('/') {
        return Ok(format!("mailto:{t}"));
    }
    Ok(format!("https://{t}"))
}

impl Sessao {
    /// Põe o link `endereco` em todos os elementos selecionados.
    pub fn link_definir(&mut self, endereco: &str) -> Result<Mudanca, String> {
        if self.selecao.is_empty() {
            return Err("Selecione uma forma, imagem ou caixa de texto para ligar.".into());
        }
        let url = endereco_valido(endereco)?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                e.link = url.clone();
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Tira o link dos elementos selecionados.
    pub fn link_remover(&mut self) -> Result<Mudanca, String> {
        self.editar(|s| {
            for e in s.selecionados_mut() {
                e.link.clear();
            }
            Ok((vec![s.atual], false))
        })
    }

    /// O link do elemento `id` do slide `indice`, se tiver.
    pub fn link_do_elemento(&self, indice: usize, id: u64) -> Option<String> {
        let e = self.ap.slides.get(indice)?.elementos.iter().find(|e| e.id == id)?;
        (!e.link.is_empty()).then(|| e.link.clone())
    }

    /// Acrescenta um comentário ao slide atual.
    pub fn comentario_adicionar(&mut self, autor: &str, texto: &str) -> Result<Mudanca, String> {
        let texto = texto.trim();
        if texto.is_empty() {
            return Err("O comentário está vazio.".into());
        }
        let autor = if autor.trim().is_empty() { "Autor" } else { autor.trim() }.to_string();
        let texto = texto.to_string();
        self.editar(|s| {
            s.slide_mut().comentarios.push(Comentario { autor, texto, resolvido: false });
            Ok((vec![s.atual], false))
        })
    }

    /// Apaga o comentário `indice` do slide atual.
    pub fn comentario_excluir(&mut self, indice: usize) -> Result<Mudanca, String> {
        if indice >= self.slide().comentarios.len() {
            return Err("Esse comentário não existe.".into());
        }
        self.editar(|s| {
            s.slide_mut().comentarios.remove(indice);
            Ok((vec![s.atual], false))
        })
    }

    /// Marca (ou desmarca) o comentário `indice` do slide atual como resolvido.
    pub fn comentario_resolver(&mut self, indice: usize, resolvido: bool) -> Result<Mudanca, String> {
        if indice >= self.slide().comentarios.len() {
            return Err("Esse comentário não existe.".into());
        }
        self.editar(|s| {
            s.slide_mut().comentarios[indice].resolvido = resolvido;
            Ok((vec![s.atual], false))
        })
    }

    /// Apaga todos os comentários do slide atual, ou da apresentação inteira.
    pub fn comentarios_excluir_todos(&mut self, apresentacao_inteira: bool) -> Result<Mudanca, String> {
        self.editar(|s| {
            if apresentacao_inteira {
                s.ap.slides.iter_mut().for_each(|sl| sl.comentarios.clear());
                Ok(((0..s.ap.slides.len()).collect(), false))
            } else {
                s.slide_mut().comentarios.clear();
                Ok((vec![s.atual], false))
            }
        })
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn endereco_normaliza() {
        assert_eq!(endereco_valido("exemplo.com").unwrap(), "https://exemplo.com");
        assert_eq!(endereco_valido(" http://a.b/x ").unwrap(), "http://a.b/x");
        assert_eq!(endereco_valido("fulano@x.com").unwrap(), "mailto:fulano@x.com");
        assert!(endereco_valido("").is_err());
        assert!(endereco_valido("a b").is_err());
        assert!(endereco_valido("javascript:alert(1)").is_err());
        assert!(endereco_valido("file:///etc/passwd").is_err());
    }
}
