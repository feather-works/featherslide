//! Configuração do slide: dimensões, ajuste de conteúdo e fundo local.
use crate::modelo::{Elemento, Papel, Tipo};
use crate::sessao::{Mudanca, Sessao};
use serde::{Deserialize, Serialize};

/// Aparência do fundo. A cor principal fica em `Slide::fundo`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fundo {
    /// `solido`, `imagem`, `gradiente`, `textura` ou `padrao`.
    pub tipo: String,
    pub cor2: String,
    pub imagem: crate::modelo::Imagem,
    /// Opacidade entre zero e um.
    pub opacidade: f32,
}
impl Default for Fundo {
    fn default() -> Self {
        Self {
            tipo: "solido".into(),
            cor2: "#ffffff".into(),
            imagem: crate::modelo::Imagem::default(),
            opacidade: 1.0,
        }
    }
}
/// Tratamento do conteúdo ao trocar dimensões.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ajuste {
    Esticar,
    Ajustar,
    Maximizar,
}

/// Valida uma dimensão em pontos, antes de editar ou carregar o documento.
pub fn validar_dimensoes(w: f32, h: f32) -> Result<(), String> {
    if !w.is_finite()
        || !h.is_finite()
        || !(36.0..=7200.0).contains(&w)
        || !(36.0..=7200.0).contains(&h)
    {
        return Err("Cada dimensão deve estar entre 36 e 7200 pontos.".into());
    }
    Ok(())
}
fn rgb(cor: &str) -> Result<[u8; 3], String> {
    let s = cor.strip_prefix('#').unwrap_or("");
    if ![6, 8].contains(&s.len()) || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Use cores no formato #RRGGBB.".into());
    }
    let n = u32::from_str_radix(&s[..6], 16).map_err(|_| "Cor inválida.")?;
    Ok([(n >> 16) as u8, (n >> 8) as u8, n as u8])
}
pub(crate) fn cor_alpha(cor: &str, opacidade: f32) -> String {
    if !cor.is_ascii() || (cor.len() != 7 && cor.len() != 9) {
        return cor.to_string();
    }
    let alfa = if cor.len() == 9 {
        u8::from_str_radix(&cor[7..], 16).unwrap_or(255)
    } else {
        255
    };
    let alfa = (alfa as f32 * opacidade).round() as u8;
    if alfa == 255 {
        cor[..7].into()
    } else {
        format!("{}{:02x}", &cor[..7], alfa)
    }
}

