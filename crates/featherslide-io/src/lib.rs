//! Ler e gravar os arquivos do FeatherSlide: o `.fsdz` (o formato próprio) e a leitura de
//! `.pptx` (PowerPoint e Google Apresentações) e `.odp` (LibreOffice Impress). Exportar
//! `.pptx` e PDF são os próximos (ver `CLAUDE.md`).

pub mod base64;
pub mod arquivo;
pub mod documentos;
pub mod drawing;
/// Pipeline Image compartilhado pela GUI e exportação PDF.
pub mod imagem_compartilhada;
pub mod fsdz;
/// Conversão opcional de gráficos do .fsdz para o motor comum Charts.
pub mod grafico_compartilhado;
mod metadados_externos;
pub mod odp;
pub mod pdf;
pub mod svg;
pub mod pptx;

mod pacote;
mod xml;
