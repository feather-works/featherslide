//! Os dados da apresentação. Coordenadas em **pontos** (1/72 pol.), com a origem no
//! canto de cima à esquerda do slide: o 16:9 tem 960 × 540 pt, como o PowerPoint e o
//! Google Apresentações. A tela nunca entra aqui — o zoom é da interface.

use serde::{Deserialize, Serialize};

/// Uma cor `#rrggbb`, ou `#rrggbbaa` com transparência. Vazia quer dizer "sem cor" (sem
/// preenchimento, sem contorno).
pub type Cor = String;

/// A proporção do slide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Proporcao {
    /// 16:9, o padrão (960 × 540 pt).
    Larga,
    /// 4:3, a dos projetores antigos (720 × 540 pt).
    Padrao,
}

impl Proporcao {
    /// Largura e altura do slide, em pontos.
    pub fn tamanho(self) -> (f32, f32) {
        match self {
            Proporcao::Larga => (960.0, 540.0),
            Proporcao::Padrao => (720.0, 540.0),
        }
    }
}

/// O que um elemento é.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tipo {
    /// Caixa de texto (sem fundo, sem contorno, por padrão).
    Texto,
    Retangulo,
    RetanguloArredondado,
    Elipse,
    Triangulo,
    Seta,
    Linha,
    /// Uma imagem: o endereço (`data:` embutido) fica em `Elemento::imagem`.
    Imagem,
    /// Fatia de pizza da elipse: os ângulos em `Elemento::ajustes` (o `pie` do PowerPoint).
    Setor,
    /// Tabela de verdade: a grade de células fica em `Elemento::dados` (ver `dados.rs`).
    Tabela,
    /// Gráfico de verdade: tipo, título e a grade de dados em `Elemento::dados`.
    Grafico,
    /// Ícone vetorial da biblioteca (`icones.rs`): o nome em `dados`, a cor em `preenchimento`.
    Icone,
    Losango,
    Pentagono,
    Hexagono,
    Octogono,
    Trapezio,
    Paralelogramo,
    Estrela,
    Cruz,
    SetaEsquerda,
    SetaDupla,
    SetaCima,
    SetaBaixo,
    Chevron,
    Documento,
    /// Texto artístico editável, com contorno, sombra e transformação vetorial.
    WordArt,
    /// Caminho vetorial composto resultante da mesclagem de formas; contornos em `dados`.
    Caminho,
    // Acrescentar novas variantes SOMENTE ao final: o .fsdz usa códigos posicionais.
    TrianguloRetangulo,
    Heptagono,
    Decagono,
    Dodecagono,
    /// Formas básicas vetoriais adicionais. Acrescentadas ao final para manter códigos .fsdz.
    Circulo,
    Anel,
    Arco,
    Corda,
    Cubo,
    Cilindro,
    Moldura,
    CantoEmL,
    Placa,
    Sol,
    Lua,
    Coracao,
    Raio,
    Nuvem,
    Estrela4,
    Estrela6,
    Estrela8,
    Estrela10,
    Estrela12,
}

impl Tipo {
    /// Todos, na ordem dos códigos gravados no arquivo.
    pub const TODOS: [Tipo; 51] = [Tipo::Texto, Tipo::Retangulo, Tipo::RetanguloArredondado, Tipo::Elipse, Tipo::Triangulo, Tipo::Seta, Tipo::Linha, Tipo::Imagem, Tipo::Setor, Tipo::Tabela, Tipo::Grafico, Tipo::Icone, Tipo::Losango, Tipo::Pentagono, Tipo::Hexagono, Tipo::Octogono, Tipo::Trapezio, Tipo::Paralelogramo, Tipo::Estrela, Tipo::Cruz, Tipo::SetaEsquerda, Tipo::SetaDupla, Tipo::SetaCima, Tipo::SetaBaixo, Tipo::Chevron, Tipo::Documento, Tipo::WordArt, Tipo::Caminho, Tipo::TrianguloRetangulo, Tipo::Heptagono, Tipo::Decagono, Tipo::Dodecagono, Tipo::Circulo, Tipo::Anel, Tipo::Arco, Tipo::Corda, Tipo::Cubo, Tipo::Cilindro, Tipo::Moldura, Tipo::CantoEmL, Tipo::Placa, Tipo::Sol, Tipo::Lua, Tipo::Coracao, Tipo::Raio, Tipo::Nuvem, Tipo::Estrela4, Tipo::Estrela6, Tipo::Estrela8, Tipo::Estrela10, Tipo::Estrela12];

