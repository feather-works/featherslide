//! Verificar ortografia (Revisão → Ortografia, F7). Os dicionários são os do Hunspell
//! (`pt_BR.aff`/`pt_BR.dic`, `en_US.*`), lidos do disco; nada de rede. A palavra vale se
//! **qualquer** dicionário carregado a aceita (texto misto português/inglês é comum em slide).
//!
//! O motor aqui só encontra os erros e troca a palavra; a interface mostra a lista.

pub mod gramatica;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use featherslide_core::cursor::{self, Disposicao};
use featherslide_core::medida;
use featherslide_core::modelo::{Apresentacao, Elemento, Slide};
use featherslide_core::quadro::Quadro;
use featherslide_core::sessao::{Mudanca, Sessao};
use spellbook::Dictionary;

/// Os idiomas usados quando a pessoa ainda não escolheu (o primeiro é o principal).
pub const IDIOMAS: [&str; 2] = ["pt_BR", "en_US"];

/// Um dicionário achado no disco (`<codigo>.dic` e `<codigo>.aff` na mesma pasta).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Idioma {
    /// O código do arquivo: `pt_BR`, `en_US`, `es_ES`…
    pub codigo: String,
    pub pasta: PathBuf,
}

/// Todos os dicionários Hunspell que se acham nas pastas de dicionários (o da pasta mais
/// específica vence quando o código se repete), em ordem alfabética.
pub fn idiomas_disponiveis() -> Vec<Idioma> {
    let mut achados: Vec<Idioma> = Vec::new();
    for pasta in pastas_de_dicionarios() {
        let Ok(entradas) = std::fs::read_dir(&pasta) else { continue };
        for e in entradas.flatten() {
            let caminho = e.path();
            let (Some("dic"), Some(codigo)) = (caminho.extension().and_then(|x| x.to_str()), caminho.file_stem().and_then(|x| x.to_str())) else { continue };
            if pasta.join(format!("{codigo}.aff")).is_file() && !achados.iter().any(|i| i.codigo == codigo) {
                achados.push(Idioma { codigo: codigo.to_string(), pasta: pasta.clone() });
            }
        }
    }
    achados.sort_by(|a, b| a.codigo.cmp(&b.codigo));
    achados
}

/// Instala um dicionário Hunspell escolhido pela pessoa (`xx_YY.dic`, com o `xx_YY.aff` ao lado)
/// em `~/.config/featherslide/dicionarios/`, depois de conferir que ele abre. Devolve o código.
pub fn instalar_dicionario(dic: &Path) -> Result<String, String> {
    let codigo = dic.file_stem().and_then(|s| s.to_str()).filter(|_| dic.extension().is_some_and(|e| e == "dic")).ok_or("Escolha um arquivo .dic (com o .aff de mesmo nome ao lado).")?.to_string();
    let aff = dic.with_extension("aff");
    if !aff.is_file() {
        return Err(format!("Falta o arquivo {codigo}.aff na mesma pasta."));
    }
    let (texto_aff, texto_dic) = (ler_texto(&aff).ok_or("Não consegui ler o .aff.")?, ler_texto(dic).ok_or("Não consegui ler o .dic.")?);
    Dictionary::new(&texto_aff, &texto_dic).map_err(|e| format!("Esse dicionário não abriu: {e}"))?;
    let destino = pasta_de_configuracao().ok_or("Não achei a pasta de configuração.")?.join("dicionarios");
    std::fs::create_dir_all(&destino).map_err(|e| e.to_string())?;
    std::fs::write(destino.join(format!("{codigo}.aff")), texto_aff).map_err(|e| e.to_string())?;
    std::fs::write(destino.join(format!("{codigo}.dic")), texto_dic).map_err(|e| e.to_string())?;
    Ok(codigo)
}

fn arquivo_pessoal() -> Option<PathBuf> {
    pasta_de_configuracao().map(|p| p.join("dicionario-pessoal.txt"))
}

