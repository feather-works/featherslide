//! Prévia temporária dos efeitos ao passar o mouse: não escreve na apresentação nem no histórico.

use crate::apresentacao::Programa;
use crate::desenho::Efeitos;
use featherslide_core::modelo::{Animacao, Transicao};
use featherslide_core::quadro::Quadro;
use pelicano::{Point, Program, Rect, Renderer, Theme};
use std::time::Instant;

const ESPERA: f32 = 0.0;

/// O efeito e os quadros capturados quando o mouse entra no botão.
pub enum Efeito {
    Transicao(Quadro, Transicao),
    Animacao(Vec<u64>, Animacao),
}

/// Uma prévia isolada da sessão de edição.
pub struct Previa {
    quadro: Quadro,
    efeito: Efeito,
    inicio: Instant,
}

impl Previa {
    /// Começa a prévia imediatamente; a reprodução segue até o fim mesmo ao sair do botão.
    pub fn nova(quadro: Quadro, efeito: Efeito) -> Self {
        Self {
            quadro,
            efeito,
            inicio: Instant::now(),
        }
    }

    fn duracao(&self) -> f32 {
        match &self.efeito {
            Efeito::Transicao(_, t) => t.duracao,
            Efeito::Animacao(_, a) => a.duracao,
        }
        .max(0.05)
    }

    /// Diz quando parar o temporizador e devolver o palco ao editor.
    pub fn terminou(&self) -> bool {
        self.inicio.elapsed().as_secs_f32() >= ESPERA + self.duracao()
    }

    /// O quadro da prévia reutiliza exatamente o renderizador da apresentação.
    pub fn programa(&self) -> Option<Programa> {
        let tempo = self.inicio.elapsed().as_secs_f32() - ESPERA;
        if tempo < 0.0 {
            return None;
        }
        let p = (tempo / self.duracao()).clamp(0.0, 1.0);
        let mut efeitos = Efeitos::new();
        let transicao = match &self.efeito {
            Efeito::Transicao(de, t) => Some((de.clone(), t.clone(), p)),
            Efeito::Animacao(ids, a) => {
                for id in ids {
                    efeitos.insert(*id, (p, a.clone()));
                }
                None
            }
        };
        Some(Programa {
            quadro: self.quadro.clone(),
            fundo_externo: crate::tema::COR_MESA(),
            escala_relativa: 0.95,
            pan: (0.0, 0.0),
            efeitos,
            transicao,
            tela: None,
            gatilhos: vec![],
            links: vec![],
        })
    }
}

/// Canvas de prévia sem gestos de navegação ou edição.
pub struct CanvasPrevia(pub Programa);

impl Program<()> for CanvasPrevia {
    type State = ();

    fn draw(&self, _: &(), renderer: &mut Renderer, bounds: Rect, tema: &Theme, cursor: Option<Point>) {
        Program::<crate::apresentacao::Acao>::draw(&self.0, &0.0, renderer, bounds, tema, cursor)
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use featherslide_core::{modelo::TipoAnimacao, sessao::Sessao};
    use std::time::Duration;

    #[test]
    fn espera_e_anima_apenas_os_elementos_indicados() {
        let s = Sessao::default();
        let q = s.quadro(0, None, false).unwrap().unwrap();
        let id = q.elementos[0].id;
        let mut p = Previa::nova(
            q,
            Efeito::Animacao(
                vec![id],
                Animacao {
                    tipo: TipoAnimacao::Zoom,
                    duracao: 1.0,
                    ..Animacao::default()
                },
            ),
        );
        assert!(p.programa().is_some());
        p.inicio = Instant::now() - Duration::from_millis(500);
        let programa = p.programa().unwrap();
        assert_eq!(programa.fundo_externo, crate::tema::COR_MESA());
        assert_eq!(programa.efeitos.len(), 1);
        assert!((0.49..0.52).contains(&programa.efeitos[&id].0));
        assert!(!p.terminou());
        p.inicio = Instant::now() - Duration::from_secs(2);
        assert!(p.terminou());
        assert!(!s.estado().sujo);
    }
}
