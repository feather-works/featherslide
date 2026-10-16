//! O cursor de escrita (a barrinha `|`) dentro do texto de uma caixa: onde ele está, como anda
//! (letra, palavra, linha, para cima e para baixo), onde o clique do mouse o põe e como se
//! insere e apaga ao redor dele. A posição é um deslocamento em **bytes** do texto, sempre num
//! limite de letra. A interface só pinta a barrinha onde o motor diz e repassa as teclas.

use crate::medida;
use crate::quadro::LinhaDeTexto;

/// Para onde o cursor anda.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Movimento {
    Esquerda,
    Direita,
    PalavraEsquerda,
    PalavraDireita,
    InicioDaLinha,
    FimDaLinha,
    InicioDoTexto,
    FimDoTexto,
    Cima,
    Baixo,
}

/// O texto já quebrado em linhas, como o quadro o desenha.
#[derive(Clone, Copy)]
pub struct Disposicao<'a> {
    pub linhas: &'a [LinhaDeTexto],
    pub tamanho: f32,
    pub negrito: bool,
}

/// Leva `c` ao limite de letra mais próximo para trás, dentro do texto.
pub fn limitar(t: &str, c: usize) -> usize {
    let mut c = c.min(t.len());
    while !t.is_char_boundary(c) {
        c -= 1;
    }
    c
}

/// Insere `novo` no cursor e o põe depois do que entrou.
pub fn inserir(t: &mut String, c: &mut usize, novo: &str) {
    *c = limitar(t, *c);
    t.insert_str(*c, novo);
    *c += novo.len();
}

/// A seleção (ordenada, sem estar vazia) entre a âncora e o cursor; `None` se não há âncora ou
/// ela está no mesmo ponto do cursor.
pub fn intervalo(t: &str, ancora: Option<usize>, cursor: usize) -> Option<(usize, usize)> {
    let (a, c) = (limitar(t, ancora?), limitar(t, cursor));
    (a != c).then(|| (a.min(c), a.max(c)))
}

/// A palavra que o duplo clique em `c` seleciona (letras e dígitos); em cima de espaço ou
/// pontuação, a sequência de iguais àquele caractere.
pub fn palavra_em(t: &str, c: usize) -> (usize, usize) {
    let c = limitar(t, c);
    let depois = t[c..].chars().next();
    let antes = t[..c].chars().next_back();
    let alvo = depois.filter(|&ch| eh_palavra(ch)).or(antes.filter(|&ch| eh_palavra(ch))).or(depois).or(antes);
    let Some(alvo) = alvo else { return (c, c) };
    let mesma = |ch: char| eh_palavra(ch) == eh_palavra(alvo) && (eh_palavra(alvo) || ch == alvo);
    let ini = t[..c].char_indices().rev().take_while(|&(_, ch)| mesma(ch)).last().map_or(c, |(i, _)| i);
    let fim = t[c..].char_indices().take_while(|&(_, ch)| mesma(ch)).last().map_or(c, |(i, ch)| c + i + ch.len_utf8());
    (ini, fim)
}

/// Apaga o trecho `[a, b)` e põe o cursor onde ele começava.
pub fn apagar_intervalo(t: &mut String, c: &mut usize, (a, b): (usize, usize)) {
    let (a, b) = (limitar(t, a), limitar(t, b));
    t.replace_range(a..b, "");
    *c = a;
}

/// Escreve `novo` no lugar da seleção (ou no cursor, se não há seleção).
pub fn substituir(t: &mut String, c: &mut usize, ancora: Option<usize>, novo: &str) {
    if let Some(sel) = intervalo(t, ancora, *c) {
        apagar_intervalo(t, c, sel);
    }
    inserir(t, c, novo);
}

/// Apaga a letra antes do cursor (Backspace). Diz se apagou algo.
pub fn apagar_antes(t: &mut String, c: &mut usize) -> bool {
    *c = limitar(t, *c);
    let Some(ch) = t[..*c].chars().next_back() else { return false };
    *c -= ch.len_utf8();
    t.remove(*c);
    true
}

/// Apaga a letra depois do cursor (Delete). Diz se apagou algo.
pub fn apagar_depois(t: &mut String, c: usize) -> bool {
    let c = limitar(t, c);
    if c >= t.len() {
        return false;
    }
    t.remove(c);
    true
}

