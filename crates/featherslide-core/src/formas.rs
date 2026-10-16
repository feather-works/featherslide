//! Formas adicionais: a geometria é calculada no `feather-shapes` e compartilhada por tela,
//! PDF e MCP. Aqui fica a lista do menu do Slide, com o `Tipo` de cada forma.

use crate::modelo::Tipo;

/// Formas adicionais disponíveis na galeria (tipo e rótulo).
pub const ADICIONAIS: &[(Tipo, &str)] = &[
    (Tipo::Losango, "Losango"),
    (Tipo::Circulo, "Círculo"),
    (Tipo::Anel, "Anel"),
    (Tipo::Arco, "Arco"),
    (Tipo::Corda, "Corda"),
    (Tipo::Cubo, "Cubo"),
    (Tipo::Cilindro, "Cilindro"),
    (Tipo::Moldura, "Moldura"),
    (Tipo::CantoEmL, "Canto em L"),
    (Tipo::Placa, "Placa"),
    (Tipo::Sol, "Sol"),
    (Tipo::Lua, "Lua"),
    (Tipo::Coracao, "Coração"),
    (Tipo::Raio, "Raio"),
    (Tipo::Nuvem, "Nuvem"),
    (Tipo::Estrela4, "Estrela (4 pontas)"),
    (Tipo::Estrela6, "Estrela (6 pontas)"),
    (Tipo::Estrela8, "Estrela (8 pontas)"),
    (Tipo::Estrela10, "Estrela (10 pontas)"),
    (Tipo::Estrela12, "Estrela (12 pontas)"),
    (Tipo::TrianguloRetangulo, "Triângulo retângulo"),
    (Tipo::Heptagono, "Heptágono"),
    (Tipo::Decagono, "Decágono"),
    (Tipo::Dodecagono, "Dodecágono"),
    (Tipo::Pentagono, "Pentágono"),
    (Tipo::Hexagono, "Hexágono"),
    (Tipo::Octogono, "Octógono"),
    (Tipo::Trapezio, "Trapézio"),
    (Tipo::Paralelogramo, "Paralelogramo"),
    (Tipo::Estrela, "Estrela"),
    (Tipo::Cruz, "Cruz"),
    (Tipo::SetaEsquerda, "Seta à esquerda"),
    (Tipo::SetaDupla, "Seta dupla"),
    (Tipo::SetaCima, "Seta para cima"),
    (Tipo::SetaBaixo, "Seta para baixo"),
    (Tipo::Chevron, "Chevron"),
    (Tipo::Documento, "Documento"),
];

/// Contorno de uma forma adicional em coordenadas de slide; `None` para tipos anteriores.
/// A geometria vem do `feather-shapes` (`poligonos`); `nome` é o nome do `Tipo` do slide.
pub fn pontos(nome: &str, x: f32, y: f32, largura: f32, altura: f32) -> Option<Vec<[f32; 2]>> {
    let forma = crate::formas_comuns::forma_do_tipo(Tipo::pelo_nome(nome)?)?;
    feather_shapes::poligonos::pontos(forma, x, y, largura, altura)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn todas_as_formas_tem_contorno_e_preservam_o_tipo() {
        for &(tipo, _) in ADICIONAIS {
            assert_eq!(Tipo::pelo_nome(tipo.nome()), Some(tipo));
            if crate::novas_formas::suporta(tipo) {
                let d = crate::novas_formas::desenho(tipo, 10.0, 20.0, 200.0, 100.0).unwrap();
                assert!(!d.contornos.is_empty() || !d.tracos.is_empty());
            } else {
                let p = pontos(tipo.nome(), 10.0, 20.0, 200.0, 100.0).unwrap();
                assert!(p.len() >= 3);
                assert!(p.iter().all(|[x, y]|
                    (10.0..=210.0).contains(x) && (20.0..=120.0).contains(y)));
            }
        }
        assert!(pontos("texto", 0.0, 0.0, 1.0, 1.0).is_none());
    }

    #[test]
    fn novos_poligonos_e_triangulo_retangulo_tem_vertices_corretos() {
        for (tipo, quantidade) in [
            (Tipo::TrianguloRetangulo, 3),
            (Tipo::Heptagono, 7),
            (Tipo::Decagono, 10),
            (Tipo::Dodecagono, 12),
        ] {
            let pts = pontos(tipo.nome(), 10.0, 20.0, 200.0, 100.0).unwrap();
            assert_eq!(pts.len(), quantidade, "{}", tipo.nome());
            assert!(pts.iter().all(|[x, y]| (10.0..=210.0).contains(x) && (20.0..=120.0).contains(y)));
            assert_eq!(Tipo::pelo_nome(tipo.nome()), Some(tipo));
        }
        assert_eq!(
            pontos("triangulo_retangulo", 10.0, 20.0, 200.0, 100.0).unwrap(),
            vec![[10.0, 20.0], [210.0, 120.0], [10.0, 120.0]],
        );
        assert_eq!(Tipo::TODOS[27], Tipo::Caminho, "não alterar códigos dos tipos anteriores");
    }

    #[test]
    fn setas_apontam_para_o_lado_escolhido() {
        for (nome, ponta) in [
            ("seta_esquerda", [0.0, 0.5]),
            ("seta_cima", [0.5, 0.0]),
            ("seta_baixo", [0.5, 1.0]),
        ] {
            assert_eq!(pontos(nome, 0.0, 0.0, 1.0, 1.0).unwrap()[3], ponta);
        }
    }
}
