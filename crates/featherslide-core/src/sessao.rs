//! A apresentação aberta: o slide atual, a seleção, o desfazer e a versão de cada slide
//! (a interface só pede de novo o quadro do slide cuja versão mudou). Os comandos estão
//! em `slides.rs` (incluir, mover, excluir slides) e `elementos.rs` (inserir, mover,
//! formatar elementos); o desfazer, em `edicao.rs`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::layouts;
use crate::modelo::{Apresentacao, Elemento, Proporcao, Slide};
use crate::quadro::{self, Dicas, Quadro};

/// O que mudou depois de um comando: a versão nova da apresentação e os slides tocados
/// (índices na ordem atual). A interface repinta só esses.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Mudanca {
    pub versao: u64,
    pub slides: Vec<usize>,
    /// A lista de slides mudou (incluir, excluir, mover): as miniaturas se refazem.
    pub lista: bool,
}

/// Um slide na lista do painel da esquerda: o bastante para saber se a miniatura vale.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ItemDaLista {
    pub id: u64,
    pub versao: u64,
    pub oculto: bool,
}

/// O estilo do elemento selecionado (o primeiro), para a faixa mostrar marcado.
#[derive(Clone, Debug, Serialize, PartialEq, Default)]
pub struct Formato {
    pub tipo: String,
    pub fonte: String,
    pub tamanho: f32,
    pub cor: String,
    pub negrito: bool,
    pub italico: bool,
    pub sublinhado: bool,
    pub tachado: bool,
    pub sublinhado_duplo: bool,
    pub tachado_duplo: bool,
    pub entrelinha: f32,
    pub colunas: u8,
    pub alinhamento: String,
    pub vertical: String,
    pub preenchimento: String,
    pub contorno: String,
    pub espessura: f32,
}

/// O estado que a interface lê depois de cada ação.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Estado {
    pub versao: u64,
    pub titulo: String,
    pub largura: f32,
    pub altura: f32,
    pub proporcao: Proporcao,
    pub atual: usize,
    pub slides: Vec<ItemDaLista>,
    pub selecao: Vec<u64>,
    /// O retângulo de cada elemento selecionado (x, y, largura, altura), para as alças.
    pub caixas: Vec<[f32; 4]>,
    pub formato: Option<Formato>,
    pub notas: String,
    pub layout: String,
    pub fundo: String,
    pub sujo: bool,
    pub pode_desfazer: bool,
    pub pode_refazer: bool,
    pub elementos: usize,
}

/// Um passo do desfazer: a apresentação inteira e onde se estava.
#[derive(Clone)]
pub(crate) struct Retrato {
    pub ap: Apresentacao,
    pub atual: usize,
    pub selecao: Vec<u64>,
}

/// A apresentação aberta e tudo o que a edição precisa.
#[derive(Clone)]
pub struct Sessao {
    pub(crate) ap: Apresentacao,
    pub(crate) atual: usize,
    pub(crate) selecao: Vec<u64>,
    pub(crate) desfazer: Vec<Retrato>,
    /// O estado de antes de uma prévia em andamento (o seletor de cor aberto): vira UM passo do
    /// desfazer quando a prévia é confirmada.
    pub(crate) previa: Option<Retrato>,
    pub(crate) previa_sujo: bool,
    pub(crate) refazer: Vec<Retrato>,
    pub(crate) versao: u64,
    /// A versão de cada slide, pelo `id` (não vai para o arquivo).
    pub(crate) versoes: HashMap<u64, u64>,
    pub(crate) sujo: bool,
    pub(crate) dicas: Dicas,
    /// O que Copiar/Recortar guardou (os elementos inteiros, para colar com estilo).
    pub(crate) transferencia: Vec<Elemento>,
    pub(crate) busca: Option<crate::busca::EstadoDaBusca>,
    /// Instante do último comando de edição (não serializado), para estimar tempo ativo.
    pub(crate) ultima_edicao: Option<std::time::Instant>,
}

