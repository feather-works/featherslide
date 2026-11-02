//! Rascunho da aparência: aplicar cria uma única entrada no histórico.
use crate::tema::*;
use featherslide_core::estilo_forma::{Aparencia, Gradiente, PontoCor, TipoGradiente};
use featherslide_core::modelo::{Elemento, Sombra};
use pelicano::column;
use pelicano::*;

pub const LARGURA: f32 = 320.0;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Secao {
    Preenchimento,
    Contorno,
    Sombra,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alvo {
    Preenchimento,
    Contorno,
    Sombra,
    Ponto(usize),
}

#[derive(Clone, Debug)]
pub enum Msg {
    Fechar,
    Aplicar,
    Campo(&'static str, String),
    Tipo(String),
    Posicao(usize, String),
    Adicionar,
    Remover(usize),
    Inverter,
    Alternar(Secao),
    Contorno(bool),
    Sombra(bool),
    SelecionarPonto(usize),
    AbrirCor(Alvo),
    Seletor(feather_ui::seletor_cor::Evento),
    Preset(usize),
    SombraPreset(usize),
    MoverPonto(usize, f32),
    SombraDirecao(f32),
    SombraDistancia(f32),
    SombraTransparencia(f32),
}
#[derive(Clone, Debug)]
pub struct Estado {
    pub tipo: String,
    pub cor: String,
    pub contorno: String,
    pub angulo: String,
    pub espessura: String,
    pub transparencia: String,
    pub sombra: String,
    pub desfoque: String,
    pub dx: String,
    pub dy: String,
    pub pontos: Vec<(String, String)>,
    pub aviso: String,
    pub aberta: Secao,
    recolhida: bool,
    pub ponto_atual: usize,
    pub cor_aberta: Option<Alvo>,
    hsva: [f32; 4],
    texto_hex: String,
    contorno_guardado: String,
    sombra_guardada: String,
    #[cfg(test)]
    modelo: Elemento,
}
impl Estado {
    pub fn novo(e: &Elemento) -> Self {
        let g = e.gradiente.clone().unwrap_or_else(|| {
            let mut g = Gradiente::default();
            if !e.preenchimento.is_empty() {
                g.pontos[0].cor = e.preenchimento.clone();
            }
            g
        });
        Self {
            tipo: if e.gradiente.is_none() && e.preenchimento.is_empty() {
                "Sem preenchimento"
            } else if e.gradiente.is_none() {
                "Sólido"
            } else if g.tipo == TipoGradiente::Linear {
                "Linear"
            } else {
                "Radial"
            }
            .into(),
            cor: e.preenchimento.clone(),
            contorno: e.contorno.clone(),
            angulo: g.angulo.to_string(),
            espessura: e.espessura.to_string(),
            transparencia: ((1.0 - e.opacidade) * 100.0).to_string(),
            sombra: e.sombra.cor.clone(),
            desfoque: e.sombra.desfoque.to_string(),
            dx: e.sombra.dx.to_string(),
            dy: e.sombra.dy.to_string(),
            pontos: g
                .pontos
                .iter()
                .map(|p| (p.cor.clone(), (p.posicao * 100.0).to_string()))
                .collect(),
            aviso: String::new(),
            aberta: Secao::Preenchimento,
            recolhida: false,
            ponto_atual: 0,
            cor_aberta: None,
            hsva: [0.0, 0.0, 0.0, 1.0],
            texto_hex: String::new(),
            contorno_guardado: if e.contorno.is_empty() {
                "#1a56c4".into()
            } else {
                e.contorno.clone()
            },
            sombra_guardada: if e.sombra.cor.is_empty() {
                "#00000060".into()
            } else {
                e.sombra.cor.clone()
            },
            #[cfg(test)]
            modelo: e.clone(),
        }
    }
    pub fn tratar(&mut self, m: Msg) {
        match m {
            Msg::Campo(c, v) => {
                let alvo = match c {
                    "cor" => &mut self.cor,
                    "contorno" => &mut self.contorno,
                    "angulo" => &mut self.angulo,
                    "espessura" => &mut self.espessura,
                    "transparencia" => &mut self.transparencia,
                    "sombra" => &mut self.sombra,
                    "desfoque" => &mut self.desfoque,
                    "dx" => &mut self.dx,
                    _ => &mut self.dy,
                };
                *alvo = v;
                self.aviso.clear();
            }
            Msg::Tipo(v) => {
                self.tipo = v;
                if self.tipo == "Sólido" && self.cor.is_empty() {
                    self.cor = "#4285f4".into();
                }
                self.cor_aberta = None;
            }
            Msg::Posicao(i, v) => {
                if let Some(p) = self.pontos.get_mut(i) {
                    p.1 = v;
                }
            }
            Msg::Adicionar => {
                if self.pontos.len() < 16 {
                    self.pontos.push(("#ffffff".into(), "50".into()));
                    self.ponto_atual = self.pontos.len() - 1;
                }
            }
            Msg::Remover(i) => {
                if self.pontos.len() > 2 && i < self.pontos.len() {
                    self.pontos.remove(i);
                    self.ponto_atual = self.ponto_atual.min(self.pontos.len() - 1);
                    self.cor_aberta = None;
                }
            }
            Msg::Inverter => {
                self.cor_aberta = None;
                self.ponto_atual = self.pontos.len() - 1 - self.ponto_atual;
                self.pontos.reverse();
                for p in &mut self.pontos {
                    if let Ok(v) = numero(&p.1) {
                        p.1 = (100.0 - v).to_string();
                    }
                }
            }
            Msg::Alternar(s) => {
                if self.aberta == s {
                    self.recolhida = !self.recolhida;
                } else {
                    self.aberta = s;
                    self.recolhida = false;
                }
                self.cor_aberta = None;
            }
            Msg::SelecionarPonto(i) => {
                if i < self.pontos.len() {
                    self.ponto_atual = i;
                    self.cor_aberta = None;
                }
            }
            Msg::Contorno(ligado) => {
                if ligado {
                    self.contorno = self.contorno_guardado.clone();
                } else {
                    self.contorno_guardado = self.contorno.clone();
                    self.contorno.clear();
                }
                self.cor_aberta = None;
            }
            Msg::Sombra(ligado) => {
                if ligado {
                    self.sombra = self.sombra_guardada.clone();
                    if numero(&self.desfoque).unwrap_or(0.0) == 0.0 {
                        self.desfoque = "4".into();
                        self.dx = "3".into();
                        self.dy = "3".into();
                    }
                } else {
                    self.sombra_guardada = self.sombra.clone();
                    self.sombra.clear();
                }
                self.cor_aberta = None;
            }
            Msg::AbrirCor(alvo) => {
                if self.cor_aberta == Some(alvo) {
                    self.cor_aberta = None;
                } else {
                    let cor = self.cor(alvo).to_string();
                    self.hsva = feather_ui::seletor_cor::rgba_para_hsva(
                        feather_ui::seletor_cor::rgba_de(&cor),
                        0.0,
                    );
                    self.texto_hex = cor;
                    self.cor_aberta = Some(alvo);
                }
            }
            Msg::Seletor(ev) => self.tratar_cor(ev),
            Msg::Preset(i) => {
                let (a, b) = match i {
                    0 => ("#4285f4", "#34a853"),
                    1 => ("#7d5cc6", "#ec4899"),
                    2 => ("#f59e0b", "#ef4444"),
                    _ => ("#0f172a", "#94a3b8"),
                };
                self.pontos = vec![(a.into(), "0".into()), (b.into(), "100".into())];
                self.ponto_atual = 0;
                self.cor_aberta = None;
            }
            Msg::MoverPonto(i, v) => {
                if let Some(p) = self.pontos.get_mut(i) {
                    p.1 = format!("{:.1}", v.clamp(0.0, 100.0));
                    self.ponto_atual = i;
                }
            }
            Msg::SombraDirecao(a) => {
                let d = self.distancia_sombra();
                let (s, c) = a.to_radians().sin_cos();
                self.dx = format!("{:.1}", d * c);
                self.dy = format!("{:.1}", d * s);
            }
            Msg::SombraDistancia(d) => {
                let (s, c) = self.direcao_sombra().to_radians().sin_cos();
                self.dx = format!("{:.1}", d * c);
                self.dy = format!("{:.1}", d * s);
            }
            Msg::SombraTransparencia(t) => {
                let mut c = feather_ui::seletor_cor::rgba_de(&self.sombra);
                c[3] = ((1.0 - t.clamp(0.0, 100.0) / 100.0) * 255.0).round() as u8;
                self.sombra = feather_ui::seletor_cor::hex_de(c);
            }
            Msg::SombraPreset(i) => {
                let (dx, dy, blur) = match i {
                    0 => ("2", "3", "4"),
                    1 => ("6", "4", "3"),
                    _ => ("0", "6", "6"),
                };
                self.dx = dx.into();
                self.dy = dy.into();
                self.desfoque = blur.into();
                if self.sombra.is_empty() {
                    self.sombra = self.sombra_guardada.clone();
                }
            }
            _ => {}
        }
    }
    fn cor(&self, alvo: Alvo) -> &str {
        match alvo {
            Alvo::Preenchimento => &self.cor,
            Alvo::Contorno => &self.contorno,
            Alvo::Sombra => &self.sombra,
            Alvo::Ponto(i) => self.pontos.get(i).map_or("", |p| p.0.as_str()),
        }
    }
    fn definir_cor(&mut self, alvo: Alvo, cor: String) {
        let cor = if matches!(alvo, Alvo::Ponto(_)) && cor.is_empty() {
            "#00000000".into()
        } else {
            cor
        };
        match alvo {
            Alvo::Preenchimento => {
                self.cor = cor;
                if self.cor.is_empty() {
                    self.tipo = "Sem preenchimento".into();
                }
            }
            Alvo::Contorno => {
                if !cor.is_empty() {
                    self.contorno_guardado = cor.clone();
                }
                self.contorno = cor;
            }
            Alvo::Sombra => {
                if !cor.is_empty() {
                    self.sombra_guardada = cor.clone();
                }
                self.sombra = cor;
            }
            Alvo::Ponto(i) => {
                if let Some(p) = self.pontos.get_mut(i) {
                    p.0 = cor;
                }
            }
        }
        self.aviso.clear();
    }
    fn tratar_cor(&mut self, evento: feather_ui::seletor_cor::Evento) {
        use feather_ui::seletor_cor::{
            hex_de, hex_valido, hsv_para_rgb, rgba_de, rgba_para_hsva, Evento,
        };
        let Some(alvo) = self.cor_aberta else { return };
        match evento {
            Evento::Fechar => {
                self.cor_aberta = None;
                return;
            }
            Evento::TextoHex(v) => {
                self.texto_hex = v.clone();
                if hex_valido(&v) {
                    let c = hex_de(rgba_de(&v));
                    self.hsva = rgba_para_hsva(rgba_de(&c), self.hsva[0]);
                    self.definir_cor(alvo, c);
                }
                return;
            }
            Evento::Escolhida(c) => {
                self.hsva = rgba_para_hsva(rgba_de(&c), self.hsva[0]);
                self.texto_hex = c.clone();
                self.definir_cor(alvo, c);
                return;
            }
            Evento::Canal(i, v) => {
                if i >= 4 {
                    return;
                }
                if let Ok(v) = v.parse::<u8>() {
                    let mut c = rgba_de(self.cor(alvo));
                    c[i] = v;
                    let cor = hex_de(c);
                    self.hsva = rgba_para_hsva(c, self.hsva[0]);
                    self.texto_hex = cor.clone();
                    self.definir_cor(alvo, cor);
                }
                return;
            }
            Evento::SaturacaoBrilho(s, v) => {
                self.hsva[1] = s.clamp(0.0, 1.0);
                self.hsva[2] = v.clamp(0.0, 1.0);
            }
            Evento::Matiz(h) => self.hsva[0] = h.rem_euclid(360.0),
            Evento::Alfa(a) => self.hsva[3] = a.clamp(0.0, 1.0),
        }
        let [r, g, b] = hsv_para_rgb(self.hsva[0], self.hsva[1], self.hsva[2]);
        let c = hex_de([r, g, b, (self.hsva[3] * 255.0).round() as u8]);
        self.texto_hex = c.clone();
        self.definir_cor(alvo, c);
    }
    pub fn distancia_sombra(&self) -> f32 {
        numero(&self.dx)
            .unwrap_or(0.0)
            .hypot(numero(&self.dy).unwrap_or(0.0))
    }
    pub fn direcao_sombra(&self) -> f32 {
        numero(&self.dy)
            .unwrap_or(0.0)
            .atan2(numero(&self.dx).unwrap_or(1.0))
            .to_degrees()
            .rem_euclid(360.0)
    }
    pub fn controle_gradiente(&self) -> Option<ControleGradiente> {
        if self.aberta != Secao::Preenchimento
            || self.recolhida
            || !matches!(self.tipo.as_str(), "Linear" | "Radial")
        {
            return None;
        }
        Some(ControleGradiente {
            radial: self.tipo == "Radial",
            angulo: numero(&self.angulo).unwrap_or(0.0),
            pontos: self
                .pontos
                .iter()
                .map(|(c, p)| (numero(p).unwrap_or(0.0) / 100.0, c.clone()))
                .collect(),
            selecionado: self.ponto_atual,
        })
    }
    pub fn aparencia(&self) -> Result<Aparencia, String> {
        let gradiente = if self.tipo == "Sólido" || self.tipo == "Sem preenchimento" {
            None
        } else {
            let mut pontos: Vec<PontoCor> = self
                .pontos
                .iter()
                .map(|(c, p)| {
                    Ok(PontoCor {
                        cor: c.clone(),
                        posicao: numero(p)? / 100.0,
                    })
                })
                .collect::<Result<_, String>>()?;
            pontos.sort_by(|a, b| a.posicao.total_cmp(&b.posicao));
            Some(Gradiente {
                tipo: if self.tipo == "Linear" {
                    TipoGradiente::Linear
                } else {
                    TipoGradiente::Radial
                },
                angulo: numero(&self.angulo)?.rem_euclid(360.0),
                pontos,
            })
        };
        Ok(Aparencia {
            preenchimento: if self.tipo == "Sem preenchimento" {
                String::new()
            } else {
                self.cor.clone()
            },
            gradiente,
            contorno: self.contorno.clone(),
            espessura: numero(&self.espessura)?,
            opacidade: 1.0 - numero(&self.transparencia)? / 100.0,
            sombra: Sombra {
                cor: self.sombra.clone(),
                desfoque: numero(&self.desfoque)?,
                dx: numero(&self.dx)?,
                dy: numero(&self.dy)?,
            },
        })
    }
}
fn numero(v: &str) -> Result<f32, String> {
    let n: f32 = v
        .trim()
        .replace(',', ".")
        .parse()
        .map_err(|_| "Preencha os valores numéricos corretamente.".to_string())?;
    if !n.is_finite() {
        return Err("Use valores numéricos finitos.".into());
    }
    Ok(n)
}

fn linha() -> Element<'static, Msg> {
    container(Space::new(Length::FILL, 1))
        .width(Length::FILL)
        .style(estilo_linha)
        .into()
}
fn titulo_secao(titulo: &str, icone: &str, secao: Secao, e: &Estado) -> Element<'static, Msg> {
    button(
        row![
            icone_secao(icone),
            text(titulo.to_owned()).size(13),
            horizontal_space(),
            feather_ui::icones::icone(
                if e.aberta == secao && !e.recolhida {
                    "chevron-down"
                } else {
                    "chevron-right"
                },
                12.0
            ),
        ]
        .spacing(9)
        .align(Align::Center),
    )
    .padding([12, 14])
    .width(Length::FILL)
    .style(estilo_botao_pequeno(e.aberta == secao && !e.recolhida))
    .on_press(Msg::Alternar(secao))
    .into()
}
fn campo<'a>(rotulo: &str, id: &'static str, valor: &'a str, unidade: &str) -> Element<'a, Msg> {
    row![
        text(rotulo.to_owned()).size(12).width(Length::FILL),
        text_input("", valor)
            .on_input(move |v| Msg::Campo(id, v))
            .size(12)
            .padding([5, 7])
            .width(58)
            .style(estilo_entrada_suspensa),
        text(unidade.to_owned())
            .size(11)
            .color(COR_TEXTO_MUTED())
            .width(20),
    ]
    .spacing(6)
    .align(Align::Center)
    .into()
}
fn ajuste<'a>(
    rotulo: &str,
    id: &'static str,
    valor: &'a str,
    min: f32,
    max: f32,
    passo: f32,
    unidade: &str,
) -> Element<'a, Msg> {
    column![
        campo(rotulo, id, valor, unidade),
        slider(
            min..=max,
            numero(valor).unwrap_or(min).clamp(min, max),
            move |v| Msg::Campo(id, format!("{v:.1}"))
        )
        .step(passo)
        .color(COR_AZUL_ACENTO()),
    ]
    .spacing(6)
    .into()
}
fn seletor_cor<'a>(rotulo: &str, alvo: Alvo, e: &'a Estado) -> Element<'a, Msg> {
    let mut c = column![row![
        text(rotulo.to_owned()).size(12).width(Length::FILL),
        feather_ui::formatacao::botao_cor(
            if e.cor(alvo).is_empty() {
                "Sem cor"
            } else {
                "Cor"
            },
            e.cor(alvo),
            Some(Msg::AbrirCor(alvo)),
            e.cor_aberta == Some(alvo)
        ),
    ]
    .spacing(8)
    .align(Align::Center)]
    .spacing(8);
    if e.cor_aberta == Some(alvo) {
        c = c.push(
            feather_ui::seletor_cor::painel_embutido(
                "Escolher cor",
                e.cor(alvo),
                e.hsva,
                &e.texto_hex,
            )
            .map(Msg::Seletor),
        );
        c = c.push(
            button(text("Concluir cor").size(11))
                .padding([4, 8])
                .style(estilo_botao_secundario)
                .on_press(Msg::AbrirCor(alvo)),
        );
    }
    c.into()
}
fn chip<'a>(titulo: &str, msg: Msg, ativo: bool) -> Element<'a, Msg> {
    button(container(text(titulo.to_owned()).size(11)).width(Length::FILL).align_x(Align::Center))
        .padding([7, 6])
        .width(Length::FILL)
        .style(estilo_botao_pequeno(ativo))
        .on_press(msg)
        .into()
}
fn preenchimento<'a>(e: &'a Estado) -> Element<'a, Msg> {
    const GRADIENTES: [&str; 2] = ["Linear", "Radial"];
    let gradiente = matches!(e.tipo.as_str(), "Linear" | "Radial");
    let mut c = column![container(
        row![
            chip(
                "Nenhum",
                Msg::Tipo("Sem preenchimento".into()),
                e.tipo == "Sem preenchimento"
            ),
            chip("Sólido", Msg::Tipo("Sólido".into()), e.tipo == "Sólido"),
            chip(
                "Gradiente",
                Msg::Tipo(if gradiente {
                    e.tipo.clone()
                } else {
                    "Linear".into()
                }),
                gradiente
            ),
        ]
        .spacing(3)
    )
    .padding(3)
    .style(|_| Style {
        background: Some(Background::Color(COR_FUNDO_JANELA())),
        border: Border::rounded(6.0),
        ..Default::default()
    }),]
    .spacing(14);
    if e.tipo == "Sólido" {
        c = c.push(seletor_cor("Preenchimento", Alvo::Preenchimento, e));
    }
    if gradiente {
        c = c.push(
            row![
                text("Tipo").size(12).width(Length::FILL),
                pick_list(
                    &GRADIENTES,
                    Some(e.tipo.as_str()),
                    |v: &str| Msg::Tipo(v.to_string())
                )
                .size(12)
                .padding([5, 9])
                .width(130)
                .style(crate::faixa_abas::estilo_menu_da_faixa).handle_color(COR_AZUL_ACENTO())
            ]
            .align(Align::Center),
        );
        if e.tipo == "Linear" {
            c = c.push(ajuste("Ângulo", "angulo", &e.angulo, 0.0, 360.0, 1.0, "°"));
            c = c.push(
                row([0, 45, 90, 135, 180].into_iter().map(|a| {
                    chip(
                        &format!("{a}°"),
                        Msg::Campo("angulo", a.to_string()),
                        numero(&e.angulo).is_ok_and(|v| (v - a as f32).abs() < 0.1),
                    )
                }))
                .spacing(2),
            );
        }
        c = c.push(linha()).push(
            row![
                text("Pontos do gradiente").size(12).width(Length::FILL),
                button(feather_ui::icones::icone("plus", 14.0))
                    .padding(5)
                    .style(estilo_botao_secundario)
                    .on_press_maybe((e.pontos.len() < 16).then_some(Msg::Adicionar)),
                button(crate::icones::icone_menos(14.0, "#5f6368"))
                    .padding(5)
                    .style(estilo_botao_secundario)
                    .on_press_maybe((e.pontos.len() > 2).then_some(Msg::Remover(e.ponto_atual)))
            ]
            .spacing(5)
            .align(Align::Center),
        );
        c = c.push(text("Combinações").size(11).color(COR_TEXTO_MUTED()));
        c=c.push(row((0..4).map(|i|{
            let cores=match i{0=>("#4285f4","#34a853"),1=>("#7d5cc6","#ec4899"),2=>("#f59e0b","#ef4444"),_=>("#0f172a","#94a3b8")};
            button(svg(Svg::from_bytes(format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"52\" height=\"24\"><defs><linearGradient id=\"g\"><stop stop-color=\"{}\"/><stop offset=\"1\" stop-color=\"{}\"/></linearGradient></defs><rect width=\"52\" height=\"24\" rx=\"4\" fill=\"url(#g)\"/></svg>",cores.0,cores.1).as_bytes()).expect("SVG do degradê é gerado aqui")).width(52).height(24)).padding(3).style(estilo_botao_pequeno(false)).on_press(Msg::Preset(i)).into()
        })).spacing(7));
        let mut pontos = column![].spacing(4);
        for inicio in (0..e.pontos.len()).step_by(8) {
            pontos = pontos.push(
                row(e
                    .pontos
                    .iter()
                    .enumerate()
                    .skip(inicio)
                    .take(8)
                    .map(|(i, (cor, _))| {
                        let c = feather_ui::seletor_cor::rgba_de(cor);
                        let selecionado = i == e.ponto_atual;
                        button(
                            container(Space::new(18, 18)).style(move |_| Style {
                                background: Some(Background::Color(Color::rgb8(c[0], c[1], c[2]).alpha(c[3] as f32 / 255.0))),
                                border: Border::rounded(3.0),
                                ..Default::default()
                            }),
                        )
                        .padding(4)
                        .style(move |_, _| Style {
                            background: Some(Background::Color(COR_PAINEL())),
                            border: Border::rounded(5.0).color(
                                if selecionado { COR_AZUL_ACENTO() } else { COR_BORDA_SUAVE() },
                                if selecionado { 2.0 } else { 1.0 },
                            ),
                            ..Default::default()
                        })
                        .on_press(Msg::SelecionarPonto(i))
                        .into()
                    }))
                .spacing(5),
            );
        }
        c = c.push(pontos);
        if let Some((_, pos)) = e.pontos.get(e.ponto_atual) {
            c = c.push(seletor_cor(
                &format!("Cor do ponto {}", e.ponto_atual + 1),
                Alvo::Ponto(e.ponto_atual),
                e,
            ));
            let i = e.ponto_atual;
            c = c.push(
                column![
                    row![
                        text("Posição").size(12).width(Length::FILL),
                        text_input("", pos)
                            .on_input(move |v| Msg::Posicao(i, v))
                            .size(12)
                            .padding([5, 7])
                            .width(58)
                            .style(estilo_entrada_suspensa),
                        text("%").size(11).width(20).color(COR_TEXTO_MUTED())
                    ]
                    .spacing(6)
                    .align(Align::Center),
                    slider(
                        0.0..=100.0,
                        numero(pos).unwrap_or(0.0).clamp(0.0, 100.0),
                        move |v| Msg::Posicao(i, format!("{v:.0}"))
                    )
                    .step(1.0_f32)
                    .color(COR_AZUL_ACENTO())
                ]
                .spacing(6),
            );
        }
        c = c.push(
            button(
                row![
                    feather_ui::icones::icone("arrows-horizontal", 14.0),
                    text("Inverter cores").size(11)
                ]
                .spacing(6),
            )
            .padding([5, 8])
            .style(estilo_botao_secundario)
            .on_press(Msg::Inverter),
        );
    }
    if e.tipo != "Sem preenchimento" {
        c = c.push(ajuste(
            "Transparência",
            "transparencia",
            &e.transparencia,
            0.0,
            100.0,
            1.0,
            "%",
        ));
    }
    c.into()
}
fn contorno<'a>(e: &'a Estado) -> Element<'a, Msg> {
    let ligado = !e.contorno.is_empty();
    let mut c = column![checkbox("Mostrar contorno", ligado)
        .color(COR_AZUL_ACENTO())
        .on_toggle(Msg::Contorno)
        .size(12)]
    .spacing(14);
    if ligado {
        c = c.push(seletor_cor("Cor", Alvo::Contorno, e)).push(ajuste(
            "Espessura",
            "espessura",
            &e.espessura,
            0.0,
            12.0,
            0.5,
            "pt",
        ));
    }
    c.into()
}
fn sombra<'a>(e: &'a Estado) -> Element<'a, Msg> {
    let ligado = !e.sombra.is_empty();
    let mut c = column![checkbox("Ativar sombra", ligado)
        .color(COR_AZUL_ACENTO())
        .on_toggle(Msg::Sombra)
        .size(12)]
    .spacing(14);
    if ligado {
        c = c
            .push(
                row![
                    chip("Suave", Msg::SombraPreset(0), false),
                    chip("Diagonal", Msg::SombraPreset(1), false),
                    chip("Abaixo", Msg::SombraPreset(2), false)
                ]
                .spacing(3),
            )
            .push(seletor_cor("Cor", Alvo::Sombra, e))
            .push(ajuste(
                "Desfoque",
                "desfoque",
                &e.desfoque,
                0.0,
                14.0,
                0.5,
                "pt",
            ))
            .push(
                column![
                    row![
                        text("Direção").size(12).width(Length::FILL),
                        text(format!("{:.0}°", e.direcao_sombra())).size(12)
                    ],
                    slider(0.0..=360.0, e.direcao_sombra(), Msg::SombraDirecao)
                        .step(1.0_f32)
                        .color(COR_AZUL_ACENTO())
                ]
                .spacing(6),
            )
            .push(
                column![
                    row![
                        text("Distância").size(12).width(Length::FILL),
                        text(format!("{:.1} pt", e.distancia_sombra())).size(12)
                    ],
                    slider(
                        0.0..=200.0,
                        e.distancia_sombra().min(200.0),
                        Msg::SombraDistancia
                    )
                    .step(0.5_f32)
                    .color(COR_AZUL_ACENTO())
                ]
                .spacing(6),
            )
            .push(
                column![
                    row![
                        text("Transparência").size(12).width(Length::FILL),
                        text(format!(
                            "{:.0}%",
                            100.0
                                * (1.0
                                    - feather_ui::seletor_cor::rgba_de(&e.sombra)[3] as f32
                                        / 255.0)
                        ))
                        .size(12)
                    ],
                    slider(
                        0.0..=100.0,
                        100.0
                            * (1.0 - feather_ui::seletor_cor::rgba_de(&e.sombra)[3] as f32 / 255.0),
                        Msg::SombraTransparencia
                    )
                    .step(1.0_f32)
                    .color(COR_AZUL_ACENTO())
                ]
                .spacing(6),
            )
            .push(campo("Horizontal", "dx", &e.dx, "pt"))
            .push(campo("Vertical", "dy", &e.dy, "pt"));
    }
    c.into()
}
/// Painel encaixado na linha do editor: ocupa espaço real, sem cobrir o palco.
pub fn view<'a>(e: &'a Estado) -> Element<'a, Msg> {
    let cabecalho = container(
        row![
            text("Formato da forma").size(15),
            horizontal_space(),
            button(feather_ui::icones::icone("x", 14.0))
                .padding(5)
                .style(estilo_botao_pequeno(false))
                .on_press(Msg::Fechar)
        ]
        .align(Align::Center),
    )
    .padding([14, 16]);
    let mut corpo = column![].spacing(0);
    for (secao, titulo, icone) in [
        (Secao::Preenchimento, "Preenchimento", "bucket"),
        (Secao::Contorno, "Contorno", "square"),
        (Secao::Sombra, "Sombra", "window"),
    ] {
        corpo = corpo.push(titulo_secao(titulo, icone, secao, e));
        if e.aberta == secao && !e.recolhida {
            corpo = corpo.push(
                container(match secao {
                    Secao::Preenchimento => preenchimento(e),
                    Secao::Contorno => contorno(e),
                    Secao::Sombra => sombra(e),
                })
                .padding(Padding {
                    top: 6.0,
                    right: 16.0,
                    bottom: 18.0,
                    left: 16.0,
                })
                .width(Length::FILL),
            );
        }
        corpo = corpo.push(linha());
    }
    let mut rodape = column![].spacing(9);
    if !e.aviso.is_empty() {
        rodape = rodape.push(text(&e.aviso).size(11).color(COR_TEXTO_MUTED()));
    }
    rodape = rodape.push(
        row![
            button(text("Cancelar").size(12))
                .padding([7, 12])
                .style(estilo_botao_secundario)
                .on_press(Msg::Fechar),
            horizontal_space(),
            button(text("Aplicar").size(12))
                .padding([7, 18])
                .style(estilo_botao_primario)
                .on_press(Msg::Aplicar)
        ]
        .spacing(8)
        .align(Align::Center),
    );
    row![
        linha_vertical(),
        container(column![
            cabecalho,
            linha(),
            scrollable(corpo).scrollbar_width(10).scrollbar_track(COR_TRILHO_ROLAGEM()).scrollbar_color(COR_POLEGAR_ROLAGEM()).height(Length::FILL),
            container(rodape).padding([12, 16])
        ])
        .width(LARGURA - 1.0)
        .height(Length::FILL)
        .style(|_| Style {
            background: Some(Background::Color(COR_PAINEL())),
            text_color: Some(COR_TEXTO()),
            ..Default::default()
        })
    ]
    .width(LARGURA)
    .height(Length::FILL)
    .into()
}
fn linha_vertical() -> Element<'static, Msg> {
    container(Space::new(1, Length::FILL))
        .height(Length::FILL)
        .style(estilo_linha)
        .into()
}

