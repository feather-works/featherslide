//! Adaptador de gráficos FeatherSlide → motores compartilhados (CHT-014).
//!
//! O `.fsdz` continua a serializar `Elemento::dados` pelo `featherslide-core`;
//! nenhuma conversão é gravada no elemento. A geometria compartilhada é uma
//! representação opcional para renderização/exportação dos apps.

use featherslide_core::{
    dados::{self, Grafico as GraficoLegado},
    modelo::{Elemento, Tipo as TipoElemento},
};
use feather_charts::{
    AparenciaGrafico, ErroGrafico, ErroPdfGrafico, ErroSvgGrafico,
    Grafico, Serie, Tipo,
};
use feather_draw::{exportar_pdf_com_texto, exportar_svg, Cena, Cor, FontePdf, TextoPdf};

/// Paleta do renderizador legado (indexação por série/categoria), preservada
/// na conversão para evitar a troca incidental das cores do Slide.
pub const CORES_SLIDE: [Cor; 6] = [
    Cor::rgb(0x1a, 0x73, 0xe8),
    Cor::rgb(0x34, 0xa8, 0x53),
    Cor::rgb(0xfb, 0xbc, 0x04),
    Cor::rgb(0xea, 0x43, 0x35),
    Cor::rgb(0x8e, 0x24, 0xaa),
    Cor::rgb(0x00, 0xac, 0xc1),
];

const MAX_CATEGORIAS: usize = 100_000;
const MAX_SERIES: usize = 256;
const MAX_CELULAS: usize = 1_000_000;
const MAX_DIMENSAO_PONTOS: f32 = 1_000_000.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroAdaptacaoGrafico {
    NaoEhGrafico,
    TipoNaoSuportado,
    GradeInvalida,
    TamanhoExcessivo,
    DimensaoInvalida,
    ValorNaoFinito,
    Comum(ErroGrafico),
    Svg(ErroSvgGrafico),
    Pdf(ErroPdfGrafico),
}

impl From<ErroGrafico> for ErroAdaptacaoGrafico {
    fn from(e: ErroGrafico) -> Self { Self::Comum(e) }
}
impl From<ErroSvgGrafico> for ErroAdaptacaoGrafico {
    fn from(e: ErroSvgGrafico) -> Self { Self::Svg(e) }
}
impl From<ErroPdfGrafico> for ErroAdaptacaoGrafico {
    fn from(e: ErroPdfGrafico) -> Self { Self::Pdf(e) }
}

/// Um gráfico do Slide com identidade e posicionamento preservados.
///
/// As dimensões são positivas e a origem é o canto superior esquerdo do
/// retângulo visual. `rotacao` é preservada como metadado: cabe ao adaptador
/// Iced/PDF do app aplicá-la ao grupo, não à engine de Charts.
/// `grafico` contém dados novos e **não** altera `Elemento::dados`.
#[derive(Debug, Clone, PartialEq)]
pub struct GraficoCompartilhadoSlide {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub largura: f32,
    pub altura: f32,
    pub rotacao: f32,
    pub grafico: Grafico,
}

impl GraficoCompartilhadoSlide {
    /// CHT-018: opção de renderização pelo núcleo Rust modular dos 25 tipos.
    /// Não altera a exportação FSDZ e mantém a renderização legada estável.
    pub fn cena_motor(
        &self,
        tipo: feather_charts::motor::TipoGrafico,
    ) -> Result<Cena, String> {
        use feather_charts::motor::{renderizar, Entrada};
        let entrada = Entrada {
            titulo: self.grafico.titulo.clone(),
            categorias: self.grafico.categorias.clone(),
            series: self.grafico.series.clone(),
            x_values: None,
            z_values: None,
        };
        renderizar(tipo, &entrada, self.largura, self.altura)
            .map_err(|e| format!("Motor Charts: {e:?}"))
    }
    /// Cena em coordenadas locais do elemento, em pontos, usando as cores
    /// originais do FeatherSlide. Não aplica translação nem rotação ao slide.
    pub fn cena(&self) -> Result<Cena, ErroAdaptacaoGrafico> {
        let aparencia = AparenciaGrafico {
            paleta: CORES_SLIDE.to_vec(),
            ..Default::default()
        };
        Ok(self.grafico.cena_estilizada(self.largura, self.altura, &aparencia)?)
    }

