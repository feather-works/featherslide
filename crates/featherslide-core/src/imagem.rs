//! Configurações visuais não destrutivas da foto, persistidas sem JSON.
//! Os bytes originais permanecem no elemento até uma compressão explícita.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VisualImagem {
    pub brilho: f32,
    pub contraste: f32,
    pub nitidez: f32,
    pub saturacao: f32,
    pub temperatura: f32,
    pub matiz: f32,
    pub inverter_horizontal: bool,
    pub inverter_vertical: bool,
    /// 0 = preencher a caixa; 1 = ajustar imagem inteira à caixa.
    pub encaixe: u8,
    /// 0 = normal, 1 = cinza, 2 = sépia, 3 = negativo, 4 = posterizar, 5 = solarizar.
    pub filtro: u8,
    pub reflexo: f32,
    pub brilho_externo: f32,
    pub perspectiva: f32,
    /// Não recomprime bytes até que o usuário confirme explicitamente.
    pub preservar_original: bool,
}

impl Default for VisualImagem {
    fn default() -> Self {
        Self {
            brilho: 0.0, contraste: 0.0, nitidez: 0.0, saturacao: 1.0,
            temperatura: 0.0, matiz: 0.0,
            inverter_horizontal: false, inverter_vertical: false,
            encaixe: 0, filtro: 0, reflexo: 0.0, brilho_externo: 0.0,
            perspectiva: 0.0, preservar_original: true,
        }
    }
}

impl VisualImagem {
    /// Usa parâmetros normalizados, rejeita infinito/NaN e campos não suportados.
    pub fn definir(&mut self, campo: &str, valor: f32) -> Result<(), String> {
        if !valor.is_finite() { return Err("Ajuste da imagem não pode ser NaN ou infinito.".into()); }
        match campo {
            "brilho" => self.brilho = valor.clamp(-1.0, 1.0),
            "contraste" => self.contraste = valor.clamp(-1.0, 1.0),
            "nitidez" => self.nitidez = valor.clamp(0.0, 1.0),
            "saturacao" => self.saturacao = valor.clamp(0.0, 2.0),
            "temperatura" => self.temperatura = valor.clamp(-1.0, 1.0),
            "matiz" => self.matiz = valor.clamp(-180.0, 180.0),
            "inverter_horizontal" => self.inverter_horizontal = valor >= 0.5,
            "inverter_vertical" => self.inverter_vertical = valor >= 0.5,
            "encaixe" => self.encaixe = valor.clamp(0.0, 1.0).round() as u8,
            "filtro" => self.filtro = valor.clamp(0.0, 5.0).round() as u8,
            "reflexo" => self.reflexo = valor.clamp(0.0, 1.0),
            "brilho_externo" => self.brilho_externo = valor.clamp(0.0, 1.0),
            "perspectiva" => self.perspectiva = valor.clamp(-1.0, 1.0),
            "preservar_original" => self.preservar_original = valor >= 0.5,
            _ => return Err(format!("Ajuste de imagem desconhecido: {campo}")),
        }
        Ok(())
    }

    /// Coluna do Arrow: 11 floats normalizados, flags binárias e versão.
    pub fn codificar(&self) -> String {
        let bits = u8::from(self.inverter_horizontal)
            | (u8::from(self.inverter_vertical) << 1)
            | (u8::from(self.preservar_original) << 2);
        format!("{},{},{},{},{},{},{},{},{},{},{},{},1",
            self.brilho, self.contraste, self.nitidez, self.saturacao,
            self.temperatura, self.matiz, self.encaixe, self.filtro,
            self.reflexo, self.brilho_externo, self.perspectiva, bits)
    }

    pub fn decodificar(dados: &str) -> Result<Self, String> {
        if dados.is_empty() { return Ok(Self::default()); }
        let partes: Vec<&str> = dados.split(',').collect();
        if partes.len() != 13 || partes[12] != "1" {
            return Err("Ajustes de imagem incompletos.".into());
        }
        let f = |i: usize| -> Result<f32, String> {
            partes[i].parse::<f32>().map_err(|_| format!("Ajuste inválido na coluna {i}"))
        };
        let bits: u8 = partes[11].parse().map_err(|_| "Bits de imagem inválidos.")?;
        let mut v = Self::default();
        for (nome, valor) in [
            ("brilho",f(0)?), ("contraste",f(1)?), ("nitidez",f(2)?),
            ("saturacao",f(3)?), ("temperatura",f(4)?), ("matiz",f(5)?),
            ("encaixe",f(6)?), ("filtro",f(7)?),
            ("reflexo",f(8)?), ("brilho_externo",f(9)?), ("perspectiva",f(10)?)
        ] { v.definir(nome, valor)?; }
        v.inverter_horizontal = bits & 1 != 0;
        v.inverter_vertical = bits & 2 != 0;
        v.preservar_original = bits & 4 != 0;
        Ok(v)
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn formato_e_validacao() {
        let mut v = VisualImagem::default();
        for (campo, valor) in [("brilho",0.25),("contraste",-0.3),
            ("saturacao",1.5),("temperatura",-0.2),("matiz",40.0),
            ("nitidez",0.7),("filtro",2.0),("encaixe",1.0),
            ("reflexo",0.5),("brilho_externo",0.4),("perspectiva",0.2),
            ("inverter_horizontal",1.0),("inverter_vertical",1.0)] {
            v.definir(campo, valor).unwrap();
        }
        assert_eq!(VisualImagem::decodificar(&v.codificar()).unwrap(), v);
        assert!(VisualImagem::decodificar("2,3").is_err());
        assert!(v.definir("brilho",f32::NAN).is_err());
        assert!(v.definir("nao_existe",0.0).is_err());
    }
}
