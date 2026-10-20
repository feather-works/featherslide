//! Tabelas do FeatherSlide pelo motor comum `feather-tables` (antes a parte "slide" do
//! `featheroffice-table-adapters` do monorepo, trazida para cá na publicação).
//!
//! Não muda a serialização do .fsdz, nem faz os motores dos aplicativos
//! dependerem do crate compartilhado. Os atributos fora do subconjunto comum
//! permanecem no modelo original (parâmetro \`original\` da volta).
//! Ao tentar modificar conteúdo que não cabe no modelo nativo, retorna erro.

pub mod edicao;
pub use edicao::{slide_editar_celula, slide_editar_estrutura, slide_navegar_celula};

use featherslide_core::dados;
use featherslide_core::modelo::{Elemento, Tipo};
use feather_tables::{
    Celula, ErroTabela, PoliticaBordas, Tabela, Valor,
};

/// Limite de trabalho explícito para não materializar tabelas hostis.
const MAX_CELULAS: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroAdaptacao {
    TipoInvalido,
    GradeIrregular,
    DimensoesExcessivas,
    DadosInvalidos,
    FormatoIncompativel,
    MesclagensNaoSuportadas,
    TextoRicoAlterado,
}

impl From<ErroTabela> for ErroAdaptacao {
    fn from(_: ErroTabela) -> Self {
        Self::DadosInvalidos
    }
}

fn validar_dimensoes(linhas: usize, colunas: usize) -> Result<(), ErroAdaptacao> {
    if (linhas == 0 && colunas != 0)
        || linhas > MAX_CELULAS
        || colunas > MAX_CELULAS
        || linhas.checked_mul(colunas).map_or(true, |n| n > MAX_CELULAS)
    {
        return Err(ErroAdaptacao::DimensoesExcessivas);
    }
    Ok(())
}

/// Por ora os formatos nativos continuam donos da formatação. O retorno não
/// deve descartar estilos/margens/mesclagens editados no modelo compartilhado:
/// essas operações pertencem à futura migração TBL-010.
fn validar_sem_atributos_novos(tabela: &Tabela) -> Result<(), ErroAdaptacao> {
    let (linhas, colunas) = tabela.dimensoes();
    validar_dimensoes(linhas, colunas)?;
    if !tabela.mesclagens().is_empty() {
        return Err(ErroAdaptacao::MesclagensNaoSuportadas);
    }
    if tabela.iter().any(|(_, c)| c.estilo != Default::default())
        || tabela.iter_layout().next().is_some()
        || tabela.politica_bordas() != PoliticaBordas::Separadas
    {
        return Err(ErroAdaptacao::FormatoIncompativel);
    }
    if (0..linhas).any(|linha| tabela.altura(linha).is_some())
        || (0..colunas).any(|coluna| tabela.largura(coluna).is_some())
    {
        return Err(ErroAdaptacao::FormatoIncompativel);
    }
    Ok(())
}

fn texto_da_celula(tabela: &Tabela, linha: usize, coluna: usize) -> Result<String, ErroAdaptacao> {
    match tabela.obter(linha, coluna).map(|c| &c.valor) {
        None | Some(Valor::Vazio) => Ok(String::new()),
        Some(Valor::Texto(t)) => Ok(t.clone()),
        // Os modelos nativos são textuais: não converter número ou booleano
        // implicitamente, pois uma volta alteraria o tipo semântico do valor.
        Some(Valor::Numero(_)) | Some(Valor::Logico(_)) => {
            Err(ErroAdaptacao::FormatoIncompativel)
        }
    }
}

fn armazenar_texto(tabela: &mut Tabela, r: usize, c: usize, texto: String) -> Result<(), ErroAdaptacao> {
    if !texto.is_empty() {
        tabela.definir(r, c, Celula {
            valor: Valor::Texto(texto),
            ..Celula::default()
        })?;
    }
    Ok(())
}

/// Converte a grade nativa de um Elemento::Tabela sem reinterpretar strings
/// como números, fórmulas ou estilos por célula. Os atributos gráficos
/// permanecem no \`Elemento\` original usado por \`comum_para_slide\`.
pub fn slide_para_comum(elemento: &Elemento) -> Result<Tabela, ErroAdaptacao> {
    if elemento.tipo != Tipo::Tabela {
        return Err(ErroAdaptacao::TipoInvalido);
    }
    let grade = dados::ler_tabela(elemento);
    let linhas = grade.len();
    let colunas = grade.first().map_or(0, Vec::len);
    validar_dimensoes(linhas, colunas)?;
    if grade.iter().any(|linha| linha.len() != colunas)
        || dados::gravar_tabela(&grade) != elemento.dados
    {
        return Err(ErroAdaptacao::GradeIrregular);
    }
    let mut comum = Tabela::nova(linhas, colunas);
    for (r, linha) in grade.into_iter().enumerate() {
        for (c, texto) in linha.into_iter().enumerate() {
            armazenar_texto(&mut comum, r, c, texto)?;
        }
    }
    Ok(comum)
}

/// Volta ao Elemento do Slide preservando id, posição, cores, fonte, tamanho,
/// animação e demais campos. Somente \`dados\` é reconstruído. Não introduz
/// campos novos no FSDZ; rejeita estilos individuais não representáveis.
pub fn comum_para_slide(tabela: &Tabela, original: &Elemento) -> Result<Elemento, ErroAdaptacao> {
    if original.tipo != Tipo::Tabela {
        return Err(ErroAdaptacao::TipoInvalido);
    }
    // Também recusa entrada nativa corrompida na volta, sem a normalizar.
    slide_para_comum(original)?;
    validar_sem_atributos_novos(tabela)?;
    let (linhas, colunas) = tabela.dimensoes();
    if linhas == 0 || colunas == 0 {
        return Err(ErroAdaptacao::GradeIrregular);
    }
    let mut grade = Vec::with_capacity(linhas);
    for r in 0..linhas {
        let mut linha = Vec::with_capacity(colunas);
        for c in 0..colunas {
            linha.push(texto_da_celula(tabela, r, c)?);
        }
        grade.push(linha);
    }
    let mut convertido = original.clone();
    convertido.dados = dados::gravar_tabela(&grade);
    Ok(convertido)
}


