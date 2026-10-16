//! Escala uniforme de uma seleção, ancorada no elemento cuja alça foi arrastada.

/// Transformação comum: todos os objetos mantêm as proporções e as distâncias relativas.
#[derive(Clone, Copy, Debug)]
pub struct Escala {
    pub fator: f32,
    pub dx: f32,
    pub dy: f32,
}

impl Escala {
    /// Calcula a escala pelo lado que mais mudou; aceita linhas com um dos lados nulo.
    pub fn entre(antes: [f32; 4], depois: [f32; 4]) -> Option<Self> {
        if !antes.iter().chain(depois.iter()).all(|v| v.is_finite()) {
            return None;
        }
        let [x, y, w, h] = antes;
        let [nx, ny, nw, nh] = depois;
        let fw = (w.abs() > 0.001).then(|| nw / w);
        let fh = (h.abs() > 0.001).then(|| nh / h);
        let fator = match (fw, fh) {
            (Some(a), Some(b)) => {
                if (a - 1.0).abs() >= (b - 1.0).abs() {
                    a
                } else {
                    b
                }
            }
            (Some(a), None) | (None, Some(a)) => a,
            _ => return None,
        };
        if fator <= 0.0 || !fator.is_finite() {
            return None;
        }
        // O lado oposto à alça permanece no mesmo lugar também quando se puxa só uma dimensão.
        let ax = if (nx - x).abs() > 0.001 { x + w } else { x };
        let ay = if (ny - y).abs() > 0.001 { y + h } else { y };
        Some(Self {
            fator,
            dx: ax * (1.0 - fator),
            dy: ay * (1.0 - fator),
        })
    }

    /// Aplica a transformação a um retângulo do slide.
    pub fn retangulo(self, [x, y, w, h]: [f32; 4]) -> [f32; 4] {
        [
            x * self.fator + self.dx,
            y * self.fator + self.dy,
            w * self.fator,
            h * self.fator,
        ]
    }
}
