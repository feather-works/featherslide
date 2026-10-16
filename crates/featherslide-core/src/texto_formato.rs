//! Índices por trecho e espaçamento tipográfico, sem substituir os caracteres do documento.
use crate::{
    medida,
    modelo::Elemento,
    sessao::{Mudanca, Sessao},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Indice {
    pub inicio: usize,
    pub fim: usize,
    pub nivel: i8,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Glifo {
    pub texto: String,
    pub x: f32,
    pub y: f32,
    pub tamanho: f32,
}
pub fn nivel(e: &Elemento, byte: usize) -> i8 {
    e.indices_texto
        .iter()
        .find(|i| byte >= i.inicio && byte < i.fim)
        .map_or(0, |i| i.nivel)
}
pub fn avancar(e: &Elemento, byte: usize, c: char) -> f32 {
    let t = e.estilo.tamanho * if nivel(e, byte) == 0 { 1.0 } else { 0.65 };
    (medida::largura(&c.to_string(), t, e.estilo.negrito) + e.espacamento_letras).max(0.1)
}
pub fn atualizar_indices(e: &mut Elemento, novo: &str) {
    let mut prefixo = 0;
    for ((i, a), (_, b)) in e.texto.char_indices().zip(novo.char_indices()) {
        if a != b {
            break;
        }
        prefixo = i + a.len_utf8();
    }
    let mut sufixo = 0;
    for (a, b) in e.texto[prefixo..]
        .chars()
        .rev()
        .zip(novo[prefixo..].chars().rev())
    {
        if a != b {
            break;
        }
        sufixo += a.len_utf8();
    }
    let antigo_fim = e.texto.len() - sufixo;
    let novo_fim = novo.len() - sufixo;
    let delta = novo_fim as isize - antigo_fim as isize;
    let mut saida = Vec::new();
    for i in &e.indices_texto {
        if i.fim <= prefixo {
            saida.push(i.clone());
        } else if i.inicio >= antigo_fim {
            saida.push(Indice {
                inicio: i.inicio.saturating_add_signed(delta),
                fim: i.fim.saturating_add_signed(delta),
                nivel: i.nivel,
            });
        } else {
            if i.inicio < prefixo {
                saida.push(Indice {
                    inicio: i.inicio,
                    fim: prefixo,
                    nivel: i.nivel,
                });
            }
            if i.fim > antigo_fim {
                saida.push(Indice {
                    inicio: novo_fim,
                    fim: i.fim.saturating_add_signed(delta),
                    nivel: i.nivel,
                });
            }
        }
    }
    e.indices_texto = saida;
}
impl Sessao {
    pub fn texto_indice(
        &mut self,
        id: u64,
        inicio: usize,
        fim: usize,
        valor: i8,
    ) -> Result<Mudanca, String> {
        let e = self
            .slide()
            .elementos
            .iter()
            .find(|e| e.id == id)
            .ok_or("Selecione texto.")?;
        if !e.tipo.aceita_texto()
            || inicio >= fim
            || fim > e.texto.len()
            || !e.texto.is_char_boundary(inicio)
            || !e.texto.is_char_boundary(fim)
            || !(-1..=1).contains(&valor)
        {
            return Err("Selecione um trecho de texto para aplicar x² ou x₂.".into());
        }
        let valor = if (inicio..fim).all(|b| nivel(e, b) == valor) {
            0
        } else {
            valor
        };
        self.editar(|s| {
            let e = s
                .slide_mut()
                .elementos
                .iter_mut()
                .find(|e| e.id == id)
                .unwrap();
            definir_nivel(e, inicio, fim, valor);
            Ok((vec![s.atual], false))
        })
    }

    /// O texto digitado na caixa `id`, com o trecho recém-escrito `inicio..fim` no nível
    /// `valor` (x² ligado antes de digitar). Mesmo passo de desfazer que `texto_definir`.
    pub fn texto_definir_nivelado(&mut self, id: u64, texto: &str, inicio: usize, fim: usize, valor: i8) -> Result<Mudanca, String> {
        if inicio >= fim || fim > texto.len() || !texto.is_char_boundary(inicio) || !texto.is_char_boundary(fim) || !(-1..=1).contains(&valor) {
            return self.texto_definir(id, texto);
        }
        let atual = self.slide().elementos.iter().find(|e| e.id == id).ok_or("Esse elemento não está no slide.")?;
        if !atual.tipo.aceita_texto() {
            return Err("Esse elemento não leva texto.".into());
        }
        self.editar(|s| {
            if let Some(e) = s.slide_mut().elementos.iter_mut().find(|e| e.id == id) {
                atualizar_indices(e, texto);
                e.texto = texto.into();
                definir_nivel(e, inicio, fim, valor);
            }
            Ok((vec![s.atual], false))
        })
    }
    pub fn texto_espacamento(&mut self, valor: f32) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        if !valor.is_finite() || !(-5.0..=20.0).contains(&valor) {
            return Err("Espaçamento entre letras deve ficar entre -5 e 20 pt.".into());
        }
        self.editar(|s| {
            for e in s.selecionados_mut() {
                if e.tipo.aceita_texto() {
                    e.espacamento_letras = valor;
                }
            }
            Ok((vec![s.atual], false))
        })
    }
}

/// Os modos de "Maiúsculas e minúsculas" da aba Início.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Caixa {
    /// Primeira letra de cada frase maiúscula, o resto minúsculo.
    Frase,
    Minusculas,
    Maiusculas,
    /// Primeira letra de cada palavra maiúscula.
    CadaPalavra,
    /// Inverte maiúsculas e minúsculas.
    Alternar,
}

