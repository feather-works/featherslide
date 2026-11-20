//! Substituição de arquivos por um temporário exclusivo no mesmo diretório.
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQUENCIA: AtomicU64 = AtomicU64::new(0);

/// Grava e sincroniza os dados antes de substituir o destino por renomeação.
/// Em caso de erro antes da renomeação, conserva o destino e remove o temporário.
/// Não garante persistência dos metadados do diretório após queda de energia.
pub fn gravar_atomico(destino: &Path, bytes: &[u8]) -> Result<(), String> {
    // Um atalho (symlink) é seguido: o renomear trocava o atalho por um arquivo e o original ficava velho.
    let real = fs::canonicalize(destino).ok();
    let destino = real.as_deref().unwrap_or(destino);
    let pai = destino.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let nome = destino.file_name().ok_or("O destino precisa ter um nome de arquivo.")?;
    let (temporario, mut arquivo) = loop {
        let mut nome_temp = std::ffi::OsString::from(".");
        nome_temp.push(nome);
        nome_temp.push(format!(".{}-{}.tmp", std::process::id(), SEQUENCIA.fetch_add(1, Ordering::Relaxed)));
        let caminho = pai.join(nome_temp);
        match OpenOptions::new().write(true).create_new(true).open(&caminho) {
            Ok(arquivo) => break (caminho, arquivo),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("Não foi possível criar o arquivo temporário: {e}")),
        }
    };
    // As permissões do arquivo que já existe ficam (0600 não vira 0644).
    let permissoes = fs::metadata(destino).map(|m| m.permissions());
    let escrita = permissoes.map_or(Ok(()), |p| arquivo.set_permissions(p))
        .and_then(|()| arquivo.write_all(bytes)).and_then(|()| arquivo.sync_all());
    drop(arquivo);
    let resultado = escrita.and_then(|()| fs::rename(&temporario, destino));
    if resultado.is_err() {
        let _ = fs::remove_file(&temporario);
    }
    resultado.map_err(|e| format!("Não foi possível salvar {}: {e}", destino.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn gravar_por_um_atalho_grava_no_original_e_mantem_permissoes() {
        use std::os::unix::fs::PermissionsExt;
        let pasta = std::env::temp_dir().join(format!("fsl-atalho-{}", std::process::id()));
        fs::create_dir_all(&pasta).unwrap();
        let (real, atalho) = (pasta.join("real.pptx"), pasta.join("atalho.pptx"));
        fs::write(&real, b"velho").unwrap();
        fs::set_permissions(&real, fs::Permissions::from_mode(0o600)).unwrap();
        std::os::unix::fs::symlink(&real, &atalho).unwrap();
        gravar_atomico(&atalho, b"novo").unwrap();
        let conteudo = fs::read(&real).unwrap();
        let link = fs::symlink_metadata(&atalho).unwrap().file_type().is_symlink();
        let modo = fs::metadata(&real).unwrap().permissions().mode() & 0o777;
        let _ = fs::remove_dir_all(&pasta);
        assert_eq!((conteudo.as_slice(), link, modo), (&b"novo"[..], true, 0o600));
    }

    #[test]
    fn substitui_arquivo_e_limpa_temporarios_em_falha() {
        let pasta = std::env::temp_dir().join(format!("ftslide-atomico-{}-{}", std::process::id(), SEQUENCIA.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&pasta).unwrap();
        let destino = pasta.join("apresentacao.fsdz");
        fs::write(&destino, b"anterior").unwrap();
        gravar_atomico(&destino, b"novo").unwrap();
        assert_eq!(fs::read(&destino).unwrap(), b"novo");
        let bloqueado = pasta.join("diretorio.fsdz");
        fs::create_dir(&bloqueado).unwrap();
        fs::write(bloqueado.join("original"), b"preservado").unwrap();
        assert!(gravar_atomico(&bloqueado, b"invalido").is_err());
        assert_eq!(fs::read(bloqueado.join("original")).unwrap(), b"preservado");
        assert_eq!(fs::read_dir(&pasta).unwrap().count(), 2);
        fs::remove_dir_all(pasta).unwrap();
    }
}
