//! O seletor de cor: um quadrado de saturação/brilho, a barra de matiz, a barra de
//! transparência, o conta-gotas (pega a cor de um ponto do slide) e os campos R, G, B, A e hex.
//! Abre embaixo do botão da cor; cada mudança já vale no elemento.

use std::f32::consts::FRAC_PI_2;

use pelicano::{button, column, container, mouse_area, program, row, text, text_input, Space};
use pelicano::{Align, Background, Border, Captured, Color, Element, Event, Length, MouseButton, Path, Point, Program, Rect, Renderer, Shadow, Size, Stroke, Style, Theme};

use crate::app::Mensagem;
use crate::faixa::{AlvoCor, FaixaMensagem};
use crate::icones::icone_conta_gotas;
use crate::tema::*;

// ------------------------------------------------------------------ conversões

// Conversões extraídas para feather-ui, compartilhadas com FeatherSheet.
pub use feather_ui::seletor_cor::{rgba_de, hex_de, hex_valido, hsv_para_rgb, rgba_para_hsva};

fn cor(hex: &str) -> Color {
    let [r, g, b, a] = rgba_de(hex);
    Color::rgb8(r, g, b).alpha(a as f32 / 255.0)
}

// ------------------------------------------------------------------ as três áreas de arrastar

/// Traduz o arrasto do mouse na área em (x, y) de 0 a 1; `None` se o evento não conta.
fn arrasto(arrastando: &mut bool, ev: &Event, b: Rect, c: Option<Point>) -> Option<(f32, f32)> {
    let norm = |p: Point| (((p.x - b.x) / b.width).clamp(0.0, 1.0), ((p.y - b.y) / b.height).clamp(0.0, 1.0));
    match ev {
        Event::MousePressed(MouseButton::Left) if c.is_some_and(|p| b.contains(p)) => {
            *arrastando = true;
            c.map(norm)
        }
        Event::CursorMoved(p) if *arrastando => Some(norm(*p)),
        Event::MouseReleased(MouseButton::Left) => {
            *arrastando = false;
            None
        }
        _ => None,
    }
}

/// `angulo` em radianos: 0 = esquerda→direita, π/2 = cima→baixo.
fn degrade(r: &mut Renderer, area: Rect, angulo: f32, paradas: Vec<(f32, Color)>) {
    r.fill_quad(area, Some(&Background::Linear { angle: angulo, stops: paradas }), Border::default(), None);
}

fn cheio(r: &mut Renderer, area: Rect, c: Color) {
    r.fill_quad(area, Some(&c.into()), Border::default(), None);
}

fn marcador(r: &mut Renderer, p: Point) {
    if let Some(c) = Path::circle(p, 6.0) {
        r.stroke_path(&c, &Stroke::new(3.0), Color::BLACK);
        r.stroke_path(&c, &Stroke::new(1.5), Color::WHITE);
    }
}

/// A alça das barras: faixa branca com fio preto.
fn alca(r: &mut Renderer, area: Rect) {
    r.fill_quad(area, Some(&Color::WHITE.into()), Border::default().color(Color::BLACK, 1.0), None);
}

/// O quadrado: saturação para a direita, brilho para cima.
struct Quadrado {
    alvo: AlvoCor,
    hsva: [f32; 4],
}

impl Program<FaixaMensagem> for Quadrado {
    type State = bool;

    fn update(&self, estado: &mut bool, ev: &Event, b: Rect, c: Option<Point>, publicar: &mut dyn FnMut(FaixaMensagem)) -> Captured {
        match arrasto(estado, ev, b, c) {
            Some((x, y)) => {
                publicar(FaixaMensagem::SeletorSV(self.alvo, x, 1.0 - y));
                Captured::Yes
            }
            None => Captured::No,
        }
    }

    fn draw(&self, _: &bool, r: &mut Renderer, b: Rect, _: &Theme, _: Option<Point>) {
        let [vr, vg, vb] = hsv_para_rgb(self.hsva[0], 1.0, 1.0);
        degrade(r, b, 0.0, vec![(0.0, Color::WHITE), (1.0, Color::rgb8(vr, vg, vb))]);
        degrade(r, b, FRAC_PI_2, vec![(0.0, Color::TRANSPARENT), (1.0, Color::BLACK)]);
        marcador(r, Point::new(b.x + self.hsva[1] * b.width, b.y + (1.0 - self.hsva[2]) * b.height));
    }
}

/// A barra vertical do matiz (as cores do arco-íris).
struct BarraMatiz {
    alvo: AlvoCor,
    hsva: [f32; 4],
}

impl Program<FaixaMensagem> for BarraMatiz {
    type State = bool;

