//! O seletor de cor: um quadrado de saturação/brilho, a barra de matiz, a barra de
//! transparência, o conta-gotas (pega a cor de um ponto do slide) e os campos R, G, B, A e hex.
//! Abre embaixo do botão da cor; cada mudança já vale no elemento.

use iced::mouse;
use iced::widget::canvas::{self, event, gradient, Canvas, Fill, Frame, Geometry, Gradient, Path, Program, Stroke};
use iced::widget::{button, column, container, mouse_area, row, text, text_input, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Point, Rectangle, Renderer, Shadow, Size, Theme, Vector};

use crate::app::Mensagem;
use crate::faixa::{AlvoCor, FaixaMensagem};
use crate::icones::icone_conta_gotas;
use crate::tema::*;

// ------------------------------------------------------------------ conversões

// Conversões extraídas para feather-ui, compartilhadas com FeatherSheet.
pub use feather_ui::seletor_cor::{rgba_de, hex_de, hex_valido, hsv_para_rgb, rgba_para_hsva};

fn cor(hex: &str) -> Color {
    let [r, g, b, a] = rgba_de(hex);
    Color::from_rgba8(r, g, b, a as f32 / 255.0)
}

// ------------------------------------------------------------------ as três áreas de arrastar

/// Traduz o arrasto do mouse na área em (x, y) de 0 a 1; `None` se o evento não conta.
fn arrasto(arrastando: &mut bool, ev: &canvas::Event, b: Rectangle, c: mouse::Cursor) -> Option<(f32, f32)> {
    let norm = |p: Point| (((p.x - b.x) / b.width).clamp(0.0, 1.0), ((p.y - b.y) / b.height).clamp(0.0, 1.0));
    match ev {
        canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if c.position_in(b).is_some() => {
            *arrastando = true;
            c.position().map(norm)
        }
        canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if *arrastando => c.position().map(norm),
        canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
            *arrastando = false;
            None
        }
        _ => None,
    }
}

fn degrade_horizontal(w: f32, paradas: &[(f32, Color)]) -> Fill {
    let mut l = gradient::Linear::new(Point::new(0.0, 0.0), Point::new(w, 0.0));
    for (o, c) in paradas {
        l = l.add_stop(*o, *c);
    }
    Fill::from(Gradient::Linear(l))
}

fn degrade_vertical(h: f32, paradas: &[(f32, Color)]) -> Fill {
    let mut l = gradient::Linear::new(Point::new(0.0, 0.0), Point::new(0.0, h));
    for (o, c) in paradas {
        l = l.add_stop(*o, *c);
    }
    Fill::from(Gradient::Linear(l))
}

fn marcador(frame: &mut Frame, p: Point) {
    frame.stroke(&Path::circle(p, 6.0), Stroke::default().with_color(Color::BLACK).with_width(3.0));
    frame.stroke(&Path::circle(p, 6.0), Stroke::default().with_color(Color::WHITE).with_width(1.5));
}

/// O quadrado: saturação para a direita, brilho para cima.
struct Quadrado {
    alvo: AlvoCor,
    hsva: [f32; 4],
}

impl Program<FaixaMensagem> for Quadrado {
    type State = bool;

    fn update(&self, estado: &mut bool, ev: canvas::Event, b: Rectangle, c: mouse::Cursor) -> (event::Status, Option<FaixaMensagem>) {
        match arrasto(estado, &ev, b, c) {
            Some((x, y)) => (event::Status::Captured, Some(FaixaMensagem::SeletorSV(self.alvo, x, 1.0 - y))),
            None => (event::Status::Ignored, None),
        }
    }

    fn draw(&self, _: &bool, renderer: &Renderer, _: &Theme, b: Rectangle, _: mouse::Cursor) -> Vec<Geometry> {
        let mut f = Frame::new(renderer, b.size());
        let [r, g, bl] = hsv_para_rgb(self.hsva[0], 1.0, 1.0);
        let pura = Color::from_rgb8(r, g, bl);
        f.fill_rectangle(Point::ORIGIN, b.size(), degrade_horizontal(b.width, &[(0.0, Color::WHITE), (1.0, pura)]));
        f.fill_rectangle(Point::ORIGIN, b.size(), degrade_vertical(b.height, &[(0.0, Color::TRANSPARENT), (1.0, Color::BLACK)]));
        marcador(&mut f, Point::new(self.hsva[1] * b.width, (1.0 - self.hsva[2]) * b.height));
        vec![f.into_geometry()]
    }
}