/// Quantas palavras há no dicionário pessoal.
pub fn palavras_pessoais() -> usize {
    arquivo_pessoal().and_then(|p| ler_texto(&p)).map_or(0, |t| t.lines().filter(|l| !l.trim().is_empty()).count())
}

/// Esvazia o dicionário pessoal (apaga o arquivo). As palavras voltam a ser marcadas.
pub fn limpar_dicionario_pessoal() -> Result<(), String> {
    match arquivo_pessoal() {
        Some(p) if p.exists() => std::fs::remove_file(p).map_err(|e| e.to_string()),
        _ => Ok(()),
    }
}

/// Uma palavra errada: onde está (slide, elemento, bytes dentro do texto do elemento).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Erro {
    /// Índice do slide (0 = primeiro).
    pub slide: usize,
    pub elemento: u64,
    pub campo: CampoErro,
    /// Posição em bytes no `texto` do elemento.
    pub inicio: usize,
    pub fim: usize,
    pub palavra: String,
}

/// O que a ondulação aponta: palavra que não existe (vermelha) ou frase com problema de gramática (azul).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CampoErro { Texto, Notas, Celula(usize, usize) }

/// Tipo de marca visual de revisão.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipoOndulado {
    Ortografia,
    Gramatica,
}

/// Uma palavra errada (ou frase com problema) já posicionada no slide, para a tela sublinhar:
/// `x` e `y` (a linha de base) em pontos do slide, `largura` do trecho e o `tamanho` da fonte.
/// `erro` diz de onde é no texto e o que trocar (na gramática, `erro.palavra` é o trecho a
/// trocar e `sugestao` o que entra no lugar).
#[derive(Clone, Debug, PartialEq)]
pub struct Ondulado {
    pub tipo: TipoOndulado,
    /// A explicação (gramática); vazia na ortografia.
    pub mensagem: String,
    pub sugestao: String,
    /// Chave para "ignorar" (gramática): regra + trecho.
    pub chave: String,
    pub erro: Erro,
    pub x: f32,
    pub y: f32,
    pub largura: f32,
    pub tamanho: f32,
}

pub struct Corretor {
    dicionarios: Vec<Dictionary>,
    ignoradas: HashSet<String>,
    ignoradas_gramatica: HashSet<String>,
    pessoal: Option<PathBuf>,
}

/// As pastas onde procurar os dicionários, da mais específica para a mais geral.
pub fn pastas_de_dicionarios() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(p) = std::env::var_os("FEATHERSLIDE_DICIONARIOS") {
        v.push(PathBuf::from(p));
    }
    if let Some(c) = pasta_de_configuracao() {
        v.push(c.join("dicionarios"));
    }
    for p in ["/usr/share/hunspell", "/usr/share/myspell", "/usr/share/myspell/dicts", "/usr/local/share/hunspell", "/Library/Spelling"] {
        v.push(PathBuf::from(p));
    }
    v
}

/// `~/.config/featherslide` (Linux), `%APPDATA%\featherslide` (Windows) ou `~/Library/Application Support/featherslide` (macOS).
pub fn pasta_de_configuracao() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("XDG_CONFIG_HOME").filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(p).join("featherslide"));
    }
    if let Some(p) = std::env::var_os("APPDATA") {
        return Some(PathBuf::from(p).join("featherslide"));
    }
    let casa = PathBuf::from(std::env::var_os("HOME")?);
    Some(if cfg!(target_os = "macos") { casa.join("Library/Application Support/featherslide") } else { casa.join(".config/featherslide") })
}

fn ler_texto(p: &Path) -> Option<String> {
    let t = std::fs::read_to_string(p).ok()?;
    Some(t.trim_start_matches('\u{feff}').to_string())
}

fn carregar_dicionario(pasta: &Path, idioma: &str) -> Option<Dictionary> {
    let aff = ler_texto(&pasta.join(format!("{idioma}.aff")))?;
    let dic = ler_texto(&pasta.join(format!("{idioma}.dic")))?;
    Dictionary::new(&aff, &dic).ok()
}