    fn update(&self, estado: &mut bool, ev: &Event, b: Rect, c: Option<Point>, publicar: &mut dyn FnMut(FaixaMensagem)) -> Captured {
        match arrasto(estado, ev, b, c) {
            Some((_, y)) => {
                publicar(FaixaMensagem::SeletorMatiz(self.alvo, (y * 360.0).min(359.99)));
                Captured::Yes
            }
            None => Captured::No,
        }
    }

    fn draw(&self, _: &bool, r: &mut Renderer, b: Rect, _: &Theme, _: Option<Point>) {
        let paradas = (0..=6)
            .map(|i| {
                let [vr, vg, vb] = hsv_para_rgb(i as f32 * 60.0, 1.0, 1.0);
                (i as f32 / 6.0, Color::rgb8(vr, vg, vb))
            })
            .collect();
        degrade(r, b, FRAC_PI_2, paradas);
        let y = (self.hsva[0] / 360.0 * b.height - 2.0).max(0.0);
        alca(r, Rect::new(Point::new(b.x, b.y + y), Size::new(b.width, 4.0)));
    }
}

/// A barra horizontal de transparência, sobre um xadrez.
struct BarraAlfa {
    alvo: AlvoCor,
    hsva: [f32; 4],
}

impl Program<FaixaMensagem> for BarraAlfa {
    type State = bool;

    fn update(&self, estado: &mut bool, ev: &Event, b: Rect, c: Option<Point>, publicar: &mut dyn FnMut(FaixaMensagem)) -> Captured {
        match arrasto(estado, ev, b, c) {
            Some((x, _)) => {
                publicar(FaixaMensagem::SeletorAlfa(self.alvo, x));
                Captured::Yes
            }
            None => Captured::No,
        }
    }

    fn draw(&self, _: &bool, r: &mut Renderer, b: Rect, _: &Theme, _: Option<Point>) {
        xadrez(r, b);
        let [vr, vg, vb] = hsv_para_rgb(self.hsva[0], self.hsva[1], self.hsva[2]);
        let c1 = Color::rgb8(vr, vg, vb);
        degrade(r, b, 0.0, vec![(0.0, c1.alpha(0.0)), (1.0, c1)]);
        let x = (self.hsva[3] * b.width - 2.0).max(0.0);
        alca(r, Rect::new(Point::new(b.x + x, b.y), Size::new(4.0, b.height)));
    }
}

/// O xadrez cinza e branco que mostra o que é transparente.
fn xadrez(r: &mut Renderer, b: Rect) {
    let lado = 6.0;
    cheio(r, b, Color::WHITE);
    let (colunas, linhas) = ((b.width / lado).ceil() as i32, (b.height / lado).ceil() as i32);
    for l in 0..linhas {
        for c in 0..colunas {
            if (l + c) % 2 == 1 {
                let w = lado.min(b.width - c as f32 * lado);
                let h = lado.min(b.height - l as f32 * lado);
                cheio(r, Rect::new(Point::new(b.x + c as f32 * lado, b.y + l as f32 * lado), Size::new(w, h)), Color::rgb8(0xcc, 0xcc, 0xcc));
            }
        }
    }
}

/// A prévia da cor atual (com xadrez por baixo se tem transparência).
struct Previa {
    cor: Color,
}

impl<M> Program<M> for Previa {
    type State = ();

    fn draw(&self, _: &(), r: &mut Renderer, b: Rect, _: &Theme, _: Option<Point>) {
        xadrez(r, b);
        cheio(r, b, self.cor);
    }
}

// ------------------------------------------------------------------ o menu

const PRONTAS: [&str; 12] = [
    "#000000", "#666666", "#cccccc", "#ffffff", "#d93025", "#f29900", "#fbbc04", "#34a853", "#1a73e8", "#0b2545", "#9334e6", "#e52592",
];

