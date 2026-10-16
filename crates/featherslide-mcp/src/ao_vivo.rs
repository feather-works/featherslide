//! O MCP ao vivo do FeatherSlide: canal local (Unix socket / TCP local com token).
//! Adaptado de `feathercad/crates/feathercad-mcp/src/ao_vivo.rs`.
//! Quando o aplicativo FeatherSlide está aberto, cada `tools/call` vai para ele e age
//! sobre a apresentação na janela. Sem ele (ou com `FEATHERSLIDE_VIVO=0`), o servidor trabalha isolado.

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::time::Duration;

use serde_json::{json, Value};

/// O método da mensagem para pedir foco e entregar arquivos à instância única aberta.
pub const METODO_INSTANCIA: &str = "featherslide/instancia";

const ESPERA_DO_FOCO: Duration = Duration::from_secs(2);

fn trocar(escrita: &mut impl Write, leitura: &mut impl BufRead, linha: &str) -> std::io::Result<String> {
    writeln!(escrita, "{linha}")?;
    escrita.flush()?;
    let mut resposta = String::new();
    match leitura.read_line(&mut resposta)? {
        0 => Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "o aplicativo fechou a conexão")),
        _ => Ok(resposta.trim_end().to_string()),
    }
}

fn mensagem_de_foco(arquivos: &[String]) -> String {
    json!({ "jsonrpc": "2.0", "id": 1, "method": METODO_INSTANCIA, "params": { "arquivos": arquivos } }).to_string()
}

fn atender_conexao(leitura: impl BufRead, mut escrita: impl Write, tratar: &(impl Fn(Value) -> Option<Value> + ?Sized)) {
    for linha in leitura.lines() {
        let Ok(linha) = linha else { break };
        let Ok(msg) = serde_json::from_str::<Value>(&linha) else { continue };
        if let Some(r) = tratar(msg) {
            if writeln!(escrita, "{r}").and_then(|()| escrita.flush()).is_err() {
                break;
            }
        }
    }
}

// ------------------------------------------------------------------ transporte TCP (Windows / testes)
#[cfg_attr(unix, allow(dead_code))]
pub mod tcp {
    use std::io::{BufRead, BufReader, Write};
    use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
    use std::path::PathBuf;
    use std::sync::Arc;

    use serde_json::Value;

    use super::{atender_conexao, mensagem_de_foco, trocar, ESPERA_DO_FOCO};

    pub fn arquivo_da_porta() -> PathBuf {
        let base = std::env::var_os("FEATHERSLIDE_PORTA_DIR")
            .or_else(|| std::env::var_os("LOCALAPPDATA"))
            .or_else(|| std::env::var_os("XDG_RUNTIME_DIR"))
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        base.join("featherslide").join("mcp.porta")
    }

