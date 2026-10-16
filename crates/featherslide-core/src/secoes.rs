//! Seções: pastas que agrupam slides (como no PowerPoint). Uma seção **começa** num slide
//! (`Slide::secao` com o nome) e vai até o slide antes da próxima. Se há seções e o primeiro
//! slide não abre nenhuma, os slides do começo formam a "Seção padrão" (implícita).
//! Recolher e expandir é só da interface.

use crate::sessao::{slide_com_layout, Mudanca, Sessao};

/// Uma seção na ordem da apresentação.
#[derive(Clone, Debug, PartialEq)]
pub struct SecaoInfo {
    /// O índice do slide que abre a seção.
    pub inicio: usize,
    /// O índice depois do último slide dela.
    pub fim: usize,
    pub nome: String,
    /// A "Seção padrão" que ainda não tem nome gravado (os slides antes da primeira seção).
    pub implicita: bool,
}

const PADRAO: &str = "Seção padrão";

impl Sessao {
    /// As seções (vazio se a apresentação não usa seções).
    pub fn secoes(&self) -> Vec<SecaoInfo> {
        let slides = &self.ap.slides;
        let cabecas: Vec<usize> = slides.iter().enumerate().filter(|(_, s)| !s.secao.is_empty()).map(|(i, _)| i).collect();
        if cabecas.is_empty() {
            return Vec::new();
        }
        let mut inicios = cabecas.clone();
        let implicita = cabecas[0] != 0;
        if implicita {
            inicios.insert(0, 0);
        }
        inicios
            .iter()
            .enumerate()
            .map(|(k, &inicio)| SecaoInfo {
                inicio,
                fim: inicios.get(k + 1).copied().unwrap_or(slides.len()),
                nome: if implicita && k == 0 { PADRAO.into() } else { slides[inicio].secao.clone() },
                implicita: implicita && k == 0,
            })
            .collect()
    }

    /// Sugere um nome diferente das seções existentes, sem alterar o documento.
    /// O usuário ainda pode renomear duas seções com nomes iguais se desejar.
    pub fn secao_nome_sugerido(&self) -> String {
        let nomes: Vec<_> = self.secoes().into_iter().map(|s| s.nome).collect();
        for numero in 1..=nomes.len() + 1 {
            let nome = format!("Seção {numero}");
            if !nomes.iter().any(|s| s.eq_ignore_ascii_case(&nome)) {
                return nome;
            }
        }
        "Nova seção".into()
    }

    /// Começa uma seção no slide atual. Se ele já abre uma seção, cria uma
    /// página em branco no FINAL daquela seção e inicia a próxima nela.
    /// É necessário um slide por seção: jamais sobrescrevemos o cabeçalho atual.
    pub fn secao_adicionar(&mut self, nome: &str) -> Result<Mudanca, String> {
        let nome = if nome.trim().is_empty() { "Seção sem título" } else { nome.trim() }.to_string();
        let proxima = if self.ap.slides[self.atual].secao.is_empty() {
            None
        } else {
            self.secoes().into_iter()
                .find(|sec| sec.inicio == self.atual)
                .map(|sec| sec.fim)
        };
        self.editar(move |s| {
            if let Some(pos) = proxima {
                let mut slide = slide_com_layout(&mut s.ap, "em_branco");
                slide.secao = nome;
                s.ap.slides.insert(pos, slide);
                s.atual = pos;
                s.selecao.clear();
            } else {
                let nenhuma = s.ap.slides.iter().all(|sl| sl.secao.is_empty());
                if nenhuma && s.atual > 0 {
                    // Os slides anteriores passam a pertencer à seção padrão.
                    s.ap.slides[0].secao = PADRAO.into();
                }
                s.ap.slides[s.atual].secao = nome;
            }
            Ok(((0..s.ap.slides.len()).collect(), true))
        })
    }

