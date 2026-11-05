//! A largura do texto, para quebrar as linhas dentro da caixa, com as métricas (AFM) reais
//! das fontes que a tela e o PDF usam: Helvetica = Liberation Sans/Arial (a fonte padrão das
//! caixas e a do PDF) e Times = Liberation Serif. As tabelas vêm do FeatherDocs (`medida.rs`,
//! copiadas pela regra 3 da suíte). shortcut: outra família (Calibri vinda de um .pptx) é medida
//! como a mais parecida das duas; acento usa a largura da letra-base; fora do Latin-1, 556.
//! Medir com a fonte exata da tela pede o medidor do pelicano no motor — trocar só este arquivo.

/// De `' '` (32) a `'~'` (126), em milésimos de em.
const REGULAR: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, // espaço ! " # $ % & ' ( ) * + , - . /
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, // 0-9 : ; < = > ?
    1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, // @ A-O
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556, // P-Z [ \ ] ^ _
    333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, // ` a-o
    556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584, // p-z { | } ~
];

const NEGRITADO: [u16; 95] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278, //
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 333, 333, 584, 584, 584, 611, //
    975, 722, 722, 722, 722, 667, 611, 778, 722, 278, 556, 722, 611, 833, 722, 778, //
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 333, 278, 333, 584, 556, //
    333, 556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556, 278, 889, 611, 611, //
    611, 611, 389, 556, 333, 611, 556, 778, 556, 556, 500, 389, 280, 389, 584, //
];

/// Liberation Serif (as métricas do Times): uma tabela por face, porque no Times o itálico e o negrito têm larguras próprias.
const SERIFA_REGULAR: [u16; 95] = [
    250, 333, 408, 500, 500, 833, 778, 180, 333, 333, 500, 564, 250, 333, 250, 278,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 278, 278, 564, 564, 564, 444,
    921, 722, 667, 667, 722, 611, 556, 722, 722, 333, 389, 722, 611, 889, 722, 722,
    556, 722, 667, 556, 611, 722, 722, 944, 722, 722, 611, 333, 278, 333, 469, 500,
    333, 444, 500, 444, 500, 444, 333, 500, 500, 278, 278, 500, 278, 778, 500, 500,
    500, 500, 333, 389, 278, 500, 500, 722, 500, 500, 444, 480, 200, 480, 541,
];

const SERIFA_NEGRITO: [u16; 95] = [
    250, 333, 555, 500, 500, 1000, 833, 278, 333, 333, 500, 570, 250, 333, 250, 278,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 333, 333, 570, 570, 570, 500,
    930, 722, 667, 722, 722, 667, 611, 778, 778, 389, 500, 778, 667, 944, 722, 778,
    611, 778, 722, 556, 667, 722, 722, 1000, 722, 722, 667, 333, 278, 333, 581, 500,
    333, 500, 556, 444, 556, 444, 333, 500, 556, 278, 333, 556, 278, 833, 556, 500,
    556, 556, 444, 389, 333, 556, 500, 722, 500, 500, 444, 394, 220, 394, 520,
];

const SERIFA_ITALICO: [u16; 95] = [
    250, 333, 420, 500, 500, 833, 778, 214, 333, 333, 500, 675, 250, 333, 250, 278,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 333, 333, 675, 675, 675, 500,
    920, 611, 611, 667, 722, 611, 611, 722, 722, 333, 444, 667, 556, 833, 667, 722,
    611, 722, 611, 500, 556, 722, 611, 833, 611, 556, 556, 389, 278, 389, 422, 500,
    333, 500, 500, 444, 500, 444, 278, 500, 500, 278, 278, 444, 278, 722, 500, 500,
    500, 500, 389, 389, 278, 500, 444, 667, 444, 444, 389, 400, 275, 400, 541,
];

const SERIFA_NEGRITO_ITALICO: [u16; 95] = [
    250, 389, 555, 500, 500, 833, 778, 278, 333, 333, 500, 570, 250, 333, 250, 278,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 333, 333, 570, 570, 570, 500,
    832, 667, 667, 667, 722, 667, 667, 722, 778, 389, 500, 667, 611, 889, 722, 722,
    611, 722, 667, 556, 611, 722, 667, 889, 667, 611, 611, 333, 278, 333, 570, 500,
    333, 500, 500, 444, 500, 444, 333, 500, 556, 278, 278, 500, 278, 778, 556, 500,
    500, 500, 389, 389, 278, 556, 444, 667, 500, 444, 389, 348, 220, 348, 570,
];

/// Largura de cada caractere numa fonte monoespaçada, em milésimos de em.
const MONO: f32 = 600.0;

/// A letra sem acento que dá a largura de um caractere acentuado.
fn base_do_acento(c: char) -> char {
    match c {
        'À'..='Å' => 'A',
        'Ç' => 'C',
        'È'..='Ë' => 'E',
        'Ì'..='Ï' => 'I',
        'Ñ' => 'N',
        'Ò'..='Ö' | 'Ø' => 'O',
        'Ù'..='Ü' => 'U',
        'Ý' => 'Y',
        'à'..='å' => 'a',
        'ç' => 'c',
        'è'..='ë' => 'e',
        'ì'..='ï' => 'i',
        'ñ' => 'n',
        'ò'..='ö' | 'ø' => 'o',
        'ù'..='ü' => 'u',
        'ý' | 'ÿ' => 'y',
        _ => c,
    }
}

