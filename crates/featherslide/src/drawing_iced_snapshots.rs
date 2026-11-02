//! DRW-012: contrato testado na borda REAL Drawing -> Canvas Iced.
//! Snapshot versionado contém coordenadas em pontos, px lógicos e px físicos.
use super::{caminho_para_iced, validar};
use feather_draw::{
    exportar_svg, Cena, Cor, Estilo, Figura, Ponto, Retangulo, Unidade, Viewport,
};

const COORDENADAS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"), "/../../../FeatherOffice/featheroffice/fixtures/drw012/iced-coordenadas.csv"
));
const SVG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"), "/../../../FeatherOffice/featheroffice/fixtures/drw012/snapshot.svg"
));

fn cena() -> Cena {
    let mut cena = Cena::nova(160.0, 100.0);
    for (area, cor) in [
        (Retangulo::novo(10.0, 10.0, 70.0, 35.0), Cor::rgb(255, 0, 0)),
        (Retangulo::novo(35.0, 25.0, 80.0, 45.0), Cor::rgb(0, 0, 255)),
    ] {
        cena.adicionar(Figura::Retangulo {
            area,
            estilo: Estilo {
                preenchimento: Some(cor),
                contorno: None,
                espessura: 1.0,
            },
        });
    }
    cena
}

#[test]
fn drw012_iced_cria_os_mesmos_caminhos_do_snapshot_svg() {
    let vp = Viewport::de_escala_logica(
        Unidade::Ponto, 1.5, Ponto::novo(0.0, 0.0),
        Ponto::novo(3.0, 5.0), 2.0,
    ).unwrap();
    let cena = cena();
    // Mesmo corpus semântico SVG e mesmo modelo que o Canvas percorre.
    assert_eq!(exportar_svg(&cena).unwrap(), SVG);
    assert_eq!(cena.figuras.len(), 2);
    for figura in &cena.figuras {
        validar(figura, vp).expect("Figura do snapshot precisa pintar no Iced");
        let caminho = figura.caminho(0.1);
        let bruto = caminho_para_iced(&caminho, vp).expect("Conversão Canvas");
        assert!(bruto.segments().next().is_some());
    }

    let mut total = 0;
    for linha in COORDENADAS.lines().skip(1) {
        if linha.is_empty() { continue; }
        let n: Vec<f32> = linha.split(',').map(|v| v.parse::<f32>().unwrap()).collect();
        assert_eq!(n.len(), 6);
        let p = Ponto::novo(n[0], n[1]);
        let q = vp.logico(p);
        let fisico = vp.fisico(p);
        assert!((q.x - n[2]).abs() <= 0.25, "px lógico X {linha}");
        assert!((q.y - n[3]).abs() <= 0.25, "px lógico Y {linha}");
        assert!((fisico.x - n[4]).abs() <= 0.5, "px físico X {linha}");
        assert!((fisico.y - n[5]).abs() <= 0.5, "px físico Y {linha}");
        assert!((vp.documento(q).x - p.x).abs() < 0.001);
        assert!((vp.documento(q).y - p.y).abs() < 0.001);
        total += 1;
    }
    assert_eq!(total, 4);
}

#[test]
fn drw012_iced_rejeita_efeitos_sem_paridade_explicita() {
    use feather_draw::{EstiloVetor, Preenchimento, Parada};
    let vp = Viewport::de_escala_logica(
        Unidade::Ponto, 1.5, Ponto::novo(0.0, 0.0),
        Ponto::novo(3.0, 5.0), 1.0,
    ).unwrap();
    let figura = Figura::Elipse {
        centro: Ponto::novo(50.0, 30.0),
        raio_x: 15.0, raio_y: 10.0,
        estilo: EstiloVetor {
            preenchimento: Some(Preenchimento::Linear {
                de: Ponto::novo(20.0, 30.0), ate: Ponto::novo(80.0, 30.0),
                paradas: vec![
                    Parada { posicao: 0.0, cor: Cor::rgb(255, 0, 0) },
                    Parada { posicao: 1.0, cor: Cor::rgb(0, 0, 255) },
                ],
            }),
            ..EstiloVetor::default()
        },
    };
    // SVG/PDF têm gradientes; Iced deve continuar devolvendo erro e
    // acionando fallback, nunca omitir o efeito silenciosamente.
    assert_eq!(validar(&figura, vp), Err(super::ErroIced::GradienteNaoSuportado));
}
