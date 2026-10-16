//! Edição conjunta dos rótulos de um grupo, preservando formas, ligações e estilos.
use crate::{
    modelo::{Elemento, Papel, Tipo},
    sessao::{Mudanca, Sessao},
};

/// Um item do editor: id, texto, nível (`linha`) e pai (do `feather-shapes`).
pub use feather_shapes::arranjo::NoDiagrama as RotuloDiagrama;
/// Lista, Processo, Ciclo ou Organograma (do `feather-shapes`).
pub use feather_shapes::arranjo::Arranjo;

use feather_shapes::arranjo as arvore;

#[derive(Clone, Debug)]
pub struct EdicaoDiagrama {
    pub nos: Vec<RotuloDiagrama>,
    pub arranjo: Arranjo,
    reorganizar: bool,
    /// Os itens como foram lidos do slide (o texto é o original).
    originais: Vec<RotuloDiagrama>,
    slide: u64,
    grupo: u32,
    membros: Vec<u64>,
    conectar: bool,
}

impl EdicaoDiagrama {
    pub fn adicionar(&mut self, texto: String) -> Result<(), String> {
        self.adicionar_a(texto, None)
    }
    pub fn adicionar_a(&mut self, texto: String, pai: Option<u64>) -> Result<(), String> {
        arvore::adicionar(&mut self.nos, texto, pai)?;
        self.reorganizar = true;
        Ok(())
    }
    pub fn excluir(&mut self, i: usize) -> Result<(), String> {
        arvore::excluir(&mut self.nos, i)?;
        self.reorganizar = true;
        Ok(())
    }
    pub fn mover(&mut self, i: usize, delta: isize) {
        if arvore::mover(&mut self.nos, i, delta) {
            self.reorganizar = true;
        }
    }
    pub fn arranjo_definir(&mut self, arranjo: Arranjo) {
        arvore::trocar_arranjo(&mut self.nos, self.arranjo, arranjo);
        self.arranjo = arranjo;
        self.reorganizar = true;
    }
    pub fn nivel_alterar(&mut self, i: usize, aprofundar: bool) {
        if arvore::alterar_nivel(&mut self.nos, self.arranjo, i, aprofundar) {
            self.reorganizar = true;
        }
    }
    /// O texto original do item (vazio para itens novos).
    fn original(&self, id: u64) -> &str {
        self.originais.iter().find(|n| n.id == id).map_or("", |n| n.texto.as_str())
    }
}

