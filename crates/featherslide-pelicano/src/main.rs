//! Ponto de entrada da interface gráfica nativa em Rust do FeatherSlide (pelicano).

mod configuracao_slide;
mod ajuda;
mod revisao;
mod propriedades;
mod imprimir;
mod regua;
mod ortografia;
mod ferramentas_texto;
mod editor_diagrama;
mod localidade;
mod apresentacao;
mod cronometro;
mod previa;
mod app;
mod arquivos;
mod vetores;
mod configuracao;
mod configuracoes;
mod dados_janela;
mod monitor;
mod painel_animacao;
mod recentes;
mod simbolos;
mod monitores;
mod medido;
mod menu_fonte;
mod galeria_figuras;
mod seletor_cor;
mod modelos_usuario;
mod galeria_layouts;
mod reutilizar_slides;
mod personalizadas;
mod registro;
mod desenho;
mod drawing_iced;
mod imagem_processamento;
mod imagem_painel;
mod imagem_arquivo;
mod texto_artistico;
mod faixa;
mod faixa_abas;
mod menu_contexto;
mod miniaturas;
mod palco;
#[cfg(test)]
mod rastros;
pub mod icones;
pub mod tema;

use app::FeatherSlideApp;

pub fn main() -> Result<(), pelicano::Error> {
    registro::iniciar();
    registro::registrar("[abrir] FeatherSlide iniciado");
    // Pânico (bug do programa) também aparece no Console, além do terminal.
    let anterior = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        registro::registrar(format!("[erro] pânico: {info}"));
        anterior(info);
    }));
    // O pelicano roda as tarefas no tokio dele; este dá um contexto ao que o `update` chama direto.
    let rt = tokio::runtime::Runtime::new().expect("falha ao iniciar tokio runtime");
    let _guard = rt.enter();

    // Várias janelas (o Monitor, F12, abre uma à parte). A principal abre pelo `FeatherSlideApp::new`,
    // com a moldura que a configuração pede; quem fecha o app é ela (`Mensagem::FecharJanela`).
    pelicano::run(
        pelicano::Settings { start_hidden: true, ..Default::default() },
        FeatherSlideApp::new,
    )
}

mod formato_forma;
