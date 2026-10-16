//! Gramática por regras (crase, concordância de gênero, vírgula, repetição…). **Não é um corretor
//! gramatical completo**: o português precisa de análise morfológica que não existe em Rust puro.
//! Aqui ficam só regras de alta confiança, com listas curadas de palavras, para quase não errar
//! para o lado de marcar o que está certo. O que cada regra acha vem como um `Problema`, com a
//! troca sugerida; marcar e trocar é com quem chama.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Uma frase que provavelmente está errada, com a troca sugerida.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problema {
    /// O que se troca: bytes `[inicio, fim)` do texto, por `sugestao`.
    pub inicio: usize,
    pub fim: usize,
    pub sugestao: String,
    /// O que se sublinha (inclui as palavras em volta que explicam o erro).
    pub ancora: (usize, usize),
    pub mensagem: String,
    /// O nome curto da regra (serve de chave para "ignorar").
    pub regra: &'static str,
}

/// As palavras do texto, com os bytes em que estão: letras (com acento), dígitos e o `'`/`-` no meio.
fn palavras(t: &str) -> Vec<(usize, usize)> {
    let mut v = Vec::new();
    let mut atual: Option<usize> = None;
    let mut it = t.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        let dentro = c.is_alphanumeric() || (matches!(c, '\'' | '’' | '-') && atual.is_some() && it.peek().is_some_and(|&(_, p)| p.is_alphabetic()));
        match (dentro, atual) {
            (true, None) => atual = Some(i),
            (false, Some(ini)) => {
                v.push((ini, i));
                atual = None;
            }
            _ => {}
        }
    }
    if let Some(ini) = atual {
        v.push((ini, t.len()));
    }
    v
}

/// Só espaços e tabs entre duas palavras (sem pontuação nem quebra de linha)?
fn so_espaco(t: &str, de: usize, ate: usize) -> bool {
    de < ate && t[de..ate].chars().all(|c| c == ' ' || c == '\t' || c == '\u{a0}')
}

