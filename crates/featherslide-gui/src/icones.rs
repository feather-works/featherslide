//! Ícones da interface nativa: Tabler Icons 3.49.0 (MIT, © Paweł Kuna).
//! Usa os mesmos caminhos SVG embutidos da galeria Inserir → Ícones.
//! Origem e licença: `dados/TABLER-LICENSE.txt` do feather-shapes (`featherslide_core::icones::LICENCA_TABLER`).

use iced::widget::svg;
use iced::Length;

/// Monta o SVG original do Tabler com a cor do tema, sem acesso à rede.
fn svg_tabler(id: &str, cor: &str) -> String {
    let icone = featherslide_core::icones::buscar_por_id(id)
        .expect("ícone da interface deve existir na biblioteca Tabler embutida");
    let caminhos: String = icone.caminhos_svg().map(|d| format!(r#"<path d="{d}"/>"#)).collect();
    format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="{}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{caminhos}</svg>"#, crate::tema::cor_do_icone(cor))
}

/// Desenha um ícone Tabler no tamanho solicitado, mantendo o traço proporcional.
fn tabler(id: &str, tamanho: f32, cor: &str) -> svg::Svg<'static, iced::Theme> {
    svg(svg::Handle::from_memory(svg_tabler(id, cor).into_bytes()))
        .width(Length::Fixed(tamanho))
        .height(Length::Fixed(tamanho))
}

/// Ícones das operações de mesclagem, da biblioteca Tabler já embutida.
pub fn icone_mesclar_formas(op: featherslide_core::booleanas::OperacaoBooleana, tamanho: f32, cor: &str) -> svg::Svg<'static, iced::Theme> {
    use featherslide_core::booleanas::OperacaoBooleana;
    let id = match op {
        OperacaoBooleana::Uniao => "layers-union",
        OperacaoBooleana::Intersecao => "layers-intersect-2",
        OperacaoBooleana::Subtracao => "layers-subtract",
        OperacaoBooleana::Combinacao => "layers-difference",
        OperacaoBooleana::Fragmentacao => "layers-intersect",
    };
    tabler(id, tamanho, cor)
}

/// Pinta um ícone Tabler num canvas usando os mesmos caminhos da interface SVG.
pub fn pintar_no_canvas(frame: &mut iced::widget::canvas::Frame, id: &str, x: f32, y: f32, tamanho: f32, cor: iced::Color) {
    use iced::widget::canvas::{Path, Stroke};
    let icone = featherslide_core::icones::buscar_por_id(id)
        .expect("ícone do canvas deve existir na biblioteca Tabler embutida");
    let escala = tamanho / 24.0;
    for d in icone.caminhos_svg() {
        for traco in featherslide_core::svgpath::tracos(d) {
            let path = Path::new(|b| {
                for (i, p) in traco.pontos.iter().enumerate() {
                    let ponto = iced::Point::new(x + p[0] * escala, y + p[1] * escala);
                    if i == 0 { b.move_to(ponto); } else { b.line_to(ponto); }
                }
                if traco.fechado { b.close(); }
            });
            frame.stroke(&path, Stroke::default().with_color(cor).with_width(2.0 * escala));
        }
    }
}

/// Ícone Tabler para aumentar o tamanho da fonte.
pub fn icone_fonte_mais(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("text-increase", tamanho, cor)
}

/// Ícone Tabler para diminuir o tamanho da fonte.
pub fn icone_fonte_menos(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("text-decrease", tamanho, cor)
}

/// Ícone Tabler para colar.
pub fn icone_colar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("clipboard", tamanho, cor)
}

/// Ícone Tabler para recortar.
pub fn icone_recortar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("scissors", tamanho, cor)
}

/// Ícone Tabler para cortar figura.
pub fn icone_cortar_figura(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("crop", tamanho, cor)
}

/// Ícone Tabler para copiar.
pub fn icone_copiar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("copy", tamanho, cor)
}

/// Ícone Tabler para duplicar.
pub fn icone_duplicar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("copy-plus", tamanho, cor)
}

/// Ícone Tabler para novo slide.
pub fn icone_novo_slide(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("presentation", tamanho, cor)
}