impl Sessao {
    pub fn diagrama_textos_selecionados(&self) -> Result<EdicaoDiagrama, String> {
        let grupo = self
            .slide()
            .elementos
            .iter()
            .find(|e| self.selecao.contains(&e.id))
            .map(|e| e.grupo)
            .filter(|&g| g > 0)
            .ok_or("Selecione um diagrama agrupado para editar seus textos.")?;
        if self
            .slide()
            .elementos
            .iter()
            .any(|e| self.selecao.contains(&e.id) && e.grupo != grupo)
        {
            return Err("Selecione apenas um diagrama por vez.".into());
        }
        let membros = self
            .slide()
            .elementos
            .iter()
            .filter(|e| e.grupo == grupo)
            .map(|e| e.id)
            .collect();
        let mut formas: Vec<_> = self
            .slide()
            .elementos
            .iter()
            .filter(|e| e.grupo == grupo && e.tipo.aceita_texto() && e.papel != Papel::Fundo)
            .collect();
        formas.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
        if formas.is_empty() {
            return Err("Esse grupo não contém textos editáveis.".into());
        }
        let arranjo = if let Some(layout) = formas
            .iter()
            .find_map(|e| e.dados.lines().find_map(|l| l.strip_prefix("#DIA_LAYOUT ")))
        {
            Arranjo::ler(layout)
        } else if formas.len() >= 3 && formas.iter().all(|e| e.tipo == Tipo::Elipse) {
            Arranjo::Ciclo
        } else if formas.iter().skip(1).all(|e| e.y > formas[0].y + 16.0) {
            Arranjo::Organograma
        } else if formas.iter().all(|e| (e.y - formas[0].y).abs() < 16.0) {
            Arranjo::Processo
        } else {
            Arranjo::Lista
        };
        if formas.iter().all(|e| e.dados.contains("#DIA_ORDER ")) {
            formas.sort_by_key(|e| {
                e.dados
                    .lines()
                    .find_map(|l| l.strip_prefix("#DIA_ORDER ")?.parse::<usize>().ok())
                    .unwrap_or(0)
            });
        }
        let nos: Vec<_> = formas
            .iter()
            .map(|e| {
                let pai =
                    if let Some(p) = e.dados.lines().find_map(|l| l.strip_prefix("#DIA_PARENT ")) {
                        p.parse::<u64>().ok().filter(|&id| id != 0)
                    } else if arranjo == Arranjo::Organograma {
                        let precedente = formas
                            .iter()
                            .filter(|p| p.y < e.y - 16.)
                            .map(|p| p.y)
                            .max_by(f32::total_cmp);
                        precedente.and_then(|y| pai_mais_proximo(&formas, y, e.x + e.largura / 2.))
                    } else {
                        None
                    };
                RotuloDiagrama {
                    id: e.id,
                    texto: e.texto.clone(),
                    linha: 0,
                    pai,
                }
            })
            .collect();
        let mut edicao = EdicaoDiagrama {
            conectar: formas
                .iter()
                .find_map(|e| e.dados.lines().find_map(|l| l.strip_prefix("#DIA_LINKS ")))
                .map(|v| v == "1")
                .unwrap_or_else(|| {
                    self.slide().elementos.iter().any(|e| {
                        e.grupo == grupo
                            && (e.tipo == Tipo::Linha
                                || e.dados.lines().any(|l| l.starts_with("#CON ")))
                    })
                }),
            originais: nos.clone(),
            arranjo,
            reorganizar: false,
            nos,
            slide: self.slide().id,
            grupo,
            membros,
        };
        arvore::ordenar(&mut edicao.nos);
        Ok(edicao)
    }

