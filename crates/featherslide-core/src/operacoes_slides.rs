//! Operações da seção 2.1: área de transferência interna, layouts e inserção.
use std::collections::{BTreeMap, BTreeSet};
use crate::layouts;
use crate::modelo::{Apresentacao, Elemento, FonteEmbutida, Papel, Proporcao, Slide};
use crate::sessao::{slide_com_layout, Mudanca, Sessao};

#[derive(Clone, Debug)]
pub struct SlidesCopiados {
    pub origem: Vec<(usize, Slide)>,
    pub proporcao: Proporcao,
    pub tamanho: (f32, f32),
    pub tema: String,
    pub fontes: Vec<FonteEmbutida>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EstiloColagem { ManterOrigem, AdaptarAoTema }

fn indices_validos(indices: &[usize], total: usize) -> Result<Vec<usize>, String> {
    let s: BTreeSet<_> = indices.iter().copied().collect();
    if s.is_empty() || s.iter().any(|i| *i >= total) {
        return Err("Selecione slides válidos.".into());
    }
    Ok(s.into_iter().collect())
}

impl SlidesCopiados {
    pub fn da_apresentacao(ap: &Apresentacao, indices: &[usize]) -> Result<Self, String> {
        let indices = indices_validos(indices, ap.slides.len())?;
        Ok(Self {
            origem: indices.into_iter().map(|i| (i, ap.slides[i].clone())).collect(),
            proporcao: ap.proporcao, tamanho: ap.tamanho(), tema: ap.tema.clone(), fontes: ap.fontes.clone(),
        })
    }
    pub fn quantidade(&self) -> usize { self.origem.len() }
}

impl Sessao {
    /// Insere um slide exatamente na posição 0..=n, inclusive antes do primeiro.
    pub fn slide_inserir_em(&mut self, pos: usize, layout: &str) -> Result<Mudanca, String> {
        if pos > self.ap.slides.len() { return Err("Posição de inserção inválida.".into()); }
        if !layouts::LAYOUTS.contains(&layout) { return Err(format!("Layout desconhecido: {layout}")); }
        self.editar(move |s| {
            let slide = slide_com_layout(&mut s.ap, layout);
            s.ap.slides.insert(pos, slide);
            s.atual = pos;
            s.selecao.clear();
            Ok((vec![pos], true))
        })
    }

    pub fn slides_copiar(&self, indices: &[usize]) -> Result<SlidesCopiados, String> {
        SlidesCopiados::da_apresentacao(&self.ap, indices)
    }

    /// Recorta slides em uma transação; nunca deixa a apresentação vazia.
    pub fn slides_recortar(&mut self, indices: &[usize]) -> Result<(SlidesCopiados, Mudanca), String> {
        let dados = self.slides_copiar(indices)?;
        let indices: Vec<_> = dados.origem.iter().map(|(i, _)| *i).collect();
        if indices.len() >= self.ap.slides.len() {
            return Err("Não é possível recortar todos os slides: mantenha pelo menos um.".into());
        }
        let mudanca = self.editar(move |s| {
            for &i in indices.iter().rev() {
                let removido = s.ap.slides.remove(i);
                if !removido.secao.is_empty() {
                    if let Some(proximo) = s.ap.slides.get_mut(i) {
                        if proximo.secao.is_empty() { proximo.secao = removido.secao; }
                    }
                }
            }
            s.atual = s.atual.min(s.ap.slides.len() - 1);
            s.selecao.clear();
            Ok(((0..s.ap.slides.len()).collect(), true))
        })?;
        Ok((dados, mudanca))
    }

    /// Cola os slides antes do índice de destino, remapeando IDs e links internos.
    pub fn slides_colar(&mut self, pos: usize, dados: &SlidesCopiados, estilo: EstiloColagem) -> Result<Mudanca, String> {
        if pos > self.ap.slides.len() || dados.origem.is_empty() { return Err("Área de transferência vazia ou posição inválida.".into()); }
        let dados = dados.clone();
        self.editar(move |s| {
            let (ow, oh) = dados.tamanho;
            let (w, h) = s.ap.tamanho();
            if ow <= 0.0 || oh <= 0.0 { return Err("Slide de origem tem dimensões inválidas.".into()); }
            let links: BTreeMap<usize, usize> = dados.origem.iter().enumerate().map(|(j, (i, _))| (i + 1, pos + j + 1)).collect();
            let mut novos = Vec::with_capacity(dados.origem.len());
            for (_, original) in &dados.origem {
                let mut slide = original.clone();
                slide.id = s.ap.novo_id();
                let mut mapa=Vec::new();
                for e in &mut slide.elementos {
                    let antigo=e.id;e.id = s.ap.novo_id();mapa.push((antigo,e.id));
                    if e.destino != 0 { e.destino = links.get(&(e.destino as usize)).copied().unwrap_or(0) as u32; }
                    e.x *= w / ow; e.y *= h / oh;
                    e.largura *= w / ow; e.altura *= h / oh;
                }
                crate::outras_galerias::remapear_conectores(&mut slide.elementos,&mapa);
                if estilo == EstiloColagem::AdaptarAoTema {
                    crate::temas::aplicar_no_slide(&mut slide, crate::temas::tema(&dados.tema), crate::temas::tema(&s.ap.tema));
                    slide.elementos.retain(|e| e.papel != Papel::Fundo);
                    if !s.ap.modelo.is_empty() {
                        let modelo = s.ap.modelo.clone();
                        let layout = slide.layout.clone();
                        crate::modelos::decorar(&mut s.ap, &mut slide, &modelo, &layout);
                    }
                }
                novos.push(slide);
            }
            for f in &dados.fontes {
                if !s.ap.fontes.iter().any(|x| x.nome == f.nome) { s.ap.fontes.push(f.clone()); }
            }
            let n = novos.len();
            s.ap.slides.splice(pos..pos, novos);
            s.atual = pos;
            s.selecao.clear();
            Ok(((pos..pos+n).collect(), true))
        })
    }

