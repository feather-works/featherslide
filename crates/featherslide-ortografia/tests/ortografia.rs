use featherslide_core::sessao::Sessao;
use featherslide_ortografia::Corretor;
use spellbook::Dictionary;

fn corretor() -> Corretor {
    let d = Dictionary::new("SET UTF-8\n", "5\nolá\nmundo\ncasa\nbonita\nacento\n").unwrap();
    Corretor::de_dicionarios(vec![d])
}

#[test]
fn acha_corrige_e_desfaz() {
    let mut s = Sessao::default();
    let id = s.apresentacao().slides[0].elementos[0].id;
    s.texto_definir(id, "Olá mundo, a casa bnita").unwrap();
    let c = corretor();
    let erros = c.verificar(s.apresentacao());
    assert_eq!(erros.len(), 1);
    assert_eq!(erros[0].palavra, "bnita");
    assert_eq!((erros[0].slide, erros[0].elemento), (0, id));
    c.corrigir(&mut s, &erros[0], "bonita").unwrap();
    assert!(c.verificar(s.apresentacao()).is_empty());
    assert_eq!(s.slide().elementos[0].texto, "Olá mundo, a casa bonita");
    s.desfazer().unwrap();
    assert_eq!(c.verificar(s.apresentacao()).len(), 1);
    // Texto que mudou depois da verificação não é trocado às cegas.
    let velho = erros[0].clone();
    s.texto_definir(id, "outra coisa").unwrap();
    assert!(c.corrigir(&mut s, &velho, "bonita").is_err());
}

#[test]
fn ignorar_vale_para_qualquer_caixa() {
    let mut c = corretor();
    assert!(!c.palavra_ok("Xyzzy"));
    c.ignorar("xyzzy");
    assert!(c.palavra_ok("XYZZY"));
}

#[test]
fn sugere_a_palavra_com_acento_quando_somam_dois_erros() {
    let aff = "SET UTF-8\n";
    let dic = "4\nimportância\nimportante\nportanto\ncasa\n";
    let c = Corretor::de_texto(aff, dic).unwrap();
    let s = c.sugestoes("inportancia");
    assert_eq!(s.first().map(String::as_str), Some("importância"), "{s:?}");
    assert_eq!(c.sugestoes("Inportancia").first().map(String::as_str), Some("Importância"));
}

#[test]
fn ondulados_ficam_dentro_da_caixa_e_apontam_o_erro() {
    let mut s = Sessao::default();
    let id = s.apresentacao().slides[0].elementos[0].id;
    s.texto_definir(id, "casa bnita casa bnita").unwrap();
    let c = corretor();
    let q = s.quadro(0, None, false).unwrap().unwrap();
    let o = c.ondulados(0, s.slide(), &q, false);
    assert_eq!(o.len(), 2);
    let caixa = q.elementos.iter().find(|d| d.id == id).unwrap();
    for w in &o {
        assert_eq!(w.erro.palavra, "bnita");
        assert!(w.x >= caixa.x && w.x + w.largura <= caixa.x + caixa.largura + 1.0, "{w:?}");
    }
    // Cada ondulação leva ao seu próprio trecho do texto.
    assert!(o[0].erro.inicio < o[1].erro.inicio);
    // A segunda é a que a pessoa clicaria: trocar só ela.
    c.corrigir(&mut s, &o[1].erro, "bonita").unwrap();
    assert_eq!(s.slide().elementos[0].texto, "casa bnita casa bonita");
}

#[test]
fn instalar_valida_o_par_dic_aff() {
    use std::io::Write;
    let pasta = std::env::temp_dir().join(format!("fs-orto-{}", std::process::id()));
    std::fs::create_dir_all(&pasta).unwrap();
    let dic = pasta.join("xx_YY.dic");
    std::fs::File::create(&dic).unwrap().write_all(b"1\nfoo\n").unwrap();
    // Sem o .aff ao lado, não instala; com extensão errada, também não.
    assert!(featherslide_ortografia::instalar_dicionario(&dic).is_err());
    assert!(featherslide_ortografia::instalar_dicionario(&pasta.join("xx_YY.txt")).is_err());
    std::fs::remove_dir_all(&pasta).ok();
}

#[test]
fn gramatica_aparece_como_ondulado_azul_com_a_troca() {
    use featherslide_ortografia::TipoOndulado;
    let mut s = Sessao::default();
    let id = s.apresentacao().slides[0].elementos[0].id;
    s.texto_definir(id, "eu vou ao casa").unwrap();
    let mut c = Corretor::de_texto("SET UTF-8\n", "5\neu\nvou\nao\ncasa\nxx\n").unwrap();
    let q = s.quadro(0, None, false).unwrap().unwrap();
    assert!(c.ondulados(0, s.slide(), &q, false).is_empty(), "com a gramática desligada não marca");
    let o = c.ondulados(0, s.slide(), &q, true);
    assert_eq!(o.len(), 1, "{o:?}");
    assert_eq!((o[0].tipo, o[0].sugestao.as_str(), o[0].erro.palavra.as_str()), (TipoOndulado::Gramatica, "à", "ao"));
    // Trocar pelo `erro` + sugestão conserta a frase; ignorar apaga o aviso.
    c.trocar(&mut s, &o[0].erro, &o[0].sugestao).unwrap();
    assert_eq!(s.slide().elementos[0].texto, "eu vou à casa");
    s.desfazer().unwrap();
    c.ignorar_gramatica(&o[0].chave);
    let q = s.quadro(0, None, false).unwrap().unwrap();
    assert!(c.ondulados(0, s.slide(), &q, true).is_empty());
}

#[test]
fn troca_literal_aceita_vazio_e_espacos() {
    let mut s = Sessao::default();
    let id = s.apresentacao().slides[0].elementos[0].id;
    s.texto_definir(id, "casa , mesa").unwrap();
    let c = Corretor::de_texto("SET UTF-8\n", "3\ncasa\nmesa\nxx\n").unwrap();
    let q = s.quadro(0, None, false).unwrap().unwrap();
    let o = c.ondulados(0, s.slide(), &q, true);
    assert_eq!(o.len(), 1, "{o:?}");
    assert!(o[0].sugestao.is_empty());
    c.trocar(&mut s, &o[0].erro, &o[0].sugestao).unwrap();
    assert_eq!(s.slide().elementos[0].texto, "casa, mesa");
    s.texto_definir(id, "casa,mesa").unwrap();
    let q = s.quadro(0, None, false).unwrap().unwrap();
    let o = c.ondulados(0, s.slide(), &q, true);
    c.trocar(&mut s, &o[0].erro, &o[0].sugestao).unwrap();
    assert_eq!(s.slide().elementos[0].texto, "casa, mesa", "a vírgula leva o espaço junto");
}
