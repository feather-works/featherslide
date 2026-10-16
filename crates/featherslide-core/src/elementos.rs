//! Os comandos sobre os elementos do slide atual: inserir, selecionar (o clique cai no
//! elemento de cima), mover, redimensionar, escrever, formatar, ordenar, alinhar no
//! slide, copiar, recortar e colar.

use crate::modelo::{Alinhamento, Cor, Elemento, EstiloDoTexto, Papel, Sombra, Tipo, Vertical};
use crate::sessao::{Mudanca, Sessao};

/// O menor lado de um elemento, em pontos.
const MINIMO: f32 = 4.0;
/// Quanto a cópia colada anda em relação ao original.
const DESLOCAMENTO_DA_COPIA: f32 = 12.0;
/// A folga do clique numa linha (que não tem área), em pontos.
const FOLGA_DA_LINHA: f32 = 5.0;

/// Uma formatação pedida pela faixa; vale para os elementos selecionados.
#[derive(Clone, Debug, PartialEq)]
pub enum Formatacao {
    Fonte(String),
    Tamanho(f32),
    /// Aumentar (+1) ou diminuir (−1) a letra um passo.
    TamanhoPasso(i32),
    CorDoTexto(String),
    /// Cor de fundo atrás do texto (vazia = sem fundo).
    Realce(String),
    /// Cor do fundo a remover da imagem (vazia = a dos cantos); liga o remover fundo se estava desligado.
    FundoCor(String),
    Negrito,
    Italico,
    Sublinhado,
    Tachado,
    /// Liga o traço duplo (ou desliga, se já é duplo).
    SublinhadoDuplo,
    TachadoDuplo,
    Limpar,
    Entrelinha(f32),
    Colunas(u8),
    Alinhar(Alinhamento),
    Vertical(Vertical),
    Preenchimento(String),
    Contorno(String),
    Espessura(f32),
    Recorte(Tipo),
}

/// Os efeitos de uma imagem (opacidade e remover fundo), como a faixa precisa mostrá-los.
#[derive(Clone, Debug, PartialEq)]
pub struct EfeitosImagem {
    pub opacidade: f32,
    pub removido: f32,
    pub cor: String,
    pub suave: f32,
    pub modo: u8,
    pub refino: bool,
    pub borda: f32,
    pub pena: f32,
}

/// Para onde levar o elemento na pilha (quem fica por cima).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ordem {
    Frente,
    Tras,
    Avancar,
    Recuar,
}

/// Os tamanhos da lista de tamanhos, para o passo de aumentar/diminuir.
const TAMANHOS: [f32; 16] = [8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 16.0, 18.0, 20.0, 24.0, 28.0, 32.0, 36.0, 44.0, 54.0, 72.0];

fn passo_de_tamanho(atual: f32, passo: i32) -> f32 {
    if passo > 0 {
        TAMANHOS.iter().copied().find(|&t| t > atual + 0.01).unwrap_or(atual + 8.0)
    } else {
        TAMANHOS.iter().rev().copied().find(|&t| t < atual - 0.01).unwrap_or(atual.max(2.0) - 1.0).max(1.0)
    }
}

/// O tamanho e o estilo de um elemento novo de cada tipo.
fn molde(tipo: Tipo) -> (f32, f32, String, String, EstiloDoTexto) {
    let azul = "#4285f4".to_string();
    let borda = "#1a56c4".to_string();
    match tipo {
        Tipo::Texto => (320.0, 60.0, String::new(), String::new(), EstiloDoTexto::default()),
        Tipo::Linha => (240.0, 0.0, String::new(), "#202124".into(), EstiloDoTexto::default()),
        Tipo::Arco => (180.0, 180.0, String::new(), "#202124".into(), EstiloDoTexto::default()),
        Tipo::Circulo => (160.0, 160.0, azul, borda, EstiloDoTexto { cor: "#ffffff".into(), alinhamento: Alinhamento::Centro, vertical: Vertical::Meio, ..EstiloDoTexto::default() }),
        Tipo::Imagem => (320.0, 240.0, String::new(), String::new(), EstiloDoTexto::default()),
        _ => (200.0, 140.0, azul, borda, EstiloDoTexto { cor: "#ffffff".into(), alinhamento: Alinhamento::Centro, vertical: Vertical::Meio, ..EstiloDoTexto::default() }),
    }
}

/// O ponto (x, y) cai no elemento? (linha: perto do segmento; o resto: dentro do retângulo)
fn acerta(e: &Elemento, x: f32, y: f32) -> bool {
    if e.tipo == Tipo::Caminho {
        return crate::booleanas::contem_ponto(e, x, y);
    }
    if crate::novas_formas::suporta(e.tipo) {
        return crate::novas_formas::contem_ponto(e, x, y);
    }
    if e.tipo == Tipo::Linha {
        return distancia_ao_segmento(x, y, e.x, e.y, e.x + e.largura, e.y + e.altura) <= FOLGA_DA_LINHA.max(e.espessura);
    }
    let (x0, x1) = (e.x.min(e.x + e.largura), e.x.max(e.x + e.largura));
    let (y0, y1) = (e.y.min(e.y + e.altura), e.y.max(e.y + e.altura));
    x >= x0 && x <= x1 && y >= y0 && y <= y1
}

fn distancia_ao_segmento(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let (dx, dy) = (bx - ax, by - ay);
    let tamanho2 = dx * dx + dy * dy;
    let t = if tamanho2 == 0.0 { 0.0 } else { (((px - ax) * dx + (py - ay) * dy) / tamanho2).clamp(0.0, 1.0) };
    ((px - ax - t * dx).powi(2) + (py - ay - t * dy).powi(2)).sqrt()
}

/// Uma linha do painel de seleção.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemDeElemento {
    pub id: u64,
    pub tipo: &'static str,
    pub rotulo: String,
    pub oculto: bool,
    pub grupo: u32,
    pub selecionado: bool,
}

impl Sessao {
    /// Solta um slide sobre o slide atual: nasce a visão daquele slide (miniatura) que, na apresentação, leva a ele.
    /// `ponto` é o centro da miniatura (coordenadas do slide); sem ele, o meio.
    /// `imagem` é a imagem da visão salva do slide de destino (data URL PNG).
    pub fn gatilho_inserir(&mut self, destino: usize, ponto: Option<[f32; 2]>, imagem: Option<String>) -> Result<Mudanca, String> {
        if destino >= self.ap.slides.len() {
            return Err("Esse slide não existe.".into());
        }
        self.editar(|s| {
            let (sl, sa) = s.ap.tamanho();
            let [cx, cy] = ponto.unwrap_or([sl / 2.0, sa / 2.0]);
            let largura = (sl * 0.28).round();
            let altura = (sa * 0.28).round();
            let id = s.ap.novo_id();
            let tem_imagem = imagem.as_deref().map_or(false, |img| !img.is_empty());
            let e = Elemento {
                id,
                tipo: if tem_imagem { Tipo::Imagem } else { Tipo::Retangulo },
                x: (cx - largura / 2.0).clamp(0.0, (sl - largura).max(0.0)),
                y: (cy - altura / 2.0).clamp(0.0, (sa - altura).max(0.0)),
                largura,
                altura,
                preenchimento: if tem_imagem { String::new() } else { "#ffffff".into() },
                contorno: "#dadce0".into(),
                espessura: 1.0,
                imagem: imagem.unwrap_or_default().into(),
                texto: String::new(),
                destino: destino as u32 + 1,
                sombra: Sombra {
                    cor: "#00000030".into(),
                    desfoque: 8.0,
                    dx: 0.0,
                    dy: 2.0,
                },
                ..Elemento::default()
            };
            s.slide_mut().elementos.push(e);
            s.selecao = vec![id];
            Ok((vec![s.atual], false))
        })
    }

