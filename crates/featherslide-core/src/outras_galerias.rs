//! Galerias vetoriais adicionais, conectores vinculados e desenho livre.
use crate::modelo::{Elemento, Slide, Tipo};
use crate::sessao::{Mudanca, Sessao};

/// Categoria, identificador estável e nome de cada opção da galeria; a geometria
/// normalizada (prévia, tela, PDF e MCP) vem do `feather-shapes`.
pub use feather_shapes::galeria::{geometria, FORMAS};

impl Sessao {
    /// Insere uma forma vetorial da galeria sem depender da interface.
    pub fn galeria_inserir(&mut self, id: &str) -> Result<Mudanca, String> {
        let dados = geometria(id)?;
        self.editar(|s| {
            let (w, h) = s.ap.tamanho();
            let destino = match id {
                "acao_inicio" => 1,
                "acao_anterior" => s.atual.max(1) as u32,
                "acao_proximo" => (s.atual + 2).min(s.ap.slides.len()) as u32,
                _ => 0,
            };
            let e = Elemento {
                id: s.ap.novo_id(),
                tipo: Tipo::Caminho,
                x: (w - 200.0) / 2.0,
                y: (h - 120.0) / 2.0,
                largura: 200.0,
                altura: 120.0,
                dados,
                destino,
                preenchimento: "#4285f4".into(),
                contorno: "#202124".into(),
                espessura: 2.0,
                ..Elemento::default()
            };
            s.selecao = vec![e.id];
            s.slide_mut().elementos.push(e);
            Ok((vec![s.atual], false))
        })
    }

    /// Converte o traço capturado em caminho vetorial aberto, com uma única edição.
    pub fn desenho_livre_inserir(&mut self, pontos: &[[f32; 2]]) -> Result<Mudanca, String> {
        let ([x, y, w, h], dados) = feather_shapes::galeria::traco_livre(pontos)?;
        self.editar(|s| {
            let e = Elemento {
                id: s.ap.novo_id(),
                tipo: Tipo::Caminho,
                x,
                y,
                largura: w,
                altura: h,
                dados,
                preenchimento: String::new(),
                contorno: "#202124".into(),
                espessura: 2.0,
                ..Elemento::default()
            };
            s.selecao = vec![e.id];
            s.slide_mut().elementos.push(e);
            Ok((vec![s.atual], false))
        })
    }

    /// Conecta duas formas selecionadas. A ordem da seleção define origem e destino.
    pub fn conector_inserir(&mut self, estilo: &str) -> Result<Mudanca, String> {
        if !["reto", "cotovelo", "curvo"].contains(&estilo) || self.selecao.len() != 2 {
            return Err("Selecione exatamente duas formas para conectar.".into());
        }
        let ids = self.selecao.clone();
        if self
            .slide()
            .elementos
            .iter()
            .filter(|e| ids.contains(&e.id))
            .any(|e| vinculo(e).is_some())
        {
            return Err("Selecione formas, não conectores.".into());
        }
        self.editar(|s| {
            let e = Elemento {
                id: s.ap.novo_id(),
                tipo: Tipo::Caminho,
                largura: 1.0,
                altura: 1.0,
                dados: format!(
                    "SVG1\nnormal\nM 0 0\nL 1 1\n#CON {} {} {estilo}\n",
                    ids[0], ids[1]
                ),
                preenchimento: String::new(),
                contorno: "#202124".into(),
                espessura: 2.0,
                ..Elemento::default()
            };
            s.selecao = vec![e.id];
            s.slide_mut().elementos.push(e);
            Ok((vec![s.atual], false))
        })
    }
}

fn vinculo(e: &Elemento) -> Option<(u64, u64, &str)> {
    let l = e.dados.lines().find(|l| l.starts_with("#CON "))?;
    let mut c = l.split_whitespace();
    c.next()?;
    Some((c.next()?.parse().ok()?, c.next()?.parse().ok()?, c.next()?))
}

/// Remapeia as âncoras ao copiar elementos. Sem os alvos, a cópia vira traço independente.
pub fn remapear_conectores(elementos: &mut [Elemento], ids: &[(u64, u64)]) {
    for e in elementos {
        if let Some(pai) = e.dados.lines().find_map(|l| l.strip_prefix("#DIA_PARENT ")?.parse::<u64>().ok()) {
            let novo = ids.iter().find(|(de,_)|*de==pai).map(|(_,para)|*para).unwrap_or(0);
            e.dados=e.dados.replace(&format!("#DIA_PARENT {pai}\n"),&format!("#DIA_PARENT {novo}\n"));
        }
        if let Some((a, b, estilo)) = vinculo(e) {
            let antigo = format!("#CON {a} {b} {estilo}\n");
            let mapa = |id| ids.iter().find(|(de, _)| *de == id).map(|(_, para)| *para);
            let novo = match (mapa(a), mapa(b)) {
                (Some(a), Some(b)) => format!("#CON {a} {b} {estilo}\n"),
                _ => String::new(),
            };
            e.dados = e.dados.replace(&antigo, &novo);
        }
    }
}

