//! Os modelos (templates) da tela de início: apresentações prontas, com tema, slides de exemplo
//! e transição, para quem não quer começar do zero. Só dados; a interface escolhe e abre.

use crate::modelo::{Alinhamento, Apresentacao, Direcao, Elemento, Papel, Proporcao, Slide, Tipo, TipoTransicao};
use crate::temas;
use crate::sessao::slide_com_layout;

pub struct Modelo {
    pub id: &'static str,
    pub nome: &'static str,
    pub descricao: &'static str,
    /// O tema do modelo (`temas.rs`), para a interface pintar o cartão.
    pub tema: &'static str,
}

pub const MODELOS: [Modelo; 19] = [
    Modelo { id: "em_branco", nome: "Em branco", descricao: "Um slide de título, sem nada mais.", tema: "office" },
    Modelo { id: "negocios", nome: "Negócios", descricao: "Faixas diagonais em azul-marinho com detalhe dourado.", tema: "executivo" },
    Modelo { id: "aula", nome: "Aula", descricao: "Objetivos, conteúdo, exemplos e exercícios.", tema: "floresta" },
    Modelo { id: "projeto", nome: "Projeto", descricao: "Mapa de rotas: linhas e estações para cada etapa.", tema: "rota" },
    Modelo { id: "relatorio", nome: "Relatório", descricao: "Resumo, resultados e conclusões, em tom sóbrio.", tema: "papel" },
    Modelo { id: "palestra", nome: "Palestra", descricao: "Letras enormes e faixa de legenda, como em palestra gravada.", tema: "conferencia" },
    Modelo { id: "tecnologia", nome: "Tecnologia", descricao: "Fundo noturno, linhas finas e destaque néon.", tema: "neon" },
    Modelo { id: "elegante", nome: "Elegante", descricao: "Art déco: verde-escuro, dourado e cantos em degraus.", tema: "deco" },
    Modelo { id: "criativo", nome: "Criativo", descricao: "Círculos e faixas coloridas, tom alegre.", tema: "solar" },
    Modelo { id: "minimalista", nome: "Minimalista", descricao: "Muito espaço em branco e um único detalhe de cor.", tema: "grafite" },
    Modelo { id: "pitch", nome: "Pitch", descricao: "Apresentação de ideia ou startup, em roxo.", tema: "roxo" },
    Modelo { id: "resultados", nome: "Resultados", descricao: "Barras que sobem e verde-azulado sobre papel.", tema: "negocio" },
    Modelo { id: "caderno", nome: "Caderno", descricao: "Pauta, margem vermelha e marca-texto.", tema: "caderno" },
    Modelo { id: "canteiro", nome: "Canteiro", descricao: "Cronograma em barras e laranja de obra.", tema: "obra" },
    Modelo { id: "memorando", nome: "Memorando", descricao: "Coluna de margem e texto de relatório técnico.", tema: "memo" },
    Modelo { id: "holofote", nome: "Holofote", descricao: "Palco escuro com feixe de luz e poça de luz.", tema: "palco" },
    Modelo { id: "planta", nome: "Planta", descricao: "Planta azul com grade, cotas e carimbo.", tema: "planta" },
    Modelo { id: "bauhaus", nome: "Bauhaus", descricao: "Círculo, meio-disco e losango em cores primárias.", tema: "bauhaus" },
    Modelo { id: "orbita", nome: "Órbita", descricao: "Planeta, anel e satélite coral sobre índigo.", tema: "orbita" },
];