    /// Rótulos usam as caixas geométricas da pizza compartilhada, assim
    /// como os renderizadores Charts; renderizar é trabalho do Drawing.
    fn textos_pizza(&self) -> Result<Vec<TextoPdf>, ErroAdaptacaoGrafico> {
        if self.grafico.tipo != Tipo::Pizza {
            return Ok(Vec::new());
        }
        let mut textos = Vec::new();
        for fatia in self.grafico.fatias_pizza(self.largura, self.altura)? {
            let Some(r) = fatia.rotulo else { continue };
            if r.texto.is_empty() || r.area.largura <= 0.0 || r.area.altura <= 0.0 {
                continue;
            }
            let tamanho = 11.0_f32.min(r.area.altura).min(
                r.area.largura / r.texto.chars().count().max(1) as f32
            );
            if !tamanho.is_finite() || tamanho <= 0.0 { continue; }
            textos.push(TextoPdf {
                texto: r.texto,
                x: r.area.x,
                y: r.area.y + (r.area.altura + tamanho) * 0.5,
                tamanho,
                cor: Cor::rgb(32, 32, 32),
                fonte: FontePdf::Helvetica,
            });
        }
        Ok(textos)
    }

    /// Saída SVG vetorial a partir da mesma cena/paleta do Slide.
    /// Não modifica o exportador legado de apresentações nem os bytes FSDZ.
    pub fn exportar_svg(&self) -> Result<String, ErroAdaptacaoGrafico> {
        let cena = self.cena()?;
        let mut svg = exportar_svg(&cena)
            .map_err(|e| ErroAdaptacaoGrafico::Svg(ErroSvgGrafico::Svg(e)))?;
        let mut rotulos = String::new();
        for texto in self.textos_pizza()? {
            // Escape obrigatório: categorias vêm da grade editável do usuário.
            let seguro = texto.texto.chars()
                .filter(|c| !c.is_control())
                .collect::<String>()
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&apos;");
            use std::fmt::Write as _;
            writeln!(&mut rotulos,
                "<text x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"{}\" fill=\"#202020\">{}</text>",
                texto.x, texto.y, texto.tamanho, seguro).unwrap();
        }
        if let Some(fim) = svg.rfind("</svg>") {
            svg.insert_str(fim, &rotulos);
        }
        Ok(svg)
    }

    /// PDF vetorial diagnóstico com a mesma cena/paleta do SVG e rótulos.
    /// WinAnsi sem fonte embutida é uma limitação explícita do Drawing.
    pub fn exportar_pdf(&self) -> Result<Vec<u8>, ErroAdaptacaoGrafico> {
        let cena = self.cena()?;
        let textos = self.textos_pizza()?;
        exportar_pdf_com_texto(&cena, &textos)
            .map_err(|e| ErroAdaptacaoGrafico::Pdf(ErroPdfGrafico::Pdf(e)))
    }
}

fn tipo_comum(nome: &str) -> Result<Tipo, ErroAdaptacaoGrafico> {
    match nome {
        "colunas" => Ok(Tipo::Colunas),
        "barras" => Ok(Tipo::Barras),
        "linhas" => Ok(Tipo::Linhas),
        "pizza" => Ok(Tipo::Pizza),
        _ => Err(ErroAdaptacaoGrafico::TipoNaoSuportado),
    }
}

/// Usa exatamente o parser numérico da visualização legada:
/// `trim → vírgula decimal → parse f32 → erro como zero → clamp ≥ 0`.
/// Tipos fora do limite representável são rejeitados, não convertidos
/// silenciosamente em infinito; a grade original nunca é modificada.
fn numero_legado(texto: &str) -> Result<f64, ErroAdaptacaoGrafico> {
    let normalizado = texto.trim().replace(',', ".");
    match normalizado.parse::<f32>() {
        Ok(valor) if !valor.is_finite() => Err(ErroAdaptacaoGrafico::ValorNaoFinito),
        Ok(valor) => Ok(f64::from(valor.max(0.0))),
        Err(_) => Ok(0.0),
    }
}

/// Converte a grade serializada do FeatherSlide para as mesmas categorias e
/// séries do Charts, preservando ordem e os nomes vazios de cabeçalho.
///
/// A pizza legada usa somente a primeira série: o adaptador mantém isso
/// explícito, em vez de rejeitar arquivos antigos com séries adicionais.
pub fn converter_grafico(g: &GraficoLegado) -> Result<Grafico, ErroAdaptacaoGrafico> {
    let tipo = tipo_comum(&g.tipo)?;
    if g.grade.len() < 2 || g.grade[0].len() < 2 {
        return Err(ErroAdaptacaoGrafico::GradeInvalida);
    }
    let categorias = g.grade.len() - 1;
    let n_series = g.grade[0].len() - 1;
    if categorias > MAX_CATEGORIAS
        || n_series > MAX_SERIES
        || categorias.checked_mul(n_series).map_or(true, |n| n > MAX_CELULAS)
    {
        return Err(ErroAdaptacaoGrafico::TamanhoExcessivo);
    }
    // O núcleo usa colunas inexistentes como vazio/0, pois ler_grafico
    // completa grades irregulares durante a decodificação original.
    let rotulos = g.grade.iter().skip(1)
        .map(|linha| linha.first().cloned().unwrap_or_default())
        .collect::<Vec<_>>();
    let total_series = if tipo == Tipo::Pizza { 1 } else { n_series };
    let mut series = Vec::with_capacity(total_series);
    for i in 0..total_series {
        let nome = g.grade[0].get(i + 1).cloned().unwrap_or_default();
        let mut valores = Vec::with_capacity(categorias);
        for linha in g.grade.iter().skip(1) {
            valores.push(numero_legado(
                linha.get(i + 1).map_or("", String::as_str)
            )?);
        }
        series.push(Serie {
            nome,
            valores,
            cor: CORES_SLIDE[i % CORES_SLIDE.len()],
        });
    }
    let resultado = Grafico {
        titulo: g.titulo.clone(),
        tipo,
        categorias: rotulos,
        series,
    };
    resultado.validar()?;
    Ok(resultado)
}

