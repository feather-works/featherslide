//! Ponte real FeatherSlide → FeatherWorks Drawing.
//!
//! Os IDs, o `Elemento` e a serialização .fsdz continuam no core; esta
//! conversão empresta os elementos e produz geometria vetorial independente
//! para exportação. Não acrescentar campos ao modelo antigo.
use featherslide_core::{
    booleanas, dados::Primitiva, formas, icones,
    modelo::{Elemento, Tipo},
    novas_formas, vetores::Segmento,
};
use feather_draw::{
    caminhos::{Caminho, Comando, RegraPreenchimento},
    estilos::{EstiloVetor, Extremo, Juncao, Preenchimento, Stroke},
    transformar, Cena, Cor, Figura, Ponto, Transformacao,
};

/// Cena na unidade original do FeatherSlide (pontos), acompanhada do ID de
/// origem de cada Figura. Várias primitivas podem pertencer ao mesmo ID.
#[derive(Debug, Clone, PartialEq)]
pub struct CenaDoSlide {
    pub cena: Cena,
    pub ids: Vec<u64>,
}

fn ponto(p: [f32; 2]) -> Result<Ponto, String> {
    if !p[0].is_finite() || !p[1].is_finite() {
        return Err("Coordenada vetorial inválida no FeatherSlide.".into());
    }
    Ok(Ponto::novo(p[0], p[1]))
}

/// Aceita #RGB, #RRGGBB e #RRGGBBAA do modelo nativo; vazio significa
/// ausência de tinta. O Drawing representa alfa no campo `Cor::a`.
fn cor(hex: &str) -> Result<Option<Cor>, String> {
    if hex.is_empty() {
        return Ok(None);
    }
    let valor = hex.strip_prefix('#').ok_or("Cor vetorial sem prefixo #.")?;
    if !valor.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("Cor vetorial hexadecimal inválida.".into());
    }
    let ler = |parte: &str| u8::from_str_radix(parte, 16)
        .map_err(|_| "Cor vetorial hexadecimal inválida.".to_owned());
    let (r, g, b, a) = match valor.len() {
        3 => {
            let r = ler(&valor[0..1])?;
            let g = ler(&valor[1..2])?;
            let b = ler(&valor[2..3])?;
            (r * 17, g * 17, b * 17, 255)
        }
        6 => (ler(&valor[0..2])?, ler(&valor[2..4])?, ler(&valor[4..6])?, 255),
        8 => (ler(&valor[0..2])?, ler(&valor[2..4])?, ler(&valor[4..6])?, ler(&valor[6..8])?),
        _ => return Err("Cor vetorial precisa ser #RGB, #RRGGBB ou #RRGGBBAA.".into()),
    };
    Ok(Some(Cor::rgba(r, g, b, a)))
}

fn estilo(preenchimento: &str, contorno: &str, espessura: f32, icone: bool)
    -> Result<EstiloVetor, String>
{
    let preenchimento = cor(preenchimento)?.map(Preenchimento::Solido);
    let stroke = cor(contorno)?.map(|c| {
        let mut stroke = Stroke::novo(c, espessura);
        if icone {
            stroke.extremo = Extremo::Redondo;
            stroke.juncao = Juncao::Redonda;
        }
        stroke
    });
    let estilo = EstiloVetor {
        preenchimento,
        stroke,
        opacidade: 1.0,
        texto: None,
    };
    if !estilo.valido() {
        return Err("Espessura ou estilo vetorial inválido.".into());
    }
    Ok(estilo)
}

fn caminho_dos_pontos(pontos: &[[f32; 2]], fechado: bool) -> Result<Caminho, String> {
    if pontos.len() < if fechado { 3 } else { 2 } {
        return Err("Caminho vetorial sem vértices suficientes.".into());
    }
    let mut c = Caminho::novo();
    for (i, &p) in pontos.iter().enumerate() {
        c.adicionar(if i == 0 {
            Comando::Mover(ponto(p)?)
        } else {
            Comando::Linha(ponto(p)?)
        });
    }
    if fechado {
        c.adicionar(Comando::Fechar);
    }
    Ok(c)
}