impl Default for Sessao {
    fn default() -> Self {
        Self::nova(Apresentacao { titulo: String::new(), autor: String::new(), proporcao: Proporcao::Larga, dimensoes: [0.0; 2], slides: Vec::new(), proximo_id: 1, fontes: Vec::new(), tema: String::new(), modelo: String::new(), metadados: crate::metadados::Metadados::novo() })
    }
}

impl Sessao {
    /// Uma sessão com a apresentação dada. Sem slide nenhum, ganha um slide de título.
    pub fn nova(mut ap: Apresentacao) -> Self {
        // Arquivos nativos de versões anteriores podem não possuir identificador.
        // Gerar uma vez ao abrir e preservar o valor depois que o documento for salvo.
        if ap.metadados.identificador.is_empty() {
            ap.metadados.identificador = crate::metadados::novo_identificador();
        }
        if ap.slides.is_empty() {
            let slide = slide_com_layout(&mut ap, "titulo");
            ap.slides.push(slide);
        }
        Self { ap, atual: 0, selecao: Vec::new(), desfazer: Vec::new(), previa: None, previa_sujo: false, refazer: Vec::new(), versao: 1, versoes: HashMap::new(), sujo: false, dicas: Dicas::default(), transferencia: Vec::new(), busca: None, ultima_edicao: None }
    }

    /// Uma cópia recuperada ainda precisa de um destino escolhido pelo usuário.
    pub fn marcar_alterado(&mut self) { self.sujo = true; }

    /// A apresentação (para gravar).
    pub fn apresentacao(&self) -> &Apresentacao {
        &self.ap
    }

    /// A apresentação mutável.
    pub fn apresentacao_mut(&mut self) -> &mut Apresentacao {
        &mut self.ap
    }

    /// O índice do slide atual.
    pub fn atual(&self) -> usize {
        self.atual
    }

    /// Define as dicas das caixas vazias (na língua da interface).
    pub fn definir_dicas(&mut self, dicas: Dicas) -> Mudanca {
        self.dicas = dicas;
        let todos: Vec<usize> = (0..self.ap.slides.len()).collect();
        for s in &self.ap.slides {
            *self.versoes.entry(s.id).or_insert(0) += 1;
        }
        self.versao += 1;
        Mudanca { versao: self.versao, slides: todos, lista: false }
    }

    /// Depois de gravar: não há mais mudança pendente.
    pub fn marcar_gravado(&mut self) {
        self.sujo = false;
    }

    /// Atualiza título, autoria e propriedades editáveis em uma transação desfazível.
    pub fn propriedades_definir(
        &mut self, titulo: String, autor: String,
        dados: crate::metadados::Metadados,
    ) -> Result<Mudanca, String> {
        dados.validar()?;
        if titulo.len() > 4096 || autor.len() > 1024 {
            return Err("Título ou autor excede o tamanho permitido.".into());
        }
        let mut editaveis = self.ap.metadados.clone();
        editaveis.aplicar_editaveis(&dados);
        if self.ap.titulo == titulo && self.ap.autor == autor && self.ap.metadados == editaveis {
            return Ok(Mudanca { versao: self.versao, slides: Vec::new(), lista: false });
        }
        self.editar(move |s| {
            s.ap.titulo = titulo;
            s.ap.autor = autor;
            s.ap.metadados.aplicar_editaveis(&dados);
            Ok((Vec::new(), false))
        })
    }

    /// As estatísticas são recalculadas dos slides e nunca ficam desatualizadas.
    pub fn estatisticas(&self) -> crate::metadados::Estatisticas {
        crate::metadados::estatisticas(&self.ap)
    }

    /// O slide atual.
    pub fn slide(&self) -> &Slide {
        &self.ap.slides[self.atual]
    }

    /// O slide atual mutável.
    pub fn slide_mut(&mut self) -> &mut Slide {
        &mut self.ap.slides[self.atual]
    }

    /// A versão do slide `id` (0 se nunca mudou).
    fn versao_do_slide(&self, id: u64) -> u64 {
        self.versoes.get(&id).copied().unwrap_or(0)
    }

