//! Textos da interface mantidos nos JSONs de localidade.
use serde_json::Value;
use std::sync::OnceLock;

pub fn texto(chave: &str) -> &'static str {
    static PT: OnceLock<Value> = OnceLock::new();
    let dados = PT.get_or_init(|| {
        serde_json::from_str(include_str!("../dados/pt-BR.json"))
            .expect("Localidade pt-BR inválida")
    });
    dados
        .get("ferramentas_texto")
        .and_then(|v| v.get(chave))
        .and_then(Value::as_str)
        .unwrap_or("Texto indisponível")
}

#[cfg(test)]
mod tests {
    #[test]
    fn localidades_tem_as_mesmas_chaves_de_ferramentas() {
        let pt: serde_json::Value =
            serde_json::from_str(include_str!("../dados/pt-BR.json")).unwrap();
        let en: serde_json::Value =
            serde_json::from_str(include_str!("../dados/en-US.json")).unwrap();
        let chaves = |v: &serde_json::Value| {
            v["ferramentas_texto"]
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>()
        };
        assert_eq!(chaves(&pt), chaves(&en));
        assert!(!chaves(&pt).is_empty());
    }
}
