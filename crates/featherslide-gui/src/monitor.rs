//! O Monitor (F12): Memória do processo, Console e Métricas, como no FeatherSheet. Sem WebKit
//! o aplicativo é um processo só, então a memória é a dele (`/proc/self`).

use iced::widget::{button, checkbox, column, container, row, text, text_input, Space};
use iced::{Alignment, Color, Element, Length};

use crate::ajuda::t;
use crate::registro::{self, ler_metrica};
use crate::tema::*;

const LENTA_MS: f64 = 100.0;
const HISTORICO: usize = 120;
/// `sysconf(_SC_CLK_TCK)` é 100 em todo Linux comum.
const TICKS_POR_SEGUNDO: f32 = 100.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Aba {
    Memoria,
    Console,
    Metricas,
}

#[derive(Clone, Debug)]
pub enum Msg {
    Aba(Aba),
    Origem(String),
    SoLentas(bool),
    OrdenarPorTempo,
    Limpar,
    FiltroConsole(String),
    /// Copia o texto para a área de transferência (tratado em `app.rs`).
    Copiar(String),
}

pub struct Estado {
    pub aba: Aba,
    origem: String,
    so_lentas: bool,
    por_tempo: bool,
    filtro: String,
    rss: Vec<u64>,
    pico: u64,
    threads: u32,
    cpu: f32,
    cpu_hist: Vec<f32>,
    ultimo_cpu: Option<(u64, std::time::Instant)>,
}

impl Default for Estado {
    fn default() -> Self {
        Self {
            aba: Aba::Memoria,
            origem: String::new(),
            so_lentas: false,
            por_tempo: false,
            filtro: String::new(),
            rss: Vec::new(),
            pico: 0,
            threads: 0,
            cpu: 0.0,
            cpu_hist: Vec::new(),
            ultimo_cpu: None,
        }
    }
}

/// kB de um campo de `/proc/self/status`.
fn campo_kb(status: &str, nome: &str) -> u64 {
    status
        .lines()
        .find_map(|l| l.strip_prefix(nome)?.trim_start_matches(':').split_whitespace().next()?.parse::<u64>().ok())
        .unwrap_or(0)
}

/// Tempo de CPU acumulado do processo, em ticks (utime + stime de `/proc/self/stat`).
fn ticks_de_cpu() -> Option<u64> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    let depois = stat.rsplit_once(')')?.1;
    let campos: Vec<&str> = depois.split_whitespace().collect();
    Some(campos.get(11)?.parse::<u64>().ok()? + campos.get(12)?.parse::<u64>().ok()?)
}