fn segmento_svg(segmentos: &[Segmento], par: bool) -> Result<Caminho, String> {
    if !matches!(segmentos.first(), Some(Segmento::M(_))) {
        return Err("Caminho SVG sem comando inicial M.".into());
    }
    let mut c = Caminho::novo();
    c.regra = if par { RegraPreenchimento::ParImpar } else { RegraPreenchimento::NaoZero };
    for s in segmentos {
        c.adicionar(match s {
            Segmento::M(p) => Comando::Mover(ponto(*p)?),
            Segmento::L(p) => Comando::Linha(ponto(*p)?),
            Segmento::Q(a, b) => Comando::Quadratica {
                controle: ponto(*a)?, fim: ponto(*b)?,
            },
            Segmento::C(a, b, d) => Comando::Cubica {
                controle1: ponto(*a)?,
                controle2: ponto(*b)?,
                fim: ponto(*d)?,
            },
            Segmento::Z => Comando::Fechar,
        });
    }
    Ok(c)
}

/// Mapeia primitiva nativa para a Figura compartilhada usada no SVG e Iced.
pub fn converter_primitiva(p: &Primitiva, icone: bool) -> Result<Figura, String> {
    let (caminho, fill, stroke, espessura) = match p {
        Primitiva::Vetor { segmentos, par, preenchimento, contorno, espessura } => (
            segmento_svg(segmentos, *par)?,
            preenchimento.as_str(), contorno.as_str(), *espessura
        ),
        Primitiva::Caminho { contornos, preenchimento, contorno, espessura } => {
            let mut c = Caminho::novo();
            for pontos in contornos {
                c.comandos.extend(caminho_dos_pontos(pontos, true)?.comandos);
            }
            if c.comandos.is_empty() {
                return Err("Caminho composto vazio.".into());
            }
            (c, preenchimento.as_str(), contorno.as_str(), *espessura)
        }
        Primitiva::Poli { pontos, cor, espessura } => (
            caminho_dos_pontos(pontos, false)?, "", cor.as_str(), *espessura
        ),
        Primitiva::Poligono { pontos, preenchimento } => (
            caminho_dos_pontos(pontos, true)?, preenchimento.as_str(), "", 0.0
        ),
        Primitiva::Linha { x1, y1, x2, y2, cor, espessura } => (
            caminho_dos_pontos(&[[*x1, *y1], [*x2, *y2]], false)?,
            "", cor.as_str(), *espessura
        ),
        Primitiva::Ret { x, y, w, h, preenchimento, contorno: cor, espessura } => (
            caminho_dos_pontos(&[
                [*x, *y], [*x + *w, *y], [*x + *w, *y + *h], [*x, *y + *h],
            ], true)?,
            preenchimento.as_str(), cor.as_str(), *espessura
        ),
        // Texto e setor exigem renderização própria; não descartá-los em silêncio.
        Primitiva::Texto { .. } | Primitiva::Setor { .. } =>
            return Err("Primitiva com texto ou setor ainda sem adaptação vetorial fiel.".into()),
    };
    Ok(Figura::Caminho {
        caminho,
        estilo: estilo(fill, stroke, espessura, icone)?,
    })
}

/// Adapta formas nativas simples do Slide para o mesmo modelo utilizado no
/// SVG e no Canvas Iced. Coordenadas em pontos; nenhuma mutação do Elemento.
///
/// `Ok(None)` significa tipo não migrado (renderizador legado da GUI).
/// Geometrias ou estilos inválidos retornam erro explícito.
pub fn converter_forma_basica(
    tipo: &str,
    area: [f32; 4],
    preenchimento: &str,
    contorno: &str,
    espessura: f32,
) -> Result<Option<Figura>, String> {
    if !matches!(tipo, "retangulo" | "elipse" | "linha" | "triangulo") {
        return Ok(None);
    }
    let [x, y, w, h] = area;
    if area.iter().any(|v| !v.is_finite())
        || (tipo != "linha" && (w <= 0.0 || h <= 0.0))
        || !espessura.is_finite()
    {
        return Err("Forma nativa com dimensões ou espessura inválidas.".into());
    }
    // Linha nativa aceita a cor de preenchimento quando o contorno está vazio.
    let tinta = if tipo == "linha" && contorno.is_empty() {
        preenchimento
    } else {
        contorno
    };
    let estilo = estilo(
        if tipo == "linha" { "" } else { preenchimento },
        tinta,
        espessura,
        false,
    )?;
    let figura = match tipo {
        "retangulo" => Figura::Caminho {
            caminho: caminho_dos_pontos(&[
                [x, y], [x + w, y], [x + w, y + h], [x, y + h],
            ], true)?,
            estilo,
        },
        "triangulo" => Figura::Caminho {
            caminho: caminho_dos_pontos(&[
                [x + w / 2.0, y], [x + w, y + h], [x, y + h],
            ], true)?,
            estilo,
        },
        "elipse" => Figura::Elipse {
            centro: Ponto::novo(x + w / 2.0, y + h / 2.0),
            raio_x: w / 2.0,
            raio_y: h / 2.0,
            estilo,
        },
        "linha" => Figura::Caminho {
            caminho: caminho_dos_pontos(&[[x, y], [x + w, y + h]], false)?,
            estilo,
        },
        _ => unreachable!("tipo filtrado no início"),
    };
    if figura.bounding_box().is_none() {
        return Err("Forma nativa com geometria inválida.".into());
    }
    Ok(Some(figura))
}

