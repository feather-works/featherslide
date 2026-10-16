//! Propriedades do documento importadas dos metadados opcionais de PPTX e ODP.
//! A ausência de valores não inventa datas/autores. Não executa macros nem busca serviços.

use featherslide_core::metadados::{novo_identificador, Metadados};
use crate::pacote::Pacote;
use crate::xml::No;

fn filho_texto(no: &No, nome: &str) -> String {
    no.filho(nome).map(|n| n.texto.trim().to_string()).unwrap_or_default()
}

/// ISO-8601 UTC com ou sem frações; aceita sufixos Z ou offset ±HH:MM.
fn data_iso(s: &str) -> Option<u64> {
    let (d, t) = s.trim().split_once('T').or_else(|| s.trim().split_once(' '))?;
    let mut dp = d.split('-');
    let (ano, mes, dia): (i64, i64, i64) = (
        dp.next()?.parse().ok()?, dp.next()?.parse().ok()?, dp.next()?.parse().ok()?
    );
    if !(1970..=9999).contains(&ano) || !(1..=12).contains(&mes) || !(1..=31).contains(&dia) {
        return None;
    }
    let h: i64 = t.get(0..2)?.parse().ok()?;
    let m: i64 = t.get(3..5)?.parse().ok()?;
    let sec: i64 = t.get(6..8)?.parse().ok()?;
    if t.get(2..3)? != ":" || t.get(5..6)? != ":" || h > 23 || m > 59 || sec > 59 {
        return None;
    }
    let mais = &t[8..];
    let mais = mais.strip_prefix('.').map_or(mais, |f| {
        let i = f.find(|c: char| !c.is_ascii_digit()).unwrap_or(f.len());
        &f[i..]
    });
    let deslocamento = if mais.is_empty() || mais == "Z" {
        0
    } else {
        let sinal = if mais.starts_with('+') { 1 } else if mais.starts_with('-') { -1 } else { return None };
        let o = &mais[1..];
        let hh: i64 = o.get(0..2)?.parse().ok()?;
        let mm: i64 = o.get(3..5)?.parse().ok()?;
        if o.get(2..3)? != ":" || hh > 23 || mm > 59 { return None; }
        sinal * (hh * 3600 + mm * 60)
    };
    let bissexto = ano % 4 == 0 && (ano % 100 != 0 || ano % 400 == 0);
    let dias_mes = [31, if bissexto { 29 } else { 28 }, 31,30,31,30,31,31,30,31,30,31];
    if dia > dias_mes[mes as usize - 1] { return None; }
    // Algoritmo gregoriano days_from_civil, inverso do formatador do core.
    let y = ano - i64::from(mes <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = mes + if mes > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + dia - 1;
    let doe = yoe * 365 + yoe/4 - yoe/100 + doy;
    let dias = era * 146_097 + doe - 719_468;
    (dias * 86_400 + h * 3600 + m * 60 + sec - deslocamento).try_into().ok()
}

fn minutos(s: &str) -> u64 {
    s.trim().parse::<u64>().unwrap_or(0).saturating_mul(60)
}

fn duracao_iso(s: &str) -> u64 {
    let Some(resto) = s.trim().strip_prefix("PT") else { return 0 };
    let mut numero = String::new();
    let mut total = 0_u64;
    for c in resto.chars() {
        if c.is_ascii_digit() { numero.push(c); }
        else if let Ok(v) = numero.parse::<u64>() {
            total = total.saturating_add(match c {
                'H' => v.saturating_mul(3600), 'M' => v.saturating_mul(60), 'S' => v,
                _ => 0,
            });
            numero.clear();
        }
    }
    total
}

fn vazio_com_id() -> Metadados {
    Metadados { identificador: novo_identificador(), ..Metadados::default() }
}

/// Importa docProps/core.xml, app.xml e custom.xml quando presentes.
pub fn pptx(pac: &mut Pacote<'_>) -> (String, String, Metadados) {
    let mut dados = vazio_com_id();
    let mut titulo = String::new();
    let mut autor = String::new();
    if let Ok(core) = pac.xml("docProps/core.xml") {
        titulo = filho_texto(&core, "dc:title");
        autor = filho_texto(&core, "dc:creator");
        dados.assunto = filho_texto(&core, "dc:subject");
        dados.categoria = filho_texto(&core, "cp:category");
        dados.palavras_chave = filho_texto(&core, "cp:keywords");
        dados.comentarios = filho_texto(&core, "dc:description");
        dados.ultimo_autor = filho_texto(&core, "cp:lastModifiedBy");
        dados.revisoes = filho_texto(&core, "cp:revision").parse().unwrap_or(0);
        dados.criado_em_utc = data_iso(&filho_texto(&core, "dcterms:created")).unwrap_or(0);
        dados.modificado_em_utc = data_iso(&filho_texto(&core, "dcterms:modified")).unwrap_or(0);
        let idioma = filho_texto(&core, "dc:language");
        if !idioma.is_empty() { dados.idioma = idioma; }
        let estado = filho_texto(&core, "cp:contentStatus");
        if !estado.is_empty() { dados.estendidos.insert("Estado original".into(), estado); }
    }
    if let Ok(app) = pac.xml("docProps/app.xml") {
        dados.empresa = filho_texto(&app, "Company");
        dados.gerente = filho_texto(&app, "Manager");
        dados.tempo_edicao_segundos = minutos(&filho_texto(&app, "TotalTime"));
        for (origem, campo) in [
            ("Application", "Aplicativo de origem"), ("AppVersion", "Versão do aplicativo"),
            ("Template", "Modelo de origem"), ("PresentationFormat", "Formato original"),
        ] {
            let valor = filho_texto(&app, origem);
            if !valor.is_empty() { dados.estendidos.insert(campo.into(), valor); }
        }
    }
    if let Ok(custom) = pac.xml("docProps/custom.xml") {
        for prop in custom.filhos("property").take(100) {
            let Some(nome) = prop.attr("name") else { continue };
            if nome.is_empty() || nome.len() > 128 { continue; }
            let valor = prop.filhos.iter().find(|c| c.nome.starts_with("vt:"))
                .map(|c| c.texto.clone()).unwrap_or_default();
            if valor.len() <= 16_384 { dados.propriedades_personalizadas.insert(nome.into(), valor); }
        }
    }
    if dados.ultimo_autor.is_empty() { dados.ultimo_autor.clone_from(&autor); }
    (titulo, autor, dados)
}

/// Importa meta.xml de arquivos ODP. Estatísticas do ODP são recalculadas no motor.
pub fn odp(pac: &mut Pacote<'_>) -> (String, String, Metadados) {
    let mut dados = vazio_com_id();
    let mut titulo = String::new();
    let mut autor = String::new();
    if let Ok(raiz) = pac.xml("meta.xml") {
        if let Some(meta) = raiz.filho("office:meta") {
            titulo = filho_texto(meta, "dc:title");
            autor = filho_texto(meta, "meta:initial-creator");
            dados.assunto = filho_texto(meta, "dc:subject");
            dados.palavras_chave = meta.filhos("meta:keyword")
                .map(|f| f.texto.trim()).filter(|s| !s.is_empty())
                .collect::<Vec<_>>().join(", ");
            dados.comentarios = filho_texto(meta, "dc:description");
            dados.ultimo_autor = filho_texto(meta, "dc:creator");
            dados.revisoes = filho_texto(meta, "meta:editing-cycles").parse().unwrap_or(0);
            dados.criado_em_utc = data_iso(&filho_texto(meta, "meta:creation-date")).unwrap_or(0);
            dados.modificado_em_utc = data_iso(&filho_texto(meta, "dc:date")).unwrap_or(0);
            dados.tempo_edicao_segundos = duracao_iso(&filho_texto(meta, "meta:editing-duration"));
            let idioma = filho_texto(meta, "dc:language");
            if !idioma.is_empty() { dados.idioma = idioma; }
            for p in meta.filhos("meta:user-defined").take(100) {
                if let Some(chave) = p.attr("meta:name") {
                    if !chave.is_empty() && chave.len() <= 128 && p.texto.len() <= 16_384 {
                        dados.propriedades_personalizadas.insert(chave.into(), p.texto.clone());
                    }
                }
            }
            let gerador = filho_texto(meta, "meta:generator");
            if !gerador.is_empty() { dados.estendidos.insert("Aplicativo de origem".into(), gerador); }
        }
    }
    if dados.ultimo_autor.is_empty() { dados.ultimo_autor.clone_from(&autor); }
    (titulo, autor, dados)
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn datas_utc_e_fusos_sao_lidos() {
        assert_eq!(data_iso("2024-01-01T00:00:00Z"), Some(1_704_067_200));
        assert_eq!(data_iso("2024-01-01T02:00:00+02:00"), Some(1_704_067_200));
        assert_eq!(data_iso("2024-02-30T00:00:00Z"), None);
        assert_eq!(duracao_iso("PT1H2M3S"), 3_723);
    }
    #[test]
    fn metadados_odp_sao_importados_com_idioma_e_personalizados() {
        use std::io::{Cursor, Write};
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        zip.start_file("meta.xml", zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(r#"<office:document-meta><office:meta><dc:title>Apresentação técnica</dc:title><meta:initial-creator>Ana</meta:initial-creator><dc:creator>Bruno</dc:creator><dc:subject>Energia</dc:subject><dc:language>pt-BR</dc:language><meta:keyword>dados</meta:keyword><meta:keyword>energia</meta:keyword><meta:editing-cycles>7</meta:editing-cycles><meta:editing-duration>PT2H5M</meta:editing-duration><meta:creation-date>2024-01-01T00:00:00Z</meta:creation-date><meta:user-defined meta:name="Unidade">ES</meta:user-defined></office:meta></office:document-meta>"#.as_bytes()).unwrap();
        let bytes = zip.finish().unwrap().into_inner();
        let mut pacote = Pacote::abrir(&bytes, ".odp").unwrap();
        let (titulo, autor, m) = odp(&mut pacote);
        assert_eq!(titulo, "Apresentação técnica");
        assert_eq!(autor, "Ana");
        assert_eq!(m.ultimo_autor, "Bruno");
        assert_eq!(m.assunto, "Energia");
        assert_eq!(m.palavras_chave, "dados, energia");
        assert_eq!(m.revisoes, 7);
        assert_eq!(m.tempo_edicao_segundos, 7500);
        assert_eq!(m.criado_em_utc, 1_704_067_200);
        assert_eq!(m.propriedades_personalizadas["Unidade"], "ES");
    }

    #[test]
    fn metadados_pptx_reais_do_pacote() {
        use std::io::{Cursor, Write};
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let op = zip::write::SimpleFileOptions::default();
        zip.start_file("docProps/core.xml", op).unwrap();
        zip.write_all(br#"<cp:coreProperties><dc:title>Reuniao</dc:title><dc:creator>Ana</dc:creator><dc:subject>Projeto</dc:subject><cp:revision>3</cp:revision><dcterms:created>2024-01-01T00:00:00Z</dcterms:created></cp:coreProperties>"#).unwrap();
        zip.start_file("docProps/app.xml", op).unwrap();
        zip.write_all(br#"<Properties><Company>Companhia</Company><Manager>Gerente</Manager><TotalTime>8</TotalTime></Properties>"#).unwrap();
        let bytes = zip.finish().unwrap().into_inner();
        let mut pacote = Pacote::abrir(&bytes, ".pptx").unwrap();
        let (titulo, autor, m) = pptx(&mut pacote);
        assert_eq!((titulo.as_str(), autor.as_str()), ("Reuniao", "Ana"));
        assert_eq!(m.assunto, "Projeto");
        assert_eq!(m.revisoes, 3);
        assert_eq!(m.criado_em_utc, 1_704_067_200);
        assert_eq!(m.empresa, "Companhia");
        assert_eq!(m.tempo_edicao_segundos, 480);
    }
}