impl Caixa {
    pub const TODAS: [Caixa; 5] = [Caixa::Frase, Caixa::Minusculas, Caixa::Maiusculas, Caixa::CadaPalavra, Caixa::Alternar];

    pub fn rotulo(self) -> &'static str {
        match self {
            Caixa::Frase => "Primeira letra da frase",
            Caixa::Minusculas => "minúsculas",
            Caixa::Maiusculas => "MAIÚSCULAS",
            Caixa::CadaPalavra => "Cada Palavra Em Maiúscula",
            Caixa::Alternar => "aLTERNAR mAIÚSCULAS",
        }
    }
}

/// Aplica `caixa` a `texto[inicio..fim]`. Devolve o texto novo e, para cada byte
/// de fronteira de caractere do texto antigo, o byte correspondente no novo
/// (algumas letras mudam de tamanho: "ß" vira "SS").
fn trocar_caixa(texto: &str, inicio: usize, fim: usize, caixa: Caixa) -> (String, Vec<(usize, usize)>) {
    let mut saida = String::with_capacity(texto.len());
    let mut mapa = Vec::new();
    // O começo do trecho conta como começo de frase/palavra, como no PowerPoint.
    let mut inicio_frase = true;
    let mut inicio_palavra = true;
    for (i, c) in texto.char_indices() {
        mapa.push((i, saida.len()));
        if i < inicio || i >= fim {
            saida.push(c);
            continue;
        }
        let maiuscula = match caixa {
            Caixa::Maiusculas => Some(true),
            Caixa::Minusculas => Some(false),
            Caixa::Frase => Some(inicio_frase && c.is_alphabetic()),
            Caixa::CadaPalavra => Some(inicio_palavra && c.is_alphabetic()),
            Caixa::Alternar => if c.is_uppercase() { Some(false) } else if c.is_lowercase() { Some(true) } else { None },
        };
        match maiuscula {
            Some(true) => saida.extend(c.to_uppercase()),
            Some(false) => saida.extend(c.to_lowercase()),
            None => saida.push(c),
        }
        if c.is_alphanumeric() {
            inicio_frase = false;
            inicio_palavra = false;
        } else if matches!(c, '.' | '!' | '?' | '\n') {
            inicio_frase = true;
            inicio_palavra = true;
        } else if c.is_whitespace() || matches!(c, '-' | '/' | '(' | '"' | '\'' | '“' | '«') {
            inicio_palavra = true;
        }
    }
    mapa.push((texto.len(), saida.len()));
    (saida, mapa)
}

/// O byte do texto novo que corresponde a `byte` do antigo (mapa de [`trocar_caixa`]/[`recuar`]).
pub fn traduzir(mapa: &[(usize, usize)], byte: usize) -> usize {
    match mapa.binary_search_by_key(&byte, |(v, _)| *v) {
        Ok(i) => mapa[i].1,
        // Dentro de um caractere (não deveria acontecer): o começo dele.
        Err(i) => mapa[i.saturating_sub(1)].1,
    }
}