/// Letras tentadas ao inserir ou trocar: o alfabeto e os acentuados do português.
const ALFABETO: &str = "abcdefghijklmnopqrstuvwxyzáàâãéêíóôõúüç";

/// As formas da letra com e sem acento (e vice-versa): "a" ↔ "á", "â", "ã", "à".
fn acentos_de(c: char) -> &'static [char] {
    match c {
        'a' => &['á', 'â', 'ã', 'à'],
        'e' => &['é', 'ê'],
        'i' => &['í'],
        'o' => &['ó', 'ô', 'õ'],
        'u' => &['ú', 'ü'],
        'c' => &['ç'],
        'á' | 'â' | 'ã' | 'à' => &['a'],
        'é' | 'ê' => &['e'],
        'í' => &['i'],
        'ó' | 'ô' | 'õ' => &['o'],
        'ú' | 'ü' => &['u'],
        'ç' => &['c'],
        _ => &[],
    }
}

/// Todas as palavras com **uma** letra acentuada ou desacentuada.
fn variantes_de_acento(p: &[char]) -> Vec<Vec<char>> {
    let mut v = Vec::new();
    for (i, &c) in p.iter().enumerate() {
        for &n in acentos_de(c) {
            let mut q = p.to_vec();
            q[i] = n;
            v.push(q);
        }
    }
    v
}

/// Todas as palavras a **uma** edição: apagar, inverter vizinhas, trocar e inserir uma letra.
fn edicoes(p: &[char]) -> Vec<Vec<char>> {
    let letras: Vec<char> = ALFABETO.chars().collect();
    let mut v = Vec::new();
    for i in 0..p.len() {
        let mut q = p.to_vec();
        q.remove(i);
        v.push(q);
        if i + 1 < p.len() && p[i] != p[i + 1] {
            let mut q = p.to_vec();
            q.swap(i, i + 1);
            v.push(q);
        }
        for &l in &letras {
            if l != p[i] {
                let mut q = p.to_vec();
                q[i] = l;
                v.push(q);
            }
        }
    }
    for i in 0..=p.len() {
        for &l in &letras {
            let mut q = p.to_vec();
            q.insert(i, l);
            v.push(q);
        }
    }
    v
}

impl std::fmt::Debug for Corretor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Corretor({} dicionários)", self.dicionarios.len())
    }
}

impl Corretor {
    /// Carrega os dicionários dos `idiomas` pedidos (o primeiro é o principal, o das sugestões);
    /// sem lista, os de fábrica. Erro se não abrir nenhum.
    pub fn carregar(idiomas: &[String]) -> Result<Self, String> {
        let pastas = pastas_de_dicionarios();
        let pedidos: Vec<&str> = if idiomas.is_empty() { IDIOMAS.to_vec() } else { idiomas.iter().map(String::as_str).collect() };
        let dicionarios: Vec<Dictionary> = pedidos.iter().filter_map(|i| pastas.iter().find_map(|p| carregar_dicionario(p, i))).collect();
        if dicionarios.is_empty() {
            return Err("Nenhum dicionário encontrado para os idiomas escolhidos. Instale o pacote hunspell-pt-br ou use Configurações → Ortografia → Adicionar idioma.".into());
        }
        let pessoal = arquivo_pessoal();
        let mut c = Self::de_dicionarios(dicionarios);
        if let Some(texto) = pessoal.as_deref().and_then(ler_texto) {
            for palavra in texto.lines().map(str::trim).filter(|l| !l.is_empty()) {
                c.aceitar(palavra);
            }
        }
        c.pessoal = pessoal;
        Ok(c)
    }

    /// Um corretor de um dicionário Hunspell dado como texto (`.aff` e `.dic`).
    pub fn de_texto(aff: &str, dic: &str) -> Result<Self, String> {
        Ok(Self::de_dicionarios(vec![Dictionary::new(aff, dic).map_err(|e| e.to_string())?]))
    }

    pub fn de_dicionarios(dicionarios: Vec<Dictionary>) -> Self {
        Self { dicionarios, ignoradas: HashSet::new(), ignoradas_gramatica: HashSet::new(), pessoal: None }
    }