fn icone_secao(nome: &str) -> SvgView {
    let caminho=match nome {"square"=>"<rect x=\"4\" y=\"4\" width=\"16\" height=\"16\" rx=\"2\"/>","window"=>"<path d=\"M8 16v4h12V8h-4\"/><rect x=\"3\" y=\"3\" width=\"13\" height=\"13\" rx=\"2\"/>",_=>return feather_ui::icones::icone(nome,16.0)};
    svg(Svg::from_bytes(format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#808080\" stroke-width=\"1.7\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{caminho}</svg>").as_bytes()).expect("SVG do ícone é fixo")).width(16).height(16).color(COR_TEXTO())
}

#[cfg(test)]
mod testes {
    use super::*;
    fn estado() -> Estado {
        Estado::novo(&Elemento {
            tipo: featherslide_core::modelo::Tipo::RetanguloArredondado,
            preenchimento: "#4285f4".into(),
            contorno: "#123456".into(),
            ..Default::default()
        })
    }
    #[test]
    fn cor_visual_muda_somente_o_rascunho() {
        let mut e = estado();
        e.tratar(Msg::AbrirCor(Alvo::Preenchimento));
        e.tratar(Msg::Seletor(feather_ui::seletor_cor::Evento::Escolhida(
            "#ff8800".into(),
        )));
        assert_eq!(e.cor, "#ff8800");
        assert_eq!(e.modelo.preenchimento, "#4285f4");
        e.tratar(Msg::Seletor(feather_ui::seletor_cor::Evento::TextoHex(
            "#x".into(),
        )));
        assert_eq!(e.cor, "#ff8800");
    }
    #[test]
    fn desativar_e_reativar_contorno_preserva_cor() {
        let mut e = estado();
        e.tratar(Msg::Contorno(false));
        assert!(e.contorno.is_empty());
        e.tratar(Msg::Contorno(true));
        assert_eq!(e.contorno, "#123456");
        e.tratar(Msg::AbrirCor(Alvo::Contorno));
        e.tratar(Msg::Seletor(feather_ui::seletor_cor::Evento::Escolhida(
            String::new(),
        )));
        e.tratar(Msg::Contorno(true));
        assert_eq!(e.contorno, "#123456");
        e.tratar(Msg::Sombra(true));
        assert!(!e.sombra.is_empty());
        assert_eq!(e.desfoque, "4");
    }
    #[test]
    fn remover_ponto_mantem_selecao_e_gradiente_valido() {
        let mut e = estado();
        e.tratar(Msg::Tipo("Linear".into()));
        e.tratar(Msg::Adicionar);
        assert_eq!(e.ponto_atual, 2);
        e.tratar(Msg::AbrirCor(Alvo::Ponto(2)));
        e.tratar(Msg::Remover(2));
        assert!(e.cor_aberta.is_none());
        assert_eq!(e.ponto_atual, 1);
        e.aparencia().unwrap().gradiente.unwrap().validar().unwrap();
    }
    #[test]
    fn controle_do_palco_acompanha_tipo_gradiente() {
        let mut e = estado();
        assert!(e.controle_gradiente().is_none());
        e.tratar(Msg::Tipo("Radial".into()));
        e.tratar(Msg::Preset(1));
        e.tratar(Msg::Sombra(true));
        assert!(e.controle_gradiente().is_some());
    }
}

#[derive(Clone, Debug)]
pub struct ControleGradiente {
    pub angulo: f32,
    pub radial: bool,
    pub pontos: Vec<(f32, String)>,
    pub selecionado: usize,
}

#[cfg(test)]
#[test]
fn sombra_controla_direcao_distancia_e_alfa() {
    let mut e = Estado::novo(&Elemento::default());
    e.tratar(Msg::Sombra(true));
    e.tratar(Msg::SombraDirecao(90.0));
    e.tratar(Msg::SombraDistancia(10.0));
    assert!(numero(&e.dx).unwrap().abs() < 0.1);
    assert!((numero(&e.dy).unwrap() - 10.0).abs() < 0.1);
    e.tratar(Msg::SombraTransparencia(50.0));
    assert!((feather_ui::seletor_cor::rgba_de(&e.sombra)[3] as i32 - 128).abs() <= 1);
}
