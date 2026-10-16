//! O motor por fora: os comandos que a interface usa, na ordem em que ela usa.

use featherslide_core::modelo::{Proporcao, Tipo};
use featherslide_core::quadro::Dicas;
use featherslide_core::sessao::Sessao;
use featherslide_core::{Formatacao, Ordem};

fn sessao() -> Sessao {
    let mut s = Sessao::default();
    s.definir_dicas(Dicas { titulo: "Clique para adicionar título".into(), subtitulo: "Subtítulo".into(), corpo: "Texto".into() });
    s
}

#[test]
fn bordas_da_tabela_respeitam_formatacao_e_historico() {
    use featherslide_core::dados::{primitivas_tabela, Primitiva};
    let mut s = sessao();
    s.tabela_inserir(2, 3).unwrap();
    let id = s.estado().selecao[0];
    s.formatar(Formatacao::Contorno("#ff6600".into())).unwrap();
    s.formatar(Formatacao::Espessura(3.5)).unwrap();
    let conferir = |s: &Sessao, largura: f32, cor: &str| {
        let e = s.slide().elementos.iter().find(|e| e.id == id).unwrap();
        let bordas: Vec<_> = primitivas_tabela(e).into_iter().filter_map(|p| {
            if let Primitiva::Ret { contorno, espessura, .. } = p {
                Some((contorno, espessura))
            } else {
                None
            }
        }).collect();
        assert_eq!(bordas.len(), 6);
        assert!(bordas.iter().all(|(c, w)| c == cor && *w == largura));
    };
    conferir(&s, 3.5, "#ff6600");
    s.formatar(Formatacao::Espessura(0.0)).unwrap();
    conferir(&s, 0.0, "");
    s.desfazer().unwrap();
    conferir(&s, 3.5, "#ff6600");
    s.refazer().unwrap();
    conferir(&s, 0.0, "");
    assert_eq!(s.tabela_tamanho(id), Some((2, 3)));
}

#[test]
fn circulo_nativo_mantem_proporcao_e_selecao_buracos() {
    let mut s = sessao();
    s.slide_novo("em_branco").unwrap();
    s.elemento_inserir(Tipo::Circulo, Some([10.0,20.0,160.0,100.0]), None).unwrap();
    let id = s.estado().selecao[0];
    let e = s.slide().elementos.iter().find(|e| e.id==id).unwrap();
    assert_eq!((e.largura, e.altura), (100.0,100.0));
    s.selecao_redimensionar(id, [10.0,20.0,250.0,110.0]).unwrap();
    let e = s.slide().elementos.iter().find(|e| e.id==id).unwrap();
    assert!((e.largura-e.altura).abs()<0.001, "círculo permanece circular");
    s.elemento_inserir(Tipo::Anel, Some([300.0,40.0,140.0,140.0]), None).unwrap();
    s.desmarcar();
    assert_eq!(s.selecionar_no_ponto(370.0,110.0,false),None, "não selecionar o furo do anel");
    assert!(s.selecionar_no_ponto(370.0,50.0,false).is_some());
}

#[test]
fn apresentacao_nova_tem_um_slide_de_titulo_com_as_dicas() {
    let s = sessao();
    let e = s.estado();
    assert_eq!(e.slides.len(), 1);
    assert_eq!(e.layout, "titulo");
    assert!(!e.sujo);
    let q = s.quadro(0, None, true).unwrap().unwrap();
    assert_eq!(q.largura, 960.0);
    assert_eq!(q.elementos.len(), 2);
    assert!(q.elementos[0].dica);
    assert_eq!(q.elementos[0].linhas[0].texto, "Clique para adicionar título");
    let sem_dica = s.quadro(0, None, false).unwrap().unwrap();
    assert!(sem_dica.elementos[0].linhas.is_empty(), "a apresentação não mostra a dica");
}

#[test]
fn quadro_de_versao_conhecida_nao_volta() {
    let mut s = sessao();
    let v = s.quadro(0, None, true).unwrap().unwrap().versao;
    assert!(s.quadro(0, Some(v), true).unwrap().is_none());
    let id = s.apresentacao().slides[0].elementos[0].id;
    s.texto_definir(id, "Olá").unwrap();
    let q = s.quadro(0, Some(v), true).unwrap().expect("mudou");
    assert_eq!(q.elementos[0].linhas[0].texto, "Olá");
    assert!(!q.elementos[0].dica);
}

