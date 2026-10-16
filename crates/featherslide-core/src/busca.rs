//! Localizar e substituir texto em apresentações: busca em todos os slides e elementos
//! de texto, com opções de maiúsculas/minúsculas e palavra inteira.

use serde::{Deserialize, Serialize};

use crate::modelo::Slide;
use crate::sessao::{Mudanca, Sessao};

/// As opções da busca.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpcoesDeBusca {
    #[serde(default)]
    pub maiusculas: bool,
    #[serde(default)]
    pub palavra: bool,
}

/// Uma ocorrência encontrada.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Achado {
    pub slide: usize,
    pub elemento: u64,
    pub inicio: usize,
    pub fim: usize,
}

/// Estado interno da busca mantido na sessão.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EstadoDaBusca {
    pub consulta: String,
    pub opcoes: OpcoesDeBusca,
    pub achados: Vec<Achado>,
    pub indice: usize,
}

/// Resposta para a interface após localizar ou navegar.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResultadoDeBusca {
    pub total: usize,
    pub atual: usize,
    pub slide: Option<usize>,
    pub elemento: Option<u64>,
    pub erro: Option<String>,
}

/// Resposta para a interface após substituir ou substituir tudo.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResultadoSubstituicao {
    pub trocados: usize,
    pub total: usize,
    pub atual: usize,
    pub mudanca: Option<Mudanca>,
}

fn eh_borda_de_palavra(chars: &[char], inicio: usize, fim: usize) -> bool {
    let antes = if inicio > 0 { Some(chars[inicio - 1]) } else { None };
    let depois = if fim < chars.len() { Some(chars[fim]) } else { None };
    let eh_palavra = |c: char| c.is_alphanumeric() || c == '_';
    antes.map_or(true, |c| !eh_palavra(c)) && depois.map_or(true, |c| !eh_palavra(c))
}

/// Acha as ocorrências no texto dado (em índices de caracteres Unicode).
pub fn achar_em_texto(texto: &str, consulta: &str, opcoes: OpcoesDeBusca) -> Vec<(usize, usize)> {
    if consulta.is_empty() || texto.is_empty() {
        return Vec::new();
    }
    let chars_texto: Vec<char> = if opcoes.maiusculas {
        texto.chars().collect()
    } else {
        texto.to_lowercase().chars().collect()
    };
    let chars_cons: Vec<char> = if opcoes.maiusculas {
        consulta.chars().collect()
    } else {
        consulta.to_lowercase().chars().collect()
    };
    if chars_cons.is_empty() || chars_texto.len() < chars_cons.len() {
        return Vec::new();
    }
    let mut ocorrencias = Vec::new();
    let mut i = 0;
    while i + chars_cons.len() <= chars_texto.len() {
        if chars_texto[i..i + chars_cons.len()] == chars_cons[..] {
            let fim = i + chars_cons.len();
            if !opcoes.palavra || eh_borda_de_palavra(&chars_texto, i, fim) {
                ocorrencias.push((i, fim));
                i = fim;
                continue;
            }
        }
        i += 1;
    }
    ocorrencias
}

fn coletar_achados(slides: &[Slide], consulta: &str, opcoes: OpcoesDeBusca) -> Vec<Achado> {
    let mut todos = Vec::new();
    for (s_idx, s) in slides.iter().enumerate() {
        for e in &s.elementos {
            if !e.tipo.aceita_texto() || e.texto.is_empty() {
                continue;
            }
            for (inicio, fim) in achar_em_texto(&e.texto, consulta, opcoes) {
                todos.push(Achado { slide: s_idx, elemento: e.id, inicio, fim });
            }
        }
    }
    todos
}

fn substituir_texto(texto: &str, inicio_char: usize, fim_char: usize, por: &str) -> String {
    let chars: Vec<char> = texto.chars().collect();
    let antes: String = chars[..inicio_char].iter().collect();
    let depois: String = chars[fim_char..].iter().collect();
    format!("{antes}{por}{depois}")
}

impl Sessao {
    /// Inicia uma busca ou atualiza a consulta atual.
    pub fn busca_definir(&mut self, consulta: &str, opcoes: OpcoesDeBusca) -> ResultadoDeBusca {
        if consulta.is_empty() {
            self.busca = None;
            return ResultadoDeBusca::default();
        }
        let achados = coletar_achados(&self.ap.slides, consulta, opcoes);
        if achados.is_empty() {
            self.busca = Some(EstadoDaBusca { consulta: consulta.into(), opcoes, achados: Vec::new(), indice: 0 });
            return ResultadoDeBusca { total: 0, atual: 0, slide: None, elemento: None, erro: None };
        }
        let slide_atual = self.atual;
        let proximo = achados.iter().position(|a| a.slide >= slide_atual).unwrap_or(0);
        let achado = achados[proximo];
        self.atual = achado.slide;
        self.selecao = vec![achado.elemento];
        self.busca = Some(EstadoDaBusca { consulta: consulta.into(), opcoes, achados, indice: proximo });
        ResultadoDeBusca {
            total: self.busca.as_ref().unwrap().achados.len(),
            atual: proximo + 1,
            slide: Some(achado.slide),
            elemento: Some(achado.elemento),
            erro: None,
        }
    }