    /// O tipo pelo nome que a interface manda (`retangulo`, `elipse`…).
    pub fn pelo_nome(nome: &str) -> Option<Tipo> {
        Tipo::TODOS.into_iter().find(|t| t.nome() == nome)
    }

    /// O nome do tipo, como vai para a interface.
    pub fn nome(self) -> &'static str {
        match self {
            Tipo::Texto => "texto",
            Tipo::Retangulo => "retangulo",
            Tipo::RetanguloArredondado => "retangulo_arredondado",
            Tipo::Elipse => "elipse",
            Tipo::Triangulo => "triangulo",
            Tipo::Seta => "seta",
            Tipo::Linha => "linha",
            Tipo::Imagem => "imagem",
            Tipo::Setor => "setor",
            Tipo::Tabela => "tabela",
            Tipo::Grafico => "grafico",
            Tipo::Icone => "icone",
            Tipo::Losango => "losango",
            Tipo::Pentagono => "pentagono",
            Tipo::Hexagono => "hexagono",
            Tipo::Octogono => "octogono",
            Tipo::Trapezio => "trapezio",
            Tipo::Paralelogramo => "paralelogramo",
            Tipo::Estrela => "estrela",
            Tipo::Cruz => "cruz",
            Tipo::SetaEsquerda => "seta_esquerda",
            Tipo::SetaDupla => "seta_dupla",
            Tipo::SetaCima => "seta_cima",
            Tipo::SetaBaixo => "seta_baixo",
            Tipo::Chevron => "chevron",
            Tipo::Documento => "documento",
            Tipo::WordArt => "wordart",
            Tipo::Caminho => "caminho",
            Tipo::TrianguloRetangulo => "triangulo_retangulo",
            Tipo::Heptagono => "heptagono",
            Tipo::Decagono => "decagono",
            Tipo::Dodecagono => "dodecagono",
            Tipo::Circulo => "circulo",
            Tipo::Anel => "anel",
            Tipo::Arco => "arco",
            Tipo::Corda => "corda",
            Tipo::Cubo => "cubo",
            Tipo::Cilindro => "cilindro",
            Tipo::Moldura => "moldura",
            Tipo::CantoEmL => "canto_em_l",
            Tipo::Placa => "placa",
            Tipo::Sol => "sol",
            Tipo::Lua => "lua",
            Tipo::Coracao => "coracao",
            Tipo::Raio => "raio",
            Tipo::Nuvem => "nuvem",
            Tipo::Estrela4 => "estrela4",
            Tipo::Estrela6 => "estrela6",
            Tipo::Estrela8 => "estrela8",
            Tipo::Estrela10 => "estrela10",
            Tipo::Estrela12 => "estrela12",

        }
    }

    /// A forma leva texto dentro? (linha e imagem não)
    pub fn aceita_texto(self) -> bool {
        !matches!(self, Tipo::Linha | Tipo::Imagem | Tipo::Tabela | Tipo::Grafico | Tipo::Icone | Tipo::Caminho | Tipo::Arco)
    }
}

/// O papel de uma caixa criada pelo layout do slide: diz qual dica aparece quando está vazia.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Papel {
    /// Elemento posto à mão (sem dica).
    Livre,
    Titulo,
    Subtitulo,
    Corpo,
    /// Faz parte do fundo do slide (as formas do layout e do mestre de um `.pptx`): fica
    /// atrás e não se seleciona com clique, área nem "Selecionar tudo".
    Fundo,
}

impl Papel {
    /// Todos, na ordem dos códigos gravados no arquivo.
    pub const TODOS: [Papel; 5] = [Papel::Livre, Papel::Titulo, Papel::Subtitulo, Papel::Corpo, Papel::Fundo];

    /// Caixa do layout (título, subtítulo, corpo): tem dica e acompanha a troca de layout.
    pub fn do_layout(self) -> bool {
        matches!(self, Papel::Titulo | Papel::Subtitulo | Papel::Corpo)
    }
}

