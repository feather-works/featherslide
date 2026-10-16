//! O `.fsdz`: contêiner ZIP (contrato geral em `FORMATO.md`) com a apresentação em
//! **Arrow** colunar, como o `.fdoz` e o `.fshz` — nada de JSON.
//!
//! ```text
//! data/apresentacao.arrow  uma linha: título, autor, proporção, próximo id
//!                          (aplicativo e versão do formato nos metadados do esquema)
//! data/slides.arrow        um slide por linha, na ordem: id, layout, fundo, notas, oculto
//! data/elementos.arrow     um elemento por linha: o slide dele, a geometria, as cores, o texto e o estilo
//! assets/                  as imagens, como arquivo (não como texto base64)
//! ```
//!
//! O `.git/` embutido do contrato ainda não entrou (o FeatherDocs já tem; ver `CLAUDE.md`).

use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, BooleanArray, Float32Array, RecordBatch, StringArray, UInt64Array, UInt8Array};
use arrow::datatypes::{Field, Schema};
use arrow::ipc::reader::FileReader;
use arrow::ipc::writer::FileWriter;
use featherslide_core::modelo::{Alinhamento, Animacao, Apresentacao, Comentario, Direcao, Elemento, InicioAnimacao, TipoAnimacao, TipoTransicao, Transicao, EstiloDoTexto, FonteEmbutida, Papel, Proporcao, Slide, Sombra, Tipo, Vertical};
use zip::write::SimpleFileOptions;

use crate::base64;

/// A versão do formato. 3 (07/10/2026): fontes embutidas em `assets/fontes/` e `data/fontes.arrow` —
/// formatos 1 e 2 continuam abrindo normalmente. 4 (07/10/2026): coluna `dados` (tabela e gráfico de verdade,
/// em texto compacto) — os formatos anteriores abrem com a coluna ausente. Colunas `link` (elemento) e
/// `comentarios` (slide) entraram sem mudar a versão: arquivos antigos as leem como vazias.
// Versão 5: novo código estável Tipo::Caminho, contornos normalizados em `dados`.
// Versão 6: códigos de formas básicas adicionais (FTS-0285, 0298–0312).
// Versão 7: metadados de documento completos (colunas adicionais em apresentacao.arrow).
// Versão 8: efeitos de imagem não destrutivos e texto alternativo.
// Versão 9: sequências personalizadas em data/personalizadas.arrow (arquivo Arrow, sem JSON).
// Versão 10: dimensões personalizadas e aparência completa do fundo do slide.
// Versão 11: galerias vetoriais, traços livres e metadados de conectores vinculados.
// Versão 12: 12 novos tipos de animação de entrada, códigos 7..18, preservando 0..6.
const VERSAO: u32 = 12;
const APLICATIVO: &str = "FeatherSlide";
const PROPORCOES: [Proporcao; 2] = [Proporcao::Larga, Proporcao::Padrao];

fn erro(e: &dyn std::fmt::Display) -> String {
    format!("Não consegui gravar o .fsdz: {e}")
}

fn codigo_de<T: PartialEq>(tabela: &[T], v: &T) -> u8 {
    tabela.iter().position(|x| x == v).unwrap_or(0) as u8
}

fn valor_de<T: Copy>(tabela: &[T], c: u8) -> T {
    tabela.get(usize::from(c)).copied().unwrap_or(tabela[0])
}

// ---------------------------------------------------------------------------
// Tabelas Arrow
// ---------------------------------------------------------------------------

/// Uma tabela Arrow (IPC, formato de arquivo) com as colunas dadas.
fn tabela(colunas: Vec<(&str, ArrayRef)>, metadados: HashMap<String, String>) -> Result<Vec<u8>, String> {
    let campos: Vec<Field> = colunas.iter().map(|(n, a)| Field::new(*n, a.data_type().clone(), true)).collect();
    let esquema = Arc::new(Schema::new_with_metadata(campos, metadados));
    let lote = RecordBatch::try_new(esquema.clone(), colunas.into_iter().map(|(_, a)| a).collect()).map_err(|e| erro(&e))?;
    let mut bytes = Vec::new();
    let mut w = FileWriter::try_new(&mut bytes, &esquema).map_err(|e| erro(&e))?;
    w.write(&lote).map_err(|e| erro(&e))?;
    w.finish().map_err(|e| erro(&e))?;
    drop(w);
    Ok(bytes)
}

/// Uma tabela lida: as colunas pelo nome e os metadados do esquema.
struct Lida {
    lote: RecordBatch,
    nome: &'static str,
}

impl Lida {
    fn de(bytes: &[u8], nome: &'static str) -> Result<Self, String> {
        let leitor = FileReader::try_new(Cursor::new(bytes), None).map_err(|e| format!("{nome} ilegível: {e}"))?;
        let esquema = leitor.schema();
        let lotes: Vec<RecordBatch> = leitor.collect::<Result<_, _>>().map_err(|e| format!("{nome} ilegível: {e}"))?;
        let lote = arrow::compute::concat_batches(&esquema, &lotes).map_err(|e| format!("{nome} ilegível: {e}"))?;
        Ok(Self { lote, nome })
    }

    fn linhas(&self) -> usize {
        self.lote.num_rows()
    }

    fn meta(&self, chave: &str) -> Option<&str> {
        self.lote.schema_ref().metadata().get(chave).map(String::as_str)
    }

