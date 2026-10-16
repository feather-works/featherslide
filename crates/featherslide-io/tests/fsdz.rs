//! O `.fsdz` grava e reabre igual (a ida e volta), e o arquivo ruim dá erro, não pânico.

use featherslide_core::modelo::{Papel, Proporcao, Sombra, Tipo};
use featherslide_core::sessao::Sessao;
use featherslide_core::Formatacao;
use featherslide_io::fsdz;

#[test]
fn outras_galerias_conectores_e_traco_livre_persistem() {
    let mut s=Sessao::default();s.slide_novo("em_branco").unwrap();
    s.elemento_inserir(Tipo::Retangulo,Some([10.0,10.0,100.0,100.0]),None).unwrap();
    s.elemento_inserir(Tipo::Retangulo,Some([300.0,200.0,100.0,100.0]),None).unwrap();
    let alvo=s.slide().elementos[1].id;
    s.selecionar_tudo();s.conector_inserir("curvo").unwrap();
    let conector=s.slide().elementos.last().unwrap().id;
    for &(_,id,_) in featherslide_core::outras_galerias::FORMAS{s.galeria_inserir(id).unwrap();}
    s.desenho_livre_inserir(&[[30.0,40.0],[55.0,90.0],[80.0,50.0]]).unwrap();
    let ap=fsdz::ler(&fsdz::gravar(s.apresentacao()).unwrap()).unwrap();assert_eq!(&ap,s.apresentacao());
    let mut reaberta=Sessao::nova(ap);reaberta.slide_ir(1).unwrap();
    let antes=reaberta.slide().elementos.iter().find(|e|e.id==conector).unwrap().clone();
    reaberta.selecionar_por_id(alvo);reaberta.selecao_mover(90.0,30.0).unwrap();
    let depois=reaberta.slide().elementos.iter().find(|e|e.id==conector).unwrap();assert_ne!(antes.largura,depois.largura);
}

/// Um PNG de 1×1 em `data:`.
const PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

fn apresentacao_variada() -> Sessao {
    let mut s = Sessao::default();
    let id = s.apresentacao().slides[0].elementos[0].id;
    s.texto_definir(id, "Título com acentuação: ação, café").unwrap();
    s.slide_novo("titulo_conteudo").unwrap();
    s.notas_definir("lembrar de falar do orçamento").unwrap();
    s.elemento_inserir(Tipo::Elipse, Some([10.0, 20.0, 30.0, 40.0]), None).unwrap();
    s.formatar(Formatacao::Negrito).unwrap();
    s.formatar(Formatacao::Preenchimento("#34a853".into())).unwrap();
    s.elemento_inserir(Tipo::Imagem, None, Some(PNG.into())).unwrap();
    s.slide_fundo("#fef7e0", false).unwrap();
    s.slide_ocultar().unwrap();
    s.proporcao_definir(Proporcao::Padrao).unwrap();
    s
}

#[test]
fn ida_e_volta_igual() {
    let s = apresentacao_variada();
    let bytes = fsdz::gravar(s.apresentacao()).unwrap();
    let lida = fsdz::ler(&bytes).unwrap();
    assert_eq!(&lida, s.apresentacao());
}

#[test]
fn ida_e_volta_com_setor_recorte_sombra_e_fundo() {
    let mut ap = apresentacao_variada().apresentacao().clone();
    let elementos = &mut ap.slides[1].elementos;
    elementos[0].tipo = Tipo::Setor;
    elementos[0].papel = Papel::Fundo;
    elementos[0].ajustes = vec![90.0, 270.0];
    elementos[0].preenchimento = "#ffffff18".into();
    let foto = elementos.iter_mut().find(|e| e.tipo == Tipo::Imagem).unwrap();
    foto.recorte = Tipo::Elipse;
    foto.sombra = Sombra { cor: "#000000cc".into(), desfoque: 16.0, dx: 1.5, dy: -2.0 };
    let lida = fsdz::ler(&fsdz::gravar(&ap).unwrap()).unwrap();
    assert_eq!(lida, ap);
}