/// Um slide: layout e os textos (título, e o corpo ou subtítulo).
type Tela = (&'static str, &'static str, &'static str);

fn telas(id: &str) -> Vec<Tela> {
    // Os modelos novos usam o roteiro de slides de um dos anteriores.
    let id = match id {
        "resultados" => "negocios",
        "caderno" => "aula",
        "canteiro" => "projeto",
        "memorando" => "relatorio",
        "holofote" => "palestra",
        "planta" => "tecnologia",
        "bauhaus" => "criativo",
        "orbita" => "pitch",
        outro => outro,
    };
    match id {
        "negocios" => vec![
            ("titulo", "Título da proposta", "Nome da empresa · Data"),
            ("titulo_conteudo", "Agenda", "Contexto\nProposta\nInvestimento\nPróximos passos"),
            ("secao", "Contexto", ""),
            ("titulo_conteudo", "Onde estamos", "Situação atual em uma frase\nO que mudou no mercado\nO que está em jogo"),
            ("dois_conteudos", "Antes e depois", "Hoje\nProcessos manuais\nRetrabalho frequente\n\nCom a proposta\nProcessos integrados\nDecisão com dados"),
            ("titulo_conteudo", "Próximos passos", "Aprovação da proposta\nReunião de início\nPrimeira entrega em 30 dias"),
            ("titulo", "Obrigado", "contato@empresa.com"),
        ],
        "aula" => vec![
            ("titulo", "Título da aula", "Disciplina · Professor(a)"),
            ("titulo_conteudo", "Objetivos", "Entender o conceito central\nReconhecer exemplos do dia a dia\nAplicar em um exercício"),
            ("secao", "Conteúdo", ""),
            ("titulo_conteudo", "Conceito", "Definição em uma frase\nPor que isso importa\nO erro mais comum"),
            ("dois_conteudos", "Exemplo", "O que fazer\nPasso 1\nPasso 2\nPasso 3\n\nO que evitar\nAtalhos\nSuposições sem teste"),
            ("titulo_conteudo", "Exercício", "Enunciado\nTempo: 10 minutos\nDiscussão em duplas"),
            ("titulo_conteudo", "Para casa", "Leitura indicada\nLista de exercícios\nDúvidas por e-mail"),
        ],
        "projeto" => vec![
            ("titulo", "Nome do projeto", "Equipe · Versão 1.0"),
            ("titulo_conteudo", "O problema", "Quem sofre com isso\nQuanto custa hoje\nPor que ainda não foi resolvido"),
            ("titulo_conteudo", "A solução", "Ideia central\nComo funciona\nDiferenciais"),
            ("dois_conteudos", "Escopo", "Entra\nFuncionalidade A\nFuncionalidade B\n\nFica de fora\nIntegração futura\nVersão para celular"),
            ("titulo_conteudo", "Cronograma", "Mês 1: descoberta\nMês 2: protótipo\nMês 3: testes e entrega"),
            ("titulo_conteudo", "Riscos", "Prazo apertado\nDependência de terceiros\nPlano de mitigação para cada um"),
            ("titulo", "Vamos começar?", "Próximo passo: reunião de início"),
        ],
        "relatorio" => vec![
            ("titulo", "Relatório", "Período · Autoria"),
            ("titulo_conteudo", "Resumo executivo", "Principal resultado\nPrincipal desafio\nRecomendação"),
            ("titulo_conteudo", "Resultados", "Meta × realizado\nO que explica a diferença\nIndicadores acompanhados"),
            ("somente_titulo", "Gráfico dos resultados", ""),
            ("titulo_conteudo", "Conclusões", "O que aprendemos\nO que muda no próximo período"),
        ],
        "palestra" => vec![
            ("titulo", "Título da palestra", "Seu nome"),
            ("secao", "Uma ideia só", ""),
            ("somente_titulo", "Uma frase que fica na cabeça", ""),
            ("secao", "Uma história", ""),
            ("titulo_conteudo", "O que levar daqui", "Primeiro\nSegundo\nTerceiro"),
            ("titulo", "Perguntas?", "Contato e redes"),
        ],
        "tecnologia" => vec![
            ("titulo", "Título do produto", "Versão 2.0 · Equipe de engenharia"),
            ("titulo_conteudo", "Visão geral", "O que o sistema faz\nPara quem é\nO que o diferencia"),
            ("secao", "Arquitetura", ""),
            ("dois_conteudos", "Antes e depois", "Antes\nDeploy manual\nLogs espalhados\n\nDepois\nDeploy contínuo\nObservabilidade central"),
            ("titulo_conteudo", "Métricas", "Latência p95\nDisponibilidade\nCusto por requisição"),
            ("titulo", "Obrigado", "github.com/equipe · contato@exemplo.com"),
        ],
        "elegante" => vec![
            ("titulo", "Título da apresentação", "Evento · Data"),
            ("titulo_conteudo", "Programa", "Abertura\nPrimeira parte\nIntervalo\nSegunda parte"),
            ("secao", "Primeira parte", ""),
            ("dois_conteudos", "Contraste", "Tradição\nO que foi construído\nO que permanece\n\nInovação\nO que muda\nO que vem a seguir"),
            ("titulo_conteudo", "Considerações finais", "O essencial em três frases\nO convite à ação"),
            ("titulo", "Agradecimentos", "Nome · contato"),
        ],
        "criativo" => vec![
            ("titulo", "Uma ideia grande", "Quem apresenta · Quando"),
            ("titulo_conteudo", "O que vamos explorar", "De onde vem a inspiração\nO que muda quando se experimenta\nO primeiro passo"),
            ("secao", "Inspiração", ""),
            ("dois_conteudos", "Antes × Depois", "Antes\nPlano rígido\nMedo de errar\n\nDepois\nTestes rápidos\nAprender fazendo"),
            ("titulo_conteudo", "Como começar", "Escolha uma coisa pequena\nFaça hoje\nMostre para alguém"),
            ("titulo", "Vamos criar juntos!", "Seu contato"),
        ],
        "minimalista" => vec![
            ("titulo", "Título", "Autoria · Data"),
            ("titulo_conteudo", "Resumo", "Uma ideia por slide\nPoucas palavras\nMuito espaço"),
            ("secao", "Parte um", ""),
            ("dois_conteudos", "Duas visões", "Uma\nPrimeiro ponto\nSegundo ponto\n\nOutra\nPrimeiro ponto\nSegundo ponto"),
            ("titulo_conteudo", "Conclusão", "O que fica"),
            ("titulo", "Obrigado", "contato"),
        ],
        "pitch" => vec![
            ("titulo", "Nome da startup", "Uma frase que explica tudo"),
            ("titulo_conteudo", "O problema", "Quem sofre\nQuanto custa\nPor que ninguém resolveu"),
            ("titulo_conteudo", "A solução", "Como funciona em uma frase\nO momento \"uau\"\nPor que agora"),
            ("secao", "Mercado", ""),
            ("dois_conteudos", "Modelo de negócio", "Como ganhamos\nAssinatura mensal\nTaxa por uso\n\nPara quem\nPequenas empresas\nEquipes remotas"),
            ("titulo_conteudo", "Tração", "Usuários ativos\nCrescimento mensal\nPrimeiros clientes pagantes"),
            ("titulo_conteudo", "O pedido", "Quanto buscamos\nPara quê\nMarcos dos próximos 18 meses"),
        ],
        _ => vec![("titulo", "", "")],
    }
}

/// Mistura duas cores `#rrggbb`: `t` = 0 dá `a`, 1 dá `b`.
fn mistura(a: &str, b: &str, t: f32) -> String {
    let c = |h: &str, i: usize| u8::from_str_radix(h.get(1 + i * 2..3 + i * 2).unwrap_or("00"), 16).unwrap_or(0) as f32;
    let m = |i: usize| (c(a, i) + (c(b, i) - c(a, i)) * t).round() as u8;
    format!("#{:02x}{:02x}{:02x}", m(0), m(1), m(2))
}

/// Uma forma de enfeite, atrás de tudo e fora da seleção (`Papel::Fundo`).
fn forma(ap: &mut Apresentacao, tipo: Tipo, [x, y, largura, altura]: [f32; 4], cor: impl AsRef<str>) -> Elemento {
    Elemento { id: ap.novo_id(), tipo, papel: Papel::Fundo, x, y, largura, altura, preenchimento: cor.as_ref().into(), contorno: String::new(), espessura: 0.0, ..Elemento::default() }
}

/// Uma forma só com contorno (moldura, círculo vazado).
fn contorno(ap: &mut Apresentacao, tipo: Tipo, [x, y, largura, altura]: [f32; 4], cor: impl AsRef<str>, espessura: f32) -> Elemento {
    Elemento { id: ap.novo_id(), tipo, papel: Papel::Fundo, x, y, largura, altura, preenchimento: String::new(), contorno: cor.as_ref().into(), espessura, ..Elemento::default() }
}

/// Muda posição, alinhamento e cor de uma caixa de texto do slide (a `n`-ésima do papel).
fn caixa(slide: &mut Slide, papel: Papel, n: usize, rect: [f32; 4], alin: Alinhamento, cor: Option<&str>, tamanho: Option<f32>) {
    if let Some(e) = slide.elementos.iter_mut().filter(|e| e.papel == papel).nth(n) {
        [e.x, e.y, e.largura, e.altura] = rect;
        e.estilo.alinhamento = alin;
        if let Some(c) = cor {
            e.estilo.cor = c.into();
        }
        if let Some(t) = tamanho {
            e.estilo.tamanho = t;
        }
    }
}

/// O visual de cada modelo: formas de enfeite e a posição dos textos, por layout.
pub(crate) fn decorar(ap: &mut Apresentacao, slide: &mut Slide, modelo: &str, layout: &str) {
    if modelo == "em_branco" || modelo.is_empty() {
        return;
    }
    slide.transicao.tipo = TipoTransicao::Esmaecer;
    slide.transicao.direcao = Direcao::Esquerda;
    use Alinhamento::{Centro, Esquerda};
    use Papel::{Corpo, Subtitulo, Titulo};
    let tema = temas::tema(&ap.tema);
    let acento = if ap.tema.is_empty() { "#1a73e8" } else { tema.destaque };
    let fundo = if ap.tema.is_empty() { "#ffffff" } else { tema.fundo };
    let brilho = |h: &str| (1..7).step_by(2).map(|i| u8::from_str_radix(h.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0) as u32).sum::<u32>();
    let escuro = brilho(fundo) < 3 * 128;
    let suave = mistura(fundo, acento, if escuro { 0.22 } else { 0.10 });
    let forte = mistura(acento, "#000000", 0.28);
    let tinta = tema.titulo;
    let (l, a) = (960.0_f32, 540.0_f32);
    let mut enfeites: Vec<Elemento> = Vec::new();
    let branco = Some("#ffffff");
    match (modelo, layout) {
        // Negócios (executivo): faixas inclinadas em azul-marinho com um fio dourado.
        ("negocios", camada) => {
            let ouro = acento;
            let marinho = tinta;
            let faixa = |ap: &mut Apresentacao, r: [f32; 4], cor: &str, g: f32| {
                let mut e = forma(ap, Tipo::Retangulo, r, cor);
                e.rotacao = g;
                e
            };
            match camada {
                "titulo" => {
                    enfeites.push(faixa(ap, [l * 0.58, -120.0, l * 0.5, a * 1.5], marinho, 14.0));
                    enfeites.push(faixa(ap, [l * 0.58 - 14.0, -120.0, 5.0, a * 1.5], ouro, 14.0));
                    enfeites.push(faixa(ap, [l * 0.70, -120.0, 2.0, a * 1.5], &mistura(marinho, "#ffffff", 0.35), 14.0));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.07, a * 0.62, 64.0, 4.0], ouro));
                    caixa(slide, Titulo, 0, [l * 0.07, a * 0.2, l * 0.5, a * 0.38], Esquerda, None, Some(46.0));
                    caixa(slide, Subtitulo, 0, [l * 0.07, a * 0.66, l * 0.46, a * 0.1], Esquerda, None, Some(18.0));
                }
                "secao" => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 0.0, l, a], marinho));
                    enfeites.push(faixa(ap, [l * 0.66, -120.0, 120.0, a * 1.5], &mistura(marinho, "#ffffff", 0.07), 14.0));
                    enfeites.push(faixa(ap, [l * 0.66 - 12.0, -120.0, 4.0, a * 1.5], ouro, 14.0));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.08, a * 0.6, 60.0, 4.0], ouro));
                    caixa(slide, Titulo, 0, [l * 0.08, a * 0.34, l * 0.58, a * 0.24], Esquerda, branco, Some(52.0));
                }
                _ => {
                    let mut losango = forma(ap, Tipo::Retangulo, [l * 0.06, a * 0.085, 12.0, 12.0], ouro);
                    losango.rotacao = 45.0;
                    enfeites.push(losango);
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a * 0.215, l * 0.88, 1.0], marinho));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a - 46.0, l * 0.88, 1.0], mistura(fundo, marinho, 0.25)));
                    enfeites.push(faixa(ap, [l * 0.94 - 40.0, a - 64.0, 6.0, 40.0], ouro, 14.0));
                    enfeites.push(faixa(ap, [l * 0.94 - 24.0, a - 64.0, 6.0, 40.0], marinho, 14.0));
                    caixa(slide, Titulo, 0, [l * 0.06 + 26.0, a * 0.06, l * 0.8, a * 0.14], Esquerda, None, Some(30.0));
                    if camada == "dois_conteudos" {
                        for (k, x) in [l * 0.06, l * 0.52].into_iter().enumerate() {
                            enfeites.push(forma(ap, Tipo::Retangulo, [x, a * 0.28, 3.0, a * 0.56], ouro));
                            caixa(slide, Corpo, k, [x + 20.0, a * 0.28, l * 0.4, a * 0.56], Esquerda, None, Some(20.0));
                        }
                    } else {
                        caixa(slide, Corpo, 0, [l * 0.06 + 26.0, a * 0.28, l * 0.78, a * 0.58], Esquerda, None, Some(22.0));
                    }
                }
            }
        }
        // Aula: círculos grandes, faixa lateral e "folha" de conteúdo.
        ("aula", "titulo") => {
            enfeites.push(forma(ap, Tipo::Elipse, [l - 380.0, -140.0, 520.0, 520.0], acento));
            enfeites.push(forma(ap, Tipo::Elipse, [l - 250.0, a - 220.0, 330.0, 330.0], &suave));
            enfeites.push(forma(ap, Tipo::Retangulo, [0.0, a - 18.0, l, 18.0], acento));
            caixa(slide, Titulo, 0, [l * 0.07, a * 0.26, l * 0.58, a * 0.28], Esquerda, None, Some(46.0));
            caixa(slide, Subtitulo, 0, [l * 0.07, a * 0.58, l * 0.5, a * 0.12], Esquerda, None, None);
        }
        ("aula", "secao") => {
            enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 0.0, l, a], acento));
            enfeites.push(forma(ap, Tipo::Elipse, [-100.0, a - 260.0, 380.0, 380.0], &forte));
            enfeites.push(forma(ap, Tipo::Elipse, [l - 260.0, 40.0, 200.0, 200.0], &forte));
            caixa(slide, Titulo, 0, [l * 0.1, a * 0.38, l * 0.8, a * 0.2], Centro, branco, Some(50.0));
        }
        ("aula", _) => {
            enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 0.0, 26.0, a], acento));
            enfeites.push(forma(ap, Tipo::Elipse, [l - 92.0, 22.0, 56.0, 56.0], &suave));
            caixa(slide, Titulo, 0, [l * 0.08, a * 0.06, l * 0.78, a * 0.15], Esquerda, None, None);
            if layout == "dois_conteudos" {
                for (k, x) in [l * 0.08, l * 0.53].into_iter().enumerate() {
                    enfeites.push(forma(ap, Tipo::RetanguloArredondado, [x, a * 0.27, l * 0.4, a * 0.6], &suave));
                    caixa(slide, Corpo, k, [x + 18.0, a * 0.3, l * 0.4 - 34.0, a * 0.54], Esquerda, None, Some(20.0));
                }
            } else {
                caixa(slide, Corpo, 0, [l * 0.08, a * 0.27, l * 0.84, a * 0.6], Esquerda, None, None);
            }
        }
        // Projeto (rota): linhas de metrô com estações; cada etapa é uma parada.
        ("projeto", camada) => {
            let (verde, azul, ambar) = (acento, "#2563eb", "#f59e0b");
            let estacao = |ap: &mut Apresentacao, cx: f32, cy: f32, cor: &str, cheia: bool| -> Vec<Elemento> {
                let mut v = vec![forma(ap, Tipo::Elipse, [cx - 11.0, cy - 11.0, 22.0, 22.0], cor)];
                v.push(forma(ap, Tipo::Elipse, [cx - 6.0, cy - 6.0, 12.0, 12.0], if cheia { cor } else { "#ffffff" }));
                v
            };
            match camada {
                "titulo" => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [0.0, a * 0.7 - 5.0, l * 0.55, 10.0], verde));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.55 - 5.0, a * 0.52, 10.0, a * 0.18 + 5.0], verde));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.55 - 5.0, a * 0.52 - 5.0, l * 0.45 + 5.0, 10.0], verde));
                    enfeites.push(forma(ap, Tipo::Retangulo, [0.0, a * 0.8 - 5.0, l * 0.8, 10.0], azul));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.8 - 5.0, a * 0.8 - 5.0, 10.0, a * 0.2 + 5.0], azul));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.28, a * 0.7, 10.0, a * 0.1], ambar));
                    for (x, y, c, cheia) in [(l * 0.12, a * 0.7, verde, true), (l * 0.34, a * 0.7, verde, false), (l * 0.55, a * 0.61, verde, false), (l * 0.82, a * 0.52, verde, true), (l * 0.2, a * 0.8, azul, false), (l * 0.6, a * 0.8, azul, true), (l * 0.28 + 5.0, a * 0.75, ambar, true)] {
                        enfeites.extend(estacao(ap, x, y, c, cheia));
                    }
                    caixa(slide, Titulo, 0, [l * 0.07, a * 0.12, l * 0.62, a * 0.28], Esquerda, None, Some(46.0));
                    caixa(slide, Subtitulo, 0, [l * 0.07, a * 0.42, l * 0.55, a * 0.08], Esquerda, None, Some(18.0));
                }
                "secao" => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 0.0, l, a], verde));
                    for k in 0..3 {
                        enfeites.push(forma(ap, Tipo::Retangulo, [0.0, a * 0.74 + k as f32 * 22.0, l, 8.0], mistura(verde, "#ffffff", 0.35 - 0.1 * k as f32)));
                    }
                    for x in [0.15, 0.4, 0.65, 0.9] {
                        enfeites.extend(estacao(ap, l * x, a * 0.74 + 4.0, "#ffffff", true));
                    }
                    caixa(slide, Titulo, 0, [l * 0.07, a * 0.34, l * 0.7, a * 0.24], Esquerda, branco, Some(54.0));
                }
                _ => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [0.0, a * 0.1 - 4.0, l * 0.06 + 14.0, 8.0], verde));
                    enfeites.extend(estacao(ap, l * 0.06 + 18.0, a * 0.1, verde, true));
                    for k in 0..5 {
                        if k > 0 {
                            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.94 - 140.0 + (k - 1) as f32 * 28.0 + 8.0, a - 33.0, 20.0, 4.0], if k <= 2 { verde.to_string() } else { "#c9d3e0".to_string() }));
                        }
                        enfeites.push(forma(ap, Tipo::Elipse, [l * 0.94 - 140.0 + k as f32 * 28.0, a - 38.0, 14.0, 14.0], if k < 3 { verde.to_string() } else { "#c9d3e0".to_string() }));
                    }
                    caixa(slide, Titulo, 0, [l * 0.06 + 40.0, a * 0.06, l * 0.8, a * 0.14], Esquerda, None, Some(32.0));
                    let cores = [verde, azul, ambar];
                    let cartao_branco = |ap: &mut Apresentacao, r: [f32; 4]| -> Vec<Elemento> { vec![forma(ap, Tipo::Retangulo, r, "#ffffff"), contorno(ap, Tipo::Retangulo, r, "#d5dde8", 1.0)] };
                    match camada {
                        "comparacao" | "tres_colunas" => {
                            let n = if camada == "comparacao" { 2 } else { 3 };
                            let (x0, larg) = (l * 0.06, if n == 2 { l * 0.43 } else { l * 0.28 });
                            let passo = if n == 2 { l * 0.45 } else { l * 0.30 };
                            for k in 0..n {
                                let x = x0 + k as f32 * passo;
                                enfeites.extend(cartao_branco(ap, [x, a * 0.27, larg, a * 0.66]));
                                enfeites.push(forma(ap, Tipo::Retangulo, [x, a * 0.245 - 4.0, larg, 8.0], cores[k]));
                                enfeites.extend(estacao(ap, x + 12.0, a * 0.245, cores[k], k == 0));
                            }
                        }
                        "quatro_quadrantes" => {
                            for (k, (cx, cy)) in [(0.06, 0.26), (0.51, 0.26), (0.06, 0.61), (0.51, 0.61)].into_iter().enumerate() {
                                let (x, y) = (l * cx, a * cy);
                                enfeites.extend(cartao_branco(ap, [x, y, l * 0.43, a * 0.31]));
                                enfeites.push(forma(ap, Tipo::Retangulo, [x - 4.0, y, 8.0, a * 0.31], cores[k % 3]));
                                enfeites.extend(estacao(ap, x, y + 4.0, cores[k % 3], true));
                            }
                        }
                        "numeros" => {
                            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a * 0.27 - 4.0, l * 0.88, 8.0], verde));
                            for (k, cx) in [0.19, 0.5, 0.81].into_iter().enumerate() {
                                enfeites.extend(estacao(ap, l * cx, a * 0.27, cores[k], k != 1));
                            }
                        }
                        "linha_do_tempo" => {
                            let y = a * 0.255;
                            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, y - 5.0, l * 0.88, 10.0], verde));
                            for (k, x) in [0.06, 0.285, 0.51, 0.735].into_iter().enumerate() {
                                enfeites.extend(estacao(ap, l * x + l * 0.1, y, if k < 2 { verde } else { azul }, k < 2));
                            }
                            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.94 - 10.0, y - 14.0, 10.0, 28.0], ambar));
                        }
                        "citacao" => {
                            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.09 - 4.0, a * 0.2, 8.0, a * 0.5], verde));
                            enfeites.extend(estacao(ap, l * 0.09, a * 0.2, verde, true));
                            enfeites.extend(estacao(ap, l * 0.09, a * 0.7, verde, false));
                            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.09, a * 0.7 - 4.0, l * 0.12, 8.0], azul));
                        }
                        "titulo_lateral" => {
                            let x = l * 0.41;
                            enfeites.push(forma(ap, Tipo::Retangulo, [x - 4.0, a * 0.12, 8.0, a * 0.76], verde));
                            for (k, y) in [0.12, 0.5, 0.88].into_iter().enumerate() {
                                enfeites.extend(estacao(ap, x, a * y, if k == 1 { azul } else { verde }, k != 1));
                            }
                        }
                        _ => {}
                    }
                    if camada == "dois_conteudos" {
                        for (k, (x, c)) in [(l * 0.06, verde), (l * 0.51, azul)].into_iter().enumerate() {
                            enfeites.push(forma(ap, Tipo::Retangulo, [x, a * 0.27, l * 0.43, a * 0.58], "#ffffff"));
                            enfeites.push(contorno(ap, Tipo::Retangulo, [x, a * 0.27, l * 0.43, a * 0.58], "#d5dde8", 1.0));
                            enfeites.push(forma(ap, Tipo::Retangulo, [x, a * 0.27, l * 0.43, 8.0], c));
                            caixa(slide, Corpo, k, [x + 22.0, a * 0.27 + 24.0, l * 0.43 - 44.0, a * 0.5], Esquerda, None, Some(20.0));
                        }
                    } else if matches!(camada, "titulo_conteudo" | "somente_conteudo") {
                        caixa(slide, Corpo, 0, [l * 0.06 + 40.0, a * 0.27, l * 0.78, a * 0.58], Esquerda, None, Some(22.0));
                    }
                }
            }
        }
        // Relatório: fios finos, sóbrio.
        ("relatorio", "titulo") => {
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.08, a * 0.2, l * 0.84, 3.0], acento));
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.08, a * 0.74, l * 0.84, 1.0], acento));
            caixa(slide, Titulo, 0, [l * 0.08, a * 0.26, l * 0.84, a * 0.26], Esquerda, None, Some(48.0));
            caixa(slide, Subtitulo, 0, [l * 0.08, a * 0.58, l * 0.84, a * 0.12], Esquerda, None, None);
        }
        ("relatorio", _) => {
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a * 0.215, l * 0.88, 2.0], acento));
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a - 44.0, l * 0.88, 1.0], acento));
            caixa(slide, Titulo, 0, [l * 0.06, a * 0.06, l * 0.88, a * 0.14], Esquerda, None, None);
        }
        // Palestra (conferência): tipografia enorme e a faixa de legenda azul-cobalto, como num vídeo de palestra.
        ("palestra", camada) => {
            let cobalto = acento;
            match camada {
                "titulo" => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [0.0, a * 0.7, l * 0.72, 78.0], cobalto));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.72, a * 0.7, 10.0, 78.0], tinta));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a * 0.7 + 22.0, 8.0, 34.0], "#ffffff"));
                    enfeites.push(forma(ap, Tipo::Elipse, [l - 82.0, 34.0, 18.0, 18.0], "#e5322d"));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a * 0.1, 56.0, 6.0], tinta));
                    caixa(slide, Titulo, 0, [l * 0.06, a * 0.17, l * 0.86, a * 0.46], Esquerda, None, Some(78.0));
                    caixa(slide, Subtitulo, 0, [l * 0.06 + 26.0, a * 0.7 + 14.0, l * 0.62, 50.0], Esquerda, branco, Some(20.0));
                }
                "secao" => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 0.0, l, a], cobalto));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.07, a * 0.64, l * 0.3, 10.0], "#ffffff"));
                    enfeites.push(contorno(ap, Tipo::Elipse, [l * 0.62, -140.0, 560.0, 560.0], mistura(cobalto, "#ffffff", 0.35), 2.0));
                    caixa(slide, Titulo, 0, [l * 0.07, a * 0.22, l * 0.78, a * 0.4], Esquerda, branco, Some(84.0));
                }
                _ => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 0.0, 10.0, a], cobalto));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.07, a * 0.2, l * 0.86, 2.0], tinta));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.07, a * 0.2 - 3.0, 56.0, 8.0], cobalto));
                    caixa(slide, Titulo, 0, [l * 0.07, a * 0.06, l * 0.86, a * 0.13], Esquerda, None, Some(42.0));
                    if camada == "dois_conteudos" {
                        for (k, x) in [l * 0.07, l * 0.52].into_iter().enumerate() {
                            enfeites.push(forma(ap, Tipo::Retangulo, [x, a * 0.3, l * 0.41, 2.0], tinta));
                            enfeites.push(forma(ap, Tipo::Retangulo, [x, a * 0.3 - 3.0, 14.0, 8.0], cobalto));
                            caixa(slide, Corpo, k, [x, a * 0.33, l * 0.41, a * 0.54], Esquerda, None, Some(22.0));
                        }
                    } else if matches!(camada, "titulo_conteudo" | "somente_conteudo") {
                        caixa(slide, Corpo, 0, [l * 0.07, a * 0.27, l * 0.8, a * 0.6], Esquerda, None, Some(26.0));
                    }
                }
            }
        }
        // Tecnologia: noturno, linhas finas, triângulos e círculos vazados em ciano e magenta.
        ("tecnologia", "titulo") => {
            let magenta = "#e879f9";
            enfeites.push(contorno(ap, Tipo::Triangulo, [l * 0.58, -60.0, 520.0, 640.0], acento, 1.5));
            enfeites.push(contorno(ap, Tipo::Triangulo, [l * 0.68, 40.0, 340.0, 440.0], magenta, 1.5));
            enfeites.push(forma(ap, Tipo::Elipse, [l * 0.80, a * 0.62, 14.0, 14.0], acento));
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a * 0.2, 56.0, 4.0], acento));
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06 + 60.0, a * 0.2, 14.0, 4.0], magenta));
            caixa(slide, Titulo, 0, [l * 0.06, a * 0.26, l * 0.55, a * 0.28], Esquerda, None, Some(46.0));
            caixa(slide, Subtitulo, 0, [l * 0.06, a * 0.58, l * 0.5, a * 0.1], Esquerda, None, None);
        }
        ("tecnologia", "secao") => {
            enfeites.push(contorno(ap, Tipo::Elipse, [l * 0.52, -90.0, 560.0, 560.0], acento, 1.5));
            enfeites.push(contorno(ap, Tipo::Elipse, [l * 0.62, 20.0, 340.0, 340.0], "#e879f9", 1.5));
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.08, a * 0.62, 90.0, 4.0], acento));
            caixa(slide, Titulo, 0, [l * 0.08, a * 0.38, l * 0.6, a * 0.2], Esquerda, None, Some(52.0));
        }
        ("tecnologia", _) => {
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a * 0.075, 12.0, 12.0], acento));
            enfeites.push(forma(ap, Tipo::Retangulo, [0.0, a - 6.0, l * 0.4, 6.0], acento));
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.4, a - 6.0, l * 0.1, 6.0], "#e879f9"));
            enfeites.push(contorno(ap, Tipo::Triangulo, [l - 150.0, 30.0, 110.0, 96.0], acento, 1.2));
            caixa(slide, Titulo, 0, [l * 0.06 + 26.0, a * 0.06, l * 0.7, a * 0.15], Esquerda, None, None);
            if layout == "dois_conteudos" {
                for (k, x) in [l * 0.06, l * 0.51].into_iter().enumerate() {
                    enfeites.push(forma(ap, Tipo::RetanguloArredondado, [x, a * 0.27, l * 0.43, a * 0.6], &suave));
                    enfeites.push(forma(ap, Tipo::Retangulo, [x, a * 0.27, l * 0.43, 4.0], if k == 0 { acento } else { "#e879f9" }));
                    caixa(slide, Corpo, k, [x + 20.0, a * 0.3, l * 0.43 - 38.0, a * 0.54], Esquerda, None, Some(20.0));
                }
            }
        }
        // Elegante (art déco): verde-escuro, dourado, cantos em degraus e eixos de simetria.
        ("elegante", camada) => {
            let ouro = acento;
            let degraus = |ap: &mut Apresentacao, x: f32, y: f32, sx: f32, sy: f32| -> Vec<Elemento> {
                // três degraus em cada canto, como um leque de linhas retas
                let mut v = Vec::new();
                for k in 0..3 {
                    let t = 14.0 + k as f32 * 12.0;
                    v.push(forma(ap, Tipo::Retangulo, [if sx > 0.0 { x } else { x - t }, y, t, 2.0], ouro));
                    v.push(forma(ap, Tipo::Retangulo, [x - if sx > 0.0 { 0.0 } else { 2.0 }, if sy > 0.0 { y } else { y - t }, 2.0, t], ouro));
                }
                v
            };
            let m = 30.0;
            enfeites.push(contorno(ap, Tipo::Retangulo, [m, m, l - 2.0 * m, a - 2.0 * m], ouro, 1.4));
            enfeites.push(contorno(ap, Tipo::Retangulo, [m + 10.0, m + 10.0, l - 2.0 * m - 20.0, a - 2.0 * m - 20.0], mistura(fundo, ouro, 0.45), 0.8));
            for (x, y, sx, sy) in [(m + 10.0, m + 10.0, 1.0, 1.0), (l - m - 10.0, m + 10.0, -1.0, 1.0), (m + 10.0, a - m - 10.0, 1.0, -1.0), (l - m - 10.0, a - m - 10.0, -1.0, -1.0)] {
                enfeites.extend(degraus(ap, x, y, sx, sy));
            }
            let losango = |ap: &mut Apresentacao, cx: f32, cy: f32, d: f32, cheio: bool| {
                let mut e = if cheio { forma(ap, Tipo::Retangulo, [cx - d / 2.0, cy - d / 2.0, d, d], ouro) } else { contorno(ap, Tipo::Retangulo, [cx - d / 2.0, cy - d / 2.0, d, d], ouro, 1.2) };
                e.rotacao = 45.0;
                e
            };
            match camada {
                "titulo" => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [l / 2.0 - 0.75, a * 0.12, 1.5, a * 0.13], ouro));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l / 2.0 - 0.75, a * 0.75, 1.5, a * 0.13], ouro));
                    enfeites.push(losango(ap, l / 2.0, a * 0.62, 14.0, true));
                    enfeites.push(losango(ap, l / 2.0 - 40.0, a * 0.62, 8.0, false));
                    enfeites.push(losango(ap, l / 2.0 + 40.0, a * 0.62, 8.0, false));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l / 2.0 - 140.0, a * 0.62 - 0.75, 90.0, 1.5], ouro));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l / 2.0 + 50.0, a * 0.62 - 0.75, 90.0, 1.5], ouro));
                    caixa(slide, Titulo, 0, [l * 0.14, a * 0.3, l * 0.72, a * 0.26], Centro, None, Some(46.0));
                    caixa(slide, Subtitulo, 0, [l * 0.14, a * 0.68, l * 0.72, a * 0.1], Centro, None, None);
                }
                "secao" => {
                    enfeites.push(losango(ap, l / 2.0, a * 0.5, 280.0, false));
                    enfeites.push(losango(ap, l / 2.0, a * 0.5, 230.0, false));
                    caixa(slide, Titulo, 0, [l * 0.2, a * 0.4, l * 0.6, a * 0.2], Centro, None, Some(44.0));
                }
                _ => {
                    enfeites.push(losango(ap, l / 2.0, a * 0.225, 10.0, true));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l / 2.0 - 120.0, a * 0.225 - 0.75, 100.0, 1.5], ouro));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l / 2.0 + 20.0, a * 0.225 - 0.75, 100.0, 1.5], ouro));
                    caixa(slide, Titulo, 0, [l * 0.12, a * 0.07, l * 0.76, a * 0.13], Centro, None, Some(32.0));
                    if camada == "dois_conteudos" {
                        enfeites.push(forma(ap, Tipo::Retangulo, [l / 2.0 - 0.75, a * 0.3, 1.5, a * 0.52], ouro));
                        caixa(slide, Corpo, 0, [l * 0.12, a * 0.29, l * 0.36, a * 0.56], Esquerda, None, Some(20.0));
                        caixa(slide, Corpo, 1, [l * 0.52, a * 0.29, l * 0.36, a * 0.56], Esquerda, None, Some(20.0));
                    } else {
                        caixa(slide, Corpo, 0, [l * 0.16, a * 0.29, l * 0.68, a * 0.56], Esquerda, None, None);
                    }
                }
            }
        }
        // Criativo: círculos e faixas em quatro cores quentes.
        ("criativo", "titulo") => {
            enfeites.push(forma(ap, Tipo::Elipse, [l * 0.62, -60.0, 360.0, 360.0], "#f97316"));
            enfeites.push(forma(ap, Tipo::Elipse, [l * 0.80, 150.0, 220.0, 220.0], "#ec4899"));
            enfeites.push(forma(ap, Tipo::Elipse, [l * 0.58, 260.0, 150.0, 150.0], "#14b8a6"));
            enfeites.push(forma(ap, Tipo::Elipse, [l * 0.74, 400.0, 90.0, 90.0], "#facc15"));
            for (k, c) in ["#f97316", "#ec4899", "#14b8a6", "#facc15"].into_iter().enumerate() {
                enfeites.push(forma(ap, Tipo::Retangulo, [k as f32 * l / 4.0, a - 16.0, l / 4.0, 16.0], c));
            }
            caixa(slide, Titulo, 0, [l * 0.07, a * 0.26, l * 0.52, a * 0.3], Esquerda, None, Some(52.0));
            caixa(slide, Subtitulo, 0, [l * 0.07, a * 0.6, l * 0.5, a * 0.1], Esquerda, None, None);
        }
        ("criativo", "secao") => {
            enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 0.0, l, a], "#f97316"));
            enfeites.push(forma(ap, Tipo::Elipse, [l - 300.0, -80.0, 380.0, 380.0], "#ec4899"));
            enfeites.push(forma(ap, Tipo::Elipse, [-60.0, a - 200.0, 260.0, 260.0], "#14b8a6"));
            enfeites.push(forma(ap, Tipo::Triangulo, [l - 220.0, a - 190.0, 150.0, 130.0], "#facc15"));
            caixa(slide, Titulo, 0, [l * 0.1, a * 0.38, l * 0.8, a * 0.2], Centro, branco, Some(56.0));
        }
        ("criativo", _) => {
            for (k, c) in ["#f97316", "#ec4899", "#14b8a6", "#facc15"].into_iter().enumerate() {
                enfeites.push(forma(ap, Tipo::Retangulo, [k as f32 * l / 4.0, 0.0, l / 4.0, 12.0], c));
            }
            enfeites.push(forma(ap, Tipo::Elipse, [l - 96.0, 26.0, 46.0, 46.0], "#ec4899"));
            enfeites.push(forma(ap, Tipo::Elipse, [l - 62.0, 54.0, 26.0, 26.0], "#14b8a6"));
            caixa(slide, Titulo, 0, [l * 0.06, a * 0.07, l * 0.75, a * 0.15], Esquerda, None, None);
            if layout == "dois_conteudos" {
                for (k, (x, c)) in [(l * 0.06, "#f97316"), (l * 0.51, "#14b8a6")].into_iter().enumerate() {
                    enfeites.push(forma(ap, Tipo::RetanguloArredondado, [x, a * 0.28, l * 0.43, a * 0.58], mistura(fundo, c, 0.12)));
                    enfeites.push(forma(ap, Tipo::RetanguloArredondado, [x, a * 0.28, l * 0.43, 10.0], c));
                    caixa(slide, Corpo, k, [x + 20.0, a * 0.32, l * 0.43 - 38.0, a * 0.5], Esquerda, None, Some(20.0));
                }
            }
        }
        // Minimalista: quase nada; um quadradinho de cor e fios cinza.
        ("minimalista", "titulo") => {
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.08, a * 0.56, 28.0, 28.0], "#1a73e8"));
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.08, a - 60.0, l * 0.84, 1.0], "#c7c7c2"));
            caixa(slide, Titulo, 0, [l * 0.08, a * 0.2, l * 0.8, a * 0.3], Esquerda, None, Some(60.0));
            caixa(slide, Subtitulo, 0, [l * 0.08 + 44.0, a * 0.56, l * 0.6, a * 0.06], Esquerda, None, Some(18.0));
        }
        ("minimalista", "secao") => {
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.08, a * 0.3, 3.0, a * 0.4], "#1a73e8"));
            caixa(slide, Titulo, 0, [l * 0.08 + 28.0, a * 0.42, l * 0.7, a * 0.16], Esquerda, None, Some(48.0));
        }
        ("minimalista", _) => {
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.08, a * 0.085, 14.0, 14.0], "#1a73e8"));
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.08, a - 50.0, l * 0.84, 1.0], "#d6d6d1"));
            caixa(slide, Titulo, 0, [l * 0.08 + 26.0, a * 0.06, l * 0.8, a * 0.14], Esquerda, None, Some(30.0));
            if layout == "dois_conteudos" {
                enfeites.push(forma(ap, Tipo::Retangulo, [l / 2.0, a * 0.3, 1.0, a * 0.5], "#d6d6d1"));
                caixa(slide, Corpo, 0, [l * 0.08, a * 0.28, l * 0.38, a * 0.55], Esquerda, None, Some(20.0));
                caixa(slide, Corpo, 1, [l * 0.54, a * 0.28, l * 0.38, a * 0.55], Esquerda, None, Some(20.0));
            } else {
                caixa(slide, Corpo, 0, [l * 0.08, a * 0.28, l * 0.8, a * 0.55], Esquerda, None, Some(22.0));
            }
        }
        // Pitch: roxo profundo, triângulos grandes e cartões.
        ("pitch", "titulo") => {
            enfeites.push(forma(ap, Tipo::Triangulo, [l * 0.52, a * 0.1, l * 0.5, a * 0.95], "#4c1d95"));
            enfeites.push(forma(ap, Tipo::Triangulo, [l * 0.68, a * 0.38, l * 0.34, a * 0.66], acento));
            enfeites.push(forma(ap, Tipo::Elipse, [l * 0.80, a * 0.1, 70.0, 70.0], "#f0abfc"));
            caixa(slide, Titulo, 0, [l * 0.06, a * 0.28, l * 0.52, a * 0.26], Esquerda, None, Some(50.0));
            caixa(slide, Subtitulo, 0, [l * 0.06, a * 0.58, l * 0.46, a * 0.12], Esquerda, None, None);
        }
        ("pitch", "secao") => {
            enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 0.0, l, a], "#6d28d9"));
            enfeites.push(forma(ap, Tipo::Elipse, [l - 300.0, a - 300.0, 460.0, 460.0], "#4c1d95"));
            enfeites.push(forma(ap, Tipo::Elipse, [l - 200.0, a - 210.0, 120.0, 120.0], "#f0abfc"));
            caixa(slide, Titulo, 0, [l * 0.08, a * 0.38, l * 0.6, a * 0.2], Esquerda, branco, Some(54.0));
        }
        ("pitch", _) => {
            enfeites.push(forma(ap, Tipo::RetanguloArredondado, [l * 0.06, a * 0.065, 10.0, a * 0.13], acento));
            enfeites.push(forma(ap, Tipo::Triangulo, [l - 120.0, a - 110.0, 90.0, 80.0], "#4c1d95"));
            caixa(slide, Titulo, 0, [l * 0.06 + 26.0, a * 0.06, l * 0.8, a * 0.14], Esquerda, None, Some(36.0));
            if layout == "dois_conteudos" {
                for (k, x) in [l * 0.06, l * 0.51].into_iter().enumerate() {
                    enfeites.push(forma(ap, Tipo::RetanguloArredondado, [x, a * 0.27, l * 0.43, a * 0.6], mistura(fundo, "#ffffff", 0.08)));
                    enfeites.push(forma(ap, Tipo::Retangulo, [x + 20.0, a * 0.27 + 16.0, 48.0, 4.0], acento));
                    caixa(slide, Corpo, k, [x + 20.0, a * 0.27 + 28.0, l * 0.43 - 38.0, a * 0.5], Esquerda, None, Some(20.0));
                }
            } else if layout == "titulo_conteudo" {
                enfeites.push(forma(ap, Tipo::RetanguloArredondado, [l * 0.06, a * 0.27, l * 0.88, a * 0.6], mistura(fundo, "#ffffff", 0.08)));
                caixa(slide, Corpo, 0, [l * 0.06 + 30.0, a * 0.31, l * 0.88 - 60.0, a * 0.52], Esquerda, None, None);
            }
        }
        // ══ Modelos novos (07/10), ao lado dos anteriores ══════════════════════════════════
        // ── Negócios: a barra de resultados que sobe. Tudo o mais fica calado. ──────────────
        ("resultados", "titulo") => {
            let ambar = "#f2b134";
            for (k, h) in [120.0, 170.0, 230.0, 300.0, 390.0].into_iter().enumerate() {
                let cor = if k == 4 { acento.to_string() } else { mistura(fundo, acento, 0.22 + 0.17 * k as f32) };
                enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.64 + k as f32 * 58.0, a - 56.0 - h, 44.0, h], cor));
            }
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.58, a - 56.0, l * 0.36, 2.0], tinta));
            enfeites.push(forma(ap, Tipo::Elipse, [l * 0.07, a * 0.585, 14.0, 14.0], ambar));
            caixa(slide, Titulo, 0, [l * 0.07, a * 0.2, l * 0.5, a * 0.34], Esquerda, None, Some(46.0));
            caixa(slide, Subtitulo, 0, [l * 0.07 + 26.0, a * 0.57, l * 0.46, a * 0.1], Esquerda, None, Some(18.0));
        }
        ("resultados", "secao") => {
            enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 0.0, l, a], acento));
            for (k, h) in [90.0, 130.0, 180.0].into_iter().enumerate() {
                enfeites.push(contorno(ap, Tipo::Retangulo, [l - 230.0 + k as f32 * 66.0, a - 60.0 - h, 48.0, h], mistura(acento, "#ffffff", 0.5), 1.5));
            }
            caixa(slide, Titulo, 0, [l * 0.08, a * 0.36, l * 0.62, a * 0.22], Esquerda, branco, Some(54.0));
        }
        ("resultados", _) => {
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a * 0.215, l * 0.88, 1.0], tinta));
            for (k, h) in [10.0, 16.0, 24.0, 32.0].into_iter().enumerate() {
                enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.94 - 52.0 + k as f32 * 14.0, a * 0.2 - h, 9.0, h], mistura(fundo, acento, 0.3 + 0.23 * k as f32)));
            }
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a - 44.0, l * 0.88, 1.0], mistura(fundo, tinta, 0.2)));
            caixa(slide, Titulo, 0, [l * 0.06, a * 0.06, l * 0.76, a * 0.14], Esquerda, None, Some(30.0));
            if layout == "dois_conteudos" {
                enfeites.push(forma(ap, Tipo::Retangulo, [l / 2.0 - 0.5, a * 0.3, 1.0, a * 0.55], mistura(fundo, tinta, 0.2)));
                for (k, x) in [l * 0.06, l * 0.54].into_iter().enumerate() {
                    enfeites.push(forma(ap, Tipo::Retangulo, [x, a * 0.28, 34.0, 4.0], acento));
                    caixa(slide, Corpo, k, [x, a * 0.32, l * 0.4, a * 0.52], Esquerda, None, Some(20.0));
                }
            } else {
                caixa(slide, Corpo, 0, [l * 0.06, a * 0.28, l * 0.7, a * 0.58], Esquerda, None, Some(22.0));
            }
        }
        // ── Aula: caderno pautado, margem vermelha e marca-texto atrás do título. ───────────
        ("caderno", camada) => {
            let pauta = "#d6e6f2";
            for k in 0..15 {
                enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 54.0 + k as f32 * 33.0, l, 1.0], pauta));
            }
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.115, 0.0, 2.0, a], acento));
            for k in 0..3 {
                enfeites.push(forma(ap, Tipo::Elipse, [26.0, a * 0.2 + k as f32 * a * 0.3, 20.0, 20.0], "#e9e4d4"));
            }
            let marca = "#ffe66d";
            match camada {
                "titulo" => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.15 - 8.0, a * 0.47 - 46.0, l * 0.56, 46.0], marca));
                    caixa(slide, Titulo, 0, [l * 0.15, a * 0.22, l * 0.74, a * 0.25], Esquerda, None, Some(48.0));
                    caixa(slide, Subtitulo, 0, [l * 0.15, a * 0.52, l * 0.7, a * 0.1], Esquerda, None, Some(20.0));
                }
                "secao" => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.15 - 8.0, a * 0.54 - 56.0, l * 0.5, 56.0], marca));
                    caixa(slide, Titulo, 0, [l * 0.15, a * 0.3, l * 0.74, a * 0.24], Esquerda, None, Some(56.0));
                }
                _ => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.17 - 6.0, a * 0.2 - 18.0, l * 0.3, 16.0], marca));
                    caixa(slide, Titulo, 0, [l * 0.17, a * 0.06, l * 0.76, a * 0.14], Esquerda, None, Some(32.0));
                    if camada == "dois_conteudos" {
                        caixa(slide, Corpo, 0, [l * 0.17, a * 0.27, l * 0.34, a * 0.6], Esquerda, None, Some(20.0));
                        caixa(slide, Corpo, 1, [l * 0.57, a * 0.27, l * 0.34, a * 0.6], Esquerda, None, Some(20.0));
                    } else {
                        caixa(slide, Corpo, 0, [l * 0.17, a * 0.27, l * 0.74, a * 0.6], Esquerda, None, Some(22.0));
                    }
                }
            }
        }
        // ── Projeto: o cronograma em barras (gantt) é a ideia; laranja de canteiro de obra. ─
        ("canteiro", "titulo") => {
            let aco = "#6b7a89";
            for k in 0..14 {
                enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.07 + k as f32 * 60.0, a * 0.6, 1.0, if k % 2 == 0 { 14.0 } else { 8.0 }], aco));
            }
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.07, a * 0.6 + 14.0, l * 0.86, 1.0], aco));
            for (k, (x0, w)) in [(0.0, 240.0), (120.0, 300.0), (330.0, 220.0), (440.0, 280.0), (600.0, 200.0)].into_iter().enumerate() {
                enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.07 + x0, a * 0.6 + 34.0 + k as f32 * 20.0, w, 12.0], if k == 1 { acento.to_string() } else { mistura(fundo, tinta, 0.22) }));
            }
            caixa(slide, Titulo, 0, [l * 0.07, a * 0.12, l * 0.7, a * 0.3], Esquerda, None, Some(48.0));
            caixa(slide, Subtitulo, 0, [l * 0.07, a * 0.43, l * 0.6, a * 0.08], Esquerda, None, Some(18.0));
        }
        ("canteiro", "secao") => {
            enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 0.0, l, a], acento));
            for k in 0..3 {
                enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.07, a - 90.0 + k as f32 * 22.0, l * (0.86 - 0.18 * k as f32), 12.0], tinta));
            }
            caixa(slide, Titulo, 0, [l * 0.07, a * 0.3, l * 0.8, a * 0.24], Esquerda, Some(tinta), Some(56.0));
        }
        ("canteiro", _) => {
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a * 0.07, 8.0, a * 0.11], acento));
            enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.94 - 130.0, a - 38.0, 130.0, 1.0], "#6b7a89"));
            for k in 0..5 {
                enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.94 - 130.0 + k as f32 * 30.0, a - 44.0, 10.0, 10.0], if k == 0 { acento.to_string() } else { "#6b7a89".to_string() }));
            }
            caixa(slide, Titulo, 0, [l * 0.06 + 24.0, a * 0.06, l * 0.8, a * 0.14], Esquerda, None, Some(32.0));
            if layout == "dois_conteudos" {
                for (k, x) in [l * 0.06, l * 0.51].into_iter().enumerate() {
                    enfeites.push(forma(ap, Tipo::Retangulo, [x, a * 0.27, l * 0.43, a * 0.6], "#ffffff"));
                    enfeites.push(contorno(ap, Tipo::Retangulo, [x, a * 0.27, l * 0.43, a * 0.6], "#b9bec3", 1.0));
                    enfeites.push(forma(ap, Tipo::Retangulo, [x, a * 0.27, 70.0, 5.0], acento));
                    caixa(slide, Corpo, k, [x + 20.0, a * 0.27 + 20.0, l * 0.43 - 40.0, a * 0.52], Esquerda, None, Some(20.0));
                }
            } else {
                caixa(slide, Corpo, 0, [l * 0.06 + 24.0, a * 0.27, l * 0.8, a * 0.6], Esquerda, None, Some(22.0));
            }
        }
        // ── Relatório: memorando técnico; a coluna de margem separa o rótulo do texto. ──────
        ("memorando", camada) => {
            let (largura_margem, cor_margem) = if camada == "secao" { (l * 0.4, "#111827".to_string()) } else if camada == "titulo" { (l * 0.24, acento.to_string()) } else { (l * 0.2, "#e6e8ec".to_string()) };
            enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 0.0, largura_margem, a], cor_margem));
            enfeites.push(forma(ap, Tipo::Retangulo, [largura_margem, 0.0, 1.5, a], if camada == "secao" { acento.to_string() } else { "#9aa1ad".to_string() }));
            let x0 = largura_margem + 40.0;
            match camada {
                "titulo" => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [x0, a * 0.2, l - x0 - 50.0, 2.0], tinta));
                    enfeites.push(forma(ap, Tipo::Retangulo, [x0, a * 0.74, l - x0 - 50.0, 1.0], tinta));
                    caixa(slide, Titulo, 0, [x0, a * 0.26, l - x0 - 50.0, a * 0.26], Esquerda, None, Some(44.0));
                    caixa(slide, Subtitulo, 0, [x0, a * 0.58, l - x0 - 50.0, a * 0.12], Esquerda, None, Some(18.0));
                }
                "secao" => caixa(slide, Titulo, 0, [x0, a * 0.38, l - x0 - 50.0, a * 0.22], Esquerda, None, Some(44.0)),
                _ => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [x0, a * 0.215, l - x0 - 50.0, 1.0], "#9aa1ad"));
                    enfeites.push(forma(ap, Tipo::Retangulo, [largura_margem * 0.5 - 7.0, a * 0.08, 14.0, 14.0], acento));
                    caixa(slide, Titulo, 0, [x0, a * 0.06, l - x0 - 50.0, a * 0.14], Esquerda, None, Some(30.0));
                    if camada == "dois_conteudos" {
                        let col = (l - x0 - 50.0 - 30.0) / 2.0;
                        caixa(slide, Corpo, 0, [x0, a * 0.28, col, a * 0.58], Esquerda, None, Some(19.0));
                        caixa(slide, Corpo, 1, [x0 + col + 30.0, a * 0.28, col, a * 0.58], Esquerda, None, Some(19.0));
                    } else {
                        caixa(slide, Corpo, 0, [x0, a * 0.28, l - x0 - 120.0, a * 0.58], Esquerda, None, Some(21.0));
                    }
                }
            }
        }
        // ── Holofote: palco escuro, um feixe de luz que desce e a poça de luz no chão. ──────
        ("holofote", camada) => {
            let luz = mistura(fundo, "#ffffff", 0.08);
            let luz2 = mistura(fundo, "#ffffff", 0.13);
            let viga = |ap: &mut Apresentacao, cx: f32, topo: f32, base_l: f32, altura: f32, cor: &str| {
                let mut e = forma(ap, Tipo::Triangulo, [cx - base_l / 2.0, topo, base_l, altura], cor);
                e.rotacao = 180.0;
                e
            };
            match camada {
                "titulo" => {
                    enfeites.push(viga(ap, l * 0.74, -10.0, 520.0, a * 0.82, &luz));
                    enfeites.push(viga(ap, l * 0.74, -10.0, 300.0, a * 0.82, &luz2));
                    enfeites.push(forma(ap, Tipo::Elipse, [l * 0.74 - 260.0, a * 0.78, 520.0, 90.0], &luz2));
                    enfeites.push(forma(ap, Tipo::Elipse, [l * 0.74 - 150.0, a * 0.8, 300.0, 56.0], mistura(fundo, acento, 0.35)));
                    enfeites.push(forma(ap, Tipo::Elipse, [l * 0.74 - 6.0, -6.0, 12.0, 12.0], "#ffffff"));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a * 0.58, 70.0, 4.0], acento));
                    caixa(slide, Titulo, 0, [l * 0.06, a * 0.2, l * 0.52, a * 0.36], Esquerda, None, Some(60.0));
                    caixa(slide, Subtitulo, 0, [l * 0.06, a * 0.63, l * 0.46, a * 0.1], Esquerda, None, Some(18.0));
                }
                "secao" => {
                    enfeites.push(viga(ap, l / 2.0, -10.0, 620.0, a * 0.9, &luz));
                    enfeites.push(forma(ap, Tipo::Elipse, [l / 2.0 - 310.0, a * 0.86, 620.0, 100.0], &luz2));
                    caixa(slide, Titulo, 0, [l * 0.14, a * 0.36, l * 0.72, a * 0.24], Centro, None, Some(60.0));
                }
                _ => {
                    enfeites.push(viga(ap, l * 0.94, -6.0, 300.0, a * 0.5, &luz));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.07, a * 0.085, 5.0, a * 0.1], acento));
                    caixa(slide, Titulo, 0, [l * 0.07 + 24.0, a * 0.07, l * 0.7, a * 0.14], Esquerda, None, Some(38.0));
                    if camada == "dois_conteudos" {
                        for (k, x) in [l * 0.07, l * 0.52].into_iter().enumerate() {
                            enfeites.push(forma(ap, Tipo::RetanguloArredondado, [x, a * 0.28, l * 0.41, a * 0.56], &luz));
                            caixa(slide, Corpo, k, [x + 22.0, a * 0.31, l * 0.41 - 44.0, a * 0.5], Esquerda, None, Some(20.0));
                        }
                    } else {
                        caixa(slide, Corpo, 0, [l * 0.07 + 24.0, a * 0.28, l * 0.74, a * 0.58], Esquerda, None, Some(22.0));
                    }
                }
            }
        }
        // ── Tecnologia: planta de engenharia — grade, cotas e carimbo de desenho. ───────────
        ("planta", camada) => {
            let linha = mistura(fundo, "#ffffff", 0.1);
            let maior = mistura(fundo, "#ffffff", 0.2);
            for k in 1..16 {
                enfeites.push(forma(ap, Tipo::Retangulo, [k as f32 * 60.0, 0.0, 1.0, a], if k % 4 == 0 { &maior } else { &linha }));
            }
            for k in 1..9 {
                enfeites.push(forma(ap, Tipo::Retangulo, [0.0, k as f32 * 60.0, l, 1.0], if k % 4 == 0 { &maior } else { &linha }));
            }
            match camada {
                "titulo" => {
                    enfeites.push(contorno(ap, Tipo::Retangulo, [l * 0.06, a * 0.14, l * 0.88, a * 0.6], "#ffffff", 1.5));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a * 0.84, l * 0.88, 1.5], acento));
                    for x in [l * 0.06, l * 0.94 - 1.5] {
                        enfeites.push(forma(ap, Tipo::Retangulo, [x, a * 0.84 - 8.0, 1.5, 17.0], acento));
                    }
                    enfeites.push(contorno(ap, Tipo::Elipse, [l * 0.7, a * 0.2, 190.0, 190.0], acento, 1.2));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.7 + 95.0, a * 0.2 - 18.0, 1.0, 226.0], acento));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.7 - 18.0, a * 0.2 + 95.0, 226.0, 1.0], acento));
                    caixa(slide, Titulo, 0, [l * 0.09, a * 0.28, l * 0.58, a * 0.24], Esquerda, None, Some(38.0));
                    caixa(slide, Subtitulo, 0, [l * 0.09, a * 0.56, l * 0.58, a * 0.1], Esquerda, None, Some(16.0));
                }
                "secao" => {
                    enfeites.push(contorno(ap, Tipo::Elipse, [l * 0.52, a * 0.12, 380.0, 380.0], "#ffffff", 1.5));
                    enfeites.push(contorno(ap, Tipo::Elipse, [l * 0.52 + 70.0, a * 0.12 + 70.0, 240.0, 240.0], acento, 1.2));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.52 + 189.0, a * 0.12 - 20.0, 1.0, 420.0], acento));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.52 - 20.0, a * 0.12 + 189.0, 420.0, 1.0], acento));
                    caixa(slide, Titulo, 0, [l * 0.07, a * 0.38, l * 0.45, a * 0.2], Esquerda, None, Some(40.0));
                }
                _ => {
                    // O carimbo do desenho, como no canto de uma prancha.
                    enfeites.push(contorno(ap, Tipo::Retangulo, [l - 270.0, a - 70.0, 230.0, 46.0], "#ffffff", 1.2));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l - 270.0 + 150.0, a - 70.0, 1.2, 46.0], "#ffffff"));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06, a * 0.07, 10.0, 10.0], acento));
                    caixa(slide, Titulo, 0, [l * 0.06 + 24.0, a * 0.05, l * 0.8, a * 0.14], Esquerda, None, Some(26.0));
                    if camada == "dois_conteudos" {
                        for (k, x) in [l * 0.06, l * 0.52].into_iter().enumerate() {
                            enfeites.push(contorno(ap, Tipo::Retangulo, [x, a * 0.26, l * 0.42, a * 0.54], "#ffffff", 1.0));
                            enfeites.push(forma(ap, Tipo::Retangulo, [x, a * 0.26, l * 0.42, 22.0], &suave));
                            caixa(slide, Corpo, k, [x + 18.0, a * 0.26 + 34.0, l * 0.42 - 36.0, a * 0.44], Esquerda, None, Some(18.0));
                        }
                    }
                }
            }
        }
        // ── Criativo (Bauhaus): círculo, meio-disco, losango e uma barra grossa. ────────────
        ("bauhaus", camada) => {
            let (vermelho, azul, amarelo) = ("#e63946", "#1d4ed8", "#ffd23f");
            match camada {
                "titulo" => {
                    enfeites.push(forma(ap, Tipo::Elipse, [l * 0.58, a * 0.1, 300.0, 300.0], azul));
                    let mut meio = forma(ap, Tipo::Setor, [l * 0.5, a * 0.5, 260.0, 260.0], vermelho);
                    meio.ajustes = vec![180.0, 360.0];
                    enfeites.push(meio);
                    let mut losango = forma(ap, Tipo::Retangulo, [l * 0.84, a * 0.56, 90.0, 90.0], amarelo);
                    losango.rotacao = 45.0;
                    enfeites.push(losango);
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.05, a - 46.0, l * 0.9, 12.0], tinta));
                    caixa(slide, Titulo, 0, [l * 0.06, a * 0.22, l * 0.46, a * 0.34], Esquerda, None, Some(54.0));
                    caixa(slide, Subtitulo, 0, [l * 0.06, a * 0.6, l * 0.4, a * 0.1], Esquerda, None, Some(18.0));
                }
                "secao" => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 0.0, l, a], amarelo));
                    enfeites.push(forma(ap, Tipo::Elipse, [-80.0, a * 0.5, 320.0, 320.0], vermelho));
                    let mut quarto = forma(ap, Tipo::Setor, [l - 330.0, -150.0, 480.0, 480.0], azul);
                    quarto.ajustes = vec![90.0, 180.0];
                    enfeites.push(quarto);
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.08, a * 0.34, 160.0, 12.0], tinta));
                    caixa(slide, Titulo, 0, [l * 0.08, a * 0.4, l * 0.74, a * 0.24], Esquerda, None, Some(60.0));
                }
                _ => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 0.0, l, 10.0], tinta));
                    for (k, c) in [vermelho, azul, amarelo].into_iter().enumerate() {
                        enfeites.push(forma(ap, Tipo::Retangulo, [l - 120.0 + k as f32 * 34.0, 28.0, 22.0, 22.0], c));
                    }
                    caixa(slide, Titulo, 0, [l * 0.06, a * 0.07, l * 0.7, a * 0.15], Esquerda, None, Some(34.0));
                    if camada == "dois_conteudos" {
                        enfeites.push(forma(ap, Tipo::Retangulo, [l / 2.0 - 3.0, a * 0.3, 6.0, 90.0], azul));
                        caixa(slide, Corpo, 0, [l * 0.06, a * 0.28, l * 0.4, a * 0.58], Esquerda, None, Some(20.0));
                        caixa(slide, Corpo, 1, [l * 0.54, a * 0.28, l * 0.4, a * 0.58], Esquerda, None, Some(20.0));
                    } else {
                        caixa(slide, Corpo, 0, [l * 0.06, a * 0.28, l * 0.78, a * 0.58], Esquerda, None, Some(22.0));
                    }
                }
            }
        }
        // ── Pitch: órbita — um planeta cortado pela borda, um anel e um satélite coral. ─────
        ("orbita", camada) => {
            let estrelas = [(0.07, 0.1), (0.2, 0.84), (0.33, 0.14), (0.47, 0.9), (0.9, 0.08), (0.62, 0.06), (0.05, 0.6), (0.94, 0.52)];
            let brilho_estrela = mistura(fundo, "#ffffff", 0.7);
            for (x, y) in estrelas {
                enfeites.push(forma(ap, Tipo::Elipse, [l * x, a * y, 3.0, 3.0], &brilho_estrela));
            }
            match camada {
                "titulo" => {
                    enfeites.push(forma(ap, Tipo::Elipse, [l * 0.56, -70.0, 540.0, 540.0], mistura(fundo, "#ffffff", 0.08)));
                    enfeites.push(contorno(ap, Tipo::Elipse, [l * 0.46, a * 0.2, 640.0, 250.0], acento, 1.4));
                    enfeites.push(forma(ap, Tipo::Elipse, [l * 0.46 + 600.0 - 8.0, a * 0.2 + 125.0 - 8.0, 16.0, 16.0], acento));
                    caixa(slide, Titulo, 0, [l * 0.06, a * 0.26, l * 0.5, a * 0.28], Esquerda, None, Some(50.0));
                    caixa(slide, Subtitulo, 0, [l * 0.06, a * 0.58, l * 0.44, a * 0.1], Esquerda, None, Some(18.0));
                }
                "secao" => {
                    enfeites.push(forma(ap, Tipo::Retangulo, [0.0, 0.0, l, a], acento));
                    enfeites.push(contorno(ap, Tipo::Elipse, [l - 360.0, a - 360.0, 560.0, 560.0], fundo, 1.6));
                    enfeites.push(forma(ap, Tipo::Elipse, [l - 150.0, a - 150.0, 150.0, 150.0], fundo));
                    caixa(slide, Titulo, 0, [l * 0.08, a * 0.38, l * 0.6, a * 0.22], Esquerda, Some(fundo), Some(56.0));
                }
                _ => {
                    enfeites.push(forma(ap, Tipo::Elipse, [l * 0.06, a * 0.085, 14.0, 14.0], acento));
                    enfeites.push(forma(ap, Tipo::Retangulo, [l * 0.06 + 28.0, a * 0.215, 90.0, 3.0], acento));
                    caixa(slide, Titulo, 0, [l * 0.06 + 28.0, a * 0.06, l * 0.82, a * 0.14], Esquerda, None, Some(34.0));
                    if camada == "dois_conteudos" {
                        for (k, x) in [l * 0.06, l * 0.51].into_iter().enumerate() {
                            enfeites.push(forma(ap, Tipo::RetanguloArredondado, [x, a * 0.28, l * 0.43, a * 0.58], mistura(fundo, "#ffffff", 0.07)));
                            caixa(slide, Corpo, k, [x + 22.0, a * 0.32, l * 0.43 - 44.0, a * 0.5], Esquerda, None, Some(20.0));
                        }
                    } else {
                        caixa(slide, Corpo, 0, [l * 0.06 + 28.0, a * 0.28, l * 0.78, a * 0.58], Esquerda, None, Some(22.0));
                    }
                }
            }
        }
        _ => {}
    }
    // Layouts que só o motor novo conhece: os textos voltam às posições do layout (respeitando a
    // margem que o modelo reservou) e ganham o enfeite do estilo do modelo.
    if modelo != "projeto" && matches!(layout, "comparacao" | "tres_colunas" | "quatro_quadrantes" | "titulo_lateral" | "numeros" | "linha_do_tempo" | "citacao" | "somente_conteudo") {
        let mut n = 0;
        let padrao = crate::layouts::elementos_do_layout(layout, l, a, &mut || {
            n += 1;
            n
        })
        .unwrap_or_default();
        let titulo_do_modelo = slide.elementos.iter().find(|e| e.papel == Papel::Titulo).map(|e| e.x);
        let titulo_padrao = padrao.iter().find(|e| e.papel == Papel::Titulo).map(|e| e.x);
        let dx = match (titulo_do_modelo, titulo_padrao) {
            (Some(m), Some(p)) => (m - p).clamp(0.0, l * 0.3),
            _ => 0.0,
        };
        let f = (l * 0.88 - dx) / (l * 0.88);
        let textos: Vec<usize> = (0..slide.elementos.len()).filter(|&i| matches!(slide.elementos[i].papel, Papel::Titulo | Papel::Subtitulo | Papel::Corpo)).collect();
        for (&i, p) in textos.iter().zip(padrao.iter()) {
            let e = &mut slide.elementos[i];
            [e.x, e.y, e.largura, e.altura] = [(l * 0.06 + dx) + (p.x - l * 0.06) * f, p.y, p.largura * f, p.altura];
        }
        let rect = |papel: Papel| -> Vec<[f32; 4]> { slide.elementos.iter().filter(|e| e.papel == papel).map(|e| [e.x, e.y, e.largura, e.altura]).collect() };
        let (corpos, subs) = (rect(Papel::Corpo), rect(Papel::Subtitulo));
        let fio = matches!(modelo, "projeto" | "relatorio" | "memorando" | "tecnologia" | "planta" | "elegante" | "minimalista" | "aula" | "caderno" | "palestra");
        let linha_cor = if escuro { mistura(fundo, "#ffffff", 0.35) } else { mistura(fundo, tinta, 0.3) };
        let cartao = |ap: &mut Apresentacao, r: [f32; 4], v: &mut Vec<Elemento>| {
            if fio {
                v.push(contorno(ap, Tipo::Retangulo, r, &linha_cor, 1.0));
                v.push(forma(ap, Tipo::Retangulo, [r[0], r[1], 40.0, 3.0], acento));
            } else {
                v.push(forma(ap, Tipo::RetanguloArredondado, r, &suave));
                v.push(forma(ap, Tipo::Retangulo, [r[0] + 14.0, r[1] + 12.0, 36.0, 4.0], acento));
            }
        };
        match layout {
            "comparacao" => {
                for (k, c) in corpos.iter().enumerate() {
                    let topo = subs.get(k).map_or(c[1], |s| s[1]) - 10.0;
                    cartao(ap, [c[0] - 12.0, topo, c[2] + 24.0, c[1] + c[3] - topo + 8.0], &mut enfeites);
                }
            }
            "tres_colunas" | "quatro_quadrantes" => {
                for c in &corpos {
                    cartao(ap, [c[0] - 12.0, c[1] - 12.0, c[2] + 24.0, c[3] + 20.0], &mut enfeites);
                }
            }
            "numeros" => {
                for c in &corpos {
                    enfeites.push(forma(ap, Tipo::Retangulo, [c[0] + c[2] / 2.0 - 26.0, c[1] + c[3] + 6.0, 52.0, 4.0], acento));
                }
                for k in 1..3 {
                    if let Some(c) = corpos.get(k) {
                        enfeites.push(forma(ap, Tipo::Retangulo, [c[0] - (l * 0.05) / 2.0 - 1.0, a * 0.3, 1.0, a * 0.38], &linha_cor));
                    }
                }
            }
            "linha_do_tempo" => {
                if let (Some(p), Some(u)) = (subs.first(), subs.last()) {
                    let y = p[1] - 16.0;
                    enfeites.push(forma(ap, Tipo::Retangulo, [p[0], y, u[0] + u[2] - p[0], 2.0], &linha_cor));
                    for s in &subs {
                        enfeites.push(forma(ap, Tipo::Elipse, [s[0] + s[2] / 2.0 - 8.0, y - 7.0, 16.0, 16.0], acento));
                    }
                }
            }
            "citacao" => {
                let mut aspas = forma(ap, Tipo::Texto, [l * 0.08, a * 0.02, 220.0, 220.0], "");
                aspas.texto = "\u{201C}".into();
                aspas.preenchimento = String::new();
                aspas.estilo.tamanho = 190.0;
                aspas.estilo.negrito = true;
                aspas.estilo.cor = acento.into();
                aspas.estilo.fonte = "Liberation Serif".into();
                enfeites.push(aspas);
                if let Some(c) = corpos.first() {
                    enfeites.push(forma(ap, Tipo::Retangulo, [l / 2.0 - 30.0, c[1] + c[3] + 8.0, 60.0, 3.0], acento));
                }
            }
            "titulo_lateral" => {
                if let (Some(t), Some(c)) = (rect(Papel::Titulo).first().copied(), corpos.first()) {
                    let x = (t[0] + t[2] + c[0]) / 2.0;
                    enfeites.push(forma(ap, Tipo::Retangulo, [x - 1.5, a * 0.14, 3.0, a * 0.72], acento));
                }
            }
            _ => {}
        }
    }
    // Os enfeites entram atrás de tudo, na ordem em que foram criados.
    for (k, e) in enfeites.into_iter().enumerate() {
        slide.elementos.insert(k, e);
    }
}

