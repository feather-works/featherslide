//! Design, Transições e Animações: comandos da sessão que mexem no tema, na transição de
//! cada slide e na animação de cada elemento. A reprodução (o que se vê na apresentação) é
//! da interface; aqui está o que se grava e a ordem em que as animações acontecem.

use crate::modelo::{Animacao, InicioAnimacao, Papel, TipoAnimacao, Transicao};
use crate::sessao::{Mudanca, Sessao};
use crate::temas;

/// Um passo da apresentação de um slide: os elementos que entram juntos (com o atraso de
/// cada um, em segundos, contado desde o começo do passo) e quanto o passo dura no total.
#[derive(Clone, Debug, PartialEq)]
pub struct PassoDeAnimacao {
    pub elementos: Vec<(u64, f32)>,
    pub duracao: f32,
}

impl Sessao {
    /// O tema da apresentação (id).
    pub fn tema_atual(&self) -> &str {
        if self.ap.tema.is_empty() { temas::TEMAS[0].id } else { &self.ap.tema }
    }

    /// Aplica o tema à apresentação inteira.
    pub fn tema_aplicar(&mut self, id: &str) -> Result<Mudanca, String> {
        if !temas::TEMAS.iter().any(|t| t.id == id) {
            return Err("Esse tema não existe.".into());
        }
        self.editar(|s| {
            temas::aplicar(&mut s.ap, id);
            Ok(((0..s.ap.slides.len()).collect(), false))
        })
    }

    /// A transição do slide atual (e, com `todos`, de todos os slides).
    pub fn transicao_definir(&mut self, t: Transicao, todos: bool) -> Result<Mudanca, String> {
        if !(0.05..=10.0).contains(&t.duracao) {
            return Err("A duração da transição deve ficar entre 0,05 e 10 segundos.".into());
        }
        self.editar(|s| {
            let alvos: Vec<usize> = if todos { (0..s.ap.slides.len()).collect() } else { vec![s.atual] };
            for &i in &alvos {
                s.ap.slides[i].transicao = t.clone();
            }
            Ok((alvos, false))
        })
    }

    /// Os tempos de um ensaio (id do slide, segundos na tela) viram o "avançar depois de" de
    /// cada slide, numa edição só (um Ctrl+Z desfaz). Slides fora do ensaio não mudam.
    pub fn tempos_do_ensaio(&mut self, tempos: &[(u64, f32)]) -> Result<Mudanca, String> {
        if tempos.is_empty() {
            return Err("O ensaio não passou por nenhum slide.".into());
        }
        self.editar(|s| {
            let mut mudados = Vec::new();
            for (i, slide) in s.ap.slides.iter_mut().enumerate() {
                if let Some(&(_, seg)) = tempos.iter().find(|(id, _)| *id == slide.id) {
                    // 0 quer dizer "ao clicar": um slide passado rápido fica com 0,1 s.
                    slide.transicao.apos = ((seg * 10.0).round() / 10.0).clamp(0.1, 3600.0);
                    mudados.push(i);
                }
            }
            Ok((mudados, false))
        })
    }

    /// A animação dos elementos selecionados.
    pub fn animacao_definir(&mut self, a: Animacao) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        if !(0.05..=20.0).contains(&a.duracao) || !(0.0..=30.0).contains(&a.atraso) {
            return Err("Duração ou atraso fora do limite.".into());
        }
        self.editar(|s| {
            for e in s.selecionados_mut() {
                e.animacao = a.clone();
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Troca a ordem da animação do elemento `id` com a do vizinho animado (`-1` antes, `1` depois).
    /// A ordem da animação é a ordem dos elementos no slide (a mesma da pilha).
    pub fn animacao_ordem(&mut self, id: u64, delta: i32) -> Result<Mudanca, String> {
        self.editar(|s| {
            let animados: Vec<usize> = s.slide().elementos.iter().enumerate().filter(|(_, e)| e.animacao.tipo != TipoAnimacao::Nenhuma).map(|(i, _)| i).collect();
            let pos = animados.iter().position(|&i| s.slide().elementos[i].id == id).ok_or("Esse elemento não tem animação.")?;
            let alvo = pos as i32 + delta.signum();
            if alvo < 0 || alvo as usize >= animados.len() {
                return Err("Já está no limite da ordem.".into());
            }
            s.slide_mut().elementos.swap(animados[pos], animados[alvo as usize]);
            Ok((vec![s.atual], false))
        })
    }

    /// Os elementos do slide `indice` com animação, na ordem em que acontecem.
    pub fn animacoes_do_slide(&self, indice: usize) -> Vec<(u64, Animacao)> {
        self.ap.slides.get(indice).map(|s| s.elementos.iter().filter(|e| e.papel != Papel::Fundo && e.animacao.tipo != TipoAnimacao::Nenhuma).map(|e| (e.id, e.animacao.clone())).collect()).unwrap_or_default()
    }

    /// Os passos da apresentação do slide `indice`: cada clique dispara um passo. "Ao clicar"
    /// abre um passo novo; "com a anterior" entra junto; "após a anterior" entra quando o
    /// anterior termina. O primeiro elemento sempre abre um passo.
    pub fn passos_de_animacao(&self, indice: usize) -> Vec<PassoDeAnimacao> {
        let mut passos: Vec<PassoDeAnimacao> = Vec::new();
        let mut fim_do_anterior = 0.0_f32;
        for (id, a) in self.animacoes_do_slide(indice) {
            let abre = a.inicio == InicioAnimacao::AoClicar || passos.is_empty();
            if abre {
                passos.push(PassoDeAnimacao { elementos: Vec::new(), duracao: 0.0 });
                fim_do_anterior = 0.0;
            }
            let passo = passos.last_mut().expect("passo aberto");
            let comeco = match a.inicio {
                InicioAnimacao::AposAnterior if !abre => fim_do_anterior + a.atraso,
                _ => a.atraso,
            };
            passo.elementos.push((id, comeco));
            fim_do_anterior = comeco + a.duracao;
            passo.duracao = passo.duracao.max(fim_do_anterior);
        }
        passos
    }
}

#[cfg(test)]
mod testes {
    use crate::sessao::Sessao;

    #[test]
    fn tempos_do_ensaio_viram_avanco_automatico_com_um_desfazer() {
        let mut s = Sessao::default();
        s.slide_novo("em_branco").unwrap();
        s.slide_novo("em_branco").unwrap();
        let ids: Vec<u64> = s.apresentacao().slides.iter().map(|sl| sl.id).collect();
        s.tempos_do_ensaio(&[(ids[0], 12.34), (ids[2], 0.0)]).unwrap();
        let apos: Vec<f32> = s.apresentacao().slides.iter().map(|sl| sl.transicao.apos).collect();
        assert_eq!(apos, vec![12.3, 0.0, 0.1], "o slide 2 ficou fora do ensaio; o 3 passou rápido");
        s.desfazer().unwrap();
        assert!(s.apresentacao().slides.iter().all(|sl| sl.transicao.apos == 0.0));
        assert!(s.tempos_do_ensaio(&[]).is_err());
    }
}