/// Ícone Tabler para layout.
pub fn icone_layout(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("layout", tamanho, cor)
}

/// Ícone Tabler para excluir.
pub fn icone_excluir(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("trash", tamanho, cor)
}

/// Ícone Tabler para negrito.
pub fn icone_negrito(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("bold", tamanho, cor)
}

/// Ícone Tabler para italico.
pub fn icone_italico(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("italic", tamanho, cor)
}

/// Ícone Tabler para sublinhado.
pub fn icone_sublinhado(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("underline", tamanho, cor)
}

/// Um ícone Tabler de letras (`letter-case`, `letter-spacing`…) pelo nome.
pub fn icone_letras(id: &str, tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler(id, tamanho, cor)
}

/// Sublinhado duplo: o `underline` do Tabler com um segundo traço.
pub fn icone_sublinhado_duplo(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    traco_duplo("M7 4v5a5 5 0 0 0 10 0v-5M5 17h14M5 21h14", tamanho, cor)
}

/// Tachado duplo: o `strikethrough` do Tabler com dois traços.
pub fn icone_tachado_duplo(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    traco_duplo("M4 10h16M4 14h16M16 6.5a4 2 0 0 0 -4 -1.5h-1a3.5 3.5 0 0 0 -1.5 6.5M8 17.5a4 2 0 0 0 4 1.5h1a3.5 3.5 0 0 0 1.5 -5", tamanho, cor)
}

/// Desenho próprio no estilo Tabler (traço 2, cantos arredondados, 24×24).
fn traco_duplo(caminho: &str, tamanho: f32, cor: &str) -> svg::Svg<'static, iced::Theme> {
    let data = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"{caminho}\"/></svg>", crate::tema::cor_do_icone(cor));
    svg::Svg::new(svg::Handle::from_memory(data.into_bytes())).width(tamanho).height(tamanho)
}

/// Ícone Tabler para tachado.
pub fn icone_tachado(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("strikethrough", tamanho, cor)
}

/// Ícone Tabler para limpar.
pub fn icone_limpar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("eraser", tamanho, cor)
}

/// Ícone Tabler para lista.
pub fn icone_lista(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("list", tamanho, cor)
}

/// Ícone Tabler para lista num.
pub fn icone_lista_num(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("list-numbers", tamanho, cor)
}

/// Barras de parágrafo com peso compatível com os ícones de alinhamento vertical.
fn tabler_paragrafo(id: &str, tamanho: f32, cor: &str) -> svg::Svg<'static, iced::Theme> {
    let desenho = svg_tabler(id, cor).replace("stroke-width=\"2\"", "stroke-width=\"3\"");
    svg(svg::Handle::from_memory(desenho.into_bytes()))
        .width(Length::Fixed(tamanho))
        .height(Length::Fixed(tamanho))
}

/// Ícone Tabler para alinhar esq.
pub fn icone_alinhar_esq(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler_paragrafo("align-left", tamanho, cor)
}

/// Ícone Tabler para alinhar centro.
pub fn icone_alinhar_centro(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler_paragrafo("align-center", tamanho, cor)
}

/// Ícone Tabler para alinhar dir.
pub fn icone_alinhar_dir(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler_paragrafo("align-right", tamanho, cor)
}

/// Ícone Tabler para alinhar just.
pub fn icone_alinhar_just(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler_paragrafo("align-justified", tamanho, cor)
}

/// Ícone Tabler para formas.
pub fn icone_formas(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("geometry", tamanho, cor)
}

/// Ícone Tabler para paleta.
pub fn icone_paleta(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("palette", tamanho, cor)
}

/// Ícone Tabler para lapis.
pub fn icone_lapis(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("pencil", tamanho, cor)
}

/// Ícone Tabler para organizar.
pub fn icone_organizar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("layers-intersect", tamanho, cor)
}

/// Ícone Tabler para buscar.
pub fn icone_buscar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("search", tamanho, cor)
}

/// Ícone Tabler para substituir.
pub fn icone_substituir(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("replace", tamanho, cor)
}

/// Ícone Tabler para selecionar.
pub fn icone_selecionar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("pointer", tamanho, cor)
}