    /// O slide para onde o elemento `id` do slide `indice` leva, se for um gatilho.
    pub fn gatilho_do_elemento(&self, indice: usize, id: u64) -> Option<usize> {
        let e = self.ap.slides.get(indice)?.elementos.iter().find(|e| e.id == id)?;
        (e.destino > 0 && (e.destino as usize) <= self.ap.slides.len()).then(|| e.destino as usize - 1)
    }

    /// Insere um elemento no slide atual. Sem retângulo, nasce no meio do slide no tamanho padrão.
    pub fn elemento_inserir(&mut self, tipo: Tipo, rect: Option<[f32; 4]>, imagem: Option<String>) -> Result<Mudanca, String> {
        if tipo == Tipo::Imagem && imagem.as_deref().map_or(true, str::is_empty) {
            return Err("Faltou a imagem.".into());
        }
        self.editar(|s| {
            let (w, h, preenchimento, contorno, estilo) = molde(tipo);
            let (sl, sa) = s.ap.tamanho();
            let [x, y, mut largura, mut altura] = rect.unwrap_or([(sl - w) / 2.0, (sa - h) / 2.0, w, h]);
            if tipo == Tipo::Circulo { let d = largura.abs().min(altura.abs()); largura = d; altura = d; }
            let id = s.ap.novo_id();
            let e = Elemento { id, tipo, x, y, largura, altura, preenchimento, contorno, espessura: 1.5, estilo, imagem: imagem.unwrap_or_default().into(), ..Elemento::default() };
            s.slide_mut().elementos.push(e);
            s.selecao = vec![id];
            Ok((vec![s.atual], false))
        })
    }

    /// Insere um elemento com estilo e texto específicos.
    #[allow(clippy::too_many_arguments)]
    pub fn elemento_inserir_com_estilo(
        &mut self,
        tipo: Tipo,
        rect: Option<[f32; 4]>,
        texto: String,
        estilo: Option<EstiloDoTexto>,
        preenchimento: Option<Cor>,
        contorno: Option<Cor>,
        espessura: Option<f32>,
        sombra: Option<Sombra>,
        ajustes: Vec<f32>,
    ) -> Result<Mudanca, String> {
        self.editar(|s| {
            let (w, h, preench_padrao, contorno_padrao, estilo_padrao) = molde(tipo);
            let (sl, sa) = s.ap.tamanho();
            let [x, y, largura, altura] = rect.unwrap_or([(sl - w) / 2.0, (sa - h) / 2.0, w, h]);
            let id = s.ap.novo_id();
            let e = Elemento {
                id,
                tipo,
                x,
                y,
                largura,
                altura,
                preenchimento: preenchimento.unwrap_or(preench_padrao),
                contorno: contorno.unwrap_or(contorno_padrao),
                espessura: espessura.unwrap_or(1.5),
                texto,
                estilo: estilo.unwrap_or(estilo_padrao),
                sombra: sombra.unwrap_or_default(),
                ajustes,
                ..Elemento::default()
            };
            s.slide_mut().elementos.push(e);
            s.selecao = vec![id];
            Ok((vec![s.atual], false))
        })
    }

    /// Solta as formas do fundo do layout (as do mestre do `.pptx`) do slide atual (ou de todos):
    /// passam a ser elementos comuns, que se selecionam, movem, redimensionam e excluem.
    pub fn fundo_liberar(&mut self, todos: bool) -> Result<Mudanca, String> {
        self.editar(|s| {
            let alvos: Vec<usize> = if todos { (0..s.ap.slides.len()).collect() } else { vec![s.atual] };
            let mut soltas = 0;
            for &i in &alvos {
                for e in s.ap.slides[i].elementos.iter_mut().filter(|e| e.papel == Papel::Fundo) {
                    e.papel = Papel::Livre;
                    soltas += 1;
                }
            }
            if soltas == 0 {
                return Err("Não há formas de fundo para soltar.".into());
            }
            Ok((alvos, false))
        })
    }

