//! O palco de desenho do slide atual no Iced: Canvas interativo com suporte
//! a seleção, arrasto e renderização de todas as formas e textos do motor.

use std::time::Instant;

use featherslide_core::quadro::Quadro;
use iced::keyboard;
use iced::mouse;
use iced::widget::canvas::{self, event, Frame, Geometry, Path, Program, Stroke};
use iced::{Vector, Color, Point, Rectangle, Renderer, Size, Theme};

use crate::desenho::{pintar_quadro, posicoes_alcas_giradas};

thread_local! {
    /// A opacidade com que se pinta (efeitos de entrada e transições): multiplica o alfa de toda cor lida.
    pub static OPACIDADE: std::cell::Cell<f32> = const { std::cell::Cell::new(1.0) };
}

/// Converte cor hexadecimal (`#rrggbb` ou `#rrggbbaa`) para `iced::Color`.
pub fn cor_hex(hex: &str) -> Option<Color> {
    let s = hex.trim().trim_start_matches('#');
    match s.len() {
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Color::from_rgba8(r, g, b, OPACIDADE.with(|o| o.get())))
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            let a = u8::from_str_radix(&s[6..8], 16).ok()?;
            Some(Color::from_rgba8(r, g, b, a as f32 / 255.0 * OPACIDADE.with(|o| o.get())))
        }
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum PalcoMensagem {
    GradientePonto {indice:usize,posicao:f32},
    GradienteAngulo(f32),
    DesenhoLivre(Vec<[f32;2]>),
    CancelarDesenho,
    Clicou { x: f32, y: f32, somar: bool, duplo: bool },
    Arrastou { dx: f32, dy: f32 },
    /// Arrasto de enquadramento da foto dentro da forma de corte.
    ArrastouRecorte { id: u64, dx: f32, dy: f32 },
    /// Zoom da foto dentro da forma de corte pela roda do mouse.
    ZoomRecorte { id: u64, delta: f32 },
    /// Seleção por área: retângulo em pontos do slide.
    Area { x: f32, y: f32, w: f32, h: f32, somar: bool },
    /// Terminou de puxar uma alça: o retângulo novo do elemento.
    /// Ctrl + roda do mouse: positivo aproxima.
    Zoom(f32),
    /// Botão direito: o ponto no slide (pontos) e na janela (pixels).
    MenuContexto { x: f32, y: f32, janela: Point },
    Redimensionou { id: u64, x: f32, y: f32, w: f32, h: f32 },
    /// Soltou aqui uma miniatura arrastada: o slide e o ponto no slide (pontos).
    /// Terminou de girar um elemento: o ângulo novo em graus (horário, 0 = para cima).
    Girou { id: u64, graus: f32 },
    /// Mover a tela com zoom: o deslocamento novo, em pixels.
    Pan { x: f32, y: f32 },
    /// Terminou de puxar a alça de um canto: os quatro raios novos (fração do menor lado).
    Cantos { raios: [f32; 4] },
    SoltouSlide { slide: usize, x: f32, y: f32 },
    /// Apertou o botão dentro da caixa em edição: põe o cursor (ou seleciona a palavra, se `duplo`).
    TextoPressionou { x: f32, y: f32, duplo: bool },
    /// Arrastando com o botão apertado: estende a seleção até este ponto.
    TextoArrastou { x: f32, y: f32 },
}

#[derive(Default)]
pub struct PalcoEstado {
    traco_livre: Vec<Point>,
    inicio_arrasto: Option<Point>,
    ultimo_ponto: Option<Point>,
    andou: bool,
    somar: bool,
    ctrl: bool,
    shift: bool,
    ultimo_clique: Option<(Instant, Point)>,
    /// O arrasto começou no vazio: desenha e seleciona por área em vez de mover.
    area: bool,
    /// Puxando uma alça: o elemento, qual das oito e o retângulo de antes.
    alca: Option<(u64, usize, [f32; 4])>,
    /// Girando pela alça de rotação: o elemento e o centro dele (pixels da área).
    girando: Option<(u64, Point)>,
    /// Arrastando a tela com o botão do meio: onde o cursor estava (janela) e o `pan` de então.
    pan_arrasto: Option<(Point, (f32, f32))>,
    /// Puxando a alça de um canto arredondado: o elemento, o canto (0 a 3) e os raios de antes.
    canto_arrasto: Option<(u64, usize, [f32; 4])>,
    /// Arrastando o enquadramento/recorte da imagem (foto dentro da máscara).
    recorte_arrasto: Option<(u64, Point)>,
    /// Selecionando texto com o mouse dentro da caixa em edição.
    texto_arrasto: bool,
    gradiente_arrasto: Option<usize>,
}

pub struct PalcoPrograma {
    pub controle_gradiente: Option<crate::formato_forma::ControleGradiente>,
    pub desenho_livre: bool,
    pub conectando: bool,
    pub quadro: Option<Quadro>,
    pub selecao: Vec<u64>,
    /// O elemento em edição de texto: ganha o cursor piscando no fim do texto.
    pub editando: Option<u64>,
    /// O texto em edição e onde está a barrinha nele (em bytes).
    pub texto_edicao: String,
    pub cursor_edicao: usize,
    /// O trecho selecionado do texto em edição (bytes), se houver.
    pub selecao_texto: Option<(usize, usize)>,
    /// O elemento com ajuste de enquadramento ativo no palco.
    pub ajustando_recorte: Option<u64>,
    /// "Selecionar região" do remover fundo: arrastar sempre desenha a região (não move a imagem).
    pub pegando_cor: bool,
    /// Os retângulos (x, y, w, h) dos elementos que se pegam com o mouse (sem o fundo do layout).
    pub caixas: Vec<[f32; 4]>,
    /// A célula de tabela em edição (retângulo em pontos do slide): leva contorno.
    pub celula: Option<[f32; 4]>,
    /// Grade (a cada 48 pt) e guias do centro desenhadas sobre o slide.
    pub grade: bool,
    pub guias: bool,
    /// Réguas em cm nas bordas de cima e da esquerda.
    pub regua: bool,
    /// Trechos a sublinhar com ondulação: x, y (linha de base), largura e tamanho da fonte (pontos do slide) e a cor (0 vermelho = ortografia, 1 azul = gramática).
    pub ondulados: Vec<[f32; 5]>,
    pub zoom: f32,
    /// O slide da miniatura que está sendo arrastada (vira gatilho ao soltar aqui).
    pub arrastando_slide: Option<usize>,
    /// O quanto o slide foi empurrado (pixels) para ver outra parte com zoom.
    pub pan: (f32, f32),
    /// Os ids dos elementos de `caixas` (mesma ordem), para o encaixe ignorar o que está sendo movido.
    pub ids_caixas: Vec<u64>,
}

/// O retângulo depois de puxar a alça `alca` por (`dx`, `dy`) pontos.
fn retangulo_novo(orig: [f32; 4], alca: usize, dx: f32, dy: f32) -> [f32; 4] {
    let [x, y, w, h] = orig;
    let (mut x0, mut y0, mut x1, mut y1) = (x, y, x + w, y + h);
    if matches!(alca, 0 | 6 | 7) {
        x0 += dx;
    }
    if matches!(alca, 2 | 3 | 4) {
        x1 += dx;
    }
    if matches!(alca, 0 | 1 | 2) {
        y0 += dy;
    }
    if matches!(alca, 4 | 5 | 6) {
        y1 += dy;
    }
    const MIN: f32 = 8.0;
    if x1 - x0 < MIN {
        if matches!(alca, 0 | 6 | 7) { x0 = x1 - MIN } else { x1 = x0 + MIN }
    }
    if y1 - y0 < MIN {
        if matches!(alca, 0 | 1 | 2) { y0 = y1 - MIN } else { y1 = y0 + MIN }
    }
    [x0, y0, x1 - x0, y1 - y0]
}