#[test]
fn slides_novo_duplicar_mover_excluir_e_desfazer() {
    let mut s = sessao();
    s.slide_novo("titulo_conteudo").unwrap();
    assert_eq!(s.atual(), 1);
    s.slide_duplicar().unwrap();
    assert_eq!(s.estado().slides.len(), 3);
    let ids: Vec<u64> = s.estado().slides.iter().map(|x| x.id).collect();
    s.slide_mover(2, 0).unwrap();
    assert_eq!(s.estado().slides[0].id, ids[2]);
    s.slide_excluir().unwrap();
    assert_eq!(s.estado().slides.len(), 2);
    for _ in 0..4 {
        s.desfazer().unwrap();
    }
    assert_eq!(s.estado().slides.len(), 1);
    assert!(s.desfazer().is_none());
    s.refazer().unwrap();
    assert_eq!(s.estado().slides.len(), 2);
}

#[test]
fn nao_exclui_o_ultimo_slide() {
    let mut s = sessao();
    assert!(s.slide_excluir().is_err());
}

#[test]
fn inserir_selecionar_mover_formatar() {
    let mut s = sessao();
    s.slide_novo("em_branco").unwrap();
    s.elemento_inserir(Tipo::Retangulo, Some([100.0, 100.0, 200.0, 100.0]), None).unwrap();
    let id = s.estado().selecao[0];
    s.desmarcar();
    assert_eq!(s.selecionar_no_ponto(150.0, 150.0, false), Some(id));
    assert_eq!(s.selecionar_no_ponto(10.0, 10.0, false), None);
    assert!(s.estado().selecao.is_empty());
    s.selecionar_no_ponto(150.0, 150.0, false);
    s.selecao_mover(10.0, -5.0).unwrap();
    assert_eq!(s.estado().caixas[0], [110.0, 95.0, 200.0, 100.0]);
    s.formatar(Formatacao::Negrito).unwrap();
    s.formatar(Formatacao::Preenchimento("#ff0000".into())).unwrap();
    let f = s.estado().formato.unwrap();
    assert!(f.negrito);
    assert_eq!(f.preenchimento, "#ff0000");
    s.formatar(Formatacao::Negrito).unwrap();
    assert!(!s.estado().formato.unwrap().negrito, "negrito alterna");
}

#[test]
fn texto_longo_quebra_dentro_da_caixa() {
    let mut s = sessao();
    s.slide_novo("em_branco").unwrap();
    s.elemento_inserir(Tipo::Texto, Some([0.0, 0.0, 120.0, 200.0]), None).unwrap();
    let id = s.estado().selecao[0];
    s.texto_definir(id, "uma frase comprida que não cabe numa linha só\nsegundo parágrafo").unwrap();
    let q = s.quadro(1, None, true).unwrap().unwrap();
    let linhas = &q.elementos[0].linhas;
    assert!(linhas.len() >= 4, "{linhas:?}");
    assert!(linhas.windows(2).all(|p| p[1].y > p[0].y), "as linhas descem");
}

#[test]
fn copiar_e_colar_elementos_e_texto_de_fora() {
    let mut s = sessao();
    s.slide_novo("em_branco").unwrap();
    s.elemento_inserir(Tipo::Elipse, None, None).unwrap();
    let texto = s.copiar();
    s.colar(&texto).unwrap();
    assert_eq!(s.estado().elementos, 2);
    s.colar("texto de outro programa").unwrap();
    assert_eq!(s.estado().elementos, 3);
    s.desfazer().unwrap();
    assert_eq!(s.estado().elementos, 2, "colar texto é um passo só no desfazer");
}

#[test]
fn ordem_da_pilha() {
    let mut s = sessao();
    s.slide_novo("em_branco").unwrap();
    s.elemento_inserir(Tipo::Retangulo, Some([0.0, 0.0, 100.0, 100.0]), None).unwrap();
    let baixo = s.estado().selecao[0];
    s.elemento_inserir(Tipo::Retangulo, Some([0.0, 0.0, 100.0, 100.0]), None).unwrap();
    assert_ne!(s.elemento_no_ponto(50.0, 50.0), Some(baixo));
    s.selecionar_no_ponto(50.0, 50.0, false);
    s.organizar(Ordem::Tras).unwrap();
    assert_eq!(s.elemento_no_ponto(50.0, 50.0), Some(baixo));
}

#[test]
fn trocar_layout_guarda_o_texto_do_titulo() {
    let mut s = sessao();
    let id = s.apresentacao().slides[0].elementos[0].id;
    s.texto_definir(id, "Meu título").unwrap();
    s.slide_layout("titulo_conteudo").unwrap();
    let slide = &s.apresentacao().slides[0];
    assert_eq!(slide.elementos.len(), 2);
    assert!(slide.elementos.iter().any(|e| e.texto == "Meu título"));
}