    fn aceitar(&mut self, palavra: &str) {
        for d in &mut self.dicionarios {
            let _ = d.add(palavra);
        }
    }

    /// A palavra está certa (ou foi ignorada)?
    pub fn palavra_ok(&self, palavra: &str) -> bool {
        self.ignoradas.contains(&palavra.to_lowercase()) || self.dicionarios.iter().any(|d| d.check(palavra))
    }

    /// Palavras que o dicionário aceita a até duas edições da errada, onde pôr ou tirar um
    /// acento conta como uma. Pega o que o Hunspell perde quando se somam dois erros
    /// ("inportancia" → "importância": uma letra trocada e um acento faltando). Como pergunta ao
    /// próprio dicionário, vale também para as formas derivadas (plurais, conjugações).
    fn parecidas(&self, palavra: &str) -> Vec<String> {
        let original: Vec<char> = palavra.chars().collect();
        if original.len() < 3 || original.len() > 24 {
            return Vec::new();
        }
        let minuscula: Vec<char> = palavra.to_lowercase().chars().collect();
        let capital = original[0].is_uppercase();
        let mut vistas: HashSet<Vec<char>> = HashSet::from([minuscula.clone()]);
        let mut achadas: Vec<(u8, String)> = Vec::new();
        let mut tentar = |cand: Vec<char>, custo: u8, achadas: &mut Vec<(u8, String)>| {
            if !vistas.insert(cand.clone()) {
                return;
            }
            let texto: String = cand.iter().collect();
            if self.dicionarios.iter().any(|d| d.check(&texto)) {
                achadas.push((custo, texto));
            }
        };
        let acentos = variantes_de_acento(&minuscula);
        for q in acentos.iter().cloned() {
            tentar(q, 1, &mut achadas);
        }
        for q in edicoes(&minuscula) {
            tentar(q, 1, &mut achadas);
        }
        // Acento mais uma edição.
        for a in &acentos {
            for q in edicoes(a) {
                tentar(q, 2, &mut achadas);
            }
        }
        achadas.sort();
        achadas
            .into_iter()
            .map(|(_, p)| if capital { p.chars().next().map(|c| c.to_uppercase().collect::<String>() + &p[c.len_utf8()..]).unwrap_or(p) } else { p })
            .take(5)
            .collect()
    }

    /// Sugestões de correção, sem repetir: as parecidas pelo índice, depois as do Hunspell
    /// (as de uma palavra só antes das que separam em duas).
    pub fn sugestoes(&self, palavra: &str) -> Vec<String> {
        let mut todas: Vec<String> = self.parecidas(palavra);
        for d in &self.dicionarios {
            let mut s = Vec::new();
            d.suggest(palavra, &mut s);
            for x in s {
                let x = x.trim().to_string();
                if !x.is_empty() && x != palavra && !todas.contains(&x) {
                    todas.push(x);
                }
            }
        }
        let (inteiras, partidas): (Vec<String>, Vec<String>) = todas.into_iter().partition(|s| !s.contains(' '));
        let mut todas = inteiras;
        todas.extend(partidas);
        todas.truncate(8);
        todas
    }

    /// Ignora um aviso de gramática (pela `chave` do `Ondulado`) até fechar o programa.
    pub fn ignorar_gramatica(&mut self, chave: &str) {
        self.ignoradas_gramatica.insert(chave.to_string());
    }

    /// Ignora a palavra (qualquer caixa) até fechar o programa.
    pub fn ignorar(&mut self, palavra: &str) {
        self.ignoradas.insert(palavra.to_lowercase());
    }

    /// Põe a palavra no dicionário pessoal (grava em `dicionario-pessoal.txt`).
    pub fn adicionar_ao_dicionario(&mut self, palavra: &str) -> Result<(), String> {
        self.aceitar(palavra);
        let Some(caminho) = self.pessoal.clone() else { return Ok(()) };
        if let Some(pasta) = caminho.parent() {
            std::fs::create_dir_all(pasta).map_err(|e| e.to_string())?;
        }
        let mut atual = ler_texto(&caminho).unwrap_or_default();
        if !atual.is_empty() && !atual.ends_with('\n') {
            atual.push('\n');
        }
        atual.push_str(palavra);
        atual.push('\n');
        std::fs::write(&caminho, atual).map_err(|e| e.to_string())
    }

