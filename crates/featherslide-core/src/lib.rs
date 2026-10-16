//! O motor do FeatherSlide. A apresentação inteira mora aqui: os slides, os
//! elementos de cada um (caixas de texto e formas), a seleção, o desfazer e o
//! quadro pronto para desenhar. A interface só traduz mouse e teclado em pedido
//! e pinta o que volta (RECEITA.md §1).
//!
//! - [`modelo`]: os dados (apresentação, slide, elemento, estilo do texto);
//! - [`layouts`]: os layouts de slide (título, título e conteúdo…) e o que cada um cria;
//! - [`medida`]: a largura aproximada do texto, para quebrar as linhas;
//! - [`quadro`]: o que a interface desenha de um slide, já resolvido;
//! - [`sessao`]: a apresentação aberta, a seleção e cada comando de edição;
//! - [`formas_comuns`]: a ponte com o `feather-shapes` (formas, ícones e diagramas comuns da
//!   suíte): `Tipo` ↔ `TipoForma`, item de diagrama → `Elemento`, geometria → `Primitiva`.

pub mod configuracao_slide;
pub mod busca;
pub mod ferramentas_texto;
pub mod cursor;
pub mod dados;
pub mod diagramas;
pub mod editor_diagrama;
pub mod formas;
pub mod formas_comuns;
pub mod imagem;
pub mod novas_formas;
pub mod outras_galerias;
pub mod booleanas;
pub mod wordart;
pub mod escala;
pub mod icones;
pub mod layouts;
pub mod links;
pub mod medida;
pub mod modelo;
pub mod metadados;
pub mod personalizadas;
pub mod quadro;
pub mod svgpath;
pub mod vetores;
pub mod sessao;
pub mod temas;

mod edicao;
mod efeitos;
pub mod modelos;
mod elementos;
mod secoes;
mod slides;
mod operacoes_slides;

pub use operacoes_slides::{SlidesCopiados, EstiloColagem};
pub use busca::{OpcoesDeBusca, ResultadoDeBusca, ResultadoSubstituicao};
pub use efeitos::PassoDeAnimacao;
pub use secoes::SecaoInfo;
pub use elementos::{Formatacao, ItemDeElemento, Ordem};

pub mod estilo_forma;

pub mod texto_formato;
