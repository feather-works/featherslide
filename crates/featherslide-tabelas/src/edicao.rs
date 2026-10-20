//! Edição FeatherSlide sobre Tables com histórico e serialização nativos.
use featherslide_core::modelo::{Elemento, Tipo};
use featherslide_core::sessao::{Mudanca, Sessao};
use feather_tables::{Navegacao, PosicaoCelula, SelecaoTabela, Valor};
use crate::{comum_para_slide, slide_para_comum};

fn elemento(sessao: &Sessao, id: u64) -> Result<Elemento, String> {
    sessao.slide().elementos.iter()
        .find(|e| e.id == id && e.tipo == Tipo::Tabela)
        .cloned()
        .ok_or_else(|| "Essa tabela não está no slide.".to_owned())
}

/// O modelo compartilhado valida e altera a célula; a sessão continua dona
/// do único passo Ctrl+Z/Redo e o formato .fsdz permanece idêntico.
pub fn slide_editar_celula(
    sessao: &mut Sessao, id: u64, linha: usize, coluna: usize, texto: &str,
) -> Result<Mudanca, String> {
    let origem = elemento(sessao, id)?;
    let mut tabela = slide_para_comum(&origem)
        .map_err(|e| format!("Tabela nativa inválida: {e:?}"))?;
    tabela.atualizar_valor(linha, coluna, Valor::Texto(texto.to_owned()))
        .map_err(|e| format!("Não é possível editar essa célula: {e:?}"))?;
    let destino = comum_para_slide(&tabela, &origem)
        .map_err(|e| format!("Não foi possível converter a tabela: {e:?}"))?;
    sessao.tabela_dados_definir(id, &destino.dados)
}

/// Mantém exatamente os limites do Ribbon existente (30 x 15), alterando
/// a estrutura no motor compartilhado e salvando com o undo do Slide.
pub fn slide_editar_estrutura(
    sessao: &mut Sessao, id: u64, acao: &str,
) -> Result<Mudanca, String> {
    let origem = elemento(sessao, id)?;
    let mut tabela = slide_para_comum(&origem)
        .map_err(|e| format!("Tabela nativa inválida: {e:?}"))?;
    let (linhas, colunas) = tabela.dimensoes();
    match acao {
        "linha+" if linhas < 30 => tabela.inserir_linhas(linhas, 1),
        "linha-" if linhas > 1 => tabela.excluir_linhas(linhas - 1, 1),
        "coluna+" if colunas < 15 => tabela.inserir_colunas(colunas, 1),
        "coluna-" if colunas > 1 => tabela.excluir_colunas(colunas - 1, 1),
        _ => return Err("Não dá para fazer isso nessa tabela.".to_owned()),
    }.map_err(|e| format!("Falha ao alterar linhas/colunas: {e:?}"))?;
    let destino = comum_para_slide(&tabela, &origem)
        .map_err(|e| format!("Não foi possível converter a tabela: {e:?}"))?;
    sessao.tabela_dados_definir(id, &destino.dados)
}

/// Movimento conforme TBL-007; GUI continua responsável pelo foco/cursor.
pub fn slide_navegar_celula(
    sessao: &Sessao, id: u64, linha: usize, coluna: usize, movimento: Navegacao,
) -> Result<(usize, usize), String> {
    let origem = elemento(sessao, id)?;
    let tabela = slide_para_comum(&origem)
        .map_err(|e| format!("Tabela nativa inválida: {e:?}"))?;
    let mut selecao = SelecaoTabela::nova(&tabela, PosicaoCelula::nova(linha, coluna))
        .map_err(|e| format!("Célula fora da tabela: {e:?}"))?;
    selecao.mover(&tabela, movimento, false)
        .map_err(|e| format!("Navegação inválida: {e:?}"))?;
    Ok((selecao.ativo.linha, selecao.ativo.coluna))
}