#[test]
fn manipulacao_da_imagem_persiste_no_fsdz_v8_e_desfaz_os_ajustes() {
    let mut s = Sessao::default();
    s.elemento_inserir(Tipo::Imagem, Some([90.0,55.0,280.0,160.0]), Some(PNG.into())).unwrap();
    let id = s.estado().selecao[0];
    let original = s.slide().elementos.iter().find(|e| e.id==id).unwrap().imagem.clone();
    for (campo,valor) in [
        ("brilho",0.35),("contraste",-0.2),("nitidez",0.5),
        ("saturacao",1.5),("temperatura",0.4),("matiz",90.0),
        ("inverter_horizontal",1.0),("inverter_vertical",1.0),
        ("encaixe",1.0),("filtro",2.0),("reflexo",0.6),
        ("brilho_externo",0.8),("perspectiva",0.3),
    ] {
        s.imagem_visual_definir(campo,valor).unwrap();
    }
    s.imagem_cortar_margem("esquerda",0.2).unwrap();
    s.imagem_cortar_margem("topo",0.1).unwrap();
    s.imagem_ajustar_recorte(0.05,0.0,0.1).unwrap();
    s.imagem_texto_alternativo("Painel elétrico com etiqueta").unwrap();
    s.imagem_sombra(true).unwrap();
    s.formatar(Formatacao::Contorno("#ff6600".into())).unwrap();
    let atual = s.slide().elementos.iter().find(|e| e.id==id).unwrap();
    assert!(atual.imagem.compartilha(&original), "edição não destrói nem duplica bytes");
    assert_eq!(atual.texto_alternativo, "Painel elétrico com etiqueta");
    assert!((atual.ajustes[4]-0.2).abs()<0.001);
    assert!((atual.ajustes[5]-0.1).abs()<0.001);
    assert!(!atual.sombra.cor.is_empty());
    let antes = s.apresentacao().clone();
    let novo = fsdz::ler(&fsdz::gravar(&antes).unwrap()).unwrap();
    assert_eq!(novo,antes);
    assert_eq!(novo.slides[0].elementos.iter().find(|e| e.id==id).unwrap()
        .visual_imagem.filtro,2);
    s.imagem_efeitos_resetar().unwrap();
    assert_eq!(s.slide().elementos.iter().find(|e| e.id==id).unwrap()
        .visual_imagem, featherslide_core::imagem::VisualImagem::default());
    s.desfazer().unwrap();
    assert_eq!(s.slide().elementos.iter().find(|e| e.id==id).unwrap()
        .visual_imagem.filtro,2);
}

#[test]
fn imagem_vai_como_arquivo_em_assets() {
    let s = apresentacao_variada();
    let bytes = fsdz::gravar(s.apresentacao()).unwrap();
    let zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let nomes: Vec<&str> = zip.file_names().collect();
    assert!(nomes.iter().any(|n| n.starts_with("assets/imagem-1.png")), "{nomes:?}");
}

#[test]
fn copiar_e_reutilizar_slide_preserva_notas_elementos_e_imagens_no_fsdz() {
    use featherslide_core::EstiloColagem;
    let origem = apresentacao_variada();
    let pacote = origem.slides_copiar(&[1]).unwrap();
    let original = pacote.origem[0].1.clone();
    let mut destino = Sessao::default();
    destino.slides_colar(1, &pacote, EstiloColagem::ManterOrigem).unwrap();
    let inserido = &destino.apresentacao().slides[1];
    assert_eq!(inserido.notas, original.notas);
    assert_eq!(inserido.layout, original.layout);
    assert_eq!(inserido.oculto, original.oculto);
    assert_eq!(inserido.elementos.len(), original.elementos.len());
    assert!(inserido.elementos.iter().any(|e| e.tipo == Tipo::Imagem && !e.imagem.is_empty()));
    let lido = fsdz::ler(&fsdz::gravar(destino.apresentacao()).unwrap()).unwrap();
    assert_eq!(lido, *destino.apresentacao());
    assert_eq!(lido.slides[1].notas, "lembrar de falar do orçamento");
}