/// Põe `inicio..fim` no nível `valor` (0 = normal), recortando os trechos vizinhos.
fn definir_nivel(e: &mut Elemento, inicio: usize, fim: usize, valor: i8) {
    let mut indices = Vec::new();
    for i in &e.indices_texto {
        if i.fim <= inicio || i.inicio >= fim {
            indices.push(i.clone());
        } else {
            if i.inicio < inicio {
                indices.push(Indice {
                    inicio: i.inicio,
                    fim: inicio,
                    nivel: i.nivel,
                });
            }
            if i.fim > fim {
                indices.push(Indice {
                    inicio: fim,
                    fim: i.fim,
                    nivel: i.nivel,
                });
            }
        }
    }
    if valor != 0 {
        indices.push(Indice {
            inicio,
            fim,
            nivel: valor,
        });
    }
    indices.sort_by_key(|i| i.inicio);
    // Trechos colados do mesmo nível viram um só (digitar letra a letra em x²).
    let mut juntos: Vec<Indice> = Vec::new();
    for i in indices {
        match juntos.last_mut() {
            Some(u) if u.fim == i.inicio && u.nivel == i.nivel => u.fim = i.fim,
            _ => juntos.push(i),
        }
    }
    e.indices_texto = juntos;
}

/// O nível do caractere que termina em `byte` (o que está logo antes do cursor).
pub fn nivel_antes(e: &Elemento, byte: usize) -> i8 {
    e.texto.get(..byte).and_then(|t| t.char_indices().next_back()).map_or(0, |(i, _)| nivel(e, i))
}

fn trocar_texto(e: &mut Elemento, novo: String, mapa: &[(usize, usize)]) {
    for i in &mut e.indices_texto {
        i.inicio = traduzir(mapa, i.inicio);
        i.fim = traduzir(mapa, i.fim);
    }
    e.indices_texto.retain(|i| i.inicio < i.fim);
    e.texto = novo;
}

/// O recuo de um nível de lista.
pub const RECUO: &str = "    ";

/// Aumenta (`mais`) ou diminui um nível o recuo das linhas que tocam `inicio..fim`
/// (linhas vazias ficam como estão). Devolve o texto novo e o mapa de bytes.
pub fn recuar(texto: &str, inicio: usize, fim: usize, mais: bool) -> (String, Vec<(usize, usize)>) {
    let mut saida = String::with_capacity(texto.len() + 16);
    let mut mapa = Vec::new();
    let mut pos = 0;
    for linha in texto.split_inclusive('\n') {
        let conteudo = linha.trim_end_matches('\n');
        let toca = pos <= fim.max(inicio) && pos + conteudo.len() >= inicio;
        let tirar = if !toca || conteudo.trim().is_empty() {
            0
        } else if mais {
            saida.push_str(RECUO);
            0
        } else if conteudo.starts_with('\t') {
            1
        } else {
            conteudo.len() - conteudo.trim_start_matches(' ').len().max(conteudo.len().saturating_sub(RECUO.len()))
        };
        for (i, c) in linha.char_indices() {
            // O que sai do começo aponta para onde a linha passa a começar.
            mapa.push((pos + i, saida.len()));
            if i >= tirar {
                saida.push(c);
            }
        }
        pos += linha.len();
    }
    mapa.push((texto.len(), saida.len()));
    (saida, mapa)
}