    fn col<T: 'static>(&self, campo: &str) -> Result<&T, String> {
        self.lote.column_by_name(campo).and_then(|c| c.as_any().downcast_ref::<T>()).ok_or_else(|| format!("{}: falta a coluna \"{campo}\".", self.nome))
    }

    /// Coluna que entrou num formato mais novo: ausente, vale `padrao`.
    fn ou<T>(&self, campo: &str, padrao: T, ler: impl FnOnce(&Self) -> Result<T, String>) -> Result<T, String> {
        if self.lote.column_by_name(campo).is_some() {
            ler(self)
        } else {
            Ok(padrao)
        }
    }

    fn texto(&self, campo: &str, i: usize) -> Result<String, String> {
        let c = self.col::<StringArray>(campo)?;
        Ok(if c.is_null(i) { String::new() } else { c.value(i).to_string() })
    }

    fn f32(&self, campo: &str, i: usize) -> Result<f32, String> {
        Ok(self.col::<Float32Array>(campo)?.value(i))
    }

    fn u8(&self, campo: &str, i: usize) -> Result<u8, String> {
        Ok(self.col::<UInt8Array>(campo)?.value(i))
    }

    fn u64(&self, campo: &str, i: usize) -> Result<u64, String> {
        Ok(self.col::<UInt64Array>(campo)?.value(i))
    }

    fn bool(&self, campo: &str, i: usize) -> Result<bool, String> {
        Ok(self.col::<BooleanArray>(campo)?.value(i))
    }
}

// ---------------------------------------------------------------------------
// Gravar
// ---------------------------------------------------------------------------

/// Sequências opcionais do usuário. A coluna "grupo" mantém a ordem das
/// apresentações, e cada linha mantém a ordem de slides dentro de um grupo.
fn personalizadas_arrow(
    personalizadas: &[featherslide_core::personalizadas::ApresentacaoPersonalizada]
) -> Result<Vec<u8>, String> {
    let mut grupos = Vec::new();
    let mut nomes = Vec::new();
    let mut ids = Vec::new();
    let mut ocultos = Vec::new();
    for (grupo, p) in personalizadas.iter().enumerate() {
        for slide in &p.slides {
            grupos.push(grupo as u64);
            nomes.push(p.nome.as_str());
            ids.push(slide.id);
            ocultos.push(slide.oculto);
        }
    }
    tabela(vec![
        ("grupo", Arc::new(UInt64Array::from(grupos)) as ArrayRef),
        ("nome", Arc::new(StringArray::from(nomes)) as ArrayRef),
        ("slide_id", Arc::new(UInt64Array::from(ids)) as ArrayRef),
        ("oculto", Arc::new(BooleanArray::from(ocultos)) as ArrayRef),
    ], HashMap::new())
}

fn ler_personalizadas(
    t: &Lida
) -> Result<Vec<featherslide_core::personalizadas::ApresentacaoPersonalizada>, String> {
    use featherslide_core::personalizadas::{ApresentacaoPersonalizada, SlidePersonalizado};
    let mut grupos: Vec<ApresentacaoPersonalizada> = Vec::new();
    for i in 0..t.linhas() {
        let grupo = t.u64("grupo", i)? as usize;
        if grupo == grupos.len() {
            grupos.push(ApresentacaoPersonalizada { nome: t.texto("nome", i)?, slides: Vec::new() });
        } else if grupo >= grupos.len() || grupos[grupo].nome != t.texto("nome", i)? {
            return Err("personalizadas.arrow: grupos ou nomes fora de ordem.".into());
        }
        grupos[grupo].slides.push(SlidePersonalizado {
            id: t.u64("slide_id", i)?, oculto: t.bool("oculto", i)?,
        });
        if grupos.len() > 50 || grupos[grupo].slides.len() > 1000 {
            return Err("personalizadas.arrow: limite de grupos ou slides excedido.".into());
        }
    }
    Ok(grupos)
}


/// Salva a apresentação em disco substituindo o arquivo atomicamente.
pub fn salvar(ap: &Apresentacao, destino: &std::path::Path) -> Result<usize, String> {
    let bytes = gravar(ap)?;
    crate::arquivo::gravar_atomico(destino, &bytes)?;
    Ok(bytes.len())
}

/// Serializa a apresentação num contêiner `.fsdz` em memória.
pub fn gravar(ap: &Apresentacao) -> Result<Vec<u8>, String> {
    let mut imagens = Vec::new();
    let mut partes: Vec<(String, Vec<u8>)> = vec![
        ("data/apresentacao.arrow".into(), apresentacao_arrow(ap)?),
        ("data/slides.arrow".into(), slides_arrow(&ap.slides, &mut imagens)?),
        ("data/elementos.arrow".into(), elementos_arrow(&ap.slides, &mut imagens)?),
        ("data/personalizadas.arrow".into(), personalizadas_arrow(&ap.metadados.apresentacoes_personalizadas)?),
    ];
    partes.extend(imagens);
    if !ap.fontes.is_empty() {
        let (tab_fontes, arquivos_fontes) = fontes_arrow(&ap.fontes)?;
        partes.push(("data/fontes.arrow".into(), tab_fontes));
        partes.extend(arquivos_fontes);
    }
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opcoes = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (nome, conteudo) in partes {
        zip.start_file(nome, opcoes).map_err(|e| erro(&e))?;
        zip.write_all(&conteudo).map_err(|e| erro(&e))?;
    }
    Ok(zip.finish().map_err(|e| erro(&e))?.into_inner())
}