/// Alinhamento horizontal do texto.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alinhamento {
    Esquerda,
    Centro,
    Direita,
    Justificado,
}

impl Alinhamento {
    pub const TODOS: [Alinhamento; 4] = [Alinhamento::Esquerda, Alinhamento::Centro, Alinhamento::Direita, Alinhamento::Justificado];
}

/// Alinhamento vertical do texto dentro da caixa.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vertical {
    Topo,
    Meio,
    Base,
}

impl Vertical {
    pub const TODOS: [Vertical; 3] = [Vertical::Topo, Vertical::Meio, Vertical::Base];
}

fn entrelinha_padrao() -> f32 {
    1.2
}

fn colunas_padrao() -> u8 {
    1
}

/// O estilo do texto de um elemento (vale para o texto inteiro da caixa).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EstiloDoTexto {
    pub fonte: String,
    /// Tamanho em pontos.
    pub tamanho: f32,
    pub cor: Cor,
    /// Cor de fundo atrás do texto (marca-texto); vazia = sem fundo.
    #[serde(default)]
    pub realce: Cor,
    pub negrito: bool,
    pub italico: bool,
    pub sublinhado: bool,
    #[serde(default)]
    pub tachado: bool,
    /// Com `sublinhado`/`tachado` ligado, desenha dois traços em vez de um.
    #[serde(default)]
    pub sublinhado_duplo: bool,
    #[serde(default)]
    pub tachado_duplo: bool,
    #[serde(default = "entrelinha_padrao")]
    pub entrelinha: f32,
    #[serde(default = "colunas_padrao")]
    pub colunas: u8,
    pub alinhamento: Alinhamento,
    pub vertical: Vertical,
}

impl Default for EstiloDoTexto {
    fn default() -> Self {
        Self {
            fonte: "Liberation Sans".into(),
            tamanho: 18.0,
            cor: "#202124".into(),
            realce: String::new(),
            negrito: false,
            italico: false,
            sublinhado: false,
            tachado: false,
            sublinhado_duplo: false,
            tachado_duplo: false,
            entrelinha: 1.2,
            colunas: 1,
            alinhamento: Alinhamento::Esquerda,
            vertical: Vertical::Topo,
        }
    }
}

/// A sombra projetada de um elemento (cor vazia: sem sombra), do `feather-shapes`: os mesmos
/// campos de sempre (`cor`, `desfoque`, `dx`, `dy`, em pontos).
pub use feather_shapes::aparencia::Sombra;

/// Um elemento do slide: caixa de texto, forma ou imagem.
/// O endereço da imagem (`data:image/...;base64,...`), compartilhado: copiar um elemento (o
/// desfazer guarda a apresentação inteira a cada edição) não copia os megabytes da imagem, só
/// conta mais uma referência. Vai para o arquivo como texto, igual a uma `String`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Imagem(std::sync::Arc<str>);