impl Sessao {
    /// Troca maiúsculas/minúsculas num trecho (`Some((id, inicio, fim))`, em edição)
    /// ou no texto inteiro dos elementos selecionados (`None`). Um passo de desfazer;
    /// os trechos de x²/x₂ acompanham a troca.
    pub fn texto_caixa(&mut self, trecho: Option<(u64, usize, usize)>, caixa: Caixa) -> Result<Mudanca, String> {
        let alvos: Vec<(u64, usize, usize)> = match trecho {
            Some((id, a, b)) => {
                let e = self.slide().elementos.iter().find(|e| e.id == id).ok_or("Selecione texto.")?;
                if a >= b || b > e.texto.len() || !e.texto.is_char_boundary(a) || !e.texto.is_char_boundary(b) {
                    return Err("Selecione o trecho que deve mudar de maiúsculas/minúsculas.".into());
                }
                vec![(id, a, b)]
            }
            None => {
                self.exigir_selecao()?;
                let sel = self.selecao.clone();
                self.slide().elementos.iter()
                    .filter(|e| sel.contains(&e.id) && e.tipo.aceita_texto() && !e.texto.is_empty())
                    .map(|e| (e.id, 0, e.texto.len()))
                    .collect()
            }
        };
        if alvos.is_empty() {
            return Err("Nenhum texto selecionado.".into());
        }
        self.editar(|s| {
            for (id, a, b) in alvos {
                let Some(e) = s.slide_mut().elementos.iter_mut().find(|e| e.id == id) else { continue };
                let (novo, mapa) = trocar_caixa(&e.texto, a, b, caixa);
                trocar_texto(e, novo, &mapa);
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Recuo das linhas (nível da lista): as do trecho em edição (`Some`) ou todas
    /// as da seleção (`None`). Um passo de desfazer.
    pub fn texto_recuo(&mut self, trecho: Option<(u64, usize, usize)>, mais: bool) -> Result<Mudanca, String> {
        let alvos: Vec<(u64, usize, usize)> = match trecho {
            Some(t) => vec![t],
            None => {
                self.exigir_selecao()?;
                let sel = self.selecao.clone();
                self.slide().elementos.iter()
                    .filter(|e| sel.contains(&e.id) && e.tipo.aceita_texto() && !e.texto.trim().is_empty())
                    .map(|e| (e.id, 0, e.texto.len()))
                    .collect()
            }
        };
        if alvos.is_empty() {
            return Err("Nenhum texto selecionado.".into());
        }
        let muda = alvos.iter().any(|&(id, a, b)| {
            self.slide().elementos.iter().find(|e| e.id == id).is_some_and(|e| recuar(&e.texto, a, b, mais).0 != e.texto)
        });
        if !muda {
            return Err("O texto já está sem recuo.".into());
        }
        self.editar(|s| {
            for (id, a, b) in alvos {
                let Some(e) = s.slide_mut().elementos.iter_mut().find(|e| e.id == id) else { continue };
                let (novo, mapa) = recuar(&e.texto, a, b, mais);
                trocar_texto(e, novo, &mapa);
            }
            Ok((vec![s.atual], false))
        })
    }
}

pub fn validar(e: &Elemento) -> Result<(), String> {
    if !e.espacamento_letras.is_finite() || !(-5.0..=20.0).contains(&e.espacamento_letras) {
        return Err("Espaçamento entre letras inválido.".into());
    }
    let mut fim = 0;
    for i in &e.indices_texto {
        if i.inicio < fim
            || i.inicio >= i.fim
            || i.fim > e.texto.len()
            || !e.texto.is_char_boundary(i.inicio)
            || !e.texto.is_char_boundary(i.fim)
            || ![-1, 1].contains(&i.nivel)
        {
            return Err("Trechos de sobrescrito/subscrito inválidos.".into());
        }
        fim = i.fim;
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::modelo::Tipo;
    fn sessao() -> (Sessao, u64) {
        let mut s = Sessao::default();
        s.elemento_inserir(Tipo::Retangulo, None, None).unwrap();
        let id = s.slide().elementos.last().unwrap().id;
        s.texto_definir(id, "x2 + H2O").unwrap();
        (s, id)
    }
    #[test]
    fn indices_independentes_e_desfazer() {
        let (mut s, id) = sessao();
        s.texto_indice(id, 1, 2, 1).unwrap();
        s.texto_indice(id, 6, 7, -1).unwrap();
        let e = s.slide().elementos.last().unwrap();
        assert_eq!(e.texto, "x2 + H2O");
        assert_eq!(nivel(e, 1), 1);
        assert_eq!(nivel(e, 6), -1);
        s.desfazer().unwrap();
        assert_eq!(nivel(s.slide().elementos.last().unwrap(), 6), 0);
        s.refazer().unwrap();
        assert_eq!(nivel(s.slide().elementos.last().unwrap(), 6), -1);
    }
    #[test]
    fn editar_preserva_desloca_e_remove_trechos() {
        let (mut s, id) = sessao();
        s.texto_indice(id, 1, 2, 1).unwrap();
        s.texto_definir(id, "Olá x2 + H2O").unwrap();
        let e = s.slide().elementos.last().unwrap();
        assert_eq!(nivel(e, "Olá x".len()), 1);
        s.texto_definir(id, "Olá x + H2O").unwrap();
        assert!(s.slide().elementos.last().unwrap().indices_texto.is_empty());
    }
    #[test]
    fn limites_utf8_e_espacamento() {
        let (mut s, id) = sessao();
        s.texto_definir(id, "ação").unwrap();
        assert!(s.texto_indice(id, 2, 3, 1).is_err());
        assert!(s.texto_espacamento(f32::NAN).is_err());
        assert!(s.texto_espacamento(21.0).is_err());
        s.texto_espacamento(2.0).unwrap();
        assert_eq!(s.slide().elementos.last().unwrap().espacamento_letras, 2.0);
    }
    #[test]
    fn layout_resolve_tamanhos_e_posicoes() {
        let (mut s, id) = sessao();
        s.texto_indice(id, 1, 2, 1).unwrap();
        s.texto_indice(id, 6, 7, -1).unwrap();
        s.texto_espacamento(2.0).unwrap();
        let q = s.quadro(s.atual(), None, false).unwrap().unwrap();
        let e = q.elementos.iter().find(|e| e.id == id).unwrap();
        let g = &e.linhas[0].glifos;
        assert!(g[1].tamanho < g[0].tamanho);
        assert!(g[1].y < g[0].y);
        assert!(g[6].y > g[5].y);
        assert!(g.windows(2).all(|p| p[1].x > p[0].x));
    }

    #[test]
    fn caixa_modos_e_trecho() {
        let t = "olá MUNDO. tudo bem?";
        let f = |c| trocar_caixa(t, 0, t.len(), c).0;
        assert_eq!(f(Caixa::Maiusculas), "OLÁ MUNDO. TUDO BEM?");
        assert_eq!(f(Caixa::Minusculas), "olá mundo. tudo bem?");
        assert_eq!(f(Caixa::Frase), "Olá mundo. Tudo bem?");
        assert_eq!(f(Caixa::CadaPalavra), "Olá Mundo. Tudo Bem?");
        assert_eq!(f(Caixa::Alternar), "OLÁ mundo. TUDO BEM?");
        assert_eq!(trocar_caixa(t, 5, 10, Caixa::Minusculas).0, "olá mundo. tudo bem?");
    }
    #[test]
    fn caixa_desfaz_e_preserva_indices() {
        let (mut s, id) = sessao();
        s.texto_definir(id, "straße x2").unwrap();
        let x = "straße x".len();
        s.texto_indice(id, x, x + 1, 1).unwrap();
        s.selecionar_por_id(id);
        s.texto_caixa(None, Caixa::Maiusculas).unwrap();
        let e = s.slide().elementos.last().unwrap();
        assert_eq!(e.texto, "STRASSE X2");
        assert_eq!(nivel(e, "STRASSE X".len()), 1);
        validar(e).unwrap();
        s.desfazer().unwrap();
        assert_eq!(s.slide().elementos.last().unwrap().texto, "straße x2");
        assert!(s.texto_caixa(Some((id, 2, 2)), Caixa::Maiusculas).is_err());
    }
    #[test]
    fn recuo_por_linha_e_trecho() {
        let t = "• um\n• dois\n\n• três";
        let (mais, _) = recuar(t, 0, t.len(), true);
        assert_eq!(mais, "    • um\n    • dois\n\n    • três");
        let (menos, _) = recuar(&mais, 0, mais.len(), false);
        assert_eq!(menos, t);
        // Só a linha onde está o cursor.
        let c = t.find("dois").unwrap();
        assert_eq!(recuar(t, c, c, true).0, "• um\n    • dois\n\n• três");
        assert_eq!(recuar("  a", 0, 3, false).0, "a");
    }
    #[test]
    fn recuo_desfaz_e_preserva_indices() {
        let (mut s, id) = sessao();
        s.texto_definir(id, "a\nx2").unwrap();
        s.texto_indice(id, 3, 4, 1).unwrap();
        s.selecionar_por_id(id);
        s.texto_recuo(None, true).unwrap();
        let e = s.slide().elementos.last().unwrap();
        assert_eq!(e.texto, "    a\n    x2");
        assert_eq!(nivel(e, "    a\n    x".len()), 1);
        assert!(s.texto_recuo(None, false).is_ok());
        assert!(s.texto_recuo(None, false).is_err());
        s.desfazer().unwrap();
        assert_eq!(s.slide().elementos.last().unwrap().texto, "    a\n    x2");
    }
}
