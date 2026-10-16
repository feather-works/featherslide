//! API histórica Base64 do FeatherSlide.
//!
//! O algoritmo RFC 4648 e a conversão de `data:` URL agora vivem em
//! `featheroffice-image`. Reexportamos os quatro nomes para manter todos
//! os consumidores (FSDZ/PPTX/ODP, GUI e exemplos) compatíveis.

pub use feather_image::base64::{codificar, de_data_url, decodificar, para_data_url};

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn api_legada_mantem_assinaturas_e_conteudo_fsdz() {
        let bytes = b"Slide 1: \xc3\xa7 e \xce\xa9";
        let encoded = codificar(bytes);
        assert_eq!(decodificar(&encoded).as_deref(), Some(bytes.as_slice()));
        let url = para_data_url("image/png", bytes);
        assert_eq!(de_data_url(&url), Some(("image/png".into(), bytes.to_vec())));
        assert_eq!(de_data_url("https://exemplo.com/figura.png"), None);
    }
}