    fn token_novo() -> String {
        use std::hash::{BuildHasher, Hasher};
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u128(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos()));
        h.write_u32(std::process::id());
        format!("{:016x}", h.finish())
    }

    fn ler_porta() -> Option<(u16, String)> {
        let t = std::fs::read_to_string(arquivo_da_porta()).ok()?;
        let (porta, token) = t.trim().split_once(' ')?;
        Some((porta.parse().ok()?, token.to_string()))
    }

    fn conectar() -> Option<(TcpStream, BufReader<TcpStream>)> {
        let (porta, token) = ler_porta()?;
        let s = TcpStream::connect_timeout(&SocketAddr::from((Ipv4Addr::LOCALHOST, porta)), ESPERA_DO_FOCO).ok()?;
        let leitura = BufReader::new(s.try_clone().ok()?);
        let mut escrita = s.try_clone().ok()?;
        writeln!(escrita, "{token}").ok()?;
        Some((s, leitura))
    }

    #[derive(Default)]
    pub struct Cliente {
        conexao: Option<(TcpStream, BufReader<TcpStream>)>,
    }

    impl Cliente {
        pub fn repassar(&mut self, linha: &str) -> Option<Result<String, String>> {
            if self.conexao.is_none() {
                if std::env::var("FEATHERSLIDE_VIVO").is_ok_and(|v| v == "0") {
                    return None;
                }
                self.conexao = conectar();
            }
            let (escrita, leitura) = self.conexao.as_mut()?;
            let mut escrita = escrita.try_clone().ok()?;
            match trocar(&mut escrita, leitura, linha) {
                Ok(r) => Some(Ok(r)),
                Err(e) => {
                    self.conexao = None;
                    Some(Err(format!("a conversa com o aplicativo FeatherSlide caiu ({e}); confira a tela antes de repetir")))
                }
            }
        }
    }

    pub fn pedir_foco(arquivos: &[String]) -> bool {
        let Some((s, mut leitura)) = conectar() else { return false };
        let _ = s.set_read_timeout(Some(ESPERA_DO_FOCO));
        let Ok(mut escrita) = s.try_clone() else { return false };
        trocar(&mut escrita, &mut leitura, &mensagem_de_foco(arquivos)).is_ok_and(|r| serde_json::from_str::<Value>(&r).is_ok_and(|v| v["result"]["ok"] == true))
    }

    pub fn servir(tratar: impl Fn(Value) -> Option<Value> + Send + Sync + 'static) -> std::io::Result<bool> {
        if conectar().is_some() {
            return Ok(false);
        }
        let ouvinte = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let porta = ouvinte.local_addr()?.port();
        let token = token_novo();
        let arquivo = arquivo_da_porta();
        if let Some(pasta) = arquivo.parent() {
            std::fs::create_dir_all(pasta)?;
        }
        std::fs::write(&arquivo, format!("{porta} {token}"))?;
        let tratar = Arc::new(tratar);
        std::thread::spawn(move || {
            for conexao in ouvinte.incoming().flatten() {
                let (tratar, token) = (tratar.clone(), token.clone());
                std::thread::spawn(move || {
                    let Ok(leitura) = conexao.try_clone() else { return };
                    let mut leitura = BufReader::new(leitura);
                    let mut primeira = String::new();
                    if leitura.read_line(&mut primeira).is_err() || primeira.trim() != token {
                        return;
                    }
                    atender_conexao(leitura, conexao, &*tratar);
                });
            }
        });
        Ok(true)
    }

    pub fn encerrar() {
        let _ = std::fs::remove_file(arquivo_da_porta());
    }
}

// ------------------------------------------------------------------ transporte Unix
#[cfg(unix)]
pub mod unix {
    use std::io::BufReader;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::PathBuf;
    use std::sync::Arc;

    use serde_json::Value;

    use super::{atender_conexao, mensagem_de_foco, trocar, ESPERA_DO_FOCO};

    pub fn caminho_do_socket() -> PathBuf {
        let base = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(|| {
            let usuario = std::env::var("USER").unwrap_or_else(|_| "usuario".into());
            std::env::temp_dir().join(format!("featherslide-{usuario}"))
        });
        base.join("featherslide").join("mcp.sock")
    }

    #[derive(Default)]
    pub struct Cliente {
        conexao: Option<(UnixStream, BufReader<UnixStream>)>,
    }

    impl Cliente {
        fn conectar(&mut self) -> bool {
            if self.conexao.is_some() {
                return true;
            }
            if std::env::var("FEATHERSLIDE_VIVO").is_ok_and(|v| v == "0") {
                return false;
            }
            let Ok(s) = UnixStream::connect(caminho_do_socket()) else { return false };
            let Ok(leitura) = s.try_clone() else { return false };
            self.conexao = Some((s, BufReader::new(leitura)));
            true
        }

        pub fn repassar(&mut self, linha: &str) -> Option<Result<String, String>> {
            if !self.conectar() {
                return None;
            }
            let (escrita, leitura) = self.conexao.as_mut()?;
            match trocar(escrita, leitura, linha) {
                Ok(r) => Some(Ok(r)),
                Err(e) => {
                    self.conexao = None;
                    Some(Err(format!("a conversa com o aplicativo FeatherSlide caiu ({e}); confira a tela antes de repetir")))
                }
            }
        }
    }