/// A apresentação do modelo `id` (`None` se não existe).
pub fn criar(id: &str) -> Option<Apresentacao> {
    let modelo = MODELOS.iter().find(|m| m.id == id)?;
    let mut ap = Apresentacao { titulo: String::new(), autor: String::new(), proporcao: Proporcao::Larga, dimensoes: [0.0; 2], slides: Vec::new(), proximo_id: 1, fontes: Vec::new(), tema: String::new(), modelo: String::new(), metadados: crate::metadados::Metadados::novo() };
    if modelo.tema != "office" {
        ap.tema = modelo.tema.into();
    }
    if id != "em_branco" {
        ap.modelo = id.into();
    }
    for (layout, titulo, corpo) in telas(id) {
        let mut slide = slide_com_layout(&mut ap, layout);
        // Um bloco de corpo por caixa: no layout de dois conteúdos, a linha em branco separa as colunas.
        let mut blocos = corpo.split("\n\n");
        for e in &mut slide.elementos {
            match e.papel {
                Papel::Titulo if !titulo.is_empty() => e.texto = titulo.into(),
                Papel::Subtitulo | Papel::Corpo if !corpo.is_empty() => {
                    if let Some(b) = blocos.next() {
                        e.texto = b.into();
                    }
                }
                _ => {}
            }
        }
        ap.slides.push(slide);
    }
    ap.titulo = modelo.nome.into();
    Some(ap)
}