#[test]
fn arquivo_ruim_da_erro_em_portugues() {
    assert!(fsdz::ler(b"isto nao e um zip").unwrap_err().contains(".fsdz"));
    assert!(fsdz::ler(&[]).is_err());
}

#[test]
fn ida_e_volta_com_fontes_embutidas() {
    let s = apresentacao_variada();
    let mut ap = s.apresentacao().clone();
    ap.fontes.push(featherslide_core::modelo::FonteEmbutida {
        nome: "Minha Fonte".into(),
        regular: Some(vec![0, 1, 0, 0, 10, 20, 30]),
        negrito: Some(vec![0, 1, 0, 0, 40, 50, 60]),
        italico: None,
        negrito_italico: None,
    });
    let bytes = fsdz::gravar(&ap).unwrap();
    let zip = zip::ZipArchive::new(std::io::Cursor::new(&bytes)).unwrap();
    let nomes: Vec<&str> = zip.file_names().collect();
    assert!(nomes.contains(&"data/fontes.arrow"), "{nomes:?}");
    assert!(nomes.contains(&"assets/fontes/MinhaFonte-regular.ttf"), "{nomes:?}");
    assert!(nomes.contains(&"assets/fontes/MinhaFonte-negrito.ttf"), "{nomes:?}");

    let lida = fsdz::ler(&bytes).unwrap();
    assert_eq!(lida, ap);
}


#[test]
fn ida_e_volta_com_tabela_grafico_tema_transicao_e_animacao() {
    use featherslide_core::modelo::{Animacao, Direcao, InicioAnimacao, TipoAnimacao, TipoTransicao, Transicao};
    let mut s = Sessao::default();
    s.tabela_inserir(2, 3).unwrap();
    let tabela = s.estado().selecao[0];
    s.tabela_celula_definir(tabela, 1, 1, "com\ttab e\nquebra").unwrap();
    s.formatar(Formatacao::Contorno("#ff6600".into())).unwrap();
    s.formatar(Formatacao::Espessura(3.5)).unwrap();
    s.grafico_inserir("pizza").unwrap();
    s.animacao_definir(Animacao { tipo: TipoAnimacao::Voar, inicio: InicioAnimacao::AposAnterior, direcao: Direcao::Baixo, duracao: 1.5, atraso: 0.25 }).unwrap();
    s.transicao_definir(Transicao { tipo: TipoTransicao::Cortina, direcao: Direcao::Cima, duracao: 1.2, apos: 4.0 }, true).unwrap();
    s.tema_aplicar("oceano").unwrap();
    let lida = fsdz::ler(&fsdz::gravar(s.apresentacao()).unwrap()).unwrap();
    assert_eq!(&lida, s.apresentacao());
    assert_eq!(lida.tema, "oceano");
}

#[test]
fn gatilho_de_slide_ida_e_volta() {
    let mut s = apresentacao_variada();
    s.slide_novo("titulo_conteudo").unwrap();
    s.slide_ir(0).unwrap();
    s.gatilho_inserir(1, Some([300.0, 200.0]), None).unwrap();
    let id = s.estado().selecao[0];
    assert_eq!(s.gatilho_do_elemento(0, id), Some(1));
    let lido = fsdz::ler(&fsdz::gravar(s.apresentacao()).unwrap()).unwrap();
    let e = lido.slides[0].elementos.iter().find(|e| e.id == id).unwrap();
    assert_eq!(e.destino, 2);
}

#[test]
fn oculto_e_grupo_ida_e_volta() {
    let mut s = apresentacao_variada();
    s.selecionar_tudo();
    s.agrupar().ok();
    let id = s.slide().elementos.iter().find(|e| e.papel != featherslide_core::modelo::Papel::Fundo).map(|e| e.id).unwrap();
    s.elemento_ocultar(id, true).unwrap();
    let lida = fsdz::ler(&fsdz::gravar(s.apresentacao()).unwrap()).unwrap();
    assert_eq!(&lida, s.apresentacao());
    let e = lida.slides[s.atual()].elementos.iter().find(|e| e.id == id).unwrap();
    assert!(e.oculto);
}

