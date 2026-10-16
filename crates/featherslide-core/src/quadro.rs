//! O quadro de um slide: tudo o que a interface precisa para desenhá-lo, já resolvido —
//! as linhas de texto quebradas e posicionadas, as cores, a dica da caixa vazia. A
//! interface não decide nada: pinta (RECEITA.md §3, o "quadro").

use serde::Serialize;

use crate::dados::{self, Primitiva};
use crate::medida;
use crate::modelo::{Alinhamento, Elemento, Papel, Slide, Sombra, Tipo, Vertical};

/// Margem interna da caixa de texto, em pontos (a do PowerPoint: 0,1" dos lados, 0,05" em cima e embaixo).
pub const MARGEM_X: f32 = 7.2;
pub const MARGEM_Y: f32 = 3.6;
/// Altura da linha, em múltiplos do tamanho da letra.
pub const ENTRELINHA: f32 = 1.2;
/// Da linha de cima do texto até a linha de base, em múltiplos do tamanho (o ascendente da Liberation Sans).
const ASCENDENTE: f32 = 0.905;

/// As dicas das caixas vazias do layout, na língua da interface (a interface manda).
#[derive(Clone, Debug, Default)]
pub struct Dicas {
    pub titulo: String,
    pub subtitulo: String,
    pub corpo: String,
}

/// Uma linha de texto pronta: o texto e onde fica a linha de base, em pontos do slide.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct LinhaDeTexto {
    pub texto: String,
    pub x: f32,
    pub y: f32,
    pub largura: f32,
    pub glifos: Vec<crate::texto_formato::Glifo>,
}

/// Um elemento pronto para desenhar.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ElementoDesenhado {
    pub id: u64,
    pub tipo: &'static str,
    pub x: f32,
    pub y: f32,
    pub largura: f32,
    pub altura: f32,
    pub rotacao: f32,
    pub preenchimento: String,
    pub gradiente: Option<crate::estilo_forma::Gradiente>,
    pub contorno: String,
    pub espessura: f32,
    #[serde(skip_serializing_if = "crate::modelo::Imagem::esta_vazia")]
    pub imagem: crate::modelo::Imagem,
    /// Correções visuais para renderizadores e exportadores.
    pub visual_imagem: crate::imagem::VisualImagem,
    pub texto_alternativo: String,
    /// Os ajustes da forma (ver `Elemento::ajustes`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ajustes: Vec<f32>,
    /// A forma que recorta a imagem (`retangulo` = sem recorte).
    pub recorte: &'static str,
    /// Opacidade da imagem (1 = opaca) e tolerância do "remover fundo" (0 = desligado).
    pub opacidade: f32,
    pub fundo_removido: f32,
    /// Cor do fundo a remover (vazia = a dos cantos).
    pub fundo_cor: String,
    /// Suavidade da borda do remover fundo (0 duro … 1 suave).
    pub fundo_suave: f32,
    /// 0 = cor para alfa, 1 = só o fundo ligado à borda.
    pub fundo_modo: u8,
    /// Refinar a borda com filtro guiado.
    pub fundo_refino: bool,
    /// Marcas de manter/remover do watershed (ver `Elemento::fundo_marcas`).
    pub fundo_marcas: String,
    /// Ajuste (px, + expande, − contrai) e suavização (px) da borda do fundo removido.
    pub fundo_borda: f32,
    pub fundo_pena: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sombra: Option<Sombra>,
    pub linhas: Vec<LinhaDeTexto>,
    /// As linhas são a dica da caixa vazia (desenhada em cinza, só no editor).
    pub dica: bool,
    /// Transformação do texto artístico (vazia nos demais elementos).
    pub wordart: String,
    pub fonte: String,
    pub tamanho: f32,
    pub cor: String,
    /// Cor de fundo atrás do texto (vazia = sem).
    pub realce: String,
    pub negrito: bool,
    pub italico: bool,
    pub sublinhado: bool,
    pub tachado: bool,
    pub sublinhado_duplo: bool,
    pub tachado_duplo: bool,
    /// O desenho de uma tabela ou gráfico (vazio nos outros tipos); em pontos do slide.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub primitivas: Vec<Primitiva>,
    /// O retângulo (x, y, largura, altura) para o qual as primitivas foram calculadas: a
    /// interface usa para esticar o desenho enquanto se puxa uma alça.
    pub base: [f32; 4],
}

/// O slide pronto para desenhar.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Quadro {
    pub versao: u64,
    pub largura: f32,
    pub altura: f32,
    pub fundo: String,
    pub elementos: Vec<ElementoDesenhado>,
}

