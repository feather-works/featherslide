//! As preferências do aplicativo, gravadas em `~/.config/featherslide/config.json`
//! (só na máquina: sem conta, sem telemetria).

use std::collections::HashMap;
use std::path::PathBuf;

use iced::keyboard::{Key, Modifiers};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Configuracao {
    pub auto_salvar: bool,
    /// Segundos sem mexer até gravar.
    pub intervalo: u64,
    pub painel_slides: bool,
    pub painel_notas: bool,
    /// Os arquivos abertos ou salvos há pouco (o mais novo primeiro).
    pub recentes: Vec<String>,
    pub favoritos: Vec<String>,
    pub escuro: bool,
    /// Quando ativo, detecta a preferência de aparência do sistema na abertura.
    pub tema_auto: bool,
    /// Índice em `tema::ACENTOS`.
    pub acento: usize,
    /// None = preferência legada; Some(false) restaura a logo de fábrica
    /// sem alterar a cor de destaque dos controles do aplicativo.
    pub marca_personalizada: Option<bool>,
    /// Só os atalhos que o usuário mudou: id → combinação (`Ctrl+Shift+S`).
    pub atalhos: HashMap<String, String>,
    /// A barra de título própria (com minimizar, maximizar e fechar dentro dela). Vale ao abrir o app.
    pub barra_propria: bool,
    /// Com dois monitores, a plateia vê o slide no segundo e o apresentador vê notas e o próximo no principal.
    pub dois_monitores: bool,
    /// Sublinhar as palavras erradas enquanto se edita (e abrir correções com o botão direito).
    pub ortografia_automatica: bool,
    /// Os dicionários da ortografia, o primeiro é o principal (o das sugestões). Vazio = os de fábrica.
    pub idiomas_ortografia: Vec<String>,
    /// Sublinhar em azul as frases com problema de gramática (crase, concordância, vírgula…).
    pub gramatica_automatica: bool,
    pub autocorrecao: bool,
    pub capitalizacao: bool,
    pub correcoes: HashMap<String, String>,
    pub sinonimos: HashMap<String, String>,
    /// Chave: idioma origem|idioma destino|expressão. Tradução exata pelo glossário local.
    pub traducoes: HashMap<String, String>,
}

/// Os atalhos que se podem regravar: (id, rótulo, padrão).
pub const ATALHOS: [(&str, &str, &str); 19] = [
    ("agrupar", "Agrupar / desagrupar", "Ctrl+G"),
    ("slide_novo", "Novo slide", "Ctrl+M"),
    ("duplicar", "Duplicar", "Ctrl+D"),
    ("negrito", "Negrito", "Ctrl+B"),
    ("italico", "Itálico", "Ctrl+I"),
    ("sublinhado", "Sublinhado", "Ctrl+U"),
    ("desfazer", "Desfazer", "Ctrl+Z"),
    ("refazer", "Refazer", "Ctrl+Y"),
    ("salvar", "Salvar", "Ctrl+S"),
    ("salvar_copia", "Salvar cópia…", "Ctrl+Alt+S"),
    ("salvar_como", "Salvar como…", "Ctrl+Shift+S"),
    ("ortografia", "Verificar ortografia", "F7"),
    ("imprimir", "Imprimir", "Ctrl+P"),
    ("exportar_pdf", "Exportar como PDF…", "Ctrl+Shift+E"),
    ("abrir", "Abrir apresentação", "Ctrl+O"),
    ("novo", "Nova apresentação", "Ctrl+N"),
    ("do_inicio", "Apresentar do início", "F5"),
    ("do_atual", "Apresentar do slide atual", "Shift+F5"),
    ("ajuda", "Ajuda", "F1"),
];

/// A combinação de um evento de tecla em texto (`Ctrl+Shift+S`, `F5`), ou `None` se for só modificador.
pub fn combo_de(tecla: &Key, m: Modifiers) -> Option<String> {
    feather_ui::atalhos::combo_de(tecla, m)
}