/// Ícone Tabler para inicio.
pub fn icone_inicio(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("home", tamanho, cor)
}

/// Ícone Tabler para ajuda.
pub fn icone_ajuda(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("help", tamanho, cor)
}

/// Ícone Tabler para config.
pub fn icone_config(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("settings", tamanho, cor)
}

/// Ícone Tabler para salvar.
pub fn icone_salvar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("device-floppy", tamanho, cor)
}

/// Ícone Tabler para pdf.
pub fn icone_pdf(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("file-type-pdf", tamanho, cor)
}

/// Ícone Tabler para ortografia.
pub fn icone_ortografia(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("text-spellcheck", tamanho, cor)
}

/// Ícone Tabler para imprimir.
pub fn icone_imprimir(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("printer", tamanho, cor)
}

/// Ícone Tabler para desfazer.
pub fn icone_desfazer(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("arrow-back-up", tamanho, cor)
}

/// Ícone Tabler para refazer.
pub fn icone_refazer(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("arrow-forward-up", tamanho, cor)
}

/// Ícone Tabler para apresentar.
pub fn icone_apresentar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("player-play", tamanho, cor)
}

/// Ícone Tabler para tabela.
pub fn icone_tabela(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("table", tamanho, cor)
}

/// Ícone Tabler para imagem.
pub fn icone_imagem(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("photo", tamanho, cor)
}

/// Ícone Tabler para grafico.
pub fn icone_grafico(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("chart-bar", tamanho, cor)
}

/// Ícone Tabler para nota.
pub fn icone_nota(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("notes", tamanho, cor)
}

/// Ícone Tabler para caret baixo.
pub fn icone_caret_baixo(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("chevron-down", tamanho, cor)
}

/// Ícone Tabler para seta baixo.
pub fn icone_seta_baixo(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("chevron-down", tamanho, cor)
}

/// Ícone Tabler para recuo menos.
pub fn icone_recuo_menos(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("indent-decrease", tamanho, cor)
}

/// Ícone Tabler para recuo mais.
pub fn icone_recuo_mais(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("indent-increase", tamanho, cor)
}

/// Ícone Tabler para transicao.
pub fn icone_transicao(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("transition-right", tamanho, cor)
}

/// Ícone Tabler para animacao.
pub fn icone_animacao(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("sparkles", tamanho, cor)
}

/// Ícone Tabler para relogio.
pub fn icone_relogio(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("clock", tamanho, cor)
}

/// Ícone Tabler para calendario.
pub fn icone_calendario(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("calendar", tamanho, cor)
}

/// Ícone Tabler para numero.
pub fn icone_numero(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("hash", tamanho, cor)
}

/// Ícone Tabler para caixa texto.
pub fn icone_caixa_texto(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("text-size", tamanho, cor)
}

/// Ícone Tabler para wordart.
pub fn icone_wordart(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("typography", tamanho, cor)
}

/// Ícone Tabler para link.
pub fn icone_link(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("link", tamanho, cor)
}

/// Ícone Tabler para video.
pub fn icone_video(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("video", tamanho, cor)
}

/// Ícone Tabler para audio.
pub fn icone_audio(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("volume", tamanho, cor)
}

/// Ícone Tabler para equacao.
pub fn icone_equacao(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("math-pi", tamanho, cor)
}

/// Ícone Tabler para simbolo.
pub fn icone_simbolo(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("omega", tamanho, cor)
}

/// Ícone Tabler para sorriso.
pub fn icone_sorriso(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("mood-smile", tamanho, cor)
}

/// Ícone Tabler para comentario.
pub fn icone_comentario(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("message", tamanho, cor)
}

/// Ícone Tabler para grade.
pub fn icone_grade(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("grid-pattern", tamanho, cor)
}

/// Ícone Tabler para guias.
pub fn icone_guias(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("crosshair", tamanho, cor)
}

/// Ícone Tabler para regua.
pub fn icone_regua(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("ruler", tamanho, cor)
}

/// Ícone Tabler para ocultar.
pub fn icone_ocultar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("eye-off", tamanho, cor)
}