/// O texto a mostrar: o do elemento ou, vazio e com papel de layout, a dica (só no editor).
fn texto_a_mostrar<'a>(e: &'a Elemento, dicas: &'a Dicas, editor: bool) -> (&'a str, bool) {
    if !e.texto.is_empty() || !editor {
        return (&e.texto, false);
    }
    let dica = match e.papel {
        Papel::Titulo => &dicas.titulo,
        Papel::Subtitulo => &dicas.subtitulo,
        Papel::Corpo => &dicas.corpo,
        Papel::Livre | Papel::Fundo => return ("", false),
    };
    (dica, true)
}

/// As linhas do texto da caixa, quebradas na largura dela e alinhadas.
fn linhas_de(e: &Elemento, texto: &str) -> Vec<LinhaDeTexto> {
    if texto.is_empty() || !e.tipo.aceita_texto() {
        return Vec::new();
    }
    let est = &e.estilo;
    let entrelinha = if est.entrelinha > 0.0 { est.entrelinha } else { ENTRELINHA };
    let util = (e.largura - 2.0 * MARGEM_X).max(est.tamanho);
    let duas_colunas = est.colunas == 2;
    let larg_col = if duas_colunas { ((util - MARGEM_X) / 2.0).max(est.tamanho) } else { util };
    let personalizado = !e.indices_texto.is_empty() || e.espacamento_letras != 0.0;
    let quebradas: Vec<String> = if personalizado { quebrar_personalizado(e,texto,larg_col) } else { texto.split('\n').flat_map(|p| medida::quebrar(p, est.tamanho, est.negrito, larg_col)).collect() };
    let total_linhas = quebradas.len();
    let corte = if duas_colunas { total_linhas.div_ceil(2) } else { total_linhas };
    let passo = est.tamanho * entrelinha;
    let altura_texto = passo * corte as f32;
    let topo = match est.vertical {
        Vertical::Topo => e.y + MARGEM_Y,
        Vertical::Meio => e.y + (e.altura - altura_texto) / 2.0,
        Vertical::Base => e.y + e.altura - MARGEM_Y - altura_texto,
    };
    let mut byte = 0;
    quebradas
        .into_iter()
        .enumerate()
        .map(|(i, texto)| {
            let (col_idx, linha_idx) = if duas_colunas && i >= corte {
                (1, i - corte)
            } else {
                (0, i)
            };
            let offset_x = if col_idx == 1 { larg_col + MARGEM_X } else { 0.0 };
            let base_x = e.x + MARGEM_X + offset_x;
            let inicio = e.texto.get(byte..).and_then(|s|s.find(&texto)).map_or(byte,|p|byte+p);
            byte = inicio + texto.len();
            let largura = if personalizado { texto.char_indices().map(|(i,c)|crate::texto_formato::avancar(e,inicio+i,c)).sum::<f32>() - if texto.is_empty(){0.0}else{e.espacamento_letras} } else { medida::largura(&texto, est.tamanho, est.negrito) };
            let x = match est.alinhamento {
                Alinhamento::Esquerda | Alinhamento::Justificado => base_x,
                Alinhamento::Centro => base_x + (larg_col - largura) / 2.0,
                Alinhamento::Direita => base_x + larg_col - largura,
            };
            let y = topo + passo * linha_idx as f32 + (passo - est.tamanho) / 2.0 + est.tamanho * ASCENDENTE;
            let mut glifos = Vec::new();
            if personalizado {
                let mut gx=x;
                for (i,c) in texto.char_indices() {
                    let nivel=crate::texto_formato::nivel(e,inicio+i);
                    let tamanho=est.tamanho*if nivel==0{1.0}else{0.65};
                    let gy=y+if nivel>0{-est.tamanho*0.35}else if nivel<0{est.tamanho*0.2}else{0.0};
                    glifos.push(crate::texto_formato::Glifo{texto:c.to_string(),x:gx,y:gy,tamanho});
                    gx+=crate::texto_formato::avancar(e,inicio+i,c);
                }
            }
            LinhaDeTexto { texto, x, y, largura, glifos }
        })
        .collect()
}

