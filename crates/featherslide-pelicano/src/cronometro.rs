//! Tempo do apresentador, independente do avanço automático dos slides.
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Cronometro {
    acumulado: Duration,
    inicio: Option<Instant>,
}

impl Cronometro {
    pub fn reiniciar(&mut self) { self.reiniciar_em(Instant::now()); }
    fn reiniciar_em(&mut self, agora: Instant) { self.acumulado = Duration::ZERO; self.inicio = Some(agora); }
    pub fn alternar(&mut self) { self.alternar_em(Instant::now()); }
    fn alternar_em(&mut self, agora: Instant) {
        match self.inicio.take() {
            Some(inicio) => self.acumulado += agora.saturating_duration_since(inicio),
            None => self.inicio = Some(agora),
        }
    }
    pub fn pausado(&self) -> bool { self.inicio.is_none() }
    pub fn decorrido(&self) -> Duration { self.decorrido_em(Instant::now()) }
    fn decorrido_em(&self, agora: Instant) -> Duration {
        self.acumulado + self.inicio.map_or(Duration::ZERO, |i| agora.saturating_duration_since(i))
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn pausa_retomada_e_reinicio_sem_contar_intervalo_pausado() {
        let t = Instant::now();
        let mut c = Cronometro::default();
        c.reiniciar_em(t);
        c.alternar_em(t + Duration::from_secs(10));
        assert!(c.pausado());
        assert_eq!(c.decorrido_em(t + Duration::from_secs(40)).as_secs(), 10);
        c.alternar_em(t + Duration::from_secs(50));
        assert_eq!(c.decorrido_em(t + Duration::from_secs(55)).as_secs(), 15);
        c.reiniciar_em(t + Duration::from_secs(60));
        assert_eq!(c.decorrido_em(t + Duration::from_secs(61)).as_secs(), 1);
    }
}
