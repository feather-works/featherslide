//! Colagem e cópia de aparência, sem transportar conteúdo ou geometria.
use crate::{
    modelo::{EstiloDoTexto, Sombra, Tipo},
    sessao::{Mudanca, Sessao},
};

#[derive(Clone, Debug)]
pub struct Pincel {
    pub estilo: EstiloDoTexto,
    pub preenchimento: String,
    pub contorno: String,
    pub espessura: f32,
    pub sombra: Sombra,
}

impl Sessao {
    pub fn pincel_capturar(&self) -> Result<Pincel, String> {
        let e = self
            .slide()
            .elementos
            .iter()
            .find(|e| self.estado().selecao.contains(&e.id))
            .ok_or("Selecione o elemento de origem da formatação.")?;
        Ok(Pincel {
            estilo: e.estilo.clone(),
            preenchimento: e.preenchimento.clone(),
            contorno: e.contorno.clone(),
            espessura: e.espessura,
            sombra: e.sombra.clone(),
        })
    }

    pub fn pincel_aplicar(&mut self, id: u64, p: &Pincel) -> Result<Mudanca, String> {
        self.editar(|s| {
            let e = s
                .slide_mut()
                .elementos
                .iter_mut()
                .find(|e| e.id == id)
                .ok_or("Esse elemento não está no slide.")?;
            e.estilo = p.estilo.clone();
            e.preenchimento = p.preenchimento.clone();
            e.contorno = p.contorno.clone();
            e.espessura = p.espessura;
            e.sombra = p.sombra.clone();
            Ok((vec![s.atual], false))
        })
    }

    pub fn colar_sem_formatacao(&mut self, texto: &str) -> Result<Mudanca, String> {
        let texto = texto.replace("\r\n", "\n").replace('\r', "\n");
        let texto: String = texto
            .chars()
            .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
            .collect();
        if texto.trim().is_empty() {
            return Err("Não há texto para colar.".into());
        }
        self.elemento_inserir_com_estilo(
            Tipo::Texto,
            None,
            texto,
            None,
            None,
            None,
            None,
            None,
            vec![],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn colagem_ignora_copia_interna_e_desfaz_em_um_passo() {
        let mut s = Sessao::default();
        s.colar_sem_formatacao("ação\r\ntexto").unwrap();
        let id = s.estado().selecao[0];
        s.formatar(crate::Formatacao::Negrito).unwrap();
        s.copiar();
        s.colar_sem_formatacao("ação\r\ntexto").unwrap();
        let novo = s
            .slide()
            .elementos
            .iter()
            .find(|e| e.id == s.estado().selecao[0])
            .unwrap();
        assert!(!novo.estilo.negrito);
        assert_eq!(novo.texto, "ação\ntexto");
        s.desfazer().unwrap();
        assert!(s.slide().elementos.iter().any(|e| e.id == id));
        assert_eq!(
            s.slide()
                .elementos
                .iter()
                .filter(|e| e.texto == "ação\ntexto")
                .count(),
            1
        );
    }
    #[test]
    fn original_preserva_estilo_e_recusa_texto_externo() {
        let mut s = Sessao::default();
        s.colar_sem_formatacao("ação").unwrap();
        s.formatar(crate::Formatacao::Negrito).unwrap();
        let copiado = s.copiar();
        s.colar_com_formatacao_original(&copiado).unwrap();
        assert!(
            s.slide()
                .elementos
                .iter()
                .find(|e| e.id == s.estado().selecao[0])
                .unwrap()
                .estilo
                .negrito
        );
        let antes = s.apresentacao().clone();
        assert!(s.colar_com_formatacao_original("texto externo").is_err());
        assert_eq!(s.apresentacao(), &antes);
    }
    #[test]
    fn pincel_preserva_conteudo_e_geometria() {
        let mut s = Sessao::default();
        s.colar_sem_formatacao("origem").unwrap();
        s.formatar(crate::Formatacao::Negrito).unwrap();
        let pincel = s.pincel_capturar().unwrap();
        s.colar_sem_formatacao("destino").unwrap();
        let id = s.estado().selecao[0];
        let antes = s
            .slide()
            .elementos
            .iter()
            .find(|e| e.id == id)
            .unwrap()
            .clone();
        s.pincel_aplicar(id, &pincel).unwrap();
        let depois = s.slide().elementos.iter().find(|e| e.id == id).unwrap();
        assert_eq!(depois.texto, antes.texto);
        assert_eq!(
            (depois.x, depois.y, depois.largura, depois.altura),
            (antes.x, antes.y, antes.largura, antes.altura)
        );
        assert!(depois.estilo.negrito);
        s.desfazer().unwrap();
        assert!(
            !s.slide()
                .elementos
                .iter()
                .find(|e| e.id == id)
                .unwrap()
                .estilo
                .negrito
        );
    }
}
