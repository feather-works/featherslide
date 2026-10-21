//! Diálogos de SVG; leitura e geometria pertencem a IO e core.
use crate::{
    app::{FeatherSlideApp, Mensagem},
    faixa::FaixaMensagem,
};
use pelicano::Task;


/// Obtém o SVG Charts a partir dos dados reais do elemento, sem alterar FSDZ.
/// A seleção pode conter outros elementos, mas deve incluir um gráfico.
fn svg_grafico_selecionado(
    elementos: &[featherslide_core::modelo::Elemento],
    ids: &[u64],
) -> Result<String, String> {
    let elemento = elementos.iter().find(|e| {
        ids.contains(&e.id) && e.tipo == featherslide_core::modelo::Tipo::Grafico
    }).ok_or_else(|| "Selecione um gráfico para exportar.".to_owned())?;
    let comum = featherslide_io::grafico_compartilhado::converter_elemento(elemento)
        .map_err(|e| format!("Não foi possível converter o gráfico: {e:?}"))?;
    comum.exportar_svg()
        .map_err(|e| format!("Não foi possível exportar o gráfico SVG: {e:?}"))
}

impl FeatherSlideApp {
    pub(crate) fn tratar_vetores(&mut self, msg: FaixaMensagem) -> Task<Mensagem> {
        match msg {
            FaixaMensagem::InserirSvg => Task::perform(
                async {
                    let arquivo = rfd::AsyncFileDialog::new()
                        .add_filter("Desenho vetorial SVG", &["svg"])
                        .pick_file()
                        .await?;
                    let caminho = arquivo.path().to_path_buf();
                    Some(
                        tokio::task::spawn_blocking(move || {
                            let bytes = std::fs::read(caminho)
                                .map_err(|e| format!("Erro ao ler SVG: {e}"))?;
                            featherslide_io::svg::importar(&bytes)
                        })
                        .await
                        .unwrap_or_else(|e| Err(format!("Erro ao importar SVG: {e}"))),
                    )
                },
                Mensagem::SvgLido,
            ),
            FaixaMensagem::ConverterSvg => {
                self.status = match self.sessao.svg_converter() {
                    Ok(_) => "SVG convertido: peças liberadas para edição".into(),
                    Err(e) => e,
                };
                Task::none()
            }
            FaixaMensagem::ExportarGraficoSvg => {
                let ids = self.sessao.estado().selecao;
                let svg = match svg_grafico_selecionado(
                    &self.sessao.slide().elementos,
                    &ids,
                ) {
                    Ok(svg) => svg,
                    Err(e) => {
                        self.status = e;
                        return Task::none();
                    }
                };
                Task::perform(
                    async move {
                        let Some(arquivo) = rfd::AsyncFileDialog::new()
                            .add_filter("Gráfico vetorial SVG", &["svg"])
                            .set_file_name("grafico.svg")
                            .save_file()
                            .await
                        else {
                            return Ok("Exportação do gráfico cancelada".into());
                        };
                        std::fs::write(arquivo.path(), svg)
                            .map_err(|e| format!("Erro ao exportar gráfico SVG: {e}"))?;
                        Ok("Gráfico exportado pelo motor Charts para SVG".into())
                    },
                    Mensagem::SvgExportado,
                )
            }
            FaixaMensagem::ExportarSvg => {
                let ids = self.sessao.estado().selecao;
                let elementos: Vec<_> = self
                    .sessao
                    .slide()
                    .elementos
                    .iter()
                    .filter(|e| ids.contains(&e.id))
                    .cloned()
                    .collect();
                let (w, h) = self.sessao.apresentacao().tamanho();
                let svg = match featherslide_io::drawing::exportar_svg(&elementos, [w, h]) {
                    Ok(svg) => svg,
                    Err(e) => {
                        self.status = e;
                        return Task::none();
                    }
                };
                Task::perform(
                    async move {
                        let Some(arquivo) = rfd::AsyncFileDialog::new()
                            .add_filter("SVG", &["svg"])
                            .set_file_name("vetores.svg")
                            .save_file()
                            .await
                        else {
                            return Ok("Exportação SVG cancelada".into());
                        };
                        std::fs::write(arquivo.path(), svg)
                            .map_err(|e| format!("Erro ao exportar SVG: {e}"))?;
                        Ok("Vetores exportados para SVG".into())
                    },
                    Mensagem::SvgExportado,
                )
            }
            _ => Task::none(),
        }
    }
}