    /// Exclui vários slides em uma única transação, sem deixar apresentação vazia.
    pub fn slides_excluir(&mut self, indices: &[usize]) -> Result<Mudanca, String> {
        let indices = indices_validos(indices, self.ap.slides.len())?;
        if indices.len() == self.ap.slides.len() {
            return Err("Não é possível excluir todos os slides da apresentação.".into());
        }
        self.editar(move |s| {
            for &i in indices.iter().rev() {
                let removido = s.ap.slides.remove(i);
                if !removido.secao.is_empty() {
                    if let Some(proximo) = s.ap.slides.get_mut(i) {
                        if proximo.secao.is_empty() { proximo.secao = removido.secao; }
                    }
                }
            }
            s.atual = s.atual.min(s.ap.slides.len() - 1);
            s.selecao.clear();
            Ok(((0..s.ap.slides.len()).collect(), true))
        })
    }

    /// Vários slides recebem a mesma visibilidade (todos ocultos ou todos visíveis).
    pub fn slides_alternar_ocultacao(&mut self, indices: &[usize]) -> Result<Mudanca, String> {
        let indices = indices_validos(indices, self.ap.slides.len())?;
        let oculto = !self.ap.slides[indices[0]].oculto;
        self.editar(move |s| {
            for &i in &indices { s.ap.slides[i].oculto = oculto; }
            Ok((indices, true))
        })
    }

    /// Reordena um bloco de miniaturas preservando a ordem interna, um só desfazer.
    pub fn slides_reordenar(&mut self, indices: &[usize], posicao: usize) -> Result<Mudanca, String> {
        let indices = indices_validos(indices, self.ap.slides.len())?;
        if posicao >= self.ap.slides.len() {
            return Err("Posição de slide fora da apresentação.".into());
        }
        self.editar(move |s| {
            let cabecas: Vec<_> = s.ap.slides.iter().map(|sl| sl.secao.clone()).collect();
            let guardados: Vec<_> = indices.iter().map(|&i| s.ap.slides[i].clone()).collect();
            for &i in indices.iter().rev() { s.ap.slides.remove(i); }
            let posicao = posicao.min(s.ap.slides.len());
            let quantidade = guardados.len();
            s.ap.slides.splice(posicao..posicao, guardados);
            for (slide, secao) in s.ap.slides.iter_mut().zip(cabecas) {
                slide.secao = secao;
            }
            s.atual = posicao;
            s.selecao.clear();
            Ok(((posicao..posicao + quantidade).collect(), true))
        })
    }

    pub fn slides_layout(&mut self, indices: &[usize], layout: &str) -> Result<Mudanca, String> {
        if !layouts::LAYOUTS.contains(&layout) { return Err(format!("Layout desconhecido: {layout}")); }
        let alvos = indices_validos(indices, self.ap.slides.len())?;
        let layout = layout.to_string();
        self.editar(move |s| {
            for &i in &alvos { aplicar_layout(s, i, &layout); }
            s.selecao.clear();
            Ok((alvos, false))
        })
    }