    pub fn diagrama_textos_definir(&mut self, edicao: &EdicaoDiagrama) -> Result<Mudanca, String> {
        let membros: Vec<_> = self
            .slide()
            .elementos
            .iter()
            .filter(|e| e.grupo == edicao.grupo)
            .map(|e| e.id)
            .collect();
        if self.slide().id != edicao.slide || membros != edicao.membros || edicao.nos.is_empty() {
            return Err("O diagrama mudou. Reabra o editor para carregar os textos atuais.".into());
        }
        if edicao.nos.len() > 64 {
            return Err("O diagrama aceita até 64 itens.".into());
        }
        let mut vistos = std::collections::HashSet::new();
        for no in &edicao.nos {
            if !vistos.insert(no.id) {
                return Err("Identificador de item repetido.".into());
            }
            let mut pai = no.pai;
            let mut profundidade = 0;
            while let Some(id) = pai {
                profundidade += 1;
                if id == no.id || profundidade > edicao.nos.len() {
                    return Err("A hierarquia contém um ciclo.".into());
                }
                pai = edicao
                    .nos
                    .iter()
                    .find(|n| n.id == id)
                    .ok_or("O pai de um item não existe.")?
                    .pai;
            }
        }
        for no in &edicao.originais {
            if !self.slide().elementos.iter().any(|e| {
                e.id == no.id
                    && e.grupo == edicao.grupo
                    && e.tipo.aceita_texto()
                    && e.texto == no.texto
            }) {
                return Err("O texto do diagrama mudou. Reabra o editor antes de aplicar.".into());
            }
        }
        if !edicao.reorganizar && edicao.nos.iter().all(|n| n.texto == edicao.original(n.id)) {
            return Ok(Mudanca {
                versao: self.versao,
                slides: vec![],
                lista: false,
            });
        }
        self.editar(|s| {
            if edicao.reorganizar {
                let modelos: Vec<_> = s
                    .slide()
                    .elementos
                    .iter()
                    .filter(|e| e.grupo == edicao.grupo && e.tipo.aceita_texto())
                    .cloned()
                    .collect();
                let (sw, sh) = s.ap.tamanho();
                let rect = [sw * 0.12, sh * 0.14, sw * 0.76, sh * 0.72];
                let mut novos = Vec::new();
                for (i, no) in edicao.nos.iter().enumerate() {
                    let mut e = modelos
                        .iter()
                        .find(|e| e.id == no.id)
                        .unwrap_or(&modelos[i % modelos.len()])
                        .clone();
                    if !edicao.originais.iter().any(|n| n.id == no.id) {
                        e.id = s.ap.novo_id();
                    } else {
                        e.id = no.id;
                    }
                    crate::texto_formato::atualizar_indices(&mut e,&no.texto);
                    e.texto = no.texto.clone();
                    let [x, y, w, h] = arvore::retangulo(edicao.arranjo, &edicao.nos, i, rect);
                    e.x = x;
                    e.y = y;
                    e.largura = w;
                    e.altura = h;
                    e.estilo.tamanho = e.estilo.tamanho.min((h * 0.3).max(6.0));
                    novos.push(e);
                }
                let ids: Vec<_> = novos.iter().map(|e| e.id).collect();
                let ligacoes = if !edicao.conectar { vec![] } else { arvore::ligacoes(edicao.arranjo, &edicao.nos) };
                let mut conectores = Vec::new();
                for (i, no) in edicao.nos.iter().enumerate() {
                    let pai = no
                        .pai
                        .and_then(|p| edicao.nos.iter().position(|n| n.id == p))
                        .map(|p| ids[p])
                        .unwrap_or(0);
                    let layout = edicao.arranjo.identificador();
                    let anterior = novos[i]
                        .dados
                        .lines()
                        .filter(|l| !l.starts_with("#DIA_"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    novos[i].dados = format!(
                        "{anterior}\n#DIA_PARENT {pai}\n#DIA_LAYOUT {layout}\n#DIA_ORDER {i}\n#DIA_LINKS {}\n", u8::from(edicao.conectar)
                    );
                }
                for (a, b) in ligacoes {
                    conectores.push(Elemento {
                        id: s.ap.novo_id(),
                        tipo: Tipo::Caminho,
                        grupo: edicao.grupo,
                        largura: 1.,
                        altura: 1.,
                        contorno: "#8296AC".into(),
                        espessura: 2.,
                        dados: format!(
                            "SVG1\nnormal\nM 0 0\nL 1 1\n#CON {} {} {}\n",
                            ids[a],
                            ids[b],
                            edicao.arranjo.estilo_conector()
                        ),
                        ..Elemento::default()
                    });
                }
                s.slide_mut().elementos.retain(|e| e.grupo != edicao.grupo);
                s.slide_mut().elementos.extend(conectores);
                s.slide_mut().elementos.extend(novos);
                s.selecao = vec![ids[0]];
                return Ok((vec![s.atual], false));
            }
            for no in &edicao.nos {
                if let Some(e) = s.slide_mut().elementos.iter_mut().find(|e| e.id == no.id) {
                    crate::texto_formato::atualizar_indices(e,&no.texto);
                    e.texto = no.texto.clone();
                }
            }
            Ok((vec![s.atual], false))
        })
    }
}

fn pai_mais_proximo(formas: &[&Elemento], y: f32, centro: f32) -> Option<u64> {
    formas
        .iter()
        .filter(|p| (p.y - y).abs() < 1.)
        .min_by(|a, b| {
            ((a.x + a.largura / 2.) - centro)
                .abs()
                .total_cmp(&((b.x + b.largura / 2.) - centro).abs())
        })
        .map(|p| p.id)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagramas::Categoria;
    #[test]
    fn processo_sem_linhas_continua_sem_linhas_apos_editar_e_reabrir() {
        let mut s = Sessao::default();
        s.diagrama_inserir(Categoria::Processos).unwrap();
        let antes = s.apresentacao().slides.clone();
        for texto in ["Novo passo", "Outro passo"] {
            let mut e = s.diagrama_textos_selecionados().unwrap();
            e.adicionar(texto.into()).unwrap();
            s.diagrama_textos_definir(&e).unwrap();
            assert!(s
                .slide()
                .elementos
                .iter()
                .all(|n| n.tipo != Tipo::Linha && !n.dados.contains("#CON ")));
        }
        assert_eq!(s.diagrama_textos_selecionados().unwrap().nos.len(), 6);
        s.desfazer().unwrap();
        s.desfazer().unwrap();
        assert_eq!(s.apresentacao().slides, antes);
    }
    #[test]
    fn hierarquia_agrupa_filhos_de_cada_ramo_e_novos_itens() {
        let mut s = Sessao::default();
        s.diagrama_inserir(Categoria::Hierarquias).unwrap();
        let mut e = s.diagrama_textos_selecionados().unwrap();
        assert_eq!(
            e.nos
                .iter()
                .map(|n| (n.texto.as_str(), n.linha))
                .collect::<Vec<_>>(),
            [
                ("Nível 1", 0),
                ("Ramo A", 1),
                ("Item 1", 2),
                ("Item 2", 2),
                ("Ramo B", 1),
                ("Ramo C", 1),
                ("Item 3", 2),
                ("Item 4", 2)
            ]
        );
        let ramo = e.nos.iter().find(|n| n.texto == "Ramo B").unwrap().id;
        e.adicionar_a("Equipe nova".into(), Some(ramo)).unwrap();
        s.diagrama_textos_definir(&e).unwrap();
        let reaberto = s.diagrama_textos_selecionados().unwrap();
        let novo = reaberto
            .nos
            .iter()
            .find(|n| n.texto == "Equipe nova")
            .unwrap();
        assert_eq!(novo.pai, Some(ramo));
        assert_eq!(novo.linha, 2);
        s.slide_duplicar().unwrap();
        let raiz = s
            .slide()
            .elementos
            .iter()
            .find(|e| e.texto == "Nível 1")
            .unwrap()
            .id;
        s.selecionar_por_id(raiz);
        let copia = s.diagrama_textos_selecionados().unwrap();
        let ramo_copiado = copia.nos.iter().find(|n| n.texto == "Ramo B").unwrap().id;
        assert_ne!(ramo, ramo_copiado);
        assert_eq!(
            copia
                .nos
                .iter()
                .find(|n| n.texto == "Equipe nova")
                .unwrap()
                .pai,
            Some(ramo_copiado)
        );
    }
    #[test]
    fn ciclo_com_item_extra_reconecta_e_desfaz() {
        let mut s = Sessao::default();
        s.diagrama_inserir(Categoria::Ciclos).unwrap();
        let antes = s.apresentacao().slides.clone();
        let mut e = s.diagrama_textos_selecionados().unwrap();
        assert_eq!(e.arranjo, Arranjo::Ciclo);
        e.adicionar("Avaliar".into()).unwrap();
        s.diagrama_textos_definir(&e).unwrap();
        assert_eq!(s.diagrama_textos_selecionados().unwrap().nos.len(), 5);
        assert_eq!(
            s.slide()
                .elementos
                .iter()
                .filter(|e| e.dados.contains("#CON "))
                .count(),
            5
        );
        let (w, h) = s.apresentacao().tamanho();
        for n in s
            .slide()
            .elementos
            .iter()
            .filter(|e| e.grupo > 0 && e.tipo.aceita_texto())
        {
            assert!(
                n.x >= 0. && n.y >= 0. && n.x + n.largura <= w + 0.01 && n.y + n.altura <= h + 0.01
            );
        }
        s.desfazer().unwrap();
        assert_eq!(s.apresentacao().slides, antes);
    }
    #[test]
    fn excluir_promove_filhos_e_reordenar_preserva_subarvores() {
        let mut s = Sessao::default();
        s.diagrama_inserir(Categoria::Hierarquias).unwrap();
        let mut e = s.diagrama_textos_selecionados().unwrap();
        e.mover(1, 1);
        assert_eq!(e.nos[1].texto, "Ramo B");
        assert_eq!(e.nos[2].texto, "Ramo A");
        assert_eq!(e.nos[3].texto, "Item 1");
        e.excluir(2).unwrap();
        assert_eq!(e.nos[2].texto, "Item 1");
        assert_eq!(e.nos[2].linha, 1);
        s.diagrama_textos_definir(&e).unwrap();
        assert!(!s
            .diagrama_textos_selecionados()
            .unwrap()
            .nos
            .iter()
            .any(|n| n.texto == "Ramo A"));
    }
    #[test]
    fn promover_e_rebaixar_mudam_o_pai_sem_perder_filhos() {
        let mut s = Sessao::default();
        s.diagrama_inserir(Categoria::Hierarquias).unwrap();
        let mut e = s.diagrama_textos_selecionados().unwrap();
        let ramo_a = e.nos[1].id;
        let raiz = e.nos[0].id;
        e.nivel_alterar(4, true);
        let ramo_b = e.nos.iter().position(|n| n.texto == "Ramo B").unwrap();
        assert_eq!(e.nos[ramo_b].pai, Some(ramo_a));
        assert_eq!(e.nos[ramo_b].linha, 2);
        e.nivel_alterar(ramo_b, false);
        let ramo_b = e.nos.iter().find(|n| n.texto == "Ramo B").unwrap();
        assert_eq!(ramo_b.pai, Some(raiz));
        assert_eq!(ramo_b.linha, 1);
        s.diagrama_textos_definir(&e).unwrap();
    }
    #[test]
    fn organograma_edita_rotulos_e_desfaz_o_conjunto() {
        let mut s = Sessao::default();
        s.diagrama_inserir(Categoria::Organogramas).unwrap();
        let antes = s.apresentacao().clone();
        let mut e = s.diagrama_textos_selecionados().unwrap();
        assert_eq!(
            e.nos.iter().map(|n| n.texto.as_str()).collect::<Vec<_>>(),
            ["Diretoria", "Produto", "Engenharia", "Operações"]
        );
        assert_eq!(
            e.nos.iter().map(|n| n.linha).collect::<Vec<_>>(),
            [0, 1, 1, 1]
        );
        e.nos[0].texto = "Diretora".into();
        e.nos[1].texto = "Produto novo".into();
        s.diagrama_textos_definir(&e).unwrap();
        for antigo in &antes.slides[0].elementos {
            let novo = s
                .slide()
                .elementos
                .iter()
                .find(|n| n.id == antigo.id)
                .unwrap();
            let mut esperado = antigo.clone();
            if let Some(no) = e.nos.iter().find(|n| n.id == antigo.id) {
                esperado.texto = no.texto.clone();
            }
            assert_eq!(novo, &esperado);
        }
        s.desfazer().unwrap();
        assert_eq!(s.apresentacao().slides, antes.slides);
    }
    #[test]
    fn rejeita_edicao_antiga_sem_aplicar_parcialmente() {
        let mut s = Sessao::default();
        s.diagrama_inserir(Categoria::Organogramas).unwrap();
        let mut e = s.diagrama_textos_selecionados().unwrap();
        e.nos[0].texto = "Novo".into();
        s.texto_definir(e.nos[1].id, "Alterado fora do painel")
            .unwrap();
        let antes = s.apresentacao().clone();
        assert!(s.diagrama_textos_definir(&e).is_err());
        assert_eq!(s.apresentacao(), &antes);
    }
    #[test]
    fn todos_os_modelos_e_rotulos_vazios_podem_ser_reabertos() {
        for cat in Categoria::TODAS {
            let mut s = Sessao::default();
            s.diagrama_inserir(cat).unwrap();
            let mut e = s.diagrama_textos_selecionados().unwrap();
            let quantidade = e.nos.len();
            assert!(quantidade > 0);
            e.nos[0].texto.clear();
            s.diagrama_textos_definir(&e).unwrap();
            assert_eq!(
                s.diagrama_textos_selecionados().unwrap().nos.len(),
                quantidade
            );
        }
    }
}