fn poligono(p: Vec<[f32; 2]>, e: &Elemento) -> Result<Figura, String> {
    Ok(Figura::Caminho {
        caminho: caminho_dos_pontos(&p, true)?,
        estilo: estilo(&e.preenchimento, &e.contorno, e.espessura, false)?,
    })
}

fn formas_de(e: &Elemento) -> Result<Vec<Figura>, String> {
    let primitiva = match e.tipo {
        Tipo::Icone => Some(icones::primitivas(e)),
        Tipo::Caminho => Some(booleanas::primitivas(e)),
        t if novas_formas::suporta(t) => Some(novas_formas::primitivas(e)),
        _ => None,
    };
    if let Some(primitivas) = primitiva {
        if primitivas.is_empty() {
            return Err(format!("Forma {:?} sem geometria vetorial válida.", e.tipo));
        }
        return primitivas.iter()
            .map(|p| converter_primitiva(p, e.tipo == Tipo::Icone))
            .collect();
    }

    let figura = match e.tipo {
        Tipo::Retangulo => poligono(vec![
            [e.x, e.y], [e.x + e.largura, e.y],
            [e.x + e.largura, e.y + e.altura], [e.x, e.y + e.altura],
        ], e)?,
        Tipo::Elipse => Figura::Elipse {
            centro: Ponto::novo(e.x + e.largura / 2.0, e.y + e.altura / 2.0),
            raio_x: e.largura.abs() / 2.0,
            raio_y: e.altura.abs() / 2.0,
            estilo: estilo(&e.preenchimento, &e.contorno, e.espessura, false)?,
        },
        Tipo::Linha => Figura::Caminho {
            caminho: caminho_dos_pontos(&[
                [e.x, e.y], [e.x + e.largura, e.y + e.altura],
            ], false)?,
            estilo: estilo("", &e.contorno, e.espessura, false)?,
        },
        Tipo::Triangulo => poligono(vec![
            [e.x + e.largura / 2.0, e.y],
            [e.x + e.largura, e.y + e.altura], [e.x, e.y + e.altura],
        ], e)?,
        Tipo::Seta => poligono(vec![
            [e.x, e.y + 0.3 * e.altura],
            [e.x + 0.6 * e.largura, e.y + 0.3 * e.altura],
            [e.x + 0.6 * e.largura, e.y],
            [e.x + e.largura, e.y + 0.5 * e.altura],
            [e.x + 0.6 * e.largura, e.y + e.altura],
            [e.x + 0.6 * e.largura, e.y + 0.7 * e.altura],
            [e.x, e.y + 0.7 * e.altura],
        ], e)?,
        t => {
            if let Some(pontos) = formas::pontos(t.nome(), e.x, e.y, e.largura, e.altura) {
                poligono(pontos, e)?
            } else {
                return Err(format!(
                    "Tipo {:?} requer adaptador específico (texto, imagem, arredondamento ou dados).",
                    e.tipo
                ));
            }
        }
    };
    Ok(vec![figura])
}

/// Converte apenas geometria suportada, em pontos tipográficos do Slide.
/// Metadados, animações, IDs, ordem e bytes do modelo original não são mutados;
/// no SVG, a última Figura conserva a ordem Z do último Elemento.
pub fn cena_dos_elementos(elementos: &[Elemento], tamanho: [f32; 2])
    -> Result<CenaDoSlide, String>
{
    if tamanho.iter().any(|x| !x.is_finite() || *x <= 0.0) {
        return Err("Dimensões do slide inválidas.".into());
    }
    if elementos.is_empty() {
        return Err("Selecione ao menos uma forma vetorial.".into());
    }
    let mut cena = Cena::nova(tamanho[0], tamanho[1]);
    let mut ids = Vec::new();
    for e in elementos {
        if [e.x, e.y, e.largura, e.altura, e.rotacao]
            .iter().any(|v| !v.is_finite())
        {
            return Err(format!("Elemento {} possui geometria inválida.", e.id));
        }
        if !e.texto.is_empty() {
            return Err(format!(
                "Elemento {} possui texto; exportação só vetorial não pode descartá-lo.", e.id
            ));
        }
        let figuras = formas_de(e)?;
        let centro = Ponto::novo(e.x + e.largura / 2.0, e.y + e.altura / 2.0);
        let t = Transformacao::transladar(centro.x, centro.y)
            .compor(Transformacao::girar(e.rotacao.to_radians()))
            .compor(Transformacao::transladar(-centro.x, -centro.y));
        for figura in figuras {
            let transformada = if e.rotacao == 0.0 { figura } else { transformar(&figura, t) };
            cena.adicionar(transformada);
            ids.push(e.id);
        }
    }
    Ok(CenaDoSlide { cena, ids })
}

