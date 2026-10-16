//! O agente do MCP: mantém a sessão de apresentação aberta e o arquivo associado.
//! No modo ao vivo, o aplicativo troca a sessão da janela com a do agente durante a chamada da ferramenta.

use std::path::PathBuf;

use featherslide_core::sessao::Sessao;

#[derive(Default)]
pub struct Agente {
    pub sessao: Sessao,
    pub arquivo: Option<PathBuf>,
}