/// Lê um elemento `Tipo::Grafico` real, preservando ID, retângulo,
/// rotação e os bytes originais da grade codificada em `.fsdz`.
pub fn converter_elemento(e: &Elemento) -> Result<GraficoCompartilhadoSlide, ErroAdaptacaoGrafico> {
    if e.tipo != TipoElemento::Grafico {
        return Err(ErroAdaptacaoGrafico::NaoEhGrafico);
    }
    if ![e.x, e.y, e.largura, e.altura, e.rotacao].iter().all(|n| n.is_finite()) {
        return Err(ErroAdaptacaoGrafico::DimensaoInvalida);
    }
    let fim_x = e.x + e.largura;
    let fim_y = e.y + e.altura;
    if !fim_x.is_finite() || !fim_y.is_finite()
        || e.largura.abs() <= 0.0 || e.altura.abs() <= 0.0
        || e.largura.abs() > MAX_DIMENSAO_PONTOS
        || e.altura.abs() > MAX_DIMENSAO_PONTOS
    {
        return Err(ErroAdaptacaoGrafico::DimensaoInvalida);
    }
    let g = converter_grafico(&dados::ler_grafico(e))?;
    Ok(GraficoCompartilhadoSlide {
        id: e.id,
        x: e.x.min(fim_x),
        y: e.y.min(fim_y),
        largura: e.largura.abs(),
        altura: e.altura.abs(),
        rotacao: e.rotacao,
        grafico: g,
    })
}

#[cfg(test)]
mod testes {
    use super::*;

    fn elemento(tipo: &str) -> Elemento {
        let grade = dados::grafico_exemplo(tipo);
        Elemento {
            id: 42,
            tipo: TipoElemento::Grafico,
            x: 10.0,
            y: 20.0,
            largura: 640.0,
            altura: 360.0,
            rotacao: 15.0,
            dados: dados::gravar_grafico(&grade),
            ..Elemento::default()
        }
    }

    #[test]
    fn quatro_tipos_reais_preservam_grade_e_exportam_svg_pdf() {
        for (legado, comum) in [
            ("colunas", Tipo::Colunas),
            ("barras", Tipo::Barras),
            ("linhas", Tipo::Linhas),
            ("pizza", Tipo::Pizza),
        ] {
            let e = elemento(legado);
            let original = e.dados.clone();
            let convertido = converter_elemento(&e).unwrap();
            assert_eq!(convertido.grafico.tipo, comum);
            assert_eq!(convertido.id, 42);
            assert_eq!(convertido.rotacao, 15.0);
            assert_eq!((convertido.x, convertido.y), (10.0, 20.0));
            assert_eq!(convertido.grafico.categorias.len(),
                dados::grafico_exemplo(legado).grade.len() - 1);
            assert!(!convertido.cena().unwrap().figuras.is_empty());
            let svg = convertido.exportar_svg().unwrap();
            let pdf = convertido.exportar_pdf().unwrap();
            assert!(svg.starts_with("<svg"));
            assert!(pdf.starts_with(b"%PDF-1.4"));
            assert_eq!(e.dados, original);
            assert_eq!(dados::gravar_grafico(&dados::ler_grafico(&e)), original);
        }
    }

    #[test]
    fn nome_cor_valor_categoria_e_ordem_preservados() {
        let g = GraficoLegado {
            tipo: "colunas".into(), titulo: "Vendas X".into(),
            grade: vec![
                vec!["".into(), "Real".into(), "Meta".into()],
                vec!["Janeiro".into(), "12,5".into(), "20".into()],
                vec!["Fevereiro".into(), "-2".into(), "invalido".into()],
            ],
        };
        let c = converter_grafico(&g).unwrap();
        assert_eq!(c.titulo, "Vendas X");
        assert_eq!(c.categorias, vec!["Janeiro".to_string(), "Fevereiro".to_string()]);
        assert_eq!(c.series[0].nome, "Real");
        assert_eq!(c.series[1].nome, "Meta");
        assert_eq!(c.series[0].valores, vec![12.5, 0.0]);
        assert_eq!(c.series[1].valores, vec![20.0, 0.0]);
        assert_eq!(c.series[0].cor, CORES_SLIDE[0]);
        assert_eq!(c.series[1].cor, CORES_SLIDE[1]);
        assert_eq!(g.grade[1][1], "12,5");
    }