#[test]
fn link_e_comentarios_ida_e_volta() {
    let mut s = apresentacao_variada();
    s.slide_ir(0).unwrap();
    s.selecionar_tudo();
    assert!(s.link_definir("exemplo.com/a b").is_err());
    s.link_definir("exemplo.com").unwrap();
    let id = s.estado().selecao[0];
    assert_eq!(s.link_do_elemento(0, id).as_deref(), Some("https://exemplo.com"));
    s.comentario_adicionar("Ana", "rever esta frase\ncom quebra").unwrap();
    s.comentario_adicionar("", "segundo").unwrap();
    s.comentario_resolver(0, true).unwrap();
    assert!(s.comentario_excluir(5).is_err());
    let lida = fsdz::ler(&fsdz::gravar(s.apresentacao()).unwrap()).unwrap();
    assert_eq!(&lida, s.apresentacao());
    assert_eq!(lida.slides[0].comentarios.len(), 2);
    assert!(lida.slides[0].comentarios[0].resolvido);
    assert_eq!(lida.slides[0].comentarios[1].autor, "Autor");
    for _ in 0..3 {
        s.desfazer().unwrap();
    }
    assert!(s.slide().comentarios.is_empty());
}

#[test]
fn caminho_de_operacao_booleana_preserva_geometria_no_fsdz() {
    use featherslide_core::booleanas::OperacaoBooleana;
    let mut s = Sessao::default();
    s.slide_novo("em_branco").unwrap();
    s.elemento_inserir(Tipo::Retangulo, Some([0.0, 0.0, 100.0, 100.0]), None).unwrap();
    let primeiro = s.estado().selecao[0];
    s.elemento_inserir(Tipo::Retangulo, Some([25.0, 25.0, 50.0, 50.0]), None).unwrap();
    let segundo = s.estado().selecao[0];
    // A primeira selecionada é a base da subtração, como no editor.
    s.selecionar_por_id(primeiro);
    s.alternar_selecao(segundo);
    s.formas_booleanas(OperacaoBooleana::Subtracao).unwrap();
    let caminho = s.slide().elementos.iter().find(|e| e.tipo == Tipo::Caminho).unwrap();
    assert!(caminho.dados.contains('|'), "um anel interno representa o furo");
    let bytes = fsdz::gravar(s.apresentacao()).unwrap();
    let lido = fsdz::ler(&bytes).unwrap();
    assert_eq!(lido, *s.apresentacao(), "as formas compostas sobrevivem ao round-trip");
}


#[test]
fn propriedades_unicode_e_personalizadas_sobrevivem_ao_fsdz() {
    let mut sessao = Sessao::default();
    let mut dados = sessao.apresentacao().metadados.clone();
    dados.assunto = "Plano de ação – 2026".into();
    dados.empresa = "Engenharia".into();
    dados.gerente = "Gestão".into();
    dados.categoria = "Treinamento".into();
    dados.palavras_chave = "energia, gráficos, dados".into();
    dados.comentarios = "Revisão técnica".into();
    dados.idioma = "pt-BR".into();
    dados.propriedades_personalizadas.insert("Cliente".into(), "Vitória (ES)".into());
    dados.estendidos.insert("Chave\\control".into(), "Olá\u{001e}🌎;fim".into());
    sessao.propriedades_definir("Projeto α".into(), "Autor(a)".into(), dados).unwrap();
    let antes = sessao.apresentacao().clone();
    let reaberto = fsdz::ler(&fsdz::gravar(&antes).unwrap()).unwrap();
    assert_eq!(antes, reaberto);
    assert!(reaberto.metadados.revisoes >= 1);
    assert_eq!(reaberto.metadados.ultimo_autor, "Autor(a)");
}

