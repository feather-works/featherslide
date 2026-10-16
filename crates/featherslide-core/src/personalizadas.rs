//! Apresentações personalizadas: uma ordem independente, por IDs estáveis dos slides.
//! Ocultar aqui não modifica o atributo Slide::oculto da apresentação principal.

use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::sessao::{Mudanca, Sessao};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlidePersonalizado {
    pub id: u64,
    pub oculto: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApresentacaoPersonalizada {
    pub nome: String,
    pub slides: Vec<SlidePersonalizado>,
}

impl Sessao {
    /// Retorna apresentações personalizadas na ordem em que foram criadas.
    pub fn personalizadas(&self) -> &[ApresentacaoPersonalizada] {
        &self.ap.metadados.apresentacoes_personalizadas
    }

    /// Cria uma sequência com os slides indicados, na ordem recebida.
    pub fn personalizada_criar(&mut self, nome: &str, indices: &[usize]) -> Result<Mudanca, String> {
        let nome = nome.trim();
        if nome.is_empty() || nome.len() > 120 {
            return Err("Informe um nome de até 120 bytes para a apresentação.".into());
        }
        if self.personalizadas().iter().any(|p| p.nome.eq_ignore_ascii_case(nome)) {
            return Err("Já existe uma apresentação personalizada com este nome.".into());
        }
        if self.personalizadas().len() >= 50 {
            return Err("Limite de 50 apresentações personalizadas.".into());
        }
        let mut ids = HashSet::new();
        let mut slides = Vec::new();
        for &i in indices {
            let slide = self.ap.slides.get(i).ok_or("Índice de slide inválido.")?;
            if !ids.insert(slide.id) {
                return Err("A mesma página foi selecionada duas vezes.".into());
            }
            slides.push(SlidePersonalizado { id: slide.id, oculto: false });
        }
        if slides.is_empty() {
            return Err("Marque ao menos um slide para criar a apresentação.".into());
        }
        if slides.len() > 1000 {
            return Err("A sequência pode ter no máximo 1000 slides.".into());
        }
        self.editar(move |s| {
            s.ap.metadados.apresentacoes_personalizadas.push(
                ApresentacaoPersonalizada { nome: nome.to_string(), slides }
            );
            Ok((vec![], false))
        })
    }

    pub fn personalizada_excluir(&mut self, indice: usize) -> Result<Mudanca, String> {
        if indice >= self.personalizadas().len() { return Err("Apresentação personalizada não encontrada.".into()); }
        self.editar(|s| {
            s.ap.metadados.apresentacoes_personalizadas.remove(indice);
            Ok((vec![], false))
        })
    }

    /// Acrescenta slides marcados ao fim, ignorando os já presentes.
    pub fn personalizada_adicionar(&mut self, indice: usize, indices: &[usize]) -> Result<Mudanca, String> {
        let p = self.personalizadas().get(indice).ok_or("Apresentação personalizada não encontrada.")?;
        let mut ids: HashSet<_> = p.slides.iter().map(|s| s.id).collect();
        let mut novos = Vec::new();
        for &i in indices {
            let slide = self.ap.slides.get(i).ok_or("Índice de slide inválido.")?;
            if ids.insert(slide.id) {
                novos.push(SlidePersonalizado { id: slide.id, oculto: false });
            }
        }
        if novos.is_empty() { return Err("Todos os slides marcados já pertencem a esta sequência.".into()); }
        if ids.len() > 1000 { return Err("A sequência pode ter no máximo 1000 slides.".into()); }
        self.editar(move |s| {
            s.ap.metadados.apresentacoes_personalizadas[indice].slides.extend(novos);
            Ok((vec![], false))
        })
    }

    pub fn personalizada_mover(&mut self, indice: usize, pos: usize, direcao: i32) -> Result<Mudanca, String> {
        let p = self.personalizadas().get(indice).ok_or("Apresentação personalizada não encontrada.")?;
        let nova = pos as i32 + direcao;
        if !(-1..=1).contains(&direcao) || nova < 0 || nova >= p.slides.len() as i32 || nova == pos as i32 {
            return Err("O slide já está no limite desta sequência.".into());
        }
        self.editar(move |s| {
            s.ap.metadados.apresentacoes_personalizadas[indice].slides.swap(pos, nova as usize);
            Ok((vec![], false))
        })
    }

    pub fn personalizada_ocultar(&mut self, indice: usize, pos: usize) -> Result<Mudanca, String> {
        let p = self.personalizadas().get(indice).ok_or("Apresentação personalizada não encontrada.")?;
        let slide = p.slides.get(pos).ok_or("Slide não encontrado nessa sequência.")?;
        if !slide.oculto && p.slides.iter().filter(|s| !s.oculto).count() == 1 {
            return Err("A sequência precisa ter pelo menos um slide visível.".into());
        }
        self.editar(move |s| {
            let slide = &mut s.ap.metadados.apresentacoes_personalizadas[indice].slides[pos];
            slide.oculto = !slide.oculto;
            Ok((vec![], false))
        })
    }

    pub fn personalizada_remover_slide(&mut self, indice: usize, pos: usize) -> Result<Mudanca, String> {
        let p = self.personalizadas().get(indice).ok_or("Apresentação personalizada não encontrada.")?;
        if pos >= p.slides.len() { return Err("Slide não encontrado nessa sequência.".into()); }
        if p.slides.len() == 1 { return Err("A sequência precisa ter ao menos um slide. Exclua a apresentação inteira.".into()); }
        self.editar(move |s| {
            s.ap.metadados.apresentacoes_personalizadas[indice].slides.remove(pos);
            // Se sobrou apenas slide oculto, reativar o primeiro.
            let lista = &mut s.ap.metadados.apresentacoes_personalizadas[indice].slides;
            if lista.iter().all(|sl| sl.oculto) { lista[0].oculto = false; }
            Ok((vec![], false))
        })
    }

    /// Traduz os IDs persistentes para índices atuais; páginas excluídas são ignoradas.
    /// A ocultação de cada sequência é independente do slide original.
    pub fn personalizada_ordem(&self, indice: usize) -> Result<Vec<usize>, String> {
        let p = self.personalizadas().get(indice).ok_or("Apresentação personalizada não encontrada.")?;
        let ordem: Vec<_> = p.slides.iter().filter(|sl| !sl.oculto)
            .filter_map(|sl| self.ap.slides.iter().position(|s| s.id == sl.id)).collect();
        if ordem.is_empty() { return Err("Essa apresentação personalizada não tem slides disponíveis.".into()); }
        Ok(ordem)
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn personalizada_preserva_ordem_ocultacao_e_desfazer() {
        let mut s = Sessao::default();
        s.slide_novo("em_branco").unwrap();
        s.slide_novo("em_branco").unwrap();
        let ids: Vec<_> = s.apresentacao().slides.iter().map(|s| s.id).collect();
        s.personalizada_criar("Reunião", &[2,0,1]).unwrap();
        assert_eq!(s.personalizada_ordem(0).unwrap(), vec![2,0,1]);
        s.personalizada_mover(0, 1, 1).unwrap();
        assert_eq!(s.personalizada_ordem(0).unwrap(), vec![2,1,0]);
        s.personalizada_ocultar(0, 0).unwrap();
        assert_eq!(s.personalizada_ordem(0).unwrap(), vec![1,0]);
        s.desfazer().unwrap();
        assert_eq!(s.personalizada_ordem(0).unwrap(), vec![2,1,0]);
        s.slides_reordenar(&[0], 2).unwrap();
        let ordem = s.personalizada_ordem(0).unwrap();
        assert_eq!(ordem.iter().map(|&i| s.apresentacao().slides[i].id).collect::<Vec<_>>(),
            vec![ids[2], ids[1], ids[0]]);
    }
}
