//! A ponte: um despacho único (`Ponte::chamar`) casa o nome do comando com a função
//! do motor. Comando que muda algo devolve **a versão nova e onde mudou** (os slides
//! tocados); o slide em si só sai pelo `slide_quadro`, e só se a versão que a interface
//! tem já não vale (RECEITA.md §3).
//!
//! Arquivo atravessa como bytes crus (`apresentacao_abrir_fsdz`, `apresentacao_gravar_fsdz`),
//! nunca em JSON: cada byte viraria um número decimal.

use featherslide_core::modelo::{Alinhamento, Proporcao, Tipo, Vertical};
use featherslide_core::quadro::Dicas;
use featherslide_core::sessao::{Mudanca, Sessao};
use featherslide_core::{Formatacao, Ordem};
use featherslide_io::{fsdz, odp, pptx};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Os comandos que o despacho Rust atende, verificados pelos testes nativos.
pub const COMANDOS_DA_PONTE: &[&str] = &[
    "comandos_da_ponte",
    "estado",
    "apresentacao_nova",
    "apresentacao_abrir_fsdz",
    "apresentacao_abrir_pptx",
    "apresentacao_abrir_odp",
    "apresentacao_gravar_fsdz",
    "apresentacao_marcar_gravado",
    "fontes_listar",
    "fonte_bytes",
    "dicas_definir",
    "titulo_definir",
    "slide_quadro",
    "slide_novo",
    "slide_duplicar",
    "slide_excluir",
    "slide_mover",
    "slide_ir",
    "slide_layout",
    "slide_fundo",
    "slide_ocultar",
    "notas_definir",
    "proporcao_definir",
    "elemento_inserir",
    "elemento_no_ponto",
    "elemento_texto",
    "selecionar_no_ponto",
    "selecionar_na_area",
    "selecionar_tudo",
    "desmarcar",
    "selecao_mover",
    "elemento_geometria",
    "texto_definir",
    "selecao_excluir",
    "selecao_duplicar",
    "formatar",
    "organizar",
    "alinhar_no_slide",
    "copiar",
    "recortar",
    "colar",
    "desfazer",
    "refazer",
    "elemento_rotacao",
    "lista_alternar",
    "busca_definir",
    "busca_ir",
    "busca_substituir",
    "busca_substituir_tudo",
    "busca_encerrar",
];

/// O motor de uma apresentação aberta.
#[derive(Default)]
pub struct Ponte {
    sessao: Sessao,
    /// As dicas das caixas vazias na língua da interface: valem também para a próxima apresentação aberta.
    dicas: Dicas,
}

#[derive(Deserialize)]
struct PedidoDeQuadro {
    indice: usize,
    #[serde(default)]
    versao: Option<u64>,
    #[serde(default = "sim")]
    editor: bool,
}

fn sim() -> bool {
    true
}

#[derive(Deserialize)]
struct Indice {
    indice: usize,
}

#[derive(Deserialize)]
struct Mover {
    de: usize,
    para: usize,
}

#[derive(Deserialize)]
struct Layout {
    layout: String,
}

#[derive(Deserialize)]
struct Fundo {
    cor: String,
    #[serde(default)]
    todos: bool,
}

#[derive(Deserialize)]
struct Texto {
    texto: String,
}

#[derive(Deserialize)]
struct ParaProporcao {
    proporcao: Proporcao,
}

#[derive(Deserialize)]
struct Inserir {
    tipo: String,
    #[serde(default)]
    rect: Option<[f32; 4]>,
    #[serde(default)]
    imagem: Option<String>,
}

#[derive(Deserialize)]
struct Ponto {
    x: f32,
    y: f32,
    #[serde(default)]
    somar: bool,
}

#[derive(Deserialize)]
struct Area {
    rect: [f32; 4],
    #[serde(default)]
    somar: bool,
}

#[derive(Deserialize)]
struct Deslocar {
    dx: f32,
    dy: f32,
}

#[derive(Deserialize)]
struct Geometria {
    id: u64,
    rect: [f32; 4],
}

#[derive(Deserialize)]
struct PorId {
    id: u64,
}

#[derive(Deserialize)]
struct TextoDoElemento {
    id: u64,
    texto: String,
}

#[derive(Deserialize)]
struct Formatar {
    campo: String,
    #[serde(default)]
    valor: serde_json::Value,
}

