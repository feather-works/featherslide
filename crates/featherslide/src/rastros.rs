//! Rastros no palco: arrastando com o mouse (forma girada, com sombra e contorno grosso, e o
//! retângulo de seleção), o quadro redesenhado só nas regiões que mudaram tem de sair igual ao
//! quadro pintado do zero.
use crate::app::{FeatherSlideApp, Mensagem};
use featherslide_core::modelo::{Sombra, Tipo};
use pelicano::{widget, Changes, Event, EventCx, MemoryClipboard, MouseButton, Point, Renderer, Size, States, WindowId};

const TAM: Size = Size { width: 1280.0, height: 800.0 };

struct Tela {
    app: FeatherSlideApp,
    r: Renderer,
    st: States,
    area: MemoryClipboard,
    quadros: usize,
    escala: f32,
}

impl Tela {
    /// Entrega `ev` com o mouse em `p` e passa as mensagens ao app.
    fn evento(&mut self, p: Point, ev: Event) {
        let tema = crate::tema::tema();
        let msgs: Vec<Mensagem> = {
            let el = self.app.view(WindowId::MAIN);
            let n = widget::layout(&el, TAM, &mut self.r, &self.st, &tema);
            let mut msgs = Vec::new();
            let mut cx = EventCx { renderer: &mut self.r, states: &mut self.st, cursor: Some(p), messages: &mut msgs, window: &mut Vec::new(), redraw: false, clipboard: &mut self.area };
            widget::dispatch(&el, &n, &ev, &mut cx);
            msgs
        };
        for m in msgs {
            let _ = self.app.update(m);
        }
        self.quadro(p);
    }

    /// Um quadro como a janela faz (grava, compara, redesenha só as regiões) e a conferência
    /// com o mesmo quadro pintado do zero.
    fn quadro(&mut self, p: Point) {
        let tema = crate::tema::tema();
        // O relógio das transições anda uma vez por quadro, como na janela.
        self.st.tick(std::time::Instant::now());
        let el = self.app.view(WindowId::MAIN);
        let n = widget::layout(&el, TAM, &mut self.r, &self.st, &tema);
        self.r.record();
        widget::draw(&el, &n, &mut self.r, &self.st, &tema, Some(p), false);
        let ok = match self.r.changes() {
            Changes::Regions(rs) if self.quadros > 0 => rs.iter().all(|s| self.r.begin_parcial(TAM, self.escala, tema.background, *s) && {
                widget::draw(&el, &n, &mut self.r, &self.st, &tema, Some(p), false);
                self.r.end();
                true
            }),
            Changes::None if self.quadros > 0 => true,
            _ => false,
        };
        if !ok {
            self.r.begin(TAM, self.escala, tema.background);
            widget::draw(&el, &n, &mut self.r, &self.st, &tema, Some(p), false);
            self.r.end();
        }
        let mut novo = Renderer::new();
        novo.begin(TAM, self.escala, tema.background);
        let n2 = widget::layout(&el, TAM, &mut novo, &self.st, &tema);
        widget::draw(&el, &n2, &mut novo, &self.st, &tema, Some(p), false);
        novo.end();
        let (a, b) = (self.r.pixmap.data(), novo.pixmap.data());
        // Até 2 é o arredondamento de pintar a forma por máscara no quadro parcial (o `preencher_parcial`
        // do pelicano); rastro de verdade é o desenho velho que ficou, bem mais que isso.
        let ruins: Vec<usize> = (0..a.len() / 4).filter(|i| (0..4).any(|c| a[i * 4 + c].abs_diff(b[i * 4 + c]) > 2)).collect();
        if let (Some(i), Some(u)) = (ruins.first(), ruins.last()) {
            let w = self.r.pixmap.width() as usize;
            let (x0, y0) = (ruins.iter().map(|i| i % w).min().unwrap(), i / w);
            let (x1, y1) = (ruins.iter().map(|i| i % w).max().unwrap(), u / w);
            if let Some(d) = std::env::var_os("RASTRO_PNG") {
                let d = std::path::PathBuf::from(d);
                let _ = std::fs::write(d.join("parcial.png"), self.r.pixmap.encode_png().unwrap());
                let _ = std::fs::write(d.join("cheio.png"), novo.pixmap.encode_png().unwrap());
            }
            panic!("escala {}, quadro {}: {} pixels de rastro em ({x0}, {y0})–({x1}, {y1}) px", self.escala, self.quadros, ruins.len());
        }
        self.quadros += 1;
    }

