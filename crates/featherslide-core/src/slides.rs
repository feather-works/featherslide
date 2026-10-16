//! Os comandos sobre slides: novo, duplicar, excluir, mover, ir, trocar o layout, o
//! fundo, ocultar, as anotações e o tamanho do slide.

use crate::layouts;
use crate::modelo::{Elemento, Papel, Proporcao};
use crate::sessao::{slide_com_layout, Mudanca, Sessao};

impl Sessao {
    /// Um slide novo com `layout`, logo depois do atual; ele vira o atual.
    pub fn slide_novo(&mut self, layout: &str) -> Result<Mudanca, String> {
        self.editar(|s| {
            let slide = slide_com_layout(&mut s.ap, layout);
            let i = s.atual + 1;
            s.ap.slides.insert(i, slide);
            s.atual = i;
            s.selecao.clear();
            Ok((vec![i], true))
        })
    }

    /// Uma cópia do slide atual (com ids novos), logo depois dele.
    pub fn slide_duplicar(&mut self) -> Result<Mudanca, String> {
        self.editar(|s| {
            let mut copia = s.slide().clone();
            copia.id = s.ap.novo_id();
            copia.secao.clear(); // a cópia fica na seção do original
            let mut mapa=Vec::new();
            for e in &mut copia.elementos {
                let antigo=e.id;e.id = s.ap.novo_id();mapa.push((antigo,e.id));
            }
            crate::outras_galerias::remapear_conectores(&mut copia.elementos,&mapa);
            let i = s.atual + 1;
            s.ap.slides.insert(i, copia);
            s.atual = i;
            s.selecao.clear();
            Ok((vec![i], true))
        })
    }

    /// Exclui o slide atual (a apresentação nunca fica sem slide).
    pub fn slide_excluir(&mut self) -> Result<Mudanca, String> {
        if self.ap.slides.len() <= 1 {
            return Err("A apresentação precisa ter pelo menos um slide.".into());
        }
        self.editar(|s| {
            let removido = s.ap.slides.remove(s.atual);
            // Se o slide excluído abria uma seção, o seguinte (se ainda não abre outra) passa a abri-la.
            if !removido.secao.is_empty() {
                if let Some(proximo) = s.ap.slides.get_mut(s.atual) {
                    if proximo.secao.is_empty() {
                        proximo.secao = removido.secao;
                    }
                }
            }
            s.atual = s.atual.min(s.ap.slides.len() - 1);
            s.selecao.clear();
            Ok((vec![s.atual], true))
        })
    }

    /// Leva o slide `de` para a posição `para` (o que estava lá anda uma casa).
    pub fn slide_mover(&mut self, de: usize, para: usize) -> Result<Mudanca, String> {
        let n = self.ap.slides.len();
        if de >= n || para >= n {
            return Err("Posição de slide fora da apresentação.".into());
        }
        self.editar(|s| {
            // As seções ficam nas posições: quem se move troca de seção, as fronteiras não andam.
            let cabecas: Vec<String> = s.ap.slides.iter().map(|sl| sl.secao.clone()).collect();
            let slide = s.ap.slides.remove(de);
            s.ap.slides.insert(para, slide);
            for (sl, c) in s.ap.slides.iter_mut().zip(cabecas) {
                sl.secao = c;
            }
            s.atual = para;
            Ok((vec![para], true))
        })
    }

    /// Vai ao slide `indice` (não é edição: não entra no desfazer).
    pub fn slide_ir(&mut self, indice: usize) -> Result<(), String> {
        if indice >= self.ap.slides.len() {
            return Err(format!("O slide {} não existe.", indice + 1));
        }
        if indice != self.atual {
            self.atual = indice;
            self.selecao.clear();
        }
        Ok(())
    }