    /// O quadro do slide `indice`, ou `None` se a interface já tem esta versão (`conhecida`).
    pub fn quadro(&self, indice: usize, conhecida: Option<u64>, editor: bool) -> Result<Option<Quadro>, String> {
        let slide = self.ap.slides.get(indice).ok_or_else(|| format!("O slide {} não existe.", indice + 1))?;
        let versao = self.versao_do_slide(slide.id) + 1;
        if conhecida == Some(versao) {
            return Ok(None);
        }
        Ok(Some(quadro::quadro(slide, self.ap.tamanho(), versao, &self.dicas, editor)))
    }

    /// O estado para a interface.
    pub fn estado(&self) -> Estado {
        let (largura, altura) = self.ap.tamanho();
        let slide = self.slide();
        let selecionados: Vec<&Elemento> = slide.elementos.iter().filter(|e| self.selecao.contains(&e.id)).collect();
        Estado {
            versao: self.versao,
            titulo: self.ap.titulo.clone(),
            largura,
            altura,
            proporcao: self.ap.proporcao,
            atual: self.atual,
            slides: self.ap.slides.iter().map(|s| ItemDaLista { id: s.id, versao: self.versao_do_slide(s.id) + 1, oculto: s.oculto }).collect(),
            selecao: self.selecao.clone(),
            caixas: selecionados.iter().map(|e| [e.x, e.y, e.largura, e.altura]).collect(),
            formato: selecionados.first().map(|e| formato_de(e)),
            notas: slide.notas.clone(),
            layout: slide.layout.clone(),
            fundo: if slide.fundo.is_empty() { "#ffffff".into() } else { slide.fundo.clone() },
            sujo: self.sujo,
            pode_desfazer: !self.desfazer.is_empty(),
            pode_refazer: !self.refazer.is_empty(),
            elementos: slide.elementos.len(),
        }
    }
}

/// O formato que a faixa mostra para o elemento `e`.
fn formato_de(e: &Elemento) -> Formato {
    let est = &e.estilo;
    let nome = |s: &dyn std::fmt::Debug| format!("{s:?}").to_lowercase();
    Formato {
        tipo: e.tipo.nome().into(),
        fonte: est.fonte.clone(),
        tamanho: est.tamanho,
        cor: est.cor.clone(),
        negrito: est.negrito,
        italico: est.italico,
        sublinhado: est.sublinhado,
        tachado: est.tachado,
        sublinhado_duplo: est.sublinhado && est.sublinhado_duplo,
        tachado_duplo: est.tachado && est.tachado_duplo,
        entrelinha: est.entrelinha,
        colunas: est.colunas,
        alinhamento: nome(&est.alinhamento),
        vertical: nome(&est.vertical),
        preenchimento: e.preenchimento.clone(),
        contorno: e.contorno.clone(),
        espessura: e.espessura,
    }
}

/// Um slide novo com as caixas do `layout` (o desconhecido vira em branco).
pub(crate) fn slide_com_layout(ap: &mut Apresentacao, layout: &str) -> Slide {
    let (largura, altura) = ap.tamanho();
    let mut proximo = ap.proximo_id;
    let elementos = layouts::elementos_do_layout(layout, largura, altura, &mut || {
        let id = proximo;
        proximo += 1;
        id
    });
    let (layout, elementos) = match elementos {
        Some(e) => (layout, e),
        None => ("em_branco", Vec::new()),
    };
    let id = proximo;
    ap.proximo_id = proximo + 1;
    let mut slide = Slide { id, layout: layout.into(), fundo: String::new(), fundo_config: Default::default(), ocultar_mestre: false, elementos, notas: String::new(), oculto: false, transicao: Default::default(), secao: String::new(), comentarios: Vec::new() };
    if !ap.tema.is_empty() {
        // Slide novo nasce no tema da apresentação (de "office" para o tema atual).
        crate::temas::aplicar_no_slide(&mut slide, crate::temas::tema("office"), crate::temas::tema(&ap.tema));
    }
    if !ap.modelo.is_empty() {
        let modelo = ap.modelo.clone();
        crate::modelos::decorar(ap, &mut slide, &modelo, layout);
    }
    slide
}