impl std::ops::Deref for Imagem {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl Imagem {
    /// Se as duas apontam para os mesmos bytes (copiar um elemento não copia a imagem).
    pub fn compartilha(&self, outra: &Imagem) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &outra.0)
    }

    /// Para `skip_serializing_if` do serde.
    pub fn esta_vazia(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for Imagem {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

impl From<&str> for Imagem {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl Serialize for Imagem {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Imagem {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d).map(Imagem::from)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Elemento {
    pub id: u64,
    pub tipo: Tipo,
    pub papel: Papel,
    pub x: f32,
    pub y: f32,
    pub largura: f32,
    pub altura: f32,
    /// Graus, sentido horário (como na tela).
    pub rotacao: f32,
    pub preenchimento: Cor,
    #[serde(default)]
    pub gradiente: Option<crate::estilo_forma::Gradiente>,
    pub contorno: Cor,
    /// Espessura do contorno, em pontos.
    pub espessura: f32,
    pub texto: String,
    #[serde(default)]
    pub indices_texto: Vec<crate::texto_formato::Indice>,
    #[serde(default)]
    pub espacamento_letras: f32,
    pub estilo: EstiloDoTexto,
    /// O endereço da imagem (`data:image/...;base64,...`), só no tipo `Imagem`.
    pub imagem: Imagem,
    /// Correções de cor, filtros, perspectiva e espelhamento não destrutivos.
    #[serde(default)]
    pub visual_imagem: crate::imagem::VisualImagem,
    /// Texto alternativo acessível (descritivo, não exibido sobre a foto).
    #[serde(default)]
    pub texto_alternativo: String,
    /// Os ajustes da forma. No setor, os ângulos inicial e final em graus (horário, a partir
    /// das 3 horas); no retângulo arredondado, o raio dos cantos de cima e de baixo, em fração
    /// do lado menor. Vazio: o desenho padrão da forma.
    #[serde(default)]
    pub ajustes: Vec<f32>,
    /// A forma que recorta a imagem (elipse para foto redonda); `Retangulo` é sem recorte.
    #[serde(default = "sem_recorte")]
    pub recorte: Tipo,
    #[serde(default)]
    pub sombra: Sombra,
    /// Os dados da tabela ou do gráfico, em texto compacto (ver `dados.rs`).
    #[serde(default)]
    pub dados: String,
    /// O efeito de entrada do elemento na apresentação.
    #[serde(default)]
    pub animacao: Animacao,
    /// Gatilho: na apresentação, clicar no elemento leva ao slide `destino - 1` (0 = sem gatilho).
    #[serde(default)]
    pub destino: u32,
    /// Hiperlink: na apresentação, clicar no elemento abre este endereço (vazio = sem link).
    #[serde(default)]
    pub link: String,
    /// Escondido: não aparece no slide nem na apresentação (volta pelo painel de seleção).
    #[serde(default)]
    pub oculto: bool,
    /// Grupo (0 = sem grupo): os elementos do mesmo grupo se selecionam e se movem juntos.
    #[serde(default)]
    pub grupo: u32,
    /// Opacidade da imagem (1 = opaca).
    #[serde(default = "opaca")]
    pub opacidade: f32,
    /// Remover fundo da imagem: tolerância (0 = desligado). Só na exibição; o arquivo da imagem não muda.
    #[serde(default)]
    pub fundo_removido: f32,
    /// Cor do fundo a remover (`#rrggbb`); vazia = a cor dos cantos da imagem.
    #[serde(default)]
    pub fundo_cor: Cor,
    /// Suavidade da borda do "remover fundo": 0 = corte duro, 1 = degradê longo.
    #[serde(default = "suavidade_padrao")]
    pub fundo_suave: f32,
    /// Técnica do remover fundo: 0 = cor para alfa (a imagem toda), 1 = só o fundo ligado à borda, 2 = ignorar o brilho (LAB), 3 = watershed, 4 = seleção por cor das marcas, 5 = cor transparente exata.
    #[serde(default)]
    pub fundo_modo: u8,
    /// Refinar a borda do fundo removido com um filtro guiado pela própria imagem.
    #[serde(default)]
    pub fundo_refino: bool,
    /// Marcas do watershed, em coordenadas da imagem (0 a 1): `+x0,y0,x1,y1;-x0,y0,x1,y1`.
    /// `+` = manter (parte do objeto), `-` = remover (parte do fundo).
    #[serde(default)]
    pub fundo_marcas: String,
    /// Ajuste da borda do fundo removido, em pixels: positivo expande o objeto, negativo contrai.
    #[serde(default)]
    pub fundo_borda: f32,
    /// Suavização (pena) da borda do fundo removido, em pixels.
    #[serde(default)]
    pub fundo_pena: f32,
}

fn suavidade_padrao() -> f32 {
    0.5
}

fn opaca() -> f32 {
    1.0
}

fn sem_recorte() -> Tipo {
    Tipo::Retangulo
}

impl Default for Elemento {
    /// Uma caixa de texto vazia em (0, 0), sem tamanho: o ponto de partida dos construtores.
    fn default() -> Self {
        Self {
            id: 0,
            tipo: Tipo::Texto,
            papel: Papel::Livre,
            x: 0.0,
            y: 0.0,
            largura: 0.0,
            altura: 0.0,
            rotacao: 0.0,
            preenchimento: String::new(),
            gradiente: None,
            contorno: String::new(),
            espessura: 1.0,
            texto: String::new(),
            indices_texto: Vec::new(),
            espacamento_letras: 0.0,
            estilo: EstiloDoTexto::default(),
            imagem: Imagem::default(),
            visual_imagem: crate::imagem::VisualImagem::default(),
            texto_alternativo: String::new(),
            ajustes: Vec::new(),
            recorte: Tipo::Retangulo,
            sombra: Sombra::default(),
            dados: String::new(),
            animacao: Animacao::default(),
            destino: 0,
            link: String::new(),
            oculto: false,
            grupo: 0,
            opacidade: 1.0,
            fundo_removido: 0.0,
            fundo_cor: String::new(),
            fundo_suave: 0.5,
            fundo_modo: 0,
            fundo_refino: false,
            fundo_marcas: String::new(),
            fundo_borda: 0.0,
            fundo_pena: 0.0,
        }
    }
}

/// O efeito da passagem de um slide para o seguinte.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TipoTransicao {
    #[default]
    Nenhuma,
    /// O slide novo aparece aos poucos.
    Esmaecer,
    /// O slide novo desliza por cima do antigo.
    Deslizar,
    /// O slide novo empurra o antigo para fora.
    Empurrar,
    /// O slide antigo sai, revelando o novo por baixo.
    Revelar,
    /// Uma cortina varre a tela e mostra o slide novo.
    Cortina,
    /// O slide novo cresce do centro.
    Zoom,
}

impl TipoTransicao {
    /// Todos, na ordem dos códigos gravados no arquivo.
    pub const TODOS: [TipoTransicao; 7] = [
        TipoTransicao::Nenhuma,
        TipoTransicao::Esmaecer,
        TipoTransicao::Deslizar,
        TipoTransicao::Empurrar,
        TipoTransicao::Revelar,
        TipoTransicao::Cortina,
        TipoTransicao::Zoom,
    ];

    pub fn nome(self) -> &'static str {
        match self {
            TipoTransicao::Nenhuma => "Nenhuma",
            TipoTransicao::Esmaecer => "Esmaecer",
            TipoTransicao::Deslizar => "Deslizar",
            TipoTransicao::Empurrar => "Empurrar",
            TipoTransicao::Revelar => "Revelar",
            TipoTransicao::Cortina => "Cortina",
            TipoTransicao::Zoom => "Zoom",
        }
    }

    /// Esse efeito anda numa direção?
    pub fn tem_direcao(self) -> bool {
        matches!(self, TipoTransicao::Deslizar | TipoTransicao::Empurrar | TipoTransicao::Revelar | TipoTransicao::Cortina)
    }
}

/// De onde vem o movimento de uma transição ou animação.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Direcao {
    Esquerda,
    #[default]
    Direita,
    Cima,
    Baixo,
}

impl Direcao {
    pub const TODOS: [Direcao; 4] = [Direcao::Esquerda, Direcao::Direita, Direcao::Cima, Direcao::Baixo];

    pub fn nome(self) -> &'static str {
        match self {
            Direcao::Esquerda => "Da esquerda",
            Direcao::Direita => "Da direita",
            Direcao::Cima => "De cima",
            Direcao::Baixo => "De baixo",
        }
    }
}