fn apresentacao_arrow(ap: &Apresentacao) -> Result<Vec<u8>, String> {
    let meta = HashMap::from([("aplicativo".to_string(), APLICATIVO.to_string()), ("formato".to_string(), VERSAO.to_string())]);
    tabela(
        vec![
            ("titulo", Arc::new(StringArray::from(vec![ap.titulo.as_str()]))),
            ("autor", Arc::new(StringArray::from(vec![ap.autor.as_str()]))),
            ("proporcao", Arc::new(UInt8Array::from(vec![codigo_de(&PROPORCOES, &ap.proporcao)]))),
            ("largura_personalizada", Arc::new(Float32Array::from(vec![ap.dimensoes[0]]))),
            ("altura_personalizada", Arc::new(Float32Array::from(vec![ap.dimensoes[1]]))),
            ("proximo_id", Arc::new(UInt64Array::from(vec![ap.proximo_id]))),
            ("tema", Arc::new(StringArray::from(vec![ap.tema.as_str()]))),
            ("modelo", Arc::new(StringArray::from(vec![ap.modelo.as_str()]))),
            ("assunto", Arc::new(StringArray::from(vec![ap.metadados.assunto.as_str()]))),
            ("empresa", Arc::new(StringArray::from(vec![ap.metadados.empresa.as_str()]))),
            ("gerente", Arc::new(StringArray::from(vec![ap.metadados.gerente.as_str()]))),
            ("categoria", Arc::new(StringArray::from(vec![ap.metadados.categoria.as_str()]))),
            ("palavras_chave", Arc::new(StringArray::from(vec![ap.metadados.palavras_chave.as_str()]))),
            ("comentario_documento", Arc::new(StringArray::from(vec![ap.metadados.comentarios.as_str()]))),
            ("criado_em_utc", Arc::new(UInt64Array::from(vec![ap.metadados.criado_em_utc]))),
            ("modificado_em_utc", Arc::new(UInt64Array::from(vec![ap.metadados.modificado_em_utc]))),
            ("ultimo_autor", Arc::new(StringArray::from(vec![ap.metadados.ultimo_autor.as_str()]))),
            ("revisoes", Arc::new(UInt64Array::from(vec![ap.metadados.revisoes]))),
            ("tempo_edicao_segundos", Arc::new(UInt64Array::from(vec![ap.metadados.tempo_edicao_segundos]))),
            ("idioma", Arc::new(StringArray::from(vec![ap.metadados.idioma.as_str()]))),
            ("identificador", Arc::new(StringArray::from(vec![ap.metadados.identificador.as_str()]))),
            ("propriedades_personalizadas", Arc::new(StringArray::from(vec![mapa_gravar(&ap.metadados.propriedades_personalizadas)]))),
            ("metadados_estendidos", Arc::new(StringArray::from(vec![mapa_gravar(&ap.metadados.estendidos)]))),
        ],
        meta,
    )
}

/// Codificação de mapas sem JSON: pares com comprimentos em bytes, segura para Unicode
/// e caracteres de controle (inclusive separadores usados nos comentários de slides).
fn mapa_gravar(mapa: &std::collections::BTreeMap<String, String>) -> String {
    let mut resultado = String::new();
    for (chave, valor) in mapa {
        resultado.push_str(&format!("{}:{}{}:{}", chave.len(), chave, valor.len(), valor));
    }
    resultado
}

/// Lê um campo de comprimento prefixado, conferindo limites de byte e de UTF-8.
fn campo_com_tamanho(entrada: &str) -> Result<(String, &str), String> {
    let (numero, valor) = entrada.split_once(':').ok_or("Metadados sem separador de tamanho.")?;
    if numero.is_empty() || numero.len() > 8 || !numero.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Metadados têm tamanho inválido.".into());
    }
    let len: usize = numero.parse().map_err(|_| "Metadados têm tamanho inválido.")?;
    if len > 16_384 || len > valor.len() || !valor.is_char_boundary(len) {
        return Err("Metadados têm campo UTF-8 truncado.".into());
    }
    Ok((valor[..len].into(), &valor[len..]))
}

/// Descodifica pares UTF-8 com comprimentos. Arquivos truncados retornam erro, não panic.
fn mapa_ler(dados: &str) -> Result<std::collections::BTreeMap<String, String>, String> {
    let mut resto = dados;
    let mut mapa = std::collections::BTreeMap::new();
    while !resto.is_empty() {
        if mapa.len() >= 100 {
            return Err("Metadados inválidos: mais de 100 propriedades.".into());
        }
        let (chave, cauda) = campo_com_tamanho(resto)?;
        let (valor, cauda) = campo_com_tamanho(cauda)?;
        if chave.is_empty() || chave.len() > 128 { return Err("Chave de metadados inválida.".into()); }
        if mapa.insert(chave, valor).is_some() { return Err("Chave de metadados duplicada.".into()); }
        resto = cauda;
    }
    Ok(mapa)
}

/// Os comentários numa célula só: registros separados por U+001E, campos (autor, resolvido, texto) por U+001F.
fn comentarios_em_texto(c: &[Comentario]) -> String {
    c.iter().map(|c| format!("{}\u{1f}{}\u{1f}{}", c.autor, u8::from(c.resolvido), c.texto)).collect::<Vec<_>>().join("\u{1e}")
}

fn comentarios_do_texto(t: &str) -> Vec<Comentario> {
    t.split('\u{1e}')
        .filter(|r| !r.is_empty())
        .filter_map(|r| {
            let mut campos = r.splitn(3, '\u{1f}');
            Some(Comentario { autor: campos.next()?.into(), resolvido: campos.next()? == "1", texto: campos.next()?.into() })
        })
        .collect()
}