#[test]
fn formas_basicas_persistem_no_fsdz_com_cores_e_detalhes() {
    let mut sessao = Sessao::default();
    sessao.slide_novo("em_branco").unwrap();
    for &(tipo, _) in featherslide_core::formas::ADICIONAIS {
        sessao.elemento_inserir(tipo, Some([20.0, 25.0, 120.0, 120.0]), None).unwrap();
        sessao.formatar(Formatacao::Preenchimento("#fa6240".into())).unwrap();
    }
    let original = sessao.apresentacao().clone();
    let lido = fsdz::ler(&fsdz::gravar(&original).unwrap()).unwrap();
    assert_eq!(lido, original);
    let quadro = Sessao::nova(lido).quadro(1, None, false).unwrap().unwrap();
    for &(tipo, _) in featherslide_core::formas::ADICIONAIS {
        if featherslide_core::novas_formas::suporta(tipo) {
            let el = quadro.elementos.iter().find(|e| e.tipo == tipo.nome()).unwrap();
            assert!(!el.primitivas.is_empty(), "{tipo:?} sem caminho vetorial");
        }
    }
}

#[test]
fn formas_adicionais_abrem_com_o_mesmo_tipo_e_estilo() {
    let mut s = Sessao::default();
    s.slide_novo("em_branco").unwrap();
    for &(tipo, _) in featherslide_core::formas::ADICIONAIS {
        s.elemento_inserir(tipo, Some([10.0, 20.0, 200.0, 100.0]), None).unwrap();
        let id = s.estado().selecao[0];
        if tipo.aceita_texto() {
            s.texto_definir(id, "Forma nova").unwrap();
        }
    }
    let bytes = fsdz::gravar(s.apresentacao()).unwrap();
    assert_eq!(fsdz::ler(&bytes).unwrap(), *s.apresentacao());
    assert_eq!(Tipo::TODOS[11], Tipo::Icone, "os códigos anteriores continuam iguais");
}


#[test]
fn apresentacoes_personalizadas_persistem_com_ordem_e_ocultacao() {
    let mut s = Sessao::default();
    s.slide_novo("em_branco").unwrap();
    s.slide_novo("em_branco").unwrap();
    s.personalizada_criar("Diretoria", &[2, 0, 1]).unwrap();
    s.personalizada_ocultar(0, 1).unwrap();
    let ap = s.apresentacao().clone();
    let bytes = fsdz::gravar(&ap).unwrap();
    let nova = fsdz::ler(&bytes).unwrap();
    assert_eq!(nova, ap);
    let sessao = Sessao::nova(nova);
    assert_eq!(sessao.personalizada_ordem(0).unwrap(), vec![2, 1]);
}

#[test]
fn diagramas_nativos_permanecem_editaveis_apos_fsdz() {
    use featherslide_core::diagramas::Categoria;
    let mut sessao = Sessao::default();
    for categoria in Categoria::TODAS {
        sessao.diagrama_inserir(categoria).unwrap();
    }
    let antes = sessao.apresentacao().clone();
    let bytes = fsdz::gravar(&antes).unwrap();
    let depois = fsdz::ler(&bytes).unwrap();
    assert_eq!(depois, antes, "IDs, estilos, textos e grupos devem persistir");
    let agrupados: std::collections::HashSet<_> = depois.slides[0].elementos
        .iter().filter(|e| e.grupo > 0).map(|e| e.grupo).collect();
    assert_eq!(agrupados.len(), 14);
}

#[test]
fn animacoes_de_entrada_v10_salvam_e_reabrem_sem_reordenar_codigos() {
    use featherslide_core::modelo::{Animacao, TipoAnimacao};
    let mut s = Sessao::default();
    assert_eq!(TipoAnimacao::TODOS.len(), 19);
    assert_eq!(&TipoAnimacao::TODOS[..7], &[
        TipoAnimacao::Nenhuma, TipoAnimacao::Aparecer, TipoAnimacao::Esmaecer,
        TipoAnimacao::Voar, TipoAnimacao::Zoom, TipoAnimacao::Revelar,
        TipoAnimacao::Pulsar
    ]);
    let novos = &TipoAnimacao::TODOS[7..];
    assert_eq!(novos.len(), 12);
    for &tipo in novos {
        s.elemento_inserir(Tipo::Retangulo, None, None).unwrap();
        s.animacao_definir(Animacao { tipo, duracao: 1.35, atraso: 0.2, ..Animacao::default() }).unwrap();
    }
    let arquivo = fsdz::gravar(s.apresentacao()).unwrap();
    let lida = fsdz::ler(&arquivo).unwrap();
    assert_eq!(lida, *s.apresentacao());
    let gravados: Vec<_> = lida.slides[0].elementos.iter()
        .filter(|e| novos.contains(&e.animacao.tipo))
        .map(|e| e.animacao.tipo).collect();
    assert_eq!(gravados, novos);
    s.desfazer().unwrap();
    assert_eq!(s.slide().elementos.last().unwrap().animacao.tipo, TipoAnimacao::Nenhuma);
}