/// Ícone Tabler para repetir.
pub fn icone_repetir(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("repeat", tamanho, cor)
}

/// Ícone Tabler para painel.
pub fn icone_painel(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("layout-sidebar-right", tamanho, cor)
}

/// Ícone Tabler para notas painel.
pub fn icone_notas_painel(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("notes", tamanho, cor)
}

/// Ícone Tabler para zoom mais.
pub fn icone_zoom_mais(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("zoom-in", tamanho, cor)
}

/// Ícone Tabler para zoom menos.
pub fn icone_zoom_menos(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("zoom-out", tamanho, cor)
}

/// Ícone Tabler para ajustar.
pub fn icone_ajustar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("maximize", tamanho, cor)
}

/// Ícone Tabler para monitor.
pub fn icone_monitor(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("device-desktop", tamanho, cor)
}

/// Ícone Tabler para gravar.
pub fn icone_gravar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("player-record", tamanho, cor)
}

/// Ícone Tabler para classificacao.
pub fn icone_classificacao(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("layout-grid", tamanho, cor)
}

/// Ícone Tabler para normal.
pub fn icone_normal(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("layout-sidebar", tamanho, cor)
}

/// Ícone Tabler para linha mais.
pub fn icone_linha_mais(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("row-insert-bottom", tamanho, cor)
}

/// Ícone Tabler para linha menos.
pub fn icone_linha_menos(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("row-remove", tamanho, cor)
}

/// Ícone Tabler para coluna mais.
pub fn icone_coluna_mais(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("column-insert-right", tamanho, cor)
}

/// Ícone Tabler para coluna menos.
pub fn icone_coluna_menos(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("column-remove", tamanho, cor)
}

/// Ícone Tabler para dados.
pub fn icone_dados(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("chart-bar", tamanho, cor)
}

/// Ícone Tabler para subir.
pub fn icone_subir(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("arrow-up", tamanho, cor)
}

/// Ícone Tabler para descer.
pub fn icone_descer(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("arrow-down", tamanho, cor)
}

/// Ícone Tabler para tema.
pub fn icone_tema(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("palette", tamanho, cor)
}

/// Ícone Tabler para fundo.
pub fn icone_fundo(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("background", tamanho, cor)
}

/// Ícone Tabler para executar.
pub fn icone_executar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("player-play", tamanho, cor)
}

/// Ícone Tabler para cancelar.
pub fn icone_cancelar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("x", tamanho, cor)
}

/// Ícone Tabler para seta esq.
pub fn icone_seta_esq(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("chevron-left", tamanho, cor)
}

/// Ícone Tabler para seta dir.
pub fn icone_seta_dir(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("chevron-right", tamanho, cor)
}

/// Ícone Tabler para pasta.
pub fn icone_pasta(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("folder", tamanho, cor)
}

/// Ícone Tabler para casa.
pub fn icone_casa(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("home", tamanho, cor)
}

/// Ícone Tabler para voltar.
pub fn icone_voltar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("arrow-back-up", tamanho, cor)
}

/// Ícone Tabler para girar.
pub fn icone_girar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("rotate-clockwise", tamanho, cor)
}

/// Ícone Tabler para angulo.
pub fn icone_angulo(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("angle", tamanho, cor)
}

/// Ícone Tabler para arredondar.
pub fn icone_arredondar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("border-corner-rounded", tamanho, cor)
}

/// Ícone Tabler para avancar auto.
pub fn icone_avancar_auto(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("clock-play", tamanho, cor)
}

/// Ícone Tabler para ampulheta.
pub fn icone_ampulheta(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("hourglass", tamanho, cor)
}

/// Ícone Tabler para ponta.
pub fn icone_ponta(qual: usize, tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    let id = match qual {
        0 => "border-corner-rounded",
        1 => "arrow-up-left",
        2 => "arrow-up-right",
        3 => "arrow-down-right",
        4 => "arrow-down-left",
        5 => "arrow-up",
        6 => "arrow-down-right",
        _ => "arrow-down-left",
    };
    tabler(id, tamanho, cor)
}

