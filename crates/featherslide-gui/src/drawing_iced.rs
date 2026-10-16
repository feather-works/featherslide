//! Adaptador de interface: motor Drawing neutro -> Canvas Iced (pixels lógicos).
//! O motor não depende de Iced. Esta borda recebe um Viewport explícito e
//! mantém a mesma geometria das saídas SVG/PDF, inclusive curvas e fill-rule.
use feather_draw::{
    Caminho, Comando, Extremo, Figura, Juncao, Ponto,
    Preenchimento, RegraPreenchimento, Viewport,
};
use iced::{
    widget::canvas::{self, Frame, Path, Stroke},
    Color, Point,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroIced {
    EstiloInvalido,
    GeometriaInvalida,
    GradienteNaoSuportado,
    TracejadoNaoSuportado,
    RecorteNaoSuportado,
}

fn cor(c: feather_draw::Cor, opacidade: f32) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, (f32::from(c.a) / 255.0 * opacidade).clamp(0.0, 1.0))
}

fn caminho_para_iced(caminho: &Caminho, viewport: Viewport) -> Result<Path, ErroIced> {
    if caminho.comandos.is_empty() {
        return Err(ErroIced::GeometriaInvalida);
    }
    let ponto = |p: Ponto| -> Result<Point, ErroIced> {
        if !p.x.is_finite() || !p.y.is_finite() {
            return Err(ErroIced::GeometriaInvalida);
        }
        let p = viewport.logico(p);
        if !p.x.is_finite() || !p.y.is_finite() {
            return Err(ErroIced::GeometriaInvalida);
        }
        Ok(Point::new(p.x, p.y))
    };
    // Validar ANTES de construir o Path para não aceitar comandos incompletos.
    let mut inicio = false;
    for cmd in &caminho.comandos {
        match cmd {
            Comando::Mover(p) => { ponto(*p)?; inicio = true; },
            Comando::Linha(p) if inicio => { ponto(*p)?; },
            Comando::Quadratica { controle, fim } if inicio => {
                ponto(*controle)?; ponto(*fim)?;
            }
            Comando::Cubica { controle1, controle2, fim } if inicio => {
                ponto(*controle1)?; ponto(*controle2)?; ponto(*fim)?;
            }
            Comando::Fechar if inicio => { inicio = false; },
            _ => return Err(ErroIced::GeometriaInvalida),
        }
    }
    Ok(Path::new(|b| {
        for cmd in &caminho.comandos {
            match *cmd {
                Comando::Mover(p) => {
                    let q = viewport.logico(p);
                    b.move_to(Point::new(q.x, q.y));
                }
                Comando::Linha(p) => {
                    let q = viewport.logico(p);
                    b.line_to(Point::new(q.x, q.y));
                }
                Comando::Quadratica { controle, fim } => {
                    let a = viewport.logico(controle);
                    let q = viewport.logico(fim);
                    b.quadratic_curve_to(Point::new(a.x, a.y), Point::new(q.x, q.y));
                }
                Comando::Cubica { controle1, controle2, fim } => {
                    let a = viewport.logico(controle1);
                    let b2 = viewport.logico(controle2);
                    let q = viewport.logico(fim);
                    b.bezier_curve_to(
                        Point::new(a.x, a.y), Point::new(b2.x, b2.y), Point::new(q.x, q.y),
                    );
                }
                Comando::Fechar => b.close(),
            }
        }
    }))
}

fn validar(figura: &Figura, viewport: Viewport) -> Result<(), ErroIced> {
    if matches!(figura, Figura::Recorte { .. }) {
        return Err(ErroIced::RecorteNaoSuportado);
    }
    let estilo = figura.estilo_vetor().ok_or(ErroIced::EstiloInvalido)?;
    if !estilo.valido() {
        return Err(ErroIced::EstiloInvalido);
    }
    if estilo.preenchimento.as_ref().is_some_and(|p| !matches!(p, Preenchimento::Solido(_))) {
        return Err(ErroIced::GradienteNaoSuportado);
    }
    if estilo.stroke.as_ref().is_some_and(|s| {
        !s.dash.is_empty()
            || (s.juncao == Juncao::Miter && (s.limite_miter - 4.0).abs() > 0.001)
    }) {
        // O Canvas não expõe miter-limit customizado neste adaptador.
        return Err(ErroIced::TracejadoNaoSuportado);
    }
    let c = figura.caminho(0.1);
    caminho_para_iced(&c, viewport).map(|_| ())
}

