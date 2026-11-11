//! A estrutura de tópicos: a apresentação como texto — o título de cada slide numa linha e o
//! corpo logo abaixo, recuado (Tab ou espaços). Escrever ali e aplicar muda os slides numa edição.

use crate::modelo::Papel;
use crate::sessao::{slide_com_layout, Mudanca, Sessao};

/// Um slide da estrutura: o título e as linhas do corpo.
fn blocos(texto: &str) -> Vec<(String, Vec<String>)> {
    let mut v: Vec<(String, Vec<String>)> = Vec::new();
    for linha in texto.lines() {
        if linha.trim().is_empty() {
            continue;
        }
        let recuada = linha.starts_with('\t') || linha.starts_with(' ');
        match v.last_mut() {
            Some((_, corpo)) if recuada => {
                // Tira um nível de recuo (um Tab ou até 4 espaços); os níveis de dentro ficam.
                let sem = linha.strip_prefix('\t').unwrap_or_else(|| {
                    let n = linha.len() - linha.trim_start_matches(' ').len();
                    &linha[n.min(4)..]
                });
                corpo.push(sem.to_string());
            }
            _ => v.push((linha.trim().to_string(), Vec::new())),
        }
    }
    v
}

impl Sessao {
    /// A estrutura de tópicos da apresentação inteira.
    pub fn topicos(&self) -> String {
        let mut saida = String::new();
        for s in &self.ap.slides {
            let titulo = s.elementos.iter().find(|e| e.papel == Papel::Titulo).map_or("", |e| e.texto.as_str());
            saida.push_str(if titulo.trim().is_empty() { "(sem título)" } else { titulo.lines().next().unwrap_or("") });
            saida.push('\n');
            if let Some(corpo) = s.elementos.iter().find(|e| matches!(e.papel, Papel::Corpo | Papel::Subtitulo)) {
                for l in corpo.texto.lines().filter(|l| !l.trim().is_empty()) {
                    saida.push('\t');
                    saida.push_str(l);
                    saida.push('\n');
                }
            }
        }
        saida
    }

    /// Aplica a estrutura: o n-ésimo título vai para o n-ésimo slide (e o corpo, para a primeira
    /// caixa de corpo ou subtítulo dele); títulos a mais viram slides novos de título e conteúdo.
    /// Slides a mais no fim ficam como estão — nada é apagado por aqui.
    pub fn topicos_aplicar(&mut self, texto: &str) -> Result<Mudanca, String> {
        let blocos = blocos(texto);
        if blocos.is_empty() {
            return Err("A estrutura está vazia: escreva um título por linha.".into());
        }
        self.editar(|s| {
            let antes = s.ap.slides.len();
            for (k, (titulo, corpo)) in blocos.iter().enumerate() {
                if k >= s.ap.slides.len() {
                    let novo = slide_com_layout(&mut s.ap, "titulo_conteudo");
                    s.ap.slides.push(novo);
                }
                let slide = &mut s.ap.slides[k];
                if titulo != "(sem título)" {
                    // A estrutura mostra só a 1ª linha do título: igual a ela, o título (e as linhas de baixo) fica.
                    if let Some(e) = slide.elementos.iter_mut().find(|e| e.papel == Papel::Titulo && e.texto.lines().next().unwrap_or("") != titulo) {
                        crate::texto_formato::atualizar_indices(e, titulo);
                        e.texto.clone_from(titulo);
                    }
                }
                if let Some(e) = slide.elementos.iter_mut().find(|e| matches!(e.papel, Papel::Corpo | Papel::Subtitulo)) {
                    let novo = corpo.join("\n");
                    // Só troca se o texto mudou: guarda a formatação por trecho de quem não mexeu.
                    if e.texto.lines().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join("\n") != novo {
                        e.texto = novo;
                        e.indices_texto.clear();
                    }
                }
            }
            Ok(((0..blocos.len().max(antes).min(s.ap.slides.len())).collect(), s.ap.slides.len() != antes))
        })
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn estrutura_vai_e_volta_e_cria_slides() {
        let mut s = Sessao::default();
        s.topicos_aplicar("Primeiro\n\tsubtítulo\nSegundo\n\tum\n    dois\n\t\tdentro\nTerceiro\n").unwrap();
        assert_eq!(s.apresentacao().slides.len(), 3);
        let t = s.topicos();
        assert!(t.starts_with("Primeiro\n\tsubtítulo\nSegundo\n\tum\n\tdois\n\t\tdentro\nTerceiro\n"), "{t}");
        // Um Ctrl+Z desfaz tudo.
        s.desfazer().unwrap();
        assert_eq!(s.apresentacao().slides.len(), 1);
        assert!(s.topicos_aplicar("  \n").is_err());
    }

    #[test]
    fn titulo_de_duas_linhas_nao_encolhe_e_indices_seguem_o_texto() {
        let mut s = Sessao::default();
        let id = s.slide().elementos.iter().find(|e| e.papel == Papel::Titulo).unwrap().id;
        s.texto_definir(id, "Resultados\n2026").unwrap();
        s.topicos_aplicar(&s.topicos()).unwrap();
        assert_eq!(s.slide().elementos.iter().find(|e| e.id == id).unwrap().texto, "Resultados\n2026");
        // Sobrescrito no fim e título trocado por um mais curto: o trecho não pode passar do texto.
        s.texto_definir(id, "Ab2").unwrap();
        s.texto_indice(id, 2, 3, 1).unwrap();
        s.topicos_aplicar("X").unwrap();
        let e = s.slide().elementos.iter().find(|e| e.id == id).unwrap();
        assert_eq!(e.texto, "X");
        assert!(crate::texto_formato::validar(e).is_ok());
    }

    #[test]
    fn menos_titulos_nao_apaga_slides() {
        let mut s = Sessao::default();
        s.slide_novo("titulo_conteudo").unwrap();
        s.topicos_aplicar("Só um").unwrap();
        assert_eq!(s.apresentacao().slides.len(), 2);
    }
}