/// Ícone Tabler de alinhamento com referências visuais legíveis na faixa.
pub fn icone_alinhar(qual: usize, tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    let id = match qual {
        0 => "align-box-left-middle",
        1 => "align-box-center-middle",
        2 => "align-box-right-middle",
        3 => "align-box-top-center",
        4 => "align-box-center-middle",
        5 => "align-box-bottom-center",
        6 => "spacing-horizontal",
        _ => "spacing-vertical",
    };
    let mut desenho = svg_tabler(id, cor);
    if qual < 6 {
        // Preserva os traços de alinhamento do Tabler e remove somente o quadro externo.
        let icone = featherslide_core::icones::buscar_por_id(id)
            .expect("ícone de alinhamento deve existir na biblioteca Tabler");
        for caminho in icone.caminhos_svg() {
            if caminho.starts_with("M3 ") && caminho.contains("h14") {
                desenho = desenho.replace(&format!(r#"<path d="{caminho}"/>"#), "");
            }
        }
        desenho = desenho.replace("viewBox=\"0 0 24 24\"", "viewBox=\"4 4 16 16\"");
        if qual == 4 {
            desenho = desenho
                .replacen("<path", "<g transform=\"rotate(90 12 12)\"><path", 1)
                .replace("</svg>", "</g></svg>");
        }
    }
    svg(svg::Handle::from_memory(desenho.into_bytes()))
        .width(Length::Fixed(tamanho))
        .height(Length::Fixed(tamanho))
}

/// Ícone Tabler para olho.
pub fn icone_olho(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("eye", tamanho, cor)
}

/// Ícone Tabler para agrupar.
pub fn icone_agrupar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("box-multiple", tamanho, cor)
}

/// Ícone Tabler para desagrupar.
pub fn icone_desagrupar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("box", tamanho, cor)
}

/// Ícone Tabler para minimizar.
pub fn icone_minimizar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("minus", tamanho, cor)
}

/// Ícone Tabler para maximizar.
pub fn icone_maximizar(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("square", tamanho, cor)
}

/// Ícone Tabler para fechar janela.
pub fn icone_fechar_janela(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("x", tamanho, cor)
}

/// Ícone Tabler para check.
pub fn icone_check(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("check", tamanho, cor)
}

/// Alinhamento vertical do parágrafo, com os mesmos ícones sem moldura de Organizar.
pub fn icone_texto_vertical(qual: usize, tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    let alinhamento = match qual {
        0 => 3,
        1 => 4,
        _ => 5,
    };
    icone_alinhar(alinhamento, tamanho, cor)
}

/// Ícone Tabler para menos.
pub fn icone_menos(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("minus", tamanho, cor)
}

/// Ícone Tabler para mais.
pub fn icone_mais(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("plus", tamanho, cor)
}

/// Ícone Tabler para seta cima.
pub fn icone_seta_cima(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("chevron-up", tamanho, cor)
}

/// Ícone Tabler para enquadramento.
pub fn icone_enquadramento(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("focus-centered", tamanho, cor)
}

/// Ícone Tabler para centralizar recorte.
pub fn icone_centralizar_recorte(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("restore", tamanho, cor)
}