/// Como `retangulo_novo`, para um elemento girado `rot` graus: o arrasto (na tela) é levado para o eixo
/// do elemento, e o centro novo volta para a tela, de modo que o lado oposto à alça fique parado.
fn retangulo_girado(orig: [f32; 4], alca: usize, dx: f32, dy: f32, rot: f32) -> [f32; 4] {
    if rot == 0.0 {
        return retangulo_novo(orig, alca, dx, dy);
    }
    let (sen, cos) = rot.to_radians().sin_cos();
    let (ldx, ldy) = (dx * cos + dy * sen, -dx * sen + dy * cos);
    let [lx, ly, lw, lh] = retangulo_novo(orig, alca, ldx, ldy);
    let (cx, cy) = (orig[0] + orig[2] / 2.0, orig[1] + orig[3] / 2.0);
    let (ddx, ddy) = (lx + lw / 2.0 - cx, ly + lh / 2.0 - cy);
    let (nx, ny) = (cx + ddx * cos - ddy * sen, cy + ddx * sen + ddy * cos);
    [nx - lw / 2.0, ny - lh / 2.0, lw, lh]
}

/// Shift preserva a proporção no eixo local, inclusive quando o objeto está girado.
fn retangulo_proporcional(orig: [f32; 4], alca: usize, dx: f32, dy: f32, rot: f32, manter: bool) -> [f32; 4] {
    if !manter { return retangulo_girado(orig, alca, dx, dy, rot); }
    let (sen, cos) = rot.to_radians().sin_cos();
    let local = retangulo_novo(orig, alca, dx * cos + dy * sen, -dx * sen + dy * cos);
    let Some(t) = featherslide_core::escala::Escala::entre(orig, local) else { return local };
    let [lx, ly, lw, lh] = t.retangulo(orig);
    let (cx, cy) = (orig[0] + orig[2] / 2.0, orig[1] + orig[3] / 2.0);
    let (ddx, ddy) = (lx + lw / 2.0 - cx, ly + lh / 2.0 - cy);
    [cx + ddx * cos - ddy * sen - lw / 2.0, cy + ddx * sen + ddy * cos - lh / 2.0, lw, lh]
}

/// Os raios dos quatro cantos (↖ ↗ ↘ ↙), em fração do menor lado, de um retângulo.
fn raios_do(e: &featherslide_core::quadro::ElementoDesenhado) -> [f32; 4] {
    if e.tipo == "imagem" {
        let r = e.ajustes.get(3).copied().unwrap_or(0.16);
        return [r, r, r, r];
    }
    if e.ajustes.len() == 4 {
        return [e.ajustes[0], e.ajustes[1], e.ajustes[2], e.ajustes[3]];
    }
    if e.tipo == "retangulo_arredondado" {
        let cima = e.ajustes.first().copied().unwrap_or(0.16);
        let baixo = e.ajustes.get(1).copied().unwrap_or(cima);
        return [cima, cima, baixo, baixo];
    }
    [0.0; 4]
}

const FOLGA_ALCA_CANTO: f32 = 16.0;

/// A alça de cada canto (pixels da área): fica na borda de cima ou de baixo, um pouco para dentro do raio.
fn alcas_de_canto(e: &featherslide_core::quadro::ElementoDesenhado, ox: f32, oy: f32, escala: f32) -> [Point; 4] {
    let (x, y, w, h) = (ox + e.x * escala, oy + e.y * escala, e.largura * escala, e.altura * escala);
    let menor = w.abs().min(h.abs());
    let raios = raios_do(e);
    let c = Point::new(x + w / 2.0, y + h / 2.0);
    let (sen, cos) = e.rotacao.to_radians().sin_cos();
    let local = |i: usize| {
        let d = (raios[i] * menor + FOLGA_ALCA_CANTO).min(w.abs() / 2.0);
        let (px, py) = match i {
            0 => (x + d, y),
            1 => (x + w - d, y),
            2 => (x + w - d, y + h),
            _ => (x + d, y + h),
        };
        let (dx, dy) = (px - c.x, py - c.y);
        Point::new(c.x + dx * cos - dy * sen, c.y + dx * sen + dy * cos)
    };
    [local(0), local(1), local(2), local(3)]
}

/// O raio (fração do menor lado) que o cursor em `p` pede para o canto `i`.
fn raio_pelo_cursor(e: &featherslide_core::quadro::ElementoDesenhado, i: usize, p: Point, ox: f32, oy: f32, escala: f32) -> f32 {
    let (w, h) = (e.largura * escala, e.altura * escala);
    let c = Point::new(ox + e.x * escala + w / 2.0, oy + e.y * escala + h / 2.0);
    let (sen, cos) = (-e.rotacao).to_radians().sin_cos();
    let (dx, dy) = (p.x - c.x, p.y - c.y);
    let lx = dx * cos - dy * sen; // x local, a partir do centro
    let da_borda = if i == 0 || i == 3 { lx + w / 2.0 } else { w / 2.0 - lx };
    ((da_borda - FOLGA_ALCA_CANTO) / w.abs().min(h.abs()).max(1.0)).clamp(0.0, 0.5)
}

/// Distância, em pixels, da alça de rotação até a borda de cima do elemento.
const DIST_ALCA_GIRO: f32 = 28.0;

/// O centro do elemento e a posição da alça de rotação (em pixels da área), para o elemento girado.
fn alca_de_giro(e: &featherslide_core::quadro::ElementoDesenhado, ox: f32, oy: f32, escala: f32) -> (Point, Point) {
    let c = Point::new(ox + (e.x + e.largura / 2.0) * escala, oy + (e.y + e.altura / 2.0) * escala);
    let d = e.altura.abs() / 2.0 * escala + DIST_ALCA_GIRO;
    let r = e.rotacao.to_radians();
    (c, Point::new(c.x + d * r.sin(), c.y - d * r.cos()))
}

/// O ângulo (horário, 0 = para cima) do centro até o ponto; com `encaixar`, de 15 em 15°.
fn angulo_ate(c: Point, p: Point, encaixar: bool) -> f32 {
    let g = (p.x - c.x).atan2(-(p.y - c.y)).to_degrees().rem_euclid(360.0);
    let g = if encaixar { (g / 15.0).round() * 15.0 % 360.0 } else { (g * 10.0).round() / 10.0 };
    g
}

/// Uma guia de alinhamento: `true` = linha vertical em x, `false` = horizontal em y (pontos do slide).
type Guia = (bool, f32);

impl PalcoPrograma {
    /// Encaixa o arrasto (`dx`, `dy`, em pontos do slide) nas bordas e centros dos outros elementos e
    /// do slide, até 6 px de distância. Devolve o arrasto ajustado e as guias que acenderam.
    fn encaixar(&self, quadro: &Quadro, dx: f32, dy: f32, escala: f32) -> (f32, f32, Vec<Guia>) {
        let caixas: Vec<[f32; 4]> = quadro.elementos.iter().filter(|e| self.selecao.contains(&e.id)).map(|e| [e.x, e.y, e.largura, e.altura]).collect();
        if caixas.is_empty() {
            return (dx, dy, Vec::new());
        }
        let minx = caixas.iter().map(|c| c[0]).fold(f32::MAX, f32::min);
        let maxx = caixas.iter().map(|c| c[0] + c[2]).fold(f32::MIN, f32::max);
        let miny = caixas.iter().map(|c| c[1]).fold(f32::MAX, f32::min);
        let maxy = caixas.iter().map(|c| c[1] + c[3]).fold(f32::MIN, f32::max);
        let (mut alvos_x, mut alvos_y) = (vec![0.0, quadro.largura / 2.0, quadro.largura], vec![0.0, quadro.altura / 2.0, quadro.altura]);
        for (c, id) in self.caixas.iter().zip(&self.ids_caixas) {
            if self.selecao.contains(id) {
                continue;
            }
            alvos_x.extend([c[0], c[0] + c[2] / 2.0, c[0] + c[2]]);
            alvos_y.extend([c[1], c[1] + c[3] / 2.0, c[1] + c[3]]);
        }
        let limiar = 6.0 / escala;
        let melhor = |meus: [f32; 3], alvos: &[f32]| -> f32 {
            let mut m = f32::MAX;
            for v in meus {
                for t in alvos {
                    let d = t - v;
                    if d.abs() <= limiar && d.abs() < m.abs() {
                        m = d;
                    }
                }
            }
            if m == f32::MAX { 0.0 } else { m }
        };
        let ax = melhor([minx + dx, (minx + maxx) / 2.0 + dx, maxx + dx], &alvos_x);
        let ay = melhor([miny + dy, (miny + maxy) / 2.0 + dy, maxy + dy], &alvos_y);
        let (dx, dy) = (dx + ax, dy + ay);
        let mut guias: Vec<Guia> = Vec::new();
        for v in [minx + dx, (minx + maxx) / 2.0 + dx, maxx + dx] {
            for t in &alvos_x {
                if (t - v).abs() < 0.6 && !guias.contains(&(true, *t)) {
                    guias.push((true, *t));
                }
            }
        }
        for v in [miny + dy, (miny + maxy) / 2.0 + dy, maxy + dy] {
            for t in &alvos_y {
                if (t - v).abs() < 0.6 && !guias.contains(&(false, *t)) {
                    guias.push((false, *t));
                }
            }
        }
        (dx, dy, guias)
    }