#[cfg(test)]
mod testes {
    use super::svg_grafico_selecionado;
    use featherslide_core::{
        dados,
        modelo::{Elemento, Tipo},
    };

    #[test]
    fn app003_svg_da_ferramenta_usa_charts_sem_mutar_dados_do_slide() {
        let grade = dados::grafico_exemplo("colunas");
        let original = dados::gravar_grafico(&grade);
        let grafico = Elemento {
            id: 501, tipo: Tipo::Grafico, x: 15.0, y: 20.0,
            largura: 640.0, altura: 360.0,
            dados: original.clone(), ..Elemento::default()
        };
        let outros = Elemento { id: 502, tipo: Tipo::Imagem, ..Elemento::default() };
        let svg = svg_grafico_selecionado(&[outros, grafico.clone()], &[502, 501]).unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("#1a73e8"), "paleta Charts com cores do Slide");
        assert_eq!(grafico.dados, original, "o FSDZ não muda na exportação");
        assert_eq!(dados::gravar_grafico(&dados::ler_grafico(&grafico)), original);
    }

    #[test]
    fn app003_rejeita_selecao_sem_grafico_sem_salvar_arquivo() {
        let forma = Elemento { id: 2, tipo: Tipo::Retangulo, ..Elemento::default() };
        assert!(svg_grafico_selecionado(&[forma], &[2]).is_err());
        assert!(svg_grafico_selecionado(&[], &[]).is_err());
    }

    #[test]
    fn app003_tabela_native_preserva_dados_na_ponte_compartilhada() {
        let grade = vec![
            vec!["Cabeçalho".to_owned(), "Valor".to_owned()],
            vec!["Café".to_owned(), "25".to_owned()],
        ];
        let original = Elemento {
            id: 49, tipo: Tipo::Tabela, x: 30.0, y: 60.0,
            largura: 400.0, altura: 160.0,
            dados: dados::gravar_tabela(&grade), ..Elemento::default()
        };
        let comum = featherslide_tabelas::slide_para_comum(&original).unwrap();
        assert_eq!(comum.dimensoes(), (2, 2));
        let volta = featherslide_tabelas::comum_para_slide(&comum, &original).unwrap();
        assert_eq!(volta, original);
    }

    #[test]
    fn previa_renderiza_furo_sem_rasterizar_documento() {
        let d=featherslide_io::svg::importar(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><path fill="#ff0000" fill-rule="evenodd" d="M0 0H100V100H0Z M25 25H75V75H25Z"/></svg>"##).unwrap();
        let mut sessao = featherslide_core::sessao::Sessao::default();
        sessao.slide_novo("em_branco").unwrap();
        sessao.svg_inserir(d.elementos, d.tamanho).unwrap();
        let quadro = sessao.quadro(sessao.atual(), None, false).unwrap().unwrap();
        let fonte = featherslide_mcp::fonte::carregar();
        let bytes =
            featherslide_mcp::renderizar::renderizar_png(&quadro, 960, false, &fonte).unwrap();
        let imagem = image::load_from_memory(&bytes).unwrap().to_rgba8();
        let e = sessao.slide().elementos.last().unwrap();
        let vermelho = imagem.get_pixel((e.x + 10.0) as u32, (e.y + 10.0) as u32);
        let furo = imagem.get_pixel((e.x + 50.0) as u32, (e.y + 50.0) as u32);
        assert_eq!(&vermelho.0[..3], &[255, 0, 0]);
        assert_eq!(&furo.0[..3], &[255, 255, 255]);
        assert!(e.imagem.is_empty());
    }
}