impl Fundo {
    /// Valida parâmetros do fundo sem modificar o documento.
    pub fn validar(&self, cor: &str) -> Result<(), String> {
        rgb(cor)?;
        rgb(&self.cor2)?;
        if !self.opacidade.is_finite() || !(0.0..=1.0).contains(&self.opacidade) {
            return Err("A opacidade deve estar entre 0 e 100%.".into());
        }
        if !["solido", "imagem", "gradiente", "textura", "padrao"].contains(&self.tipo.as_str()) {
            return Err("Tipo de fundo inválido.".into());
        }
        if self.tipo == "imagem" && !self.imagem.starts_with("data:image/") {
            return Err("Escolha uma imagem local para o fundo.".into());
        }
        Ok(())
    }
}
impl Sessao {
    /// Troca as dimensões em uma única transação, ajustando posição e tamanho.
    pub fn tamanho_slide_definir(
        &mut self,
        w: f32,
        h: f32,
        ajuste: Ajuste,
    ) -> Result<Mudanca, String> {
        validar_dimensoes(w, h)?;
        self.editar(|s| {
            let (ant_w, ant_h) = s.ap.tamanho();
            let (sx, sy) = (w / ant_w, h / ant_h);
            let fator = match ajuste {
                Ajuste::Maximizar => sx.max(sy),
                _ => sx.min(sy),
            };
            let (fx, fy) = if ajuste == Ajuste::Esticar {
                (sx, sy)
            } else {
                (fator, fator)
            };
            let (dx, dy) = ((w - ant_w * fx) / 2.0, (h - ant_h * fy) / 2.0);
            for slide in &mut s.ap.slides {
                for e in &mut slide.elementos {
                    e.x = e.x * fx + dx;
                    e.y = e.y * fy + dy;
                    e.largura *= fx;
                    e.altura *= fy;
                    e.estilo.tamanho *= fator;
                    e.espessura *= fator;
                }
            }
            s.ap.dimensoes = [w, h];
            Ok(((0..s.ap.slides.len()).collect(), true))
        })
    }
    /// Aplica aparência do fundo ao slide atual ou a todos, com histórico.
    pub fn fundo_configurar(
        &mut self,
        cor: &str,
        fundo: Fundo,
        todos: bool,
        ocultar_mestre: bool,
    ) -> Result<Mudanca, String> {
        fundo.validar(cor)?;
        self.editar(|s| {
            let alvos: Vec<_> = if todos {
                (0..s.ap.slides.len()).collect()
            } else {
                vec![s.atual]
            };
            for &i in &alvos {
                s.ap.slides[i].fundo = cor.into();
                s.ap.slides[i].fundo_config = fundo.clone();
                s.ap.slides[i].ocultar_mestre = ocultar_mestre;
            }
            Ok((alvos, false))
        })
    }
    /// Copia todo o fundo atual para os demais slides.
    pub fn fundo_aplicar_todos(&mut self) -> Result<Mudanca, String> {
        let slide = self.slide();
        let cor = if slide.fundo.is_empty() {
            "#ffffff"
        } else {
            &slide.fundo
        }
        .to_string();
        self.fundo_configurar(&cor, slide.fundo_config.clone(), true, slide.ocultar_mestre)
    }
}

