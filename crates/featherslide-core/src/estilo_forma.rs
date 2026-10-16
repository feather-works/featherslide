//! Aparência de formas: os dados e a validação moram no `feather-shapes`; aqui ficam o
//! que o Slide aceita formatar, a transação e a prévia.
use crate::modelo::Tipo;
use crate::sessao::{Mudanca, Sessao};

/// Aparência, gradiente e validação de cor vêm do `feather-shapes`.
pub use feather_shapes::aparencia::{validar_cor, Aparencia, Gradiente, PontoCor, TipoGradiente};

pub fn aceita(tipo: Tipo) -> bool {
    !matches!(
        tipo,
        Tipo::Imagem
            | Tipo::Icone
            | Tipo::Caminho
            | Tipo::Tabela
            | Tipo::Grafico
            | Tipo::WordArt
            | Tipo::Linha
            | Tipo::Arco
    )
}
impl Sessao {
    pub fn forma_aparencia(&mut self, a: Aparencia) -> Result<Mudanca, String> {
        self.validar_aparencia(&a)?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                e.preenchimento = a.preenchimento.clone();
                e.gradiente = a.gradiente.clone();
                e.contorno = a.contorno.clone();
                e.espessura = a.espessura;
                e.opacidade = a.opacidade;
                e.sombra = a.sombra.clone();
            }
            Ok((vec![s.atual], false))
        })
    }
    fn validar_aparencia(&self, a: &Aparencia) -> Result<(), String> {
        self.exigir_selecao()?;
        a.validar()?;
        if self
            .slide()
            .elementos
            .iter()
            .any(|e| self.estado().selecao.contains(&e.id) && !aceita(e.tipo))
        {
            return Err("Selecione somente formas ou caixas de texto.".into());
        }
        Ok(())
    }
    pub fn forma_aparencia_previa(&mut self, a: Aparencia) -> Result<Mudanca, String> {
        self.validar_aparencia(&a)?;
        let mudou = self
            .slide()
            .elementos
            .iter()
            .filter(|e| self.selecao.contains(&e.id))
            .any(|e| {
                e.preenchimento != a.preenchimento
                    || e.gradiente != a.gradiente
                    || e.contorno != a.contorno
                    || e.espessura != a.espessura
                    || e.opacidade != a.opacidade
                    || e.sombra != a.sombra
            });
        if !mudou {
            return Ok(Mudanca {
                versao: self.versao,
                slides: Vec::new(),
                lista: false,
            });
        }
        if self.previa.is_none() {
            self.previa_sujo = self.sujo;
            self.previa = Some(self.retrato());
        }
        for e in self.selecionados_mut() {
            e.preenchimento = a.preenchimento.clone();
            e.gradiente = a.gradiente.clone();
            e.contorno = a.contorno.clone();
            e.espessura = a.espessura;
            e.opacidade = a.opacidade;
            e.sombra = a.sombra.clone();
        }
        Ok(self.mudou(vec![self.atual], false))
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::modelo::{Sombra, Tipo};
    fn aparencia() -> Aparencia {
        Aparencia {
            preenchimento: "#4285f4".into(),
            gradiente: Some(Gradiente::default()),
            contorno: "#112233".into(),
            espessura: 2.0,
            opacidade: 0.7,
            sombra: Sombra {
                cor: "#00000080".into(),
                desfoque: 4.0,
                dx: 3.0,
                dy: 5.0,
            },
        }
    }
    #[test]
    fn aparencia_transacao_desfazer_refazer() {
        let mut s = Sessao::default();
        s.elemento_inserir(Tipo::Retangulo, None, None).unwrap();
        let antes = s.slide().elementos.last().unwrap().clone();
        s.forma_aparencia(aparencia()).unwrap();
        let depois = s.slide().elementos.last().unwrap().clone();
        assert!(depois.gradiente.is_some());
        assert_eq!(depois.opacidade, 0.7);
        s.desfazer().unwrap();
        assert_eq!(*s.slide().elementos.last().unwrap(), antes);
        s.refazer().unwrap();
        assert_eq!(*s.slide().elementos.last().unwrap(), depois);
    }
    #[test]
    fn recusa_gradiente_invalido_sem_edicao() {
        let mut s = Sessao::default();
        s.elemento_inserir(Tipo::Elipse, None, None).unwrap();
        let antes = s.apresentacao().clone();
        let mut a = aparencia();
        a.gradiente.as_mut().unwrap().pontos[0].posicao = f32::NAN;
        assert!(s.forma_aparencia(a).is_err());
        assert_eq!(*s.apresentacao(), antes);
        let mut a = aparencia();
        a.opacidade = 2.0;
        assert!(s.forma_aparencia(a).is_err());
        assert_eq!(*s.apresentacao(), antes);
    }
    #[test]
    fn cor_solida_remove_gradiente() {
        let mut s = Sessao::default();
        s.elemento_inserir(Tipo::Retangulo, None, None).unwrap();
        s.forma_aparencia(aparencia()).unwrap();
        s.formatar(crate::Formatacao::Preenchimento("#ff0000".into()))
            .unwrap();
        assert!(s.slide().elementos.last().unwrap().gradiente.is_none());
    }
}

#[cfg(test)]
mod testes_previa {
    use super::*;
    use crate::modelo::Sombra;
    #[test]
    fn cancelar_restaura_documento_limpo_e_nao_cria_undo() {
        let mut s = Sessao::default();
        s.elemento_inserir(Tipo::Retangulo, None, None).unwrap();
        s.marcar_gravado();
        let antes = s.apresentacao().clone();
        let e = s.slide().elementos.last().unwrap();
        let a = Aparencia {
            preenchimento: e.preenchimento.clone(),
            gradiente: Some(Gradiente::default()),
            contorno: e.contorno.clone(),
            espessura: e.espessura,
            opacidade: 1.0,
            sombra: Sombra::default(),
        };
        s.forma_aparencia_previa(a.clone()).unwrap();
        let mut b = a;
        b.gradiente.as_mut().unwrap().angulo = 75.0;
        s.forma_aparencia_previa(b).unwrap();
        assert!(s.slide().elementos.last().unwrap().gradiente.is_some());
        s.cancelar_previa().unwrap();
        assert_eq!(*s.apresentacao(), antes);
        assert!(!s.estado().sujo);
        s.desfazer().unwrap();
        assert!(s
            .slide()
            .elementos
            .iter()
            .all(|e| e.tipo != Tipo::Retangulo));
    }
    #[test]
    fn movimento_continuo_confirma_um_so_passo() {
        let mut s = Sessao::default();
        s.elemento_inserir(Tipo::Retangulo, None, None).unwrap();
        let antes = s.slide().clone();
        let e = s.slide().elementos.last().unwrap();
        let mut a = Aparencia {
            preenchimento: e.preenchimento.clone(),
            gradiente: Some(Gradiente::default()),
            contorno: e.contorno.clone(),
            espessura: e.espessura,
            opacidade: 1.0,
            sombra: Sombra::default(),
        };
        for angulo in [0.0, 30.0, 60.0, 90.0] {
            a.gradiente.as_mut().unwrap().angulo = angulo;
            s.forma_aparencia_previa(a.clone()).unwrap();
        }
        s.confirmar_previa();
        s.desfazer().unwrap();
        assert_eq!(*s.slide(), antes);
        s.refazer().unwrap();
        assert_eq!(
            s.slide()
                .elementos
                .last()
                .unwrap()
                .gradiente
                .as_ref()
                .unwrap()
                .angulo,
            90.0
        );
    }
}