    /// Troca o layout do slide atual: as caixas do layout novo pegam o texto das de mesmo
    /// papel; as que sobram vazias saem, e as que sobram com texto ficam como caixa livre.
    pub fn slide_layout(&mut self, layout: &str) -> Result<Mudanca, String> {
        if !layouts::LAYOUTS.contains(&layout) {
            return Err(format!("Layout desconhecido: {layout}"));
        }
        self.editar(|s| {
            let (largura, altura) = s.ap.tamanho();
            let mut proximo = s.ap.proximo_id;
            let novas = layouts::elementos_do_layout(layout, largura, altura, &mut || {
                proximo += 1;
                proximo - 1
            })
            .unwrap_or_default();
            s.ap.proximo_id = proximo;
            let slide = s.slide_mut();
            let mut antigas: Vec<Elemento> = Vec::new();
            slide.elementos.retain(|e| {
                let do_layout = e.papel.do_layout();
                if do_layout {
                    antigas.push(e.clone());
                }
                !do_layout
            });
            for mut nova in novas {
                if let Some(i) = antigas.iter().position(|a| a.papel == nova.papel) {
                    let velha = antigas.remove(i);
                    nova.id = velha.id;
                    nova.texto = velha.texto;
                }
                slide.elementos.push(nova);
            }
            for mut sobra in antigas.into_iter().filter(|a| !a.texto.is_empty()) {
                sobra.papel = Papel::Livre;
                slide.elementos.push(sobra);
            }
            slide.layout = layout.into();
            // Apresentação de modelo: o enfeite do layout novo troca o do antigo.
            if !s.ap.modelo.is_empty() {
                let modelo = s.ap.modelo.clone();
                let mut copia = s.slide().clone();
                copia.elementos.retain(|e| e.papel != Papel::Fundo);
                crate::modelos::decorar(&mut s.ap, &mut copia, &modelo, layout);
                *s.slide_mut() = copia;
            }
            s.selecao.clear();
            Ok((vec![s.atual], false))
        })
    }

    /// O quadro de um slide novo com `layout`, no tema e no modelo desta apresentação (para a galeria).
    pub fn quadro_de_layout(&self, layout: &str) -> crate::quadro::Quadro {
        let mut leve = self.ap.clone();
        leve.slides.clear();
        let slide = slide_com_layout(&mut leve, layout);
        crate::quadro::quadro(&slide, self.ap.tamanho(), 0, &self.dicas, true)
    }

    /// O fundo do slide atual (ou de todos).
    pub fn slide_fundo(&mut self, cor: &str, todos: bool) -> Result<Mudanca, String> {
        self.editar(|s| {
            let alvos: Vec<usize> = if todos { (0..s.ap.slides.len()).collect() } else { vec![s.atual] };
            for &i in &alvos {
                s.ap.slides[i].fundo = cor.into();
                s.ap.slides[i].fundo_config = Default::default();
            }
            Ok((alvos, false))
        })
    }

    /// Oculta (ou mostra de novo) o slide atual na apresentação.
    pub fn slide_ocultar(&mut self) -> Result<Mudanca, String> {
        self.editar(|s| {
            let slide = s.slide_mut();
            slide.oculto = !slide.oculto;
            Ok((vec![s.atual], true))
        })
    }

    /// As anotações do apresentador do slide atual.
    pub fn notas_definir(&mut self, texto: &str) -> Result<Mudanca, String> {
        if self.slide().notas == texto {
            return Ok(Mudanca { versao: self.versao, slides: Vec::new(), lista: false });
        }
        self.editar(|s| {
            s.slide_mut().notas = texto.into();
            Ok((Vec::new(), false))
        })
    }