/// A palavra `nova` com a caixa (maiúscula inicial, tudo maiúsculo) da `modelo`.
fn com_a_caixa_de(modelo: &str, nova: &str) -> String {
    let maiusculas = modelo.chars().filter(|c| c.is_uppercase()).count();
    if maiusculas > 1 && maiusculas == modelo.chars().filter(|c| c.is_alphabetic()).count() {
        return nova.to_uppercase();
    }
    if modelo.chars().next().is_some_and(char::is_uppercase) {
        let mut c = nova.chars();
        return c.next().map(|p| p.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default();
    }
    nova.to_string()
}

const FEMININAS: &[&str] = &[
    "casa", "mesa", "porta", "janela", "escola", "empresa", "cidade", "pessoa", "vida", "água", "forma", "parte", "hora", "vez", "coisa", "área", "rua", "terra", "mão", "noite", "semana", "família", "história", "aula", "prova", "nota", "tela", "planilha", "tabela", "imagem", "página", "linha", "coluna", "cor", "fonte", "ideia", "equipe", "reunião", "data", "conta", "meta", "regra", "lei", "praia", "igreja", "festa", "universidade", "faculdade", "apresentação", "informação", "situação", "solução", "questão", "razão", "opinião", "mulher", "menina", "amiga", "professora", "irmã", "mãe", "filha", "viagem", "mensagem", "pergunta", "resposta", "loja", "cozinha", "sala", "parede", "cama", "chave", "luz", "paz", "voz", "flor", "dor", "árvore", "ponte", "rede", "máquina", "pasta", "caneta", "carta", "foto", "moto", "tarde", "manhã", "cadeira", "bolsa", "roupa", "comida", "bebida", "música", "tecnologia", "energia", "biblioteca", "academia", "padaria", "farmácia", "consulta", "praça", "missa", "feira",
];

const MASCULINAS: &[&str] = &[
    "livro", "carro", "dia", "mapa", "problema", "sistema", "tema", "programa", "planeta", "clima", "idioma", "poema", "drama", "cinema", "telefone", "computador", "arquivo", "caderno", "trabalho", "projeto", "slide", "texto", "gráfico", "filme", "homem", "menino", "amigo", "professor", "irmão", "pai", "filho", "pé", "olho", "dedo", "braço", "ano", "mês", "tempo", "lugar", "país", "mundo", "governo", "estudo", "resultado", "produto", "dado", "dados", "relatório", "documento", "pedido", "serviço", "valor", "preço", "exemplo", "jeito", "motivo", "grupo", "time", "jogo", "dinheiro", "banco", "hospital", "hotel", "aeroporto", "sol", "mar", "céu", "fogo", "vento", "café", "pão", "leite", "vinho", "motor", "sinal", "mercado", "supermercado", "mercadinho", "shopping", "parque", "centro", "bairro", "restaurante", "bar", "clube", "museu", "teatro", "estádio", "escritório", "colégio", "curso", "médico", "interior", "campo", "litoral", "exterior", "trabalho", "banco", "posto", "salão", "mercado",
];

/// Palavras femininas que terminam em «o» (para a regra «à + palavra em -o»).
const FEMININAS_EM_O: &[&str] = &["mão", "mãos", "foto", "fotos", "moto", "motos", "tribo", "tribos", "libido", "virago", "kodak", "nuvem"];

/// Depois de «a» fica sem artigo: «a pé», «a cavalo».
const LOCUCOES_COM_A: &[&str] = &["pé", "cavalo", "bordo", "passo", "seco", "olho", "pedido", "partir", "favor", "meio", "custo", "vista"];

/// Parece masculina: está na lista ou termina em «o» (menos as femininas em -o).
fn parece_masculina(p: &str) -> bool {
    let p = p.to_lowercase();
    eh_masculina(&p) || ((p.ends_with('o') || p.ends_with("os")) && p.chars().count() >= 4 && !FEMININAS_EM_O.contains(&p.as_str()) && !p.chars().any(|c| c.is_ascii_digit()))
}

/// Lugares e eventos que pedem crase depois de "ir a" ("vou à escola").
const DESTINOS: &[&str] = &["escola", "cidade", "praia", "igreja", "festa", "reunião", "universidade", "faculdade", "feira", "biblioteca", "academia", "empresa", "loja", "padaria", "farmácia", "consulta", "praça", "missa", "aula", "rua"];

const IR: &[&str] = &["vou", "vai", "vamos", "vão", "foi", "fui", "fomos", "foram", "irei", "iremos", "irá", "irão", "ia", "iam"];

/// Palavra masculina → feminina (o «a» fica de fora na volta: também é preposição) (artigos, contrações, demonstrativos, possessivos, indefinidos).
const PARES: &[(&str, &str)] = &[
    ("o", "a"), ("os", "as"), ("ao", "à"), ("aos", "às"), ("do", "da"), ("dos", "das"), ("no", "na"), ("nos", "nas"), ("pelo", "pela"), ("pelos", "pelas"),
    ("um", "uma"), ("uns", "umas"), ("num", "numa"), ("nuns", "numas"), ("este", "esta"), ("esse", "essa"), ("aquele", "aquela"), ("estes", "estas"), ("esses", "essas"),
    ("aqueles", "aquelas"), ("neste", "nesta"), ("nesse", "nessa"), ("naquele", "naquela"), ("deste", "desta"), ("desse", "dessa"), ("daquele", "daquela"),
    ("meu", "minha"), ("meus", "minhas"), ("teu", "tua"), ("seu", "sua"), ("seus", "suas"), ("nosso", "nossa"), ("algum", "alguma"), ("nenhum", "nenhuma"), ("outro", "outra"),
];

fn mapas() -> &'static (HashMap<&'static str, &'static str>, HashMap<&'static str, &'static str>) {
    static M: OnceLock<(HashMap<&str, &str>, HashMap<&str, &str>)> = OnceLock::new();
    M.get_or_init(|| (PARES.iter().copied().collect(), PARES.iter().map(|&(m, f)| (f, m)).filter(|&(f, _)| !matches!(f, "à" | "às" | "a")).collect()))
}

fn eh_feminina(p: &str) -> bool {
    let p = p.to_lowercase();
    FEMININAS.contains(&p.as_str()) || ((p.ends_with("ção") || p.ends_with("dade") || p.ends_with("tude")) && p.chars().count() > 5)
}

fn eh_masculina(p: &str) -> bool {
    MASCULINAS.contains(&p.to_lowercase().as_str())
}

fn problema(inicio: usize, fim: usize, sugestao: String, ancora: (usize, usize), mensagem: String, regra: &'static str) -> Problema {
    Problema { inicio, fim, sugestao, ancora, mensagem, regra }
}