#[test]
fn diagrama_personalizado_preserva_pais_ordem_e_conectores() {
    use featherslide_core::diagramas::Categoria;
    let mut s=Sessao::default();s.diagrama_inserir(Categoria::Hierarquias).unwrap();
    let mut editor=s.diagrama_textos_selecionados().unwrap();
    let pai=editor.nos.iter().find(|n|n.texto=="Ramo B").unwrap().id;
    editor.adicionar_a("Novo departamento".into(),Some(pai)).unwrap();
    s.diagrama_textos_definir(&editor).unwrap();
    let bytes=fsdz::gravar(s.apresentacao()).unwrap();
    let mut reaberta=Sessao::nova(fsdz::ler(&bytes).unwrap());
    reaberta.selecionar_por_id(pai);
    let editor=reaberta.diagrama_textos_selecionados().unwrap();
    let no=editor.nos.iter().find(|n|n.texto=="Novo departamento").unwrap();
    assert_eq!(no.pai,Some(pai));assert_eq!(no.linha,2);
    let antigo_grupo=reaberta.slide().elementos.iter().find(|n|n.id==pai).unwrap().grupo;
    reaberta.selecao_duplicar().unwrap();
    let copiado=reaberta.diagrama_textos_selecionados().unwrap();
    let novo_pai=copiado.nos.iter().find(|n|n.texto=="Ramo B").unwrap().id;
    assert_ne!(novo_pai,pai);
    assert_eq!(copiado.nos.iter().find(|n|n.texto=="Novo departamento").unwrap().pai,Some(novo_pai));
    assert_ne!(reaberta.slide().elementos.iter().find(|n|n.id==novo_pai).unwrap().grupo,antigo_grupo);
}

#[test]
fn configuracao_slide_dimensoes_fundos_e_mestre_persistem() {
    use featherslide_core::configuracao_slide::{Ajuste, Fundo};
    let mut s = Sessao::default();
    s.tamanho_slide_definir(420.0, 700.0, Ajuste::Ajustar).unwrap();
    for tipo in ["solido", "gradiente", "textura", "padrao", "imagem"] {
        let f = Fundo { tipo: tipo.into(), opacidade: 0.35,
            imagem: featherslide_io::base64::para_data_url("image/png", &[1, 2, 3]).into(), ..Fundo::default() };
        s.fundo_configurar("#127abc", f, true, true).unwrap();
        let bytes = fsdz::gravar(s.apresentacao()).unwrap();
        let ap = fsdz::ler(&bytes).unwrap();
        assert_eq!(&ap, s.apresentacao());
        assert_eq!(ap.tamanho(), (420.0, 700.0));
        if tipo == "imagem" {
            let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
            assert!(zip.by_name("assets/imagem-1.png").is_ok());
        }
    }
}

#[test]
fn dimensoes_invalidas_no_arquivo_retornam_erro() {
    let mut s = Sessao::default();
    s.apresentacao_mut().dimensoes = [f32::NAN, 700.0];
    let bytes = fsdz::gravar(s.apresentacao()).unwrap();
    assert!(fsdz::ler(&bytes).unwrap_err().contains("dimensão"));
}

