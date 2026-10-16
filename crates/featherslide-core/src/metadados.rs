//! Propriedades e estatísticas da apresentação; sem dependências da interface.
//! As datas são segundos UTC desde 1970. Valores ausentes em arquivos legados são zero.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::modelo::{Apresentacao, Papel, Tipo};

/// Metadados editáveis e automáticos de um documento FeatherSlide.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Metadados {
    pub assunto: String,
    pub empresa: String,
    pub gerente: String,
    pub categoria: String,
    /// Termos separados por vírgula; não implica indexação ou pesquisa remota.
    pub palavras_chave: String,
    /// Descrição/comentário do documento, diferente dos comentários por slide.
    pub comentarios: String,
    pub criado_em_utc: u64,
    pub modificado_em_utc: u64,
    pub ultimo_autor: String,
    pub revisoes: u64,
    /// Soma estimada de períodos de edição ativa, em segundos.
    pub tempo_edicao_segundos: u64,
    pub propriedades_personalizadas: BTreeMap<String, String>,
    pub idioma: String,
    pub identificador: String,
    pub estendidos: BTreeMap<String, String>,
    /// FTS-0121–0123: roteiros de apresentação; são dados do documento, não da UI.
    pub apresentacoes_personalizadas: Vec<crate::personalizadas::ApresentacaoPersonalizada>,
}

impl Default for Metadados {
    fn default() -> Self {
        Self {
            assunto: String::new(), empresa: String::new(), gerente: String::new(),
            categoria: String::new(), palavras_chave: String::new(),
            comentarios: String::new(), criado_em_utc: 0, modificado_em_utc: 0,
            ultimo_autor: String::new(), revisoes: 0, tempo_edicao_segundos: 0,
            propriedades_personalizadas: BTreeMap::new(),
            idioma: "pt-BR".into(), identificador: String::new(), estendidos: BTreeMap::new(),
            apresentacoes_personalizadas: Vec::new(),
        }
    }
}

/// Retorna a hora UTC atual sem bibliotecas externas. Retorna zero se o relógio falhar.
pub fn agora_utc() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_secs())
}

/// Identificador estável e local, gerado uma vez por documento, sem rede.
pub fn novo_identificador() -> String {
    static SEQUENCIA: AtomicU64 = AtomicU64::new(1);
    let instante = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_nanos());
    let numero = SEQUENCIA.fetch_add(1, Ordering::Relaxed);
    format!("featherslide-{instante:032x}-{numero:016x}")
}

impl Metadados {
    /// Metadados de um documento recém-criado.
    pub fn novo() -> Self {
        let agora = agora_utc();
        Self {
            criado_em_utc: agora, modificado_em_utc: agora,
            identificador: novo_identificador(), ..Self::default()
        }
    }

    /// Aplica apenas campos editáveis; não permite sobrescrever ID, datas ou contadores.
    pub fn aplicar_editaveis(&mut self, novo: &Self) {
        self.assunto.clone_from(&novo.assunto);
        self.empresa.clone_from(&novo.empresa);
        self.gerente.clone_from(&novo.gerente);
        self.categoria.clone_from(&novo.categoria);
        self.palavras_chave.clone_from(&novo.palavras_chave);
        self.comentarios.clone_from(&novo.comentarios);
        self.propriedades_personalizadas.clone_from(&novo.propriedades_personalizadas);
        self.idioma.clone_from(&novo.idioma);
        self.estendidos.clone_from(&novo.estendidos);
    }