    /// Escala e origem do slide dentro da área (coordenadas relativas à área).
    /// O único elemento selecionado (a alça de rotação só aparece com um).
    /// O retângulo selecionado, se for o único: só ele ganha as alças de canto.
    fn retangulo_unico<'a>(&self, quadro: &'a Quadro) -> Option<&'a featherslide_core::quadro::ElementoDesenhado> {
        self.unico_selecionado(quadro).filter(|e| e.tipo == "retangulo" || e.tipo == "retangulo_arredondado" || (e.tipo == "imagem" && e.recorte == "retangulo_arredondado"))
    }

    fn unico_selecionado<'a>(&self, quadro: &'a Quadro) -> Option<&'a featherslide_core::quadro::ElementoDesenhado> {
        match self.selecao.as_slice() {
            [id] => quadro.elementos.iter().find(|e| e.id == *id && e.tipo != "linha"),
            _ => None,
        }
    }

    /// Limita o `pan`: o slide nunca sai todo da área (sempre sobra um terço dele à vista).
    fn pan_limitado(&self, area: Size, quadro: &Quadro, x: f32, y: f32) -> (f32, f32) {
        let escala = ((area.width * 0.95) / quadro.largura).min((area.height * 0.95) / quadro.altura) * self.zoom;
        let (sw, sh) = (quadro.largura * escala, quadro.altura * escala);
        let lim = |lado: f32, area: f32| ((lado + area) / 2.0 - lado / 3.0).max(0.0);
        (x.clamp(-lim(sw, area.width), lim(sw, area.width)), y.clamp(-lim(sh, area.height), lim(sh, area.height)))
    }

    /// O cursor do mouse está dentro da caixa que está em edição?
    fn ponto_na_caixa_editada(&self, quadro: &Quadro, bounds: Rectangle, cursor: mouse::Cursor, ox: f32, oy: f32, escala: f32) -> bool {
        let (Some(id), Some(pos)) = (self.editando, cursor.position_in(bounds)) else { return false };
        quadro.elementos.iter().find(|e| e.id == id).is_some_and(|e| {
            let (x, y) = ((pos.x - ox) / escala, (pos.y - oy) / escala);
            x >= e.x && x <= e.x + e.largura && y >= e.y && y <= e.y + e.altura
        })
    }

    fn geometria(&self, area: Size, quadro: &Quadro) -> (f32, f32, f32) {
        use feather_draw::{Ponto as PontoDoc, Retangulo as RetanguloDoc, Unidade, Viewport};
        // Slide usa PT. Pan e cursor Iced são pixels LÓGICOS. O Iced aplica
        // DPI físico no framebuffer; não somar DPR ao zoom nem ao hit-test.
        // Centralizado conserva a escala visual antiga: 95% da área * zoom.
        let viewport = Viewport::centralizado(
            Unidade::Ponto,
            RetanguloDoc::novo(0.0, 0.0, area.width, area.height),
            quadro.largura, quadro.altura, 0.95,
            self.zoom, PontoDoc::novo(self.pan.0, self.pan.1), 1.0,
        );
        match viewport {
            Ok(v) => (v.escala(), v.origem.x, v.origem.y),
            // During a resize, Iced may briefly supply a zero-sized canvas.
            // Never divide mouse coordinates by zero or propagate NaN to Path.
            Err(_) => (1.0, area.width.max(0.0) / 2.0, area.height.max(0.0) / 2.0),
        }
    }
}