    /// Navega para o próximo (+1) ou anterior (-1) achado.
    pub fn busca_ir(&mut self, delta: i32) -> ResultadoDeBusca {
        let Some(ref mut b) = self.busca else {
            return ResultadoDeBusca::default();
        };
        let n = b.achados.len();
        if n == 0 {
            return ResultadoDeBusca::default();
        }
        let novo = if delta >= 0 {
            (b.indice + delta as usize) % n
        } else {
            (b.indice + n - ((-delta) as usize % n)) % n
        };
        b.indice = novo;
        let achado = b.achados[novo];
        self.atual = achado.slide;
        self.selecao = vec![achado.elemento];
        ResultadoDeBusca {
            total: n,
            atual: novo + 1,
            slide: Some(achado.slide),
            elemento: Some(achado.elemento),
            erro: None,
        }
    }

    /// Substitui o achado selecionado pelo texto `por`.
    pub fn busca_substituir(&mut self, por: &str) -> Result<ResultadoSubstituicao, String> {
        let (consulta, opcoes, achado) = {
            let Some(ref b) = self.busca else {
                return Ok(ResultadoSubstituicao::default());
            };
            if b.achados.is_empty() {
                return Ok(ResultadoSubstituicao::default());
            }
            (b.consulta.clone(), b.opcoes, b.achados[b.indice])
        };
        let slide_idx = achado.slide;
        let elem_id = achado.elemento;
        let mudanca = self.editar(|s| {
            let slide = s.ap.slides.get_mut(slide_idx).ok_or("Slide não encontrado.")?;
            let elem = slide.elementos.iter_mut().find(|e| e.id == elem_id).ok_or("Elemento não encontrado.")?;
            elem.texto = substituir_texto(&elem.texto, achado.inicio, achado.fim, por);
            Ok((vec![slide_idx], false))
        })?;
        let res = self.busca_definir(&consulta, opcoes);
        Ok(ResultadoSubstituicao {
            trocados: 1,
            total: res.total,
            atual: res.atual,
            mudanca: Some(mudanca),
        })
    }

    /// Substitui todas as ocorrências de todos os slides pelo texto `por`.
    pub fn busca_substituir_tudo(&mut self, por: &str) -> Result<ResultadoSubstituicao, String> {
        let (consulta, opcoes) = {
            let Some(ref b) = self.busca else {
                return Ok(ResultadoSubstituicao::default());
            };
            if b.achados.is_empty() {
                return Ok(ResultadoSubstituicao::default());
            }
            (b.consulta.clone(), b.opcoes)
        };
        let mut total_trocas = 0;
        let mut slides_tocados = Vec::new();
        self.editar(|s| {
            for (idx, slide) in s.ap.slides.iter_mut().enumerate() {
                let mut tocou_slide = false;
                for e in &mut slide.elementos {
                    if !e.tipo.aceita_texto() || e.texto.is_empty() {
                        continue;
                    }
                    let achados = achar_em_texto(&e.texto, &consulta, opcoes);
                    if achados.is_empty() {
                        continue;
                    }
                    tocou_slide = true;
                    total_trocas += achados.len();
                    // Substitui de trás para frente para preservar os offsets
                    let mut txt = e.texto.clone();
                    for (ini, fim) in achados.into_iter().rev() {
                        txt = substituir_texto(&txt, ini, fim, por);
                    }
                    crate::texto_formato::atualizar_indices(e,&txt);
                    e.texto = txt;
                }
                if tocou_slide {
                    slides_tocados.push(idx);
                }
            }
            Ok((slides_tocados.clone(), false))
        })?;
        self.busca_definir(&consulta, opcoes);
        let m = if total_trocas > 0 {
            Some(Mudanca { versao: self.versao, slides: slides_tocados, lista: false })
        } else {
            None
        };
        Ok(ResultadoSubstituicao {
            trocados: total_trocas,
            total: 0,
            atual: 0,
            mudanca: m,
        })
    }

    /// Encerra a busca e limpa o estado.
    pub fn busca_encerrar(&mut self) {
        self.busca = None;
    }
}