    /// Verifica limites de campos e mapas; evita metadados enormes ou nomes vazios.
    pub fn validar(&self) -> Result<(), String> {
        for (nome, valor) in [
            ("Assunto", &self.assunto), ("Empresa", &self.empresa),
            ("Gerente", &self.gerente), ("Categoria", &self.categoria),
            ("Palavras-chave", &self.palavras_chave), ("Comentários", &self.comentarios),
            ("Idioma", &self.idioma),
        ] {
            let limite = if nome == "Comentários" { 16_384 } else { 2_048 };
            if valor.len() > limite { return Err(format!("{nome} ultrapassa {limite} bytes.")); }
        }
        if self.idioma.trim().is_empty() {
            return Err("Informe o idioma do documento (ex.: pt-BR).".into());
        }
        if self.apresentacoes_personalizadas.len() > 50 {
            return Err("Máximo de 50 apresentações personalizadas.".into());
        }
        for p in &self.apresentacoes_personalizadas {
            if p.nome.trim().is_empty() || p.nome.len() > 120 || p.slides.is_empty() || p.slides.len() > 1000 {
                return Err("Apresentação personalizada com nome ou tamanho inválido.".into());
            }
        }
        for (nome, mapa) in [
            ("Propriedades personalizadas", &self.propriedades_personalizadas),
            ("Metadados estendidos", &self.estendidos),
        ] {
            if mapa.len() > 100 { return Err(format!("{nome}: máximo de 100 campos.")); }
            for (chave, valor) in mapa {
                if chave.trim().is_empty() || chave.len() > 128 {
                    return Err(format!("{nome}: chave vazia ou maior que 128 bytes."));
                }
                if valor.len() > 16_384 {
                    return Err(format!("{nome}: valor de '{chave}' excede 16 KB."));
                }
            }
        }
        Ok(())
    }
}

/// Estatísticas calculadas ao consultar a apresentação; não ficam obsoletas no arquivo.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Estatisticas {
    pub slides: usize,
    pub slides_ocultos: usize,
    pub elementos: usize,
    pub caixas_texto: usize,
    pub imagens: usize,
    pub formas: usize,
    pub tabelas: usize,
    pub graficos: usize,
    pub animacoes: usize,
    pub comentarios: usize,
    pub comentarios_resolvidos: usize,
    pub notas: usize,
    pub palavras: usize,
    pub caracteres: usize,
    pub secoes: usize,
    pub fontes_embutidas: usize,
}

/// Produz contagens dinâmicas dos slides, elementos, notas e comentários.
pub fn estatisticas(ap: &Apresentacao) -> Estatisticas {
    let mut res = Estatisticas {
        slides: ap.slides.len(), fontes_embutidas: ap.fontes.len(), ..Estatisticas::default()
    };
    let mut secoes = std::collections::BTreeSet::new();
    for slide in &ap.slides {
        res.slides_ocultos += usize::from(slide.oculto);
        res.notas += usize::from(!slide.notas.trim().is_empty());
        res.comentarios += slide.comentarios.len();
        res.comentarios_resolvidos += slide.comentarios.iter().filter(|c| c.resolvido).count();
        if !slide.secao.trim().is_empty() { secoes.insert(slide.secao.as_str()); }
        res.palavras += slide.notas.split_whitespace().count();
        res.caracteres += slide.notas.chars().count();
        for c in &slide.comentarios {
            res.palavras += c.texto.split_whitespace().count();
            res.caracteres += c.texto.chars().count();
        }
        for e in &slide.elementos {
            if e.papel == Papel::Fundo { continue; }
            res.elementos += 1;
            match e.tipo {
                Tipo::Texto | Tipo::WordArt => res.caixas_texto += 1,
                Tipo::Imagem => res.imagens += 1,
                Tipo::Tabela => res.tabelas += 1,
                Tipo::Grafico => res.graficos += 1,
                _ => res.formas += 1,
            }
            if e.animacao.tipo != crate::modelo::TipoAnimacao::Nenhuma { res.animacoes += 1; }
            res.palavras += e.texto.split_whitespace().count();
            res.caracteres += e.texto.chars().count();
        }
    }
    res.secoes = secoes.len();
    res
}