impl Default for Configuracao {
    fn default() -> Self {
        Self { auto_salvar: false, intervalo: 5, painel_slides: true, painel_notas: true, recentes: Vec::new(), favoritos: Vec::new(), escuro: true, tema_auto: false, acento: 0, marca_personalizada: None, atalhos: HashMap::new(), barra_propria: true, dois_monitores: true, ortografia_automatica: true, idiomas_ortografia: vec!["pt_BR".into(), "en_US".into()], gramatica_automatica: true,
            autocorrecao: false, capitalizacao: false,
            correcoes: [("nao", "não"), ("voce", "você"), ("tbm", "também")].into_iter().map(|(a,b)| (a.into(),b.into())).collect(),
            sinonimos: [("importante", "relevante;significativo"), ("rápido", "veloz;ligeiro"), ("bonito", "belo;formoso"), ("iniciar", "começar;principiar"), ("finalizar", "concluir;terminar")].into_iter().map(|(a,b)| (a.into(),b.into())).collect(),
            traducoes: HashMap::new() }
    }
}

fn caminho() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("featherslide").join("config.json"))
}

impl Configuracao {
    /// Antes desta preferência existir, acento diferente do padrão era escolha manual.
    pub fn logo_usa_acento(&self) -> bool {
        self.marca_personalizada.unwrap_or(self.acento != 0)
    }

    /// A combinação do atalho `id`: a do usuário ou a de fábrica.
    pub fn atalho(&self, id: &str) -> String {
        self.atalhos
            .get(id)
            .cloned()
            .or_else(|| ATALHOS.iter().find(|a| a.0 == id).map(|a| a.2.to_string()))
            .unwrap_or_default()
    }

    /// Põe `p` no começo dos recentes.
    pub fn registrar_recente(&mut self, p: &std::path::Path) {
        let s = p.to_string_lossy().to_string();
        self.recentes.retain(|r| *r != s);
        self.recentes.insert(0, s);
        self.recentes.truncate(12);
        self.gravar();
    }

    /// O comando que `combo` dispara, se houver.
    pub fn comando_de(&self, combo: &str) -> Option<&'static str> {
        ATALHOS.iter().find(|a| self.atalho(a.0).eq_ignore_ascii_case(combo)).map(|a| a.0)
    }

    /// Carregamento com caminho explícito para testar disco real sem trocar
    /// HOME/XDG_CONFIG_HOME do processo nem tocar nas preferências do usuário.
    fn carregar_de(path: &std::path::Path) -> Self {
        std::fs::read(path).ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    fn gravar_em(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(pasta) = path.parent() {
            std::fs::create_dir_all(pasta)?;
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, bytes)
    }

    pub fn carregar() -> Self {
        caminho().map(|p| Self::carregar_de(&p)).unwrap_or_default()
    }

    // O app real salva normalmente; testes de App nunca gravam em ~/.config.
    // Testes de round-trip chamam gravar_em/carregar_de com paths isolados.
    #[cfg(test)]
    pub fn gravar(&self) {}

    #[cfg(not(test))]
    pub fn gravar(&self) {
        if let Some(p) = caminho() {
            let _ = self.gravar_em(&p);
        }
    }
}

/// A pasta Documentos do usuário (`XDG_DOCUMENTS_DIR`; "Documentos" ou "Documents" se não houver).
fn pasta_documentos() -> Option<PathBuf> {
    let home = PathBuf::from(std::env::var_os("HOME")?);
    let do_xdg = std::fs::read_to_string(home.join(".config/user-dirs.dirs")).ok().and_then(|s| {
        s.lines().find_map(|l| {
            let v = l.strip_prefix("XDG_DOCUMENTS_DIR=")?.trim().trim_matches('"');
            Some(PathBuf::from(v.replace("$HOME", &home.to_string_lossy())))
        })
    });
    [do_xdg, Some(home.join("Documentos")), Some(home.join("Documents"))].into_iter().flatten().find(|p| p.is_dir())
}