fn slides_arrow(slides: &[Slide], imagens: &mut Vec<(String, Vec<u8>)>) -> Result<Vec<u8>, String> {
    tabela(
        vec![
            ("id", Arc::new(UInt64Array::from(slides.iter().map(|s| s.id).collect::<Vec<_>>()))),
            ("layout", Arc::new(StringArray::from(slides.iter().map(|s| s.layout.as_str()).collect::<Vec<_>>()))),
            ("fundo", Arc::new(StringArray::from(slides.iter().map(|s| s.fundo.as_str()).collect::<Vec<_>>()))),
            ("fundo_tipo", Arc::new(StringArray::from(slides.iter().map(|s| s.fundo_config.tipo.as_str()).collect::<Vec<_>>()))),
            ("fundo_cor2", Arc::new(StringArray::from(slides.iter().map(|s| s.fundo_config.cor2.as_str()).collect::<Vec<_>>()))),
            ("fundo_imagem", Arc::new(StringArray::from(slides.iter().map(|s| guardar_imagem(&s.fundo_config.imagem, imagens)).collect::<Vec<_>>()))),
            ("fundo_opacidade", Arc::new(Float32Array::from(slides.iter().map(|s| s.fundo_config.opacidade).collect::<Vec<_>>()))),
            ("ocultar_mestre", Arc::new(BooleanArray::from(slides.iter().map(|s| s.ocultar_mestre).collect::<Vec<_>>()))),
            ("notas", Arc::new(StringArray::from(slides.iter().map(|s| s.notas.as_str()).collect::<Vec<_>>()))),
            ("oculto", Arc::new(BooleanArray::from(slides.iter().map(|s| s.oculto).collect::<Vec<_>>()))),
            ("secao", Arc::new(StringArray::from(slides.iter().map(|s| s.secao.as_str()).collect::<Vec<_>>()))),
            ("comentarios", Arc::new(StringArray::from(slides.iter().map(|s| comentarios_em_texto(&s.comentarios)).collect::<Vec<_>>()))),
            ("trans_tipo", Arc::new(UInt8Array::from(slides.iter().map(|s| codigo_de(&TipoTransicao::TODOS, &s.transicao.tipo)).collect::<Vec<_>>()))),
            ("trans_dir", Arc::new(UInt8Array::from(slides.iter().map(|s| codigo_de(&Direcao::TODOS, &s.transicao.direcao)).collect::<Vec<_>>()))),
            ("trans_duracao", Arc::new(Float32Array::from(slides.iter().map(|s| s.transicao.duracao).collect::<Vec<_>>()))),
            ("trans_apos", Arc::new(Float32Array::from(slides.iter().map(|s| s.transicao.apos).collect::<Vec<_>>()))),
        ],
        HashMap::new(),
    )
}

/// As colunas da tabela de elementos, uma `Vec` por coluna.
#[derive(Default)]
struct Colunas {
    slide: Vec<u64>,
    id: Vec<u64>,
    tipo: Vec<u8>,
    papel: Vec<u8>,
    geometria: [Vec<f32>; 5],
    preenchimento: Vec<String>,
    gradiente: Vec<String>,
    contorno: Vec<String>,
    espessura: Vec<f32>,
    texto: Vec<String>,
    indices_texto: Vec<String>,
    espacamento_letras: Vec<f32>,
    fonte: Vec<String>,
    tamanho: Vec<f32>,
    cor: Vec<String>,
    realce: Vec<String>,
    marcas: [Vec<bool>; 6],
    entrelinha: Vec<f32>,
    colunas_texto: Vec<u8>,
    alinhamento: Vec<u8>,
    vertical: Vec<u8>,
    imagem: Vec<String>,
    ajustes: Vec<String>,
    recorte: Vec<u8>,
    visual_imagem: Vec<String>,
    texto_alternativo: Vec<String>,
    sombra_cor: Vec<String>,
    sombra: [Vec<f32>; 3],
    dados: Vec<String>,
    anim_tipo: Vec<u8>,
    anim_inicio: Vec<u8>,
    anim_dir: Vec<u8>,
    anim: [Vec<f32>; 2],
    destino: Vec<u32>,
    link: Vec<String>,
    oculto: Vec<bool>,
    grupo: Vec<u32>,
    opacidade: Vec<f32>,
    fundo_removido: Vec<f32>,
    fundo_cor: Vec<String>,
    fundo_suave: Vec<f32>,
    fundo_modo: Vec<u8>,
    fundo_refino: Vec<bool>,
    fundo_marcas: Vec<String>,
    fundo_borda: Vec<f32>,
    fundo_pena: Vec<f32>,
}

impl Colunas {
    fn por(&mut self, slide: u64, e: &Elemento, imagens: &mut Vec<(String, Vec<u8>)>) {
        self.slide.push(slide);
        self.id.push(e.id);
        self.tipo.push(codigo_de(&Tipo::TODOS, &e.tipo));
        self.papel.push(codigo_de(&Papel::TODOS, &e.papel));
        for (col, v) in self.geometria.iter_mut().zip([e.x, e.y, e.largura, e.altura, e.rotacao]) {
            col.push(v);
        }
        self.preenchimento.push(e.preenchimento.clone());
        self.gradiente.push(e.gradiente.as_ref().map(|g| serde_json::to_string(g).expect("gradiente válido")).unwrap_or_default());
        self.contorno.push(e.contorno.clone());
        self.espessura.push(e.espessura);
        self.texto.push(e.texto.clone());
        self.indices_texto.push(serde_json::to_string(&e.indices_texto).expect("índices válidos"));
        self.espacamento_letras.push(e.espacamento_letras);
        let est = &e.estilo;
        self.fonte.push(est.fonte.clone());
        self.tamanho.push(est.tamanho);
        self.cor.push(est.cor.clone());
        self.realce.push(est.realce.clone());
        for (col, v) in self.marcas.iter_mut().zip([est.negrito, est.italico, est.sublinhado, est.tachado, est.sublinhado_duplo, est.tachado_duplo]) {
            col.push(v);
        }
        self.entrelinha.push(est.entrelinha);
        self.colunas_texto.push(est.colunas);
        self.alinhamento.push(codigo_de(&Alinhamento::TODOS, &est.alinhamento));
        self.vertical.push(codigo_de(&Vertical::TODOS, &est.vertical));
        self.imagem.push(guardar_imagem(&e.imagem, imagens));
        self.ajustes.push(e.ajustes.iter().map(f32::to_string).collect::<Vec<_>>().join(";"));
        self.recorte.push(codigo_de(&Tipo::TODOS, &e.recorte));
        self.visual_imagem.push(e.visual_imagem.codificar());
        self.texto_alternativo.push(e.texto_alternativo.clone());
        self.dados.push(e.dados.clone());
        self.destino.push(e.destino);
        self.link.push(e.link.clone());
        self.oculto.push(e.oculto);
        self.grupo.push(e.grupo);
        self.opacidade.push(e.opacidade);
        self.fundo_removido.push(e.fundo_removido);
        self.fundo_cor.push(e.fundo_cor.clone());
        self.fundo_suave.push(e.fundo_suave);
        self.fundo_modo.push(e.fundo_modo);
        self.fundo_refino.push(e.fundo_refino);
        self.fundo_marcas.push(e.fundo_marcas.clone());
        self.fundo_borda.push(e.fundo_borda);
        self.fundo_pena.push(e.fundo_pena);
        self.anim_tipo.push(codigo_de(&TipoAnimacao::TODOS, &e.animacao.tipo));
        self.anim_inicio.push(codigo_de(&InicioAnimacao::TODOS, &e.animacao.inicio));
        self.anim_dir.push(codigo_de(&Direcao::TODOS, &e.animacao.direcao));
        self.anim[0].push(e.animacao.duracao);
        self.anim[1].push(e.animacao.atraso);
        self.sombra_cor.push(e.sombra.cor.clone());
        for (col, v) in self.sombra.iter_mut().zip([e.sombra.desfoque, e.sombra.dx, e.sombra.dy]) {
            col.push(v);
        }
    }

