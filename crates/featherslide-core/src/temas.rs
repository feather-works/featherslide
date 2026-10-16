//! Os temas: um conjunto de cor de fundo, cores e fontes de texto, aplicado à apresentação
//! inteira (o Design). Trocar de tema recolore os slides e os textos que já existem, e os
//! slides novos já nascem no tema.

use crate::modelo::{Apresentacao, Elemento, Papel, Slide, Tipo};

/// Um tema pronto.
pub struct Tema {
    pub id: &'static str,
    pub nome: &'static str,
    pub fundo: &'static str,
    pub titulo: &'static str,
    pub texto: &'static str,
    /// A cor de destaque: formas e gráficos que usavam a do tema anterior passam a usar esta.
    pub destaque: &'static str,
    pub fonte_titulo: &'static str,
    pub fonte_texto: &'static str,
}

/// Os temas, na ordem da galeria. O primeiro é o padrão.
pub const TEMAS: [Tema; 25] = [
    Tema { id: "office", nome: "Office", fundo: "#ffffff", titulo: "#202124", texto: "#202124", destaque: "#1a73e8", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "escuro", nome: "Escuro", fundo: "#202124", titulo: "#ffffff", texto: "#e8eaed", destaque: "#8ab4f8", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "papel", nome: "Papel", fundo: "#fbf4e6", titulo: "#4a3a24", texto: "#4a3a24", destaque: "#9a6b2f", fonte_titulo: "Liberation Serif", fonte_texto: "Liberation Serif" },
    Tema { id: "oceano", nome: "Oceano", fundo: "#0d2b45", titulo: "#ffffff", texto: "#d6e6f2", destaque: "#4fc3f7", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "floresta", nome: "Floresta", fundo: "#f2f7f2", titulo: "#1b4332", texto: "#2d3a2f", destaque: "#2d6a4f", fonte_titulo: "Liberation Serif", fonte_texto: "Liberation Sans" },
    Tema { id: "por_do_sol", nome: "Pôr do sol", fundo: "#fff4ec", titulo: "#8c2f1b", texto: "#4a2c24", destaque: "#e8710a", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "ardosia", nome: "Ardósia", fundo: "#2e3440", titulo: "#eceff4", texto: "#d8dee9", destaque: "#88c0d0", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "neon", nome: "Neon", fundo: "#0b1120", titulo: "#f8fafc", texto: "#cbd5e1", destaque: "#22d3ee", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "vinho", nome: "Vinho", fundo: "#fdf6f6", titulo: "#4a1425", texto: "#4b3a3f", destaque: "#9b2c4a", fonte_titulo: "Liberation Serif", fonte_texto: "Liberation Serif" },
    Tema { id: "solar", nome: "Solar", fundo: "#fffaf0", titulo: "#7c2d12", texto: "#44403c", destaque: "#f97316", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "roxo", nome: "Roxo", fundo: "#1e1b4b", titulo: "#ffffff", texto: "#c7d2fe", destaque: "#a78bfa", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "grafite", nome: "Grafite", fundo: "#f5f5f4", titulo: "#1c1917", texto: "#44403c", destaque: "#57534e", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "negocio", nome: "Negócio", fundo: "#f7f6f2", titulo: "#12263a", texto: "#3a4a5a", destaque: "#1f7a6d", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "caderno", nome: "Caderno", fundo: "#fdfcf7", titulo: "#2b3a55", texto: "#2b3a55", destaque: "#e4575f", fonte_titulo: "Liberation Serif", fonte_texto: "Liberation Serif" },
    Tema { id: "obra", nome: "Obra", fundo: "#ecece8", titulo: "#1a1f24", texto: "#38424c", destaque: "#f26a1b", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "memo", nome: "Memorando", fundo: "#f4f5f7", titulo: "#111827", texto: "#2b3340", destaque: "#b91c1c", fonte_titulo: "Liberation Serif", fonte_texto: "Liberation Serif" },
    Tema { id: "palco", nome: "Palco", fundo: "#14141b", titulo: "#ffffff", texto: "#d4d4dc", destaque: "#ffcf4a", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "conferencia", nome: "Conferência", fundo: "#f2f2ef", titulo: "#0d0d0d", texto: "#2a2a2a", destaque: "#2b3fff", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "executivo", nome: "Executivo", fundo: "#fbfbfd", titulo: "#0b1f3a", texto: "#334155", destaque: "#b8892b", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "rota", nome: "Rota", fundo: "#f7f9fc", titulo: "#13294b", texto: "#3b4a63", destaque: "#0e9f6e", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "deco", nome: "Art déco", fundo: "#10231f", titulo: "#f2e6c9", texto: "#d8cdb0", destaque: "#c9a45c", fonte_titulo: "Liberation Serif", fonte_texto: "Liberation Serif" },
    Tema { id: "planta", nome: "Planta", fundo: "#14367a", titulo: "#ffffff", texto: "#cfe0ff", destaque: "#8fc1ff", fonte_titulo: "Liberation Mono", fonte_texto: "Liberation Sans" },
    Tema { id: "bauhaus", nome: "Bauhaus", fundo: "#fafaf7", titulo: "#111111", texto: "#222222", destaque: "#e63946", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "orbita", nome: "Órbita", fundo: "#25235c", titulo: "#ffffff", texto: "#d9d7ff", destaque: "#ff8aa1", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
    Tema { id: "contraste", nome: "Alto contraste", fundo: "#000000", titulo: "#ffff00", texto: "#ffffff", destaque: "#00ffff", fonte_titulo: "Liberation Sans", fonte_texto: "Liberation Sans" },
];

/// O tema pelo id (o desconhecido ou vazio vira o padrão).
pub fn tema(id: &str) -> &'static Tema {
    TEMAS.iter().find(|t| t.id == id).unwrap_or(&TEMAS[0])
}

fn recolorir(e: &mut Elemento, antigo: &Tema, novo: &Tema) {
    if e.papel == Papel::Fundo {
        return;
    }
    if e.tipo.aceita_texto() {
        let titulo = e.papel == Papel::Titulo;
        e.estilo.cor = if titulo { novo.titulo } else if e.papel == Papel::Subtitulo { novo.texto } else { novo.texto }.into();
        e.estilo.fonte = if titulo { novo.fonte_titulo } else { novo.fonte_texto }.into();
    }
    if e.preenchimento.eq_ignore_ascii_case(antigo.destaque) {
        e.preenchimento = novo.destaque.into();
    }
    if e.contorno.eq_ignore_ascii_case(antigo.destaque) {
        e.contorno = novo.destaque.into();
    }
    if e.tipo == Tipo::Tabela && e.preenchimento.is_empty() {
        // o cabeçalho da tabela acompanha o destaque só se a pessoa mexer; padrão fica claro
    }
}

/// Aplica o tema aos elementos e ao fundo de um slide.
pub fn aplicar_no_slide(slide: &mut Slide, antigo: &Tema, novo: &Tema) {
    slide.fundo = if novo.id == TEMAS[0].id { String::new() } else { novo.fundo.into() };
    for e in &mut slide.elementos {
        recolorir(e, antigo, novo);
    }
}

/// Aplica o tema a todos os slides e o grava na apresentação.
pub fn aplicar(ap: &mut Apresentacao, id: &str) {
    let antigo = tema(&ap.tema);
    let novo = tema(id);
    for s in &mut ap.slides {
        aplicar_no_slide(s, antigo, novo);
    }
    ap.tema = novo.id.into();
}