impl Estado {
    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Aba(a) => self.aba = a,
            Msg::Origem(o) => self.origem = if o == t("monitor.metricas.todasOrigens") { String::new() } else { o },
            Msg::SoLentas(b) => self.so_lentas = b,
            Msg::OrdenarPorTempo => self.por_tempo = !self.por_tempo,
            Msg::Limpar => registro::limpar(),
            Msg::FiltroConsole(f) => self.filtro = f,
            Msg::Copiar(_) => {}
        }
    }

    /// Uma amostra por segundo, só com o Monitor aberto.
    pub fn amostrar(&mut self) {
        // Duas amostras coladas (abrir o Monitor + o tique) dariam dt minúsculo e CPU absurda.
        if self.ultimo_cpu.is_some_and(|(_, q)| q.elapsed().as_secs_f32() < 0.5) {
            return;
        }
        let Ok(status) = std::fs::read_to_string("/proc/self/status") else { return };
        let rss = campo_kb(&status, "VmRSS") * 1024;
        self.pico = self.pico.max(campo_kb(&status, "VmHWM") * 1024);
        self.threads = campo_kb(&status, "Threads") as u32;
        if self.rss.len() >= HISTORICO {
            self.rss.remove(0);
        }
        self.rss.push(rss);
        if let Some(ticks) = ticks_de_cpu() {
            let agora = std::time::Instant::now();
            if let Some((antes, quando)) = self.ultimo_cpu {
                let dt = agora.duration_since(quando).as_secs_f32();
                // % de um núcleo (100 % = um núcleo cheio; pode passar disso com várias threads).
                self.cpu = (ticks.saturating_sub(antes) as f32 / TICKS_POR_SEGUNDO) / dt * 100.0;
                if self.cpu_hist.len() >= HISTORICO {
                    self.cpu_hist.remove(0);
                }
                self.cpu_hist.push(self.cpu);
            }
            self.ultimo_cpu = Some((ticks, agora));
        }
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let aba = |a: Aba, rotulo: String| {
            button(text(rotulo).size(13)).padding([5, 14]).style(estilo_aba_faixa(self.aba == a)).on_press(Msg::Aba(a))
        };
        let erros = registro::copia().iter().filter(|l| l.texto.to_lowercase().contains("erro")).count();
        let console = if erros > 0 { format!("{} ({erros})", t("monitor.abas.console")) } else { t("monitor.abas.console") };
        let topo = container(
            row![
                text(t("monitor.titulo").replace("{nome}", "FeatherSlide")).size(17),
                Space::with_width(20),
                aba(Aba::Memoria, t("monitor.abas.memoria")),
                aba(Aba::Console, console),
                aba(Aba::Metricas, t("monitor.abas.metricas")),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([10, 16])
        .style(estilo_container_topo);

        let corpo: Element<'_, Msg> = match self.aba {
            Aba::Memoria => self.memoria(),
            Aba::Console => self.console(),
            Aba::Metricas => self.metricas(),
        };
        column![topo, container(corpo).padding(14).width(Length::Fill).height(Length::Fill)].into()
    }

    fn memoria(&self) -> Element<'_, Msg> {
        let agora = self.rss.last().copied().unwrap_or(0);
        let cartao = |rotulo: String, valor: String| {
            container(column![text(rotulo).size(11).color(COR_TEXTO_MUTED()), text(valor).size(20)].spacing(2))
                .padding([10, 16])
                .width(Length::Fill)
                .style(estilo_painel_faixa)
        };
        let cartoes = row![
            cartao(t("monitor.memoria.cartoes.total.rotulo"), mb(agora)),
            cartao(t("monitor.memoria.cartoes.pico.rotulo"), mb(self.pico)),
            cartao(t("monitor.memoria.cartoes.cpu.rotulo"), format!("{:.0} %", self.cpu)),
            cartao("Threads".into(), self.threads.to_string()),
        ]
        .spacing(10);

        let maximo = self.rss.iter().copied().max().unwrap_or(1).max(1) as f32;
        let mut barras = row![].spacing(1).align_y(Alignment::End).height(Length::Fixed(120.0));
        for &v in &self.rss {
            let h = (v as f32 / maximo * 118.0).max(1.0);
            barras = barras.push(
                container(Space::with_width(Length::Fixed(4.0)))
                    .height(Length::Fixed(h))
                    .style(|_| container::Style { background: Some(iced::Background::Color(COR_AZUL_ACENTO())), ..Default::default() }),
            );
        }
        let teto_cpu = self.cpu_hist.iter().copied().fold(100.0_f32, f32::max);
        let mut barras_cpu = row![].spacing(1).align_y(Alignment::End).height(Length::Fixed(80.0));
        for &v in &self.cpu_hist {
            let h = (v / teto_cpu * 78.0).max(1.0);
            barras_cpu = barras_cpu.push(
                container(Space::with_width(Length::Fixed(4.0)))
                    .height(Length::Fixed(h))
                    .style(|_| container::Style { background: Some(iced::Background::Color(Color::from_rgb8(0xe0, 0x8a, 0x1e))), ..Default::default() }),
            );
        }
        column![
            cartoes,
            text(t("monitor.memoria.ultimos").replace("{teto}", &mb(maximo as u64))).size(12).color(COR_TEXTO_MUTED()),
            container(barras).padding(8).style(estilo_painel_faixa),
            text(format!("CPU (% de um núcleo, teto {teto_cpu:.0} %)")).size(12).color(COR_TEXTO_MUTED()),
            container(barras_cpu).padding(8).style(estilo_painel_faixa),
            text("Medição do processo do aplicativo (/proc/self); a interface nativa não tem processo de WebKit.").size(11).color(COR_TEXTO_MUTED()),
        ]
        .spacing(12)
        .into()
    }

    fn console(&self) -> Element<'_, Msg> {
        let filtro = normalizar_simples(&self.filtro);
        let linhas: Vec<_> = registro::copia().into_iter().filter(|l| filtro.is_empty() || normalizar_simples(&l.texto).contains(&filtro)).collect();
        let barra = row![
            text_input(&t("monitor.console.buscar"), &self.filtro).on_input(Msg::FiltroConsole).padding([4, 8]).width(Length::Fixed(300.0)),
            button(text("Copiar tudo").size(12)).padding([4, 10]).style(estilo_botao_secundario).on_press(Msg::Copiar(
                linhas.iter().map(|l| format!("{:.1}s {}", l.quando, l.texto)).collect::<Vec<_>>().join("\n")
            )),
            button(text(t("monitor.console.limpar")).size(12)).padding([4, 10]).style(estilo_botao_secundario).on_press(Msg::Limpar),
        ]
        .spacing(8);
        let mut col = column![].spacing(2);
        for l in linhas.iter().rev().take(500) {
            let cor = if l.texto.to_lowercase().contains("erro") { Color::from_rgb8(0xc5, 0x22, 0x1f) } else { COR_TEXTO() };
            let linha = row![text(format!("{:>7.1}s", l.quando)).size(12).color(COR_TEXTO_MUTED()), text(l.texto.clone()).size(12).color(cor)].spacing(10);
            col = col.push(button(linha).padding(0).style(button::text).on_press(Msg::Copiar(l.texto.clone())));
        }
        column![barra, crate::tema::rolagem(col).height(Length::Fill).width(Length::Fill)].spacing(10).into()
    }

    fn metricas(&self) -> Element<'_, Msg> {
        let todas: Vec<_> = registro::copia().into_iter().filter_map(|l| ler_metrica(&l.texto).map(|m| (l.quando, m))).collect();
        let mut origens: Vec<String> = todas.iter().map(|(_, m)| m.origem.clone()).collect();
        origens.sort();
        origens.dedup();
        let mut botoes = row![].spacing(4);
        let todas_rotulo = t("monitor.metricas.todasOrigens");
        botoes = botoes.push(
            button(text(todas_rotulo.clone()).size(12)).padding([3, 8]).style(estilo_botao_pequeno(self.origem.is_empty())).on_press(Msg::Origem(todas_rotulo)),
        );
        for o in origens {
            let ativo = self.origem == o;
            botoes = botoes.push(button(text(format!("[{o}]")).size(12)).padding([3, 8]).style(estilo_botao_pequeno(ativo)).on_press(Msg::Origem(o)));
        }
        let mut vistas: Vec<_> = todas
            .into_iter()
            .filter(|(_, m)| (self.origem.is_empty() || m.origem == self.origem) && (!self.so_lentas || m.ms.unwrap_or(0.0) >= LENTA_MS))
            .collect();
        if self.por_tempo {
            vistas.sort_by(|a, b| b.1.ms.unwrap_or(-1.0).total_cmp(&a.1.ms.unwrap_or(-1.0)));
        } else {
            vistas.reverse();
        }
        let copia = vistas
            .iter()
            .map(|(q, m)| format!("{q:.1}s [{}] {} {}", m.origem, m.texto, m.ms.map(|v| format!("{v} ms")).unwrap_or_default()))
            .collect::<Vec<_>>()
            .join("\n");
        let barra = row![
            botoes,
            checkbox(t("monitor.metricas.soLentas").replace("{ms}", &LENTA_MS.to_string()), self.so_lentas).on_toggle(Msg::SoLentas).text_size(12),
            button(text(t("monitor.metricas.ordenar")).size(12)).padding([4, 10]).style(estilo_botao_pequeno(self.por_tempo)).on_press(Msg::OrdenarPorTempo),
            button(text("Copiar tudo").size(12)).padding([4, 10]).style(estilo_botao_secundario).on_press(Msg::Copiar(copia)),
            button(text(t("monitor.metricas.limpar")).size(12)).padding([4, 10]).style(estilo_botao_secundario).on_press(Msg::Limpar),
        ]
        .spacing(14)
        .align_y(Alignment::Center);

        let cab = |s: String, w: Length| text(s).size(11).color(COR_TEXTO_MUTED()).width(w);
        let mut tab = column![row![
            cab(t("monitor.metricas.colunas.hora"), Length::Fixed(70.0)),
            cab(t("monitor.metricas.colunas.origem"), Length::Fixed(90.0)),
            cab(t("monitor.metricas.colunas.oQueFoi"), Length::Fill),
            cab(t("monitor.metricas.colunas.tempo"), Length::Fixed(80.0)),
        ]
        .spacing(8)]
        .spacing(3);
        for (quando, m) in vistas.into_iter().take(500) {
            let lenta = m.ms.unwrap_or(0.0) >= LENTA_MS;
            let linha_txt = format!("{quando:.1}s [{}] {} {}", m.origem, m.texto, m.ms.map(|v| format!("{v} ms")).unwrap_or_default());
            let cor = if lenta { Color::from_rgb8(0xc5, 0x6a, 0x00) } else { COR_TEXTO() };
            tab = tab.push(button(
                row![
                    text(format!("{quando:.1}s")).size(12).width(Length::Fixed(70.0)).color(COR_TEXTO_MUTED()),
                    text(format!("[{}]", m.origem)).size(12).width(Length::Fixed(90.0)),
                    text(m.texto).size(12).width(Length::Fill).color(cor),
                    text(m.ms.map(|v| format!("{v} ms")).unwrap_or_default()).size(12).width(Length::Fixed(80.0)).color(cor),
                ]
                .spacing(8),
            ).padding(0).style(button::text).on_press(Msg::Copiar(linha_txt)));
        }
        column![barra, crate::tema::rolagem(tab).height(Length::Fill).width(Length::Fill)].spacing(10).into()
    }
}

fn mb(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
}

fn normalizar_simples(s: &str) -> String {
    crate::ajuda::normalizar(s.trim())
}