    fn colunas(self) -> Vec<(&'static str, ArrayRef)> {
        let [x, y, largura, altura, rotacao] = self.geometria;
        let [negrito, italico, sublinhado, tachado, sublinhado_duplo, tachado_duplo] = self.marcas;
        let [desfoque, dx, dy] = self.sombra;
        let [anim_duracao, anim_atraso] = self.anim;
        let f = |v: Vec<f32>| -> ArrayRef { Arc::new(Float32Array::from(v)) };
        let s = |v: Vec<String>| -> ArrayRef { Arc::new(StringArray::from(v)) };
        let b = |v: Vec<bool>| -> ArrayRef { Arc::new(BooleanArray::from(v)) };
        vec![
            ("slide", Arc::new(UInt64Array::from(self.slide))),
            ("id", Arc::new(UInt64Array::from(self.id))),
            ("tipo", Arc::new(UInt8Array::from(self.tipo))),
            ("papel", Arc::new(UInt8Array::from(self.papel))),
            ("x", f(x)),
            ("y", f(y)),
            ("largura", f(largura)),
            ("altura", f(altura)),
            ("rotacao", f(rotacao)),
            ("preenchimento", s(self.preenchimento)),
            ("gradiente_forma", s(self.gradiente)),
            ("contorno", s(self.contorno)),
            ("espessura", f(self.espessura)),
            ("texto", s(self.texto)),
            ("indices_texto", s(self.indices_texto)),
            ("espacamento_letras", f(self.espacamento_letras)),
            ("fonte", s(self.fonte)),
            ("tamanho", f(self.tamanho)),
            ("cor", s(self.cor)),
            ("realce", s(self.realce)),
            ("negrito", b(negrito)),
            ("italico", b(italico)),
            ("sublinhado", b(sublinhado)),
            ("tachado", b(tachado)),
            ("sublinhado_duplo", b(sublinhado_duplo)),
            ("tachado_duplo", b(tachado_duplo)),
            ("entrelinha", f(self.entrelinha)),
            ("colunas", Arc::new(UInt8Array::from(self.colunas_texto))),
            ("alinhamento", Arc::new(UInt8Array::from(self.alinhamento))),
            ("vertical", Arc::new(UInt8Array::from(self.vertical))),
            ("imagem", s(self.imagem)),
            ("ajustes", s(self.ajustes)),
            ("recorte", Arc::new(UInt8Array::from(self.recorte))),
            ("visual_imagem", s(self.visual_imagem)),
            ("texto_alternativo", s(self.texto_alternativo)),
            ("sombra_cor", s(self.sombra_cor)),
            ("sombra_desfoque", f(desfoque)),
            ("sombra_dx", f(dx)),
            ("sombra_dy", f(dy)),
            ("dados", s(self.dados)),
            ("anim_tipo", Arc::new(UInt8Array::from(self.anim_tipo))),
            ("anim_inicio", Arc::new(UInt8Array::from(self.anim_inicio))),
            ("anim_dir", Arc::new(UInt8Array::from(self.anim_dir))),
            ("anim_duracao", f(anim_duracao)),
            ("anim_atraso", f(anim_atraso)),
            ("destino", Arc::new(arrow::array::UInt32Array::from(self.destino))),
            ("link", Arc::new(StringArray::from(self.link))),
            ("oculto", Arc::new(BooleanArray::from(self.oculto))),
            ("grupo", Arc::new(arrow::array::UInt32Array::from(self.grupo))),
            ("opacidade", f(self.opacidade)),
            ("fundo_removido", f(self.fundo_removido)),
            ("fundo_cor", s(self.fundo_cor)),
            ("fundo_suave", f(self.fundo_suave)),
            ("fundo_modo", Arc::new(UInt8Array::from(self.fundo_modo))),
            ("fundo_refino", Arc::new(BooleanArray::from(self.fundo_refino))),
            ("fundo_marcas", s(self.fundo_marcas)),
            ("fundo_borda", f(self.fundo_borda)),
            ("fundo_pena", f(self.fundo_pena)),
        ]
    }
}

/// A extensão de arquivo de um tipo de imagem.
fn extensao_da_imagem(tipo: &str) -> &'static str {
    match tipo {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "image/bmp" => "bmp",
        _ => "png",
    }
}

fn tipo_da_imagem(nome: &str) -> &'static str {
    match nome.rsplit('.').next().unwrap_or("") {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        _ => "image/png",
    }
}

/// A imagem embutida (`data:`) vai para `assets/` como arquivo; a coluna guarda o caminho.
fn guardar_imagem(origem: &str, imagens: &mut Vec<(String, Vec<u8>)>) -> String {
    match base64::de_data_url(origem) {
        Some((tipo, bytes)) => {
            let nome = format!("assets/imagem-{}.{}", imagens.len() + 1, extensao_da_imagem(&tipo));
            imagens.push((nome.clone(), bytes));
            nome
        }
        None => origem.to_string(),
    }
}