    /// As palavras erradas do texto de um elemento, na ordem em que aparecem.
    fn erros_do_elemento(&self, slide: usize, e: &Elemento) -> Vec<Erro> {
        palavras(&e.texto)
            .into_iter()
            .filter(|&(ini, fim)| !self.palavra_ok(&e.texto[ini..fim]))
            .map(|(inicio, fim)| Erro { slide, elemento: e.id, campo: CampoErro::Texto, inicio, fim, palavra: e.texto[inicio..fim].to_string() })
            .collect()
    }

    /// Todas as palavras erradas dos textos dos elementos visíveis, slide a slide.
    pub fn verificar(&self, ap: &Apresentacao) -> Vec<Erro> {
        let mut erros = Vec::new();
        for (i, slide) in ap.slides.iter().enumerate() {
            let notas = Elemento { id: slide.id, texto: slide.notas.clone(), ..Elemento::default() };
            let mut encontrados = self.erros_do_elemento(i, &notas);
            for erro in &mut encontrados { erro.campo = CampoErro::Notas; }
            erros.extend(encontrados);
            for e in slide.elementos.iter().filter(|e| !e.oculto && !e.texto.is_empty()) {
                erros.extend(self.erros_do_elemento(i, e));
            }
            for e in slide.elementos.iter().filter(|e| !e.oculto && e.tipo == featherslide_core::modelo::Tipo::Tabela) {
                for (linha, celulas) in featherslide_core::dados::ler_tabela(e).into_iter().enumerate() {
                    for (coluna, texto) in celulas.into_iter().enumerate() {
                        let temporario = Elemento { id: e.id, texto, ..Elemento::default() };
                        let mut encontrados = self.erros_do_elemento(i, &temporario);
                        for erro in &mut encontrados { erro.campo = CampoErro::Celula(linha, coluna); }
                        erros.extend(encontrados);
                    }
                }
            }
        }
        erros
    }

    /// As palavras erradas do slide `indice`, com a posição em que o `quadro` as desenha (o texto
    /// já quebrado em linhas). Texto girado não entra. A largura vem da medida aproximada do
    /// motor, então a ondulação pode errar uns pixels em relação à fonte desenhada.
    pub fn ondulados(&self, indice: usize, slide: &Slide, quadro: &Quadro, com_gramatica: bool) -> Vec<Ondulado> {
        let mut saida = Vec::new();
        for d in quadro.elementos.iter().filter(|d| !d.dica && d.rotacao == 0.0 && !d.linhas.is_empty()) {
            let Some(el) = slide.elementos.iter().find(|e| e.id == d.id) else { continue };
            let erros = self.erros_do_elemento(indice, el);
            let mut proximo = 0;
            for linha in &d.linhas {
                for (ini, fim) in palavras(&linha.texto) {
                    let palavra = &linha.texto[ini..fim];
                    if self.palavra_ok(palavra) {
                        continue;
                    }
                    // O erro de mesma palavra que vem a seguir no texto do elemento.
                    let Some(pos) = erros[proximo..].iter().position(|e| e.palavra == palavra) else { continue };
                    saida.push(Ondulado {
                        tipo: TipoOndulado::Ortografia,
                        mensagem: String::new(),
                        sugestao: String::new(),
                        chave: String::new(),
                        erro: erros[proximo + pos].clone(),
                        x: linha.x + medida::largura(&linha.texto[..ini], d.tamanho, d.negrito),
                        y: linha.y,
                        largura: medida::largura(palavra, d.tamanho, d.negrito),
                        tamanho: d.tamanho,
                    });
                    proximo += pos + 1;
                }
            }
            if com_gramatica {
                self.ondulados_de_gramatica(indice, el, d, &mut saida);
            }
        }
        saida
    }