    /// O centro dos pixels do palco com a cor `rgb` (a forma; à esquerda estão as miniaturas).
    fn achar(&self, rgb: [u8; 3]) -> Point {
        let w = self.r.pixmap.width() as usize;
        let d = self.r.pixmap.data();
        let px: Vec<usize> = (0..d.len() / 4).filter(|i| i % w > 300 && d[i * 4..i * 4 + 3] == rgb).collect();
        assert!(!px.is_empty(), "forma na tela");
        let n = px.len() as f32;
        Point::new(px.iter().map(|i| (i % w) as f32).sum::<f32>() / n / self.escala, px.iter().map(|i| (i / w) as f32).sum::<f32>() / n / self.escala)
    }

    fn arrastar(&mut self, de: Point, passo: Point, n: usize) {
        self.evento(de, Event::CursorMoved(de));
        self.evento(de, Event::MousePressed(MouseButton::Left));
        let mut p = de;
        for _ in 0..n {
            p = Point::new(p.x + passo.x, p.y + passo.y);
            self.evento(p, Event::CursorMoved(p));
        }
        self.evento(p, Event::MouseReleased(MouseButton::Left));
    }
}

#[test]
fn arrastar_no_palco_nao_deixa_rastro() {
    for escala in [1.0, 1.25, 1.5] {
        arrastar_na_escala(escala);
    }
}

fn arrastar_na_escala(escala: f32) {
    let mut app = FeatherSlideApp::default();
    app.config.ortografia_automatica = false;
    app.sessao.slide_novo("em_branco").unwrap();
    let sombra = Sombra { cor: "#00000080".into(), desfoque: 12.0, dx: 6.0, dy: 8.0 };
    for (x, cor) in [(200.0, "#1e78dc"), (520.0, "#28b450")] {
        app.sessao
            .elemento_inserir_com_estilo(Tipo::Retangulo, Some([x, 150.0, 160.0, 90.0]), "Texto".into(), None, Some(cor.into()), Some("#c82828".into()), Some(6.0), Some(sombra.clone()), vec![])
            .unwrap();
        let id = app.sessao.slide().elementos.last().unwrap().id;
        app.sessao.elemento_rotacao(id, 20.0).unwrap();
    }
    let mut t = Tela { app, r: Renderer::new(), st: States::default(), area: MemoryClipboard::default(), quadros: 0, escala };
    // A janela já tem a escala do quadro anterior quando faz o layout.
    t.r.begin(TAM, escala, pelicano::Color::BLACK);
    t.quadro(Point::ORIGIN);
    // Pega a forma azul e arrasta (com a prévia e as guias de alinhamento).
    let azul = t.achar([0x1e, 0x78, 0xdc]);
    let x0 = t.app.sessao.slide().elementos[0].x;
    t.arrastar(azul, Point::new(14.0, 6.0), 8);
    assert!(t.app.sessao.slide().elementos[0].x > x0 + 50.0, "a forma não andou");
    // Retângulo de seleção no vazio, de baixo para cima.
    let verde = t.achar([0x28, 0xb4, 0x50]);
    t.arrastar(Point::new(verde.x + 200.0, verde.y + 150.0), Point::new(-50.0, -36.0), 7);
    assert!(t.app.sessao.estado().selecao.len() >= 1, "o retângulo não selecionou");
}