fn duracao_padrao() -> f32 {
    0.7
}

/// A transição que leva a este slide (o efeito de entrada dele).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transicao {
    pub tipo: TipoTransicao,
    pub direcao: Direcao,
    /// Duração em segundos.
    #[serde(default = "duracao_padrao")]
    pub duracao: f32,
    /// Avançar sozinho depois de tantos segundos (0 = só ao clicar).
    #[serde(default)]
    pub apos: f32,
}

impl Default for Transicao {
    fn default() -> Self {
        Self { tipo: TipoTransicao::Nenhuma, direcao: Direcao::Direita, duracao: 0.7, apos: 0.0 }
    }
}

/// O efeito de entrada de um elemento na apresentação.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TipoAnimacao {
    #[default]
    Nenhuma,
    /// Surge de uma vez.
    Aparecer,
    Esmaecer,
    /// Entra voando de fora do slide.
    Voar,
    /// Cresce do centro.
    Zoom,
    /// Uma cortina varre o elemento.
    Revelar,
    /// Pisca de leve chamando a atenção (ênfase).
    Pulsar,
    // Acrescentar SEMPRE ao final: .fsdz guarda códigos posicionais (v9: 0..6).
    Surgir,
    Flutuar,
    Dividir,
    Apagar,
    Forma,
    Roda,
    Barras,
    Girar,
    Rebote,
    Deslizar,
    Expandir,
    Contrair,
}

