//! MCP ao vivo: o `featherslide-mcp` que a IA abre repassa cada `tools/call` para cá pelo canal
//! local de `featherslide_mcp::ao_vivo` (socket do usuário, só nesta máquina). A thread do canal
//! entrega o pedido ao `update` por uma subscription e espera a resposta: as ferramentas rodam na
//! apresentação da tela, e cada edição é um Ctrl+Z. Mesmo desenho do `mcp_vivo.rs` do FeatherSheet.
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::time::Duration;

use featherslide_mcp::{agente::Agente, ao_vivo, rpc};
use pelicano::futures::{channel::mpsc as canal, StreamExt};
use pelicano::Subscription;
use serde_json::{json, Value};

use crate::app::{FeatherSlideApp, Mensagem};

/// Um pedido vindo do canal, com o caminho de volta da resposta.
#[derive(Clone)]
pub struct PedidoMcp(Arc<(Value, Mutex<Option<mpsc::Sender<Value>>>)>);

impl std::fmt::Debug for PedidoMcp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PedidoMcp({})", self.0 .0.get("method").and_then(Value::as_str).unwrap_or("?"))
    }
}

static RECEPTOR: OnceLock<Mutex<Option<canal::UnboundedReceiver<PedidoMcp>>>> = OnceLock::new();
static ENVIO: OnceLock<Mutex<canal::UnboundedSender<PedidoMcp>>> = OnceLock::new();

/// Quanto a thread do canal espera o aplicativo (abrir arquivo grande demora).
const ESPERA: Duration = Duration::from_secs(120);

fn recusa(id: Value, texto: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": { "isError": true, "content": [{ "type": "text", "text": texto }] } })
}

/// Abre o canal na abertura do app. Fora nos testes, na captura e com `FEATHERSLIDE_VIVO=0`;
/// um segundo FeatherSlide aberto não toma o canal do primeiro.
pub fn ligar() -> String {
    if std::env::var("FEATHERSLIDE_VIVO").is_ok_and(|v| v == "0")
        || std::env::var_os("FEATHERSLIDE_CAPTURAR").is_some()
        || cfg!(test)
    {
        return "MCP ao vivo desligado neste modo de execução".into();
    }
    let (tx, rx) = canal::unbounded::<PedidoMcp>();
    let _ = RECEPTOR.set(Mutex::new(Some(rx)));
    let _ = ENVIO.set(Mutex::new(tx));
    let tratar = |msg: Value| -> Option<Value> {
        let id = msg.get("id")?.clone();
        if msg.get("method").and_then(Value::as_str) == Some(ao_vivo::METODO_INSTANCIA) {
            return Some(json!({ "jsonrpc": "2.0", "id": id, "result": { "ok": false } }));
        }
        let (rtx, rrx) = mpsc::channel();
        ENVIO.get()?.lock().ok()?.unbounded_send(PedidoMcp(Arc::new((msg, Mutex::new(Some(rtx)))))).ok()?;
        Some(rrx.recv_timeout(ESPERA).unwrap_or_else(|_| {
            recusa(id, "O FeatherSlide não respondeu a tempo (ocupado ou com um diálogo aberto).")
        }))
    };
    match ao_vivo::servir(tratar) {
        Ok(true) => format!("MCP ao vivo escutando em {}", ao_vivo::onde_escuta().display()),
        Ok(false) => "MCP ao vivo: outro FeatherSlide já atende o canal".into(),
        Err(e) => format!("MCP ao vivo indisponível: {e}"),
    }
}

pub fn encerrar() {
    if RECEPTOR.get().is_some() {
        ao_vivo::encerrar();
    }
}

pub fn assinatura() -> Subscription<Mensagem> {
    fn fluxo() -> pelicano::futures::stream::BoxStream<'static, Mensagem> {
        match RECEPTOR.get().and_then(|m| m.lock().ok()?.take()) {
            Some(rx) => rx.map(Mensagem::Mcp).boxed(),
            None => pelicano::futures::stream::pending().boxed(),
        }
    }
    if RECEPTOR.get().is_some() { Subscription::run_with_id("featherslide.mcp", fluxo()) } else { Subscription::none() }
}

impl FeatherSlideApp {
    /// Roda o pedido na apresentação da tela: a sessão vai emprestada ao agente do MCP e volta.
    pub(crate) fn atender_mcp(&mut self, pedido: PedidoMcp) {
        let mut agente = Agente { sessao: std::mem::take(&mut self.sessao), arquivo: self.caminho_arquivo.clone() };
        let resposta = rpc::atender(&mut agente, featherslide_mcp::fonte::da_sessao(), &pedido.0 .0);
        self.sessao = agente.sessao;
        if agente.arquivo != self.caminho_arquivo {
            if let Some(p) = &agente.arquivo {
                self.nome_arquivo = p.file_name().unwrap_or_default().to_string_lossy().into();
            }
            self.caminho_arquivo = agente.arquivo;
        }
        if let (Some(r), Some(tx)) = (resposta, pedido.0 .1.lock().ok().and_then(|mut g| g.take())) {
            let _ = tx.send(r);
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn pedido_edita_a_apresentacao_da_tela() {
        let mut app = FeatherSlideApp::default();
        let antes = app.sessao.apresentacao().slides.len();
        let (tx, rx) = mpsc::channel();
        let msg = json!({ "jsonrpc": "2.0", "id": 7, "method": "tools/call",
            "params": { "name": "slide_inserir", "arguments": {} } });
        app.atender_mcp(PedidoMcp(Arc::new((msg, Mutex::new(Some(tx))))));
        let resposta = rx.recv().unwrap();
        assert_eq!(resposta["id"], 7);
        assert_ne!(resposta["result"]["isError"], true, "{resposta}");
        assert_eq!(app.sessao.apresentacao().slides.len(), antes + 1);
    }
}
