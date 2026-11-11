//! Data, número do slide e rodapé (o "Cabeçalho e rodapé"): caixas de texto geradas na hora de
//! montar o quadro, nunca guardadas no slide — o número muda sozinho quando os slides mudam de
//! lugar, e a data é a de hoje. São `Papel::Fundo`: aparecem na tela, no PDF e na apresentação,
//! mas não se selecionam.

use crate::modelo::{Alinhamento, Apresentacao, Elemento, Papel, Rodape, Tipo, Vertical};
use crate::sessao::{Mudanca, Sessao};

/// O maior número que o primeiro slide pode receber.
pub const NUMERO_INICIAL_MAXIMO: u32 = 9999;

/// Ids das caixas geradas: no topo do espaço de ids, longe dos elementos de verdade.
const PRIMEIRO_ID: u64 = u64::MAX - 8;

/// A data de hoje (`dd/mm/aaaa`) pelo relógio UTC, quando a interface não mandou a local.
fn hoje_utc() -> String {
    let iso = crate::metadados::data_utc(crate::metadados::agora_utc());
    match (iso.get(0..4), iso.get(5..7), iso.get(8..10)) {
        (Some(a), Some(m), Some(d)) => format!("{d}/{m}/{a}"),
        _ => String::new(),
    }
}

/// As caixas do rodapé do slide `indice` (vazio se não há rodapé ou é um slide de título excluído).
pub fn elementos(ap: &Apresentacao, indice: usize, hoje: &str) -> Vec<Elemento> {
    let r = &ap.rodape;
    let Some(slide) = ap.slides.get(indice) else { return Vec::new() };
    if !r.ativo() || (r.sem_no_titulo && slide.layout == "titulo") {
        return Vec::new();
    }
    let (w, h) = ap.tamanho();
    let k = h / 540.0;
    let caixa = |n: u64, x: f32, largura: f32, texto: String, alinhamento: Alinhamento| {
        let mut e = Elemento { id: PRIMEIRO_ID + n, tipo: Tipo::Texto, papel: Papel::Fundo, x, y: h - 40.0 * k, largura, altura: 30.0 * k, texto, ..Elemento::default() };
        e.estilo.tamanho = 12.0 * k;
        e.estilo.cor = "#5f6368".into();
        e.estilo.alinhamento = alinhamento;
        e.estilo.vertical = Vertical::Meio;
        e
    };
    let mut v = Vec::new();
    if r.data {
        let data = if r.data_fixa.trim().is_empty() { if hoje.is_empty() { hoje_utc() } else { hoje.to_string() } } else { r.data_fixa.clone() };
        v.push(caixa(0, 0.05 * w, 0.25 * w, data, Alinhamento::Esquerda));
    }
    if !r.texto.trim().is_empty() {
        v.push(caixa(1, 0.33 * w, 0.34 * w, r.texto.clone(), Alinhamento::Centro));
    }
    if r.numero {
        v.push(caixa(2, 0.70 * w, 0.25 * w, (indice as u64 + u64::from(r.numero_inicial)).to_string(), Alinhamento::Direita));
    }
    v
}

impl Sessao {
    /// O rodapé da apresentação.
    pub fn rodape(&self) -> &Rodape {
        &self.ap.rodape
    }

    /// Troca o rodapé da apresentação inteira (uma edição; todos os slides mudam de desenho).
    pub fn rodape_definir(&mut self, r: Rodape) -> Result<Mudanca, String> {
        if r.texto.chars().count() > 500 || r.data_fixa.chars().count() > 100 {
            return Err("O texto do rodapé é longo demais.".into());
        }
        if r.numero_inicial > NUMERO_INICIAL_MAXIMO {
            return Err(format!("O primeiro número vai de 0 a {NUMERO_INICIAL_MAXIMO}."));
        }
        self.editar(|s| {
            s.ap.rodape = r;
            Ok(((0..s.ap.slides.len()).collect(), false))
        })
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn numero_segue_a_posicao_e_titulo_pode_ficar_sem() {
        let mut s = Sessao::default();
        s.slide_novo("titulo_conteudo").unwrap();
        s.rodape_definir(Rodape { numero: true, texto: "Empresa".into(), sem_no_titulo: true, data: true, data_fixa: "01/02/2026".into(), ..Rodape::default() }).unwrap();
        assert!(elementos(s.apresentacao(), 0, "").is_empty(), "o slide 1 é de título");
        let e = elementos(s.apresentacao(), 1, "");
        let textos: Vec<&str> = e.iter().map(|e| e.texto.as_str()).collect();
        assert_eq!(textos, vec!["01/02/2026", "Empresa", "2"]);
        assert!(e.iter().all(|e| e.papel == Papel::Fundo));
        // No quadro, as caixas aparecem; mover o slide muda o número.
        let q = s.quadro(1, None, false).unwrap().unwrap();
        assert!(q.elementos.iter().any(|d| d.linhas.iter().any(|l| l.texto == "2")));
        s.desfazer().unwrap();
        assert!(elementos(s.apresentacao(), 1, "").is_empty());
    }

    #[test]
    fn numeracao_comeca_onde_a_pessoa_pediu() {
        let mut s = Sessao::default();
        s.slide_novo("em_branco").unwrap();
        s.rodape_definir(Rodape { numero: true, numero_inicial: 0, ..Rodape::default() }).unwrap();
        assert_eq!(elementos(s.apresentacao(), 1, "")[0].texto, "1", "capa = 0, o segundo slide é o 1");
        s.rodape_definir(Rodape { numero: true, numero_inicial: 12, ..Rodape::default() }).unwrap();
        assert_eq!(elementos(s.apresentacao(), 1, "")[0].texto, "13");
        assert!(s.rodape_definir(Rodape { numero: true, numero_inicial: 10_000, ..Rodape::default() }).is_err());
    }

    #[test]
    fn data_de_hoje_vem_da_interface_ou_do_relogio() {
        let mut s = Sessao::default();
        s.rodape_definir(Rodape { data: true, ..Rodape::default() }).unwrap();
        assert_eq!(elementos(s.apresentacao(), 0, "05/11/2026")[0].texto, "05/11/2026");
        let utc = &elementos(s.apresentacao(), 0, "")[0].texto;
        assert_eq!(utc.len(), 10, "{utc}");
    }
}