fn pintar_validado(frame: &mut Frame, figura: &Figura, viewport: Viewport, alfa_extra: f32) {
    let e = figura.estilo_vetor().expect("estilo validado");
    let caminho = figura.caminho(0.1);
    let path = caminho_para_iced(&caminho, viewport).expect("caminho validado");
    if let Some(Preenchimento::Solido(c)) = e.preenchimento {
        let regra = match caminho.regra {
            RegraPreenchimento::ParImpar => canvas::fill::Rule::EvenOdd,
            RegraPreenchimento::NaoZero => canvas::fill::Rule::NonZero,
        };
        frame.fill(&path, canvas::Fill { style: cor(c, e.opacidade * alfa_extra).into(), rule: regra });
    }
    if let Some(stroke) = e.stroke {
        let cap = match stroke.extremo {
            Extremo::Plano => canvas::LineCap::Butt,
            Extremo::Redondo => canvas::LineCap::Round,
            Extremo::Quadrado => canvas::LineCap::Square,
        };
        let join = match stroke.juncao {
            Juncao::Miter => canvas::LineJoin::Miter,
            Juncao::Redonda => canvas::LineJoin::Round,
            Juncao::Bevel => canvas::LineJoin::Bevel,
        };
        frame.stroke(
            &path,
            Stroke::default()
                .with_color(cor(stroke.cor, e.opacidade * alfa_extra))
                .with_width(stroke.largura * viewport.escala())
                .with_line_cap(cap)
                .with_line_join(join),
        );
    }
}

/// Desenha uma figura sem descartar gradientes, clipping ou tracejado: quando
/// não há equivalência fiel no Iced, retorna erro para que o chamador opte
/// explicitamente pelo backend legado, em vez de pintar algo enganoso.
pub fn desenhar_figura(
    frame: &mut Frame,
    figura: &Figura,
    viewport: Viewport,
) -> Result<(), ErroIced> {
    validar(figura, viewport)?;
    pintar_validado(frame, figura, viewport, 1.0);
    Ok(())
}

/// Opacidade adicional do app consumidor (animações/transições Iced).
pub fn desenhar_figura_com_opacidade(
    frame: &mut Frame,
    figura: &Figura,
    viewport: Viewport,
    alfa_extra: f32,
) -> Result<(), ErroIced> {
    if !alfa_extra.is_finite() || !(0.0..=1.0).contains(&alfa_extra) {
        return Err(ErroIced::EstiloInvalido);
    }
    if alfa_extra == 1.0 {
        return desenhar_figura(frame, figura, viewport);
    }
    validar(figura, viewport)?;
    pintar_validado(frame, figura, viewport, alfa_extra);
    Ok(())
}

/// Ponte de formas *nativas* FeatherSlide → Drawing → Iced.
/// Mantém o modelo e a posição em pontos; o pai do Canvas já aplica a rotação,
/// as escalas das animações e os demais efeitos. Quando a forma possui cantos
/// ajustados ou primitivas especiais, o caller usa seu pintor anterior.
pub fn desenhar_forma_basica(
    frame: &mut Frame,
    el: &featherslide_core::quadro::ElementoDesenhado,
    origem: (f32, f32),
    escala_logica: f32,
    arrasto: (f32, f32),
    alfa_animacao: f32,
) -> bool {
    if !el.ajustes.is_empty() || !el.primitivas.is_empty() {
        return false;
    }
    let Ok(vp) = Viewport::de_escala_logica(
        feather_draw::Unidade::Ponto,
        escala_logica,
        Ponto::novo(0.0, 0.0),
        Ponto::novo(origem.0, origem.1),
        1.0,
    ) else {
        return false;
    };
    let espessura = el.espessura.max(0.5 / escala_logica);
    let Ok(Some(figura)) = featherslide_io::drawing::converter_forma_basica(
        el.tipo,
        [el.x + arrasto.0, el.y + arrasto.1, el.largura, el.altura],
        &el.preenchimento,
        &el.contorno,
        espessura,
    ) else {
        return false;
    };
    desenhar_figura_com_opacidade(frame, &figura, vp, alfa_animacao).is_ok()
}

#[cfg(test)]
#[path = "drawing_iced_snapshots.rs"]
mod snapshots_drw012;