/// Um elemento pronto para desenhar.
fn desenhar(e: &Elemento, dicas: &Dicas, editor: bool) -> ElementoDesenhado {
    let (texto, dica) = texto_a_mostrar(e, dicas, editor);
    ElementoDesenhado {
        id: e.id,
        tipo: e.tipo.nome(),
        x: e.x,
        y: e.y,
        largura: e.largura,
        altura: e.altura,
        rotacao: e.rotacao,
        // A cor do ícone vai dentro das primitivas; a caixa em si não leva fundo.
        preenchimento: if matches!(e.tipo, Tipo::Icone | Tipo::Caminho) || (crate::novas_formas::suporta(e.tipo) && e.gradiente.is_none() && e.opacidade==1.0 && e.sombra.cor.is_empty()) { String::new() } else { e.preenchimento.clone() },
        gradiente: e.gradiente.clone(),
        contorno: if matches!(e.tipo, Tipo::Icone | Tipo::Caminho) || (crate::novas_formas::suporta(e.tipo) && e.gradiente.is_none() && e.opacidade==1.0 && e.sombra.cor.is_empty()) { String::new() } else { e.contorno.clone() },
        espessura: e.espessura,
        imagem: e.imagem.clone(),
        visual_imagem: e.visual_imagem.clone(),
        texto_alternativo: e.texto_alternativo.clone(),
        ajustes: e.ajustes.clone(),
        recorte: e.recorte.nome(),
        opacidade: e.opacidade,
        fundo_removido: e.fundo_removido,
        fundo_cor: e.fundo_cor.clone(),
        fundo_suave: e.fundo_suave,
        fundo_modo: e.fundo_modo,
        fundo_refino: e.fundo_refino,
        fundo_marcas: e.fundo_marcas.clone(),
        fundo_borda: e.fundo_borda,
        fundo_pena: e.fundo_pena,
        sombra: (!e.sombra.cor.is_empty()).then(|| e.sombra.clone()),
        linhas: linhas_de(e, texto),
        dica,
        wordart: if e.tipo == Tipo::WordArt { e.dados.clone() } else { String::new() },
        fonte: e.estilo.fonte.clone(),
        tamanho: e.estilo.tamanho,
        cor: if dica { "#80868b".into() } else { e.estilo.cor.clone() },
        realce: if dica { String::new() } else { e.estilo.realce.clone() },
        negrito: e.estilo.negrito,
        italico: e.estilo.italico,
        sublinhado: e.estilo.sublinhado && !dica,
        tachado: e.estilo.tachado && !dica,
        sublinhado_duplo: e.estilo.sublinhado_duplo,
        tachado_duplo: e.estilo.tachado_duplo,
        primitivas: match e.tipo {
            Tipo::Tabela => dados::primitivas_tabela(e),
            Tipo::Grafico => dados::primitivas_grafico(e),
            Tipo::Icone => crate::icones::primitivas(e),
            Tipo::Caminho => crate::booleanas::primitivas(e),
            tipo if crate::novas_formas::suporta(tipo) => crate::novas_formas::primitivas(e),
            _ => Vec::new(),
        },
        base: [e.x, e.y, e.largura, e.altura],
    }
}

/// O quadro do slide. `editor` mostra as dicas das caixas vazias (a apresentação não mostra).
pub fn quadro(slide: &Slide, tamanho: (f32, f32), versao: u64, dicas: &Dicas, editor: bool) -> Quadro {
    let fundo = &slide.fundo_config;
    let cor = if slide.fundo.is_empty() { "#ffffff" } else { &slide.fundo };
    let cor_fundo = if fundo.tipo != "solido" { "#ffffff".to_string() } else if fundo.opacidade < 1.0 { crate::configuracao_slide::cor_alpha(cor, fundo.opacidade) } else { cor.to_string() };
    let mut elementos: Vec<_> = crate::configuracao_slide::elementos_fundo(slide, tamanho).iter().map(|e| desenhar(e, dicas, false)).collect();
    elementos.extend(slide.elementos.iter().filter(|e| !e.oculto && !(slide.ocultar_mestre && e.papel == Papel::Fundo)).map(|e| desenhar(e, dicas, editor)));
    Quadro {
        versao,
        largura: tamanho.0,
        altura: tamanho.1,
        fundo: cor_fundo,
        elementos,
    }
}

fn quebrar_personalizado(e: &Elemento, texto: &str, maximo: f32) -> Vec<String> {
    let mut linhas = Vec::new();
    let mut inicio = 0;
    let mut largura = 0.0;
    let mut espaco = None;
    for (i, c) in texto.char_indices() {
        if c == '\n' {
            linhas.push(texto[inicio..i].to_string());
            inicio = i + 1;
            largura = 0.0;
            espaco = None;
            continue;
        }
        if c.is_whitespace() {
            espaco = Some(i);
        }
        let av = crate::texto_formato::avancar(e, i, c);
        if largura + av > maximo && i > inicio {
            let corte = espaco.filter(|p| *p > inicio && *p < i).unwrap_or(i);
            linhas.push(texto[inicio..corte].to_string());
            inicio = corte;
            while inicio < i && texto[inicio..].starts_with(' ') {
                inicio += 1;
            }
            largura = texto[inicio..i]
                .char_indices()
                .map(|(j, c)| crate::texto_formato::avancar(e, inicio + j, c))
                .sum();
            espaco = None;
        }
        largura += av;
    }
    linhas.push(texto[inicio..].to_string());
    linhas
}