/// O menu por cima da tela. Clicar fora fecha, menos com o conta-gotas ligado (o clique então vai
/// para o slide). `atual` é a cor do elemento (vazia = sem cor); `hsva` é o estado do seletor.
pub fn sobrepor<'a>(
    base: Element<'a, Mensagem>,
    alvo: AlvoCor,
    atual: &str,
    hsva: [f32; 4],
    texto_hex: &str,
    conta_gotas: bool,
    largura_janela: f32,
) -> Element<'a, Mensagem> {
    const LARGURA: f32 = 264.0;
    let (nome, titulo) = match alvo {
        AlvoCor::Preenchimento => ("cor_preenchimento", "Preenchimento"),
        AlvoCor::Contorno => ("cor_contorno", "Contorno"),
        AlvoCor::Texto => ("cor_texto", "Cor do texto"),
        AlvoCor::Realce => ("cor_realce", "Fundo do texto"),
        AlvoCor::Fundo => ("cor_fundo_imagem", "Cor do fundo a remover"),
    };
    let (esquerda, topo) = crate::medido::ancora(nome, LARGURA, largura_janela, (200.0, 128.0));
    let rgba = rgba_de(atual);

    let quadrado = program(Quadrado { alvo, hsva }).width(Length::FILL).height(Length::Fixed(150.0));
    let matiz = program(BarraMatiz { alvo, hsva }).width(Length::Fixed(16.0)).height(Length::Fixed(150.0));
    let alfa = program(BarraAlfa { alvo, hsva }).width(Length::FILL).height(Length::Fixed(14.0));
    let previa_cor = if atual.is_empty() { Color::TRANSPARENT } else { cor(atual) };
    let previa = program(Previa { cor: previa_cor }).width(Length::Fixed(34.0)).height(Length::Fixed(34.0));

    let gota = button(icone_conta_gotas(18.0, if conta_gotas { "#1a73e8" } else { "#5f6368" }))
        .padding(6)
        .style(estilo_botao_pequeno(conta_gotas))
        .on_press(FaixaMensagem::ConteGotas(alvo));

    let campo = |rotulo: &'static str, i: usize| -> Element<'a, FaixaMensagem> {
        column![
            text_input("", rgba[i].to_string()).on_input(move |t| FaixaMensagem::TextoCanal(alvo, i, t)).size(12).padding([3, 4]).width(Length::FILL),
            text(rotulo).size(10).color(COR_TEXTO_MUTED()),
        ]
        .spacing(2)
        .align(Align::Center)
        .width(Length::FILL)
        .into()
    };

    let prontas = row(PRONTAS.iter().map(|hex| {
        let c = cor(hex);
        let escolhida = atual.eq_ignore_ascii_case(hex);
        Element::from(
            button(Space::new(Length::Fixed(16.0), Length::Fixed(16.0)))
                .padding(0)
                .style(move |_, _| Style {
                    background: Some(Background::Color(c)),
                    border: Border { color: if escolhida { COR_AZUL_ACENTO() } else { COR_BORDA() }, width: if escolhida { 2.0 } else { 1.0 }, radius: [8.0; 4] },
                    ..Style::default()
                })
                .on_press(FaixaMensagem::CorEscolhida(alvo, hex.to_string())),
        )
    }))
    .spacing(4);

    let conteudo = column![
        text(titulo).size(12).color(COR_TEXTO_MUTED()),
        row![quadrado, matiz].spacing(8).height(Length::Fixed(150.0)),
        alfa,
        row![gota, previa, Space::new(Length::FILL, 0.0)].spacing(10).align(Align::Center),
        row![
            column![
                text_input("#rrggbb", texto_hex.to_string()).on_input(move |t| FaixaMensagem::TextoCor(alvo, t)).size(12).padding([3, 4]).width(Length::FILL),
                text("Hex").size(10).color(COR_TEXTO_MUTED()),
            ]
            .spacing(2)
            .align(Align::Center)
            .width(Length::Fill(3)),
            campo("R", 0),
            campo("G", 1),
            campo("B", 2),
            campo("A", 3),
        ]
        .spacing(6),
        prontas,
        row![
            button(text("Sem cor").size(12)).padding([3, 8]).style(estilo_botao_pequeno(atual.is_empty())).on_press(FaixaMensagem::CorEscolhida(alvo, String::new())),
            Space::new(Length::FILL, 0.0),
            button(text("Fechar").size(12)).padding([3, 8]).style(estilo_botao_pequeno(false)).on_press(FaixaMensagem::FecharSeletorCor),
        ],
    ]
    .spacing(10);

    let painel = container(Element::from(conteudo).map(Mensagem::Faixa))
        .padding(12)
        .width(Length::Fixed(LARGURA))
        .style(|_| Style {
            background: Some(Background::Color(COR_PAINEL())),
            text_color: Some(COR_TEXTO()),
            border: Border { color: COR_BORDA(), width: 1.0, radius: [10.0; 4] },
            shadow: Some(Shadow { color: Color::rgb8(0, 0, 0).alpha(0.3), offset: Point::new(0.0, 6.0), blur: 18.0 }), ..Style::default() });
    let sobreposto = container(painel).padding(pelicano::Padding { top: topo, left: esquerda, right: 0.0, bottom: 0.0 });
    if conta_gotas {
        pelicano::stack![base, sobreposto].into()
    } else {
        let fundo = mouse_area(Space::new(Length::FILL, Length::FILL)).on_press(Mensagem::Faixa(FaixaMensagem::FecharSeletorCor));
        pelicano::stack![base, fundo, sobreposto].into()
    }
}