fn elementos_arrow(slides: &[Slide], imagens: &mut Vec<(String, Vec<u8>)>) -> Result<Vec<u8>, String> {
    let mut c = Colunas::default();
    for s in slides {
        for e in &s.elementos {
            c.por(s.id, e, imagens);
        }
    }
    tabela(c.colunas(), HashMap::new())
}

// ---------------------------------------------------------------------------
// Ler
// ---------------------------------------------------------------------------

/// Lê um `.fsdz`. Arquivo malformado dá erro em português, nunca pânico.
pub fn ler(bytes: &[u8]) -> Result<Apresentacao, String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| "Este arquivo não é um .fsdz (não é um ZIP).".to_string())?;
    let mut parte = |nome: &str| -> Result<Vec<u8>, String> {
        let mut f = zip.by_name(nome).map_err(|_| format!("O .fsdz está incompleto: falta {nome}."))?;
        let mut v = Vec::new();
        f.read_to_end(&mut v).map_err(|e| format!("Não consegui ler {nome}: {e}"))?;
        Ok(v)
    };
    let cabeca = Lida::de(&parte("data/apresentacao.arrow")?, "apresentacao.arrow")?;
    if cabeca.meta("aplicativo") != Some(APLICATIVO) {
        return Err("Este arquivo não foi gravado pelo FeatherSlide.".into());
    }
    let formato: u32 = cabeca.meta("formato").and_then(|f| f.parse().ok()).unwrap_or(0);
    if formato > VERSAO {
        return Err(format!("Este .fsdz é de uma versão mais nova do FeatherSlide (formato {formato})."));
    }
    if cabeca.linhas() != 1 {
        return Err("apresentacao.arrow: precisa ter exatamente uma linha.".into());
    }
    let slides = Lida::de(&parte("data/slides.arrow")?, "slides.arrow")?;
    let elementos = Lida::de(&parte("data/elementos.arrow")?, "elementos.arrow")?;
    let fontes = ler_fontes_fsdz(&mut parte)?;
    let mut ap = Apresentacao {
        titulo: cabeca.texto("titulo", 0)?,
        autor: cabeca.texto("autor", 0)?,
        proporcao: valor_de(&PROPORCOES, cabeca.u8("proporcao", 0)?),
        dimensoes: [cabeca.ou("largura_personalizada", 0.0, |t| t.f32("largura_personalizada", 0))?, cabeca.ou("altura_personalizada", 0.0, |t| t.f32("altura_personalizada", 0))?],
        slides: ler_slides(&slides)?,
        proximo_id: cabeca.u64("proximo_id", 0)?,
        fontes,
        tema: cabeca.ou("tema", String::new(), |t| t.texto("tema", 0))?,
        modelo: cabeca.ou("modelo", String::new(), |t| t.texto("modelo", 0))?,
        metadados: featherslide_core::metadados::Metadados {
            assunto: cabeca.ou("assunto", String::new(), |t| t.texto("assunto", 0))?,
            empresa: cabeca.ou("empresa", String::new(), |t| t.texto("empresa", 0))?,
            gerente: cabeca.ou("gerente", String::new(), |t| t.texto("gerente", 0))?,
            categoria: cabeca.ou("categoria", String::new(), |t| t.texto("categoria", 0))?,
            palavras_chave: cabeca.ou("palavras_chave", String::new(), |t| t.texto("palavras_chave", 0))?,
            comentarios: cabeca.ou("comentario_documento", String::new(), |t| t.texto("comentario_documento", 0))?,
            criado_em_utc: cabeca.ou("criado_em_utc", 0, |t| t.u64("criado_em_utc", 0))?,
            modificado_em_utc: cabeca.ou("modificado_em_utc", 0, |t| t.u64("modificado_em_utc", 0))?,
            ultimo_autor: cabeca.ou("ultimo_autor", String::new(), |t| t.texto("ultimo_autor", 0))?,
            revisoes: cabeca.ou("revisoes", 0, |t| t.u64("revisoes", 0))?,
            tempo_edicao_segundos: cabeca.ou("tempo_edicao_segundos", 0, |t| t.u64("tempo_edicao_segundos", 0))?,
            idioma: cabeca.ou("idioma", "pt-BR".to_string(), |t| t.texto("idioma", 0))?,
            identificador: cabeca.ou("identificador", String::new(), |t| t.texto("identificador", 0))?,
            propriedades_personalizadas: mapa_ler(&cabeca.ou("propriedades_personalizadas", String::new(), |t| t.texto("propriedades_personalizadas", 0))?)?,
            estendidos: mapa_ler(&cabeca.ou("metadados_estendidos", String::new(), |t| t.texto("metadados_estendidos", 0))?)?,
            apresentacoes_personalizadas: Vec::new(),
        },
    };
    if formato >= 9 {
        let dados = Lida::de(&parte("data/personalizadas.arrow")?, "personalizadas.arrow")?;
        ap.metadados.apresentacoes_personalizadas = ler_personalizadas(&dados)?;
    }
    if ap.dimensoes != [0.0; 2] { featherslide_core::configuracao_slide::validar_dimensoes(ap.dimensoes[0], ap.dimensoes[1])?; }
    for slide in &mut ap.slides {
        if slide.fundo_config.imagem.starts_with("assets/") {
            let bytes = parte(&slide.fundo_config.imagem)?;
            let tipo = tipo_da_imagem(&slide.fundo_config.imagem);
            slide.fundo_config.imagem = base64::para_data_url(tipo, &bytes).into();
        }
        if formato >= 10 {
            slide.fundo_config.validar(if slide.fundo.is_empty() { "#ffffff" } else { &slide.fundo })?;
        }
    }
    distribuir_elementos(&mut ap, &elementos, &mut parte)?;
    let maior = ap.slides.iter().flat_map(|s| std::iter::once(s.id).chain(s.elementos.iter().map(|e| e.id))).max().unwrap_or(0);
    ap.proximo_id = ap.proximo_id.max(maior + 1);
    Ok(ap)
}

