//! Os monitores ligados: de onde sai o segundo para a plateia na apresentação com dois monitores.
//! Lê `xrandr --listmonitors` (vale no X11 e no XWayland); sem ele, devolve a lista vazia e a
//! apresentação fica numa tela só.

#[derive(Clone, Debug, PartialEq)]
pub struct Monitor {
    pub nome: String,
    pub x: i32,
    pub y: i32,
    pub largura: u32,
    pub altura: u32,
    pub principal: bool,
}

/// Uma linha como ` 0: +*HDMI-1 1920/527x1080/296+0+0  HDMI-1`.
fn ler_linha(linha: &str) -> Option<Monitor> {
    let (_, resto) = linha.split_once(':')?;
    let mut campos = resto.split_whitespace();
    let nome_com_marcas = campos.next()?;
    let geometria = campos.next()?;
    let nome = campos.next().unwrap_or(nome_com_marcas).to_string();
    // 1920/527x1080/296+0+0
    let (tamanho, posicao) = geometria.split_once('+')?;
    let (largura_txt, altura_txt) = tamanho.split_once('x')?;
    let largura = largura_txt.split('/').next()?.parse().ok()?;
    let altura = altura_txt.split('/').next()?.parse().ok()?;
    let mut p = posicao.split('+');
    let x = p.next()?.parse().ok()?;
    let y = p.next()?.parse().ok()?;
    Some(Monitor { nome, x, y, largura, altura, principal: nome_com_marcas.contains('*') })
}

pub fn listar() -> Vec<Monitor> {
    let Ok(saida) = std::process::Command::new("xrandr").arg("--listmonitors").output() else { return Vec::new() };
    if !saida.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&saida.stdout).lines().filter_map(ler_linha).collect()
}

/// O monitor da plateia: o primeiro que não é o principal.
pub fn da_plateia(todos: &[Monitor]) -> Option<&Monitor> {
    if todos.len() < 2 {
        return None;
    }
    todos.iter().find(|m| !m.principal).or_else(|| todos.get(1))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn le_a_saida_do_xrandr() {
        let m = ler_linha(" 1: +HDMI-1 1920/527x1080/296+1920+0  HDMI-1").unwrap();
        assert_eq!((m.x, m.y, m.largura, m.altura, m.principal), (1920, 0, 1920, 1080, false));
        assert_eq!(m.nome, "HDMI-1");
        let p = ler_linha(" 0: +*eDP-1 1366/310x768/170+0+0  eDP-1").unwrap();
        assert!(p.principal);
        assert_eq!(da_plateia(&[p.clone(), m.clone()]).unwrap().nome, "HDMI-1");
        assert!(da_plateia(&[p]).is_none());
    }
}