impl TipoAnimacao {
    pub const TODOS: [TipoAnimacao; 19] = [
        TipoAnimacao::Nenhuma,
        TipoAnimacao::Aparecer,
        TipoAnimacao::Esmaecer,
        TipoAnimacao::Voar,
        TipoAnimacao::Zoom,
        TipoAnimacao::Revelar,
        TipoAnimacao::Pulsar,
        TipoAnimacao::Surgir,
        TipoAnimacao::Flutuar,
        TipoAnimacao::Dividir,
        TipoAnimacao::Apagar,
        TipoAnimacao::Forma,
        TipoAnimacao::Roda,
        TipoAnimacao::Barras,
        TipoAnimacao::Girar,
        TipoAnimacao::Rebote,
        TipoAnimacao::Deslizar,
        TipoAnimacao::Expandir,
        TipoAnimacao::Contrair,
    ];

    pub fn nome(self) -> &'static str {
        match self {
            TipoAnimacao::Nenhuma => "Nenhuma",
            TipoAnimacao::Aparecer => "Aparecer",
            TipoAnimacao::Esmaecer => "Esmaecer",
            TipoAnimacao::Voar => "Voar para dentro",
            TipoAnimacao::Zoom => "Zoom",
            TipoAnimacao::Revelar => "Revelar",
            TipoAnimacao::Pulsar => "Pulsar",
            TipoAnimacao::Surgir => "Surgir",
            TipoAnimacao::Flutuar => "Flutuar",
            TipoAnimacao::Dividir => "Dividir",
            TipoAnimacao::Apagar => "Apagar",
            TipoAnimacao::Forma => "Forma",
            TipoAnimacao::Roda => "Roda",
            TipoAnimacao::Barras => "Barras",
            TipoAnimacao::Girar => "Girar",
            TipoAnimacao::Rebote => "Rebote",
            TipoAnimacao::Deslizar => "Deslizar",
            TipoAnimacao::Expandir => "Expandir",
            TipoAnimacao::Contrair => "Contrair",
        }
    }

    pub fn tem_direcao(self) -> bool {
        matches!(self, TipoAnimacao::Voar | TipoAnimacao::Revelar
            | TipoAnimacao::Surgir | TipoAnimacao::Flutuar | TipoAnimacao::Apagar
            | TipoAnimacao::Deslizar)
    }
}

impl std::fmt::Display for TipoAnimacao {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.nome())
    }
}

#[cfg(test)]
mod animacoes_entrada_testes {
    use super::TipoAnimacao;

    #[test]
    fn codigos_v9_permanecem_nas_mesmas_posicoes() {
        assert_eq!(&TipoAnimacao::TODOS[..7], &[
            TipoAnimacao::Nenhuma, TipoAnimacao::Aparecer, TipoAnimacao::Esmaecer,
            TipoAnimacao::Voar, TipoAnimacao::Zoom, TipoAnimacao::Revelar,
            TipoAnimacao::Pulsar,
        ]);
        assert_eq!(TipoAnimacao::TODOS.len(), 19);
        let diferentes: std::collections::HashSet<_> = TipoAnimacao::TODOS.iter()
            .map(|v| v.nome()).collect();
        assert_eq!(diferentes.len(), 19);
    }
}

/// Quando a animação de um elemento começa.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InicioAnimacao {
    /// Espera um clique (ou a tecla de avançar).
    #[default]
    AoClicar,
    /// Junto com a anterior.
    ComAnterior,
    /// Logo depois que a anterior termina.
    AposAnterior,
}

impl InicioAnimacao {
    pub const TODOS: [InicioAnimacao; 3] = [InicioAnimacao::AoClicar, InicioAnimacao::ComAnterior, InicioAnimacao::AposAnterior];

