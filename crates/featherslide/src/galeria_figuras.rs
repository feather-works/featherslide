//! As galerias de Formas e de Gráficos: cada opção aparece desenhada, não só escrita.

use pelicano::{button, column, container, mouse_area, program, row, text, Space};
use pelicano::{Align, Background, Border, Color, Element, FillRule, Length, LineCap, LineJoin, Path, Point, Program, Rect, Renderer, Shadow, Size, Status, Stroke, Style, Theme, Transform};

use crate::app::Mensagem;
use crate::desenho::caminho_da_forma;
use crate::faixa::FaixaMensagem;
use crate::tema::*;
use featherslide_core::modelo::Tipo;
use featherslide_core::diagramas::{self, Categoria};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Figuras {
    Formas,
    Graficos,
    Diagramas,
    Recorte,
    Icones,
    TrocarIcone,
}

/// Uma opção da galeria: o rótulo, o que ela faz e o que se desenha.
struct Opcao {
    rotulo: &'static str,
    msg: FaixaMensagem,
    desenho: Desenho,
}

#[derive(Clone, Copy)]
enum Desenho {
    Forma(&'static str, &'static [f32]),
    Modelo(&'static str),
    Colunas,
    Barras,
    Linhas,
    Pizza,
    Diagrama(Categoria),
    Icone(&'static str),
}

fn formas() -> Vec<Opcao> {
    let mut opcoes = vec![
        Opcao { rotulo: "Retângulo", msg: FaixaMensagem::Inserir(Tipo::Retangulo), desenho: Desenho::Forma("retangulo", &[]) },
        Opcao { rotulo: "Retângulo arredondado", msg: FaixaMensagem::Inserir(Tipo::RetanguloArredondado), desenho: Desenho::Forma("retangulo_arredondado", &[]) },
        Opcao { rotulo: "Elipse", msg: FaixaMensagem::Inserir(Tipo::Elipse), desenho: Desenho::Forma("elipse", &[]) },
        Opcao { rotulo: "Círculo parcial", msg: FaixaMensagem::Inserir(Tipo::Setor), desenho: Desenho::Forma("setor", &[0.0, 270.0]) },
        Opcao { rotulo: "Triângulo", msg: FaixaMensagem::Inserir(Tipo::Triangulo), desenho: Desenho::Forma("triangulo", &[]) },
        Opcao { rotulo: "Seta", msg: FaixaMensagem::Inserir(Tipo::Seta), desenho: Desenho::Forma("seta", &[]) },
        Opcao { rotulo: "Linha", msg: FaixaMensagem::Inserir(Tipo::Linha), desenho: Desenho::Forma("linha", &[]) },
    ];
    opcoes.extend(featherslide_core::formas::ADICIONAIS.iter().map(|&(tipo, rotulo)| Opcao {
        rotulo, msg: FaixaMensagem::Inserir(tipo), desenho: Desenho::Forma(tipo.nome(), &[]),
    }));
    opcoes.extend(featherslide_core::outras_galerias::FORMAS.iter().map(|&(_,id,rotulo)| Opcao {
        rotulo, msg: FaixaMensagem::InserirGaleria(id), desenho: Desenho::Modelo(id),
    }));
    for (id,rotulo) in [("reto","Conector reto"),("cotovelo","Conector em cotovelo"),("curvo","Conector curvo")] {
        let modelo=match id{"cotovelo"=>"conector_cotovelo","curvo"=>"conector_curvo",_=>"conector_reto"};
        opcoes.push(Opcao{rotulo,msg:FaixaMensagem::InserirConector(id),desenho:Desenho::Modelo(modelo)});
    }
    opcoes.push(Opcao{rotulo:"Desenho livre",msg:FaixaMensagem::DesenhoLivre,desenho:Desenho::Modelo("livre")});
    opcoes
}

fn recorte() -> Vec<Opcao> {
    vec![
        Opcao { rotulo: "Retângulo (sem corte)", msg: FaixaMensagem::RecorteForma(Tipo::Retangulo), desenho: Desenho::Forma("retangulo", &[]) },
        Opcao { rotulo: "Elipse (círculo)", msg: FaixaMensagem::RecorteForma(Tipo::Elipse), desenho: Desenho::Forma("elipse", &[]) },
        Opcao { rotulo: "Cantos arredondados", msg: FaixaMensagem::RecorteForma(Tipo::RetanguloArredondado), desenho: Desenho::Forma("retangulo_arredondado", &[]) },
        Opcao { rotulo: "Triângulo", msg: FaixaMensagem::RecorteForma(Tipo::Triangulo), desenho: Desenho::Forma("triangulo", &[]) },
        Opcao { rotulo: "Círculo parcial", msg: FaixaMensagem::RecorteForma(Tipo::Setor), desenho: Desenho::Forma("setor", &[0.0, 270.0]) },
        Opcao { rotulo: "Seta", msg: FaixaMensagem::RecorteForma(Tipo::Seta), desenho: Desenho::Forma("seta", &[]) },
    ]
}

fn diagramas() -> Vec<Opcao> {
    Categoria::TODAS.into_iter()
        .map(|categoria| Opcao {
            rotulo: categoria.rotulo(),
            msg: FaixaMensagem::DiagramaInserir(categoria),
            desenho: Desenho::Diagrama(categoria),
        })
        .collect()
}

fn graficos() -> Vec<Opcao> {
    vec![
        Opcao { rotulo: "Colunas", msg: FaixaMensagem::InserirGrafico("colunas".into()), desenho: Desenho::Colunas },
        Opcao { rotulo: "Barras", msg: FaixaMensagem::InserirGrafico("barras".into()), desenho: Desenho::Barras },
        Opcao { rotulo: "Linhas", msg: FaixaMensagem::InserirGrafico("linhas".into()), desenho: Desenho::Linhas },
        Opcao { rotulo: "Pizza", msg: FaixaMensagem::InserirGrafico("pizza".into()), desenho: Desenho::Pizza },
    ]
}

struct Figura(Desenho);

fn encher(r: &mut Renderer, p: &Path, c: Color) {
    r.fill_path(p, c, FillRule::NonZero);
}

fn tracar(r: &mut Renderer, p: &Path, c: Color, largura: f32) {
    r.stroke_path(p, &Stroke::new(largura), c);
}

impl Program<FaixaMensagem> for Figura {
    type State = ();

    fn draw(&self, _: &(), f: &mut Renderer, bounds: Rect, _: &Theme, _: Option<Point>) {
        // Os desenhos abaixo estão no espaço do canvas; só caminhos seguem a transformação.
        f.push_transform(Transform::translate(bounds.x, bounds.y));
        let (w, h) = (bounds.width, bounds.height);
        let acento = COR_AZUL_ACENTO();
        let suave = Color { a: 0.45, ..acento };
        let eixo = COR_TEXTO_MUTED();
        match self.0 {
            Desenho::Modelo(id) => {
                use featherslide_core::vetores::Segmento;
                let e = featherslide_core::modelo::Elemento {tipo:Tipo::Caminho,x:8.0,y:8.0,largura:w-16.0,altura:h-16.0,dados:featherslide_core::outras_galerias::geometria(id).unwrap_or_default(),..Default::default()};
                if let Ok((_,segmentos))=featherslide_core::vetores::decodificar(&e) {
                    let p=|v:[f32;2]|Point::new(v[0],v[1]);
                    if let Some(path)=Path::new(|b|for s in segmentos {match s{Segmento::M(v)=>b.move_to(p(v)),Segmento::L(v)=>b.line_to(p(v)),Segmento::Q(a,v)=>b.quadratic_to(p(a),p(v)),Segmento::C(a,c,v)=>b.bezier_to(p(a),p(c),p(v)),Segmento::Z=>b.close()};}) {
                        if !id.starts_with("conector_") && id!="livre" {encher(f,&path,suave);}
                        tracar(f,&path,acento,1.5);
                    }
                }
            }
            Desenho::Forma(tipo, aj) => {
                let (m, mut cw, mut ch) = (10.0, w - 20.0, h - 20.0);
                if tipo == "retangulo" || tipo == "retangulo_arredondado" || tipo == "seta" {
                    ch = ch * 0.7;
                }
                if tipo == "elipse" || tipo == "setor" {
                    cw = cw.min(ch);
                    ch = cw;
                }
                let (x, y) = ((w - cw) / 2.0, (h - ch) / 2.0);
                let nova = Tipo::pelo_nome(tipo).and_then(|t| featherslide_core::novas_formas::desenho(t, x, y, cw, ch));
                if let Some(d) = nova {
                    if !d.contornos.is_empty() {
                        // Todos os anéis no MESMO path: anel e moldura preservam o furo.
                        let p = Path::new(|b| {
                            for contorno in &d.contornos {
                                if let Some(primeiro) = contorno.first() {
                                    b.move_to(Point::new(primeiro[0], primeiro[1]));
                                    for q in &contorno[1..] { b.line_to(Point::new(q[0], q[1])); }
                                    b.close();
                                }
                            }
                        });
                        if let Some(p) = p {
                            encher(f, &p, suave);
                            tracar(f, &p, acento, 2.0);
                        }
                    }
                    for traco in &d.tracos {
                        let p = Path::new(|b| {
                            if let Some(primeiro) = traco.first() {
                                b.move_to(Point::new(primeiro[0], primeiro[1]));
                                for q in &traco[1..] { b.line_to(Point::new(q[0], q[1])); }
                            }
                        });
                        if let Some(p) = p { tracar(f, &p, acento, 2.0); }
                    }
                } else {
                    let p = caminho_da_forma(tipo, x, y, cw, ch, aj, false);
                    if tipo == "linha" {
                        tracar(f, &Path::line(Point::new(m, h - m), Point::new(w - m, m)), acento, 3.0);
                    } else if let Some(p) = p {
                        encher(f, &p, suave);
                        tracar(f, &p, acento, 2.0);
                    }
                }
            }
            Desenho::Diagrama(categoria) => {
                // A miniatura usa os mesmos componentes vetoriais do motor.
                // Aqui só se escala a geometria; não se rasteriza nem se grava PNG.
                for el in diagramas::modelo(categoria, 2.0, 2.0, w - 4.0, h - 4.0) {
                    let hex = el.preenchimento.trim_start_matches('#');
                    let colorir = |s: &str| -> Color {
                        let h = s.trim_start_matches('#');
                        let rgb = u32::from_str_radix(h.get(..6).unwrap_or("D3DFEC"), 16).unwrap_or(0xD3DFEC);
                        let a = u8::from_str_radix(h.get(6..8).unwrap_or("FF"), 16).unwrap_or(255);
                        Color::rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8).alpha(a as f32 / 255.0)
                    };
                    if el.tipo == Tipo::Linha {
                        tracar(f, &Path::line(Point::new(el.x, el.y),
                            Point::new(el.x + el.largura, el.y + el.altura)),
                            colorir(&el.contorno), 1.0);
                    } else if !hex.is_empty() && el.tipo != Tipo::Texto {
                        let path = caminho_da_forma(el.tipo.nome(), el.x, el.y, el.largura, el.altura, &el.ajustes, false);
                        if let Some(path) = path { encher(f, &path, colorir(&el.preenchimento)); }
                    }
                }
            }
            Desenho::Colunas => {
                for (i, v) in [0.45, 0.8, 0.6, 0.95].into_iter().enumerate() {
                    if let Some(p) = Path::rect(Rect::new(Point::new(14.0 + i as f32 * 20.0, h - 12.0 - (h - 28.0) * v), Size::new(14.0, (h - 28.0) * v))) {
                        encher(f, &p, if i == 3 { acento } else { suave });
                    }
                }
                tracar(f, &Path::line(Point::new(10.0, h - 12.0), Point::new(w - 8.0, h - 12.0)), eixo, 1.0);
            }
            Desenho::Barras => {
                for (i, v) in [0.5, 0.85, 0.65, 0.95].into_iter().enumerate() {
                    if let Some(p) = Path::rect(Rect::new(Point::new(12.0, 10.0 + i as f32 * 17.0), Size::new((w - 24.0) * v, 11.0))) {
                        encher(f, &p, if i == 3 { acento } else { suave });
                    }
                }
                tracar(f, &Path::line(Point::new(12.0, 8.0), Point::new(12.0, h - 8.0)), eixo, 1.0);
            }
            Desenho::Linhas => {
                let pontos = [(0.0, 0.7), (0.3, 0.4), (0.55, 0.6), (0.8, 0.2), (1.0, 0.3)];
                let xy = |(u, v): (f32, f32)| Point::new(12.0 + (w - 24.0) * u, 10.0 + (h - 28.0) * v);
                let caminho = Path::new(|b| {
                    b.move_to(xy(pontos[0]));
                    for p in &pontos[1..] {
                        b.line_to(xy(*p));
                    }
                });
                if let Some(caminho) = caminho { tracar(f, &caminho, acento, 2.5); }
                for p in pontos {
                    if let Some(c) = Path::circle(xy(p), 3.2) { encher(f, &c, acento); }
                }
                tracar(f, &Path::line(Point::new(10.0, h - 12.0), Point::new(w - 8.0, h - 12.0)), eixo, 1.0);
            }
            Desenho::Icone(id) => {
                use featherslide_core::modelo::Elemento;
                let e = Elemento { tipo: Tipo::Icone, dados: id.to_string(), preenchimento: "#1a73e8".into(), x: 0.0, y: 0.0, largura: w, altura: h, ..Elemento::default() };
                for p in featherslide_core::icones::primitivas(&e) {
                    pintar_primitiva(f, &p);
                }
            }
            Desenho::Pizza => {
                let (c, r) = (Point::new(w / 2.0, h / 2.0), (w.min(h) / 2.0 - 8.0).max(8.0));
                let fatias = [(0.0, 130.0, acento), (130.0, 250.0, suave), (250.0, 360.0, Color { a: 0.75, ..acento })];
                for (de, ate, cor) in fatias {
                    let caminho = Path::new(|b| {
                        b.move_to(c);
                        for i in 0..=36 {
                            let g = (de + (ate - de) * i as f32 / 36.0_f32).to_radians();
                            b.line_to(Point::new(c.x + r * g.cos(), c.y + r * g.sin()));
                        }
                        b.close();
                    });
                    if let Some(caminho) = caminho {
                        encher(f, &caminho, cor);
                        tracar(f, &caminho, Color::WHITE, 1.5);
                    }
                }
            }
        }
        f.pop_transform();
    }
}

/// Pinta uma primitiva de ícone (polígono cheio ou traço) com a cor que ela traz.
fn pintar_primitiva(f: &mut Renderer, p: &featherslide_core::dados::Primitiva) {
    use featherslide_core::dados::Primitiva;
    let cor = |s: &str| {
        let h = s.trim_start_matches('#');
        u32::from_str_radix(h.get(..6)?, 16).ok().map(|v| Color::rgb8((v >> 16) as u8, (v >> 8) as u8, v as u8))
    };
    match p {
        Primitiva::Poligono { pontos, preenchimento } if pontos.len() > 2 => {
            if let Some(c) = cor(preenchimento) {
                if let Some(p) = Path::new(|b| {
                    b.move_to(Point::new(pontos[0][0], pontos[0][1]));
                    for q in &pontos[1..] {
                        b.line_to(Point::new(q[0], q[1]));
                    }
                    b.close();
                }) {
                    encher(f, &p, c);
                }
            }
        }
        Primitiva::Poli { pontos, cor: c, espessura } if pontos.len() > 1 => {
            if let Some(c) = cor(c) {
                let caminho = Path::new(|b| {
                    b.move_to(Point::new(pontos[0][0], pontos[0][1]));
                    for q in &pontos[1..] {
                        b.line_to(Point::new(q[0], q[1]));
                    }
                });
                if let Some(caminho) = caminho {
                    f.stroke_path(&caminho, &Stroke::new(*espessura).cap(LineCap::Round).join(LineJoin::Round), c);
                }
            }
        }
        _ => {}
    }
}

/// Quantos resultados a galeria mostra de uma vez.
const LIMITE_ICONES: usize = 160;
const POR_LINHA_ICONES: usize = 8;
pub const ID_BUSCA_ICONES: &str = "busca_icones";

/// A galeria de ícones: caixa de busca (português ou inglês) e a grade dos que casam.
fn galeria_icones<'a>(busca: &str, trocar: bool) -> Element<'a, FaixaMensagem> {
    let achados = featherslide_core::icones::buscar(busca, LIMITE_ICONES);
    let mut grade = column![].spacing(2);
    let mut linha = row![].spacing(2);
    let mut n = 0;
    for ic in &achados {
        let celula = feather_ui::formatacao::dica(
            button(program(Figura(Desenho::Icone(ic.id))).width(Length::Fixed(40.0)).height(Length::Fixed(40.0)))
                .padding(5)
                .style(|_, status| Style {
                    background: Some(Background::Color(if matches!(status, Status::Hovered | Status::Pressed) { COR_HOVER() } else { Color::TRANSPARENT })),
                    text_color: Some(COR_TEXTO()),
                    border: Border { color: if matches!(status, Status::Hovered | Status::Pressed) { COR_AZUL_ACENTO() } else { Color::TRANSPARENT }, width: 1.5, radius: [6.0; 4] },
                    shadow: None, ..Style::default() })
                .on_press(if trocar { FaixaMensagem::IconeSubstituir(ic.id.to_string()) } else { FaixaMensagem::InserirIcone(ic.id.to_string()) }),
            &ic.rotulo(),
        );
        linha = linha.push(celula);
        n += 1;
        if n % POR_LINHA_ICONES == 0 {
            grade = grade.push(std::mem::replace(&mut linha, row![].spacing(2)));
        }
    }
    if n % POR_LINHA_ICONES != 0 {
        grade = grade.push(linha);
    }
    let resumo = if busca.trim().is_empty() {
        format!("Mais usados — busque entre {} ícones", featherslide_core::icones::todos().len())
    } else if achados.is_empty() {
        "Nenhum ícone encontrado. Tente outra palavra (ex.: casa, seta, gráfico, user, chart).".to_string()
    } else if achados.len() == LIMITE_ICONES {
        format!("{LIMITE_ICONES}+ resultados — refine a busca")
    } else {
        format!("{} resultado(s)", achados.len())
    };
    column![
        pelicano::text_input("Buscar ícone… (casa, seta, relógio, chart)", busca.to_string())
            .id(ID_BUSCA_ICONES)
            .on_input(FaixaMensagem::BuscaIcones)
            .size(13)
            .padding([6, 9]),
        text(resumo).size(11).color(COR_TEXTO_MUTED()),
        pelicano::scrollable(grade).scrollbar_width(10).scrollbar_track(COR_TRILHO_ROLAGEM()).scrollbar_color(COR_POLEGAR_ROLAGEM()).height(Length::Fixed(330.0)),
    ]
    .spacing(6)
    .into()
}

fn cartao<'a>(o: Opcao) -> Element<'a, FaixaMensagem> {
    button(
        column![
            program(Figura(o.desenho)).width(Length::Fixed(92.0)).height(Length::Fixed(64.0)),
            text(o.rotulo).size(11.5).width(Length::Fixed(92.0)).align_x(Align::Center),
        ]
        .spacing(5)
        .align(Align::Center),
    )
    .padding(7)
    .style(|_, status| Style {
        background: Some(Background::Color(if matches!(status, Status::Hovered | Status::Pressed) { COR_HOVER() } else { Color::TRANSPARENT })),
        text_color: Some(COR_TEXTO()),
        border: Border { color: if matches!(status, Status::Hovered | Status::Pressed) { COR_AZUL_ACENTO() } else { Color::TRANSPARENT }, width: 1.5, radius: [6.0; 4] },
        shadow: None, ..Style::default() })
    .on_press(o.msg)
    .into()
}

/// A galeria por cima da tela; clicar fora fecha.
pub fn sobrepor<'a>(base: Element<'a, Mensagem>, qual: Figuras, busca_icones: &str, largura_janela: f32) -> Element<'a, Mensagem> {
    let fundo = mouse_area(Space::new(Length::FILL, Length::FILL)).on_press(Mensagem::Faixa(FaixaMensagem::FecharFiguras));
    if matches!(qual, Figuras::Icones | Figuras::TrocarIcone) {
        let (esquerda, topo) = crate::medido::ancora("icones", 400.0, largura_janela, (250.0, 128.0));
        let painel = container(column![text("Ícones").size(12).color(COR_TEXTO_MUTED()), galeria_icones(busca_icones, qual == Figuras::TrocarIcone)].spacing(8))
            .width(Length::Fixed(400.0))
            .padding(12)
            .style(|_| Style {
                background: Some(Background::Color(COR_PAINEL())),
                text_color: Some(COR_TEXTO()),
                border: Border { color: COR_BORDA(), width: 1.0, radius: [8.0; 4] },
                shadow: Some(Shadow { color: Color::rgb8(0, 0, 0).alpha(0.3), offset: Point::new(0.0, 6.0), blur: 18.0 }), ..Style::default() });
        let sobreposto = container(Element::from(painel).map(Mensagem::Faixa)).padding(pelicano::Padding { top: topo, left: esquerda, right: 0.0, bottom: 0.0 });
        return pelicano::stack![base, fundo, sobreposto].into();
    }
    let (titulo, opcoes, nome, padrao) = match qual {
        Figuras::Formas => ("Formas", formas(), "formas", 250.0),
        Figuras::Graficos => ("Gráfico", graficos(), "grafico", 330.0),
        Figuras::Diagramas => ("Diagramas · categorias", diagramas(), "diagramas", 435.0),
        Figuras::Recorte => ("Cortar como figura", recorte(), "recorte_figura", 330.0),
        Figuras::Icones | Figuras::TrocarIcone => unreachable!("tratado acima"),
    };
    let (esquerda, topo) = crate::medido::ancora(nome, 440.0, largura_janela, (padrao, 128.0));
    let mut grade = column![].spacing(4);
    let mut atual = row![].spacing(4);
    let mut n = 0;
    let mut categoria = "";
    for o in opcoes {
        let nova = match &o.msg {
            FaixaMensagem::InserirGaleria(id) => featherslide_core::outras_galerias::FORMAS.iter().find(|(_,i,_)|i==id).map(|(g,_,_)|*g).unwrap_or("Outras formas"),
            FaixaMensagem::InserirConector(_) => "Conectores — clique na origem e no destino",
            FaixaMensagem::DesenhoLivre => "Formas livres — arraste no palco",
            _ => "Formas básicas",
        };
        if qual==Figuras::Formas && nova!=categoria {
            if n%4!=0 {grade=grade.push(std::mem::replace(&mut atual,row![].spacing(4)));}
            n=0;categoria=nova;grade=grade.push(text(nova).size(12).color(COR_TEXTO_MUTED()));
        }
        atual = atual.push(cartao(o));
        n += 1;
        if n % 4 == 0 {
            grade = grade.push(std::mem::replace(&mut atual, row![].spacing(4)));
        }
    }
    if n % 4 != 0 {
        grade = grade.push(atual);
    }
    let painel = container(column![text(titulo).size(12).color(COR_TEXTO_MUTED()), container(pelicano::scrollable(grade).scrollbar_width(10).scrollbar_track(COR_TRILHO_ROLAGEM()).scrollbar_color(COR_POLEGAR_ROLAGEM()).height(Length::Shrink)).max_height(420.0)].spacing(8))
        .padding(12)
        .style(|_| Style {
            background: Some(Background::Color(COR_PAINEL())),
            text_color: Some(COR_TEXTO()),
            border: Border { color: COR_BORDA(), width: 1.0, radius: [8.0; 4] },
            shadow: Some(Shadow { color: Color::rgb8(0, 0, 0).alpha(0.3), offset: Point::new(0.0, 6.0), blur: 18.0 }), ..Style::default() });
    let sobreposto = container(Element::from(painel).map(Mensagem::Faixa)).padding(pelicano::Padding { top: topo, left: esquerda, right: 0.0, bottom: 0.0 });
    pelicano::stack![base, fundo, sobreposto].into()
}