    /// Põe os slides `indices` juntos numa seção nova chamada `nome`: se estavam separados, vêm
    /// para perto do primeiro (na ordem em que estavam). Devolve o índice onde a seção abre.
    pub fn secao_dos_slides(&mut self, indices: &[usize], nome: &str) -> Result<usize, String> {
        let total = self.ap.slides.len();
        let mut escolhidos: Vec<usize> = indices.iter().copied().filter(|&i| i < total).collect();
        escolhidos.sort_unstable();
        escolhidos.dedup();
        if escolhidos.is_empty() {
            return Err("Marque pelo menos um slide.".into());
        }
        let nome = if nome.trim().is_empty() { "Seção sem título" } else { nome.trim() }.to_string();
        let mut posicao = 0;
        self.editar(|s| {
            let havia_secoes = s.ap.slides.iter().any(|slide| !slide.secao.is_empty());
            // Identidade de cada grupo é sua POSIÇÃO ORIGINAL, não o nome.
            // Nomes repetidos são permitidos sem mesclar seções distintas.
            let mut grupos: Vec<usize> = Vec::with_capacity(total);
            let mut nomes: Vec<String> = Vec::new();
            let mut grupo = 0;
            for (i, slide) in s.ap.slides.iter().enumerate() {
                if i == 0 {
                    nomes.push(if slide.secao.is_empty() { PADRAO.into() } else { slide.secao.clone() });
                } else if !slide.secao.is_empty() {
                    grupo += 1;
                    nomes.push(slide.secao.clone());
                }
                grupos.push(grupo);
            }
            let novo_grupo = nomes.len();
            // A nova seção entra na posição do primeiro slide selecionado.
            // Quem ficou antes ou depois mantém seu grupo original.
            posicao = (0..escolhidos[0]).filter(|i| escolhidos.binary_search(i).is_err()).count();
            let (mut fora, mut bloco): (Vec<_>, Vec<_>) = (Vec::new(), Vec::new());
            for (i, (sl, g)) in std::mem::take(&mut s.ap.slides)
                .into_iter().zip(grupos).enumerate()
            {
                if escolhidos.binary_search(&i).is_ok() {
                    bloco.push((sl, novo_grupo));
                } else {
                    fora.push((sl, g));
                }
            }
            let depois = fora.split_off(posicao);
            fora.extend(bloco);
            fora.extend(depois);
            let mut anterior: Option<usize> = None;
            let mut passou_pela_nova = false;
            for (mut slide, grupo) in fora {
                // Preserva os nomes já existentes. Na apresentação originalmente
                // sem seções, o bloco que sobra depois da nova continua com
                // o rótulo "Continuação", como antes desta correção.
                slide.secao = if anterior == Some(grupo) {
                    String::new()
                } else if grupo == novo_grupo {
                    nome.clone()
                } else if !havia_secoes && passou_pela_nova {
                    "Continuação".into()
                } else {
                    nomes[grupo].clone()
                };
                if grupo == novo_grupo {
                    passou_pela_nova = true;
                }
                anterior = Some(grupo);
                s.ap.slides.push(slide);
            }
            s.atual = posicao;
            Ok(((0..s.ap.slides.len()).collect(), true))
        })?;
        Ok(posicao)
    }

    /// Dá outro nome à seção que abre no slide `inicio` (a padrão implícita também pode).
    pub fn secao_renomear(&mut self, inicio: usize, nome: &str) -> Result<Mudanca, String> {
        if inicio >= self.ap.slides.len() {
            return Err("Esse slide não existe.".into());
        }
        let nome = if nome.trim().is_empty() { "Seção sem título" } else { nome.trim() }.to_string();
        self.editar(|s| {
            s.ap.slides[inicio].secao = nome;
            Ok((vec![inicio], true))
        })
    }

    /// Tira a seção que abre no slide `inicio`: os slides passam para a seção de cima. Com
    /// `com_slides`, apaga também os slides dela (nunca todos).
    pub fn secao_remover(&mut self, inicio: usize, com_slides: bool) -> Result<Mudanca, String> {
        let info = self.secoes().into_iter().find(|i| i.inicio == inicio).ok_or("Essa seção não existe.")?;
        if com_slides && info.fim - info.inicio >= self.ap.slides.len() {
            return Err("A apresentação precisa ficar com pelo menos um slide.".into());
        }
        self.editar(|s| {
            if com_slides {
                s.ap.slides.drain(info.inicio..info.fim);
                s.atual = s.atual.min(s.ap.slides.len() - 1);
                s.selecao.clear();
            } else {
                s.ap.slides[inicio].secao.clear();
            }
            Ok(((0..s.ap.slides.len()).collect(), true))
        })
    }

    /// Tira todas as seções (os slides ficam como estão).
    pub fn secoes_remover_todas(&mut self) -> Result<Mudanca, String> {
        self.editar(|s| {
            for sl in &mut s.ap.slides {
                sl.secao.clear();
            }
            Ok(((0..s.ap.slides.len()).collect(), true))
        })
    }
}