#[derive(Deserialize)]
struct Organizar {
    ordem: String,
}

#[derive(Deserialize)]
struct Como {
    como: String,
}

#[derive(Deserialize)]
struct Rotacao {
    id: u64,
    graus: f32,
}

#[derive(Deserialize)]
struct ListaTipo {
    tipo: String,
}

#[derive(Deserialize)]
struct BuscaDefinir {
    #[serde(default)]
    consulta: String,
    #[serde(default)]
    opcoes: featherslide_core::OpcoesDeBusca,
}

#[derive(Deserialize)]
struct BuscaIr {
    #[serde(default)]
    delta: i32,
}

#[derive(Deserialize)]
struct BuscaSubstituir {
    #[serde(default)]
    por: String,
}

#[derive(Deserialize)]
struct DicasDaInterface {
    titulo: String,
    subtitulo: String,
    corpo: String,
}

#[derive(Serialize)]
struct Acertado {
    id: Option<u64>,
}

#[derive(Serialize)]
struct Recortado {
    texto: String,
    mudanca: Mudanca,
}

#[derive(Serialize)]
struct FonteResumo {
    nome: String,
    variantes: Vec<&'static str>,
}

#[derive(Deserialize)]
struct FontePedido {
    nome: String,
    variante: String,
}

fn listar_fontes(s: &Sessao) -> Result<Vec<u8>, String> {
    let lista: Vec<FonteResumo> = s.apresentacao().fontes.iter().map(|f| {
        let mut variantes = Vec::new();
        if f.regular.is_some() { variantes.push("regular"); }
        if f.negrito.is_some() { variantes.push("negrito"); }
        if f.italico.is_some() { variantes.push("italico"); }
        if f.negrito_italico.is_some() { variantes.push("negrito_italico"); }
        FonteResumo {
            nome: f.nome.clone(),
            variantes,
        }
    }).collect();
    json(&lista)
}

fn bytes_da_fonte(s: &Sessao, corpo: &[u8]) -> Result<Vec<u8>, String> {
    let p: FontePedido = args(corpo)?;
    let f = s.apresentacao().fontes.iter().find(|fonte| fonte.nome == p.nome)
        .ok_or_else(|| format!("Fonte não encontrada: {}", p.nome))?;
    let var = match p.variante.as_str() {
        "negrito" | "bold" => f.negrito.as_ref(),
        "italico" | "italic" => f.italico.as_ref(),
        "negrito_italico" | "bold_italic" | "boldItalic" => f.negrito_italico.as_ref(),
        _ => f.regular.as_ref(),
    };
    let bytes = var.ok_or_else(|| format!("Variante '{}' não encontrada para '{}'", p.variante, p.nome))?;
    Ok(bytes.clone())
}

fn args<T: DeserializeOwned>(corpo: &[u8]) -> Result<T, String> {
    serde_json::from_slice(corpo).map_err(|e| format!("pedido malformado: {e}"))
}

fn json<T: Serialize + ?Sized>(v: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(v).map_err(|e| format!("resposta malformada: {e}"))
}

/// O `valor` da formatação como texto, número ou nada.
fn texto_de(v: &serde_json::Value) -> Result<String, String> {
    v.as_str().map(str::to_string).ok_or_else(|| "a formatação pede um texto".into())
}

fn numero_de(v: &serde_json::Value) -> Result<f32, String> {
    v.as_f64().map(|n| n as f32).ok_or_else(|| "a formatação pede um número".into())
}