#[test]
fn formato_anterior_sem_colunas_de_configuracao_abre_com_padrao() {
    use std::io::{Cursor, Read, Write};
    use std::sync::Arc;
    use arrow::ipc::{reader::FileReader, writer::FileWriter};
    use arrow::datatypes::Schema;
    use arrow::record_batch::RecordBatch;
    let s = Sessao::default();
    let mut entrada = zip::ZipArchive::new(Cursor::new(fsdz::gravar(s.apresentacao()).unwrap())).unwrap();
    let mut saida = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..entrada.len() {
        let mut arquivo = entrada.by_index(i).unwrap();
        let nome = arquivo.name().to_string();
        let mut bytes = Vec::new();
        arquivo.read_to_end(&mut bytes).unwrap();
        if nome == "data/apresentacao.arrow" || nome == "data/slides.arrow" {
            let mut leitor = FileReader::try_new(Cursor::new(bytes), None).unwrap();
            let lote = leitor.next().unwrap().unwrap();
            let excluir = ["largura_personalizada", "altura_personalizada", "fundo_tipo", "fundo_cor2", "fundo_imagem", "fundo_opacidade", "ocultar_mestre"];
            let indices: Vec<_> = lote.schema().fields().iter().enumerate().filter(|(_, f)| !excluir.contains(&f.name().as_str())).map(|(i, _)| i).collect();
            let lote = lote.project(&indices).unwrap();
            let mut meta = lote.schema().metadata().clone();
            if nome == "data/apresentacao.arrow" { meta.insert("formato".into(), "8".into()); }
            let esquema = Arc::new(Schema::new_with_metadata(lote.schema().fields().clone(), meta));
            let lote = RecordBatch::try_new(esquema.clone(), lote.columns().to_vec()).unwrap();
            bytes = Vec::new();
            let mut escritor = FileWriter::try_new(&mut bytes, &esquema).unwrap();
            escritor.write(&lote).unwrap();
            escritor.finish().unwrap();
        }
        saida.start_file(nome, zip::write::SimpleFileOptions::default()).unwrap();
        saida.write_all(&bytes).unwrap();
    }
    let antiga = saida.finish().unwrap().into_inner();
    assert_eq!(fsdz::ler(&antiga).unwrap(), *s.apresentacao());
}

#[test]
fn forma_gradiente_e_texto_tipografico_persistem() {
    use featherslide_core::estilo_forma::{Aparencia, Gradiente, TipoGradiente};
    let mut s = Sessao::default();
    s.elemento_inserir(Tipo::RetanguloArredondado, None, None)
        .unwrap();
    let id = s.slide().elementos.last().unwrap().id;
    s.texto_definir(id, "x2 e H2O").unwrap();
    s.texto_indice(id, 1, 2, 1).unwrap();
    s.texto_indice(id, 6, 7, -1).unwrap();
    s.texto_espacamento(2.0).unwrap();
    let mut g = Gradiente::default();
    g.tipo = TipoGradiente::Radial;
    g.angulo = 45.0;
    s.forma_aparencia(Aparencia {
        preenchimento: "#4285f4".into(),
        gradiente: Some(g),
        contorno: "#123456".into(),
        espessura: 2.0,
        opacidade: 0.6,
        sombra: Sombra {
            cor: "#00000080".into(),
            desfoque: 4.0,
            dx: 3.0,
            dy: 5.0,
        },
    })
    .unwrap();
    let ap = fsdz::ler(&fsdz::gravar(s.apresentacao()).unwrap()).unwrap();
    assert_eq!(&ap, s.apresentacao());
}

#[test]
fn traco_duplo_persiste() {
    use featherslide_core::Formatacao;
    let mut s = Sessao::default();
    s.elemento_inserir(Tipo::Texto, None, None).unwrap();
    let id = s.slide().elementos.last().unwrap().id;
    s.texto_definir(id, "Duplo").unwrap();
    s.selecionar_por_id(id);
    s.formatar(Formatacao::SublinhadoDuplo).unwrap();
    s.formatar(Formatacao::TachadoDuplo).unwrap();
    let ap = fsdz::ler(&fsdz::gravar(s.apresentacao()).unwrap()).unwrap();
    assert_eq!(&ap, s.apresentacao());
    let est = &ap.slides[s.atual()].elementos.last().unwrap().estilo;
    assert!(est.sublinhado && est.sublinhado_duplo && est.tachado && est.tachado_duplo);
}