impl Sessao {
    /// Move a seção de origem para perto do destino em uma única edição.
    /// Ao arrastar para baixo, insere DEPOIS do destino; para cima, ANTES.
    /// O índice é o slide que abre a seção, nunca o nome (nomes repetidos são válidos).
    /// Não afeta os IDs dos slides e funciona mesmo com seções recolhidas na GUI.
    pub fn secao_mover_para(&mut self, inicio: usize, destino_inicio: usize) -> Result<Mudanca, String> {
        let secoes = self.secoes();
        let origem = secoes.iter().position(|s| s.inicio == inicio)
            .ok_or("Selecione o cabeçalho de uma seção válida.")?;
        let destino = secoes.iter().position(|s| s.inicio == destino_inicio)
            .ok_or("O cabeçalho de destino não existe.")?;
        if origem == destino {
            return Err("Solte sobre outra seção para mover.".into());
        }
        let (comeco, fim) = (secoes[origem].inicio, secoes[origem].fim);
        let comprimento = fim - comeco;
        let inserir = if origem < destino {
            secoes[destino].fim - comprimento
        } else {
            secoes[destino].inicio
        };
        self.editar(move |s| {
            // A seção padrão implícita precisa de um cabeçalho próprio antes
            // de mudar de lugar; caso contrário, seria absorvida pela anterior.
            if secoes.first().is_some_and(|sec| sec.implicita) {
                s.ap.slides[0].secao = PADRAO.into();
            }
            let bloco: Vec<_> = s.ap.slides.drain(comeco..fim).collect();
            s.ap.slides.splice(inserir..inserir, bloco);
            s.atual = inserir;
            s.selecao.clear();
            Ok(((0..s.ap.slides.len()).collect(), true))
        })
    }

    /// Move uma seção um grupo para cima ou para baixo (menu de contexto).
    pub fn secao_mover(&mut self, inicio: usize, direcao: i32) -> Result<Mudanca, String> {
        let secoes = self.secoes();
        let origem = secoes.iter().position(|s| s.inicio == inicio)
            .ok_or("Selecione o cabeçalho de uma seção válida.")?;
        if !matches!(direcao, -1 | 1) {
            return Err("Essa seção não pode ser movida nessa direção.".into());
        }
        let destino = origem as i32 + direcao;
        if destino < 0 || destino >= secoes.len() as i32 {
            return Err("Essa seção não pode ser movida nessa direção.".into());
        }
        self.secao_mover_para(inicio, secoes[destino as usize].inicio)
    }

    /// Transfere os slides escolhidos para o FINAL da seção de destino.
    /// Os slides não são duplicados: IDs, notas, animações e links permanecem.
    /// Se uma seção de origem ficar sem slides, ela deixa de existir.
    pub fn slides_para_secao(&mut self, indices: &[usize], destino_inicio: usize) -> Result<Mudanca, String> {
        let secoes = self.secoes();
        let destino = secoes.iter().position(|s| s.inicio == destino_inicio)
            .ok_or("A seção de destino não existe.")?;
        let total = self.ap.slides.len();
        let mut escolhidos: Vec<usize> = indices.iter().copied().collect();
        escolhidos.sort_unstable();
        escolhidos.dedup();
        if escolhidos.is_empty() || escolhidos.iter().any(|&i| i >= total) {
            return Err("Marque pelo menos um slide válido para transferir.".into());
        }
        // Identidade da seção é a posição de seu cabeçalho, NÃO o texto do nome:
        // duas seções com nomes iguais continuam independentes.
        let grupos: Vec<usize> = (0..total).map(|i| {
            secoes.iter().position(|s| s.inicio <= i && i < s.fim).unwrap_or(0)
        }).collect();
        if escolhidos.iter().all(|&i| grupos[i] == destino) {
            return Err("Os slides já estão nessa seção.".into());
        }
        if (secoes[destino].inicio..secoes[destino].fim).all(|i| escolhidos.contains(&i)) {
            return Err("Deixe ao menos um slide na seção de destino.".into());
        }
        self.editar(move |s| {
            let mut restantes = Vec::new();
            let mut movidos = Vec::new();
            for (i, sl) in std::mem::take(&mut s.ap.slides).into_iter().enumerate() {
                if escolhidos.binary_search(&i).is_ok() {
                    movidos.push((sl, destino));
                } else {
                    restantes.push((sl, grupos[i]));
                }
            }
            let pos = restantes.iter().rposition(|(_, grupo)| *grupo == destino)
                .ok_or("A seção de destino ficou vazia.")? + 1;
            let quantidade = movidos.len();
            restantes.splice(pos..pos, movidos);
            let mut anterior = None;
            for (mut sl, grupo) in restantes {
                sl.secao = if anterior == Some(grupo) {
                    String::new()
                } else {
                    secoes[grupo].nome.clone()
                };
                s.ap.slides.push(sl);
                anterior = Some(grupo);
            }
            s.atual = pos;
            s.selecao.clear();
            Ok(((0..s.ap.slides.len()).collect(), quantidade > 0))
        })
    }
}