impl Program<PalcoMensagem> for PalcoPrograma {
    type State = PalcoEstado;

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (event::Status, Option<PalcoMensagem>) {
        let quadro = match &self.quadro {
            Some(q) => q,
            None => return (event::Status::Ignored, None),
        };
        let (escala, ox, oy) = self.geometria(bounds.size(), quadro);

        if let (Some(g),Some(e))=(&self.controle_gradiente,self.unico_selecionado(quadro)) {
            let (p0,p1)=linha_gradiente(e,g,ox,oy,escala);
            let ponto=|v:f32|Point::new(p0.x+(p1.x-p0.x)*v,p0.y+(p1.y-p0.y)*v);
            let v=Vector::new(p1.x-p0.x,p1.y-p0.y);let len=v.x.hypot(v.y).max(0.001);
            let giro=Point::new(p1.x+v.x/len*18.0,p1.y+v.y/len*18.0);
            match &event {
                canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))=>if let Some(p)=cursor.position_in(bounds){
                    if giro.distance(p)<8.0{state.gradiente_arrasto=Some(usize::MAX);return(event::Status::Captured,None);}
                    if let Some(i)=g.pontos.iter().enumerate().rev().find_map(|(i,(t,_))|(ponto(*t).distance(p)<8.0).then_some(i)){
                        state.gradiente_arrasto=Some(i);return(event::Status::Captured,Some(PalcoMensagem::GradientePonto{indice:i,posicao:g.pontos[i].0*100.0}));
                    }
                },
                canvas::Event::Mouse(mouse::Event::CursorMoved{..})=>if let (Some(i),Some(p))=(state.gradiente_arrasto,cursor.position()){
                    let p=Point::new(p.x-bounds.x,p.y-bounds.y);
                    if i==usize::MAX{let c=if g.radial{p0}else{ponto(0.5)};return(event::Status::Captured,Some(PalcoMensagem::GradienteAngulo(((p.y-c.y).atan2(p.x-c.x).to_degrees()-e.rotacao).rem_euclid(360.0))));}
                    let t=((p.x-p0.x)*v.x+(p.y-p0.y)*v.y)/(len*len);
                    return(event::Status::Captured,Some(PalcoMensagem::GradientePonto{indice:i,posicao:(t*100.0).clamp(0.0,100.0)}));
                },
                canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.gradiente_arrasto.take().is_some()=>return(event::Status::Captured,None),
                _=>{}
            }
        }
        if self.conectando {
            match &event {
                canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    if let Some(p)=cursor.position_in(bounds){return(event::Status::Captured,Some(PalcoMensagem::Clicou{x:(p.x-ox)/escala,y:(p.y-oy)/escala,somar:false,duplo:false}));}
                }
                canvas::Event::Mouse(mouse::Event::CursorMoved{..}) | canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => return(event::Status::Captured,None),
                _=>{}
            }
        }
        if self.desenho_livre {
            match &event {
                canvas::Event::Keyboard(keyboard::Event::KeyPressed { key: keyboard::Key::Named(keyboard::key::Named::Escape), .. }) => {
                    state.traco_livre.clear();return(event::Status::Captured,Some(PalcoMensagem::CancelarDesenho));
                }
                canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    if let Some(p)=cursor.position_in(bounds) {let p=Point::new((p.x-ox)/escala,(p.y-oy)/escala);if p.x>=0.0 && p.y>=0.0 && p.x<=quadro.largura && p.y<=quadro.altura{state.traco_livre=vec![p];}}
                    return(event::Status::Captured,None);
                }
                canvas::Event::Mouse(mouse::Event::CursorMoved {..}) if !state.traco_livre.is_empty() => {
                    if let Some(p)=cursor.position_in(bounds){let p=Point::new(((p.x-ox)/escala).clamp(0.0,quadro.largura),((p.y-oy)/escala).clamp(0.0,quadro.altura));if state.traco_livre.len()<10_000 && state.traco_livre.last().is_some_and(|q|q.distance(p)>=1.0){state.traco_livre.push(p);}}
                    return(event::Status::Captured,None);
                }
                canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    let pontos=std::mem::take(&mut state.traco_livre).into_iter().map(|p|[p.x,p.y]).collect();
                    return(event::Status::Captured,Some(PalcoMensagem::DesenhoLivre(pontos)));
                }
                _ => {}
            }
        }
        match event {
            canvas::Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => {
                state.somar = m.shift() || m.control();
                state.ctrl = m.control();
                state.shift = m.shift();
            }
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if let Some(id) = self.ajustando_recorte {
                    if !state.ctrl && cursor.position_in(bounds).is_some() {
                        let passo = match delta {
                            mouse::ScrollDelta::Lines { y, .. } => y,
                            mouse::ScrollDelta::Pixels { y, .. } => y / 40.0,
                        };
                        return (event::Status::Captured, Some(PalcoMensagem::ZoomRecorte { id, delta: passo * 0.08 }));
                    }
                }
                if state.ctrl && cursor.position_in(bounds).is_some() {
                    let passo = match delta {
                        mouse::ScrollDelta::Lines { y, .. } => y,
                        mouse::ScrollDelta::Pixels { y, .. } => y / 40.0,
                    };
                    return (event::Status::Captured, Some(PalcoMensagem::Zoom(passo * 0.1)));
                }
                // Sem Ctrl, com zoom: a roda rola a tela (Shift para o lado).
                if self.zoom > 1.0 && cursor.position_in(bounds).is_some() {
                    let pixels = match delta {
                        mouse::ScrollDelta::Lines { x, y } => (x * 48.0, y * 48.0),
                        mouse::ScrollDelta::Pixels { x, y } => (x, y),
                    };
                    let (dx, dy) = if state.somar { (pixels.1 + pixels.0, 0.0) } else { (pixels.0, pixels.1) };
                    let (x, y) = self.pan_limitado(bounds.size(), quadro, self.pan.0 + dx, self.pan.1 + dy);
                    return (event::Status::Captured, Some(PalcoMensagem::Pan { x, y }));
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                if let (Some(_), Some(abs)) = (cursor.position_in(bounds), cursor.position()) {
                    state.pan_arrasto = Some((abs, self.pan));
                    return (event::Status::Captured, None);
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) => {
                if state.pan_arrasto.take().is_some() {
                    return (event::Status::Captured, None);
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.pan_arrasto.is_some() => {
                if let (Some((inicio, pan0)), Some(abs)) = (state.pan_arrasto, cursor.position()) {
                    let (x, y) = self.pan_limitado(bounds.size(), quadro, pan0.0 + abs.x - inicio.x, pan0.1 + abs.y - inicio.y);
                    return (event::Status::Captured, Some(PalcoMensagem::Pan { x, y }));
                }
            }
            // Dentro da caixa que está sendo escrita, o mouse seleciona texto (não move a caixa).
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if self.editando.is_some() && !self.pegando_cor && self.arrastando_slide.is_none() && self.ponto_na_caixa_editada(quadro, bounds, cursor, ox, oy, escala) => {
                let pos = cursor.position_in(bounds).unwrap_or_default();
                let agora = Instant::now();
                let duplo = state.ultimo_clique.map(|(t, p)| agora.duration_since(t).as_millis() < 400 && p.distance(pos) < 6.0).unwrap_or(false);
                state.ultimo_clique = if duplo { None } else { Some((agora, pos)) };
                state.texto_arrasto = !duplo;
                return (event::Status::Captured, Some(PalcoMensagem::TextoPressionou { x: (pos.x - ox) / escala, y: (pos.y - oy) / escala, duplo }));
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.texto_arrasto => {
                if let Some(abs) = cursor.position() {
                    let (x, y) = ((abs.x - bounds.x - ox) / escala, (abs.y - bounds.y - oy) / escala);
                    return (event::Status::Captured, Some(PalcoMensagem::TextoArrastou { x, y }));
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.texto_arrasto => {
                state.texto_arrasto = false;
                return (event::Status::Captured, None);
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if let (Some(pos), Some(abs)) = (cursor.position_in(bounds), cursor.position()) {
                    return (event::Status::Captured, Some(PalcoMensagem::MenuContexto { x: (pos.x - ox) / escala, y: (pos.y - oy) / escala, janela: abs }));
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    // Selecionando uma região para o remover fundo: o arrasto só desenha o retângulo.
                    if self.pegando_cor {
                        state.area = true;
                        state.inicio_arrasto = Some(pos);
                        state.ultimo_ponto = Some(pos);
                        state.andou = false;
                        return (event::Status::Captured, None);
                    }
                    // Modo de ajuste de enquadramento ativo: clique na imagem arrasta o conteúdo interno
                    if let Some(id) = self.ajustando_recorte {
                        if let Some(e) = quadro.elementos.iter().find(|e| e.id == id) {
                            let (ex, ey, ew, eh) = (ox + e.x * escala, oy + e.y * escala, e.largura * escala, e.altura * escala);
                            if pos.x >= ex && pos.x <= ex + ew && pos.y >= ey && pos.y <= ey + eh {
                                state.recorte_arrasto = Some((id, pos));
                                state.inicio_arrasto = Some(pos);
                                state.ultimo_ponto = Some(pos);
                                state.andou = false;
                                return (event::Status::Captured, None);
                            }
                        }
                    }

                    // Tudo em coordenadas relativas à área, como o `draw`.
                    let agora = Instant::now();
                    let duplo = state
                        .ultimo_clique
                        .map(|(t, p)| agora.duration_since(t).as_millis() < 400 && p.distance(pos) < 6.0)
                        .unwrap_or(false);
                    state.ultimo_clique = if duplo { None } else { Some((agora, pos)) };
                    // As alças de canto (bolinhas laranja): puxar arredonda o canto; com Shift, todos.
                    if let Some(e) = self.retangulo_unico(quadro) {
                        let alcas = alcas_de_canto(e, ox, oy, escala);
                        if let Some(i) = alcas.iter().position(|a| a.distance(pos) <= 8.0) {
                            state.canto_arrasto = Some((e.id, i, raios_do(e)));
                            state.inicio_arrasto = Some(pos);
                            state.ultimo_ponto = Some(pos);
                            state.andou = true;
                            return (event::Status::Captured, None);
                        }
                    }
                    // A alça de rotação (bolinha em cima do elemento selecionado).
                    if let Some(e) = self.unico_selecionado(quadro) {
                        let (c, h) = alca_de_giro(e, ox, oy, escala);
                        if h.distance(pos) <= 9.0 {
                            state.girando = Some((e.id, c));
                            state.inicio_arrasto = Some(pos);
                            state.ultimo_ponto = Some(pos);
                            state.andou = true;
                            return (event::Status::Captured, None);
                        }
                    }
                    // Primeiro as alças dos elementos selecionados.
                    let alca = quadro.elementos.iter().filter(|e| self.selecao.contains(&e.id)).find_map(|e| {
                        posicoes_alcas_giradas(ox + e.x * escala, oy + e.y * escala, e.largura * escala, e.altura * escala, e.rotacao)
                            .iter()
                            .position(|p| p.distance(pos) <= 7.0)
                            .map(|i| (e.id, i, [e.x, e.y, e.largura, e.altura]))
                    });
                    state.alca = alca;
                    state.inicio_arrasto = Some(pos);
                    state.ultimo_ponto = Some(pos);
                    state.andou = false;
                    let (sx, sy) = ((pos.x - ox) / escala, (pos.y - oy) / escala);
                    if alca.is_some() {
                        state.ultimo_ponto = Some(pos);
                        return (event::Status::Captured, None);
                    }
                    state.area = !self.caixas.iter().any(|[x, y, w, h]| sx >= *x && sx <= x + w && sy >= *y && sy <= y + h);
                    let msg = PalcoMensagem::Clicou {
                        x: (pos.x - ox) / escala,
                        y: (pos.y - oy) / escala,
                        somar: state.somar,
                        duplo,
                    };
                    return (event::Status::Captured, Some(msg));
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.recorte_arrasto.is_some() {
                    if let Some(inicio) = state.inicio_arrasto {
                        let pos = match cursor.position() {
                            Some(p) => Point::new(p.x - bounds.x, p.y - bounds.y),
                            None => return (event::Status::Ignored, None),
                        };
                        if state.andou || pos.distance(inicio) > 3.0 {
                            state.andou = true;
                        }
                        state.ultimo_ponto = Some(pos);
                        return (event::Status::Captured, None);
                    }
                }
                if let Some(inicio) = state.inicio_arrasto {
                    // `position_over` perde o cursor fora da área; o arrasto continua nas bordas.
                    let pos = match cursor.position() {
                        Some(p) => Point::new(p.x - bounds.x, p.y - bounds.y),
                        None => return (event::Status::Ignored, None),
                    };
                    if state.andou || pos.distance(inicio) > 4.0 {
                        state.andou = true;
                    }
                    state.ultimo_ponto = Some(pos);
                    return (event::Status::Captured, None);
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if self.arrastando_slide.is_some() => {
                if let (Some(slide), Some(pos)) = (self.arrastando_slide, cursor.position_in(bounds)) {
                    return (event::Status::Captured, Some(PalcoMensagem::SoltouSlide { slide, x: (pos.x - ox) / escala, y: (pos.y - oy) / escala }));
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.recorte_arrasto.is_some() => {
                let arrasto = state.recorte_arrasto.take();
                let fim = state.ultimo_ponto.take();
                state.inicio_arrasto = None;
                if let (Some((id, inicio)), Some(fim)) = (arrasto, fim) {
                    if std::mem::take(&mut state.andou) {
                        let (dx, dy) = ((fim.x - inicio.x) / escala, (fim.y - inicio.y) / escala);
                        return (event::Status::Captured, Some(PalcoMensagem::ArrastouRecorte { id, dx, dy }));
                    }
                }
                return (event::Status::Captured, None);
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.canto_arrasto.is_some() => {
                let arrasto = state.canto_arrasto.take();
                let fim = state.ultimo_ponto.take();
                state.inicio_arrasto = None;
                state.andou = false;
                if let (Some((id, i, antes)), Some(p)) = (arrasto, fim) {
                    if let Some(e) = quadro.elementos.iter().find(|e| e.id == id) {
                        let r = raio_pelo_cursor(e, i, p, ox, oy, escala);
                        let mut novos = antes;
                        if state.somar { novos = [r; 4] } else { novos[i] = r }
                        return (event::Status::Captured, Some(PalcoMensagem::Cantos { raios: novos }));
                    }
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.girando.is_some() => {
                let (id, c) = state.girando.take().unwrap_or((0, Point::ORIGIN));
                let fim = state.ultimo_ponto.take();
                state.inicio_arrasto = None;
                state.andou = false;
                if let Some(p) = fim {
                    return (event::Status::Captured, Some(PalcoMensagem::Girou { id, graus: angulo_ate(c, p, state.somar) }));
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let (Some(inicio), Some(fim)) = (state.inicio_arrasto.take(), state.ultimo_ponto.take()) {
                    if let Some((id, alca, orig)) = state.alca.take() {
                        if std::mem::take(&mut state.andou) {
                            let rot = quadro.elementos.iter().find(|e| e.id == id).map_or(0.0, |e| e.rotacao);
                            let [x, y, w, h] = retangulo_proporcional(orig, alca, (fim.x - inicio.x) / escala, (fim.y - inicio.y) / escala, rot, state.shift || self.selecao.len() > 1);
                            return (event::Status::Captured, Some(PalcoMensagem::Redimensionou { id, x, y, w, h }));
                        }
                        return (event::Status::Captured, None);
                    }
                    let area = std::mem::take(&mut state.area);
                    if std::mem::take(&mut state.andou) {
                        if area {
                            let (x0, x1) = (inicio.x.min(fim.x), inicio.x.max(fim.x));
                            let (y0, y1) = (inicio.y.min(fim.y), inicio.y.max(fim.y));
                            let msg = PalcoMensagem::Area { x: (x0 - ox) / escala, y: (y0 - oy) / escala, w: (x1 - x0) / escala, h: (y1 - y0) / escala, somar: state.somar };
                            return (event::Status::Captured, Some(msg));
                        }
                        let (dx, dy, _) = self.encaixar(quadro, (fim.x - inicio.x) / escala, (fim.y - inicio.y) / escala, escala);
                        let msg = PalcoMensagem::Arrastou { dx, dy };
                        return (event::Status::Captured, Some(msg));
                    }
                    if self.pegando_cor && area {
                        // Sem arrasto: um clique, que pega a cor daquele ponto.
                        let msg = PalcoMensagem::Clicou { x: (fim.x - ox) / escala, y: (fim.y - oy) / escala, somar: false, duplo: false };
                        return (event::Status::Captured, Some(msg));
                    }
                    return (event::Status::Captured, None);
                }
            }
            _ => {}
        }

        (event::Status::Ignored, None)
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        // Tudo é desenhado dentro de UM recorte da área do palco: as imagens do Iced não são
        // aparadas pelo canvas e passariam por cima da faixa de opções. (O recorte é desenhado
        // antes do quadro principal; por isso a mesa e o slide têm de estar dentro dele.)
        let mut base = Frame::new(renderer, bounds.size());
        base.with_clip(Rectangle::new(Point::ORIGIN, bounds.size()), |frame| self.desenhar(state, frame, bounds, cursor));
        vec![base.into_geometry()]
    }
}

impl PalcoPrograma {
    fn desenhar(&self, state: &PalcoEstado, frame: &mut Frame, bounds: Rectangle, cursor: mouse::Cursor) {
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), crate::tema::COR_MESA());

        let quadro = match &self.quadro {
            Some(q) => q,
            None => return,
        };
        let (escala, ox, oy) = self.geometria(bounds.size(), quadro);
        let (sw, sh) = (quadro.largura * escala, quadro.altura * escala);

        // Sombra e borda do slide

        // Prévia do arrasto: o motor só é chamado ao soltar.
        let (desloc, guias) = match (state.inicio_arrasto, state.ultimo_ponto) {
            (Some(i), Some(u)) if state.andou && !state.area && state.alca.is_none() && state.girando.is_none() && state.canto_arrasto.is_none() && state.recorte_arrasto.is_none() => {
                let (dx, dy, guias) = self.encaixar(quadro, (u.x - i.x) / escala, (u.y - i.y) / escala, escala);
                ((dx, dy), guias)
            }
            _ => ((0.0, 0.0), Vec::new()),
        };
        let mut tmp;
        let quadro_pintado = match (state.alca, state.inicio_arrasto, state.ultimo_ponto) {
            _ if state.recorte_arrasto.is_some() => {
                tmp = quadro.clone();
                if let (Some((id, inicio)), Some(u)) = (state.recorte_arrasto, state.ultimo_ponto) {
                    if let Some(e) = tmp.elementos.iter_mut().find(|e| e.id == id) {
                        let ddx = (u.x - inicio.x) / escala;
                        let ddy = (u.y - inicio.y) / escala;
                        let delta_x = ddx / e.largura.max(1.0);
                        let delta_y = ddy / e.altura.max(1.0);
                        let px = e.ajustes.first().copied().unwrap_or(0.0) + delta_x;
                        let py = e.ajustes.get(1).copied().unwrap_or(0.0) + delta_y;
                        let z = e.ajustes.get(2).copied().unwrap_or(1.0);
                        let r = e.ajustes.get(3).copied().unwrap_or(0.16);
                        e.ajustes = vec![px.clamp(-1.0, 1.0), py.clamp(-1.0, 1.0), z, r];
                    }
                }
                &tmp
            }
            _ if state.canto_arrasto.is_some() => {
                tmp = quadro.clone();
                if let (Some((id, i, antes)), Some(u)) = (state.canto_arrasto, state.ultimo_ponto) {
                    if let Some(idx) = tmp.elementos.iter().position(|e| e.id == id) {
                        let r = raio_pelo_cursor(&tmp.elementos[idx], i, u, ox, oy, escala);
                        let mut novos = antes;
                        if state.somar { novos = [r; 4] } else { novos[i] = r }
                        tmp.elementos[idx].ajustes = novos.to_vec();
                    }
                }
                &tmp
            }
            _ if state.girando.is_some() => {
                tmp = quadro.clone();
                if let (Some((id, c)), Some(u)) = (state.girando, state.ultimo_ponto) {
                    if let Some(e) = tmp.elementos.iter_mut().find(|e| e.id == id) {
                        e.rotacao = angulo_ate(c, u, state.somar);
                    }
                }
                &tmp
            }
            (Some((id, alca, orig)), Some(i), Some(u)) if state.andou => {
                tmp = quadro.clone();
                let rot = quadro.elementos.iter().find(|e| e.id == id).map_or(0.0, |e| e.rotacao);
                let [x, y, w, h] = retangulo_proporcional(orig, alca, (u.x - i.x) / escala, (u.y - i.y) / escala, rot, state.shift || self.selecao.len() > 1);
                if self.selecao.len() > 1 {
                    if let Some(t) = featherslide_core::escala::Escala::entre(orig, [x, y, w, h]) {
                        for e in tmp.elementos.iter_mut().filter(|e| self.selecao.contains(&e.id)) {
                            [e.x, e.y, e.largura, e.altura] = t.retangulo([e.x, e.y, e.largura, e.altura]);
                            e.tamanho *= t.fator;
                            e.espessura *= t.fator;
                            for linha in &mut e.linhas {
                                linha.x = linha.x * t.fator + t.dx;
                                linha.y = linha.y * t.fator + t.dy;
                            }
                        }
                    }
                } else if let Some(e) = tmp.elementos.iter_mut().find(|e| e.id == id) {
                    (e.x, e.y, e.largura, e.altura) = (x, y, w, h);
                }
                &tmp
            }
            _ => quadro,
        };
        crate::desenho::ENQUADRANDO.with(|e| e.set(self.ajustando_recorte));
        crate::desenho::AREA_VISIVEL.with(|a| a.set(Some(Rectangle::new(Point::ORIGIN, bounds.size()))));
        pintar_quadro(frame, quadro_pintado, ox, oy, escala, &self.selecao, desloc, 8.0, None);
        if self.desenho_livre && !state.traco_livre.is_empty() {
            let path=Path::new(|b|for (i,p) in state.traco_livre.iter().enumerate(){let p=Point::new(ox+p.x*escala,oy+p.y*escala);if i==0{b.move_to(p)}else{b.line_to(p)}});
            frame.stroke(&path,Stroke::default().with_color(crate::tema::COR_AZUL_ACENTO()).with_width(2.0*escala));
        }
        crate::desenho::AREA_VISIVEL.with(|a| a.set(None));
        crate::desenho::ENQUADRANDO.with(|e| e.set(None));
        // As guias de alinhamento acendem enquanto o arrasto encaixa em algo.
        for (vertical, pos) in &guias {
            let cor = Color::from_rgb8(0xff, 0x2d, 0x95);
            let linha = if *vertical { Path::line(Point::new(ox + pos * escala, oy - 6.0), Point::new(ox + pos * escala, oy + sh + 6.0)) } else { Path::line(Point::new(ox - 6.0, oy + pos * escala), Point::new(ox + sw + 6.0, oy + pos * escala)) };
            frame.stroke(&linha, Stroke::default().with_color(cor).with_width(1.0));
        }
        // O que sangra para fora do slide (círculos de enfeite dos modelos) é coberto com a cor da mesa.
        let mesa = crate::tema::COR_MESA();
        let (w, h) = (bounds.width, bounds.height);
        frame.fill_rectangle(Point::new(0.0, 0.0), Size::new(w, oy.max(0.0)), mesa);
        frame.fill_rectangle(Point::new(0.0, oy + sh), Size::new(w, (h - oy - sh).max(0.0)), mesa);
        frame.fill_rectangle(Point::new(0.0, oy), Size::new(ox.max(0.0), sh), mesa);
        frame.fill_rectangle(Point::new(ox + sw, oy), Size::new((w - ox - sw).max(0.0), sh), mesa);
        // A sombra do slide, só do lado de fora (por cima do conteúdo escureceria o slide).
        let sombra = Color::from_rgba8(0, 0, 0, 0.16);
        frame.fill_rectangle(Point::new(ox + sw, oy + 5.0), Size::new(3.0, sh), sombra);
        frame.fill_rectangle(Point::new(ox + 3.0, oy + sh), Size::new(sw, 5.0), sombra);
        if let Some(e) = self.retangulo_unico(quadro_pintado) {
            for p in alcas_de_canto(e, ox + desloc.0 * escala, oy + desloc.1 * escala, escala) {
                frame.fill(&Path::circle(p, 5.0), Color::from_rgb8(0xff, 0xb3, 0x00));
                frame.stroke(&Path::circle(p, 5.0), Stroke::default().with_color(Color::WHITE).with_width(1.5));
            }
        }
        if let Some(e) = self.unico_selecionado(quadro_pintado) {
            let acento = crate::tema::COR_AZUL_ACENTO();
            let (c, h) = alca_de_giro(e, ox + desloc.0 * escala, oy + desloc.1 * escala, escala);
            let r = e.rotacao.to_radians();
            let borda = Point::new(c.x + (e.altura.abs() / 2.0 * escala) * r.sin(), c.y - (e.altura.abs() / 2.0 * escala) * r.cos());
            frame.stroke(&Path::line(borda, h), Stroke::default().with_color(acento).with_width(1.0));
            frame.fill(&Path::circle(h, 6.0), Color::WHITE);
            frame.stroke(&Path::circle(h, 6.0), Stroke::default().with_color(acento).with_width(1.5));
            // O arco de seta dentro da bolinha: dá a ideia de "girar".
            frame.stroke(&Path::circle(h, 2.5), Stroke::default().with_color(acento).with_width(1.0));
        }

        if let (Some(g),Some(e))=(&self.controle_gradiente,self.unico_selecionado(quadro_pintado)){
            let (p0,p1)=linha_gradiente(e,g,ox,oy,escala);
            frame.stroke(&Path::line(p0,p1),Stroke::default().with_color(Color::WHITE).with_width(4.0));
            frame.stroke(&Path::line(p0,p1),Stroke::default().with_color(crate::tema::COR_AZUL_ACENTO()).with_width(1.5));
            for (i,(t,cor)) in g.pontos.iter().enumerate(){let p=Point::new(p0.x+(p1.x-p0.x)*t,p0.y+(p1.y-p0.y)*t);let r=if i==g.selecionado{7.0}else{5.5};frame.fill(&Path::circle(p,r),cor_hex(cor).unwrap_or(Color::WHITE));frame.stroke(&Path::circle(p,r),Stroke::default().with_color(Color::WHITE).with_width(2.0));frame.stroke(&Path::circle(p,r+1.0),Stroke::default().with_color(crate::tema::COR_AZUL_ACENTO()).with_width(1.0));}
            let v=Vector::new(p1.x-p0.x,p1.y-p0.y);let len=v.x.hypot(v.y).max(0.001);let h=Point::new(p1.x+v.x/len*18.0,p1.y+v.y/len*18.0);
            frame.stroke(&Path::line(p1,h),Stroke::default().with_color(crate::tema::COR_AZUL_ACENTO()).with_width(1.0));frame.fill(&Path::circle(h,5.0),Color::WHITE);frame.stroke(&Path::circle(h,5.0),Stroke::default().with_color(crate::tema::COR_AZUL_ACENTO()).with_width(2.0));
        }

        if let Some([cx, cy, cw, ch]) = self.celula {
            let acento = crate::tema::COR_AZUL_ACENTO();
            frame.stroke(
                &Path::rectangle(Point::new(ox + cx * escala, oy + cy * escala), Size::new(cw * escala, ch * escala)),
                Stroke::default().with_color(acento).with_width(2.0),
            );
        }

        if let Some(id) = self.ajustando_recorte {
            if let Some(e) = quadro.elementos.iter().find(|e| e.id == id) {
                let (ex, ey, ew, eh) = (ox + e.x * escala, oy + e.y * escala, e.largura * escala, e.altura * escala);
                let azul = Color::from_rgb8(0x1a, 0x73, 0xe8);
                frame.stroke(
                    &Path::rectangle(Point::new(ex, ey), Size::new(ew, eh)),
                    Stroke::default().with_color(azul).with_width(2.0),
                );
                let l = 16.0;
                let traco = Stroke::default().with_color(Color::from_rgb8(0x20, 0x21, 0x24)).with_width(3.5);
                frame.stroke(&Path::line(Point::new(ex - 1.0, ey), Point::new(ex + l, ey)), traco);
                frame.stroke(&Path::line(Point::new(ex, ey - 1.0), Point::new(ex, ey + l)), traco);
                frame.stroke(&Path::line(Point::new(ex + ew + 1.0, ey), Point::new(ex + ew - l, ey)), traco);
                frame.stroke(&Path::line(Point::new(ex + ew, ey - 1.0), Point::new(ex + ew, ey + l)), traco);
                frame.stroke(&Path::line(Point::new(ex - 1.0, ey + eh), Point::new(ex + l, ey + eh)), traco);
                frame.stroke(&Path::line(Point::new(ex, ey + eh + 1.0), Point::new(ex, ey + eh - l)), traco);
                frame.stroke(&Path::line(Point::new(ex + ew + 1.0, ey + eh), Point::new(ex + ew - l, ey + eh)), traco);
                frame.stroke(&Path::line(Point::new(ex + ew, ey + eh + 1.0), Point::new(ex + ew, ey + eh - l)), traco);
                let mx = ex + ew / 2.0;
                let my = ey + eh / 2.0;
                frame.stroke(&Path::line(Point::new(mx - l / 2.0, ey), Point::new(mx + l / 2.0, ey)), traco);
                frame.stroke(&Path::line(Point::new(mx - l / 2.0, ey + eh), Point::new(mx + l / 2.0, ey + eh)), traco);
                frame.stroke(&Path::line(Point::new(ex, my - l / 2.0), Point::new(ex, my + l / 2.0)), traco);
                frame.stroke(&Path::line(Point::new(ex + ew, my - l / 2.0), Point::new(ex + ew, my + l / 2.0)), traco);
            }
        }
        if let Some(el) = self.editando.and_then(|id| quadro.elementos.iter().find(|e| e.id == id)) {
            let disp = featherslide_core::cursor::Disposicao { linhas: &el.linhas, tamanho: el.tamanho, negrito: el.negrito };
            if let Some((a, b)) = self.selecao_texto {
                let cor = Color { a: 0.32, ..crate::tema::COR_AZUL_ACENTO() };
                for (x, y, w) in featherslide_core::cursor::trechos(&self.texto_edicao, &disp, a, b) {
                    frame.fill_rectangle(Point::new(ox + x * escala, oy + (y - el.tamanho * 0.88) * escala), Size::new(w.max(2.0) * escala, el.tamanho * 1.2 * escala), cor);
                }
            }
            let fim = featherslide_core::cursor::ponto_do_cursor(&self.texto_edicao, &disp, self.cursor_edicao).map(|(x, y, _)| (x, y)).unwrap_or((el.x + 4.0, el.y + el.tamanho));
            let (cx, cy) = (ox + fim.0 * escala + 1.0, oy + fim.1 * escala);
            frame.stroke(
                &Path::line(Point::new(cx, cy - el.tamanho * escala * 0.9), Point::new(cx, cy + el.tamanho * escala * 0.15)),
                Stroke::default().with_color(Color::from_rgb8(0x1a, 0x73, 0xe8)).with_width(1.5),
            );
        }

        if let (true, true, Some(i), Some(u)) = (state.area, state.andou, state.inicio_arrasto, state.ultimo_ponto) {
            let (topo, tam) = (Point::new(i.x.min(u.x), i.y.min(u.y)), Size::new((u.x - i.x).abs(), (u.y - i.y).abs()));
            let acento = crate::tema::COR_AZUL_ACENTO();
            frame.fill_rectangle(topo, tam, Color { a: 0.12, ..acento });
            frame.stroke(&Path::rectangle(topo, tam), Stroke::default().with_color(acento).with_width(1.0));
        }

        if self.grade {
            let passo = 48.0 * escala;
            let linha = Stroke::default().with_color(Color::from_rgba8(128, 128, 128, 0.35)).with_width(1.0);
            let mut x = passo;
            while x < sw - 1.0 {
                frame.stroke(&Path::line(Point::new(ox + x, oy), Point::new(ox + x, oy + sh)), linha);
                x += passo;
            }
            let mut y = passo;
            while y < sh - 1.0 {
                frame.stroke(&Path::line(Point::new(ox, oy + y), Point::new(ox + sw, oy + y)), linha);
                y += passo;
            }
        }
        if self.guias {
            let guia = Stroke::default().with_color(crate::tema::COR_AZUL_ACENTO()).with_width(1.0);
            frame.stroke(&Path::line(Point::new(ox + sw / 2.0, oy), Point::new(ox + sw / 2.0, oy + sh)), guia);
            frame.stroke(&Path::line(Point::new(ox, oy + sh / 2.0), Point::new(ox + sw, oy + sh / 2.0)), guia);
        }

        frame.stroke(
            &Path::rectangle(Point::new(ox, oy), Size::new(sw, sh)),
            Stroke::default().with_color(Color::from_rgb8(0xd0, 0xd4, 0xdc)).with_width(1.0),
        );

        for [x, y, w, tam, tipo] in &self.ondulados {
            let (x0, x1, base) = (ox + x * escala, ox + (x + w) * escala, oy + (y + tam * 0.16) * escala);
            let (passo, amp) = (3.0_f32, 1.4_f32);
            let caminho = Path::new(|b| {
                let (mut px, mut cima) = (x0, true);
                b.move_to(Point::new(px, base));
                while px < x1 {
                    px = (px + passo).min(x1);
                    b.line_to(Point::new(px, base + if cima { -amp } else { amp }));
                    cima = !cima;
                }
            });
            frame.stroke(&caminho, Stroke::default().with_color(if *tipo > 0.5 { Color::from_rgb8(0x1a, 0x73, 0xe8) } else { Color::from_rgb8(0xd9, 0x30, 0x25) }).with_width(1.2));
        }

        if self.regua {
            let bandas: Vec<[f32; 4]> = quadro.elementos.iter().filter(|e| self.selecao.contains(&e.id)).map(|e| [e.x.min(e.x + e.largura), e.y.min(e.y + e.altura), e.largura.abs(), e.altura.abs()]).collect();
            let selecao = bandas.iter().copied().reduce(|a, b| {
                let (x0, y0, x1, y1) = (a[0].min(b[0]), a[1].min(b[1]), (a[0] + a[2]).max(b[0] + b[2]), (a[1] + a[3]).max(b[1] + b[3]));
                [x0, y0, x1 - x0, y1 - y0]
            });
            crate::regua::desenhar(frame, &crate::regua::Medidas { area: bounds.size(), ox, oy, escala, largura: quadro.largura, altura: quadro.altura, selecao, cursor: cursor.position_in(bounds) });
        }

        if self.arrastando_slide.is_some() {
            if let Some(pos) = cursor.position_in(bounds) {
                if pos.x >= ox && pos.x <= ox + sw && pos.y >= oy && pos.y <= oy + sh {
                    let gw = quadro.largura * 0.28 * escala;
                    let gh = quadro.altura * 0.28 * escala;
                    let gx = (pos.x - gw / 2.0).clamp(ox, ox + sw - gw);
                    let gy = (pos.y - gh / 2.0).clamp(oy, oy + sh - gh);
                    let ret = Rectangle::new(Point::new(gx, gy), Size::new(gw, gh));
                    let acento = crate::tema::COR_AZUL_ACENTO();
                    frame.fill_rectangle(ret.position(), ret.size(), Color { a: 0.16, ..acento });
                    frame.stroke(
                        &Path::rectangle(ret.position(), ret.size()),
                        Stroke::default().with_color(acento).with_width(2.0),
                    );
                }
            }
        }

    }
}

#[cfg(test)]
mod testes_proporcao {
    use super::*;

    #[test]
    fn viewport_slide_preserva_encaixe_zoom_pan_e_dpi() {
        let programa = PalcoPrograma {
            controle_gradiente: None,
            quadro: None,
            selecao: vec![], editando: None, texto_edicao: String::new(),
            cursor_edicao: 0, selecao_texto: None, grade: false, guias: false,
            regua: false, ondulados: vec![], celula: None, caixas: vec![],
            zoom: 1.5, arrastando_slide: None, pan: (28.0, -14.0),
            ids_caixas: vec![], ajustando_recorte: None,
            pegando_cor: false, desenho_livre: false, conectando: false,
        };
        let quadro = Quadro {
            versao: 0,
            largura: 960.0,
            altura: 540.0,
            fundo: String::new(),
            elementos: vec![],
        };
        let (s, ox, oy) = programa.geometria(Size::new(1200.0, 750.0), &quadro);
        let s_legacy = ((1200.0 * 0.95_f32) / 960.0)
            .min((750.0 * 0.95_f32) / 540.0) * 1.5;
        assert!((s - s_legacy).abs() < 0.0001);
        assert!((ox - ((1200.0 - quadro.largura * s) / 2.0 + 28.0)).abs() < 0.0001);
        assert!((oy - ((750.0 - quadro.altura * s) / 2.0 - 14.0)).abs() < 0.0001);
    }

    #[test]
    fn viewport_slide_com_area_zero_retorna_geometria_finita() {
        let programa = PalcoPrograma {
            controle_gradiente: None,
            quadro: None, selecao: vec![], editando: None,
            texto_edicao: String::new(), cursor_edicao: 0,
            selecao_texto: None, grade: false, guias: false, regua: false,
            ondulados: vec![], celula: None, caixas: vec![], zoom: 1.0,
            arrastando_slide: None, pan: (0.0, 0.0), ids_caixas: vec![],
            ajustando_recorte: None, pegando_cor: false,
            desenho_livre: false, conectando: false,
        };
        let quadro = Quadro {
            versao: 0, largura: 960.0, altura: 540.0,
            fundo: String::new(), elementos: vec![],
        };
        let (s, x, y) = programa.geometria(Size::new(0.0, 0.0), &quadro);
        assert!(s.is_finite() && s > 0.0);
        assert!(x.is_finite() && y.is_finite());
    }

    #[test]
    fn shift_preserva_proporcao_e_canto_oposto() {
        let original = [10.0, 20.0, 200.0, 100.0];
        let r = retangulo_proporcional(original, 4, 100.0, 10.0, 0.0, true);
        assert_eq!(r, [10.0, 20.0, 300.0, 150.0]);
        let livre = retangulo_proporcional(original, 4, 100.0, 10.0, 0.0, false);
        assert_eq!(livre, [10.0, 20.0, 300.0, 110.0]);
        let girado = retangulo_proporcional(original, 4, 0.0, 100.0, 90.0, true);
        assert!((girado[2] / girado[3] - 2.0).abs() < 0.001);
    }
}

fn linha_gradiente(e:&featherslide_core::quadro::ElementoDesenhado,g:&crate::formato_forma::ControleGradiente,ox:f32,oy:f32,escala:f32)->(Point,Point){
    let c=Point::new(ox+(e.x+e.largura/2.0)*escala,oy+(e.y+e.altura/2.0)*escala);
    let (s0,c0)=g.angulo.to_radians().sin_cos();let raio=if g.radial{e.largura.hypot(e.altura)/2.0}else{(e.largura*c0.abs()+e.altura*s0.abs())/2.0}*escala;
    let (s,cosa)=(g.angulo+e.rotacao).to_radians().sin_cos();
    (if g.radial{c}else{Point::new(c.x-cosa*raio,c.y-s*raio)},Point::new(c.x+cosa*raio,c.y+s*raio))
}

#[cfg(test)]
#[test]
fn pontos_gradiente_respondem_a_arrasto_no_canvas(){
    let mut sessao=featherslide_core::sessao::Sessao::default();sessao.elemento_inserir(featherslide_core::modelo::Tipo::Retangulo,Some([200.0,150.0,200.0,120.0]),None).unwrap();let id=sessao.slide().elementos.last().unwrap().id;
    let q=sessao.quadro(sessao.atual(),None,true).unwrap().unwrap();let g=crate::formato_forma::ControleGradiente{angulo:0.0,radial:false,pontos:vec![(0.0,"#000000".into()),(1.0,"#ffffff".into())],selecionado:0};
    let programa=PalcoPrograma{controle_gradiente:Some(g.clone()),quadro:Some(q.clone()),selecao:vec![id],editando:None,texto_edicao:String::new(),cursor_edicao:0,selecao_texto:None,grade:false,guias:false,regua:false,ondulados:vec![],celula:None,caixas:vec![],zoom:1.0,arrastando_slide:None,pan:(0.0,0.0),ids_caixas:vec![id],ajustando_recorte:None,pegando_cor:false,desenho_livre:false,conectando:false};
    let bounds=Rectangle::new(Point::ORIGIN,Size::new(960.0,540.0));let (s,ox,oy)=programa.geometria(bounds.size(),&q);let e=q.elementos.iter().find(|e|e.id==id).unwrap();let (p0,p1)=linha_gradiente(e,&g,ox,oy,s);let mut state=PalcoEstado::default();
    let (status,msg)=programa.update(&mut state,canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),bounds,mouse::Cursor::Available(p0));assert_eq!(status,event::Status::Captured);assert!(matches!(msg,Some(PalcoMensagem::GradientePonto{indice:0,..})));
    let p=Point::new(p0.x+(p1.x-p0.x)*0.4,p0.y+(p1.y-p0.y)*0.4);let (_,msg)=programa.update(&mut state,canvas::Event::Mouse(mouse::Event::CursorMoved{position:p}),bounds,mouse::Cursor::Available(p));assert!(matches!(msg,Some(PalcoMensagem::GradientePonto{indice:0,posicao}) if (posicao-40.0).abs()<0.01));
}
