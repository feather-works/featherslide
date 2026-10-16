//! Formas básicas adicionais do FeatherSlide (FTS-0285, 0298–0312). A geometria vem do
//! `feather-shapes` (`basicas`); aqui o `Elemento` vira primitivas e teste de acerto.

use crate::dados::Primitiva;
use crate::modelo::{Elemento, Tipo};

pub use feather_shapes::basicas::Desenho;

use crate::formas_comuns::forma_do_tipo;

/// Quais elementos nativos possuem geometria adicional centralizada no `feather-shapes`.
pub fn suporta(tipo: Tipo) -> bool {
    forma_do_tipo(tipo).is_some_and(feather_shapes::basicas::suporta)
}

/// Calcula o desenho de uma forma no retângulo. Furos possuem orientação oposta.
pub fn desenho(tipo: Tipo, x: f32, y: f32, w: f32, h: f32) -> Option<Desenho> {
    feather_shapes::basicas::desenho(forma_do_tipo(tipo)?, x, y, w, h)
}

/// O desenho nativo convertido em primitivas consumidas por Iced, PDF e outras saídas.
pub fn primitivas(e: &Elemento) -> Vec<Primitiva> {
    let Some(d) = desenho(e.tipo, e.x, e.y, e.largura, e.altura) else { return Vec::new() };
    let mut saida = Vec::new();
    if !d.contornos.is_empty() {
        saida.push(Primitiva::Caminho {
            contornos: d.contornos,
            preenchimento: e.preenchimento.clone(),
            contorno: e.contorno.clone(),
            espessura: e.espessura,
        });
    }
    for traco in d.tracos {
        saida.push(Primitiva::Poli {
            pontos: traco, cor: e.contorno.clone(), espessura: e.espessura,
        });
    }
    saida
}

/// Testa acerto do mouse no preenchimento real (inclusive furos) ou no arco aberto.
pub fn contem_ponto(e: &Elemento, x: f32, y: f32) -> bool {
    let Some(forma) = forma_do_tipo(e.tipo) else { return false };
    feather_shapes::basicas::contem_ponto(forma, [e.x, e.y, e.largura, e.altura], e.rotacao, e.espessura, x, y)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn element(tipo: Tipo) -> Elemento {
        Elemento { tipo, x: 0.0, y: 0.0, largura: 200.0, altura: 200.0,
            contorno: "#202124".into(), preenchimento: "#4285f4".into(), espessura: 2.0,
            ..Elemento::default() }
    }

    #[test]
    fn todos_os_novos_tipos_tem_geometria_editavel() {
        for &(tipo,_) in crate::formas::ADICIONAIS {
            if !suporta(tipo) { continue; }
            let e=element(tipo);
            let d=desenho(tipo, 10.0, 20.0, 180.0, 120.0).unwrap();
            assert!(d.contornos.iter().all(|p| p.len()>=3));
            assert!(d.tracos.iter().all(|p| p.len()>=2));
            assert!(!d.contornos.is_empty()||!d.tracos.is_empty());
            assert!(d.contornos.iter().chain(&d.tracos).flatten()
                .all(|[x,y]| x.is_finite()&&y.is_finite()&&*x>=9.9&&*x<=190.1&&*y>=19.9&&*y<=140.1),"{tipo:?}");
            assert!(!primitivas(&e).is_empty(),"{tipo:?}");
        }
    }

    #[test]
    fn anel_e_moldura_tem_buraco_real() {
        for tipo in [Tipo::Anel,Tipo::Moldura] {
            let e=element(tipo);
            assert!(!contem_ponto(&e,100.0,100.0),"{tipo:?}");
            assert!(contem_ponto(&e,100.0,15.0),"{tipo:?}");
            let d=desenho(tipo,0.0,0.0,200.0,200.0).unwrap();
            assert!(feather_shapes::basicas::area(&d.contornos[0])>0.0 && feather_shapes::basicas::area(&d.contornos[1])<0.0);
        }
    }

    #[test]
    fn arco_aberto_nao_preenche() {
        let e=element(Tipo::Arco);
        let d=desenho(Tipo::Arco,0.0,0.0,200.0,200.0).unwrap();
        assert!(d.contornos.is_empty());
        assert_eq!(d.tracos.len(),1);
        assert!(!contem_ponto(&e,100.0,100.0));
        assert_eq!(primitivas(&e).len(),1);
    }
}
