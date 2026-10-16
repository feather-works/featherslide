//! Desfazer e refazer: antes de cada edição a apresentação inteira é guardada (um
//! retrato). Uma apresentação tem dezenas de slides, não milhões de células — o
//! retrato custa pouco e não erra. Depois de desfazer, só os slides que de fato
//! mudaram ganham versão nova (as outras miniaturas ficam).

use std::collections::HashMap;
use std::time::Instant;

use crate::modelo::Slide;
use crate::sessao::{Mudanca, Retrato, Sessao};

/// Quantos passos o desfazer guarda.
const PASSOS: usize = 200;

impl Sessao {
    pub(crate) fn retrato(&self) -> Retrato {
        Retrato { ap: self.ap.clone(), atual: self.atual, selecao: self.selecao.clone() }
    }

    /// Roda `f` como uma edição: guarda o retrato antes e, se `f` recusar, nada fica.
    /// `f` devolve os índices dos slides tocados e se a lista de slides mudou.
    pub(crate) fn editar(&mut self, f: impl FnOnce(&mut Self) -> Result<(Vec<usize>, bool), String>) -> Result<Mudanca, String> {
        self.confirmar_previa();
        let antes = self.retrato();
        match f(self) {
            Ok((slides, lista)) => {
                for slide in &mut self.ap.slides { crate::outras_galerias::atualizar_conectores(slide); }
                self.desfazer.push(antes);
                if self.desfazer.len() > PASSOS {
                    self.desfazer.remove(0);
                }
                self.refazer.clear();
                Ok(self.mudou(slides, lista))
            }
            Err(e) => {
                self.restaurar(antes);
                Err(e)
            }
        }
    }

    /// Anota a mudança: versão nova, slides tocados com versão nova, documento sujo.
    pub(crate) fn mudou(&mut self, slides: Vec<usize>, lista: bool) -> Mudanca {
        self.versao += 1;
        // Estatísticas de edição são metadados automáticos: não dependem do teclado ou de login.
        let agora = Instant::now();
        if let Some(ultima) = self.ultima_edicao {
            // Intervalos longos representam inatividade; não contar tempo com o app ocioso.
            let intervalo = agora.saturating_duration_since(ultima).as_secs();
            if (1..=300).contains(&intervalo) {
                self.ap.metadados.tempo_edicao_segundos = self.ap.metadados
                    .tempo_edicao_segundos.saturating_add(intervalo);
            }
        }
        self.ultima_edicao = Some(agora);
        self.ap.metadados.modificado_em_utc = crate::metadados::agora_utc();
        self.ap.metadados.revisoes = self.ap.metadados.revisoes.saturating_add(1);
        if !self.ap.autor.is_empty() {
            self.ap.metadados.ultimo_autor.clone_from(&self.ap.autor);
        }
        if self.ap.metadados.criado_em_utc == 0 {
            self.ap.metadados.criado_em_utc = self.ap.metadados.modificado_em_utc;
        }
        if self.ap.metadados.identificador.is_empty() {
            self.ap.metadados.identificador = crate::metadados::novo_identificador();
        }
        for &i in &slides {
            if let Some(s) = self.ap.slides.get(i) {
                *self.versoes.entry(s.id).or_insert(0) += 1;
            }
        }
        self.sujo = true;
        Mudanca { versao: self.versao, slides, lista }
    }

    fn restaurar(&mut self, r: Retrato) {
        self.ap = r.ap;
        self.atual = r.atual.min(self.ap.slides.len().saturating_sub(1));
        self.selecao = r.selecao;
    }

    /// Troca o estado atual pelo retrato e devolve o que mudou de fato.
    fn trocar(&mut self, r: Retrato) -> Mudanca {
        let antigos: HashMap<u64, Slide> = self.ap.slides.iter().map(|s| (s.id, s.clone())).collect();
        let ordem_antiga: Vec<u64> = self.ap.slides.iter().map(|s| s.id).collect();
        // Desfazer altera o conteúdo, mas não volta o relógio nem o contador de revisões.
        let revisoes = self.ap.metadados.revisoes;
        let tempo = self.ap.metadados.tempo_edicao_segundos;
        let criado = self.ap.metadados.criado_em_utc;
        let id = self.ap.metadados.identificador.clone();
        self.restaurar(r);
        self.ap.metadados.revisoes = self.ap.metadados.revisoes.max(revisoes);
        self.ap.metadados.tempo_edicao_segundos = self.ap.metadados.tempo_edicao_segundos.max(tempo);
        if criado != 0 { self.ap.metadados.criado_em_utc = criado; }
        if !id.is_empty() { self.ap.metadados.identificador = id; }
        let tocados: Vec<usize> = self.ap.slides.iter().enumerate().filter(|(_, s)| antigos.get(&s.id) != Some(s)).map(|(i, _)| i).collect();
        let lista = ordem_antiga != self.ap.slides.iter().map(|s| s.id).collect::<Vec<_>>();
        self.mudou(tocados, lista)
    }

    /// Fecha a prévia em andamento: o estado de antes dela vira um único passo do desfazer
    /// (cor antiga → cor nova), não um passo por movimento do mouse.
    pub fn confirmar_previa(&mut self) {
        if let Some(antes) = self.previa.take() {
            self.desfazer.push(antes);
            if self.desfazer.len() > PASSOS {
                self.desfazer.remove(0);
            }
            self.refazer.clear();
        }
    }

    /// Descarta a prévia sem criar um passo de desfazer nem alterar os metadados originais.
    pub fn cancelar_previa(&mut self)->Option<Mudanca>{
        let antes=self.previa.take()?;
        let meta=antes.ap.metadados.clone();let sujo=self.previa_sujo;
        let m=self.trocar(antes);self.ap.metadados=meta;self.sujo=sujo;Some(m)
    }

    /// Desfaz a última edição (`None` se não há o que desfazer).
    pub fn desfazer(&mut self) -> Option<Mudanca> {
        self.confirmar_previa();
        let r = self.desfazer.pop()?;
        let agora = self.retrato();
        self.refazer.push(agora);
        Some(self.trocar(r))
    }

    /// Refaz o que foi desfeito (`None` se não há o que refazer).
    pub fn refazer(&mut self) -> Option<Mudanca> {
        self.confirmar_previa();
        let r = self.refazer.pop()?;
        let agora = self.retrato();
        self.desfazer.push(agora);
        Some(self.trocar(r))
    }
}