/// A formatação pelo nome do campo que a interface manda.
fn formatacao(campo: &str, v: &serde_json::Value) -> Result<Formatacao, String> {
    Ok(match campo {
        "fonte" => Formatacao::Fonte(texto_de(v)?),
        "tamanho" => Formatacao::Tamanho(numero_de(v)?),
        "tamanho_passo" => Formatacao::TamanhoPasso(numero_de(v)? as i32),
        "cor" => Formatacao::CorDoTexto(texto_de(v)?),
        "negrito" => Formatacao::Negrito,
        "italico" => Formatacao::Italico,
        "sublinhado" => Formatacao::Sublinhado,
        "tachado" => Formatacao::Tachado,
        "sublinhado_duplo" => Formatacao::SublinhadoDuplo,
        "tachado_duplo" => Formatacao::TachadoDuplo,
        "limpar" => Formatacao::Limpar,
        "entrelinha" => Formatacao::Entrelinha(numero_de(v)?),
        "colunas" => Formatacao::Colunas(numero_de(v)? as u8),
        "alinhar" => Formatacao::Alinhar(match texto_de(v)?.as_str() {
            "centro" => Alinhamento::Centro,
            "direita" => Alinhamento::Direita,
            "justificado" => Alinhamento::Justificado,
            _ => Alinhamento::Esquerda,
        }),
        "vertical" => Formatacao::Vertical(match texto_de(v)?.as_str() {
            "meio" => Vertical::Meio,
            "base" => Vertical::Base,
            _ => Vertical::Topo,
        }),
        "preenchimento" => Formatacao::Preenchimento(texto_de(v)?),
        "contorno" => Formatacao::Contorno(texto_de(v)?),
        "espessura" => Formatacao::Espessura(numero_de(v)?),
        _ => return Err(format!("Formatação desconhecida: {campo}")),
    })
}

fn ordem(nome: &str) -> Result<Ordem, String> {
    Ok(match nome {
        "frente" => Ordem::Frente,
        "tras" => Ordem::Tras,
        "avancar" => Ordem::Avancar,
        "recuar" => Ordem::Recuar,
        _ => return Err(format!("Ordem desconhecida: {nome}")),
    })
}

/// "Nada mudou" (desfazer sem passo guardado, por exemplo).
fn nada(s: &Sessao) -> Mudanca {
    Mudanca { versao: s.estado().versao, slides: Vec::new(), lista: false }
}

impl Ponte {
    /// A sessão ativa da apresentação aberta.
    pub fn sessao(&self) -> &Sessao {
        &self.sessao
    }

    /// A sessão mutável da apresentação aberta (para operações diretas e MCP).
    pub fn sessao_mut(&mut self) -> &mut Sessao {
        &mut self.sessao
    }

    /// Os comandos que só leem: podem rodar ao mesmo tempo (trava de leitura no aplicativo).
    /// `None` = não é de leitura, vá para `chamar`.
    pub fn chamar_leitura(&self, nome: &str, corpo: &[u8]) -> Option<Result<Vec<u8>, String>> {
        let s = &self.sessao;
        Some(match nome {
            "comandos_da_ponte" => json(COMANDOS_DA_PONTE),
            "estado" => json(&s.estado()),
            "slide_quadro" => args::<PedidoDeQuadro>(corpo).and_then(|a| json(&s.quadro(a.indice, a.versao, a.editor)?)),
            "apresentacao_gravar_fsdz" => fsdz::gravar(s.apresentacao()),
            "fontes_listar" => listar_fontes(s),
            "fonte_bytes" => bytes_da_fonte(s, corpo),
            "elemento_no_ponto" => args::<Ponto>(corpo).and_then(|a| json(&Acertado { id: s.elemento_no_ponto(a.x, a.y) })),
            "elemento_texto" => args::<PorId>(corpo).and_then(|a| {
                let e = s.apresentacao().slides[s.atual()].elementos.iter().find(|e| e.id == a.id).ok_or("Esse elemento não está no slide.")?;
                json(&e.texto)
            }),
            _ => return None,
        })
    }

    /// Atende um comando. Erro é sempre uma mensagem em português para mostrar ao usuário.
    pub fn chamar(&mut self, nome: &str, corpo: &[u8]) -> Result<Vec<u8>, String> {
        if let Some(r) = self.chamar_leitura(nome, corpo) {
            return r;
        }
        match nome {
            "apresentacao_nova" | "apresentacao_abrir_fsdz" | "apresentacao_abrir_pptx" | "apresentacao_abrir_odp" | "apresentacao_marcar_gravado" | "dicas_definir" | "titulo_definir" => self.arquivo(nome, corpo),
            "slide_novo" | "slide_duplicar" | "slide_excluir" | "slide_mover" | "slide_ir" | "slide_layout" | "slide_fundo" | "slide_ocultar" | "notas_definir" | "proporcao_definir" => self.slides(nome, corpo),
            "selecionar_no_ponto" | "selecionar_na_area" | "selecionar_tudo" | "desmarcar" => self.selecao(nome, corpo),
            _ => self.elementos(nome, corpo),
        }
    }