/// Palavra repetida ("de de").
fn repetidas(t: &str, p: &[(usize, usize)], v: &mut Vec<Problema>) {
    for w in p.windows(2) {
        let (a, b) = (&t[w[0].0..w[0].1], &t[w[1].0..w[1].1]);
        if a.to_lowercase() == b.to_lowercase() && so_espaco(t, w[0].1, w[1].0) && !a.chars().any(|c| c.is_ascii_digit()) && a.chars().count() > 1 {
            v.push(problema(w[0].0, w[1].1, a.to_string(), (w[0].0, w[1].1), format!("Palavra repetida: «{a}»."), "repetida"));
        }
    }
}

/// Gênero: "ao casa" → "à casa"; "da livro" → "do livro"; "à carro" → "a carro" (sem crase).
fn concordancia(t: &str, p: &[(usize, usize)], v: &mut Vec<Problema>) {
    let (masc_para_fem, fem_para_masc) = mapas();
    for w in p.windows(2) {
        let (det, nome) = (&t[w[0].0..w[0].1], &t[w[1].0..w[1].1]);
        if !so_espaco(t, w[0].1, w[1].0) {
            continue;
        }
        let d = det.to_lowercase();
        let ancora = (w[0].0, w[1].1);
        if let (Some(f), true) = (masc_para_fem.get(d.as_str()), eh_feminina(nome)) {
            let msg = if d == "ao" || d == "aos" { format!("Crase: «{nome}» é feminino, use «{}».", com_a_caixa_de(det, f)) } else { format!("Concordância: «{nome}» é feminino.") };
            v.push(problema(w[0].0, w[0].1, com_a_caixa_de(det, f), ancora, msg, "genero"));
        } else if (d == "à" || d == "às") && parece_masculina(nome) {
            // «a pé», «a cavalo» ficam sem artigo; nos outros casos o certo é «ao», «aos».
            let sem = match (d == "à", LOCUCOES_COM_A.contains(&nome.to_lowercase().as_str())) {
                (true, true) => "a",
                (true, false) => "ao",
                (false, true) => "a",
                (false, false) => "aos",
            };
            let certo = com_a_caixa_de(det, sem);
            v.push(problema(w[0].0, w[0].1, certo.clone(), ancora, format!("Não há crase antes de palavra masculina: «{certo} {nome}»."), "crase_masculino"));
        } else if let (Some(m), true) = (fem_para_masc.get(d.as_str()), eh_masculina(nome)) {
            v.push(problema(w[0].0, w[0].1, com_a_caixa_de(det, m), ancora, format!("Concordância: «{nome}» é masculino."), "genero"));
        }
    }
}

/// Crase depois de "ir a": "vou a escola" → "vou à escola".
fn crase_depois_de_ir(t: &str, p: &[(usize, usize)], v: &mut Vec<Problema>) {
    for w in p.windows(3) {
        let (verbo, a, nome) = (t[w[0].0..w[0].1].to_lowercase(), &t[w[1].0..w[1].1], t[w[2].0..w[2].1].to_lowercase());
        if IR.contains(&verbo.as_str()) && a == "a" && DESTINOS.contains(&nome.as_str()) && so_espaco(t, w[0].1, w[1].0) && so_espaco(t, w[1].1, w[2].0) {
            v.push(problema(w[1].0, w[1].1, "à".into(), (w[0].0, w[2].1), format!("Crase: «ir a» + «{nome}» (feminino) leva crase."), "crase_ir"));
        }
    }
}

/// "para mim fazer" → "para eu fazer".
fn para_mim_infinitivo(t: &str, p: &[(usize, usize)], v: &mut Vec<Problema>) {
    for w in p.windows(3) {
        let (prep, mim, verbo) = (t[w[0].0..w[0].1].to_lowercase(), t[w[1].0..w[1].1].to_lowercase(), t[w[2].0..w[2].1].to_lowercase());
        let infinitivo = verbo.chars().count() >= 4 && (verbo.ends_with("ar") || verbo.ends_with("er") || verbo.ends_with("ir")) && !matches!(verbo.as_str(), "mulher" | "colher" | "lugar" | "mar");
        if matches!(prep.as_str(), "para" | "pra") && mim == "mim" && infinitivo && so_espaco(t, w[0].1, w[1].0) && so_espaco(t, w[1].1, w[2].0) {
            v.push(problema(w[1].0, w[1].1, com_a_caixa_de(&t[w[1].0..w[1].1], "eu"), (w[0].0, w[2].1), "Use «eu» antes de verbo: «para eu fazer».".into(), "mim_infinitivo"));
        }
    }
}