#[test]
fn proporcao_4_3_encolhe_na_horizontal() {
    let mut s = sessao();
    let antes = s.apresentacao().slides[0].elementos[0].largura;
    s.proporcao_definir(Proporcao::Padrao).unwrap();
    assert_eq!(s.estado().largura, 720.0);
    let depois = s.apresentacao().slides[0].elementos[0].largura;
    assert!((depois - antes * 0.75).abs() < 0.01);
}

#[test]
fn o_desenho_do_fundo_nao_se_seleciona_e_fica_na_troca_de_layout() {
    use featherslide_core::modelo::{Elemento, Papel};
    let mut ap = sessao().apresentacao().clone();
    let fundo = Elemento { id: 900, tipo: Tipo::Setor, papel: Papel::Fundo, x: 0.0, y: 0.0, largura: 960.0, altura: 540.0, preenchimento: "#ffffff18".into(), ajustes: vec![0.0, 90.0], ..Elemento::default() };
    ap.slides[0].elementos.insert(0, fundo);
    ap.proximo_id = 901;
    let mut s = Sessao::nova(ap);
    assert_eq!(s.selecionar_no_ponto(5.0, 5.0, false), None, "clique no desenho do fundo não seleciona");
    s.selecionar_tudo();
    assert!(!s.estado().selecao.contains(&900));
    s.selecionar_na_area([0.0, 0.0, 960.0, 540.0], false);
    assert!(!s.estado().selecao.contains(&900));
    s.slide_layout("titulo_conteudo").unwrap();
    assert!(s.apresentacao().slides[0].elementos.iter().any(|e| e.id == 900 && e.papel == Papel::Fundo), "a troca de layout não leva o fundo");
    let q = s.quadro(0, None, true).unwrap().unwrap();
    let d = q.elementos.iter().find(|e| e.id == 900).unwrap();
    assert_eq!((d.tipo, d.ajustes.clone()), ("setor", vec![0.0, 90.0]));
    assert!(d.linhas.is_empty() && !d.dica);
}

#[test]
fn formatacoes_novas_e_listas() {
    let mut s = sessao();
    let id = s.apresentacao().slides[0].elementos[0].id;
    s.selecionar_por_id(id);
    s.texto_definir(id, "Primeira linha\nSegunda linha").unwrap();

    // Tachado e Entrelinha
    s.formatar(Formatacao::Tachado).unwrap();
    s.formatar(Formatacao::Entrelinha(1.5)).unwrap();
    let f = s.estado().formato.unwrap();
    assert!(f.tachado);
    assert!((f.entrelinha - 1.5).abs() < 0.01);

    // Limpar formatação
    s.formatar(Formatacao::Limpar).unwrap();
    let f2 = s.estado().formato.unwrap();
    assert!(!f2.tachado);
    assert!((f2.entrelinha - 1.2).abs() < 0.01);

    // Listas: marcadores
    s.lista_alternar("marcadores").unwrap();
    let txt = s.apresentacao().slides[0].elementos[0].texto.clone();
    assert_eq!(txt, "• Primeira linha\n• Segunda linha");

    // Listas: desalternar marcadores
    s.lista_alternar("marcadores").unwrap();
    let txt2 = s.apresentacao().slides[0].elementos[0].texto.clone();
    assert_eq!(txt2, "Primeira linha\nSegunda linha");

    // Listas: numerada
    s.lista_alternar("numerada").unwrap();
    let txt3 = s.apresentacao().slides[0].elementos[0].texto.clone();
    assert_eq!(txt3, "1. Primeira linha\n2. Segunda linha");
}

#[test]
fn localizar_e_substituir() {
    use featherslide_core::OpcoesDeBusca;
    let mut s = sessao();
    let id = s.apresentacao().slides[0].elementos[0].id;
    s.texto_definir(id, "O pato nada no lago. O pato é feliz.").unwrap();

    let r = s.busca_definir("pato", OpcoesDeBusca::default());
    assert_eq!(r.total, 2);
    assert_eq!(r.atual, 1);

    let r_prox = s.busca_ir(1);
    assert_eq!(r_prox.atual, 2);

    let sub = s.busca_substituir("ganso").unwrap();
    assert_eq!(sub.trocados, 1);
    let txt = s.apresentacao().slides[0].elementos[0].texto.clone();
    assert_eq!(txt, "O pato nada no lago. O ganso é feliz.");

    let sub_tudo = s.busca_substituir_tudo("cisne").unwrap();
    assert_eq!(sub_tudo.trocados, 1);
    let txt_final = s.apresentacao().slides[0].elementos[0].texto.clone();
    assert_eq!(txt_final, "O cisne nada no lago. O ganso é feliz.");
}