    /// Troca o tamanho do slide; o que está nos slides se ajusta na horizontal.
    pub fn proporcao_definir(&mut self, p: Proporcao) -> Result<Mudanca, String> {
        if self.ap.proporcao == p && self.ap.dimensoes == [0.0; 2] {
            return Ok(Mudanca { versao: self.versao, slides: Vec::new(), lista: false });
        }
        self.editar(|s| {
            let fator = p.tamanho().0 / s.ap.tamanho().0;
            let vertical = p.tamanho().1 / s.ap.tamanho().1;
            for slide in &mut s.ap.slides {
                for e in &mut slide.elementos {
                    e.x *= fator;
                    e.largura *= fator;
                    e.y *= vertical;
                    e.altura *= vertical;
                }
            }
            s.ap.proporcao = p;
            s.ap.dimensoes = [0.0; 2];
            Ok(((0..s.ap.slides.len()).collect(), true))
        })
    }

    /// O título da apresentação (as propriedades do arquivo).
    pub fn titulo_definir(&mut self, titulo: &str) -> Result<Mudanca, String> {
        self.editar(|s| {
            s.ap.titulo = titulo.into();
            Ok((Vec::new(), false))
        })
    }
}

impl Sessao {
    /// Acrescenta slides em um passo de desfazer, remapeando IDs sem colisões.
    pub fn apresentacao_importar(&mut self, origem: &crate::modelo::Apresentacao) -> Result<Mudanca, String> {
        self.editar(|s| {
            let inicio = s.ap.slides.len();
            let (ow, oh) = origem.tamanho();
            let (w, h) = s.ap.tamanho();
            for original in &origem.slides {
                let mut slide = original.clone();
                slide.id = s.ap.novo_id();
                let mut grupos = std::collections::HashMap::new();
                let mut grupo_novo = 0u32;
                for e in &mut slide.elementos {
                    e.id = s.ap.novo_id();
                    if e.grupo != 0 {
                        e.grupo = *grupos.entry(e.grupo).or_insert_with(|| { grupo_novo += 1; grupo_novo });
                    }
                    if e.destino > 0 { e.destino = if e.destino as usize <= origem.slides.len() { e.destino + inicio as u32 } else { 0 }; }
                    e.x *= w / ow; e.y *= h / oh; e.largura *= w / ow; e.altura *= h / oh;
                }
                s.ap.slides.push(slide);
            }
            for fonte in &origem.fontes { if !s.ap.fontes.iter().any(|f| f.nome == fonte.nome) { s.ap.fontes.push(fonte.clone()); } }
            Ok(((inicio..s.ap.slides.len()).collect(), !origem.slides.is_empty()))
        })
    }
}

#[cfg(test)]
mod testes_importacao {
    use super::*;
    #[test]
    fn importar_remapeia_ids_grupos_gatilhos_e_desfaz_o_conjunto() {
        let mut origem = Sessao::default(); origem.wordart_inserir().unwrap();
        origem.slide_novo("em_branco").unwrap();
        let ap = origem.apresentacao_mut();
        for e in &mut ap.slides[0].elementos { e.grupo = 7; e.destino = 2; }
        let original = ap.clone();
        let mut destino = Sessao::default(); let antes = destino.apresentacao().clone();
        destino.apresentacao_importar(&original).unwrap();
        assert_eq!(destino.ap.slides.len(), 3);
        let ids: Vec<_> = destino.ap.slides.iter().flat_map(|s| std::iter::once(s.id).chain(s.elementos.iter().map(|e| e.id))).collect();
        let unicos: std::collections::HashSet<_> = ids.iter().collect(); assert_eq!(ids.len(), unicos.len());
        assert!(destino.ap.slides[1].elementos.iter().all(|e| e.destino == 3));
        assert_eq!(origem.apresentacao(), &original);
        destino.desfazer().unwrap();
        // Desfazer restaura o conteúdo, mas NÃO retrocede datas/contadores
        // históricos de edição do documento.
        let mut restaurado = destino.apresentacao().clone();
        let revisoes = restaurado.metadados.revisoes;
        assert!(revisoes >= antes.metadados.revisoes);
        restaurado.metadados = antes.metadados.clone();
        assert_eq!(restaurado, antes);
        destino.refazer().unwrap(); assert_eq!(destino.ap.slides.len(), 3);
    }
}
