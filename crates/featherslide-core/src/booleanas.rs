//! Operações booleanas de formas vetoriais, sem rasterização. O recorte mora no
//! `feather-shapes` (`booleanas`); aqui ficam o `Elemento` e o desfazer.
//! O caminho resultante é gravado no campo \`dados\` de um Elemento::Caminho,
//! em coordenadas normalizadas. Assim, mover/redimensionar/duplicar continuam nativos.

use feather_shapes::booleanas::{self as motor, Contornos, Ponto, MAX_FORMAS};

use crate::dados::Primitiva;
use crate::modelo::{Elemento, Papel, Tipo};
use crate::sessao::{Mudanca, Sessao};

const PRECISAO: f64 = 0.00001;

/// Operação disponível em Inserir > Organizar > Mesclar formas (do `feather-shapes`).
pub use feather_shapes::booleanas::OperacaoBooleana;

fn poligono_de(e: &Elemento) -> Result<Contornos, String> {
    let x = f64::from(e.x.min(e.x + e.largura));
    let y = f64::from(e.y.min(e.y + e.altura));
    let w = f64::from(e.largura.abs());
    let h = f64::from(e.altura.abs());
    if !(w > 0.0 && h > 0.0) || !w.is_finite() || !h.is_finite() {
        return Err("A forma precisa ter largura e altura maiores que zero.".into());
    }
    let mut pol = match e.tipo {
        Tipo::Caminho => feather_shapes::caminho::decodificar_poligonos(&e.dados, x, y, w, h)?,
        outro => {
            let regioes = match crate::formas_comuns::forma_do_tipo(outro) {
                Some(forma) => motor::regioes(forma, &e.ajustes, x, y, w, h)?,
                None => None,
            };
            regioes.ok_or_else(|| format!("O tipo '{}' não é uma forma vetorial mesclável.", outro.nome()))?
        }
    };
    let centro = [f64::from(e.x + e.largura * 0.5), f64::from(e.y + e.altura * 0.5)];
    motor::girar(&mut pol, centro, f64::from(e.rotacao));
    if e.tipo != Tipo::Caminho {
        motor::orientar_positivo(&mut pol);
    }
    Ok(pol)
}

fn gravar_caminho(contornos: &Contornos, base: &Elemento, id: u64) -> Result<Elemento, String> {
    let pontos: Vec<Ponto> = contornos.iter().flatten().copied().collect();
    if pontos.is_empty() { return Err("A operação não produziu nenhuma região.".into()); }
    let x = pontos.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
    let y = pontos.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
    let max_x = pontos.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
    let max_y = pontos.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
    let w = max_x - x;
    let h = max_y - y;
    if w < PRECISAO || h < PRECISAO { return Err("O resultado tem área insuficiente.".into()); }
    let dados = feather_shapes::caminho::codificar_poligonos(contornos, x, y, w, h);
    let mut e = base.clone();
    e.id = id;
    e.tipo = Tipo::Caminho;
    e.papel = Papel::Livre;
    e.x = x as f32;
    e.y = y as f32;
    e.largura = w as f32;
    e.altura = h as f32;
    e.rotacao = 0.0;
    e.dados = dados;
    e.texto.clear();
    e.imagem = Default::default();
    e.ajustes.clear();
    e.grupo = 0;
    e.destino = 0;
    e.link.clear();
    e.animacao = Default::default();
    e.oculto = false;
    Ok(e)
}

/// Renderização do caminho em coordenadas absolutas, pronta para Iced/PDF/MCP.
pub fn primitivas(e: &Elemento) -> Vec<Primitiva> {
    if crate::vetores::eh_svg(e) { return crate::vetores::primitivas(e); }
    // O Iced aplica a rotação ao pintar o elemento: aqui a geometria fica no eixo local.
    let mut sem_giro = e.clone();
    sem_giro.rotacao = 0.0;
    let Ok(contornos) = poligono_de(&sem_giro) else { return Vec::new() };
    vec![Primitiva::Caminho {
        contornos: contornos.iter().map(|anel| anel.iter().map(|p| [p[0] as f32, p[1] as f32]).collect()).collect(),
        preenchimento: e.preenchimento.clone(),
        contorno: e.contorno.clone(),
        espessura: e.espessura,
    }]
}

/// Acerto geométrico na forma composta: clicar dentro de um furo não seleciona a forma.
pub fn contem_ponto(e: &Elemento, x: f32, y: f32) -> bool {
    if crate::vetores::eh_svg(e) { return x >= e.x.min(e.x+e.largura) && x <= e.x.max(e.x+e.largura)
        && y >= e.y.min(e.y+e.altura) && y <= e.y.max(e.y+e.altura); }
    poligono_de(e).is_ok_and(|regioes| motor::contem([f64::from(x), f64::from(y)], &regioes))
}