#[test]
fn inserir_tabela_e_grafico() {
    use featherslide_core::modelo::Tipo;
    let mut s = sessao();
    let n_antes = s.apresentacao().slides[0].elementos.len();

    // Uma tabela é UM elemento, com os dados dentro.
    s.tabela_inserir(3, 4).unwrap();
    let id = s.estado().selecao[0];
    assert_eq!(s.apresentacao().slides[0].elementos.len(), n_antes + 1);
    assert_eq!(s.tabela_tamanho(id), Some((3, 4)));
    assert_eq!(s.tabela_celula_texto(id, 0, 0).as_deref(), Some("Coluna 1"));

    s.tabela_celula_definir(id, 1, 2, "olá	tab").unwrap();
    assert_eq!(s.tabela_celula_texto(id, 1, 2).as_deref(), Some("olá	tab"));
    s.tabela_estrutura(id, "linha+").unwrap();
    s.tabela_estrutura(id, "coluna-").unwrap();
    assert_eq!(s.tabela_tamanho(id), Some((4, 3)));

    // A célula sob o ponto e o desenho (primitivas) saem do motor.
    let e = s.apresentacao().slides[0].elementos.iter().find(|e| e.id == id).unwrap().clone();
    assert!(e.tipo == Tipo::Tabela);
    let centro = s.tabela_celula_no_ponto(id, e.x + e.largura / 2.0, e.y + 1.0);
    assert_eq!(centro.map(|c| c.0), Some(0));
    let q = s.quadro(0, None, true).unwrap().unwrap();
    let desenhada = q.elementos.iter().find(|d| d.id == id).unwrap();
    assert!(desenhada.primitivas.len() >= 12, "uma tabela 4x3 tem ao menos 12 células");

    // Desfaz a inserção inteira de uma vez.
    for _ in 0..4 {
        s.desfazer().unwrap();
    }
    assert_eq!(s.apresentacao().slides[0].elementos.len(), n_antes);

    // Gráfico: um elemento, dados editáveis e desenho para cada tipo.
    for tipo in ["colunas", "barras", "linhas", "pizza"] {
        s.grafico_inserir(tipo).unwrap();
        let id = s.estado().selecao[0];
        let g = s.grafico_dados(id).unwrap();
        assert_eq!(g.tipo, tipo);
        let q = s.quadro(0, None, true).unwrap().unwrap();
        assert!(q.elementos.iter().find(|d| d.id == id).unwrap().primitivas.len() > 3, "{tipo} desenha");
    }
    let id = s.estado().selecao[0];
    let mut g = s.grafico_dados(id).unwrap();
    g.grade[1][1] = "99".into();
    g.titulo = "Meu título".into();
    s.grafico_definir(id, g).unwrap();
    assert_eq!(s.grafico_dados(id).unwrap().grade[1][1], "99");
}


#[test]
fn tema_transicao_e_animacao() {
    use featherslide_core::modelo::{Animacao, Direcao, InicioAnimacao, TipoAnimacao, TipoTransicao, Transicao};
    let mut s = sessao();
    s.tema_aplicar("escuro").unwrap();
    assert_eq!(s.tema_atual(), "escuro");
    assert_eq!(s.apresentacao().slides[0].fundo, "#202124");
    // Slide novo já nasce no tema.
    s.slide_novo("titulo_conteudo").unwrap();
    assert_eq!(s.slide().fundo, "#202124");
    assert!(s.tema_aplicar("não existe").is_err());

    s.transicao_definir(Transicao { tipo: TipoTransicao::Empurrar, direcao: Direcao::Esquerda, duracao: 1.0, apos: 0.0 }, true).unwrap();
    assert!(s.apresentacao().slides.iter().all(|sl| sl.transicao.tipo == TipoTransicao::Empurrar));
    assert!(s.transicao_definir(Transicao { duracao: 99.0, ..Transicao::default() }, false).is_err());

    // Animações: duas ao clicar e uma com a anterior formam dois passos.
    let ids: Vec<u64> = s.slide().elementos.iter().map(|e| e.id).collect();
    for (i, id) in ids.iter().enumerate() {
        s.selecionar_por_id(*id);
        let inicio = if i == 1 { InicioAnimacao::ComAnterior } else { InicioAnimacao::AoClicar };
        s.animacao_definir(Animacao { tipo: TipoAnimacao::Esmaecer, inicio, ..Animacao::default() }).unwrap();
    }
    let passos = s.passos_de_animacao(s.atual());
    assert_eq!(passos.len(), 1, "{passos:?}");
    assert_eq!(passos[0].elementos.len(), ids.len());
    s.desfazer().unwrap();
}