/// A barra vertical do matiz (as cores do arco-íris).
struct BarraMatiz {
    alvo: AlvoCor,
    hsva: [f32; 4],
}

impl Program<FaixaMensagem> for BarraMatiz {
    type State = bool;

    fn update(&self, estado: &mut bool, ev: canvas::Event, b: Rectangle, c: mouse::Cursor) -> (event::Status, Option<FaixaMensagem>) {
        match arrasto(estado, &ev, b, c) {
            Some((_, y)) => (event::Status::Captured, Some(FaixaMensagem::SeletorMatiz(self.alvo, (y * 360.0).min(359.99)))),
            None => (event::Status::Ignored, None),
        }
    }

    fn draw(&self, _: &bool, renderer: &Renderer, _: &Theme, b: Rectangle, _: mouse::Cursor) -> Vec<Geometry> {
        let mut f = Frame::new(renderer, b.size());
        let paradas: Vec<(f32, Color)> = (0..=6)
            .map(|i| {
                let [r, g, bl] = hsv_para_rgb(i as f32 * 60.0, 1.0, 1.0);
                (i as f32 / 6.0, Color::from_rgb8(r, g, bl))
            })
            .collect();
        f.fill_rectangle(Point::ORIGIN, b.size(), degrade_vertical(b.height, &paradas));
        let y = self.hsva[0] / 360.0 * b.height;
        f.fill_rectangle(Point::new(0.0, (y - 2.0).max(0.0)), Size::new(b.width, 4.0), Color::WHITE);
        f.stroke(&Path::rectangle(Point::new(0.5, (y - 2.0).max(0.0)), Size::new(b.width - 1.0, 4.0)), Stroke::default().with_color(Color::BLACK).with_width(1.0));
        vec![f.into_geometry()]
    }
}

/// A barra horizontal de transparência, sobre um xadrez.
struct BarraAlfa {
    alvo: AlvoCor,
    hsva: [f32; 4],
}

impl Program<FaixaMensagem> for BarraAlfa {
    type State = bool;

    fn update(&self, estado: &mut bool, ev: canvas::Event, b: Rectangle, c: mouse::Cursor) -> (event::Status, Option<FaixaMensagem>) {
        match arrasto(estado, &ev, b, c) {
            Some((x, _)) => (event::Status::Captured, Some(FaixaMensagem::SeletorAlfa(self.alvo, x))),
            None => (event::Status::Ignored, None),
        }
    }

    fn draw(&self, _: &bool, renderer: &Renderer, _: &Theme, b: Rectangle, _: mouse::Cursor) -> Vec<Geometry> {
        let mut f = Frame::new(renderer, b.size());
        xadrez(&mut f, b.size());
        let [r, g, bl] = hsv_para_rgb(self.hsva[0], self.hsva[1], self.hsva[2]);
        let (c0, c1) = (Color::from_rgba8(r, g, bl, 0.0), Color::from_rgb8(r, g, bl));
        f.fill_rectangle(Point::ORIGIN, b.size(), degrade_horizontal(b.width, &[(0.0, c0), (1.0, c1)]));
        let x = self.hsva[3] * b.width;
        f.fill_rectangle(Point::new((x - 2.0).max(0.0), 0.0), Size::new(4.0, b.height), Color::WHITE);
        f.stroke(&Path::rectangle(Point::new((x - 2.0).max(0.0), 0.5), Size::new(4.0, b.height - 1.0)), Stroke::default().with_color(Color::BLACK).with_width(1.0));
        vec![f.into_geometry()]
    }
}