/// Expressões erradas muito comuns.
fn expressoes(t: &str, p: &[(usize, usize)], v: &mut Vec<Problema>) {
    for (n, &(i, f)) in p.iter().enumerate() {
        let palavra = t[i..f].to_lowercase();
        let proxima = |k: usize| p.get(n + k).map(|&(a, b)| (&t[a..b], a, b));
        if palavra == "houveram" {
            v.push(problema(i, f, com_a_caixa_de(&t[i..f], "houve"), (i, f), "«Haver» no sentido de «existir» não vai para o plural: «houve».".into(), "houveram"));
        } else if palavra == "a" {
            if let (Some((nivel, _, b)), Some((de, _, fim))) = (proxima(1), proxima(2)) {
                if nivel.eq_ignore_ascii_case("nível") && de.eq_ignore_ascii_case("de") && so_espaco(t, f, p[n + 1].0) {
                    v.push(problema(i, f, com_a_caixa_de(&t[i..f], "em"), (i, fim), "Prefira «em nível de» (ou «no nível de»).".into(), "a_nivel_de"));
                    let _ = b;
                }
            }
        } else if palavra == "afim" {
            if let Some((de, _, fim)) = proxima(1) {
                if de.eq_ignore_ascii_case("de") && so_espaco(t, f, p[n + 1].0) {
                    v.push(problema(i, f, com_a_caixa_de(&t[i..f], "a fim"), (i, fim), "Finalidade se escreve «a fim de».".into(), "afim_de"));
                }
            }
        }
    }
}

/// Vírgula antes de «mas», «porém», «contudo», «todavia», «entretanto».
fn virgula_antes_de_adversativa(t: &str, p: &[(usize, usize)], v: &mut Vec<Problema>) {
    for w in p.windows(2) {
        let adv = t[w[1].0..w[1].1].to_lowercase();
        if !matches!(adv.as_str(), "mas" | "porém" | "contudo" | "todavia" | "entretanto") || !so_espaco(t, w[0].1, w[1].0) {
            continue;
        }
        // «não só A mas também B» pede a vírgula só se a pessoa quiser.
        if t[w[1].1..].trim_start().to_lowercase().starts_with("também") {
            continue;
        }
        let original = &t[w[1].0..w[1].1];
        v.push(problema(w[0].1, w[1].1, format!(", {original}"), (w[0].0, w[1].1), format!("Use vírgula antes de «{adv}»."), "virgula_adversativa"));
    }
}

fn eh_abreviatura(palavra: &str) -> bool {
    let p = palavra.to_lowercase();
    palavra.chars().count() == 1 || p.chars().all(|c| c.is_ascii_digit()) || matches!(p.as_str(), "sr" | "sra" | "dr" | "dra" | "prof" | "profa" | "etc" | "ex" | "pág" | "av" | "obs" | "cap" | "fig" | "tel" | "vs" | "ltda" | "tb")
}

/// Espaço antes da pontuação, falta de espaço depois da vírgula e minúscula depois de ponto final.
fn pontuacao(t: &str, v: &mut Vec<Problema>) {
    let cs: Vec<(usize, char)> = t.char_indices().collect();
    let p = palavras(t);
    for (k, &(i, c)) in cs.iter().enumerate() {
        // " ," → ","
        if matches!(c, ',' | ';' | '!' | '?') && k > 0 && cs[k - 1].1 == ' ' && k > 1 && !cs[k - 2].1.is_whitespace() && !matches!(cs[k - 2].1, '!' | '?' | '.') {
            let ini = cs[k - 1].0;
            v.push(problema(ini, i, String::new(), (cs.get(k.saturating_sub(2)).map_or(ini, |x| x.0), i + c.len_utf8()), format!("Tire o espaço antes de «{c}»."), "espaco_antes"));
        }
        // "casa,mesa" → "casa, mesa"
        if c == ',' && k > 0 && k + 1 < cs.len() && cs[k - 1].1.is_alphabetic() && cs[k + 1].1.is_alphabetic() {
            let chunk = t[..i].rsplit(char::is_whitespace).next().unwrap_or("").to_string() + &t[i..].split(char::is_whitespace).next().unwrap_or("").to_string();
            if !chunk.contains('@') && !chunk.contains("://") && !chunk.contains("www.") {
                v.push(problema(i, i + 1, ", ".into(), (cs[k - 1].0, cs[k + 1].0 + cs[k + 1].1.len_utf8()), "Falta um espaço depois da vírgula.".into(), "espaco_depois"));
            }
        }
        // ". casa" → ". Casa"
        if matches!(c, '.' | '!' | '?') && cs.get(k + 1).is_some_and(|x| x.1 == ' ') {
            let depois = &t[i + 1..];
            let sem = depois.trim_start_matches(' ');
            if let Some(prox) = sem.chars().next().filter(|c| c.is_lowercase()) {
                let ini = i + 1 + (depois.len() - sem.len());
                let anterior = p.iter().rev().find(|&&(_, f)| f <= i).map(|&(a, b)| &t[a..b]).unwrap_or("");
                let reticencias = c == '.' && k > 0 && cs[k - 1].1 == '.';
                if !eh_abreviatura(anterior) && !reticencias && so_espaco(t, i + 1, ini) {
                    let maiuscula: String = prox.to_uppercase().collect();
                    v.push(problema(ini, ini + prox.len_utf8(), maiuscula, (i, ini + prox.len_utf8()), "Use maiúscula no começo da frase.".into(), "maiuscula"));
                }
            }
        }
    }
}