impl Sessao {
    /// Mescla as formas selecionadas diretamente no motor Rust; um único passo de desfazer.
    pub fn formas_booleanas(&mut self, op: OperacaoBooleana) -> Result<Mudanca, String> {
        if !(2..=MAX_FORMAS).contains(&self.selecao.len()) {
            return Err(format!("Selecione de 2 a {MAX_FORMAS} formas vetoriais para mesclar."));
        }
        let selecionados: Vec<Elemento> = self.selecao.iter()
            .map(|id| self.slide().elementos.iter().find(|e| e.id == *id && e.papel != Papel::Fundo && !e.oculto)
                .cloned().ok_or("A seleção contém um elemento indisponível."))
            .collect::<Result<_, _>>()?;
        let geometrias = selecionados.iter().map(poligono_de).collect::<Result<Vec<_>, _>>()?;
        let componentes: Vec<Contornos> = motor::mesclar(&geometrias, op)?;
        if componentes.is_empty() { return Err("As formas selecionadas não produzem uma área nessa operação.".into()); }
        let base = selecionados[0].clone();
        let excluir = self.selecao.clone();
        let pos = self.slide().elementos.iter().position(|e| excluir.contains(&e.id))
            .ok_or("Nenhum elemento encontrado.")?;
        self.editar(move |s| {
            let novos = componentes.iter().map(|regiao| gravar_caminho(regiao, &base, s.ap.novo_id()))
                .collect::<Result<Vec<_>, _>>()?;
            let ids = novos.iter().map(|e| e.id).collect();
            s.slide_mut().elementos.retain(|e| !excluir.contains(&e.id));
            s.slide_mut().elementos.splice(pos..pos, novos);
            s.selecao = ids;
            Ok((vec![s.atual], false))
        })
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use feather_shapes::booleanas::{area, calcular};

    fn ret(x: f64, y: f64, w: f64, h: f64) -> Contornos {
        vec![vec![[x, y], [x + w, y], [x + w, y + h], [x, y + h]]]
    }

    fn superficie(p: &Contornos) -> f64 {
        p.iter().map(|a| area(a)).sum::<f64>()
    }

    #[test]
    fn todas_as_cinco_operacoes_sobrepostas() {
        let a = ret(0.0, 0.0, 100.0, 100.0);
        let b = ret(50.0, 0.0, 100.0, 100.0);
        for (op, esperado) in [
            (OperacaoBooleana::Uniao, 15000.0),
            (OperacaoBooleana::Intersecao, 5000.0),
            (OperacaoBooleana::Subtracao, 5000.0),
            (OperacaoBooleana::Combinacao, 10000.0),
        ] {
            let p = calcular(&a, &b, op).unwrap();
            assert!((superficie(&p) - esperado).abs() < 0.1, "{op:?}: {p:?}");
        }
        let mut s = Sessao::default();
        s.slide_novo("em_branco").unwrap();
        s.elemento_inserir(Tipo::Retangulo, Some([0.0, 0.0, 100.0, 100.0]), None).unwrap();
        let id = s.estado().selecao[0];
        s.elemento_inserir(Tipo::Retangulo, Some([50.0, 0.0, 100.0, 100.0]), None).unwrap();
        s.alternar_selecao(id);
        let antes = s.slide().elementos.len();
        s.formas_booleanas(OperacaoBooleana::Fragmentacao).unwrap();
        assert_eq!(s.estado().selecao.len(), 3);
        assert_eq!(s.slide().elementos.len(), antes + 1);
        s.desfazer().unwrap();
        assert_eq!(s.slide().elementos.len(), antes);
    }

    #[test]
    fn furos_e_geometria_persistente() {
        let a = ret(0.0, 0.0, 100.0, 100.0);
        let b = ret(25.0, 25.0, 50.0, 50.0);
        let p = calcular(&a, &b, OperacaoBooleana::Subtracao).unwrap();
        assert_eq!(p.len(), 2);
        assert!((superficie(&p) - 7500.0).abs() < 0.1);
        let e = gravar_caminho(&p, &Elemento::default(), 9).unwrap();
        assert_eq!(e.tipo, Tipo::Caminho);
        let ler = poligono_de(&e).unwrap();
        assert!((superficie(&ler) - 7500.0).abs() < 0.1);
        assert_eq!(primitivas(&e).len(), 1);
        assert!(contem_ponto(&e, 10.0, 10.0));
        assert!(!contem_ponto(&e, 50.0, 50.0), "furo não é superfície selecionável");
        let mut movido = e.clone();
        movido.x += 100.0;
        assert!(contem_ponto(&movido, 110.0, 10.0));
        assert!(!contem_ponto(&movido, 10.0, 10.0));
    }

    #[test]
    fn intersecao_vazia_nao_apaga_formas() {
        let mut s = Sessao::default();
        s.slide_novo("em_branco").unwrap();
        s.elemento_inserir(Tipo::Retangulo, Some([5.0, 5.0, 50.0, 50.0]), None).unwrap();
        let primeiro = s.estado().selecao[0];
        s.elemento_inserir(Tipo::Elipse, Some([200.0, 200.0, 40.0, 40.0]), None).unwrap();
        s.alternar_selecao(primeiro);
        let quantos = s.slide().elementos.len();
        assert!(s.formas_booleanas(OperacaoBooleana::Intersecao).is_err());
        assert_eq!(s.slide().elementos.len(), quantos);
    }
}