fn eh_palavra(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn palavra_antes(t: &str, c: usize) -> usize {
    let mut i = c;
    let mut anterior = t[..i].chars().rev().peekable();
    while let Some(&ch) = anterior.peek() {
        if eh_palavra(ch) {
            break;
        }
        i -= ch.len_utf8();
        anterior.next();
    }
    while let Some(&ch) = anterior.peek() {
        if !eh_palavra(ch) {
            break;
        }
        i -= ch.len_utf8();
        anterior.next();
    }
    i
}

fn palavra_depois(t: &str, c: usize) -> usize {
    let mut i = c;
    let mut seguinte = t[c..].chars().peekable();
    while let Some(&ch) = seguinte.peek() {
        if eh_palavra(ch) {
            break;
        }
        i += ch.len_utf8();
        seguinte.next();
    }
    while let Some(&ch) = seguinte.peek() {
        if !eh_palavra(ch) {
            break;
        }
        i += ch.len_utf8();
        seguinte.next();
    }
    i
}

/// Onde cada linha desenhada começa no texto (em bytes). As linhas são pedaços do texto, na
/// ordem; o que fica entre elas (espaço da quebra, `\n`) não pertence a nenhuma.
fn inicios(t: &str, linhas: &[LinhaDeTexto]) -> Vec<usize> {
    let mut pos = 0;
    let mut v = Vec::with_capacity(linhas.len());
    for l in linhas {
        if l.texto.is_empty() {
            // Linha em branco: fica logo depois de uma quebra.
            if t[pos..].starts_with('\n') {
                pos += 1;
            }
            v.push(pos);
        } else if let Some(off) = t[pos..].find(&l.texto) {
            v.push(pos + off);
            pos += off + l.texto.len();
        } else {
            v.push(pos);
        }
    }
    v
}

/// A linha onde o cursor está: a última que começa até ele.
fn linha_do_cursor(inicios: &[usize], c: usize) -> usize {
    inicios.iter().rposition(|&i| i <= c).unwrap_or(0)
}

fn x_no_texto(l: &LinhaDeTexto, t: &str, inicio: usize, c: usize, d: &Disposicao) -> f32 {
    let fim = (inicio + l.texto.len()).min(t.len());
    let c = limitar(t, c.clamp(inicio, fim));
    if !l.glifos.is_empty() {
        let n=t[inicio..c].chars().count();
        return l.glifos.get(n).map_or(l.x+l.largura,|g|g.x);
    }
    l.x + medida::largura(&t[inicio..c], d.tamanho, d.negrito)
}

/// O deslocamento dentro da linha cuja posição horizontal é a mais próxima de `x`.
fn coluna_por_x(l: &LinhaDeTexto, t: &str, inicio: usize, x: f32, d: &Disposicao) -> usize {
    let fim = (inicio + l.texto.len()).min(t.len());
    let mut melhor = (f32::MAX, inicio);
    for (i, _) in l.texto.char_indices().chain([(l.texto.len(), ' ')]) {
        let pos = (inicio + i).min(fim);
        let dist = (x_no_texto(l, t, inicio, pos, d) - x).abs();
        if dist < melhor.0 {
            melhor = (dist, pos);
        }
    }
    melhor.1
}

/// Onde pintar a barrinha: `x`, a linha de base `y` e o índice da linha. `None` sem linhas.
pub fn ponto_do_cursor(t: &str, d: &Disposicao, c: usize) -> Option<(f32, f32, usize)> {
    if d.linhas.is_empty() {
        return None;
    }
    let ini = inicios(t, d.linhas);
    let i = linha_do_cursor(&ini, limitar(t, c));
    Some((x_no_texto(&d.linhas[i], t, ini[i], c, d), d.linhas[i].y, i))
}

/// Os segmentos desenhados do trecho `[ini, fim)` do texto: (x, linha de base, largura), um por
/// linha que o trecho atravessa. Serve para sublinhar uma frase que quebra de linha.
pub fn trechos(t: &str, d: &Disposicao, ini: usize, fim: usize) -> Vec<(f32, f32, f32)> {
    let inicios = inicios(t, d.linhas);
    let (ini, fim) = (limitar(t, ini), limitar(t, fim));
    let mut v = Vec::new();
    for (i, l) in d.linhas.iter().enumerate() {
        let (a, b) = (inicios[i].max(ini), (inicios[i] + l.texto.len()).min(fim));
        if a < b {
            let x0 = x_no_texto(l, t, inicios[i], a, d);
            v.push((x0, l.y, x_no_texto(l, t, inicios[i], b, d) - x0));
        }
    }
    v
}

/// Onde o clique em (`x`, `y`) (pontos do slide) põe o cursor.
pub fn cursor_no_ponto(t: &str, d: &Disposicao, x: f32, y: f32) -> usize {
    if d.linhas.is_empty() {
        return t.len();
    }
    let ini = inicios(t, d.linhas);
    let centro = |l: &LinhaDeTexto| l.y - d.tamanho * 0.3;
    let i = (0..d.linhas.len()).min_by(|&a, &b| (centro(&d.linhas[a]) - y).abs().total_cmp(&(centro(&d.linhas[b]) - y).abs())).unwrap_or(0);
    coluna_por_x(&d.linhas[i], t, ini[i], x, d)
}

/// O cursor depois de andar; `d` só é preciso para linha, cima e baixo.
pub fn mover(t: &str, c: usize, m: Movimento, d: Option<&Disposicao>) -> usize {
    let c = limitar(t, c);
    match m {
        Movimento::Esquerda => t[..c].chars().next_back().map_or(0, |ch| c - ch.len_utf8()),
        Movimento::Direita => t[c..].chars().next().map_or(t.len(), |ch| c + ch.len_utf8()),
        Movimento::PalavraEsquerda => palavra_antes(t, c),
        Movimento::PalavraDireita => palavra_depois(t, c),
        Movimento::InicioDoTexto => 0,
        Movimento::FimDoTexto => t.len(),
        Movimento::InicioDaLinha | Movimento::FimDaLinha | Movimento::Cima | Movimento::Baixo => {
            let Some(d) = d.filter(|d| !d.linhas.is_empty()) else {
                // Sem linhas desenhadas: o texto inteiro é uma linha só.
                return match m {
                    Movimento::InicioDaLinha | Movimento::Cima => 0,
                    _ => t.len(),
                };
            };
            let ini = inicios(t, d.linhas);
            let i = linha_do_cursor(&ini, c);
            let fim_de = |k: usize| (ini[k] + d.linhas[k].texto.len()).min(t.len());
            match m {
                Movimento::InicioDaLinha => ini[i],
                Movimento::FimDaLinha => fim_de(i),
                Movimento::Cima if i == 0 => 0,
                Movimento::Baixo if i + 1 >= d.linhas.len() => t.len(),
                _ => {
                    let alvo = if m == Movimento::Cima { i - 1 } else { i + 1 };
                    let x = x_no_texto(&d.linhas[i], t, ini[i], c, d);
                    coluna_por_x(&d.linhas[alvo], t, ini[alvo], x, d)
                }
            }
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn linha(texto: &str, y: f32) -> LinhaDeTexto {
        LinhaDeTexto { texto: texto.into(), x: 10.0, y, largura: medida::largura(texto, 20.0, false), glifos: Vec::new() }
    }

    fn disp(linhas: &[LinhaDeTexto]) -> Disposicao<'_> {
        Disposicao { linhas, tamanho: 20.0, negrito: false }
    }

    #[test]
    fn anda_por_letra_com_acento_e_apaga_no_meio() {
        let mut t = String::from("ação ok");
        let mut c = t.len();
        c = mover(&t, c, Movimento::Esquerda, None);
        c = mover(&t, c, Movimento::Esquerda, None);
        c = mover(&t, c, Movimento::Esquerda, None);
        assert_eq!(c, "ação".len());
        // Apagar antes do cursor tira o "o".
        assert!(apagar_antes(&mut t, &mut c));
        assert_eq!(t, "açã ok");
        inserir(&mut t, &mut c, "O");
        assert_eq!((t.as_str(), c), ("açãO ok", "açãO".len()));
        assert!(apagar_depois(&mut t, c));
        assert_eq!(t, "açãOok");
        assert_eq!(mover("ã", 1, Movimento::Direita, None), 2, "cursor no meio de uma letra volta ao limite");
    }

    #[test]
    fn trechos_quebram_por_linha() {
        let t = "uma frase longa demais";
        let linhas = [linha("uma frase", 30.0), linha("longa demais", 60.0)];
        let d = disp(&linhas);
        // "frase longa" atravessa a quebra: um pedaço em cada linha.
        let v = trechos(t, &d, 4, "uma frase longa".len());
        assert_eq!(v.len(), 2);
        assert_eq!((v[0].1, v[1].1), (30.0, 60.0));
        assert!(v[0].0 > 10.0 && v[1].0 == 10.0 && v[0].2 > 0.0 && v[1].2 > 0.0);
    }

    #[test]
    fn selecao_palavra_apagar_e_substituir() {
        let t = String::from("uma casa bonita");
        assert_eq!(intervalo(&t, None, 3), None);
        assert_eq!(intervalo(&t, Some(3), 3), None);
        assert_eq!(intervalo(&t, Some(8), 4), Some((4, 8)), "ordena, não importa o sentido");
        // Duplo clique: no meio da palavra, na ponta e no espaço.
        assert_eq!(palavra_em(&t, 6), (4, 8));
        assert_eq!(palavra_em(&t, 4), (4, 8));
        assert_eq!(palavra_em(&t, 8), (4, 8), "logo depois da palavra ainda pega ela");
        assert_eq!(palavra_em("ação  x", 7), (6, 8), "espaços seguidos");
        assert_eq!(palavra_em("ação", 2), (0, "ação".len()), "com acento");
        let mut s = t.clone();
        let mut c = 8;
        apagar_intervalo(&mut s, &mut c, (4, 8));
        assert_eq!((s.as_str(), c), ("uma  bonita", 4));
        let mut s = t.clone();
        let mut c = 8;
        substituir(&mut s, &mut c, Some(4), "casinha");
        assert_eq!((s.as_str(), c), ("uma casinha bonita", "uma casinha".len()));
        let mut s = String::from("ab");
        let mut c = 1;
        substituir(&mut s, &mut c, None, "X");
        assert_eq!((s.as_str(), c), ("aXb", 2));
    }

    #[test]
    fn palavras_e_extremos() {
        let t = "uma casa, bonita";
        assert_eq!(mover(t, t.len(), Movimento::PalavraEsquerda, None), "uma casa, ".len());
        assert_eq!(mover(t, 0, Movimento::PalavraDireita, None), 3);
        assert_eq!(mover(t, 4, Movimento::PalavraDireita, None), "uma casa".len());
        assert_eq!(mover(t, 5, Movimento::InicioDoTexto, None), 0);
        assert_eq!(mover(t, 5, Movimento::FimDoTexto, None), t.len());
    }

    #[test]
    fn sobe_desce_e_clica_em_texto_quebrado() {
        let t = "primeira linha\nsegunda\n\nquarta";
        let linhas = [linha("primeira linha", 30.0), linha("segunda", 60.0), linha("", 90.0), linha("quarta", 120.0)];
        let d = disp(&linhas);
        // Fim do texto está na linha 3; cima volta para a em branco, depois para "segunda".
        let c = mover(t, t.len(), Movimento::Cima, Some(&d));
        assert_eq!(c, "primeira linha\nsegunda\n".len(), "a linha em branco");
        let c = mover(t, c, Movimento::Cima, Some(&d));
        assert!(t[..c].ends_with("primeira linha\n") || t[..c].contains("segunda"), "{c}");
        assert_eq!(ponto_do_cursor(t, &d, 0).map(|p| p.2), Some(0));
        // Clique no começo e no fim da 2ª linha.
        assert_eq!(cursor_no_ponto(t, &d, 0.0, 58.0), "primeira linha\n".len());
        assert_eq!(cursor_no_ponto(t, &d, 900.0, 58.0), "primeira linha\nsegunda".len());
        // Home e End da linha.
        let meio = "primeira linha\nseg".len();
        assert_eq!(mover(t, meio, Movimento::InicioDaLinha, Some(&d)), "primeira linha\n".len());
        assert_eq!(mover(t, meio, Movimento::FimDaLinha, Some(&d)), "primeira linha\nsegunda".len());
        // Cima na primeira linha vai ao começo; baixo na última, ao fim.
        assert_eq!(mover(t, 3, Movimento::Cima, Some(&d)), 0);
        assert_eq!(mover(t, t.len() - 2, Movimento::Baixo, Some(&d)), t.len());
    }

    #[test]
    fn linha_quebrada_por_largura_conserva_o_espaco_da_quebra() {
        let t = "uma frase longa demais";
        let linhas = [linha("uma frase", 30.0), linha("longa demais", 60.0)];
        let d = disp(&linhas);
        // O espaço da quebra não é de nenhuma linha: o cursor depois dele fica no início da 2ª.
        assert_eq!(ponto_do_cursor(t, &d, "uma frase ".len()).map(|p| p.2), Some(1));
        assert_eq!(mover(t, "uma frase ".len(), Movimento::Cima, Some(&d)), 0);
    }
}