/// Formata segundos Unix UTC como ISO-8601, sem relógio nem crate externo.
pub fn data_utc(segundos: u64) -> String {
    if segundos == 0 { return "Não disponível".into(); }
    let dias = (segundos / 86_400) as i64;
    let resto = segundos % 86_400;
    // Conversão gregoriana civil_from_days (anos bissextos incluídos).
    let z = dias + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe/1460 + doe/36524 - doe/146096) / 365;
    let mut ano = yoe + era * 400;
    let doy = doe - (365*yoe + yoe/4 - yoe/100);
    let mp = (5*doy+2)/153;
    let dia = doy - (153*mp+2)/5 + 1;
    let mes = mp + if mp < 10 { 3 } else { -9 };
    ano += i64::from(mes <= 2);
    format!("{ano:04}-{mes:02}-{dia:02} {:02}:{:02}:{:02} UTC",
        resto/3600, (resto%3600)/60, resto%60)
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn campos_editaveis_nao_alteram_automaticos() {
        let mut antigo = Metadados::novo();
        antigo.revisoes = 8;
        antigo.tempo_edicao_segundos = 123;
        let id = antigo.identificador.clone();
        let mut novo = Metadados::novo();
        novo.assunto = "Assunto".into();
        novo.propriedades_personalizadas.insert("Centro".into(), "Vitória".into());
        antigo.aplicar_editaveis(&novo);
        assert_eq!(antigo.assunto, "Assunto");
        assert_eq!(antigo.revisoes, 8);
        assert_eq!(antigo.tempo_edicao_segundos, 123);
        assert_eq!(antigo.identificador, id);
        assert_eq!(antigo.propriedades_personalizadas["Centro"], "Vitória");
    }
    #[test]
    fn data_conhecida() {
        assert_eq!(data_utc(1_704_067_200), "2024-01-01 00:00:00 UTC");
        assert_eq!(data_utc(0), "Não disponível");
    }
    #[test]
    fn revisoes_automaticas_nao_sao_desfeitas() {
        let mut sessao = crate::sessao::Sessao::default();
        let id = sessao.apresentacao().metadados.identificador.clone();
        let dados = sessao.apresentacao().metadados.clone();
        sessao.propriedades_definir("Minha apresentação".into(), "Autora".into(), dados.clone()).unwrap();
        assert_eq!(sessao.apresentacao().metadados.revisoes, 1);
        assert_eq!(sessao.apresentacao().metadados.ultimo_autor, "Autora");
        let versao = sessao.estado().versao;
        sessao.propriedades_definir("Minha apresentação".into(), "Autora".into(), dados).unwrap();
        assert_eq!(sessao.estado().versao, versao, "edição idêntica não cria revisão");
        sessao.desfazer().unwrap();
        assert!(sessao.apresentacao().titulo.is_empty());
        assert!(sessao.apresentacao().metadados.revisoes >= 2);
        assert_eq!(sessao.apresentacao().metadados.identificador, id);
        sessao.refazer().unwrap();
        assert_eq!(sessao.apresentacao().titulo, "Minha apresentação");
    }

    #[test]
    fn estatisticas_de_slides_e_textos_sao_dinamicas() {
        let mut sessao = crate::sessao::Sessao::default();
        let antes = sessao.estatisticas();
        sessao.slide_novo("em_branco").unwrap();
        sessao.elemento_inserir(crate::modelo::Tipo::Texto,
            Some([10.0, 15.0, 200.0, 100.0]), None).unwrap();
        let id = sessao.estado().selecao[0];
        sessao.texto_definir(id, "Olá mundo!").unwrap();
        let depois = sessao.estatisticas();
        assert_eq!(depois.slides, antes.slides + 1);
        assert_eq!(depois.caixas_texto, antes.caixas_texto + 1);
        assert!(depois.palavras >= antes.palavras + 2);
        sessao.desfazer().unwrap();
        assert!(sessao.estatisticas().palavras < depois.palavras);
    }

    #[test]
    fn identificadores_diferentes() {
        assert_ne!(novo_identificador(), novo_identificador());
    }
}