    /// Abrir, criar, gravar e o que vale para a apresentação inteira.
    fn arquivo(&mut self, nome: &str, corpo: &[u8]) -> Result<Vec<u8>, String> {
        let dicas = self.dicas.clone();
        match nome {
            "apresentacao_nova" | "apresentacao_abrir_fsdz" | "apresentacao_abrir_pptx" | "apresentacao_abrir_odp" => {
                let ap = match nome {
                    "apresentacao_abrir_fsdz" => Some(fsdz::ler(corpo)?),
                    "apresentacao_abrir_pptx" => Some(pptx::ler(corpo)?),
                    "apresentacao_abrir_odp" => Some(odp::ler(corpo)?),
                    _ => None,
                };
                self.sessao = ap.map(Sessao::nova).unwrap_or_default();
                self.sessao.definir_dicas(dicas);
                self.sessao.marcar_gravado();
                json(&self.sessao.estado())
            }
            "apresentacao_marcar_gravado" => {
                self.sessao.marcar_gravado();
                json(&self.sessao.estado())
            }
            "dicas_definir" => {
                let d = args::<DicasDaInterface>(corpo)?;
                self.dicas = Dicas { titulo: d.titulo, subtitulo: d.subtitulo, corpo: d.corpo };
                json(&self.sessao.definir_dicas(self.dicas.clone()))
            }
            _ => json(&self.sessao.titulo_definir(&args::<Texto>(corpo)?.texto)?),
        }
    }

    fn slides(&mut self, nome: &str, corpo: &[u8]) -> Result<Vec<u8>, String> {
        let s = &mut self.sessao;
        let m = match nome {
            "slide_novo" => s.slide_novo(&args::<Layout>(corpo)?.layout)?,
            "slide_duplicar" => s.slide_duplicar()?,
            "slide_excluir" => s.slide_excluir()?,
            "slide_mover" => args::<Mover>(corpo).and_then(|a| s.slide_mover(a.de, a.para))?,
            "slide_ir" => {
                s.slide_ir(args::<Indice>(corpo)?.indice)?;
                return json(&s.estado());
            }
            "slide_layout" => s.slide_layout(&args::<Layout>(corpo)?.layout)?,
            "slide_fundo" => args::<Fundo>(corpo).and_then(|a| s.slide_fundo(&a.cor, a.todos))?,
            "slide_ocultar" => s.slide_ocultar()?,
            "notas_definir" => s.notas_definir(&args::<Texto>(corpo)?.texto)?,
            _ => s.proporcao_definir(args::<ParaProporcao>(corpo)?.proporcao)?,
        };
        json(&m)
    }

    /// A seleção não é edição (não entra no desfazer): devolve o elemento acertado ou o estado.
    fn selecao(&mut self, nome: &str, corpo: &[u8]) -> Result<Vec<u8>, String> {
        let s = &mut self.sessao;
        match nome {
            "selecionar_no_ponto" => {
                let a = args::<Ponto>(corpo)?;
                json(&Acertado { id: s.selecionar_no_ponto(a.x, a.y, a.somar) })
            }
            "selecionar_na_area" => {
                let a = args::<Area>(corpo)?;
                s.selecionar_na_area(a.rect, a.somar);
                json(&s.estado())
            }
            "selecionar_tudo" => {
                s.selecionar_tudo();
                json(&s.estado())
            }
            _ => {
                s.desmarcar();
                json(&s.estado())
            }
        }
    }