#[test]
fn soltar_e_mandar_para_o_fundo() {
    use featherslide_core::modelo::{Elemento, Papel, Tipo};
    let mut s = sessao();
    s.slide_mut().elementos.insert(0, Elemento { id: 777, tipo: Tipo::Elipse, papel: Papel::Fundo, x: 10.0, y: 10.0, largura: 50.0, altura: 50.0, preenchimento: "#ff0000".into(), ..Elemento::default() });
    assert!(s.elemento_no_ponto(20.0, 20.0).is_none(), "o fundo não se pega");
    s.fundo_liberar(false).unwrap();
    assert_eq!(s.elemento_no_ponto(20.0, 20.0), Some(777));
    s.selecionar_por_id(777);
    s.selecao_para_fundo().unwrap();
    assert!(s.elemento_no_ponto(20.0, 20.0).is_none());
    assert!(s.fundo_liberar(true).is_ok());
    assert!(s.fundo_liberar(true).is_err(), "nada mais a soltar");
}

#[test]
fn secoes_agrupam_slides() {
    let mut s = sessao();
    for _ in 0..4 {
        s.slide_novo("titulo_conteudo").unwrap();
    }
    assert!(s.secoes().is_empty());
    s.slide_ir(2).unwrap();
    s.secao_adicionar("Resultados").unwrap();
    let r = s.secoes();
    assert_eq!(r.len(), 2, "{r:?}");
    assert!(!r[0].implicita && r[0].nome == "Seção padrão" && (r[0].inicio, r[0].fim) == (0, 2));
    assert_eq!((r[1].nome.as_str(), r[1].inicio, r[1].fim), ("Resultados", 2, 5));
    // Agora o segundo pedido cria um novo slide e uma seção, sem apagar Resultados.
    let ids: Vec<u64> = s.apresentacao().slides.iter().map(|slide| slide.id).collect();
    s.secao_adicionar("de novo").unwrap();
    assert_eq!(s.secoes().len(), 3);
    assert_eq!(s.secoes()[2].nome, "de novo");
    assert_eq!(s.apresentacao().slides.len(), 6);
    assert_eq!(s.apresentacao().slides.iter().take(5).map(|slide| slide.id).collect::<Vec<_>>(), ids);
    s.desfazer().unwrap();
    assert_eq!(s.secoes().len(), 2);

    // Duplicar o slide que abre a seção não duplica a abertura.
    s.slide_duplicar().unwrap();
    assert!(s.apresentacao().slides[3].secao.is_empty());
    // Excluir quem abre passa a abertura ao seguinte.
    s.slide_ir(2).unwrap();
    s.slide_excluir().unwrap();
    assert_eq!(s.apresentacao().slides[2].secao, "Resultados");
    // Mover slides não anda a fronteira.
    s.slide_mover(0, 3).unwrap();
    assert_eq!(s.secoes()[1].inicio, 2);

    s.secao_renomear(2, "Conclusão").unwrap();
    assert_eq!(s.secoes()[1].nome, "Conclusão");
    s.secao_remover(2, false).unwrap();
    assert_eq!(s.apresentacao().slides.len(), 5);
    s.secao_remover(0, true).unwrap_or_default();
    s.secoes_remover_todas().unwrap();
    assert!(s.secoes().is_empty());
}

#[test]
fn todo_modelo_abre_com_slides_e_tema() {
    use featherslide_core::modelos::{criar, MODELOS};
    for m in MODELOS.iter() {
        let ap = criar(m.id).unwrap();
        assert!(!ap.slides.is_empty(), "{}", m.id);
        let s = Sessao::nova(ap);
        assert!(s.quadro(0, None, false).unwrap().is_some());
    }
    assert!(criar("nao_existe").is_none());
    assert!(criar("negocios").unwrap().slides.len() >= 5);
}