    pub fn slides_restaurar_layout(&mut self, indices: &[usize]) -> Result<Mudanca, String> {
        let alvos = indices_validos(indices, self.ap.slides.len())?;
        self.editar(move |s| {
            for &i in &alvos {
                let layout = s.ap.slides[i].layout.clone();
                if layouts::LAYOUTS.contains(&layout.as_str()) { aplicar_layout(s, i, &layout); }
            }
            s.selecao.clear();
            Ok((alvos, false))
        })
    }
}

fn aplicar_layout(s: &mut Sessao, i: usize, layout: &str) {
    let (w,h) = s.ap.tamanho();
    let mut proximo = s.ap.proximo_id;
    let mut novas = layouts::elementos_do_layout(layout, w, h, &mut || {
        let id = proximo; proximo += 1; id
    }).unwrap_or_default();
    s.ap.proximo_id = proximo;
    let slide = &mut s.ap.slides[i];
    let mut antigas = Vec::new();
    slide.elementos.retain(|e| {
        if e.papel.do_layout() { antigas.push(e.clone()); false } else { true }
    });
    for n in &mut novas {
        if let Some(pos) = antigas.iter().position(|a: &Elemento| a.papel == n.papel) {
            let velho = antigas.remove(pos);
            n.id = velho.id;
            n.texto = velho.texto;
        }
    }
    slide.elementos.extend(novas);
    for mut e in antigas.into_iter().filter(|e| !e.texto.is_empty()) {
        e.papel = Papel::Livre;
        slide.elementos.push(e);
    }
    slide.layout = layout.into();
    if !s.ap.modelo.is_empty() {
        let modelo = s.ap.modelo.clone();
        let mut clone = slide.clone();
        clone.elementos.retain(|e| e.papel != Papel::Fundo);
        crate::modelos::decorar(&mut s.ap, &mut clone, &modelo, layout);
        s.ap.slides[i] = clone;
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::modelo::Tipo;
    #[test]
    fn inserir_antes_e_depois() {
        let mut s = Sessao::default();
        let primeiro = s.slide().id;
        s.slide_inserir_em(0, "em_branco").unwrap();
        assert_eq!(s.ap.slides[1].id, primeiro);
        s.desfazer().unwrap();
        s.slide_inserir_em(1, "titulo_conteudo").unwrap();
        assert_eq!(s.ap.slides.len(), 2);
    }
    #[test]
    fn copiar_recortar_colar_undo_e_ids() {
        let mut s = Sessao::default();
        s.slide_novo("em_branco").unwrap();
        s.elemento_inserir(Tipo::Retangulo, None, None).unwrap();
        let id = s.slide().elementos.last().unwrap().id;
        let cp = s.slides_copiar(&[1]).unwrap();
        s.slides_colar(2, &cp, EstiloColagem::ManterOrigem).unwrap();
        assert_eq!(s.ap.slides.len(), 3);
        assert_ne!(s.ap.slides[2].elementos.last().unwrap().id, id);
        s.desfazer().unwrap();
        assert_eq!(s.ap.slides.len(), 2);
        s.slides_recortar(&[1]).unwrap();
        assert_eq!(s.ap.slides.len(), 1);
        s.desfazer().unwrap();
        assert_eq!(s.ap.slides.len(), 2);
        assert!(s.slides_recortar(&[0,1]).is_err());
    }
    #[test]
    fn excluir_ocultar_e_reordenar_lote_sao_uma_acao() {
        let mut s = Sessao::default();
        for _ in 0..3 { s.slide_novo("em_branco").unwrap(); }
        let ids: Vec<_> = s.ap.slides.iter().map(|e| e.id).collect();
        s.slides_reordenar(&[0,1], 2).unwrap();
        assert_eq!(s.ap.slides.iter().map(|e| e.id).collect::<Vec<_>>(), vec![ids[2], ids[3], ids[0], ids[1]]);
        s.desfazer().unwrap();
        assert_eq!(s.ap.slides.iter().map(|e| e.id).collect::<Vec<_>>(), ids);
        s.slides_alternar_ocultacao(&[1,3]).unwrap();
        assert!(s.ap.slides[1].oculto && s.ap.slides[3].oculto);
        s.desfazer().unwrap();
        assert!(!s.ap.slides[1].oculto && !s.ap.slides[3].oculto);
        s.slides_excluir(&[1,3]).unwrap();
        assert_eq!(s.ap.slides.len(), 2);
        s.desfazer().unwrap();
        assert_eq!(s.ap.slides.len(), 4);
        assert!(s.slides_excluir(&[0,1,2,3]).is_err());
    }

    #[test]
    fn colagem_adapta_tema_e_layout_em_lote() {
        let mut fonte = Sessao::default();
        fonte.slide_novo("em_branco").unwrap();
        fonte.elemento_inserir(Tipo::Retangulo, None, None).unwrap();
        let cp = fonte.slides_copiar(&[1]).unwrap();
        let mut alvo = Sessao::default();
        crate::temas::aplicar(alvo.apresentacao_mut(), "escuro");
        alvo.slides_colar(1, &cp, EstiloColagem::AdaptarAoTema).unwrap();
        assert_eq!(alvo.ap.slides[1].fundo, "#202124");
        alvo.slides_layout(&[0,1], "titulo_conteudo").unwrap();
        assert!(alvo.ap.slides.iter().all(|sl| sl.layout == "titulo_conteudo"));
        alvo.slides_restaurar_layout(&[1]).unwrap();
        alvo.desfazer().unwrap();
    }
}