/// Elementos temporários do fundo: a tela, o PDF e as miniaturas usam a mesma geometria.
pub fn elementos_fundo(slide: &crate::modelo::Slide, tamanho: (f32, f32)) -> Vec<Elemento> {
    let f = &slide.fundo_config;
    if f.tipo == "solido" {
        return Vec::new();
    }
    let (w, h) = tamanho;
    let cor = if slide.fundo.is_empty() {
        "#ffffff"
    } else {
        &slide.fundo
    };
    let base = Elemento {
        id: slide.id,
        papel: Papel::Fundo,
        tipo: Tipo::Retangulo,
        largura: w,
        altura: h,
        espessura: 0.0,
        ..Elemento::default()
    };
    if f.tipo == "imagem" {
        let mut e = base;
        e.tipo = Tipo::Imagem;
        e.imagem = f.imagem.clone();
        e.opacidade = f.opacidade;
        return vec![e];
    }
    let (Ok(a), Ok(b)) = (rgb(cor), rgb(&f.cor2)) else {
        return Vec::new();
    };
    let mut elementos = Vec::new();
    if f.tipo == "gradiente" {
        for i in 0..128 {
            let t = i as f32 / 127.0;
            let c: Vec<_> = a
                .iter()
                .zip(b)
                .map(|(a, b)| (*a as f32 * (1.0 - t) + b as f32 * t).round() as u8)
                .collect();
            elementos.push(Elemento {
                x: w * i as f32 / 128.0,
                largura: w / 128.0 + 0.05,
                preenchimento: cor_alpha(
                    &format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]),
                    f.opacidade,
                ),
                ..base.clone()
            });
        }
    } else {
        elementos.push(Elemento {
            preenchimento: cor_alpha(cor, f.opacidade),
            ..base.clone()
        });
        // Textura pontilhada determinística; padrão xadrez vetorial.
        let passo = (if f.tipo == "textura" {
            16.0_f32
        } else {
            32.0_f32
        })
        .max(w.max(h) / 64.0);
        for r in 0..(h / passo).ceil() as usize {
            for c in 0..(w / passo).ceil() as usize {
                if f.tipo == "padrao" && (r + c) % 2 == 0 {
                    continue;
                }
                let lado = if f.tipo == "textura" {
                    1.5 + ((r * 13 + c * 7) % 3) as f32
                } else {
                    passo
                };
                let x = c as f32 * passo;
                let y = r as f32 * passo;
                elementos.push(Elemento {
                    x,
                    y,
                    largura: lado.min(w - x),
                    altura: lado.min(h - y),
                    preenchimento: cor_alpha(&f.cor2, f.opacidade),
                    ..base.clone()
                });
            }
        }
    }
    elementos
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn tamanho_orientacao_ajuste_e_historico() {
        let mut s = Sessao::default();
        s.elemento_inserir(Tipo::Retangulo, Some([0.0, 0.0, 960.0, 540.0]), None)
            .unwrap();
        let id = s.estado().selecao[0];
        s.tamanho_slide_definir(540.0, 960.0, Ajuste::Ajustar)
            .unwrap();
        let e = s.slide().elementos.iter().find(|e| e.id == id).unwrap();
        assert_eq!((e.largura, e.altura), (540.0, 303.75));
        assert_eq!(e.y, (960.0 - 303.75) / 2.0);
        s.desfazer().unwrap();
        assert_eq!(s.apresentacao().tamanho(), (960.0, 540.0));
        s.tamanho_slide_definir(540.0, 960.0, Ajuste::Maximizar)
            .unwrap();
        let e = s.slide().elementos.iter().find(|e| e.id == id).unwrap();
        assert!(e.largura > 540.0 && e.x < 0.0);
        assert_eq!(e.altura, 960.0);
        s.tamanho_slide_definir(300.0, 600.0, Ajuste::Esticar)
            .unwrap();
        assert_eq!(s.apresentacao().tamanho(), (300.0, 600.0));
        let antes = s.apresentacao().clone();
        assert!(s
            .tamanho_slide_definir(f32::NAN, 600.0, Ajuste::Ajustar)
            .is_err());
        assert_eq!(s.apresentacao(), &antes);
        s.proporcao_definir(crate::modelo::Proporcao::Larga)
            .unwrap();
        assert_eq!(s.apresentacao().tamanho(), (960.0, 540.0));
    }
    #[test]
    fn fundos_mestre_selecao_e_aplicar_a_todos() {
        let mut s = Sessao::default();
        s.slide_novo("em_branco").unwrap();
        s.elemento_inserir(Tipo::Retangulo, Some([20.0, 20.0, 40.0, 40.0]), None)
            .unwrap();
        s.selecao_para_fundo().unwrap();
        for tipo in ["gradiente", "textura", "padrao"] {
            let fundo = Fundo {
                tipo: tipo.into(),
                cor2: "#000000".into(),
                opacidade: 0.5,
                ..Fundo::default()
            };
            s.fundo_configurar("#ffffff", fundo, false, true).unwrap();
            let q = s.quadro(s.atual(), None, false).unwrap().unwrap();
            assert!(!q.elementos.is_empty());
            assert!(!q.elementos.iter().any(|e| e.x == 20.0 && e.y == 20.0));
            assert!(q.elementos.iter().all(|e| e.preenchimento.ends_with("80")));
            s.selecionar_tudo();
            assert!(s.estado().selecao.is_empty());
        }
        s.fundo_aplicar_todos().unwrap();
        assert!(s
            .apresentacao()
            .slides
            .iter()
            .all(|slide| slide.fundo_config == s.slide().fundo_config && slide.ocultar_mestre));
        s.desfazer().unwrap();
        assert_eq!(s.apresentacao().slides[0].fundo_config, Fundo::default());
        s.slide_fundo("#abcdef", false).unwrap();
        assert_eq!(s.slide().fundo_config, Fundo::default());
    }
    #[test]
    fn rejeita_fundo_invalido_sem_editar() {
        let mut s = Sessao::default();
        let antes = s.apresentacao().clone();
        let f = Fundo {
            opacidade: f32::INFINITY,
            ..Fundo::default()
        };
        assert!(s.fundo_configurar("#ffffff", f, false, false).is_err());
        assert!(s
            .fundo_configurar("inválida", Fundo::default(), false, false)
            .is_err());
        assert_eq!(s.apresentacao(), &antes);
    }
}