    pub fn nome(self) -> &'static str {
        match self {
            InicioAnimacao::AoClicar => "Ao clicar",
            InicioAnimacao::ComAnterior => "Com a anterior",
            InicioAnimacao::AposAnterior => "Após a anterior",
        }
    }
}

/// A animação de um elemento: efeito, quando começa, quanto dura e de onde vem.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Animacao {
    pub tipo: TipoAnimacao,
    pub inicio: InicioAnimacao,
    pub direcao: Direcao,
    /// Duração em segundos.
    pub duracao: f32,
    /// Espera antes de começar, em segundos.
    pub atraso: f32,
}

impl Default for Animacao {
    fn default() -> Self {
        Self { tipo: TipoAnimacao::Nenhuma, inicio: InicioAnimacao::AoClicar, direcao: Direcao::Esquerda, duracao: 0.5, atraso: 0.0 }
    }
}

/// Um slide.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slide {
    pub id: u64,
    /// O nome do layout com que nasceu (`titulo`, `titulo_conteudo`…).
    pub layout: String,
    pub fundo: Cor,
    #[serde(default)]
    pub fundo_config: crate::configuracao_slide::Fundo,
    #[serde(default)]
    pub ocultar_mestre: bool,
    pub elementos: Vec<Elemento>,
    /// As anotações do apresentador.
    pub notas: String,
    /// Oculto: fica no arquivo, mas a apresentação pula.
    pub oculto: bool,
    /// O efeito de entrada deste slide na apresentação.
    #[serde(default)]
    pub transicao: Transicao,
    /// Não vazio: este slide **começa** uma seção com esse nome (os seguintes, até a próxima, são dela).
    #[serde(default)]
    pub secao: String,
    /// Os comentários de revisão presos ao slide.
    #[serde(default)]
    pub comentarios: Vec<Comentario>,
}

/// Um comentário de revisão num slide.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comentario {
    pub autor: String,
    pub texto: String,
    /// Resolvido: continua no arquivo, mas a lista o mostra apagado.
    #[serde(default)]
    pub resolvido: bool,
}

/// Uma fonte embutida na apresentação (lida de um `.pptx` ou gravada no `.fsdz`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FonteEmbutida {
    /// O nome da família (ex.: "Maven Pro", "Nunito").
    pub nome: String,
    /// Bytes TTF/OTF da variante normal (regular).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regular: Option<Vec<u8>>,
    /// Bytes TTF/OTF da variante em negrito (bold).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negrito: Option<Vec<u8>>,
    /// Bytes TTF/OTF da variante em itálico (italic).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub italico: Option<Vec<u8>>,
    /// Bytes TTF/OTF da variante em negrito e itálico (bold italic).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negrito_italico: Option<Vec<u8>>,
}

/// A apresentação inteira.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Apresentacao {
    pub titulo: String,
    pub autor: String,
    pub proporcao: Proporcao,
    /// Dimensões personalizadas em pontos; zero usa a proporção padrão.
    #[serde(default)]
    pub dimensoes: [f32; 2],
    pub slides: Vec<Slide>,
    /// O próximo `id` livre (slides e elementos dividem a mesma conta).
    pub proximo_id: u64,
    /// As fontes embutidas na apresentação.
    #[serde(default)]
    pub fontes: Vec<FonteEmbutida>,
    /// O tema de cores e fontes aplicado (id em `temas.rs`; vazio = o padrão).
    #[serde(default)]
    pub tema: String,
    /// O modelo (`modelos.rs`) de onde a apresentação nasceu: slides novos ganham o mesmo visual.
    #[serde(default)]
    pub modelo: String,
    /// Propriedades do documento, preservadas no .fsdz e disponíveis ao usuário.
    #[serde(default)]
    pub metadados: crate::metadados::Metadados,
}

impl Apresentacao {
    /// Largura e altura do slide, em pontos.
    pub fn tamanho(&self) -> (f32, f32) {
        if self.dimensoes[0] > 0.0 && self.dimensoes[1] > 0.0 {
            (self.dimensoes[0], self.dimensoes[1])
        } else { self.proporcao.tamanho() }
    }

    /// Um `id` novo, nunca usado nesta apresentação.
    pub fn novo_id(&mut self) -> u64 {
        let id = self.proximo_id;
        self.proximo_id += 1;
        id
    }
}
