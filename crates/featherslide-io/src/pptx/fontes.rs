//! Leitura e decodificação de fontes embutidas de apresentações `.pptx`.
//!
//! O PowerPoint e o Google Apresentações podem embutir fontes no arquivo:
//! - `.fntdata` em formato Embedded OpenType (EOT), comprimido com MicroType Express (MTX);
//! - `.odttf` ofuscado com a GUID do nome da parte ou do atributo `fontKey` (ECMA-376);
//! - Fontes TTF/OTF puras.

use std::collections::HashMap;

use featherslide_core::modelo::FonteEmbutida;

use crate::pacote::Pacote;
use crate::xml::No;

/// Lê as fontes embutidas em `ppt/presentation.xml` (`<p:embeddedFontLst>`).
pub(super) fn ler_fontes(pres: &No, pac: &mut Pacote, rels: &HashMap<String, String>) -> Vec<FonteEmbutida> {
    let Some(lista) = pres.filho("p:embeddedFontLst") else { return Vec::new() };
    let mut fontes = Vec::new();
    for ef in lista.filhos("p:embeddedFont") {
        let Some(nome) = ef.filho("p:font").and_then(|f| f.attr("typeface")).filter(|n| !n.is_empty()) else { continue };
        let regular = ler_variante(ef, "p:regular", pac, rels);
        let negrito = ler_variante(ef, "p:bold", pac, rels);
        let italico = ler_variante(ef, "p:italic", pac, rels);
        let negrito_italico = ler_variante(ef, "p:boldItalic", pac, rels);
        if regular.is_some() || negrito.is_some() || italico.is_some() || negrito_italico.is_some() {
            fontes.push(FonteEmbutida {
                nome: nome.to_string(),
                regular,
                negrito,
                italico,
                negrito_italico,
            });
        }
    }
    fontes
}

/// Lê e decodifica os bytes de uma variante de fonte.
fn ler_variante(ef: &No, tag: &str, pac: &mut Pacote, rels: &HashMap<String, String>) -> Option<Vec<u8>> {
    let no = ef.filho(tag)?;
    let r_id = no.attr("r:id")?;
    let alvo = rels.get(r_id)?;
    if !pac.tem(alvo) {
        return None;
    }
    let bytes = pac.parte(alvo).ok()?;
    let font_key = no.attr("fontKey");
    decodificar_fonte(&bytes, alvo, font_key)
}

/// É um formato de fonte TrueType ou OpenType válido?
fn e_ttf(bytes: &[u8]) -> bool {
    if bytes.len() < 4 {
        return false;
    }
    matches!(&bytes[..4], [0x00, 0x01, 0x00, 0x00] | b"true" | b"OTTO" | b"typ1")
}

/// Decodifica os bytes da fonte: puro, EOT (MicroType Express) ou ofuscado por GUID.
pub(crate) fn decodificar_fonte(bytes: &[u8], caminho: &str, font_key: Option<&str>) -> Option<Vec<u8>> {
    if bytes.len() < 4 {
        return None;
    }
    if e_ttf(bytes) {
        return Some(bytes.to_vec());
    }
    if let Some(ttf) = decodificar_eot(bytes) {
        return Some(ttf);
    }
    if let Some(ttf) = desofuscar_guid(bytes, caminho, font_key) {
        return Some(ttf);
    }
    None
}

/// Decodifica fonte EOT (Embedded OpenType) para TTF padrão.
fn decodificar_eot(bytes: &[u8]) -> Option<Vec<u8>> {
    match eot_parse::eot_to_ttf(bytes) {
        Ok((_, ttf)) if e_ttf(&ttf) => Some(ttf),
        _ => None,
    }
}

/// Desofusca fonte protegida com XOR de GUID (ECMA-376 Parte 2).
fn desofuscar_guid(bytes: &[u8], caminho: &str, font_key: Option<&str>) -> Option<Vec<u8>> {
    if bytes.len() < 32 {
        return None;
    }
    let chave = font_key.and_then(extrair_guid).or_else(|| extrair_guid(caminho))?;
    // O padrão ECMA-376 inverte a ordem dos bytes da GUID.
    let chave_invertida = {
        let mut inv = [0u8; 16];
        for i in 0..16 {
            inv[i] = chave[15 - i];
        }
        inv
    };
    for k in [&chave_invertida, &chave] {
        let mut saida = bytes.to_vec();
        for i in 0..32 {
            saida[i] ^= k[i % 16];
        }
        if e_ttf(&saida) {
            return Some(saida);
        }
    }
    None
}

/// Extrai os 16 bytes de uma GUID a partir de texto contendo 32 caracteres hexadecimais.
fn extrair_guid(texto: &str) -> Option<[u8; 16]> {
    let hex: Vec<u8> = texto.bytes().filter(|b| b.is_ascii_hexdigit()).collect();
    if hex.len() < 32 {
        return None;
    }
    // Pega os primeiros 32 caracteres hexadecimais
    let mut chave = [0u8; 16];
    for i in 0..16 {
        let h1 = char::from(hex[i * 2]).to_digit(16)? as u8;
        let h2 = char::from(hex[i * 2 + 1]).to_digit(16)? as u8;
        chave[i] = (h1 << 4) | h2;
    }
    Some(chave)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn ttf_puro_retorna_os_mesmos_bytes() {
        let mut ttf = vec![0x00, 0x01, 0x00, 0x00];
        ttf.extend(vec![0xAA; 100]);
        let dec = decodificar_fonte(&ttf, "fontes/minha.ttf", None);
        assert_eq!(dec, Some(ttf));
    }

    #[test]
    fn desofusca_guid_ecma376() {
        let mut original = vec![0x00, 0x01, 0x00, 0x00];
        original.extend(vec![0x42; 60]);
        let guid_str = "{ba81b498-583e-42a0-838d-ae3f418751ab}";
        let chave = extrair_guid(guid_str).expect("guid valida");
        let mut chave_inv = [0u8; 16];
        for i in 0..16 {
            chave_inv[i] = chave[15 - i];
        }
        let mut ofuscado = original.clone();
        for i in 0..32 {
            ofuscado[i] ^= chave_inv[i % 16];
        }
        assert!(!e_ttf(&ofuscado));
        let dec = decodificar_fonte(&ofuscado, "fonts/qualquer.odttf", Some(guid_str));
        assert_eq!(dec, Some(original));
    }
}