#[cfg(test)]
mod estrutura_22_testes {
    use crate::sessao::Sessao;

    fn tres_secoes() -> Sessao {
        let mut s = Sessao::default();
        for _ in 0..4 { s.slide_novo("em_branco").unwrap(); }
        for (inicio, nome) in [(0, "A"), (2, "B"), (4, "C")] {
            s.slide_ir(inicio).unwrap();
            s.secao_adicionar(nome).unwrap();
        }
        s
    }

    #[test]
    fn criar_varias_secoes_seguidas_preserva_slides_e_desfazer() {
        let mut s = Sessao::default();
        let original = s.apresentacao().slides[0].id;
        s.secao_adicionar("Seção 1").unwrap();
        let titulo = s.apresentacao().slides[0].elementos.clone();
        s.secao_adicionar("Seção 2").unwrap();
        assert_eq!(s.secoes().len(), 2);
        assert_eq!(s.secoes().iter().map(|x| x.nome.as_str()).collect::<Vec<_>>(),
            vec!["Seção 1", "Seção 2"]);
        assert_eq!(s.apresentacao().slides[0].id, original);
        assert_eq!(s.apresentacao().slides[0].elementos, titulo);
        assert_eq!(s.atual(), 1);
        s.secao_adicionar("Seção 3").unwrap();
        assert_eq!(s.secoes().len(), 3);
        assert_eq!(s.apresentacao().slides.len(), 3);
        s.desfazer().unwrap();
        assert_eq!(s.secoes().len(), 2);
        s.refazer().unwrap();
        assert_eq!(s.secoes().len(), 3);
    }

    #[test]
    fn criar_nova_secao_entre_outras_nao_move_slides_existentes() {
        let mut s = tres_secoes();
        let ids: Vec<_> = s.apresentacao().slides.iter().map(|sl| sl.id).collect();
        s.slide_ir(2).unwrap();
        s.secao_adicionar("Entre B e C").unwrap();
        assert_eq!(s.secoes().iter().map(|s| s.nome.as_str()).collect::<Vec<_>>(),
            vec!["A", "B", "Entre B e C", "C"]);
        assert_eq!(s.apresentacao().slides.len(), ids.len() + 1);
        for (id, sl) in ids.iter().zip(s.apresentacao().slides.iter().filter(|sl| sl.secao != "Entre B e C")) {
            assert_eq!(*id, sl.id);
        }
        s.desfazer().unwrap();
        assert_eq!(s.apresentacao().slides.iter().map(|sl| sl.id).collect::<Vec<_>>(), ids);
    }

    #[test]
    fn nova_secao_com_slides_nao_mescla_grupos_com_mesmo_nome() {
        let mut s = Sessao::default();
        for _ in 0..4 { s.slide_novo("em_branco").unwrap(); }
        s.slide_ir(0).unwrap(); s.secao_adicionar("Repetida").unwrap();
        s.slide_ir(3).unwrap(); s.secao_adicionar("Repetida").unwrap();
        assert_eq!(s.secoes().len(), 2);
        s.secao_dos_slides(&[1], "Repetida").unwrap();
        assert_eq!(s.secoes().len(), 4); // original, nova, continuação e outra homônima
        assert_eq!(s.secoes().iter().map(|s| s.nome.as_str()).collect::<Vec<_>>(),
            vec!["Repetida", "Repetida", "Repetida", "Repetida"]);
        s.desfazer().unwrap();
        assert_eq!(s.secoes().len(), 2);
    }

    #[test]
    fn nome_sugerido_nao_colide_com_os_existentes() {
        let mut s = Sessao::default();
        assert_eq!(s.secao_nome_sugerido(), "Seção 1");
        s.secao_adicionar("Seção 1").unwrap();
        assert_eq!(s.secao_nome_sugerido(), "Seção 2");
        s.secao_adicionar("Seção 2").unwrap();
        assert_eq!(s.secao_nome_sugerido(), "Seção 3");
    }