#[test]
fn slide_novo_e_layout_de_apresentacao_de_modelo_levam_o_enfeite() {
    use featherslide_core::modelo::Papel;
    use featherslide_core::modelos::criar;
    let mut s = Sessao::nova(criar("aula").unwrap());
    s.slide_novo("titulo_conteudo").unwrap();
    let enfeites = |s: &Sessao| s.slide().elementos.iter().filter(|e| e.papel == Papel::Fundo).count();
    assert!(enfeites(&s) > 0);
    s.slide_layout("secao").unwrap();
    assert!(enfeites(&s) > 0);
    let q = s.quadro_de_layout("dois_conteudos");
    assert!(q.elementos.len() > 3);
    // sem modelo, nada de enfeite
    let mut limpa = Sessao::default();
    limpa.slide_novo("titulo_conteudo").unwrap();
    assert_eq!(enfeites(&limpa), 0);
}

#[test]
fn todo_layout_da_lista_cria_slide() {
    for l in featherslide_core::layouts::LAYOUTS {
        let mut s = Sessao::default();
        s.slide_novo(l).unwrap();
        assert_eq!(s.slide().layout, l);
        assert!(s.quadro(1, None, true).unwrap().is_some());
    }
}

#[test]
fn layouts_novos_em_todo_modelo_ficam_dentro_do_slide() {
    use featherslide_core::modelos::{criar, MODELOS};
    for m in MODELOS.iter() {
        for l in featherslide_core::layouts::LAYOUTS {
            let mut s = Sessao::nova(criar(m.id).unwrap());
            s.slide_novo(l).unwrap();
            let q = s.quadro(s.atual(), None, true).unwrap().unwrap();
            for e in &q.elementos {
                assert!(e.largura >= 0.0 && e.altura >= 0.0, "{} / {l}", m.id);
            }
            assert!(q.elementos.iter().all(|e| e.x >= -400.0 && e.x <= q.largura + 400.0), "{} / {l}", m.id);
        }
    }
}

#[test]
fn ajustes_da_selecao_chegam_ao_quadro() {
    let mut s = Sessao::default();
    s.elemento_inserir(Tipo::Retangulo, None, None).unwrap();
    assert_eq!(s.estado().selecao.len(), 1);
    s.selecao_ajustes(vec![0.2, 0.0, 0.0, 0.0]).unwrap();
    let q = s.quadro(0, None, true).unwrap().unwrap();
    let r = q.elementos.iter().find(|e| e.tipo == "retangulo").unwrap();
    assert_eq!(r.ajustes, vec![0.2, 0.0, 0.0, 0.0]);
}

#[test]
fn alinhar_entre_si_e_distribuir() {
    let mut s = Sessao::default();
    s.slide_novo("em_branco").unwrap();
    for (x, y) in [(10.0, 20.0), (300.0, 80.0), (100.0, 150.0)] {
        s.elemento_inserir(Tipo::Retangulo, Some([x, y, 50.0, 40.0]), None).unwrap();
    }
    s.selecionar_tudo();
    assert!(s.alinhar_entre_si("esquerda").is_ok());
    let xs: Vec<f32> = s.slide().elementos.iter().filter(|e| e.tipo == Tipo::Retangulo).map(|e| e.x).collect();
    assert!(xs.iter().all(|&x| x == 10.0));
    s.selecionar_tudo();
    s.distribuir("v").unwrap();
    let mut ys: Vec<f32> = s.slide().elementos.iter().filter(|e| e.tipo == Tipo::Retangulo).map(|e| e.y).collect();
    ys.sort_by(|a, b| a.total_cmp(b));
    assert!(((ys[1] - ys[0]) - (ys[2] - ys[1])).abs() < 0.01);
}

#[test]
fn secao_dos_slides_junta_e_nomeia() {
    let mut s = Sessao::default();
    for _ in 0..4 {
        s.slide_novo("titulo_conteudo").unwrap();
    }
    // 5 slides; marca o 2º e o 4º: viram uma seção junta, começando no lugar do 2º.
    let inicio = s.secao_dos_slides(&[1, 3], "Meio").unwrap();
    assert_eq!(inicio, 1);
    let nomes: Vec<String> = s.secoes().iter().map(|x| x.nome.clone()).collect();
    assert_eq!(nomes, vec!["Seção padrão", "Meio", "Continuação"]);
    let sec = s.secoes();
    assert_eq!((sec[1].inicio, sec[1].fim), (1, 3));
}