fn ler_slides(t: &Lida) -> Result<Vec<Slide>, String> {
    (0..t.linhas())
        .map(|i| Ok(Slide { id: t.u64("id", i)?, layout: t.texto("layout", i)?, fundo: t.texto("fundo", i)?, elementos: Vec::new(), notas: t.texto("notas", i)?, oculto: t.bool("oculto", i)?,
                fundo_config: featherslide_core::configuracao_slide::Fundo {
                    tipo: t.ou("fundo_tipo", "solido".to_string(), |t| t.texto("fundo_tipo", i))?,
                    cor2: t.ou("fundo_cor2", "#ffffff".to_string(), |t| t.texto("fundo_cor2", i))?,
                    imagem: t.ou("fundo_imagem", String::new(), |t| t.texto("fundo_imagem", i))?.into(),
                    opacidade: t.ou("fundo_opacidade", 1.0, |t| t.f32("fundo_opacidade", i))?,
                },
                ocultar_mestre: t.ou("ocultar_mestre", false, |t| t.bool("ocultar_mestre", i))?,
                secao: t.ou("secao", String::new(), |t| t.texto("secao", i))?,
                comentarios: comentarios_do_texto(&t.ou("comentarios", String::new(), |t| t.texto("comentarios", i))?),
                transicao: Transicao {
                    tipo: valor_de(&TipoTransicao::TODOS, t.ou("trans_tipo", 0, |t| t.u8("trans_tipo", i))?),
                    direcao: valor_de(&Direcao::TODOS, t.ou("trans_dir", 1, |t| t.u8("trans_dir", i))?),
                    duracao: t.ou("trans_duracao", 0.7, |t| t.f32("trans_duracao", i))?,
                    apos: t.ou("trans_apos", 0.0, |t| t.f32("trans_apos", i))?,
                },
            }))
        .collect()
}

/// Uma linha da tabela de elementos. `imagem` volta como o caminho em `assets/` (ou o endereço guardado).
fn ler_elemento(t: &Lida, i: usize) -> Result<Elemento, String> {
    Ok(Elemento {
        id: t.u64("id", i)?,
        tipo: valor_de(&Tipo::TODOS, t.u8("tipo", i)?),
        papel: valor_de(&Papel::TODOS, t.u8("papel", i)?),
        x: t.f32("x", i)?,
        y: t.f32("y", i)?,
        largura: t.f32("largura", i)?,
        altura: t.f32("altura", i)?,
        rotacao: t.f32("rotacao", i)?,
        preenchimento: t.texto("preenchimento", i)?,
        gradiente: {
            let texto = t.ou("gradiente_forma", String::new(), |t| t.texto("gradiente_forma", i))?;
            if texto.is_empty() { None } else {
                let g: featherslide_core::estilo_forma::Gradiente = serde_json::from_str(&texto).map_err(|e| format!("Gradiente inválido: {e}"))?;
                g.validar()?; Some(g)
            }
        },
        contorno: t.texto("contorno", i)?,
        espessura: t.f32("espessura", i)?,
        texto: t.texto("texto", i)?,
        indices_texto: serde_json::from_str(&t.ou("indices_texto", "[]".to_string(), |t| t.texto("indices_texto", i))?).map_err(|e|format!("Índices inválidos: {e}"))?,
        espacamento_letras: t.ou("espacamento_letras", 0.0, |t|t.f32("espacamento_letras", i))?,
        estilo: EstiloDoTexto {
            fonte: t.texto("fonte", i)?,
            tamanho: t.f32("tamanho", i)?,
            cor: t.texto("cor", i)?,
            realce: t.ou("realce", String::new(), |t| t.texto("realce", i))?,
            negrito: t.bool("negrito", i)?,
            italico: t.bool("italico", i)?,
            sublinhado: t.bool("sublinhado", i)?,
            tachado: t.ou("tachado", false, |t| t.bool("tachado", i))?,
            sublinhado_duplo: t.ou("sublinhado_duplo", false, |t| t.bool("sublinhado_duplo", i))?,
            tachado_duplo: t.ou("tachado_duplo", false, |t| t.bool("tachado_duplo", i))?,
            entrelinha: t.ou("entrelinha", 1.2, |t| t.f32("entrelinha", i))?,
            colunas: t.ou("colunas", 1, |t| t.u8("colunas", i))?,
            alinhamento: valor_de(&Alinhamento::TODOS, t.u8("alinhamento", i)?),
            vertical: valor_de(&Vertical::TODOS, t.u8("vertical", i)?),
        },
        imagem: t.texto("imagem", i)?.into(),
        visual_imagem: featherslide_core::imagem::VisualImagem::decodificar(
            &t.ou("visual_imagem", String::new(), |t| t.texto("visual_imagem", i))?
        )?,
        texto_alternativo: t.ou("texto_alternativo", String::new(), |t| t.texto("texto_alternativo", i))?,
        ajustes: t.ou("ajustes", String::new(), |t| t.texto("ajustes", i))?.split(';').filter_map(|v| v.parse().ok()).collect(),
        recorte: valor_de(&Tipo::TODOS, t.ou("recorte", 1, |t| t.u8("recorte", i))?),
        sombra: Sombra {
            cor: t.ou("sombra_cor", String::new(), |t| t.texto("sombra_cor", i))?,
            desfoque: t.ou("sombra_desfoque", 0.0, |t| t.f32("sombra_desfoque", i))?,
            dx: t.ou("sombra_dx", 0.0, |t| t.f32("sombra_dx", i))?,
            dy: t.ou("sombra_dy", 0.0, |t| t.f32("sombra_dy", i))?,
        },
        dados: t.ou("dados", String::new(), |t| t.texto("dados", i))?,
        destino: t.ou("destino", 0, |t| Ok(t.col::<arrow::array::UInt32Array>("destino")?.value(i)))?,
        link: t.ou("link", String::new(), |t| t.texto("link", i))?,
        oculto: t.ou("oculto", false, |t| t.bool("oculto", i))?,
        opacidade: t.ou("opacidade", 1.0, |t| t.f32("opacidade", i))?,
        fundo_removido: t.ou("fundo_removido", 0.0, |t| t.f32("fundo_removido", i))?,
        fundo_cor: t.ou("fundo_cor", String::new(), |t| t.texto("fundo_cor", i))?,
        fundo_suave: t.ou("fundo_suave", 0.5, |t| t.f32("fundo_suave", i))?,
        fundo_modo: t.ou("fundo_modo", 0, |t| t.u8("fundo_modo", i))?,
        fundo_refino: t.ou("fundo_refino", false, |t| t.bool("fundo_refino", i))?,
        fundo_marcas: t.ou("fundo_marcas", String::new(), |t| t.texto("fundo_marcas", i))?,
        fundo_borda: t.ou("fundo_borda", 0.0, |t| t.f32("fundo_borda", i))?,
        fundo_pena: t.ou("fundo_pena", 0.0, |t| t.f32("fundo_pena", i))?,
        grupo: t.ou("grupo", 0, |t| Ok(t.col::<arrow::array::UInt32Array>("grupo")?.value(i)))?,
        animacao: Animacao {
            tipo: valor_de(&TipoAnimacao::TODOS, t.ou("anim_tipo", 0, |t| t.u8("anim_tipo", i))?),
            inicio: valor_de(&InicioAnimacao::TODOS, t.ou("anim_inicio", 0, |t| t.u8("anim_inicio", i))?),
            direcao: valor_de(&Direcao::TODOS, t.ou("anim_dir", 0, |t| t.u8("anim_dir", i))?),
            duracao: t.ou("anim_duracao", 0.5, |t| t.f32("anim_duracao", i))?,
            atraso: t.ou("anim_atraso", 0.0, |t| t.f32("anim_atraso", i))?,
        },
    })
}