/// Que tabela uma família usa.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Familia {
    Sans,
    Serifa,
    Mono,
}

fn familia(fonte: &str) -> Familia {
    let f = fonte.to_lowercase();
    if ["mono", "courier", "consolas", "menlo", "code"].iter().any(|n| f.contains(n)) {
        Familia::Mono
    } else if ["serif", "times", "georgia", "cambria", "garamond", "palatino", "antiqua", "book", "baskerville", "caslon"].iter().any(|n| f.contains(n)) && !f.contains("sans") {
        Familia::Serifa
    } else {
        Familia::Sans
    }
}

/// A largura de um caractere, em milésimos do tamanho da letra.
fn largura_do_caractere(c: char, negrito: bool, italico: bool, fam: Familia) -> f32 {
    if fam == Familia::Mono {
        return MONO;
    }
    let tabela = match (fam, negrito, italico) {
        (Familia::Serifa, false, false) => &SERIFA_REGULAR,
        (Familia::Serifa, true, false) => &SERIFA_NEGRITO,
        (Familia::Serifa, false, true) => &SERIFA_ITALICO,
        (Familia::Serifa, true, true) => &SERIFA_NEGRITO_ITALICO,
        (_, true, _) => &NEGRITADO,
        _ => &REGULAR,
    };
    f32::from(match base_do_acento(c) {
        c @ ' '..='~' => tabela[c as usize - 32],
        '\u{a0}' => 278,
        '–' => 556,
        '—' => 1000,
        '•' => 350,
        '’' | '‘' => 222,
        '“' | '”' => 333,
        '…' => 1000,
        _ => 556,
    })
}

/// A largura de `texto` em pontos, no `tamanho` dado, na fonte padrão (Liberation Sans).
pub fn largura(texto: &str, tamanho: f32, negrito: bool) -> f32 {
    largura_na_fonte(texto, tamanho, negrito, false, "")
}

/// A largura de `texto` em pontos, na fonte `fonte` (o nome da família), negrito e itálico.
pub fn largura_na_fonte(texto: &str, tamanho: f32, negrito: bool, italico: bool, fonte: &str) -> f32 {
    let fam = familia(fonte);
    texto.chars().map(|c| largura_do_caractere(c, negrito, italico, fam)).sum::<f32>() * tamanho / 1000.0
}

/// Quebra um parágrafo em linhas que caibam em `maximo` pontos, na fonte padrão.
pub fn quebrar(paragrafo: &str, tamanho: f32, negrito: bool, maximo: f32) -> Vec<String> {
    quebrar_na_fonte(paragrafo, tamanho, negrito, false, "", maximo)
}

/// Quebra um parágrafo em linhas que caibam em `maximo` pontos (por palavra; a palavra
/// maior que a linha é cortada por letra), medindo na fonte dada.
pub fn quebrar_na_fonte(paragrafo: &str, tamanho: f32, negrito: bool, italico: bool, fonte: &str, maximo: f32) -> Vec<String> {
    let medir = |t: &str| largura_na_fonte(t, tamanho, negrito, italico, fonte);
    let mut linhas = Vec::new();
    let mut atual = String::new();
    for palavra in paragrafo.split(' ') {
        let tentativa = if atual.is_empty() { palavra.to_string() } else { format!("{atual} {palavra}") };
        if medir(&tentativa) <= maximo || atual.is_empty() && medir(palavra) <= maximo {
            atual = tentativa;
            continue;
        }
        if !atual.is_empty() {
            linhas.push(std::mem::take(&mut atual));
        }
        // A palavra comprida vira pedaços de uma linha; o resto fica na linha atual.
        let mut pedaco = String::new();
        for c in palavra.chars() {
            pedaco.push(c);
            if medir(&pedaco) > maximo && pedaco.chars().count() > 1 {
                pedaco.pop();
                linhas.push(std::mem::take(&mut pedaco));
                pedaco.push(c);
            }
        }
        atual = pedaco;
    }
    linhas.push(atual);
    linhas
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
    fn metricas_reais_da_helvetica_e_da_times() {
        // "Hello" em Helvetica 10 pt: (722+556+222+222+556)/100
        assert!((largura("Hello", 10.0, false) - 22.78).abs() < 0.01);
        assert!((largura_na_fonte("Hello", 10.0, false, false, "Liberation Serif") - 22.22).abs() < 0.01);
        assert_eq!(largura_na_fonte("ção", 10.0, false, false, ""), largura("cao", 10.0, false), "acento = letra-base");
        assert_eq!(largura_na_fonte("iiii", 10.0, false, false, "DejaVu Sans Mono"), 24.0);
        assert_eq!(familia("Liberation Sans"), Familia::Sans);
        assert_eq!(familia("PT Serif"), Familia::Serifa);
        assert_eq!(familia("Noto Sans Serif"), Familia::Sans, "\"sans\" vence");
    }

    #[test]
    fn paragrafo_vazio_da_uma_linha_vazia() {
        assert_eq!(quebrar("", 18.0, false, 100.0), vec![String::new()]);
    }
}