#[test]
fn ocultar_agrupar_e_listar_elementos() {
    let mut s = Sessao::default();
    s.slide_novo("em_branco").unwrap();
    for x in [10.0, 200.0, 400.0] {
        s.elemento_inserir(Tipo::Retangulo, Some([x, 50.0, 100.0, 80.0]), None).unwrap();
    }
    let lista = s.lista_de_elementos();
    assert_eq!(lista.len(), 3);
    // agrupar os dois de cima: clicar num seleciona o outro junto
    s.selecionar_por_id(lista[0].id);
    s.selecao_mover(0.0, 0.0).unwrap();
    s.selecionar_tudo();
    s.agrupar().unwrap();
    s.selecionar_por_id(lista[1].id);
    assert_eq!(s.estado().selecao.len(), 3);
    s.desagrupar().unwrap();
    s.selecionar_por_id(lista[1].id);
    assert_eq!(s.estado().selecao.len(), 1);
    // ocultar tira do quadro e do clique
    s.elemento_ocultar(lista[0].id, true).unwrap();
    let q = s.quadro(s.atual(), None, true).unwrap().unwrap();
    assert_eq!(q.elementos.len(), 2);
    assert_eq!(s.elemento_no_ponto(450.0, 90.0), None);
    assert!(s.lista_de_elementos().iter().any(|i| i.id == lista[0].id && i.oculto));
    s.elemento_ocultar(lista[0].id, false).unwrap();
    assert_eq!(s.quadro(s.atual(), None, true).unwrap().unwrap().elementos.len(), 3);
}

#[test]
fn recorte_de_imagem_altera_forma_e_quadro() {
    let mut s = Sessao::default();
    s.slide_novo("em_branco").unwrap();
    s.elemento_inserir(Tipo::Imagem, Some([100.0, 100.0, 200.0, 200.0]), Some("data:image/png;base64,abc".into())).unwrap();
    assert_eq!(s.selecao_recorte(), Some(Tipo::Retangulo));

    s.elemento_recorte(Tipo::Elipse).unwrap();
    assert_eq!(s.selecao_recorte(), Some(Tipo::Elipse));

    let q = s.quadro(s.atual(), None, true).unwrap().unwrap();
    let img = q.elementos.iter().find(|e| e.tipo == "imagem").expect("imagem no quadro");
    assert_eq!(img.recorte, "elipse");

    s.elemento_recorte(Tipo::RetanguloArredondado).unwrap();
    assert_eq!(s.selecao_recorte(), Some(Tipo::RetanguloArredondado));
    let q2 = s.quadro(s.atual(), None, true).unwrap().unwrap();
    let img2 = q2.elementos.iter().find(|e| e.tipo == "imagem").expect("imagem no quadro");
    assert_eq!(img2.recorte, "retangulo_arredondado");

    s.desfazer().unwrap();
    assert_eq!(s.selecao_recorte(), Some(Tipo::Elipse));
}

#[test]
fn imagem_ajustar_e_resetar_enquadramento() {
    let mut s = Sessao::default();
    s.slide_novo("em_branco").unwrap();
    s.elemento_inserir(Tipo::Imagem, Some([100.0, 100.0, 200.0, 200.0]), Some("data:image/png;base64,abc".into())).unwrap();

    // Ajusta deslocamento e zoom
    s.imagem_ajustar_recorte(0.15, -0.25, 0.5).unwrap();
    let id = s.estado().selecao[0];
    let el = s.slide().elementos.iter().find(|e| e.id == id).unwrap();
    assert_eq!(el.ajustes[0], 0.15);
    assert_eq!(el.ajustes[1], -0.25);
    assert_eq!(el.ajustes[2], 1.5);

    // Ajusta novamente acumulando
    s.imagem_ajustar_recorte(-0.05, 0.1, -0.2).unwrap();
    let el2 = s.slide().elementos.iter().find(|e| e.id == id).unwrap();
    assert!((el2.ajustes[0] - 0.10).abs() < 1e-4);
    assert!((el2.ajustes[1] - (-0.15)).abs() < 1e-4);
    assert!((el2.ajustes[2] - 1.3).abs() < 1e-4);

    // Reseta recorte
    s.imagem_resetar_recorte().unwrap();
    let el3 = s.slide().elementos.iter().find(|e| e.id == id).unwrap();
    assert_eq!(el3.ajustes[0], 0.0);
    assert_eq!(el3.ajustes[1], 0.0);
    assert_eq!(el3.ajustes[2], 1.0);
}