    #[test]
    fn paleta_svg_pdf_usa_mesma_cena_e_escapa_rotulos() {
        let mut e = elemento("pizza");
        let mut g = dados::ler_grafico(&e);
        g.grade[1][0] = "A<&".into();
        e.dados = dados::gravar_grafico(&g);
        let convertido = converter_elemento(&e).unwrap();
        let svg = convertido.exportar_svg().unwrap();
        let pdf = convertido.exportar_pdf().unwrap();
        assert!(svg.contains("#1a73e8"), "paleta original no primeiro setor");
        assert!(svg.contains("A&lt;&amp;"));
        assert!(!svg.contains("A<&"));
        assert!(String::from_utf8_lossy(&pdf).contains("/MediaBox [0 0 640 360]"));
        assert_eq!(svg, convertido.exportar_svg().unwrap());
        assert_eq!(pdf, convertido.exportar_pdf().unwrap());
        assert_eq!(e.dados, dados::gravar_grafico(&g));
    }

    #[test]
    fn pizza_antiga_usa_primeira_serie_e_fatias_zero_nao_quebram() {
        let g = GraficoLegado {
            tipo: "pizza".into(), titulo: String::new(),
            grade: vec![
                vec!["".into(), "Quantidade".into(), "Ignorada".into()],
                vec!["A".into(), "10".into(), "100".into()],
                vec!["B".into(), "0".into(), "100".into()],
                vec!["C".into(), "20".into(), "100".into()],
            ],
        };
        let c = converter_grafico(&g).unwrap();
        assert_eq!(c.series.len(), 1);
        assert_eq!(c.series[0].valores, vec![10., 0., 20.]);
        assert_eq!(c.fatias_pizza(640., 360.).unwrap().len(), 2);
    }

    #[test]
    fn limites_tipos_ausentes_e_numero_nao_finito_sao_erros() {
        let mut e = elemento("colunas");
        e.tipo = TipoElemento::Imagem;
        assert_eq!(converter_elemento(&e), Err(ErroAdaptacaoGrafico::NaoEhGrafico));
        e.tipo = TipoElemento::Grafico;
        e.largura = f32::NAN;
        assert_eq!(converter_elemento(&e), Err(ErroAdaptacaoGrafico::DimensaoInvalida));
        let g = GraficoLegado { tipo: "radar".into(), titulo: String::new(),
            grade: vec![vec!["".into(), "A".into()], vec!["B".into(), "1".into()]] };
        assert_eq!(converter_grafico(&g), Err(ErroAdaptacaoGrafico::TipoNaoSuportado));
        let mut g = dados::grafico_exemplo("colunas");
        g.grade[1][1] = "1e1000".into();
        assert_eq!(converter_grafico(&g), Err(ErroAdaptacaoGrafico::ValorNaoFinito));
        g.grade = vec![vec![String::new()]];
        assert_eq!(converter_grafico(&g), Err(ErroAdaptacaoGrafico::GradeInvalida));
    }

    #[test]
    fn coordenadas_invertidas_sao_normalizadas_sem_mutar_elemento() {
        let mut e = elemento("linhas");
        e.x = 100.0;
        e.y = 150.0;
        e.largura = -80.0;
        e.altura = -100.0;
        let original = e.clone();
        let a = converter_elemento(&e).unwrap();
        assert_eq!((a.x, a.y, a.largura, a.altura), (20.0, 50.0, 80.0, 100.0));
        assert_eq!(e, original);
    }

    #[test]
    fn recusa_dados_excessivos_antes_de_alocar_series() {
        let mut g = dados::grafico_exemplo("linhas");
        g.grade = vec![vec![String::new(), "A".into()]];
        g.grade.extend((0..MAX_CATEGORIAS + 1).map(|i| vec![
            format!("Cat {i}"), "1".into()
        ]));
        assert_eq!(converter_grafico(&g), Err(ErroAdaptacaoGrafico::TamanhoExcessivo));
    }
    #[test]
    fn motor_compartilhado_renderiza_colunas_empilhadas_sem_mudar_elemento() {
        let origem = elemento("colunas");
        let antigo = origem.clone();
        let grafico = converter_elemento(&origem).unwrap();
        let cena = grafico.cena_motor(
            feather_charts::motor::TipoGrafico::ColunasEmpilhadas
        ).unwrap();
        assert!(!cena.figuras.is_empty());
        assert_eq!(origem, antigo, "motor compartilhado não pode modificar FSDZ");
    }
}
