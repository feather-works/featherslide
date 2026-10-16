//! A ponte entre o modelo neutro do `feather-shapes` e o FeatherSlide: o `Tipo` do slide
//! (cuja ordem está gravada no `.fsdz` e não muda) ↔ o `TipoForma` da biblioteca, o item de
//! diagrama ↔ `Elemento`, e a geometria da biblioteca ↔ `Primitiva`, que a tela e o PDF do
//! Slide desenham.

use feather_shapes::caminho::Segmento;
use feather_shapes::diagramas::ItemDiagrama;
use feather_shapes::feather_draw::{Comando, RegraPreenchimento, Retangulo};
use feather_shapes::{Aparencia, Forma, Geometria, TipoForma};

use crate::dados::Primitiva;
use crate::modelo::{Alinhamento, Elemento, EstiloDoTexto, Tipo, Vertical};

/// O tipo de forma da biblioteca que corresponde ao `Tipo` do slide. `None` para o que não é
/// forma (imagem, tabela, gráfico, ícone, WordArt e caminho livre).
pub fn forma_do_tipo(tipo: Tipo) -> Option<TipoForma> {
    Some(match tipo {
        Tipo::Texto => TipoForma::CaixaTexto,
        Tipo::Retangulo => TipoForma::Retangulo,
        Tipo::RetanguloArredondado => TipoForma::RetanguloArredondado,
        Tipo::Elipse => TipoForma::Elipse,
        Tipo::Triangulo => TipoForma::Triangulo,
        Tipo::Seta => TipoForma::SetaDireita,
        Tipo::Linha => TipoForma::Linha,
        Tipo::Setor => TipoForma::Setor,
        Tipo::Losango => TipoForma::Losango,
        Tipo::Pentagono => TipoForma::Pentagono,
        Tipo::Hexagono => TipoForma::Hexagono,
        Tipo::Octogono => TipoForma::Octogono,
        Tipo::Trapezio => TipoForma::Trapezio,
        Tipo::Paralelogramo => TipoForma::Paralelogramo,
        Tipo::Estrela => TipoForma::Estrela5,
        Tipo::Cruz => TipoForma::Cruz,
        Tipo::SetaEsquerda => TipoForma::SetaEsquerda,
        Tipo::SetaDupla => TipoForma::SetaDupla,
        Tipo::SetaCima => TipoForma::SetaCima,
        Tipo::SetaBaixo => TipoForma::SetaBaixo,
        Tipo::Chevron => TipoForma::Chevron,
        Tipo::Documento => TipoForma::Documento,
        Tipo::TrianguloRetangulo => TipoForma::TrianguloRetangulo,
        Tipo::Heptagono => TipoForma::Heptagono,
        Tipo::Decagono => TipoForma::Decagono,
        Tipo::Dodecagono => TipoForma::Dodecagono,
        Tipo::Circulo => TipoForma::Circulo,
        Tipo::Anel => TipoForma::Anel,
        Tipo::Arco => TipoForma::Arco,
        Tipo::Corda => TipoForma::Corda,
        Tipo::Cubo => TipoForma::Cubo,
        Tipo::Cilindro => TipoForma::Cilindro,
        Tipo::Moldura => TipoForma::Moldura,
        Tipo::CantoEmL => TipoForma::CantoEmL,
        Tipo::Placa => TipoForma::Placa,
        Tipo::Sol => TipoForma::Sol,
        Tipo::Lua => TipoForma::Lua,
        Tipo::Coracao => TipoForma::Coracao,
        Tipo::Raio => TipoForma::Raio,
        Tipo::Nuvem => TipoForma::Nuvem,
        Tipo::Estrela4 => TipoForma::Estrela4,
        Tipo::Estrela6 => TipoForma::Estrela6,
        Tipo::Estrela8 => TipoForma::Estrela8,
        Tipo::Estrela10 => TipoForma::Estrela10,
        Tipo::Estrela12 => TipoForma::Estrela12,
        Tipo::Imagem | Tipo::Tabela | Tipo::Grafico | Tipo::Icone | Tipo::WordArt | Tipo::Caminho => return None,
    })
}

/// O `Tipo` nativo do slide para um tipo da biblioteca. `None` para as formas que o Slide
/// guarda como caminho vetorial (`Tipo::Caminho` com a geometria SVG1 em `dados`).
pub fn tipo_da_forma(forma: TipoForma) -> Option<Tipo> {
    Tipo::TODOS.into_iter().find(|&t| forma_do_tipo(t) == Some(forma))
}

/// O elemento (sem id nem grupo) de um item de diagrama da biblioteca.
pub fn elemento_do_item(item: &ItemDiagrama) -> Elemento {
    let tipo = tipo_da_forma(item.tipo).unwrap_or(Tipo::Retangulo);
    match &item.rotulo {
        Some(r) => Elemento {
            tipo, x: item.x, y: item.y, largura: item.largura, altura: item.altura,
            texto: item.texto.clone(),
            preenchimento: item.preenchimento.clone(), contorno: item.contorno.clone(),
            espessura: item.espessura,
            estilo: EstiloDoTexto {
                cor: r.cor.clone(), tamanho: r.tamanho, negrito: r.negrito,
                alinhamento: Alinhamento::Centro, vertical: Vertical::Meio,
                ..EstiloDoTexto::default()
            },
            ..Elemento::default()
        },
        // Linha de ligação: o resto fica no padrão do elemento.
        None => Elemento {
            tipo, x: item.x, y: item.y, largura: item.largura, altura: item.altura,
            contorno: item.contorno.clone(), espessura: item.espessura,
            ..Elemento::default()
        },
    }
}