/// Atualiza ancoragem nas bordas das caixas após cada transação do documento.
pub fn atualizar_conectores(slide: &mut Slide) {
    let caixas: Vec<_> = slide
        .elementos
        .iter()
        .map(|e| (e.id, [e.x, e.y, e.largura, e.altura]))
        .collect();
    for e in &mut slide.elementos {
        let Some((a, b, estilo)) = vinculo(e) else {
            continue;
        };
        let estilo = estilo.to_owned();
        let (Some((_, a)), Some((_, b))) = (
            caixas.iter().find(|(id, _)| *id == a),
            caixas.iter().find(|(id, _)| *id == b),
        ) else {
            continue;
        };
        let (caixa, caminho) = feather_shapes::galeria::rota_conector(&estilo, a, b);
        [e.x, e.y, e.largura, e.altura] = caixa;
        let (a, b, _) = vinculo(e).unwrap();
        e.dados = format!("{caminho}#CON {a} {b} {estilo}\n");
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn copia_remapeia_ancoras_e_acoes_tem_destino() {
        let mut s = Sessao::default();
        s.slide_novo("em_branco").unwrap();
        s.elemento_inserir(Tipo::Retangulo, Some([10.0, 10.0, 100.0, 100.0]), None)
            .unwrap();
        s.elemento_inserir(Tipo::Retangulo, Some([300.0, 200.0, 100.0, 100.0]), None)
            .unwrap();
        s.selecionar_tudo();
        s.conector_inserir("cotovelo").unwrap();
        s.slide_duplicar().unwrap();
        let elementos = &s.slide().elementos;
        let (a, b, _) = vinculo(elementos.last().unwrap()).unwrap();
        assert_eq!(a, elementos[0].id);
        assert_eq!(b, elementos[1].id);
        s.selecionar_tudo();
        s.selecao_duplicar().unwrap();
        let elementos = &s.slide().elementos;
        let (a, b, _) = vinculo(elementos.last().unwrap()).unwrap();
        assert_eq!(a, elementos[3].id);
        assert_eq!(b, elementos[4].id);
        let id = elementos.last().unwrap().id;
        s.selecionar_por_id(id);
        s.selecao_duplicar().unwrap();
        let isolado = s.slide().elementos.last().unwrap();
        assert!(vinculo(isolado).is_none());
        assert!(crate::vetores::decodificar(isolado).is_ok());
        s.galeria_inserir("acao_inicio").unwrap();
        assert_eq!(s.slide().elementos.last().unwrap().destino, 1);
        s.galeria_inserir("acao_anterior").unwrap();
        assert_eq!(s.slide().elementos.last().unwrap().destino, 2);
    }
    #[test]
    fn todas_as_galerias_renderizam_selecionam_e_desfazem() {
        for &(_, id, _) in FORMAS {
            let mut s = Sessao::default();
            s.galeria_inserir(id).unwrap();
            let e = s.slide().elementos.last().unwrap().clone();
            assert!(!crate::vetores::primitivas(&e).is_empty(), "{id}");
            assert_eq!(
                s.selecionar_no_ponto(e.x + e.largura / 2.0, e.y + e.altura / 2.0, false),
                Some(e.id)
            );
            s.desfazer().unwrap();
            assert!(!s.slide().elementos.iter().any(|p| p.id == e.id));
        }
    }
    #[test]
    fn conectores_seguem_as_formas_e_desfazem_juntos() {
        for estilo in ["reto", "cotovelo", "curvo"] {
            let mut s = Sessao::default();
            s.elemento_inserir(Tipo::Retangulo, Some([0.0, 0.0, 100.0, 100.0]), None)
                .unwrap();
            let a = s.selecao[0];
            s.elemento_inserir(Tipo::Retangulo, Some([300.0, 200.0, 100.0, 100.0]), None)
                .unwrap();
            let b = s.selecao[0];
            s.selecao = vec![a, b];
            s.conector_inserir(estilo).unwrap();
            let antes = s.slide().elementos.last().unwrap().clone();
            assert!(crate::vetores::decodificar(&antes).is_ok());
            s.selecao = vec![b];
            s.selecao_mover(100.0, 20.0).unwrap();
            let depois = s.slide().elementos.last().unwrap();
            assert_ne!(
                (depois.x, depois.y, depois.largura, depois.altura),
                (antes.x, antes.y, antes.largura, antes.altura)
            );
            s.desfazer().unwrap();
            assert_eq!(s.slide().elementos.last().unwrap(), &antes);
        }
    }
    #[test]
    fn traco_livre_aberto_preserva_geometria_e_rejeita_dados_invalidos() {
        let mut s = Sessao::default();
        let pontos = [[10.0, 20.0], [40.0, 60.0], [90.0, 30.0]];
        s.desenho_livre_inserir(&pontos).unwrap();
        let e = s.slide().elementos.last().unwrap();
        let (_, segmentos) = crate::vetores::decodificar(e).unwrap();
        assert_eq!(segmentos.len(), 3);
        assert!(e.preenchimento.is_empty());
        assert_eq!(segmentos[2], crate::vetores::Segmento::L(pontos[2]));
        assert!(s
            .desenho_livre_inserir(&[[f32::NAN, 0.0], [1.0, 2.0]])
            .is_err());
    }
}
