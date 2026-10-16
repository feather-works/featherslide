//! CHT-017: regressão semântica e vetorial do FeatherSlide legado x Charts.
//! Usa as primitivas realmente renderizadas pelo core, e NÃO reimplementa
//! a geometria nem altera o modelo serializado .fsdz.
use featherslide_core::{
    dados::{self, Grafico as GraficoLegado, Primitiva},
    modelo::{Elemento, Slide, Tipo as TipoElemento},
    quadro::{self, Dicas},
};
use featherslide_io::{grafico_compartilhado::{converter_elemento, CORES_SLIDE}, pdf};
use feather_charts::Tipo;
use feather_draw::{Cor, Figura};

const NOMES: [&str; 4] = ["colunas", "barras", "linhas", "pizza"];

fn legado(tipo: &str) -> Elemento {
    let g = GraficoLegado {
        tipo: tipo.into(),
        titulo: "Vendas 2026".into(),
        grade: vec![
            vec!["".into(), "Receita".into(), "Meta".into()],
            vec!["Jan".into(), "10".into(), "5".into()],
            vec!["Fev".into(), "20".into(), "12".into()],
            vec!["Mar".into(), "70".into(), "32".into()],
        ],
    };
    Elemento {
        id: 731, tipo: TipoElemento::Grafico, x: 40.0, y: 70.0,
        largura: 640.0, altura: 360.0,
        dados: dados::gravar_grafico(&g), ..Elemento::default()
    }
}

fn cor_hex(c: Cor) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

fn texto_legado<'a>(p: &'a [Primitiva]) -> impl Iterator<Item = &'a str> {
    p.iter().filter_map(|x| match x {
        Primitiva::Texto { texto, .. } => Some(texto.as_str()),
        _ => None,
    })
}

#[test]
fn categorias_series_titulos_valores_e_soma_permanecem_iguais() {
    for tipo in NOMES {
        let e = legado(tipo);
        let original = e.dados.clone();
        let antiga = dados::ler_grafico(&e);
        let atual = converter_elemento(&e).unwrap().grafico;
        assert_eq!(atual.titulo, antiga.titulo, "{tipo}");
        assert_eq!(atual.categorias,
            antiga.grade.iter().skip(1).map(|r| r[0].clone()).collect::<Vec<_>>());
        let n_series = if tipo == "pizza" { 1 } else { 2 };
        assert_eq!(atual.series.len(), n_series);
        for (indice, serie) in atual.series.iter().enumerate() {
            assert_eq!(serie.nome, antiga.grade[0][indice + 1]);
            let original: Vec<f64> = antiga.grade.iter().skip(1)
                .map(|r| r[indice + 1].parse::<f32>().unwrap() as f64).collect();
            assert_eq!(serie.valores, original);
            assert_eq!(serie.valores.iter().sum::<f64>(), original.iter().sum::<f64>());
            assert_eq!(serie.cor, CORES_SLIDE[indice]);
        }
        let primitivas = dados::primitivas_grafico(&e);
        let rotulos: Vec<_> = texto_legado(&primitivas).collect();
        assert!(rotulos.contains(&"Vendas 2026"), "título legado ausente: {tipo}");
        assert!(rotulos.contains(&"Jan"), "primeira categoria ausente: {tipo}");
        assert!(rotulos.contains(&"Mar"), "última categoria ausente: {tipo}");
        assert_eq!(e.dados, original);
        assert_eq!(dados::gravar_grafico(&dados::ler_grafico(&e)), original);
    }
}

