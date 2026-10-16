//! IO-013: muitos slides, Unicode, metadados e comentários após FSdz ZIP/Arrow.
//! cargo run --manifest-path featherslide/crates/featherslide-io/Cargo.toml \
//!   --example io013_extremos -j 2
use featherslide_core::{modelo::{Comentario, Tipo}, sessao::Sessao};
use featherslide_io::fsdz;

fn conferir(qtd: usize) -> Result<Vec<u8>, String> {
    let mut sessao = Sessao::default();
    sessao.apresentacao_mut().titulo = "Apresentação ⚡ Unicode – revisão de 2026".into();
    sessao.apresentacao_mut().autor = "Engenharia: São José 🇧🇷".into();
    for numero in 0..qtd {
        sessao.slide_novo("em_branco")?;
        for k in 0..5 {
            let x = 20.0 + k as f32 * 80.0;
            sessao.elemento_inserir(Tipo::Retangulo, Some([x, 45.0, 70.0, 42.0]), None)?;
        }
        sessao.elemento_inserir(Tipo::Texto, Some([30.0, 140.0, 680.0, 80.0]), None)?;
        let id = sessao.slide().elementos.last()
            .ok_or("não foi possível inserir texto")?.id;
        sessao.texto_definir(id, &format!("Seção {numero}: ação, café, 日本語, ⚡"))?;
    }
    let original = sessao.apresentacao_mut();
    original.metadados.assunto = "Teste Unicode e preservação de metadados".into();
    original.metadados.propriedades_personalizadas
        .insert("Unidade".into(), "Vitória/ES 🇧🇷".into());
    original.metadados.estendidos
        .insert("Fonte".into(), "UTF-8: ação + café".into());
    for (i, slide) in original.slides.iter_mut().enumerate() {
        slide.notas = format!("Nota {i}: cálculo ΔP = 18,25 MW. Revisão A. ");
        slide.notas.push_str(&"medição ⚡; ".repeat(20));
        if i % 9 == 0 {
            slide.secao = format!("Etapa {i}: manutenção e energia");
            slide.comentarios.push(Comentario {
                autor: "Verificadora".into(),
                texto: format!("Confirmar o texto do slide {i}: português e Ελληνικά"),
                resolvido: i % 2 == 0,
            });
        }
    }
    let antes = original.clone();
    let bytes = fsdz::gravar(&antes)?;
    if !bytes.starts_with(b"PK\x03\x04") {
        return Err("FSdz não possui assinatura ZIP".into());
    }
    let mut arquivo = zip::ZipArchive::new(std::io::Cursor::new(bytes.as_slice()))
        .map_err(|e| e.to_string())?;
    for nome in ["data/apresentacao.arrow", "data/slides.arrow", "data/elementos.arrow"] {
        let mut entrada = arquivo.by_name(nome).map_err(|e| e.to_string())?;
        let mut marca = [0u8; 6];
        std::io::Read::read_exact(&mut entrada, &mut marca).map_err(|e| e.to_string())?;
        if &marca != b"ARROW1" {
            return Err(format!("{nome} não é IPC Arrow"));
        }
    }
    let segunda = fsdz::ler(&bytes)?;
    if segunda != antes {
        return Err("slides, comentários ou metadados alterados na primeira leitura".into());
    }
    let bytes2 = fsdz::gravar(&segunda)?;
    if fsdz::ler(&bytes2)? != antes {
        return Err("slides, comentários ou metadados alterados após regravar".into());
    }
    if fsdz::ler(&bytes[..bytes.len() / 2]).is_ok() {
        return Err("FSdz truncado foi aceito".into());
    }
    Ok(bytes2)
}

fn main() {
    
    let destino = std::env::args().nth(1)
        .unwrap_or_else(|| "/tmp/io013_grande.fsdz".into());
    let resultado = (|| -> Result<usize, String> {
        let bytes = conferir(96)?;
        std::fs::write(&destino, &bytes).map_err(|e| e.to_string())?;
        let disco = std::fs::read(&destino).map_err(|e| e.to_string())?;
        if disco != bytes {
            return Err("bytes em disco diferem do buffer original".into());
        }
        fsdz::ler(&disco)?;
        Ok(bytes.len())
    })();
    match resultado {
        Ok(bytes) => println!("IO-013 FSdz OK: {bytes} bytes em {destino} (registros validados; Unicode e metadados)"),
        Err(e) => {
            eprintln!("IO-013 FSdz FALHOU: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn io013_fsdz_varios_slides_unicode_metadados_e_corrupcao() {
        assert!(conferir(12).unwrap().len() > 512);
    }
}