    /// Os avisos de gramática do texto de `el`, posicionados nas linhas desenhadas de `d`.
    fn ondulados_de_gramatica(&self, indice: usize, el: &Elemento, d: &featherslide_core::quadro::ElementoDesenhado, saida: &mut Vec<Ondulado>) {
        let disp = Disposicao { linhas: &d.linhas, tamanho: d.tamanho, negrito: d.negrito };
        for p in gramatica::verificar(&el.texto) {
            let chave = format!("{}:{}", p.regra, el.texto[p.ancora.0..p.ancora.1].to_lowercase());
            if self.ignoradas_gramatica.contains(&chave) {
                continue;
            }
            let erro = Erro { slide: indice, elemento: el.id, campo: CampoErro::Texto, inicio: p.inicio, fim: p.fim, palavra: el.texto[p.inicio..p.fim].to_string() };
            for (x, y, largura) in cursor::trechos(&el.texto, &disp, p.ancora.0, p.ancora.1) {
                saida.push(Ondulado { tipo: TipoOndulado::Gramatica, mensagem: p.mensagem.clone(), sugestao: p.sugestao.clone(), chave: chave.clone(), erro: erro.clone(), x, y, largura, tamanho: d.tamanho });
            }
        }
    }

    /// Troca a palavra errada por `nova` (uma edição só, que o desfazer reverte). Depois de
    /// corrigir, as posições dos outros erros do mesmo elemento mudam: verifique de novo.
    pub fn corrigir(&self, sessao: &mut Sessao, erro: &Erro, nova: &str) -> Result<Mudanca, String> {
        let nova = nova.trim();
        if nova.is_empty() {
            return Err("A correção está vazia.".into());
        }
        self.trocar(sessao, erro, nova)
    }

    /// Como `corrigir`, mas põe `nova` **do jeito que veio**, até vazia ou com espaço nas pontas:
    /// é o que a gramática sugere («, mas», tirar um espaço).
    pub fn trocar(&self, sessao: &mut Sessao, erro: &Erro, nova: &str) -> Result<Mudanca, String> {
        sessao.slide_ir(erro.slide)?;
        let atual = match erro.campo {
            CampoErro::Notas => sessao.slide().notas.clone(),
            CampoErro::Texto => sessao.slide().elementos.iter().find(|e| e.id == erro.elemento).map(|e| e.texto.clone()).ok_or("Esse elemento não existe mais.")?,
            CampoErro::Celula(l,c) => {
                let e = sessao.slide().elementos.iter().find(|e| e.id == erro.elemento).ok_or("Essa tabela não existe mais.")?;
                featherslide_core::dados::ler_tabela(e).get(l).and_then(|linha| linha.get(c)).cloned().ok_or("Essa célula não existe mais.")?
            },
        };
        if atual.get(erro.inicio..erro.fim) != Some(erro.palavra.as_str()) {
            return Err("O texto mudou desde a verificação; verifique de novo.".into());
        }
        let novo = format!("{}{}{}", &atual[..erro.inicio], nova, &atual[erro.fim..]);
        match erro.campo {
            CampoErro::Texto => sessao.texto_definir(erro.elemento, &novo),
            CampoErro::Notas => sessao.notas_definir(&novo),
            CampoErro::Celula(l,c) => sessao.tabela_celula_definir(erro.elemento,l,c,&novo),
        }
    }
}

fn eh_letra(c: char) -> bool {
    c.is_alphabetic()
}

