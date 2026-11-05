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
        // Pedidos seguidos do agente (sem edição da pessoa no meio, até 30 s um do outro) viram
        // um passo só do desfazer: um Ctrl+Z volta ao que era antes do agente começar.
        let versao = self.sessao.estado().versao;
        let marca = match self.mcp_grupo {
            Some((marca, v, quando)) if v == versao && quando.elapsed() < GRUPO_DO_AGENTE => marca,
            _ => self.sessao.marca_desfazer(),
        };
        let antes = self.sessao.apresentacao().clone();
        let mut agente = Agente { sessao: std::mem::take(&mut self.sessao), arquivo: self.caminho_arquivo.clone() };
        let resposta = rpc::atender(&mut agente, featherslide_mcp::fonte::da_sessao(), &pedido.0 .0);
        self.sessao = agente.sessao;
        if self.sessao.estado().versao != versao {
            self.sessao.juntar_desfazer_desde(marca);
            self.mcp_grupo = Some((marca, self.sessao.estado().versao, std::time::Instant::now()));
            self.mostrar_mudancas_do_agente(&antes);
        }
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

/// Até quanto tempo entre dois pedidos do agente eles ainda contam como o mesmo trabalho.
const GRUPO_DO_AGENTE: Duration = Duration::from_secs(30);

impl FeatherSlideApp {
    /// Seleciona no slide da tela o que o agente criou ou mudou e diz na barra de status quantos
    /// slides ele tocou, para a pessoa ver o que aconteceu antes de seguir.
    fn mostrar_mudancas_do_agente(&mut self, antes: &featherslide_core::modelo::Apresentacao) {
        let depois = self.sessao.apresentacao();
        let tocados = depois.slides.iter().filter(|s| antes.slides.iter().find(|a| a.id == s.id) != Some(*s)).count()
            + antes.slides.iter().filter(|a| !depois.slides.iter().any(|s| s.id == a.id)).count();
        let anterior = antes.slides.iter().find(|a| a.id == self.sessao.slide().id);
        let mudados: Vec<u64> = self.sessao.slide().elementos.iter()
            .filter(|e| anterior.and_then(|a| a.elementos.iter().find(|x| x.id == e.id)) != Some(*e))
            .map(|e| e.id)
            .collect();
        if !mudados.is_empty() {
            self.sessao.desmarcar();
            for id in &mudados {
                self.sessao.alternar_selecao(*id);
            }
        }
        self.status = format!("IA mudou {tocados} slide(s); {} elemento(s) marcados neste slide. Ctrl+Z desfaz o trabalho da IA de uma vez.", mudados.len());
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

    fn chamar(app: &mut FeatherSlideApp, id: u64, nome: &str, args: Value) -> Value {
        let (tx, rx) = mpsc::channel();
        let msg = json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": nome, "arguments": args } });
        app.atender_mcp(PedidoMcp(Arc::new((msg, Mutex::new(Some(tx))))));
        rx.recv().unwrap()
    }

    #[test]
    fn varios_pedidos_do_agente_saem_com_um_ctrl_z() {
        let mut app = FeatherSlideApp::default();
        let original = app.sessao.apresentacao().slides.len();
        for i in 0..3 {
            let r = chamar(&mut app, i, "slide_inserir", json!({}));
            assert_ne!(r["result"]["isError"], true, "{r}");
        }
        assert_eq!(app.sessao.apresentacao().slides.len(), original + 3);
        assert!(app.status.contains("IA mudou"), "{}", app.status);
        app.sessao.desfazer().unwrap();
        assert_eq!(app.sessao.apresentacao().slides.len(), original, "um Ctrl+Z desfaz os três pedidos");
    }

    #[test]
    fn edicao_da_pessoa_no_meio_separa_os_trabalhos_do_agente() {
        let mut app = FeatherSlideApp::default();
        let original = app.sessao.apresentacao().slides.len();
        chamar(&mut app, 1, "slide_inserir", json!({}));
        app.sessao.slide_novo("em_branco").unwrap();
        chamar(&mut app, 2, "slide_inserir", json!({}));
        app.sessao.desfazer().unwrap();
        assert_eq!(app.sessao.apresentacao().slides.len(), original + 2, "só o último pedido sai");
    }
}