/// A forma neutra de um elemento do slide (tipo, ajustes, aparência, texto e giro), para
/// entregar a outro aplicativo ou desenhar pela biblioteca. `None` se não é forma.
pub fn forma_do_elemento(e: &Elemento) -> Option<Forma> {
    Some(Forma {
        tipo: forma_do_tipo(e.tipo)?,
        ajustes: e.ajustes.clone(),
        aparencia: Aparencia {
            preenchimento: e.preenchimento.clone(),
            gradiente: e.gradiente.clone(),
            contorno: e.contorno.clone(),
            espessura: e.espessura,
            opacidade: e.opacidade,
            sombra: e.sombra.clone(),
        },
        texto: (!e.texto.is_empty()).then(|| e.texto.clone()),
        rotacao: e.rotacao,
    })
}

fn segmentos(c: &feather_shapes::feather_draw::Caminho) -> Vec<Segmento> {
    let p = |q: feather_shapes::feather_draw::Ponto| [q.x, q.y];
    c.comandos.iter().map(|k| match *k {
        Comando::Mover(a) => Segmento::M(p(a)),
        Comando::Linha(a) => Segmento::L(p(a)),
        Comando::Quadratica { controle, fim } => Segmento::Q(p(controle), p(fim)),
        Comando::Cubica { controle1, controle2, fim } => Segmento::C(p(controle1), p(controle2), p(fim)),
        Comando::Fechar => Segmento::Z,
    }).collect()
}

/// A geometria da biblioteca como primitivas do Slide: as regiões viram um vetor preenchido
/// e contornado; os traços abertos, um vetor só de contorno.
pub fn primitivas_da_geometria(g: &Geometria, preenchimento: &str, contorno: &str, espessura: f32) -> Vec<Primitiva> {
    let mut saida = Vec::new();
    if !g.preenchida.comandos.is_empty() {
        saida.push(Primitiva::Vetor {
            segmentos: segmentos(&g.preenchida),
            par: g.preenchida.regra == RegraPreenchimento::ParImpar,
            preenchimento: preenchimento.into(),
            contorno: contorno.into(),
            espessura,
        });
    }
    if !g.tracos.comandos.is_empty() {
        saida.push(Primitiva::Vetor {
            segmentos: segmentos(&g.tracos),
            par: false,
            preenchimento: String::new(),
            contorno: contorno.into(),
            espessura,
        });
    }
    saida
}

/// As primitivas de uma forma da biblioteca na caixa `[x, y, largura, altura]` (sem giro:
/// o Slide gira ao pintar o elemento).
pub fn primitivas_da_forma(forma: &Forma, caixa: [f32; 4]) -> Vec<Primitiva> {
    let r = Retangulo::novo(caixa[0], caixa[1], caixa[2], caixa[3]);
    forma.geometria(r).map_or_else(Vec::new, |g| {
        let a = &forma.aparencia;
        primitivas_da_geometria(&g, &a.preenchimento, &a.contorno, a.espessura)
    })
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn todo_tipo_de_forma_vai_e_volta_e_os_codigos_do_arquivo_nao_mudam() {
        for t in Tipo::TODOS {
            if let Some(f) = forma_do_tipo(t) {
                assert_eq!(tipo_da_forma(f), Some(t), "{t:?}");
            }
        }
        // O .fsdz grava o Tipo pela posição: o feather-shapes não pode mexer nisso.
        assert_eq!(Tipo::TODOS[27], Tipo::Caminho);
        assert_eq!(Tipo::TODOS[50], Tipo::Estrela12);
        assert_eq!(tipo_da_forma(TipoForma::FluxoDecisao), None);
    }

    #[test]
    fn rotulos_da_galeria_sao_os_nomes_da_biblioteca() {
        for &(tipo, rotulo) in crate::formas::ADICIONAIS {
            assert_eq!(forma_do_tipo(tipo).map(TipoForma::nome), Some(rotulo));
        }
        let ids: Vec<_> = crate::formas::ADICIONAIS.iter().map(|(t, _)| forma_do_tipo(*t).unwrap()).collect();
        assert_eq!(ids, feather_shapes::poligonos::ADICIONAIS);
    }

    #[test]
    fn forma_do_elemento_desenha_pela_biblioteca() {
        let e = Elemento { tipo: Tipo::Coracao, x: 10.0, y: 10.0, largura: 100.0, altura: 80.0,
            preenchimento: "#ff0000".into(), contorno: "#000000".into(), espessura: 2.0, ..Elemento::default() };
        let f = forma_do_elemento(&e).unwrap();
        assert_eq!(f.tipo, TipoForma::Coracao);
        let p = primitivas_da_forma(&f, [e.x, e.y, e.largura, e.altura]);
        assert!(matches!(&p[0], Primitiva::Vetor { preenchimento, .. } if preenchimento == "#ff0000"));
        assert!(forma_do_elemento(&Elemento { tipo: Tipo::Imagem, ..Elemento::default() }).is_none());
        let cubo = forma_do_elemento(&Elemento { tipo: Tipo::Cubo, largura: 10.0, altura: 10.0, ..Elemento::default() }).unwrap();
        assert_eq!(primitivas_da_forma(&cubo, [0.0, 0.0, 10.0, 10.0]).len(), 2, "corpo e arestas");
    }
}