    #[test]
    fn mover_secao_preserva_grupos_e_desfazer() {
        let mut s = tres_secoes();
        let ids: Vec<_> = s.apresentacao().slides.iter().map(|s| s.id).collect();
        s.secao_mover(2, 1).unwrap();
        assert_eq!(s.apresentacao().slides.iter().map(|s| s.id).collect::<Vec<_>>(),
            vec![ids[0], ids[1], ids[4], ids[2], ids[3]]);
        assert_eq!(s.secoes().iter().map(|s| s.nome.as_str()).collect::<Vec<_>>(),
            vec!["A", "C", "B"]);
        s.desfazer().unwrap();
        assert_eq!(s.apresentacao().slides.iter().map(|s| s.id).collect::<Vec<_>>(), ids);
    }

    #[test]
    fn arrastar_secao_para_outro_grupo_nao_adjacente_e_desfazer() {
        let mut s = tres_secoes();
        let ids: Vec<_> = s.apresentacao().slides.iter().map(|sl| sl.id).collect();
        // A (slides 0,1) movida após C (slide 4).
        s.secao_mover_para(0, 4).unwrap();
        assert_eq!(s.secoes().iter().map(|sec| sec.nome.as_str()).collect::<Vec<_>>(),
            vec!["B", "C", "A"]);
        assert_eq!(s.apresentacao().slides.iter().map(|sl| sl.id).collect::<Vec<_>>(),
            vec![ids[2], ids[3], ids[4], ids[0], ids[1]]);
        s.desfazer().unwrap();
        assert_eq!(s.apresentacao().slides.iter().map(|sl| sl.id).collect::<Vec<_>>(), ids);
        s.refazer().unwrap();
        assert_eq!(s.secoes().iter().map(|sec| sec.nome.as_str()).collect::<Vec<_>>(),
            vec!["B", "C", "A"]);
        assert!(s.secao_mover_para(0, 0).is_err());
    }

    #[test]
    fn arrastar_secao_para_cima_preserva_slides_e_nomes_repetidos() {
        let mut s = tres_secoes();
        s.secao_renomear(4, "A").unwrap();
        let ids: Vec<_> = s.apresentacao().slides.iter().map(|sl| sl.id).collect();
        // C (agora nome A, mas grupo distinto) sobe para antes de A.
        s.secao_mover_para(4, 0).unwrap();
        assert_eq!(s.secoes().iter().map(|sec| sec.nome.as_str()).collect::<Vec<_>>(),
            vec!["A", "A", "B"]);
        assert_eq!(s.apresentacao().slides.iter().map(|sl| sl.id).collect::<Vec<_>>(),
            vec![ids[4], ids[0], ids[1], ids[2], ids[3]]);
    }

    #[test]
    fn arrastar_secao_padrao_implicita_preserva_limite() {
        let mut s = Sessao::default();
        for _ in 0..3 { s.slide_novo("em_branco").unwrap(); }
        s.slide_ir(2).unwrap();
        s.secao_adicionar("Segunda").unwrap();
        assert!(s.secoes()[0].implicita == false);
        // Forçar padrão implícita: é um formato ainda possível ao importar arquivos.
        s.ap.slides[0].secao.clear();
        assert!(s.secoes()[0].implicita);
        s.secao_mover_para(0, 2).unwrap();
        assert_eq!(s.secoes().iter().map(|sec| sec.nome.as_str()).collect::<Vec<_>>(),
            vec!["Segunda", "Seção padrão"]);
        assert_eq!(s.secoes().len(), 2);
    }

    #[test]
    fn transferir_slide_conserva_ids_e_fronteiras() {
        let mut s = tres_secoes();
        let ids: Vec<_> = s.apresentacao().slides.iter().map(|s| s.id).collect();
        s.slides_para_secao(&[1, 2], 4).unwrap();
        assert_eq!(s.apresentacao().slides.iter().map(|s| s.id).collect::<Vec<_>>(),
            vec![ids[0], ids[3], ids[4], ids[1], ids[2]]);
        assert_eq!(s.secoes().iter().map(|s| s.nome.as_str()).collect::<Vec<_>>(),
            vec!["A", "B", "C"]);
        s.desfazer().unwrap();
        assert_eq!(s.apresentacao().slides.iter().map(|s| s.id).collect::<Vec<_>>(), ids);
    }
}