#[cfg(test)]
mod testes {
    use super::*;
    use feather_draw::{
        Cena, Cor, EstiloVetor, Ponto as DocPonto, Preenchimento, Stroke, Unidade, Viewport,
    };
    fn viewport() -> Viewport {
        Viewport::de_escala_logica(
            Unidade::Ponto, 2.0, DocPonto::novo(0.0, 0.0),
            DocPonto::novo(10.0, 20.0), 2.0,
        ).unwrap()
    }
    #[test]
    fn caminho_compartilhado_preserva_curvas_e_unidades_logicas() {
        let caminho = Caminho {
            comandos: vec![
                Comando::Mover(DocPonto::novo(0.0, 0.0)),
                Comando::Quadratica {
                    controle: DocPonto::novo(10.0, 0.0),
                    fim: DocPonto::novo(20.0, 20.0),
                },
                Comando::Cubica {
                    controle1: DocPonto::novo(20.0, 30.0),
                    controle2: DocPonto::novo(40.0, 30.0),
                    fim: DocPonto::novo(40.0, 40.0),
                },
                Comando::Fechar,
            ],
            regra: RegraPreenchimento::ParImpar,
        };
        let path = caminho_para_iced(&caminho, viewport()).unwrap();
        assert!(path.raw().iter().next().is_some());
        let p = viewport().logico(DocPonto::novo(10.0, 12.0));
        assert!((p.x - 30.0).abs() < 0.001);
        assert!((p.y - 44.0).abs() < 0.001);
        // DPR 2 não altera a posição lógica calculada.
        assert_eq!(viewport().escala(), 2.0);
    }
    #[test]
    fn estilos_nao_representaveis_nao_desaparecem_silenciosamente() {
        let mut figura = Figura::Caminho {
            caminho: Caminho { comandos: vec![
                Comando::Mover(DocPonto::novo(0.0, 0.0)),
                Comando::Linha(DocPonto::novo(20.0, 20.0)),
            ], regra: RegraPreenchimento::NaoZero },
            estilo: EstiloVetor {
                preenchimento: None,
                stroke: Some(Stroke::novo(Cor::rgb(0, 0, 0), 1.0)),
                opacidade: 1.0, texto: None,
            },
        };
        assert_eq!(validar(&figura, viewport()), Ok(()));
        if let Figura::Caminho { estilo, .. } = &mut figura {
            estilo.stroke.as_mut().unwrap().dash = vec![2.0, 2.0];
        }
        assert_eq!(validar(&figura, viewport()), Err(ErroIced::TracejadoNaoSuportado));
        if let Figura::Caminho { estilo, .. } = &mut figura {
            estilo.stroke = None;
            estilo.preenchimento = Some(Preenchimento::Linear {
                de: DocPonto::novo(0.0, 0.0),
                ate: DocPonto::novo(20.0, 0.0),
                paradas: vec![
                    feather_draw::Parada { posicao: 0.0, cor: Cor::rgb(0, 0, 0) },
                    feather_draw::Parada { posicao: 1.0, cor: Cor::rgb(255, 255, 255) },
                ],
            });
        }
        assert_eq!(validar(&figura, viewport()), Err(ErroIced::GradienteNaoSuportado));
    }
    #[test]
    fn caminho_real_do_slide_compartilha_curvas_com_exportacao_svg() {
        use featherslide_core::{dados::Primitiva, vetores::Segmento};
        let primitiva = Primitiva::Vetor {
            segmentos: vec![
                Segmento::M([5.0, 10.0]),
                Segmento::Q([15.0, 20.0], [30.0, 15.0]),
                Segmento::C([40.0, 10.0], [50.0, 30.0], [60.0, 20.0]),
                Segmento::Z,
            ],
            par: true,
            preenchimento: "#ff000080".into(),
            contorno: "#0000ff".into(),
            espessura: 2.0,
        };
        let original = primitiva.clone();
        let figura = featherslide_io::drawing::converter_primitiva(&primitiva, false).unwrap();
        assert_eq!(primitiva, original);
        assert_eq!(validar(&figura, viewport()), Ok(()));
        let mut cena = Cena::nova(120.0, 80.0);
        cena.adicionar(figura);
        let svg = feather_draw::exportar_svg(&cena).unwrap();
        assert!(svg.contains("Q 15 20 30 15"));
        assert!(svg.contains("C 40 10 50 30 60 20"));
        assert!(svg.contains(r#"fill-rule="evenodd""#));
        assert!(svg.contains(r#"fill-opacity="0.502""#));
    }

    #[test]
    fn rejeita_comandos_invalidos_e_preserva_z_order_entrada() {
        let mut caminho = Caminho::novo();
        caminho.adicionar(Comando::Linha(DocPonto::novo(10.0, 0.0)));
        assert_eq!(caminho_para_iced(&caminho, viewport()).err(), Some(ErroIced::GeometriaInvalida));
        let mut cena = Cena::nova(100.0, 100.0);
        cena.adicionar(Figura::Elipse {
            centro: DocPonto::novo(40.0, 40.0),
            raio_x: 20.0,
            raio_y: 20.0,
            estilo: EstiloVetor::default(),
        });
        cena.adicionar(Figura::Elipse {
            centro: DocPonto::novo(60.0, 60.0),
            raio_x: 20.0,
            raio_y: 20.0,
            estilo: EstiloVetor::default(),
        });
        assert_eq!(cena.figuras.len(), 2);
        assert!(cena.figuras.iter().all(|f| validar(f, viewport()).is_ok()));
        assert!((cena.figuras[0].bounding_box().unwrap().x - 20.0).abs() < 0.1);
        assert!((cena.figuras[1].bounding_box().unwrap().x - 40.0).abs() < 0.1);
    }
}