    fn elementos(&mut self, nome: &str, corpo: &[u8]) -> Result<Vec<u8>, String> {
        let s = &mut self.sessao;
        let m = match nome {
            "elemento_inserir" => {
                let a = args::<Inserir>(corpo)?;
                let tipo = Tipo::pelo_nome(&a.tipo).ok_or_else(|| format!("Tipo de elemento desconhecido: {}", a.tipo))?;
                s.elemento_inserir(tipo, a.rect, a.imagem)?
            }
            "selecao_mover" => args::<Deslocar>(corpo).and_then(|a| s.selecao_mover(a.dx, a.dy))?,
            "elemento_geometria" => args::<Geometria>(corpo).and_then(|a| s.elemento_geometria(a.id, a.rect))?,
            "texto_definir" => args::<TextoDoElemento>(corpo).and_then(|a| s.texto_definir(a.id, &a.texto))?,
            "selecao_excluir" => s.selecao_excluir()?,
            "selecao_duplicar" => s.selecao_duplicar()?,
            "formatar" => args::<Formatar>(corpo).and_then(|a| s.formatar(formatacao(&a.campo, &a.valor)?))?,
            "organizar" => args::<Organizar>(corpo).and_then(|a| s.organizar(ordem(&a.ordem)?))?,
            "alinhar_no_slide" => s.alinhar_no_slide(&args::<Como>(corpo)?.como)?,
            "copiar" => return json(&s.copiar()),
            "recortar" => {
                let (texto, mudanca) = s.recortar()?;
                return json(&Recortado { texto, mudanca });
            }
            "colar" => s.colar(&args::<Texto>(corpo)?.texto)?,
            "desfazer" => s.desfazer().unwrap_or_else(|| nada(s)),
            "refazer" => s.refazer().unwrap_or_else(|| nada(s)),
            "elemento_rotacao" => args::<Rotacao>(corpo).and_then(|a| s.elemento_rotacao(a.id, a.graus))?,
            "lista_alternar" => args::<ListaTipo>(corpo).and_then(|a| s.lista_alternar(&a.tipo))?,
            "busca_definir" => return args::<BuscaDefinir>(corpo).and_then(|a| json(&s.busca_definir(&a.consulta, a.opcoes))),
            "busca_ir" => return args::<BuscaIr>(corpo).and_then(|a| json(&s.busca_ir(a.delta))),
            "busca_substituir" => return args::<BuscaSubstituir>(corpo).and_then(|a| json(&s.busca_substituir(&a.por)?)),
            "busca_substituir_tudo" => return args::<BuscaSubstituir>(corpo).and_then(|a| json(&s.busca_substituir_tudo(&a.por)?)),
            "busca_encerrar" => {
                s.busca_encerrar();
                return json(&());
            }
            _ => return Err(format!("Comando desconhecido: {nome}")),
        };
        json(&m)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn chamar(p: &mut Ponte, nome: &str, corpo: &str) -> serde_json::Value {
        serde_json::from_slice(&p.chamar(nome, corpo.as_bytes()).unwrap()).unwrap()
    }

    #[test]
    fn todo_comando_listado_e_atendido() {
        let mut p = Ponte::default();
        for nome in COMANDOS_DA_PONTE {
            let r = p.chamar(nome, b"{}");
            if let Err(e) = r {
                assert!(!e.starts_with("Comando desconhecido"), "{nome}: {e}");
            }
        }
    }

    #[test]
    fn gravar_e_abrir_pela_ponte() {
        let mut p = Ponte::default();
        chamar(&mut p, "dicas_definir", r#"{"titulo":"T","subtitulo":"S","corpo":"C"}"#);
        chamar(&mut p, "slide_novo", r#"{"layout":"titulo_conteudo"}"#);
        chamar(&mut p, "elemento_inserir", r#"{"tipo":"elipse"}"#);
        chamar(&mut p, "formatar", r##"{"campo":"preenchimento","valor":"#ff0000"}"##);
        assert_eq!(chamar(&mut p, "estado", "")["sujo"], true);
        let bytes = p.chamar("apresentacao_gravar_fsdz", b"").unwrap();
        let mut outra = Ponte::default();
        let e = serde_json::from_slice::<serde_json::Value>(&outra.chamar("apresentacao_abrir_fsdz", &bytes).unwrap()).unwrap();
        assert_eq!(e["slides"].as_array().unwrap().len(), 2);
        assert_eq!(e["sujo"], false);
    }

    #[test]
    fn quadro_e_selecao_pela_ponte() {
        let mut p = Ponte::default();
        let q = chamar(&mut p, "slide_quadro", r#"{"indice":0}"#);
        let v = q["versao"].as_u64().unwrap();
        assert!(chamar(&mut p, "slide_quadro", &format!(r#"{{"indice":0,"versao":{v}}}"#)).is_null());
        let el = &q["elementos"][0];
        let (x, y) = (el["x"].as_f64().unwrap() + 5.0, el["y"].as_f64().unwrap() + 5.0);
        let a = chamar(&mut p, "selecionar_no_ponto", &format!(r#"{{"x":{x},"y":{y}}}"#));
        assert_eq!(a["id"], el["id"]);
        assert!(p.chamar("formatar", br#"{"campo":"brilho"}"#).unwrap_err().contains("desconhecida"));
    }
}