/// Ícone Tabler para conta gotas.
pub fn icone_conta_gotas(tamanho: f32, cor: &'static str) -> svg::Svg<'static, iced::Theme> {
    tabler("color-picker", tamanho, cor)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn svg_preserva_todos_os_caminhos_originais_e_a_cor() {
        let icone = featherslide_core::icones::buscar_por_id("settings").unwrap();
        let xml = svg_tabler("settings", "#123456");
        assert!(xml.contains("viewBox=\"0 0 24 24\""));
        assert!(xml.contains("stroke=\"#123456\""));
        assert_eq!(xml.matches("<path ").count(), icone.caminhos_svg().count());
        for d in icone.caminhos_svg() {
            assert!(xml.contains(&format!("d=\"{d}\"")));
        }
    }

    #[test]
    fn todos_os_icones_da_interface_existem_no_tabler() {
        assert!(featherslide_core::icones::buscar_por_id("text-increase").is_some(), "text-increase");
        assert!(featherslide_core::icones::buscar_por_id("text-decrease").is_some(), "text-decrease");
        assert!(featherslide_core::icones::buscar_por_id("chevron-right").is_some(), "chevron-right");
        assert!(featherslide_core::icones::buscar_por_id("layout-align-left").is_some(), "layout-align-left");
        assert!(featherslide_core::icones::buscar_por_id("layout-align-right").is_some(), "layout-align-right");
        assert!(featherslide_core::icones::buscar_por_id("layout-align-top").is_some(), "layout-align-top");
        assert!(featherslide_core::icones::buscar_por_id("layout-align-bottom").is_some(), "layout-align-bottom");
        assert!(featherslide_core::icones::buscar_por_id("layout-align-center").is_some(), "layout-align-center");
        assert!(featherslide_core::icones::buscar_por_id("layout-align-middle").is_some(), "layout-align-middle");
        assert!(featherslide_core::icones::buscar_por_id("align-box-bottom-center").is_some(), "align-box-bottom-center");
        assert!(featherslide_core::icones::buscar_por_id("align-box-center-middle").is_some(), "align-box-center-middle");
        assert!(featherslide_core::icones::buscar_por_id("align-box-left-middle").is_some(), "align-box-left-middle");
        assert!(featherslide_core::icones::buscar_por_id("align-box-right-middle").is_some(), "align-box-right-middle");
        assert!(featherslide_core::icones::buscar_por_id("align-box-top-center").is_some(), "align-box-top-center");
        assert!(featherslide_core::icones::buscar_por_id("align-center").is_some(), "align-center");
        assert!(featherslide_core::icones::buscar_por_id("align-justified").is_some(), "align-justified");
        assert!(featherslide_core::icones::buscar_por_id("align-left").is_some(), "align-left");
        assert!(featherslide_core::icones::buscar_por_id("align-right").is_some(), "align-right");
        assert!(featherslide_core::icones::buscar_por_id("angle").is_some(), "angle");
        assert!(featherslide_core::icones::buscar_por_id("arrow-back-up").is_some(), "arrow-back-up");
        assert!(featherslide_core::icones::buscar_por_id("arrow-down").is_some(), "arrow-down");
        assert!(featherslide_core::icones::buscar_por_id("arrow-down-left").is_some(), "arrow-down-left");
        assert!(featherslide_core::icones::buscar_por_id("arrow-down-right").is_some(), "arrow-down-right");
        assert!(featherslide_core::icones::buscar_por_id("arrow-forward-up").is_some(), "arrow-forward-up");
        assert!(featherslide_core::icones::buscar_por_id("arrow-up").is_some(), "arrow-up");
        assert!(featherslide_core::icones::buscar_por_id("arrow-up-left").is_some(), "arrow-up-left");
        assert!(featherslide_core::icones::buscar_por_id("arrow-up-right").is_some(), "arrow-up-right");
        assert!(featherslide_core::icones::buscar_por_id("background").is_some(), "background");
        assert!(featherslide_core::icones::buscar_por_id("bold").is_some(), "bold");
        assert!(featherslide_core::icones::buscar_por_id("border-corner-rounded").is_some(), "border-corner-rounded");
        assert!(featherslide_core::icones::buscar_por_id("box").is_some(), "box");
        assert!(featherslide_core::icones::buscar_por_id("box-multiple").is_some(), "box-multiple");
        assert!(featherslide_core::icones::buscar_por_id("calendar").is_some(), "calendar");
        assert!(featherslide_core::icones::buscar_por_id("chart-bar").is_some(), "chart-bar");
        assert!(featherslide_core::icones::buscar_por_id("check").is_some(), "check");
        assert!(featherslide_core::icones::buscar_por_id("chevron-down").is_some(), "chevron-down");
        assert!(featherslide_core::icones::buscar_por_id("chevron-left").is_some(), "chevron-left");
        assert!(featherslide_core::icones::buscar_por_id("chevron-right").is_some(), "chevron-right");
        assert!(featherslide_core::icones::buscar_por_id("chevron-up").is_some(), "chevron-up");
        assert!(featherslide_core::icones::buscar_por_id("clipboard").is_some(), "clipboard");
        assert!(featherslide_core::icones::buscar_por_id("clock").is_some(), "clock");
        assert!(featherslide_core::icones::buscar_por_id("clock-play").is_some(), "clock-play");
        assert!(featherslide_core::icones::buscar_por_id("color-picker").is_some(), "color-picker");
        assert!(featherslide_core::icones::buscar_por_id("column-insert-right").is_some(), "column-insert-right");
        assert!(featherslide_core::icones::buscar_por_id("column-remove").is_some(), "column-remove");
        assert!(featherslide_core::icones::buscar_por_id("copy").is_some(), "copy");
        assert!(featherslide_core::icones::buscar_por_id("copy-plus").is_some(), "copy-plus");
        assert!(featherslide_core::icones::buscar_por_id("crop").is_some(), "crop");
        assert!(featherslide_core::icones::buscar_por_id("crosshair").is_some(), "crosshair");
        assert!(featherslide_core::icones::buscar_por_id("device-desktop").is_some(), "device-desktop");
        assert!(featherslide_core::icones::buscar_por_id("device-floppy").is_some(), "device-floppy");
        assert!(featherslide_core::icones::buscar_por_id("eraser").is_some(), "eraser");
        assert!(featherslide_core::icones::buscar_por_id("eye").is_some(), "eye");
        assert!(featherslide_core::icones::buscar_por_id("eye-off").is_some(), "eye-off");
        assert!(featherslide_core::icones::buscar_por_id("file-type-pdf").is_some(), "file-type-pdf");
        assert!(featherslide_core::icones::buscar_por_id("focus-centered").is_some(), "focus-centered");
        assert!(featherslide_core::icones::buscar_por_id("folder").is_some(), "folder");
        assert!(featherslide_core::icones::buscar_por_id("geometry").is_some(), "geometry");
        assert!(featherslide_core::icones::buscar_por_id("grid-pattern").is_some(), "grid-pattern");
        assert!(featherslide_core::icones::buscar_por_id("hash").is_some(), "hash");
        assert!(featherslide_core::icones::buscar_por_id("help").is_some(), "help");
        assert!(featherslide_core::icones::buscar_por_id("home").is_some(), "home");
        assert!(featherslide_core::icones::buscar_por_id("hourglass").is_some(), "hourglass");
        assert!(featherslide_core::icones::buscar_por_id("indent-decrease").is_some(), "indent-decrease");
        assert!(featherslide_core::icones::buscar_por_id("indent-increase").is_some(), "indent-increase");
        assert!(featherslide_core::icones::buscar_por_id("italic").is_some(), "italic");
        assert!(featherslide_core::icones::buscar_por_id("layers-intersect").is_some(), "layers-intersect");
        assert!(featherslide_core::icones::buscar_por_id("layout").is_some(), "layout");
        assert!(featherslide_core::icones::buscar_por_id("layout-grid").is_some(), "layout-grid");
        assert!(featherslide_core::icones::buscar_por_id("layout-sidebar").is_some(), "layout-sidebar");
        assert!(featherslide_core::icones::buscar_por_id("layout-sidebar-right").is_some(), "layout-sidebar-right");
        assert!(featherslide_core::icones::buscar_por_id("link").is_some(), "link");
        assert!(featherslide_core::icones::buscar_por_id("list").is_some(), "list");
        assert!(featherslide_core::icones::buscar_por_id("list-numbers").is_some(), "list-numbers");
        assert!(featherslide_core::icones::buscar_por_id("math-pi").is_some(), "math-pi");
        assert!(featherslide_core::icones::buscar_por_id("maximize").is_some(), "maximize");
        assert!(featherslide_core::icones::buscar_por_id("message").is_some(), "message");
        assert!(featherslide_core::icones::buscar_por_id("minus").is_some(), "minus");
        assert!(featherslide_core::icones::buscar_por_id("mood-smile").is_some(), "mood-smile");
        assert!(featherslide_core::icones::buscar_por_id("notes").is_some(), "notes");
        assert!(featherslide_core::icones::buscar_por_id("omega").is_some(), "omega");
        assert!(featherslide_core::icones::buscar_por_id("palette").is_some(), "palette");
        assert!(featherslide_core::icones::buscar_por_id("pencil").is_some(), "pencil");
        assert!(featherslide_core::icones::buscar_por_id("photo").is_some(), "photo");
        assert!(featherslide_core::icones::buscar_por_id("player-play").is_some(), "player-play");
        assert!(featherslide_core::icones::buscar_por_id("player-record").is_some(), "player-record");
        assert!(featherslide_core::icones::buscar_por_id("plus").is_some(), "plus");
        assert!(featherslide_core::icones::buscar_por_id("pointer").is_some(), "pointer");
        assert!(featherslide_core::icones::buscar_por_id("presentation").is_some(), "presentation");
        assert!(featherslide_core::icones::buscar_por_id("printer").is_some(), "printer");
        assert!(featherslide_core::icones::buscar_por_id("repeat").is_some(), "repeat");
        assert!(featherslide_core::icones::buscar_por_id("replace").is_some(), "replace");
        assert!(featherslide_core::icones::buscar_por_id("restore").is_some(), "restore");
        assert!(featherslide_core::icones::buscar_por_id("rotate-clockwise").is_some(), "rotate-clockwise");
        assert!(featherslide_core::icones::buscar_por_id("row-insert-bottom").is_some(), "row-insert-bottom");
        assert!(featherslide_core::icones::buscar_por_id("row-remove").is_some(), "row-remove");
        assert!(featherslide_core::icones::buscar_por_id("ruler").is_some(), "ruler");
        assert!(featherslide_core::icones::buscar_por_id("scissors").is_some(), "scissors");
        assert!(featherslide_core::icones::buscar_por_id("search").is_some(), "search");
        assert!(featherslide_core::icones::buscar_por_id("settings").is_some(), "settings");
        assert!(featherslide_core::icones::buscar_por_id("spacing-horizontal").is_some(), "spacing-horizontal");
        assert!(featherslide_core::icones::buscar_por_id("spacing-vertical").is_some(), "spacing-vertical");
        assert!(featherslide_core::icones::buscar_por_id("sparkles").is_some(), "sparkles");
        assert!(featherslide_core::icones::buscar_por_id("square").is_some(), "square");
        assert!(featherslide_core::icones::buscar_por_id("strikethrough").is_some(), "strikethrough");
        assert!(featherslide_core::icones::buscar_por_id("table").is_some(), "table");
        assert!(featherslide_core::icones::buscar_por_id("text-size").is_some(), "text-size");
        assert!(featherslide_core::icones::buscar_por_id("text-spellcheck").is_some(), "text-spellcheck");
        assert!(featherslide_core::icones::buscar_por_id("transition-right").is_some(), "transition-right");
        assert!(featherslide_core::icones::buscar_por_id("trash").is_some(), "trash");
        assert!(featherslide_core::icones::buscar_por_id("typography").is_some(), "typography");
        assert!(featherslide_core::icones::buscar_por_id("underline").is_some(), "underline");
        assert!(featherslide_core::icones::buscar_por_id("video").is_some(), "video");
        assert!(featherslide_core::icones::buscar_por_id("volume").is_some(), "volume");
        assert!(featherslide_core::icones::buscar_por_id("x").is_some(), "x");
        assert!(featherslide_core::icones::buscar_por_id("zoom-in").is_some(), "zoom-in");
        assert!(featherslide_core::icones::buscar_por_id("zoom-out").is_some(), "zoom-out");
    }
}

/// Índices com traços vetoriais: independe de glifos Unicode da fonte do sistema.
pub fn icone_indice_texto(tamanho: f32, superior: bool) -> svg::Svg<'static, iced::Theme> {
    let y = if superior { 2 } else { 12 };
    let data=format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#5f6368\" stroke-width=\"1.7\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M4 8l8 12M12 8L4 20\"/><g transform=\"translate(15 {y})\"><path d=\"M0 2c0-3 6-3 6 0 0 2-6 3-6 6h6\"/></g></svg>");
    svg::Svg::new(svg::Handle::from_memory(data.into_bytes()))
        .width(tamanho)
        .height(tamanho)
}
