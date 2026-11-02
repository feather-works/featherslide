//! WordArt do Slide: o contorno de cada linha do elemento, guarda em cache e entrega a
//! `Primitiva`. O contorno das letras vem do `feather-ui` (`texto_contorno`) e a deformação do
//! `feather-shapes` (`wordart::transformar`): o mesmo desenho no palco, apresentação e exportação.
use std::cell::RefCell;
use std::collections::HashMap;
use featherslide_core::dados::Primitiva;
use featherslide_core::quadro::ElementoDesenhado;
use featherslide_core::wordart::{self, Transformacao};
use pelicano::{Point, Renderer};

thread_local! {
    static CACHE: RefCell<HashMap<String, Vec<Primitiva>>> = RefCell::new(HashMap::new());
    // shortcut: renderer só para medir contornos (carrega as fontes do sistema uma vez); fontes
    // embutidas registradas depois com `load_font` só valem se o pelicano recarregar aqui também.
    static MEDIDOR: RefCell<Renderer> = RefCell::new(Renderer::new());
}

pub fn wordart(e: &ElementoDesenhado) -> Vec<Primitiva> {
    let chave = format!("{:?}|{}|{}|{}|{}|{}|{}|{}|{:?}|{:?}", e.linhas, e.fonte, e.tamanho, e.negrito, e.italico, e.cor, e.contorno, e.espessura, e.ajustes, (e.base, &e.wordart, &e.sombra));
    CACHE.with(|cache| {
        if let Some(v) = cache.borrow().get(&chave) { return v.clone(); }
        let v = criar(e);
        let mut cache = cache.borrow_mut();
        if cache.len() >= 128 { cache.clear(); }
        cache.insert(chave, v.clone());
        v
    })
}

fn criar(e: &ElementoDesenhado) -> Vec<Primitiva> {
    let modo = Transformacao::ler(&e.wordart);
    let intensidade = e.ajustes.first().copied().unwrap_or(0.5);
    let mut partes = Vec::new();
    for linha in &e.linhas {
        let em = Point::new(linha.x, linha.y - e.tamanho * 0.8);
        let curvas = MEDIDOR.with(|r| {
            feather_ui::texto_contorno::contornos_deformados(&mut r.borrow_mut(), &linha.texto, e.tamanho, crate::desenho::fonte_do_elemento(e), em, |q| wordart::transformar(q, e.base, modo, intensidade))
        });
        if let Some(s) = &e.sombra {
            let sombra = curvas.iter().map(|p| p.iter().map(|[x, y]| [x + s.dx, y + s.dy]).collect()).collect();
            partes.push(Primitiva::Caminho { contornos: sombra, preenchimento: s.cor.clone(), contorno: String::new(), espessura: 0.0 });
        }
        partes.push(Primitiva::Caminho { contornos: curvas, preenchimento: e.cor.clone(), contorno: e.contorno.clone(), espessura: e.espessura });
    }
    partes
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn letra_o_preserva_miolo_e_transformacao_muda_contorno() {
        let mut s = featherslide_core::sessao::Sessao::default();
        s.wordart_inserir().unwrap();
        let id = s.estado().selecao[0];
        s.texto_definir(id, "O").unwrap();
        let q = s.quadro(s.atual(), None, false).unwrap().unwrap();
        let e = q.elementos.iter().find(|e| e.id == id).unwrap();
        let reto = wordart(e);
        let Primitiva::Caminho { contornos, .. } = &reto[0] else { panic!("esperado contorno") };
        assert!(contornos.len() >= 2, "O precisa de contornos externo e interno");
        s.wordart_transformacao(Transformacao::Arco, 1.0).unwrap();
        let q = s.quadro(s.atual(), None, false).unwrap().unwrap();
        assert_ne!(reto, wordart(q.elementos.iter().find(|e| e.id == id).unwrap()));
    }
}