#[test]
fn previa_de_cor_vira_um_unico_passo_do_desfazer() {
    let mut s = sessao();
    s.slide_novo("em_branco").unwrap();
    s.elemento_inserir(Tipo::Retangulo, Some([100.0, 100.0, 200.0, 100.0]), None).unwrap();
    s.formatar(Formatacao::Preenchimento("#111111".into())).unwrap();
    let passos = |s: &mut Sessao| {
        let mut n = 0;
        while s.desfazer().is_some() {
            n += 1;
        }
        n
    };
    let mut copia = sessao();
    copia.slide_novo("em_branco").unwrap();
    copia.elemento_inserir(Tipo::Retangulo, Some([100.0, 100.0, 200.0, 100.0]), None).unwrap();
    copia.formatar(Formatacao::Preenchimento("#111111".into())).unwrap();
    let base = passos(&mut copia);

    // Arrastar no seletor: 50 mudanças de cor, nenhuma entra no desfazer sozinha.
    for i in 0..50 {
        s.formatar_previa(Formatacao::Preenchimento(format!("#{i:02x}00ff"))).unwrap();
    }
    assert_eq!(s.estado().formato.unwrap().preenchimento, "#3100ff");
    s.confirmar_previa();
    assert_eq!(passos(&mut s), base + 1, "só um passo a mais: cor antiga → cor nova");
}

#[test]
fn desfazer_depois_da_previa_volta_a_cor_antiga() {
    let mut s = sessao();
    s.slide_novo("em_branco").unwrap();
    s.elemento_inserir(Tipo::Retangulo, Some([100.0, 100.0, 200.0, 100.0]), None).unwrap();
    s.formatar(Formatacao::Preenchimento("#111111".into())).unwrap();
    s.formatar_previa(Formatacao::Preenchimento("#222222".into())).unwrap();
    s.formatar_previa(Formatacao::Preenchimento("#333333".into())).unwrap();
    s.desfazer().unwrap();
    assert_eq!(s.estado().formato.unwrap().preenchimento, "#111111");
}

#[test]
fn desfazer_nao_copia_os_bytes_da_imagem() {
    let mut s = sessao();
    s.slide_novo("em_branco").unwrap();
    let grande = format!("data:image/png;base64,{}", "A".repeat(2_000_000));
    s.elemento_inserir(Tipo::Imagem, Some([10.0, 10.0, 100.0, 100.0]), Some(grande)).unwrap();
    let antes = s.apresentacao().slides[1].elementos.last().unwrap().imagem.clone();
    // Dez edições: cada uma guarda um retrato da apresentação inteira no desfazer.
    for i in 0..10 {
        s.formatar(Formatacao::Contorno(format!("#00{i}000"))).unwrap();
    }
    let depois = s.apresentacao().slides[1].elementos.last().unwrap().imagem.clone();
    assert!(antes.compartilha(&depois), "a imagem continua sendo os mesmos bytes, não uma cópia por edição");
    assert_eq!(depois.len(), 2_000_000 + "data:image/png;base64,".len());
}

#[test]
fn redimensionar_selecao_preserva_proporcoes_e_desfaz_o_conjunto() {
    let mut s = Sessao::default();
    s.slide_novo("em_branco").unwrap();
    s.elemento_inserir(Tipo::Retangulo, Some([10.0, 20.0, 200.0, 100.0]), None).unwrap();
    let id = s.estado().selecao[0];
    s.elemento_inserir(Tipo::Elipse, Some([310.0, 220.0, 100.0, 50.0]), None).unwrap();
    s.selecionar_tudo();
    let antes = s.apresentacao().clone();
    s.selecao_redimensionar(id, [10.0, 20.0, 100.0, 90.0]).unwrap();
    let a = &s.slide().elementos[0];
    let b = &s.slide().elementos[1];
    assert_eq!([a.x, a.y, a.largura, a.altura], [10.0, 20.0, 100.0, 50.0]);
    assert_eq!([b.x, b.y, b.largura, b.altura], [160.0, 120.0, 50.0, 25.0]);
    s.desfazer().unwrap();
    let mut conteudo_restaurado = s.apresentacao().clone();
    assert!(conteudo_restaurado.metadados.revisoes >= antes.metadados.revisoes);
    // O histórico automático avança, mas a geometria do documento deve ser idêntica.
    conteudo_restaurado.metadados = antes.metadados.clone();
    assert_eq!(conteudo_restaurado, antes);
    assert!(s.selecao_redimensionar(id, [0.0, 0.0, f32::NAN, 20.0]).is_err());
}

#[test]
fn trocar_icone_preserva_geometria_e_cor() {
    let mut s = Sessao::default();
    s.icone_inserir("home").unwrap();
    let antes = s.slide().elementos.last().unwrap().clone();
    s.icone_substituir("heart").unwrap();
    let mut esperado = antes.clone();
    esperado.dados = "heart".into();
    assert_eq!(s.slide().elementos.last().unwrap(), &esperado);
    s.desfazer().unwrap();
    assert_eq!(s.slide().elementos.last().unwrap(), &antes);
    assert!(s.icone_substituir("icone-inexistente").is_err());
}
