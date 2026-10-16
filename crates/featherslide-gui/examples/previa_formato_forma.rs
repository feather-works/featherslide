//! Prévia Iced do painel real. Com `--capturar`, salva PNG em /tmp e fecha.
// A prévia isolada não usa as mensagens de arrasto do palco do aplicativo.
#[allow(dead_code)]
#[path = "../src/formato_forma.rs"]
mod formato_forma;
mod tema {
    pub use feather_ui::tema::*;
}
mod icones {
    pub fn icone_menos(t: f32, _: &str) -> iced::widget::svg::Svg<'static, iced::Theme> {
        iced::widget::svg::Svg::new(iced::widget::svg::Handle::from_memory(b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#5f6368' stroke-width='2'><path d='M5 12h14'/></svg>".as_slice())).width(t).height(t)
    }
}
use iced::widget::{column, container, row, text, Space};
use iced::{Element, Length, Subscription, Task};
#[derive(Clone, Debug)]
enum Msg {
    Painel(formato_forma::Msg),
    Capturar,
    Capturada(iced::window::Screenshot),
}
struct Previa {
    painel: formato_forma::Estado,
    capturando: bool,
    modo: String,
}
impl Previa {
    fn novo() -> Self {
        tema::definir_tema(false, 2);
        let modelo = featherslide_core::modelo::Elemento {
            tipo: featherslide_core::modelo::Tipo::RetanguloArredondado,
            largura: 200.0,
            altura: 140.0,
            preenchimento: "#4285f4".into(),
            contorno: "#1a56c4".into(),
            espessura: 1.5,
            ..Default::default()
        };
        let mut painel = formato_forma::Estado::novo(&modelo);
        let modo = std::env::args()
            .find(|s| s == "gradiente" || s == "sombra")
            .unwrap_or_else(|| "solido".into());
        if modo == "gradiente" {
            painel.tratar(formato_forma::Msg::Tipo("Linear".into()));
            painel.tratar(formato_forma::Msg::Preset(1));
            painel.tratar(formato_forma::Msg::Campo("angulo", "45".into()));
        }
        if modo == "sombra" {
            painel.tratar(formato_forma::Msg::Alternar(formato_forma::Secao::Sombra));
            painel.tratar(formato_forma::Msg::Sombra(true));
        }
        Self {
            painel,
            capturando: false,
            modo,
        }
    }
    fn update(&mut self, m: Msg) -> Task<Msg> {
        match m {
            Msg::Painel(m) => self.painel.tratar(m),
            Msg::Capturar => {
                if !self.capturando {
                    self.capturando = true;
                    return iced::window::get_latest()
                        .and_then(iced::window::screenshot)
                        .map(Msg::Capturada);
                }
            }
            Msg::Capturada(s) => {
                let largura = (formato_forma::LARGURA * s.scale_factor as f32).round() as u32;
                let painel = s
                    .crop(iced::Rectangle {
                        x: s.size.width - largura,
                        y: 0,
                        width: largura,
                        height: s.size.height,
                    })
                    .unwrap();
                image::save_buffer(
                    format!("/tmp/featherslide-painel-{}.png", self.modo),
                    &painel.bytes,
                    painel.size.width,
                    painel.size.height,
                    image::ColorType::Rgba8,
                )
                .unwrap();
                image::save_buffer(
                    format!("/tmp/featherslide-formato-{}.png", self.modo),
                    &s.bytes,
                    s.size.width,
                    s.size.height,
                    image::ColorType::Rgba8,
                )
                .unwrap();
                return iced::exit();
            }
        }
        Task::none()
    }
    fn view(&self) -> Element<'_, Msg> {
        let palco = container(
            column![
                text("FeatherSlide").size(26),
                text("Prévia do painel nativo")
                    .size(14)
                    .color(tema::COR_TEXTO_MUTED()),
                Space::with_height(24),
                container(
                    column![
                        text("O slide continua visível").size(26),
                        text("O painel ocupa uma coluna do editor.").size(14)
                    ]
                    .spacing(12)
                )
                .padding(34)
                .width(Length::Fill)
                .height(350)
                .style(|_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(tema::COR_PAINEL())),
                    border: iced::Border {
                        color: tema::COR_BORDA(),
                        width: 1.0,
                        ..Default::default()
                    },
                    ..Default::default()
                })
            ]
            .spacing(10),
        )
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(tema::COR_FUNDO_JANELA())),
            ..Default::default()
        });
        row![palco, formato_forma::view(&self.painel).map(Msg::Painel)]
            .height(Length::Fill)
            .into()
    }
    fn subscription(&self) -> Subscription<Msg> {
        if std::env::args().any(|a| a == "--capturar") && !self.capturando {
            iced::time::every(std::time::Duration::from_secs(2)).map(|_| Msg::Capturar)
        } else {
            Subscription::none()
        }
    }
}
fn main() -> iced::Result {
    iced::application("Prévia · Formato da forma", Previa::update, Previa::view)
        .theme(|_| tema::tema_iced())
        .subscription(Previa::subscription)
        .window_size(iced::Size::new(1000.0, 760.0))
        .antialiasing(true)
        .run_with(|| (Previa::novo(), Task::none()))
}