    /// Manda os elementos selecionados para o fundo: ficam atrás e deixam de ser selecionáveis.
    pub fn selecao_para_fundo(&mut self) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                e.papel = Papel::Fundo;
            }
            s.selecao.clear();
            Ok((vec![s.atual], false))
        })
    }

    /// Insere uma tabela de verdade (um elemento só) com `linhas` × `colunas`, no meio do slide.
    pub fn tabela_inserir(&mut self, linhas: usize, colunas: usize) -> Result<Mudanca, String> {
        if !(1..=30).contains(&linhas) || !(1..=15).contains(&colunas) {
            return Err("A tabela deve ter de 1 a 30 linhas e de 1 a 15 colunas.".into());
        }
        self.editar(|s| {
            let (sl, sa) = s.ap.tamanho();
            let largura = (sl - 80.0).min(60.0 * colunas as f32 + 120.0).min(760.0);
            let altura = (34.0 * linhas as f32).min(sa * 0.75);
            let mut grade: crate::dados::Grade = vec![vec![String::new(); colunas]; linhas];
            for (c, cel) in grade[0].iter_mut().enumerate() {
                *cel = format!("Coluna {}", c + 1);
            }
            let id = s.ap.novo_id();
            let el = Elemento {
                id,
                tipo: Tipo::Tabela,
                x: (sl - largura) / 2.0,
                y: (sa - altura) / 2.0,
                largura,
                altura,
                dados: crate::dados::gravar_tabela(&grade),
                estilo: EstiloDoTexto { tamanho: 14.0, ..EstiloDoTexto::default() },
                ..Elemento::default()
            };
            s.slide_mut().elementos.push(el);
            s.selecao = vec![id];
            Ok((vec![s.atual], false))
        })
    }

    /// O elemento `id` do slide atual, se for do `tipo`.
    fn elemento_do_tipo(&mut self, id: u64, tipo: Tipo) -> Result<&mut Elemento, String> {
        self.slide_mut().elementos.iter_mut().find(|e| e.id == id && e.tipo == tipo).ok_or_else(|| "Esse elemento não está no slide.".to_string())
    }

    /// Escreve o texto da célula (`linha`, `coluna`) da tabela `id`.
    pub fn tabela_celula_definir(&mut self, id: u64, linha: usize, coluna: usize, texto: &str) -> Result<Mudanca, String> {
        let atual = self.slide().elementos.iter().find(|e| e.id == id && e.tipo == Tipo::Tabela).ok_or("Essa tabela não está no slide.")?;
        let mut grade = crate::dados::ler_tabela(atual);
        let celula = grade.get_mut(linha).and_then(|l| l.get_mut(coluna)).ok_or("Essa célula não existe.")?;
        if celula == texto {
            return Ok(self.mudou(Vec::new(), false));
        }
        *celula = texto.to_string();
        let dados = crate::dados::gravar_tabela(&grade);
        self.editar(|s| {
            s.elemento_do_tipo(id, Tipo::Tabela)?.dados = dados;
            Ok((vec![s.atual], false))
        })
    }

    /// Aplica uma grade já validada pelo adaptador Tables como UM comando
    /// reversível. Mantém id/estilo/layout do elemento e os passos Ctrl+Z/Y
    /// de toda a apresentação. O codec FSDZ continua dono da serialização.
    pub fn tabela_dados_definir(&mut self, id: u64, dados: &str) -> Result<Mudanca, String> {
        let atual = self.slide().elementos.iter()
            .find(|e| e.id == id && e.tipo == Tipo::Tabela)
            .ok_or("Essa tabela não está no slide.")?;
        // A interface impõe 30 x 15; validar novamente protege a API pública.
        let grade = crate::dados::decodificar(dados);
        if grade.is_empty()
            || grade.len() > 30
            || grade[0].is_empty()
            || grade[0].len() > 15
            || grade.iter().any(|linha| linha.len() != grade[0].len())
            || crate::dados::gravar_tabela(&grade) != dados
        {
            return Err("A grade da tabela é inválida (máximo 30 x 15).".into());
        }
        if atual.dados == dados {
            return Ok(self.mudou(Vec::new(), false));
        }
        let dados = dados.to_owned();
        self.editar(|s| {
            s.elemento_do_tipo(id, Tipo::Tabela)?.dados = dados;
            Ok((vec![s.atual], false))
        })
    }

    /// O texto da célula, para a interface começar a editar.
    pub fn tabela_celula_texto(&self, id: u64, linha: usize, coluna: usize) -> Option<String> {
        let e = self.slide().elementos.iter().find(|e| e.id == id && e.tipo == Tipo::Tabela)?;
        crate::dados::ler_tabela(e).get(linha)?.get(coluna).cloned()
    }

    /// A célula da tabela `id` sob o ponto (x, y) do slide.
    pub fn tabela_celula_no_ponto(&self, id: u64, x: f32, y: f32) -> Option<(usize, usize)> {
        let e = self.slide().elementos.iter().find(|e| e.id == id && e.tipo == Tipo::Tabela)?;
        crate::dados::celula_no_ponto(e, x, y)
    }

    /// O retângulo da célula (para a interface marcar a que está em edição).
    pub fn tabela_celula_retangulo(&self, id: u64, linha: usize, coluna: usize) -> Option<[f32; 4]> {
        let e = self.slide().elementos.iter().find(|e| e.id == id && e.tipo == Tipo::Tabela)?;
        crate::dados::celula_retangulo(e, linha, coluna)
    }

    /// Quantas linhas e colunas a tabela tem.
    pub fn tabela_tamanho(&self, id: u64) -> Option<(usize, usize)> {
        let e = self.slide().elementos.iter().find(|e| e.id == id && e.tipo == Tipo::Tabela)?;
        let g = crate::dados::ler_tabela(e);
        Some((g.len(), g[0].len()))
    }

    /// Acrescenta ou tira uma linha ou coluna: `acao` = `linha+`, `linha-`, `coluna+`, `coluna-`
    /// (a linha e a coluna do fim; a tabela mantém o tamanho na tela).
    pub fn tabela_estrutura(&mut self, id: u64, acao: &str) -> Result<Mudanca, String> {
        let atual = self.slide().elementos.iter().find(|e| e.id == id && e.tipo == Tipo::Tabela).ok_or("Essa tabela não está no slide.")?;
        let mut g = crate::dados::ler_tabela(atual);
        match acao {
            "linha+" if g.len() < 30 => g.push(vec![String::new(); g[0].len()]),
            "linha-" if g.len() > 1 => {
                g.pop();
            }
            "coluna+" if g[0].len() < 15 => g.iter_mut().for_each(|l| l.push(String::new())),
            "coluna-" if g[0].len() > 1 => g.iter_mut().for_each(|l| {
                l.pop();
            }),
            _ => return Err("Não dá para fazer isso nessa tabela.".into()),
        }
        let dados = crate::dados::gravar_tabela(&g);
        self.editar(|s| {
            s.elemento_do_tipo(id, Tipo::Tabela)?.dados = dados;
            Ok((vec![s.atual], false))
        })
    }

    /// Insere um gráfico de verdade (`colunas`, `barras`, `pizza` ou `linhas`) com dados de exemplo.
    pub fn grafico_inserir(&mut self, tipo: &str) -> Result<Mudanca, String> {
        self.editar(|s| {
            let (sl, sa) = s.ap.tamanho();
            let (w, h) = (520.0_f32.min(sl - 60.0), 320.0_f32.min(sa - 60.0));
            let id = s.ap.novo_id();
            let el = Elemento {
                id,
                tipo: Tipo::Grafico,
                x: (sl - w) / 2.0,
                y: (sa - h) / 2.0,
                largura: w,
                altura: h,
                dados: crate::dados::gravar_grafico(&crate::dados::grafico_exemplo(tipo)),
                ..Elemento::default()
            };
            s.slide_mut().elementos.push(el);
            s.selecao = vec![id];
            Ok((vec![s.atual], false))
        })
    }

    /// Os dados do gráfico `id`: tipo, título e a grade (nomes das séries na linha 0, das categorias na coluna 0).
    pub fn grafico_dados(&self, id: u64) -> Option<crate::dados::Grafico> {
        let e = self.slide().elementos.iter().find(|e| e.id == id && e.tipo == Tipo::Grafico)?;
        Some(crate::dados::ler_grafico(e))
    }

    /// Troca os dados do gráfico `id` (tipo, título e grade) de uma vez.
    pub fn grafico_definir(&mut self, id: u64, g: crate::dados::Grafico) -> Result<Mudanca, String> {
        if !["colunas", "barras", "linhas", "pizza"].contains(&g.tipo.as_str()) {
            return Err("Tipo de gráfico desconhecido.".into());
        }
        let dados = crate::dados::gravar_grafico(&g);
        self.editar(|s| {
            let e = s.elemento_do_tipo(id, Tipo::Grafico)?;
            if e.dados == dados {
                return Ok((Vec::new(), false));
            }
            e.dados = dados;
            Ok((vec![s.atual], false))
        })
    }

    /// O elemento de cima no ponto (x, y) do slide atual, se houver.
    pub fn elemento_no_ponto(&self, x: f32, y: f32) -> Option<u64> {
        self.slide().elementos.iter().rev().filter(|e| e.papel != Papel::Fundo && !e.oculto).find(|e| acerta(e, x, y)).map(|e| e.id)
    }

    /// Clique no slide: seleciona o elemento de cima (com `somar`, junta ou tira da seleção).
    /// Devolve o elemento acertado.
    pub fn selecionar_no_ponto(&mut self, x: f32, y: f32, somar: bool) -> Option<u64> {
        let achado = self.elemento_no_ponto(x, y);
        match (achado, somar) {
            (Some(id), true) => {
                if let Some(i) = self.selecao.iter().position(|&s| s == id) {
                    self.selecao.remove(i);
                } else {
                    self.selecao.push(id);
                }
            }
            (Some(id), false) => {
                if !self.selecao.contains(&id) {
                    self.selecao = vec![id];
                }
            }
            (None, false) => self.selecao.clear(),
            (None, true) => {}
        }
        self.expandir_grupos();
        achado
    }

    /// Quem está num grupo entra na seleção junto com o resto do grupo.
    fn expandir_grupos(&mut self) {
        let grupos: Vec<u32> = self.slide().elementos.iter().filter(|e| e.grupo != 0 && self.selecao.contains(&e.id)).map(|e| e.grupo).collect();
        if grupos.is_empty() {
            return;
        }
        let extras: Vec<u64> = self.slide().elementos.iter().filter(|e| grupos.contains(&e.grupo) && !e.oculto).map(|e| e.id).collect();
        for id in extras {
            if !self.selecao.contains(&id) {
                self.selecao.push(id);
            }
        }
    }

    /// Seleciona os elementos que cabem inteiros no retângulo (a seleção por arrasto).
    pub fn selecionar_na_area(&mut self, rect: [f32; 4], somar: bool) {
        let [x, y, w, h] = rect;
        let dentro = |e: &&Elemento| e.x.min(e.x + e.largura) >= x && e.y.min(e.y + e.altura) >= y && e.x.max(e.x + e.largura) <= x + w && e.y.max(e.y + e.altura) <= y + h;
        let ids: Vec<u64> = self.slide().elementos.iter().filter(|e| e.papel != Papel::Fundo && !e.oculto).filter(dentro).map(|e| e.id).collect();
        if !somar {
            self.selecao.clear();
        }
        for id in ids {
            if !self.selecao.contains(&id) {
                self.selecao.push(id);
            }
        }
        self.expandir_grupos();
    }

    /// Seleciona tudo no slide atual.
    pub fn selecionar_tudo(&mut self) {
        self.selecao = self.slide().elementos.iter().filter(|e| e.papel != Papel::Fundo && !e.oculto).map(|e| e.id).collect();
    }

    /// Soma o elemento à seleção, ou o tira dela se já estava (a lista do painel de seleção usa).
    pub fn alternar_selecao(&mut self, id: u64) {
        if !self.slide().elementos.iter().any(|e| e.id == id && !e.oculto) {
            return;
        }
        if let Some(i) = self.selecao.iter().position(|&s| s == id) {
            self.selecao.remove(i);
        } else {
            self.selecao.push(id);
            self.expandir_grupos();
        }
    }

    /// Tira a seleção.
    pub fn desmarcar(&mut self) {
        self.selecao.clear();
    }

    /// Seleciona o elemento pelo ID no slide atual (se existir).
    pub fn selecionar_por_id(&mut self, id: u64) -> bool {
        if self.slide().elementos.iter().any(|e| e.id == id) {
            self.selecao = vec![id];
            self.expandir_grupos();
            true
        } else {
            self.selecao.clear();
            false
        }
    }

    pub(crate) fn exigir_selecao(&self) -> Result<(), String> {
        if self.selecao.is_empty() {
            Err("Selecione um elemento primeiro.".into())
        } else {
            Ok(())
        }
    }

    /// Os elementos selecionados do slide atual, para mudar.
    pub(crate) fn selecionados_mut(&mut self) -> impl Iterator<Item = &mut Elemento> {
        let sel = self.selecao.clone();
        self.slide_mut().elementos.iter_mut().filter(move |e| sel.contains(&e.id))
    }

    /// Move a seleção `dx`, `dy` pontos.
    pub fn selecao_mover(&mut self, dx: f32, dy: f32) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                e.x += dx;
                e.y += dy;
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Dá ao elemento `id` o retângulo novo (redimensionar pelas alças).
    pub fn elemento_geometria(&mut self, id: u64, rect: [f32; 4]) -> Result<Mudanca, String> {
        let [x, y, w, h] = rect;
        self.editar(|s| {
            let e = s.slide_mut().elementos.iter_mut().find(|e| e.id == id).ok_or("Esse elemento não está no slide.")?;
            let linha = e.tipo == Tipo::Linha;
            e.x = x;
            e.y = y;
            e.largura = if linha { w } else { w.max(MINIMO) };
            e.altura = if linha { h } else { h.max(MINIMO) };
            Ok((vec![s.atual], false))
        })
    }

    /// Redimensiona toda a seleção pela alça de `id`, com uma única escala e um único desfazer.
    pub fn selecao_redimensionar(&mut self, id: u64, rect: [f32; 4]) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        if !self.selecao.contains(&id) { return Err("O elemento não está selecionado.".into()); }
        let e = self.slide().elementos.iter().find(|e| e.id == id).ok_or("Esse elemento não está no slide.")?;
        let escala = crate::escala::Escala::entre([e.x, e.y, e.largura, e.altura], rect)
            .ok_or("O tamanho deve ser positivo e finito." )?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                [e.x, e.y, e.largura, e.altura] = escala.retangulo([e.x, e.y, e.largura, e.altura]);
                if e.tipo == Tipo::Circulo {
                    let lado = e.largura.abs().min(e.altura.abs()).max(4.0);
                    e.largura = lado;
                    e.altura = lado;
                }
                e.estilo.tamanho *= escala.fator;
                e.espessura *= escala.fator;
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Os ajustes da forma dos elementos selecionados (no setor: ângulo inicial e final, em graus).
    pub fn selecao_ajustes(&mut self, ajustes: Vec<f32>) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                e.ajustes = ajustes.clone();
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Dá ao elemento `id` a rotação (em graus anti-horário).
    pub fn elemento_rotacao(&mut self, id: u64, graus: f32) -> Result<Mudanca, String> {
        self.editar(|s| {
            let e = s.slide_mut().elementos.iter_mut().find(|e| e.id == id).ok_or("Esse elemento não está no slide.")?;
            e.rotacao = graus;
            Ok((vec![s.atual], false))
        })
    }

    /// O texto do elemento `id` (o que se escreveu na caixa).
    pub fn texto_definir(&mut self, id: u64, texto: &str) -> Result<Mudanca, String> {
        let atual = self.slide().elementos.iter().find(|e| e.id == id).ok_or("Esse elemento não está no slide.")?;
        if !atual.tipo.aceita_texto() {
            return Err("Esse elemento não leva texto.".into());
        }
        if atual.texto == texto {
            return Ok(Mudanca { versao: self.versao, slides: Vec::new(), lista: false });
        }
        self.editar(|s| {
            if let Some(e) = s.slide_mut().elementos.iter_mut().find(|e| e.id == id) {
                crate::texto_formato::atualizar_indices(e, texto);
                e.texto = texto.into();
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Exclui os elementos selecionados.
    pub fn selecao_excluir(&mut self) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            let sel = std::mem::take(&mut s.selecao);
            s.slide_mut().elementos.retain(|e| !sel.contains(&e.id));
            Ok((vec![s.atual], false))
        })
    }

    /// Aplica a formatação aos elementos selecionados. Negrito, itálico e sublinhado
    /// alternam pelo estado do primeiro (como no PowerPoint).
    pub fn formatar(&mut self, f: Formatacao) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        let primeiro = self.slide().elementos.iter().find(|e| self.selecao.contains(&e.id)).map(|e| e.estilo.clone()).unwrap_or_default();
        self.editar(|s| {
            for e in s.selecionados_mut() {
                aplicar(e, &f, &primeiro);
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Como `formatar`, mas como prévia: nada vai para o desfazer a cada chamada. Só o estado de
    /// antes da primeira chamada é guardado, e vira um único passo em `confirmar_previa` (que
    /// qualquer outra edição, desfazer ou refazer também chama). É o que o seletor de cor usa
    /// enquanto se arrasta: sem uma cópia da apresentação por movimento do mouse.
    pub fn formatar_previa(&mut self, f: Formatacao) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        if self.previa.is_none() {
            self.previa_sujo = self.sujo;
            self.previa = Some(self.retrato());
        }
        let primeiro = self.slide().elementos.iter().find(|e| self.selecao.contains(&e.id)).map(|e| e.estilo.clone()).unwrap_or_default();
        for e in self.selecionados_mut() {
            aplicar(e, &f, &primeiro);
        }
        let atual = self.atual;
        Ok(self.mudou(vec![atual], false))
    }

    /// Altera a forma de corte dos elementos selecionados (ex.: elipse para foto redonda).
    pub fn elemento_recorte(&mut self, forma: Tipo) -> Result<Mudanca, String> {
        self.formatar(Formatacao::Recorte(forma))
    }

    /// A forma de corte atual do primeiro elemento selecionado (se houver).
    pub fn selecao_recorte(&self) -> Option<Tipo> {
        self.selecao.first()
            .and_then(|id| self.slide().elementos.iter().find(|e| e.id == *id))
            .map(|e| e.recorte)
    }

    /// Ajusta a posição e o zoom da imagem dentro da figura de recorte.
    pub fn imagem_ajustar_recorte(&mut self, dx: f32, dy: f32, delta_zoom: f32) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                if e.tipo != Tipo::Imagem {
                    continue;
                }
                let pan_x = e.ajustes.first().copied().unwrap_or(0.0);
                let pan_y = e.ajustes.get(1).copied().unwrap_or(0.0);
                let zoom = e.ajustes.get(2).copied().unwrap_or(1.0);
                let raio = e.ajustes.get(3).copied().unwrap_or(0.16);

                let novo_pan_x = (pan_x + dx).clamp(-1.0, 1.0);
                let novo_pan_y = (pan_y + dy).clamp(-1.0, 1.0);
                let novo_zoom = (zoom + delta_zoom).clamp(0.2, 5.0);

                let margens = if e.ajustes.len() >= 8 { e.ajustes[4..8].to_vec() } else { Vec::new() };
                e.ajustes = vec![novo_pan_x, novo_pan_y, novo_zoom, raio];
                e.ajustes.extend(margens);
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Configura um efeito não destrutivo para todas as imagens selecionadas.
    pub fn imagem_visual_definir(&mut self, campo: &str, valor: f32) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        let mut valida = crate::imagem::VisualImagem::default();
        valida.definir(campo, valor)?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                if e.tipo == Tipo::Imagem { e.visual_imagem.definir(campo, valor)?; }
            }
            Ok((vec![s.atual], false))
        })
    }

    pub fn imagem_texto_alternativo(&mut self, texto: &str) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        if texto.len() > 4096 { return Err("Texto alternativo maior que 4 KB.".into()); }
        let texto = texto.to_string();
        self.editar(move |s| {
            for e in s.selecionados_mut() {
                if e.tipo == Tipo::Imagem { e.texto_alternativo.clone_from(&texto); }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Substitui a fonte da imagem preservando geometria, máscara e formatação.
    pub fn imagem_substituir(&mut self, origem: String, compressao: bool) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        if !origem.starts_with("data:image/") || !origem.contains(";base64,") {
            return Err("É necessário selecionar uma imagem local válida.".into());
        }
        self.editar(move |s| {
            for e in s.selecionados_mut() {
                if e.tipo == Tipo::Imagem {
                    e.imagem = origem.clone().into();
                    e.visual_imagem.preservar_original = !compressao;
                }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Liga uma sombra suave na imagem, usando a estrutura de sombra do elemento.
    pub fn imagem_sombra(&mut self, ligado: bool) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                if e.tipo == Tipo::Imagem {
                    e.sombra = if ligado {
                        Sombra { cor: "#00000080".into(), desfoque: 8.0, dx: 5.0, dy: 6.0 }
                    } else { Sombra::default() };
                }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Redefine todos os efeitos da imagem sem descartar os bytes originais.
    pub fn imagem_efeitos_resetar(&mut self) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                if e.tipo == Tipo::Imagem {
                    e.visual_imagem = crate::imagem::VisualImagem::default();
                }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Recorta a origem pela margem escolhida, sem destruir os bytes originais.
    /// As margens são frações do arquivo fonte (0.0..0.45).
    pub fn imagem_cortar_margem(&mut self, borda: &str, valor: f32) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        if !valor.is_finite() { return Err("Margem de corte inválida.".into()); }
        let indice = match borda {
            "esquerda" => 4, "topo" => 5, "direita" => 6, "base" => 7,
            _ => return Err("Borda de corte inválida.".into()),
        };
        self.editar(move |s| {
            for e in s.selecionados_mut() {
                if e.tipo != Tipo::Imagem { continue; }
                if e.ajustes.is_empty() { e.ajustes = vec![0.0,0.0,1.0,0.16]; }
                e.ajustes.resize(8, 0.0);
                e.ajustes[indice] = valor.clamp(0.0,0.45);
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Opacidade (0..1) das imagens selecionadas.
    pub fn imagem_opacidade(&mut self, valor: f32) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                if e.tipo == Tipo::Imagem {
                    e.opacidade = valor.clamp(0.0, 1.0);
                }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Remove o fundo (a cor do canto) das imagens selecionadas com a tolerância dada (0 desliga).
    /// Só muda a exibição: os bytes da imagem continuam inteiros.
    pub fn imagem_remover_fundo(&mut self, tolerancia: f32) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                if e.tipo == Tipo::Imagem {
                    e.fundo_removido = tolerancia.clamp(0.0, 1.0);
                }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Suavidade (0 duro … 1 suave) da borda do "remover fundo" nas imagens selecionadas.
    pub fn imagem_suavidade_fundo(&mut self, valor: f32) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                if e.tipo == Tipo::Imagem {
                    e.fundo_suave = valor.clamp(0.0, 1.0);
                }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Técnica do remover fundo nas imagens selecionadas: 0 = cor para alfa, 1 = só o fundo ligado à borda, 2 = ignorar o brilho (LAB), 3 = watershed.
    pub fn imagem_modo_fundo(&mut self, modo: u8) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                if e.tipo == Tipo::Imagem {
                    e.fundo_modo = modo.min(5);
                }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Acrescenta uma marca ao watershed nas imagens selecionadas: `manter` = parte do objeto,
    /// senão parte do fundo; `ret` é [x0, y0, x1, y1] em coordenadas da imagem (0 a 1).
    pub fn imagem_marca_adicionar(&mut self, manter: bool, ret: [f32; 4]) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                if e.tipo == Tipo::Imagem {
                    if !e.fundo_marcas.is_empty() {
                        e.fundo_marcas.push(';');
                    }
                    let r = ret.map(|v| v.clamp(0.0, 1.0));
                    e.fundo_marcas.push_str(&format!("{}{:.4},{:.4},{:.4},{:.4}", if manter { '+' } else { '-' }, r[0], r[1], r[2], r[3]));
                    if e.fundo_removido <= 0.0 {
                        e.fundo_removido = 0.25;
                    }
                }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Apaga as marcas do watershed das imagens selecionadas.
    pub fn imagem_marcas_limpar(&mut self) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                if e.tipo == Tipo::Imagem {
                    e.fundo_marcas.clear();
                }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Quantas marcas (manter, remover) a imagem selecionada tem.
    pub fn imagem_marcas_contagem(&self) -> (usize, usize) {
        self.selecao
            .first()
            .and_then(|id| self.slide().elementos.iter().find(|e| e.id == *id))
            .map(|e| (e.fundo_marcas.split(';').filter(|m| m.starts_with('+')).count(), e.fundo_marcas.split(';').filter(|m| m.starts_with('-')).count()))
            .unwrap_or((0, 0))
    }

    /// Liga ou desliga o refino da borda (filtro guiado) do remover fundo.
    pub fn imagem_refinar_borda(&mut self, ligado: bool) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                if e.tipo == Tipo::Imagem {
                    e.fundo_refino = ligado;
                }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Escolhe a cor do fundo a remover (`#rrggbb`); vazia volta à cor dos cantos.
    pub fn imagem_cor_do_fundo(&mut self, cor: &str) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        let cor = cor.to_string();
        self.editar(move |s| {
            for e in s.selecionados_mut() {
                if e.tipo == Tipo::Imagem {
                    e.fundo_cor = cor.clone();
                    if !cor.is_empty() && e.fundo_removido <= 0.0 {
                        e.fundo_removido = 0.25;
                    }
                }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Os efeitos da imagem selecionada (para a faixa mostrar o valor atual).
    pub fn imagem_efeitos(&self) -> Option<EfeitosImagem> {
        self.selecao.first().and_then(|id| self.slide().elementos.iter().find(|e| e.id == *id)).map(|e| EfeitosImagem {
            opacidade: e.opacidade,
            removido: e.fundo_removido,
            cor: e.fundo_cor.clone(),
            suave: e.fundo_suave,
            modo: e.fundo_modo,
            refino: e.fundo_refino,
            borda: e.fundo_borda,
            pena: e.fundo_pena,
        })
    }

    /// Ajusta a borda do fundo removido nas imagens selecionadas: `borda` em pixels (+ expande o
    /// objeto, − contrai) e `pena` (suavização) em pixels.
    pub fn imagem_ajuste_borda(&mut self, borda: f32, pena: f32) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                if e.tipo == Tipo::Imagem {
                    e.fundo_borda = borda.clamp(-30.0, 30.0);
                    e.fundo_pena = pena.clamp(0.0, 30.0);
                    if e.fundo_removido <= 0.0 {
                        e.fundo_removido = 0.25;
                    }
                }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Redefine o enquadramento da imagem dentro do recorte para o centro padrão.
    pub fn imagem_resetar_recorte(&mut self) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            for e in s.selecionados_mut() {
                if e.tipo != Tipo::Imagem {
                    continue;
                }
                let raio = e.ajustes.get(3).copied().unwrap_or(0.16);
                e.ajustes = vec![0.0, 0.0, 1.0, raio];
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Alterna marcadores ("• ") ou numeração ("1. ") nas linhas de texto dos elementos selecionados.
    pub fn lista_alternar(&mut self, tipo: &str) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        let tipo = tipo.to_string();
        self.editar(move |s| {
            let mut tocou = false;
            for e in s.selecionados_mut() {
                if !e.tipo.aceita_texto() || e.texto.trim().is_empty() {
                    continue;
                }
                tocou = true;
                let linhas: Vec<&str> = e.texto.split('\n').collect();
                let novas = match tipo.as_str() {
                    "marcadores" => alternar_marcadores(&linhas),
                    "numerada" => alternar_numerada(&linhas),
                    _ => return Err(format!("Tipo de lista desconhecido: {tipo}")),
                };
                let texto=novas.join("\n");
                crate::texto_formato::atualizar_indices(e,&texto);
                e.texto = texto;
            }
            if !tocou {
                return Err("Nenhum texto selecionado para formatar como lista.".into());
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Leva a seleção para a frente, para trás, uma casa acima ou abaixo na pilha.
    pub fn organizar(&mut self, ordem: Ordem) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            let sel = s.selecao.clone();
            let lista = &mut s.slide_mut().elementos;
            reordenar(lista, &sel, ordem);
            Ok((vec![s.atual], false))
        })
    }

    /// Alinha a seleção no slide (`esquerda`, `centro`, `direita`, `topo`, `meio`, `base`).
    pub fn alinhar_no_slide(&mut self, como: &str) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        let (sl, sa) = self.ap.tamanho();
        let como = como.to_string();
        self.editar(move |s| {
            for e in s.selecionados_mut() {
                match como.as_str() {
                    "esquerda" => e.x = 0.0,
                    "centro" => e.x = (sl - e.largura) / 2.0,
                    "direita" => e.x = sl - e.largura,
                    "topo" => e.y = 0.0,
                    "meio" => e.y = (sa - e.altura) / 2.0,
                    "base" => e.y = sa - e.altura,
                    _ => return Err(format!("Alinhamento desconhecido: {como}")),
                }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Alinha os elementos selecionados entre si (pelo retângulo que envolve todos). Pede dois ou mais.
    pub fn alinhar_entre_si(&mut self, como: &str) -> Result<Mudanca, String> {
        if self.selecao.len() < 2 {
            return Err("Selecione dois ou mais elementos para alinhar entre si.".into());
        }
        let como = como.to_string();
        self.editar(move |s| {
            let caixas: Vec<[f32; 4]> = s.selecionados_mut().map(|e| [e.x, e.y, e.largura, e.altura]).collect();
            let minx = caixas.iter().map(|c| c[0]).fold(f32::MAX, f32::min);
            let maxx = caixas.iter().map(|c| c[0] + c[2]).fold(f32::MIN, f32::max);
            let miny = caixas.iter().map(|c| c[1]).fold(f32::MAX, f32::min);
            let maxy = caixas.iter().map(|c| c[1] + c[3]).fold(f32::MIN, f32::max);
            for e in s.selecionados_mut() {
                match como.as_str() {
                    "esquerda" => e.x = minx,
                    "centro" => e.x = (minx + maxx) / 2.0 - e.largura / 2.0,
                    "direita" => e.x = maxx - e.largura,
                    "topo" => e.y = miny,
                    "meio" => e.y = (miny + maxy) / 2.0 - e.altura / 2.0,
                    "base" => e.y = maxy - e.altura,
                    _ => return Err(format!("Alinhamento desconhecido: {como}")),
                }
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Distribui os elementos selecionados com o mesmo espaço entre eles, na horizontal (`"h"`) ou na vertical (`"v"`).
    /// Os dois das pontas ficam onde estão. Pede três ou mais.
    pub fn distribuir(&mut self, eixo: &str) -> Result<Mudanca, String> {
        if self.selecao.len() < 3 {
            return Err("Selecione três ou mais elementos para distribuir.".into());
        }
        let horizontal = match eixo {
            "h" => true,
            "v" => false,
            _ => return Err(format!("Eixo desconhecido: {eixo}")),
        };
        self.editar(move |s| {
            let mut itens: Vec<(u64, f32, f32)> = s.selecionados_mut().map(|e| if horizontal { (e.id, e.x, e.largura) } else { (e.id, e.y, e.altura) }).collect();
            itens.sort_by(|a, b| a.1.total_cmp(&b.1));
            let inicio = itens[0].1;
            let fim = itens.iter().map(|i| i.1 + i.2).fold(f32::MIN, f32::max);
            let soma: f32 = itens.iter().map(|i| i.2).sum();
            let vao = (fim - inicio - soma) / (itens.len() - 1) as f32;
            let mut cursor = inicio;
            for (id, _, tamanho) in itens {
                if let Some(e) = s.selecionados_mut().find(|e| e.id == id) {
                    if horizontal { e.x = cursor } else { e.y = cursor }
                }
                cursor += tamanho + vao;
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Esconde (ou mostra de novo) o elemento `id`.
    pub fn elemento_ocultar(&mut self, id: u64, oculto: bool) -> Result<Mudanca, String> {
        self.editar(|s| {
            let e = s.slide_mut().elementos.iter_mut().find(|e| e.id == id).ok_or("Esse elemento não está no slide.")?;
            e.oculto = oculto;
            s.selecao.retain(|&x| x != id || !oculto);
            Ok((vec![s.atual], false))
        })
    }

    /// Junta os elementos selecionados num grupo (movem-se e se selecionam juntos). Pede dois ou mais.
    pub fn agrupar(&mut self) -> Result<Mudanca, String> {
        if self.selecao.len() < 2 {
            return Err("Selecione dois ou mais elementos para agrupar.".into());
        }
        self.editar(|s| {
            let novo = s.slide().elementos.iter().map(|e| e.grupo).max().unwrap_or(0) + 1;
            for e in s.selecionados_mut() {
                e.grupo = novo;
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Desfaz os grupos dos elementos selecionados (e dos que estavam juntos com eles).
    pub fn desagrupar(&mut self) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        self.editar(|s| {
            let grupos: Vec<u32> = s.selecionados_mut().map(|e| e.grupo).filter(|&g| g != 0).collect();
            if grupos.is_empty() {
                return Err("A seleção não tem grupo.".to_string());
            }
            for e in s.slide_mut().elementos.iter_mut().filter(|e| grupos.contains(&e.grupo)) {
                e.grupo = 0;
            }
            Ok((vec![s.atual], false))
        })
    }

    /// Os elementos do slide atual para o painel de seleção: o de cima primeiro (o fundo do layout fica de fora).
    pub fn lista_de_elementos(&self) -> Vec<ItemDeElemento> {
        self.slide()
            .elementos
            .iter()
            .rev()
            .filter(|e| e.papel != Papel::Fundo)
            .map(|e| {
                let texto: String = e.texto.lines().next().unwrap_or("").chars().take(26).collect();
                let rotulo = if texto.trim().is_empty() { e.tipo.nome().to_string() } else { texto };
                ItemDeElemento { id: e.id, tipo: e.tipo.nome(), rotulo, oculto: e.oculto, grupo: e.grupo, selecionado: self.selecao.contains(&e.id) }
            })
            .collect()
    }

    /// Guarda a seleção para colar; devolve o texto dela (para a área de transferência do sistema).
    pub fn copiar(&mut self) -> String {
        let sel = self.selecao.clone();
        self.transferencia = self.slide().elementos.iter().filter(|e| sel.contains(&e.id)).cloned().collect();
        self.texto_da_transferencia()
    }

    fn texto_da_transferencia(&self) -> String {
        self.transferencia.iter().map(|e| e.texto.as_str()).filter(|t| !t.is_empty()).collect::<Vec<_>>().join("\n")
    }

    /// Copia e exclui a seleção.
    pub fn recortar(&mut self) -> Result<(String, Mudanca), String> {
        self.exigir_selecao()?;
        let texto = self.copiar();
        Ok((texto, self.selecao_excluir()?))
    }

    /// Cola. Se o texto do sistema é o que o FeatherSlide copiou, cola os elementos (com o
    /// estilo); senão, o texto vira uma caixa de texto nova.
    pub fn colar(&mut self, texto_do_sistema: &str) -> Result<Mudanca, String> {
        let nossos = !self.transferencia.is_empty() && (texto_do_sistema.is_empty() || texto_do_sistema == self.texto_da_transferencia());
        if nossos {
            return self.colar_elementos();
        }
        if texto_do_sistema.trim().is_empty() {
            return Err("Não há nada para colar.".into());
        }
        let texto = texto_do_sistema.replace("\r\n", "\n");
        self.elemento_inserir(Tipo::Texto, None, None)?;
        let id = self.selecao[0];
        // Junta o "inserir" e o "escrever" num passo só do desfazer.
        let inserido = self.desfazer.pop();
        let m = self.texto_definir(id, &texto)?;
        if let (Some(r), Some(ultimo)) = (inserido, self.desfazer.last_mut()) {
            *ultimo = r;
        }
        Ok(m)
    }

    /// Preserva a aparência das cópias internas. Não descarta silenciosamente estilos externos.
    pub fn colar_com_formatacao_original(&mut self, texto: &str) -> Result<Mudanca, String> {
        if self.transferencia.is_empty() || (!texto.is_empty() && texto != self.texto_da_transferencia()) {
            return Err("A formatação original só está disponível para elementos copiados no FeatherSlide. Para texto de outros programas, use Colar sem formatação.".into());
        }
        self.colar_elementos()
    }

    fn colar_elementos(&mut self) -> Result<Mudanca, String> {
        let copias = self.transferencia.clone();
        self.editar(|s| {
            let mut grupos = std::collections::HashMap::new();
            let mut proximo = s.slide().elementos.iter().map(|e|e.grupo).max().unwrap_or(0);
            for e in &copias {
                if e.grupo > 0 && !grupos.contains_key(&e.grupo) {
                    proximo=proximo.checked_add(1).ok_or("Limite de grupos no slide atingido.")?;
                    grupos.insert(e.grupo,proximo);
                }
            }
            let mut novos = Vec::new();
            let mut mapa=Vec::new();
            let inicio=s.slide().elementos.len();
            for mut e in copias {
                if let Some(&grupo)=grupos.get(&e.grupo){e.grupo=grupo;}
                let antigo=e.id;e.id = s.ap.novo_id();mapa.push((antigo,e.id));
                e.x += DESLOCAMENTO_DA_COPIA;
                e.y += DESLOCAMENTO_DA_COPIA;
                e.papel = Papel::Livre;
                novos.push(e.id);
                s.slide_mut().elementos.push(e);
            }
            s.selecao = novos;
            crate::outras_galerias::remapear_conectores(&mut s.slide_mut().elementos[inicio..],&mapa);
            Ok((vec![s.atual], false))
        })?;
        // A próxima colagem anda mais um passo, como no PowerPoint.
        for e in &mut self.transferencia {
            e.x += DESLOCAMENTO_DA_COPIA;
            e.y += DESLOCAMENTO_DA_COPIA;
        }
        Ok(Mudanca { versao: self.versao, slides: vec![self.atual], lista: false })
    }

    /// Duplica a seleção no mesmo slide (Ctrl+D).
    pub fn selecao_duplicar(&mut self) -> Result<Mudanca, String> {
        self.exigir_selecao()?;
        let guardado = std::mem::take(&mut self.transferencia);
        self.copiar();
        let r = self.colar_elementos();
        self.transferencia = guardado;
        r
    }
}

/// Aplica uma formatação a um elemento; `primeiro` é o estilo do primeiro selecionado.
fn aplicar(e: &mut Elemento, f: &Formatacao, primeiro: &EstiloDoTexto) {
    let est = &mut e.estilo;
    match f {
        Formatacao::Fonte(nome) => est.fonte.clone_from(nome),
        Formatacao::Tamanho(t) => est.tamanho = t.clamp(1.0, 400.0),
        Formatacao::TamanhoPasso(p) => est.tamanho = passo_de_tamanho(est.tamanho, *p),
        Formatacao::CorDoTexto(c) => est.cor.clone_from(c),
        Formatacao::Realce(c) => est.realce.clone_from(c),
        Formatacao::FundoCor(c) => {
            if e.tipo == Tipo::Imagem {
                e.fundo_cor.clone_from(c);
                if !c.is_empty() && e.fundo_removido <= 0.0 {
                    e.fundo_removido = 0.25;
                }
            }
        }
        Formatacao::Negrito => est.negrito = !primeiro.negrito,
        Formatacao::Italico => est.italico = !primeiro.italico,
        Formatacao::Sublinhado => { est.sublinhado = !primeiro.sublinhado; est.sublinhado_duplo = false; }
        Formatacao::Tachado => { est.tachado = !primeiro.tachado; est.tachado_duplo = false; }
        Formatacao::SublinhadoDuplo => { let ligar = !(primeiro.sublinhado && primeiro.sublinhado_duplo); est.sublinhado = ligar; est.sublinhado_duplo = ligar; }
        Formatacao::TachadoDuplo => { let ligar = !(primeiro.tachado && primeiro.tachado_duplo); est.tachado = ligar; est.tachado_duplo = ligar; }
        Formatacao::Limpar => { *est = EstiloDoTexto::default();e.indices_texto.clear();e.espacamento_letras=0.0; },
        Formatacao::Entrelinha(v) => est.entrelinha = v.clamp(0.5, 5.0),
        Formatacao::Colunas(c) => est.colunas = (*c).clamp(1, 2),
        Formatacao::Alinhar(a) => est.alinhamento = *a,
        Formatacao::Vertical(v) => est.vertical = *v,
        Formatacao::Preenchimento(c) | Formatacao::Contorno(c) if e.tipo == Tipo::Icone => {
            // O ícone é um traço só: pintar por qualquer dos dois controles muda a cor dele.
            e.preenchimento.clone_from(c);
            e.contorno.clone_from(c);
        }
        Formatacao::Preenchimento(c) => { e.preenchimento.clone_from(c); e.gradiente = None; },
        Formatacao::Contorno(c) => e.contorno.clone_from(c),
        Formatacao::Espessura(w) => e.espessura = w.clamp(0.0, 50.0),
        Formatacao::Recorte(r) => e.recorte = *r,
    }
}

fn alternar_marcadores(linhas: &[&str]) -> Vec<String> {
    let eh_marcador = |l: &str| l.trim_start().starts_with("• ");
    let todas_com = linhas.iter().filter(|l| !l.trim().is_empty()).all(|l| eh_marcador(l));
    linhas
        .iter()
        .map(|&l| {
            if l.trim().is_empty() {
                l.to_string()
            } else if todas_com {
                let recuo = l.len() - l.trim_start().len();
                let sem = l.trim_start();
                let resto = sem.strip_prefix("• ").unwrap_or(sem);
                format!("{}{resto}", &l[..recuo])
            } else {
                let recuo = l.len() - l.trim_start().len();
                let limpo = limpar_prefixo_de_lista(l.trim_start());
                format!("{}• {limpo}", &l[..recuo])
            }
        })
        .collect()
}

fn alternar_numerada(linhas: &[&str]) -> Vec<String> {
    let eh_num = |l: &str| {
        let sem = l.trim_start();
        sem.find(". ").is_some_and(|i| !sem[..i].is_empty() && sem[..i].chars().all(|c| c.is_ascii_digit()))
    };
    let todas_com = linhas.iter().filter(|l| !l.trim().is_empty()).all(|l| eh_num(l));
    let mut num = 1;
    linhas
        .iter()
        .map(|&l| {
            if l.trim().is_empty() {
                l.to_string()
            } else if todas_com {
                let recuo = l.len() - l.trim_start().len();
                let sem = l.trim_start();
                let resto = sem.find(". ").map_or(sem, |i| &sem[i + 2..]);
                format!("{}{resto}", &l[..recuo])
            } else {
                let recuo = l.len() - l.trim_start().len();
                let limpo = limpar_prefixo_de_lista(l.trim_start());
                let res = format!("{recuo_str}{num}. {limpo}", recuo_str = &l[..recuo]);
                num += 1;
                res
            }
        })
        .collect()
}

fn limpar_prefixo_de_lista(s: &str) -> &str {
    if let Some(r) = s.strip_prefix("• ") {
        r
    } else if let Some(i) = s.find(". ") {
        if !s[..i].is_empty() && s[..i].chars().all(|c| c.is_ascii_digit()) {
            &s[i + 2..]
        } else {
            s
        }
    } else {
        s
    }
}

/// Muda a ordem da pilha: o fim da lista é quem fica por cima.
fn reordenar(lista: &mut Vec<Elemento>, sel: &[u64], ordem: Ordem) {
    let escolhido = |e: &Elemento| sel.contains(&e.id);
    match ordem {
        Ordem::Frente | Ordem::Tras => {
            let (mut sim, nao): (Vec<Elemento>, Vec<Elemento>) = lista.drain(..).partition(|e| escolhido(e));
            if ordem == Ordem::Frente {
                *lista = nao;
                lista.append(&mut sim);
            } else {
                sim.extend(nao);
                *lista = sim;
            }
        }
        Ordem::Avancar => {
            for i in (0..lista.len().saturating_sub(1)).rev() {
                if escolhido(&lista[i]) && !escolhido(&lista[i + 1]) {
                    lista.swap(i, i + 1);
                }
            }
        }
        Ordem::Recuar => {
            for i in 1..lista.len() {
                if escolhido(&lista[i]) && !escolhido(&lista[i - 1]) {
                    lista.swap(i, i - 1);
                }
            }
        }
    }
}