/// As palavras do texto como (início, fim) em bytes. Pula números, siglas em maiúsculas, letras
/// soltas, endereços (`://`, `@`, `www.`) e o que mistura letras com dígitos.
fn palavras(texto: &str) -> Vec<(usize, usize)> {
    let mut saida = Vec::new();
    let mut base = 0;
    for pedaco in texto.split_inclusive(char::is_whitespace) {
        let inicio_pedaco = base;
        base += pedaco.len();
        if pedaco.contains("://") || pedaco.contains('@') || pedaco.contains("www.") {
            continue;
        }
        let mut atual: Option<usize> = None;
        let mut chars = pedaco.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            let dentro = eh_letra(c) || c.is_ascii_digit() || (matches!(c, '\'' | '’' | '-') && atual.is_some() && chars.peek().is_some_and(|&(_, p)| eh_letra(p)));
            match (dentro, atual) {
                (true, None) => atual = Some(i),
                (false, Some(ini)) => {
                    guardar(&mut saida, texto, inicio_pedaco + ini, inicio_pedaco + i);
                    atual = None;
                }
                _ => {}
            }
        }
        if let Some(ini) = atual {
            guardar(&mut saida, texto, inicio_pedaco + ini, inicio_pedaco + pedaco.len());
        }
    }
    saida
}

fn guardar(saida: &mut Vec<(usize, usize)>, texto: &str, ini: usize, fim: usize) {
    let p = &texto[ini..fim];
    let letras = p.chars().filter(|c| eh_letra(*c)).count();
    let tem_digito = p.chars().any(|c| c.is_ascii_digit());
    // Sigla: tudo em maiúsculas, com `s` de plural no fim ("APIs").
    let corpo = p.strip_suffix('s').unwrap_or(p);
    let sigla = corpo.chars().filter(|c| eh_letra(*c)).count() > 1 && corpo.chars().filter(|c| eh_letra(*c)).all(char::is_uppercase);
    if letras >= 2 && !tem_digito && !sigla {
        saida.push((ini, fim));
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn revisao_corrige_notas_e_celulas_com_historico_e_detecta_alvo_alterado() {
        let c = Corretor::de_texto("SET UTF-8\n", "3\ncasa\nbonita\nação\n").unwrap();
        let mut s = Sessao::default();
        s.notas_definir("casa bnita").unwrap();
        s.tabela_inserir(1,1).unwrap();
        let id = s.estado().selecao[0];
        s.tabela_celula_definir(id,0,0,"casa bnita").unwrap();
        let erros = c.verificar(s.apresentacao());
        let nota = erros.iter().find(|e| e.campo == CampoErro::Notas).unwrap();
        let celula = erros.iter().find(|e| e.campo == CampoErro::Celula(0,0)).unwrap();
        c.corrigir(&mut s,nota,"bonita").unwrap();
        assert_eq!(s.slide().notas,"casa bonita");
        s.desfazer().unwrap(); assert_eq!(s.slide().notas,"casa bnita");
        c.corrigir(&mut s,celula,"bonita").unwrap();
        let e = s.slide().elementos.iter().find(|e| e.id == id).unwrap();
        assert_eq!(featherslide_core::dados::ler_tabela(e)[0][0],"casa bonita");
        assert!(c.corrigir(&mut s,celula,"bonita").is_err());
    }

    fn tokens(t: &str) -> Vec<&str> {
        palavras(t).into_iter().map(|(a, b)| &t[a..b]).collect()
    }

    #[test]
    fn variantes_e_edicoes() {
        let p: Vec<char> = "casa".chars().collect();
        assert!(variantes_de_acento(&p).iter().any(|q| q.iter().collect::<String>() == "cása"));
        let e: Vec<String> = edicoes(&p).into_iter().map(|q| q.into_iter().collect()).collect();
        assert!(e.contains(&"asa".to_string()) && e.contains(&"acsa".to_string()) && e.contains(&"casas".to_string()) && e.contains(&"cama".to_string()));
    }

    #[test]
    fn separa_palavras_e_pula_o_que_nao_e_palavra() {
        assert_eq!(tokens("Olá, mundo! É a casa-grande."), vec!["Olá", "mundo", "casa-grande"]);
        assert_eq!(tokens("PDF 2026 abc123 site www.x.com a@b.com https://x.y"), vec!["site"]);
        assert_eq!(tokens("APIs ONGs Apis"), vec!["Apis"]);
        assert_eq!(tokens("d'água ação"), vec!["d'água", "ação"]);
    }
}
