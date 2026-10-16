//! A largura do texto, para quebrar as linhas dentro da caixa. **Aproximada**: uma
//! tabela de larguras por classe de letra (estreita, comum, larga) da Liberation Sans,
//! em milésimos do corpo. O FeatherDocs mede com as métricas reais da fonte; trazer o
//! mesmo para cá (um crate comum de medida) é o próximo passo — ver `CLAUDE.md`.

/// A largura de um caractere, em milésimos do tamanho da letra.
fn largura_do_caractere(c: char, negrito: bool) -> f32 {
    let base = match c {
        ' ' => 278.0,
        'i' | 'j' | 'l' | '.' | ',' | ':' | ';' | '\'' | '!' | '|' | 'í' | 'ì' | 'ï' => 222.0,
        'f' | 't' | 'r' | 'I' | '(' | ')' | '[' | ']' | '-' | '/' => 333.0,
        'm' | 'w' | 'M' | 'W' | '@' => 833.0,
        'A'..='Z' | 'Á' | 'À' | 'Â' | 'Ã' | 'É' | 'Ê' | 'Í' | 'Ó' | 'Ô' | 'Õ' | 'Ú' | 'Ç' => 667.0,
        '0'..='9' => 556.0,
        _ => 556.0,
    };
    if negrito {
        base * 1.06
    } else {
        base
    }
}

/// A largura de `texto` em pontos, no `tamanho` dado.
pub fn largura(texto: &str, tamanho: f32, negrito: bool) -> f32 {
    texto.chars().map(|c| largura_do_caractere(c, negrito)).sum::<f32>() * tamanho / 1000.0
}

/// Quebra um parágrafo em linhas que caibam em `maximo` pontos (por palavra; a palavra
/// maior que a linha é cortada por letra).
pub fn quebrar(paragrafo: &str, tamanho: f32, negrito: bool, maximo: f32) -> Vec<String> {
    let mut linhas = Vec::new();
    let mut atual = String::new();
    for palavra in paragrafo.split(' ') {
        let tentativa = if atual.is_empty() { palavra.to_string() } else { format!("{atual} {palavra}") };
        if largura(&tentativa, tamanho, negrito) <= maximo || atual.is_empty() && largura(palavra, tamanho, negrito) <= maximo {
            atual = tentativa;
            continue;
        }
        if !atual.is_empty() {
            linhas.push(std::mem::take(&mut atual));
        }
        atual = cortar_palavra(palavra, tamanho, negrito, maximo, &mut linhas);
    }
    linhas.push(atual);
    linhas
}

/// Corta uma palavra comprida em pedaços de uma linha; devolve o resto (o que sobrou na última).
fn cortar_palavra(palavra: &str, tamanho: f32, negrito: bool, maximo: f32, linhas: &mut Vec<String>) -> String {
    let mut pedaco = String::new();
    for c in palavra.chars() {
        pedaco.push(c);
        if largura(&pedaco, tamanho, negrito) > maximo && pedaco.chars().count() > 1 {
            pedaco.pop();
            linhas.push(std::mem::take(&mut pedaco));
            pedaco.push(c);
        }
    }
    pedaco
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn quebra_por_palavra_e_mantem_as_que_cabem() {
        let linhas = quebrar("um dois tres quatro", 20.0, false, 60.0);
        assert!(linhas.len() >= 2, "{linhas:?}");
        assert!(linhas.iter().all(|l| largura(l, 20.0, false) <= 60.0 || !l.contains(' ')));
        assert_eq!(linhas.join(" "), "um dois tres quatro");
    }

    #[test]
    fn palavra_maior_que_a_caixa_e_cortada() {
        let linhas = quebrar("paralelepipedo", 20.0, false, 40.0);
        assert!(linhas.len() > 1);
        assert_eq!(linhas.concat(), "paralelepipedo");
    }

    #[test]
    fn paragrafo_vazio_da_uma_linha_vazia() {
        assert_eq!(quebrar("", 18.0, false, 100.0), vec![String::new()]);
    }
}