/// Põe cada elemento no slide dele; a imagem em `assets/` volta a ser `data:`.
fn distribuir_elementos(ap: &mut Apresentacao, t: &Lida, parte: &mut dyn FnMut(&str) -> Result<Vec<u8>, String>) -> Result<(), String> {
    let posicao: HashMap<u64, usize> = ap.slides.iter().enumerate().map(|(i, s)| (s.id, i)).collect();
    for i in 0..t.linhas() {
        let slide = t.u64("slide", i)?;
        let &onde = posicao.get(&slide).ok_or_else(|| format!("elementos.arrow: o elemento da linha {} aponta um slide que não existe.", i + 1))?;
        let mut e = ler_elemento(t, i)?;
        featherslide_core::texto_formato::validar(&e)?;
        if e.imagem.starts_with("assets/") {
            let bytes = parte(&e.imagem)?;
            e.imagem = base64::para_data_url(tipo_da_imagem(&e.imagem), &bytes).into();
        }
        ap.slides[onde].elementos.push(e);
    }
    Ok(())
}

fn sanitizar_nome_fonte(nome: &str) -> String {
    let s: String = nome.chars().filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
    if s.is_empty() {
        "fonte".into()
    } else {
        s
    }
}

type ArquivoZip = (String, Vec<u8>);

fn fontes_arrow(fontes: &[FonteEmbutida]) -> Result<(Vec<u8>, Vec<ArquivoZip>), String> {
    let mut nomes = Vec::new();
    let mut regs = Vec::new();
    let mut negs = Vec::new();
    let mut itas = Vec::new();
    let mut nits = Vec::new();
    let mut arquivos = Vec::new();

    for f in fontes {
        let base = sanitizar_nome_fonte(&f.nome);
        nomes.push(f.nome.clone());
        let mut guardar_var = |var: &Option<Vec<u8>>, sufixo: &str| -> String {
            match var {
                Some(b) => {
                    let nome_arq = format!("assets/fontes/{base}-{sufixo}.ttf");
                    arquivos.push((nome_arq.clone(), b.clone()));
                    nome_arq
                }
                None => String::new(),
            }
        };
        regs.push(guardar_var(&f.regular, "regular"));
        negs.push(guardar_var(&f.negrito, "negrito"));
        itas.push(guardar_var(&f.italico, "italico"));
        nits.push(guardar_var(&f.negrito_italico, "negrito-italico"));
    }

    let colunas: Vec<(&str, ArrayRef)> = vec![
        ("nome", Arc::new(StringArray::from(nomes))),
        ("regular", Arc::new(StringArray::from(regs))),
        ("negrito", Arc::new(StringArray::from(negs))),
        ("italico", Arc::new(StringArray::from(itas))),
        ("negrito_italico", Arc::new(StringArray::from(nits))),
    ];
    let tab = tabela(colunas, HashMap::new())?;
    Ok((tab, arquivos))
}

fn ler_fontes_fsdz(parte: &mut dyn FnMut(&str) -> Result<Vec<u8>, String>) -> Result<Vec<FonteEmbutida>, String> {
    let bytes_tab = match parte("data/fontes.arrow") {
        Ok(b) => b,
        Err(_) => return Ok(Vec::new()),
    };
    let t = Lida::de(&bytes_tab, "fontes.arrow")?;
    let mut fontes = Vec::new();
    for i in 0..t.linhas() {
        let nome = t.texto("nome", i)?;
        let mut ler_var = |col: &str| -> Result<Option<Vec<u8>>, String> {
            let cam = t.texto(col, i)?;
            if cam.is_empty() {
                Ok(None)
            } else {
                parte(&cam).map(Some)
            }
        };
        fontes.push(FonteEmbutida {
            nome,
            regular: ler_var("regular")?,
            negrito: ler_var("negrito")?,
            italico: ler_var("italico")?,
            negrito_italico: ler_var("negrito_italico")?,
        });
    }
    Ok(fontes)
}