/// Caminho de exportação agora usado pela ferramenta SVG da GUI FeatherSlide.
/// A serialização .fsdz não é tocada; a representação neutra só existe durante
/// a renderização. Falhas são explícitas para tipos ainda não migrados.
pub fn exportar_svg(elementos: &[Elemento], tamanho: [f32; 2]) -> Result<String, String> {
    let mapeamento = cena_dos_elementos(elementos, tamanho)?;
    feather_draw::exportar_svg(&mapeamento.cena)
        .map_err(|e| format!("Falha no Drawing SVG: {e:?}"))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn formas_basicas_compartilham_modelo_svg_sem_modificar_dados() {
        let area = [12.0, 18.0, 120.0, 80.0];
        let mut cena = Cena::nova(960.0, 540.0);
        for tipo in ["retangulo", "elipse", "triangulo", "linha"] {
            let figura = converter_forma_basica(
                tipo, area, "#23456780", "#fedcba", 2.0,
            ).unwrap().unwrap();
            assert!(figura.bounding_box().is_some(), "{tipo}");
            if tipo == "elipse" {
                assert!(matches!(figura, Figura::Elipse { .. }));
            }
            if tipo == "linha" {
                let estilo = figura.estilo_vetor().unwrap();
                assert!(estilo.preenchimento.is_none());
            }
            cena.adicionar(figura);
        }
        assert_eq!(cena.figuras.len(), 4);
        let svg = feather_draw::exportar_svg(&cena).unwrap();
        assert!(svg.contains(r#"viewBox="0 0 960 540""#));
        assert!(svg.contains(r#"fill-opacity="0.502""#));
        assert!(svg.contains("#fedcba"));
    }

    #[test]
    fn linha_usa_preenchimento_quando_nao_existe_contorno() {
        let figura = converter_forma_basica(
            "linha", [10.0, 20.0, -30.0, 40.0], "#ff0000", "", 1.5,
        ).unwrap().unwrap();
        let estilo = figura.estilo_vetor().unwrap();
        assert_eq!(estilo.stroke.unwrap().cor, Cor::rgb(255, 0, 0));
        assert!(estilo.preenchimento.is_none());
        let comandos = figura.caminho(0.1).comandos;
        assert_eq!(comandos, vec![
            Comando::Mover(Ponto::novo(10.0, 20.0)),
            Comando::Linha(Ponto::novo(-20.0, 60.0)),
        ]);
    }

    #[test]
    fn formatos_desconhecidos_e_ajustes_invalidos_usam_fallback() {
        assert_eq!(converter_forma_basica(
            "imagem", [0.0, 0.0, 10.0, 10.0], "#000000", "", 1.0,
        ).unwrap(), None);
        for area in [
            [f32::NAN, 0.0, 10.0, 10.0],
            [1.0, 2.0, -10.0, 5.0],
            [0.0, 0.0, 0.0, 10.0],
        ] {
            assert!(converter_forma_basica(
                "retangulo", area, "#ffffff", "#000000", 1.0,
            ).is_err());
        }
        assert!(converter_forma_basica(
            "elipse", [0.0, 0.0, 10.0, 10.0], "#12345g", "", 1.0,
        ).is_err());
        assert!(converter_forma_basica(
            "triangulo", [0.0, 0.0, 10.0, 10.0], "#00ff00", "", f32::INFINITY,
        ).is_err());
        assert!(converter_forma_basica(
            "linha", [0.0, 0.0, f32::INFINITY, 5.0], "", "#000000", 1.0,
        ).is_err());
    }

    fn forma(id: u64, tipo: Tipo) -> Elemento {
        Elemento {
            id, tipo, x: 10.0, y: 20.0, largura: 120.0, altura: 80.0,
            preenchimento: "#33669980".into(), contorno: "#102030".into(),
            espessura: 2.0, ..Elemento::default()
        }
    }

    #[test]
    fn ids_z_order_e_modelo_original_ficam_intactos() {
        let a = forma(12, Tipo::Retangulo);
        let b = forma(54, Tipo::Elipse);
        let entrada = vec![a, b];
        let antes = entrada.clone();
        let resultado = cena_dos_elementos(&entrada, [960.0, 540.0]).unwrap();
        assert_eq!(entrada, antes);
        assert_eq!(resultado.ids, [12, 54]);
        assert_eq!(resultado.cena.figuras.len(), 2);
        assert_eq!((resultado.cena.largura, resultado.cena.altura), (960.0, 540.0));
        let svg = exportar_svg(&entrada, [960.0, 540.0]).unwrap();
        assert_eq!(svg.matches("<path d=").count(), 2);
        assert!(svg.contains("fill-opacity=\"0.502\""));
    }

    #[test]
    fn rotacao_em_graus_e_pontos_do_slide() {
        let mut r = forma(10, Tipo::Retangulo);
        r.x = 0.0; r.y = 0.0; r.largura = 80.0; r.altura = 40.0;
        r.rotacao = 90.0;
        let resultado = cena_dos_elementos(&[r], [960.0, 540.0]).unwrap();
        let limites = resultado.cena.figuras[0].bounding_box().unwrap();
        // Centro 40,20; retângulo de 80x40 passa a 40x80.
        assert!((limites.x - 20.0).abs() <= 1.01);
        assert!((limites.y + 20.0).abs() <= 1.01);
        assert!((limites.largura - 40.0).abs() <= 2.01);
        assert!((limites.altura - 80.0).abs() <= 2.01);
    }

    #[test]
    fn icone_tabler_e_caminho_svg_usam_motor_neutro() {
        let mut icone = forma(25, Tipo::Icone);
        icone.dados = "home".into();
        let mut caminho = forma(26, Tipo::Caminho);
        caminho.dados = "SVG1\npar\nM 0 0\nQ 0.5 0 1 1\nL 0 1\nZ".into();
        let r = cena_dos_elementos(&[icone, caminho], [960.0, 540.0]).unwrap();
        assert_eq!(r.ids.last(), Some(&26));
        assert!(r.ids.iter().any(|&id| id == 25));
        let svg = feather_draw::exportar_svg(&r.cena).unwrap();
        assert!(svg.contains(" Q ") || svg.contains("Q "));
        assert!(svg.contains("fill-rule=\"evenodd\""));
    }

    #[test]
    fn contornos_com_furos_preservam_hit_test() {
        let r = forma(100, Tipo::Anel);
        let cena = cena_dos_elementos(&[r], [960.0, 540.0]).unwrap().cena;
        assert!(!cena.figuras[0].atinge(Ponto::novo(70.0, 60.0), 0.0));
        assert!(cena.figuras[0].atinge(Ponto::novo(70.0, 27.0), 0.0));
    }

    #[test]
    fn sem_perda_silenciosa_em_tipos_invalidos_e_cores() {
        assert!(cena_dos_elementos(&[forma(1, Tipo::Imagem)], [960.0, 540.0]).is_err());
        assert!(cena_dos_elementos(&[forma(1, Tipo::Texto)], [960.0, 540.0]).is_err());
        assert!(cena_dos_elementos(&[], [960.0, 540.0]).is_err());
        assert!(cena_dos_elementos(&[forma(1, Tipo::Retangulo)], [f32::NAN, 540.0]).is_err());
        let mut e = forma(7, Tipo::Retangulo);
        e.preenchimento = "#xyz".into();
        assert!(cena_dos_elementos(&[e], [960.0, 540.0]).is_err());
        let mut e = forma(9, Tipo::Retangulo);
        e.preenchimento = "#ááá".into();
        assert!(cena_dos_elementos(&[e], [960.0, 540.0]).is_err());
        let mut e = forma(8, Tipo::Elipse);
        e.texto = "Não apagar texto".into();
        assert!(cena_dos_elementos(&[e], [960.0, 540.0]).is_err());
    }

    #[test]
    fn tipos_legados_sem_renumerar_variantes() {
        assert_eq!(Tipo::TODOS[27], Tipo::Caminho);
        for tipo in [Tipo::Triangulo, Tipo::Losango, Tipo::SetaEsquerda,
            Tipo::Estrela10, Tipo::Nuvem, Tipo::Arco]
        {
            let r = cena_dos_elementos(&[forma(23, tipo)], [960.0, 540.0]);
            assert!(r.is_ok(), "{tipo:?}: {r:?}");
        }
    }
}