#[test]
fn mesmas_cores_e_quantidade_de_marcas_no_desenho_vetorial() {
    for tipo in NOMES {
        let e = legado(tipo);
        let velho = dados::primitivas_grafico(&e);
        let novo = converter_elemento(&e).unwrap().cena().unwrap();
        let tipo_grafico = converter_elemento(&e).unwrap().grafico.tipo;
        match tipo_grafico {
            Tipo::Colunas | Tipo::Barras => {
                let marcacoes_velhas: Vec<_> = velho.iter().filter_map(|f| match f {
                    Primitiva::Ret { preenchimento, w, h, .. }
                        if *w > 0.0 && *h > 0.0
                        && CORES_SLIDE[..2].iter().any(|&c| cor_hex(c) == *preenchimento)
                        && !(*w == 10.0 && *h == 10.0)
                        => Some(preenchimento.as_str()),
                    _ => None,
                }).collect();
                let marcacoes_novas: Vec<_> = novo.figuras.iter().filter_map(|f| match f {
                    Figura::Retangulo { estilo, .. } => estilo.preenchimento,
                    _ => None,
                }).collect();
                assert_eq!(marcacoes_velhas.len(), 6, "barras legadas: {tipo}");
                assert_eq!(marcacoes_novas.len(), 6, "barras novas: {tipo}");
                for cor in CORES_SLIDE.iter().take(2) {
                    let old = marcacoes_velhas.iter().filter(|&&s| s == cor_hex(*cor)).count();
                    let new = marcacoes_novas.iter().filter(|&&c| c == *cor).count();
                    assert_eq!(old, 3, "{tipo}: paleta antiga");
                    assert_eq!(new, old, "{tipo}: paleta nova");
                }
            }
            Tipo::Linhas => {
                for cor in CORES_SLIDE.iter().take(2) {
                    let antigo = velho.iter().any(|x| match x {
                        Primitiva::Poli { cor: preenchimento, pontos, .. } =>
                            preenchimento == &cor_hex(*cor) && pontos.len() == 3,
                        _ => false,
                    });
                    let novo_tem_linha = novo.figuras.iter().any(|x| match x {
                        Figura::Polilinha { pontos, estilo, .. } =>
                            pontos.len() == 3 && estilo.contorno == Some(*cor),
                        _ => false,
                    });
                    assert!(antigo && novo_tem_linha, "linha sem cor/ordem: {cor:?}");
                }
            }
            Tipo::Pizza => {
                let sectores = velho.iter().filter(|x| matches!(x, Primitiva::Setor { .. })).count();
                assert_eq!(sectores, 3);
                let setores_comuns: Vec<_> = novo.figuras.iter().filter_map(|f| match f {
                    Figura::Polilinha { fechada: true, estilo, .. } => estilo.preenchimento,
                    _ => None,
                }).collect();
                assert_eq!(setores_comuns.as_slice(), &CORES_SLIDE[..3]);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn escala_legada_arredondada_e_escala_comum_exata_sao_diferencas_explicitas() {
    for nome in ["colunas", "barras", "linhas"] {
        let e = legado(nome);
        let c = converter_elemento(&e).unwrap().grafico;
        let antigo = dados::primitivas_grafico(&e);
        // O layout antigo aplica "escala bonita" (80 para máximo 70);
        // Charts usa domínio real (0..70). É diferença de layout, não dado.
        let maximo_tick_legado = texto_legado(&antigo)
            .filter_map(|txt| txt.parse::<f64>().ok())
            .fold(0.0_f64, f64::max);
        let eixos = c.eixos(640.0, 360.0).unwrap().unwrap();
        let maximo_comum = match c.tipo {
            Tipo::Barras => eixos.escala_x.unwrap().maximo,
            _ => eixos.escala_y.unwrap().maximo,
        };
        assert_eq!(maximo_comum, 70.0);
        assert_eq!(maximo_tick_legado, 80.0, "{nome}: escala bonita legada");
        assert_eq!(c.series.iter().flat_map(|s| &s.valores)
            .copied().fold(f64::NEG_INFINITY, f64::max), 70.0);
    }
}

fn slide_com_elemento(e: Elemento) -> Slide {
    Slide {
        id: 1, layout: String::new(), fundo: "#ffffff".into(),
        fundo_config: Default::default(), ocultar_mestre: false,
        elementos: vec![e], notas: String::new(), oculto: false,
        transicao: Default::default(), secao: String::new(),
        comentarios: Vec::new(),
    }
}

#[test]
fn exportacao_pdf_legada_e_svg_pdf_novos_sao_vetoriais_sem_mudar_documento() {
    for tipo in NOMES {
        let e = legado(tipo);
        let antes = e.clone();
        let slide = slide_com_elemento(e.clone());
        let quadro = quadro::quadro(&slide, (960.0, 540.0), 1, &Dicas::default(), false);
        assert_eq!(quadro.elementos[0].id, 731);
        assert!(!quadro.elementos[0].primitivas.is_empty());
        let pdf_legado = pdf::exportar("Vendas", &[quadro], &|_| None);
        let adaptador = converter_elemento(&e).unwrap();
        let svg_novo = adaptador.exportar_svg().unwrap();
        let pdf_novo = adaptador.exportar_pdf().unwrap();
        assert!(pdf_legado.starts_with(b"%PDF-"));
        assert!(pdf_novo.starts_with(b"%PDF-1.4"));
        assert!(svg_novo.starts_with("<svg"));
        assert!(svg_novo.contains("<path"));
        assert!(String::from_utf8_lossy(&pdf_novo).contains("/MediaBox [0 0 640 360]"));
        assert_ne!(pdf_legado, pdf_novo, "formatos de pagina diferentes: {tipo}");
        assert_eq!(e, antes, "modelo não pode ser modificado: {tipo}");
    }
}

#[test]
fn casos_problematicos_preservam_semantica_antiga_e_sinalizam_diferencas() {
    for tipo in NOMES {
        let mut e = legado(tipo);
        let mut g = dados::ler_grafico(&e);
        g.grade[1][1] = "-10".into();
        g.grade[2][1] = "vazio".into();
        g.grade[3][1] = "12,5".into();
        e.dados = dados::gravar_grafico(&g);
        let comum = converter_elemento(&e).unwrap().grafico;
        assert_eq!(comum.series[0].valores, [0.0, 0.0, 12.5]);
        assert_eq!(e.dados, dados::gravar_grafico(&g));
        assert!(!dados::primitivas_grafico(&e).is_empty());
    }
}