/// Onde os diálogos de abrir e salvar começam: `Documentos/FeatherSlide` (ou só Documentos).
pub fn pasta_das_apresentacoes() -> Option<PathBuf> {
    let docs = pasta_documentos()?;
    let propria = docs.join("FeatherSlide");
    Some(if propria.is_dir() { propria } else { docs })
}

/// Cada apresentação sem nome mantém seu próprio rascunho entre execuções.
pub fn novo_rascunho() -> PathBuf {
    static SEQUENCIA: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let base = caminho().and_then(|p| p.parent().map(PathBuf::from)).unwrap_or_else(|| std::env::temp_dir().join("featherslide"));
    let tempo = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    let numero = SEQUENCIA.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    base.join("rascunhos").join(format!("apresentacao-{tempo}-{}-{numero}.fsdz", std::process::id()))
}

#[cfg(test)]
mod testes_ui010 {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static CONTADOR: AtomicU64 = AtomicU64::new(0);

    fn arquivo_teste() -> PathBuf {
        let i = CONTADOR.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("featherslide-ui010-{}-{i}", std::process::id()))
            .join("config.json")
    }

    #[test]
    fn marca_padrao_vs_acento_manual_preserva_preferencias_antigas() {
        let mut c=Configuracao::default();
        assert!(!c.logo_usa_acento());
        c.acento=2; // Preferência legada de uma cor escolhida.
        assert!(c.logo_usa_acento());
        c.marca_personalizada=Some(false);
        assert!(!c.logo_usa_acento()); // Restaurar apenas a logo.
        c.marca_personalizada=Some(true);
        assert!(c.logo_usa_acento()); // Acento explícito, mesmo índice 0.
        c.acento=0;
        assert!(c.logo_usa_acento());
    }
    #[test]
    fn preferencias_e_atalhos_persistem_sem_sujar_diretorio_do_usuario() {
        let p = arquivo_teste();
        let mut conf = Configuracao::default();
        conf.escuro = false;
        conf.tema_auto = true;
        conf.acento = 3;
        conf.painel_notas = false;
        conf.barra_propria = false;
        conf.intervalo = 17;
        conf.atalhos.insert("salvar".into(), "Ctrl+Alt+K".into());
        conf.gravar_em(&p).unwrap();
        let lida = Configuracao::carregar_de(&p);
        assert!(!lida.escuro && lida.tema_auto && !lida.painel_notas && !lida.barra_propria);
        assert_eq!(lida.acento, 3);
        assert_eq!(lida.intervalo, 17);
        assert_eq!(lida.atalho("salvar"), "Ctrl+Alt+K");
        assert_eq!(lida.comando_de("ctrl+alt+k"), Some("salvar"));
        assert_eq!(lida.atalho("abrir"), "Ctrl+O");
        std::fs::remove_dir_all(p.parent().unwrap()).unwrap();
    }

    #[test]
    fn preferencias_legadas_e_json_corrompido_nao_caem_em_panic() {
        let p = arquivo_teste();
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, br#"{"escuro":false,"atalhos":{"novo":"Ctrl+Alt+N"}}"#).unwrap();
        let lida = Configuracao::carregar_de(&p);
        assert!(!lida.escuro);
        assert!(!lida.tema_auto, "a configuração legada deve permanecer manual");
        assert_eq!(lida.atalho("novo"), "Ctrl+Alt+N");
        assert_eq!(lida.intervalo, Configuracao::default().intervalo);
        std::fs::write(&p, br#"{"recentes":"#).unwrap();
        let fallback = Configuracao::carregar_de(&p);
        assert_eq!(fallback.intervalo, Configuracao::default().intervalo);
        assert_eq!(fallback.atalho("salvar"), "Ctrl+S");
        std::fs::remove_dir_all(p.parent().unwrap()).unwrap();
    }
}