    pub fn pedir_foco(arquivos: &[String]) -> bool {
        let Ok(s) = UnixStream::connect(caminho_do_socket()) else { return false };
        let _ = s.set_read_timeout(Some(ESPERA_DO_FOCO));
        let (Ok(leitura), Ok(mut escrita)) = (s.try_clone(), s.try_clone()) else { return false };
        trocar(&mut escrita, &mut BufReader::new(leitura), &mensagem_de_foco(arquivos)).is_ok_and(|r| serde_json::from_str::<Value>(&r).is_ok_and(|v| v["result"]["ok"] == true))
    }

    pub fn servir(tratar: impl Fn(Value) -> Option<Value> + Send + Sync + 'static) -> std::io::Result<bool> {
        let caminho = caminho_do_socket();
        if UnixStream::connect(&caminho).is_ok() {
            return Ok(false);
        }
        if let Some(pasta) = caminho.parent() {
            std::fs::create_dir_all(pasta)?;
            std::fs::set_permissions(pasta, std::fs::Permissions::from_mode(0o700))?;
        }
        let _ = std::fs::remove_file(&caminho);
        let ouvinte = UnixListener::bind(&caminho)?;
        std::fs::set_permissions(&caminho, std::fs::Permissions::from_mode(0o600))?;
        let tratar = Arc::new(tratar);
        std::thread::spawn(move || {
            for conexao in ouvinte.incoming().flatten() {
                let tratar = tratar.clone();
                std::thread::spawn(move || {
                    let Ok(leitura) = conexao.try_clone() else { return };
                    atender_conexao(BufReader::new(leitura), conexao, &*tratar);
                });
            }
        });
        Ok(true)
    }

    pub fn encerrar() {
        let _ = std::fs::remove_file(caminho_do_socket());
    }
}

#[cfg(unix)]
pub use unix::{caminho_do_socket, encerrar, pedir_foco, servir, Cliente};
#[cfg(not(unix))]
pub use tcp::{encerrar, pedir_foco, servir, Cliente};

#[cfg(unix)]
pub fn onde_escuta() -> PathBuf {
    caminho_do_socket()
}
#[cfg(not(unix))]
pub fn onde_escuta() -> PathBuf {
    tcp::arquivo_da_porta()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn socket_unix_conversa_e_unicidade() {
        let pasta = std::env::temp_dir().join(format!("fslide_vivo_{}", std::process::id()));
        std::env::set_var("XDG_RUNTIME_DIR", &pasta);
        assert!(servir(|m| Some(json!({ "eco": m["id"], "result": { "ok": m["method"] == METODO_INSTANCIA } }))).unwrap());
        assert!(!servir(|_| None).unwrap(), "segunda instância não toma lugar da primeira");
        let mut c = Cliente::default();
        let r = c.repassar(r#"{"id":42}"#).unwrap().unwrap();
        assert_eq!(serde_json::from_str::<Value>(&r).unwrap()["eco"], 42);
        assert!(pedir_foco(&["teste.fsdz".into()]));
        encerrar();
        assert!(Cliente::default().repassar("{}").is_none());
        let _ = std::fs::remove_dir_all(&pasta);
    }

    #[test]
    fn transporte_tcp_valida_token() {
        let pasta = std::env::temp_dir().join(format!("fslide_tcp_{}", std::process::id()));
        std::env::set_var("FEATHERSLIDE_PORTA_DIR", &pasta);
        assert!(tcp::servir(|m| Some(json!({ "eco": m["id"], "result": { "ok": true } }))).unwrap());
        assert!(!tcp::servir(|_| None).unwrap());
        let mut c = tcp::Cliente::default();
        let r = c.repassar(r#"{"id":99}"#).unwrap().unwrap();
        assert_eq!(serde_json::from_str::<Value>(&r).unwrap()["eco"], 99);
        assert!(tcp::pedir_foco(&[]));
        tcp::encerrar();
        let _ = std::fs::remove_dir_all(&pasta);
    }
}