/// Todos os problemas do texto, por ordem de posição.
pub fn verificar(texto: &str) -> Vec<Problema> {
    let p = palavras(texto);
    let mut v = Vec::new();
    repetidas(texto, &p, &mut v);
    concordancia(texto, &p, &mut v);
    crase_depois_de_ir(texto, &p, &mut v);
    para_mim_infinitivo(texto, &p, &mut v);
    expressoes(texto, &p, &mut v);
    virgula_antes_de_adversativa(texto, &p, &mut v);
    pontuacao(texto, &mut v);
    v.sort_by_key(|x| (x.inicio, x.fim));
    v.dedup_by(|a, b| a.inicio == b.inicio && a.fim == b.fim);
    v
}

#[cfg(test)]
mod testes {
    use super::*;

    fn aplicar(t: &str) -> String {
        let mut saida = t.to_string();
        for p in verificar(t).iter().rev() {
            saida.replace_range(p.inicio..p.fim, &p.sugestao);
        }
        saida
    }

    #[test]
    fn exemplos_do_usuario_e_comuns() {
        assert_eq!(aplicar("eu vou ao casa"), "eu vou à casa");
        assert_eq!(aplicar("Vou ao escola amanhã"), "Vou à escola amanhã");
        assert_eq!(aplicar("a casa e o casa"), "a casa e a casa");
        assert_eq!(aplicar("da livro novo"), "do livro novo");
        assert_eq!(aplicar("fui à pé"), "fui a pé");
        assert_eq!(aplicar("ele foi à mercado"), "ele foi ao mercado");
        assert_eq!(aplicar("Ele foi à supermercado e à parque"), "Ele foi ao supermercado e ao parque");
        assert_eq!(aplicar("vou a escola"), "vou à escola");
        assert_eq!(aplicar("isso é de de verdade"), "isso é de verdade");
        assert_eq!(aplicar("trazer para mim fazer"), "trazer para eu fazer");
        assert_eq!(aplicar("Gosto mas não vou"), "Gosto, mas não vou");
        assert_eq!(aplicar("a nível de empresa"), "em nível de empresa");
        assert_eq!(aplicar("houveram problemas"), "houve problemas");
        assert_eq!(aplicar("casa , mesa"), "casa, mesa");
        assert_eq!(aplicar("casa,mesa"), "casa, mesa");
        assert_eq!(aplicar("Acabou. depois voltou"), "Acabou. Depois voltou");
    }

    #[test]
    fn nao_marca_o_que_esta_certo() {
        for certo in [
            "vou à escola", "vou a pé", "à mão", "ele foi ao mercado", "vou à feira", "às vezes", "à tarde", "à noite", "à direita", "à vontade", "à venda", "o dia", "a casa é bonita", "um programa novo", "vou a casa", "chegou a hora", "o mapa e a mesa",
            "Dr. silva chegou", "preço de 1,5 mil", "veja www.site.com,outro", "Gosto, mas não vou", "Mas ele foi", "para mim, não", "para eu fazer",
            "isso é um problema", "uma pessoa", "3. item um", "etc. e tal", "ele disse... depois saiu", "os dados", "pelo problema",
        ] {
            assert!(verificar(certo).is_empty(), "marcou o certo: {certo:?} → {:?}", verificar(certo));
        }
    }

    #[test]
    fn mantem_a_caixa() {
        assert_eq!(aplicar("Ao casa"), "À casa");
        assert_eq!(aplicar("AO CASA"), "À casa".to_uppercase());
    }
}