/// O xadrez cinza e branco que mostra o que é transparente.
fn xadrez(f: &mut Frame, tamanho: Size) {
    let lado = 6.0;
    f.fill_rectangle(Point::ORIGIN, tamanho, Color::WHITE);
    let (colunas, linhas) = ((tamanho.width / lado).ceil() as i32, (tamanho.height / lado).ceil() as i32);
    for l in 0..linhas {
        for c in 0..colunas {
            if (l + c) % 2 == 1 {
                let w = lado.min(tamanho.width - c as f32 * lado);
                let h = lado.min(tamanho.height - l as f32 * lado);
                f.fill_rectangle(Point::new(c as f32 * lado, l as f32 * lado), Size::new(w, h), Color::from_rgb8(0xcc, 0xcc, 0xcc));
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

    fn draw(&self, _: &(), renderer: &Renderer, _: &Theme, b: Rectangle, _: mouse::Cursor) -> Vec<Geometry> {
        let mut f = Frame::new(renderer, b.size());
        xadrez(&mut f, b.size());
        f.fill_rectangle(Point::ORIGIN, b.size(), self.cor);
        vec![f.into_geometry()]
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

    let quadrado = Canvas::new(Quadrado { alvo, hsva }).width(Length::Fill).height(Length::Fixed(150.0));
    let matiz = Canvas::new(BarraMatiz { alvo, hsva }).width(Length::Fixed(16.0)).height(Length::Fixed(150.0));
    let alfa = Canvas::new(BarraAlfa { alvo, hsva }).width(Length::Fill).height(Length::Fixed(14.0));
    let previa_cor = if atual.is_empty() { Color::TRANSPARENT } else { cor(atual) };
    let previa = Canvas::new(Previa { cor: previa_cor }).width(Length::Fixed(34.0)).height(Length::Fixed(34.0));

    let gota = button(icone_conta_gotas(18.0, if conta_gotas { "#1a73e8" } else { "#5f6368" }))
        .padding(6)
        .style(estilo_botao_pequeno(conta_gotas))
        .on_press(FaixaMensagem::ConteGotas(alvo));

    let campo = |rotulo: &'static str, i: usize| -> Element<'a, FaixaMensagem> {
        column![
            text_input("", &rgba[i].to_string()).on_input(move |t| FaixaMensagem::TextoCanal(alvo, i, t)).size(12).padding([3, 4]).width(Length::Fill),
            text(rotulo).size(10).color(COR_TEXTO_MUTED()),
        ]
        .spacing(2)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .into()
    };

    let prontas = row(PRONTAS.iter().map(|hex| {
        let c = cor(hex);
        let escolhida = atual.eq_ignore_ascii_case(hex);
        Element::from(
            button(Space::new(Length::Fixed(16.0), Length::Fixed(16.0)))
                .padding(0)
                .style(move |_, _| button::Style {
                    background: Some(Background::Color(c)),
                    border: Border { color: if escolhida { COR_AZUL_ACENTO() } else { COR_BORDA() }, width: if escolhida { 2.0 } else { 1.0 }, radius: 8.0.into() },
                    ..Default::default()
                })
                .on_press(FaixaMensagem::CorEscolhida(alvo, hex.to_string())),
        )
    }))
    .spacing(4);

    let conteudo = column![
        text(titulo).size(12).color(COR_TEXTO_MUTED()),
        row![quadrado, matiz].spacing(8).height(Length::Fixed(150.0)),
        alfa,
        row![gota, previa, Space::with_width(Length::Fill)].spacing(10).align_y(Alignment::Center),
        row![
            column![
                text_input("#rrggbb", texto_hex).on_input(move |t| FaixaMensagem::TextoCor(alvo, t)).size(12).padding([3, 4]).width(Length::Fill),
                text("Hex").size(10).color(COR_TEXTO_MUTED()),
            ]
            .spacing(2)
            .align_x(Alignment::Center)
            .width(Length::FillPortion(3)),
            campo("R", 0),
            campo("G", 1),
            campo("B", 2),
            campo("A", 3),
        ]
        .spacing(6),
        prontas,
        row![
            button(text("Sem cor").size(12)).padding([3, 8]).style(estilo_botao_pequeno(atual.is_empty())).on_press(FaixaMensagem::CorEscolhida(alvo, String::new())),
            Space::with_width(Length::Fill),
            button(text("Fechar").size(12)).padding([3, 8]).style(estilo_botao_pequeno(false)).on_press(FaixaMensagem::FecharSeletorCor),
        ],
    ]
    .spacing(10);

    let painel = container(Element::from(conteudo).map(Mensagem::Faixa))
        .padding(12)
        .width(Length::Fixed(LARGURA))
        .style(|_| container::Style {
            background: Some(Background::Color(COR_PAINEL())),
            text_color: Some(COR_TEXTO()),
            border: Border { color: COR_BORDA(), width: 1.0, radius: 10.0.into() },
            shadow: Shadow { color: Color::from_rgba8(0, 0, 0, 0.3), offset: Vector::new(0.0, 6.0), blur_radius: 18.0 },
        });
    let sobreposto = container(painel).padding(iced::Padding { top: topo, left: esquerda, right: 0.0, bottom: 0.0 });
    if conta_gotas {
        iced::widget::stack![base, sobreposto].into()
    } else {
        let fundo = mouse_area(Space::new(Length::Fill, Length::Fill)).on_press(Mensagem::Faixa(FaixaMensagem::FecharSeletorCor));
        iced::widget::stack![base, fundo, sobreposto].into()
    }
}
