# Verificação de funcionalidades — FeatherSlide

## Diagrama personalizado — atualização de 08/10/2026

O painel abre ao inserir e reabre em **Inserir → Editar diagrama**. Hierarquias
mostram cada pai seguido de seus filhos: Nível 1 → Ramo A → Itens 1 e 2;
depois Ramo B e Ramo C → Itens 3 e 4. Os recuos representam profundidade.
**+ Filho**, **+ Irmão**, **Adicionar item**, **Excluir**, **Promover**,
**Rebaixar** e as setas alteram o rascunho. Excluir um pai mantém seus filhos
no nível acima. Há layouts personalizados de lista, processo, ciclo e árvore.

**Aplicar** registra uma transação; cancelar/Esc descarta o rascunho. Alterar
apenas texto preserva a geometria. Alterar a estrutura redistribui os nós no
slide e recria conectores vinculados; o ciclo ganha/retira etapas e a árvore
liga cada nó ao seu pai. São aceitos até 64 itens. Não é SmartArt OOXML.

Pais, layout e ordem persistem no campo `dados` dos nós, com marcadores
`#DIA_PARENT`, `#DIA_LAYOUT` e `#DIA_ORDER`, sem mudar o esquema `.fsdz`.
Duplicação remapeia pais/conectores e cria um grupo independente. Diagramas
antigos em árvore inferem o pai pela linha anterior e proximidade horizontal.

Controles localizados em `gui/dados/pt-BR.json` e `en-US.json`. Evidências:
`core/src/editor_diagrama.rs`, `gui/src/editor_diagrama.rs` e teste
`diagrama_personalizado_preserva_pais_ordem_e_conectores` em `io/tests/fsdz.rs`.
Validação gráfica interativa pendente.

## Ferramentas de texto — atualização de 08/10/2026

Revisão reúne dicionário, sinônimos, autocorreção, capitalização, idioma,
tradução por glossário e contagem. Página Inicial oferece colagem sem estilo,
colagem original de elementos internos e pincel (selecione a origem, ative e
clique no destino; Esc cancela). Autocorreção e capitalização vêm desligadas.
Textos dos novos controles estão em `gui/dados/pt-BR.json` e `en-US.json`;
regras pessoais ficam no `config.json` local. Não há consultas de rede.

Ortografia F7 também verifica notas e células, identifica a origem do erro,
valida que o trecho não mudou e corrige em uma edição reversível. Sinônimos
começam com um vocabulário pequeno editável. O glossário traduz apenas
expressões cadastradas no par de idiomas; tradução livre continua parcial.

Evidências: testes em `core/src/ferramentas_texto.rs`,
`gui/src/ferramentas_texto.rs`, `gui/src/localidade.rs` e
`ortografia/src/lib.rs`, além dos testes existentes da GUI e do corretor.
Validação gráfica interativa ainda pendente.

## Outras galerias — atualização de 08/10/2026

Implementação em `core/src/outras_galerias.rs`, galeria em
`gui/src/galeria_figuras.rs`, captura livre em `gui/src/palco.rs` e despacho em
`gui/src/app.rs`. Evidências automatizadas: testes do módulo, round-trip em
`io/tests/fsdz.rs` e teste `galerias_e_desenho_livre_pela_interface`.

| IDs | Estado | Implementação e limites |
| --- | --- | --- |
| FTS-0327 | Feito | Categoria Fluxogramas com decisão, processo, processo predefinido, início/fim, entrada/saída, documento e armazenamento. |
| FTS-0328 | Feito | Losango de decisão vetorial. |
| FTS-0329 | Feito | Processo simples e processo predefinido. |
| FTS-0330 | Feito | Cilindro de armazenamento e documento. |
| FTS-0331 | Feito | Reto, cotovelo e curvo entre duas formas; movimento, undo, cópia e reabertura preservam âncoras. Ancoragem por caixa, sem evitar obstáculos. |
| FTS-0332 | Feito | Balão de fala com cauda. |
| FTS-0333 | Feito | Balão lobulado com círculos de pensamento. |
| FTS-0334 | Feito | Faixa com pontas recortadas. |
| FTS-0335 | Feito | Fita dobrada com detalhes de dobra. |
| FTS-0336 | Feito | Onda Bézier fechada. |
| FTS-0337 | Feito | Explosão de 12 pontas. |
| FTS-0338 | Feito | Mais, menos, multiplicação, divisão, igual e diferente. |
| FTS-0339 | Feito | Início/anterior/próximo com destino de slide; Informação aceita hiperlink configurado pelos comandos existentes. |
| FTS-0340 | Feito | Traço livre aberto por arrasto, prévia, Esc, undo e persistência; sem editor de vértices. |

Todas são formas vetoriais editáveis por cor, contorno, escala e rotação,
armazenadas em `dados` como `SVG1` no `.fsdz` v11. Texto deve ser sobreposto
como caixa separada e agrupado. Excluir alvo de conector mantém seu último
traço. Validação visual no desktop ainda pendente.

Levantamento inicial do código local em 07/10/2026, para acompanhar a
[`checklist completa`](INVENTARIO-FUNCIONALIDADES.md). Escopo examinado:
crates nativos `featherslide-core`, `featherslide-io`, `featherslide-gui`,
`featherslide-ortografia`, `featherslide-mcp` e seus testes.

**Feito** identifica implementação no escopo anotado; não significa
equivalência completa ao PowerPoint ou validação visual. **Parcial** registra
limitações. **Pendente** indica ausência no modelo/fluxo examinado.
**A verificar** mantém a investigação em aberto. **Referência** é material
de consulta. Cada ID deve ser interpretado junto à evidência e à limitação.
Não há pontuação de conclusão calculada: os ~40% são a estimativa funcional
do usuário, e o catálogo contém duplicações, requisitos e propostas opcionais.

## Principais conclusões

- Editor nativo: slides, texto por caixa, formas, imagens, tabelas/gráficos
  básicos, seleção, proporções, organização e desfazer/refazer estão presentes.
- Há seis transições nativas e seis animações, além de Nenhuma. Hover e clique
  reproduzem a prévia completa; a renderização Cortina ainda merece conferência
  visual, porque o caminho atual usa esmaecimento em vez de recorte progressivo.
- PPTX/ODP são importação parcial. **Não existe gravação PPTX** nem preservação
  geral das partes desconhecidas. Salvar uma importação produz `.fsdz`; não
  comprova ida e volta PowerPoint sem perda.
- Tabelas e gráficos nativos existem, mas `p:graphicFrame` não é interpretado
  pelo importador PPTX. Mestres e grupos são achatados, sem editor de mestre.
- WordArt tem contornos vetoriais, estilos, sombra e deformações no palco e no PDF;
  importação/exportação PPTX e equivalência completa continuam pendentes. Equações são presets Unicode. Não há
  texto rico por trecho nem editor LaTeX/OMML. Seleção de texto por Shift/arraste
  e copiar/recortar/colar existem no cursor nativo.
- Áudio, vídeo, gravação, 3D, VBA e SmartArt **OOXML estruturado** continuam
  pendentes; há modelos locais de 13 categorias de diagramas com nós e conectores
  vetoriais editáveis e uma categoria parcial para diagramas com imagens.
  Operações booleanas de formas já estão implementadas no core.
- Modo apresentador está acessível na faixa, inclusive em uma única tela, com
  seleção do monitor, notas, próximo slide e cronômetro pausável. Teste real
  com dois monitores continua pendente. Impressão física também não foi validada.
- Ortografia depende de dicionários locais; gramática usa regras limitadas.
- SVG de ícones Tabler não equivale a importar/editar qualquer arquivo SVG.

## Contagem dos estados deste levantamento

Contagem de registros, **sem usar estes números como percentual funcional**:

| Estado | Entradas |
| --- | ---: |
| Feito | 479 |
| Parcial | 162 |
| Pendente | 455 |
| A verificar | 578 |
| Referência | 7 |
| Cancelado | 3 |

## Correção de criação consecutiva de seções (08/10/2026)

- No botão **Adicionar seção aqui**, se o slide selecionado já iniciar uma
  seção, o motor insere um slide em branco no final do grupo atual e abre uma
  segunda seção nele, preservando todos os slides preexistentes. A ação inteira
  é reversível com um Ctrl+Z.
- Novas seções recebem nomes sugeridos distintos (Seção 1, Seção 2...); o
  usuário continua podendo renomear duas seções com o mesmo nome.
- **Criar seção com slides marcados** identifica os grupos antigos por sua
  posição, e não pelo texto do nome. Seções homônimas não se fundem.
- Testes de regressão cobrem criação consecutiva, fronteira de grupos, nomes
  repetidos e desfazer/refazer. Verificação visual no Ubuntu continua necessária.

## Categorias de diagramas — seção 9.1 (08/10/2026)

- **Inserir → Diagramas:** 14 categorias com prévia vetorial, incluindo listas,
  processos, ciclos, hierarquias, organogramas, relações, matrizes, pirâmides,
  Venn, radial, engrenagens, fluxo e comparação.
- O motor `core/src/diagramas.rs` gera **formas nativas** de texto, figuras e
  conectores com um mesmo identificador de grupo; usa uma única transação de
  desfazer. Arquivos `.fsdz` preservam os elementos sem novo formato.
- **FTS-0595 (Parcial):** cartões para diagramas com imagens têm espaços
  ilustrativos vetoriais, mas inserir uma imagem como nó ainda não funciona;
  permanece pendente o requisito FTS-0617.
- As categorias **não são objetos SmartArt OOXML**. O painel lateral de rótulos
  (FTS-0612) oferece edição de estrutura nativa e quatro layouts personalizados;
  importação/exportação e demais layouts de SmartArt continuam pendentes.
- Testes verificam 14 layouts, limites dos slides 16:9 e 4:3, IDs/grupos,
  desfazer/refazer e preservação de `.fsdz`. Verificação visual no Ubuntu
  continua necessária.

## Animações de entrada — seção 14.1 (08/10/2026)

- **12 efeitos novos**: Surgir, Flutuar, Dividir, Apagar, Forma, Roda,
  Barras, Girar, Rebote, Deslizar, Expandir e Contrair. Os efeitos existentes
  Aparecer, Esmaecer e Zoom permanecem. Voar/Revelar/Pulsar também continuam.
- `core/src/modelo.rs`: códigos posicionais 0–6 preservados; 7–18 anexados,
  `tem_direcao()` atualizado. `.fsdz` **v10** para não silenciar códigos novos
  ao abrir em leitores antigos; leitor v10 abre arquivos anteriores.
- `gui/src/desenho.rs`: efeitos compartilham interpolador de movimento,
  escala, opacidade, rotação e recortes por elemento; Barras utiliza faixas
  progressivas e Dividir duas metades, sem alterar os dados dos elementos.
- `gui/src/faixa_abas.rs`: seletor compacto com todas as 19 opções (inclusive
  ênfase e nenhuma), mantendo atalhos Aparecer/Esmaecer/Zoom. Selecionar o
  efeito executa a mesma prévia que aparece durante a apresentação.
- Testes: compatibilidade dos códigos históricos, 12 efeitos gravados no
  `.fsdz`, estado inicial e final, deslocamento/escala, máscaras e undo.
  Máscaras sobre imagens raster dependem da versão/backend Iced e precisam de
  verificação visual específica no Ubuntu.

## Mapa de implementação

Caminhos abaixo são relativos a `featherslide/crates/`. IDs sem evidência
registrada permanecem **A verificar** na checklist. Este é um levantamento
inicial; não há certificação visual ou auditoria completa dos 1.684 registros.

| ID | Estado | Evidência no código ou motivo | Limitação |
| --- | --- | --- | --- |
| FTS-0001 | Feito | app.rs: tratar/abrir_dialogo_abrir/gravar_no_caminho; modelos.rs; recentes.rs | Validação visual não realizada neste levantamento. |
| FTS-0002 | Feito | arquivos.rs: Acao::EmBranco; core/src/slides.rs: slide_layout | Criação com layout em_branco, sem caixas de título/conteúdo. |
| FTS-0003 | Feito | app.rs: tratar/abrir_dialogo_abrir/gravar_no_caminho; modelos.rs; recentes.rs | Validação visual não realizada neste levantamento. |
| FTS-0004 | Feito | app.rs: tratar/abrir_dialogo_abrir/gravar_no_caminho; modelos.rs; recentes.rs | Validação visual não realizada neste levantamento. |
| FTS-0005 | Feito | app.rs: tratar/abrir_dialogo_abrir/gravar_no_caminho; modelos.rs; recentes.rs | Validação visual não realizada neste levantamento. |
| FTS-0006 | Feito | arquivos.rs: abrir_documento; app.rs: update/gravar_no_caminho | Comando explícito e reconhecimento de permissão readonly. Bloqueia edição e sobrescrita; permite salvar cópia. Teste de proteção do documento e arquivo original. |
| FTS-0007 | Feito | arquivos.rs: fechar_documento/tratar_arquivos | Fechar a apresentação ativa com confirmação de alterações pendentes e rascunho recuperável; ativa outra sessão ou volta ao Início. |
| FTS-0008 | Feito | arquivos.rs: Acao::FecharTodas/checkpoints | Fecha todas somente após preservar alterações em rascunhos. Teste de cancelamento e recuperação dos dois documentos. |
| FTS-0009 | Feito | app.rs: tratar/abrir_dialogo_abrir/gravar_no_caminho; modelos.rs; recentes.rs | Validação visual não realizada neste levantamento. |
| FTS-0010 | Feito | app.rs: tratar/abrir_dialogo_abrir/gravar_no_caminho; modelos.rs; recentes.rs | Validação visual não realizada neste levantamento. |
| FTS-0011 | Feito | app.rs: salvar_copia/abrir_dialogo_copia; configuracoes.rs; configuracao.rs: salvar_copia | Cópia .fsdz sem mudar caminho, nome, estado sujo ou original. Configurações → Salvamento; Ctrl+Alt+S regravável. Teste cobre sucesso, destino original e falha. |
| FTS-0012 | Feito | app.rs: salvar_automaticamente/update; configuracao.rs: novo_rascunho | Intervalo real após última alteração; gravação atômica de arquivo nomeado ou rascunho exclusivo sem nome. Falha mantém alterações e tenta novamente após intervalo. Recuperação ao iniciar e histórico de versões agora estão implementados nos itens FTS-0013/0014/0016. |
| FTS-0013 | Feito | arquivos.rs: Acao::Recuperar/atualizar_arquivos; app.rs: new | Painel de recuperação oferecido ao iniciar quando há rascunhos. Abre uma cópia editável sem sobrescrever a origem; checkpoints a cada 30 s. |
| FTS-0014 | Feito | arquivos.rs: checkpoints; io/src/documentos.rs: salvar/historico | Rascunhos periódicos persistem após interrupção; versões salvas anteriores são preservadas antes da substituição atômica. Recupera até o último checkpoint/salvamento. |
| FTS-0015 | Feito | arquivos.rs: Acao::Reverter | Relê o arquivo salvo; pede confirmação e preserva alterações pendentes em rascunho. Teste cobre restauração e conteúdo recuperável. |
| FTS-0016 | Feito | io/src/documentos.rs: salvar/historico; arquivos.rs: view_arquivos | Histórico persistente em diretório .<arquivo>.versoes ao lado do arquivo; painel abre versão como nova cópia. Ainda não é Git embutido no .fsdz. |
| FTS-0017 | Feito | arquivos.rs: Acao::Duplicar | Duplica o conteúdo como nova sessão sem caminho, com edições e desfazer independentes. Teste de independência e salvamento automático da sessão anterior. |
| FTS-0018 | Feito | arquivos.rs: renomear_documento | Renomeia arquivo e histórico juntos na mesma pasta; recusa destino existente e atualiza recentes/caminho. Conversão ou cópia para outro local usa Salvar como/cópia. |
| FTS-0019 | Feito | arquivos.rs: excluir_documento | Confirmação no app; remove o arquivo e preserva cópia em Recuperação. Teste cobre cancelar, confirmar e reabertura possível. |
| FTS-0020 | Feito | arquivos.rs: Documento/guardar_documento; app.rs: trocar_sessao | Várias sessões em memória com caminho, rascunho, leitura e histórico de desfazer próprios. Abrir/criar não descarta a sessão atual. |
| FTS-0021 | Feito | arquivos.rs: barra_documentos/trocar_documento | Barra de documentos com rolagem horizontal; trocar preserva edições, seleção do motor e desfazer, limpando caches e painéis de edição. |
| FTS-0022 | Feito | io/src/documentos.rs: listar/pesquisar; arquivos.rs: view_arquivos | Pesquisa por nome no local escolhido, sem diferenciar maiúsculas, filtrando formatos aceitos. Sem busca recursiva/indexação. |
| FTS-0023 | Feito | configuracao.rs: favoritos; arquivos.rs: Acao::Favorito/Local | Adicionar/remover pastas favoritas e navegar pelo painel. Persistência nas preferências; teste cobre serialização e compatibilidade com configuração antiga. |
| FTS-0024 | Feito | app.rs: new; arquivos.rs: abrir_argumentos | Abre caminhos passados ao executável, inclusive múltiplos arquivos e nomes com espaços; teste cobre duas sessões. |
| FTS-0025 | Feito | app.rs: subscription: window::Event::FileDropped | Arrastar arquivo para a janela chama o mesmo fluxo de abertura preservando a sessão anterior. A entrega do evento pelo desktop não foi validada visualmente. |
| FTS-0026 | Feito | core/src/slides.rs: apresentacao_importar; arquivos.rs: ImportarEscolhido | Importa slides nativos/PPTX/ODP para a apresentação atual, remapeia IDs/grupos/gatilhos, ajusta dimensões e incorpora fontes. Um passo de desfazer; importadores mantêm suas limitações documentadas. |
| FTS-0027 | Parcial | io/src/pptx/mod.rs: ler; io/src/odp.rs: ler | Importação parcial; não há exportador destes formatos. |
| FTS-0038 | Parcial | io/src/pptx/mod.rs: ler; io/src/odp.rs: ler | Importação parcial; não há exportador destes formatos. |
| FTS-0039 | Parcial | io/src/pdf.rs; app.rs: exportar_pdf | Sem fontes próprias embutidas; Unicode fora de WinAnsi pode virar ?; sombra não exportada. |
| FTS-0053 | Feito | Apresentacao::titulo; painel Iced; importação de dc:title do PPTX/ODP; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Disponível no .fsdz v7 e no painel Propriedades; compatibilidade de importação só para campos publicados por PPTX/ODP; inspeção visual no Ubuntu pendente. |
| FTS-0054 | Feito | metadados.assunto; importação de dc:subject; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Disponível no .fsdz v7 e no painel Propriedades; compatibilidade de importação só para campos publicados por PPTX/ODP; inspeção visual no Ubuntu pendente. |
| FTS-0055 | Feito | Apresentacao::autor; painel Iced; importação de dc:creator / meta:initial-creator; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Disponível no .fsdz v7 e no painel Propriedades; compatibilidade de importação só para campos publicados por PPTX/ODP; inspeção visual no Ubuntu pendente. |
| FTS-0056 | Feito | metadados.empresa; docProps/app.xml: Company; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Disponível no .fsdz v7 e no painel Propriedades; compatibilidade de importação só para campos publicados por PPTX/ODP; inspeção visual no Ubuntu pendente. |
| FTS-0057 | Feito | metadados.gerente; docProps/app.xml: Manager; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Disponível no .fsdz v7 e no painel Propriedades; compatibilidade de importação só para campos publicados por PPTX/ODP; inspeção visual no Ubuntu pendente. |
| FTS-0058 | Feito | metadados.categoria; cp:category em PPTX; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Disponível no .fsdz v7 e no painel Propriedades; compatibilidade de importação só para campos publicados por PPTX/ODP; inspeção visual no Ubuntu pendente. |
| FTS-0059 | Feito | metadados.palavras_chave; cp:keywords/meta:keyword; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Disponível no .fsdz v7 e no painel Propriedades; compatibilidade de importação só para campos publicados por PPTX/ODP; inspeção visual no Ubuntu pendente. |
| FTS-0060 | Feito | metadados.comentarios, diferente de comentários por slide; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Disponível no .fsdz v7 e no painel Propriedades; compatibilidade de importação só para campos publicados por PPTX/ODP; inspeção visual no Ubuntu pendente. |
| FTS-0061 | Feito | metadados.criado_em_utc; ISO de PPTX/ODP quando informado; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | UTC; arquivos legados ou importados sem data exibem 'Não disponível'; fonte PPTX/ODP só quando expõe o campo. |
| FTS-0062 | Feito | metadados.modificado_em_utc; carimbo automático da última alteração; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | UTC; arquivos legados ou importados sem data exibem 'Não disponível'; fonte PPTX/ODP só quando expõe o campo. |
| FTS-0063 | Feito | metadados.ultimo_autor; cp:lastModifiedBy e dc:creator; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Disponível no .fsdz v7 e no painel Propriedades; compatibilidade de importação só para campos publicados por PPTX/ODP; inspeção visual no Ubuntu pendente. |
| FTS-0064 | Feito | metadados.revisoes; incremento a cada edição inclusive desfazer/refazer; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Disponível no .fsdz v7 e no painel Propriedades; compatibilidade de importação só para campos publicados por PPTX/ODP; inspeção visual no Ubuntu pendente. |
| FTS-0065 | Feito | metadados.tempo_edicao_segundos; períodos ativos estimados e importação do original; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Contagem automática aproximada dos intervalos entre edições de até 300 s; inclui tempo vindo de PPTX/ODP quando informado, não é cronômetro exato de foco. |
| FTS-0066 | Feito | metadados.propriedades_personalizadas; chave/valor com suporte a Unicode; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Mapa de até 100 chaves, nomes únicos; valores novos/removidos pelo painel; Unicode em campos de comprimento prefixado no Arrow, sem JSON. |
| FTS-0067 | Feito | metadados.idioma; pt-BR por padrão; metadado distinto do corretor ortográfico; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Propriedade informativa do documento, não altera automaticamente a língua do corretor ortográfico. |
| FTS-0068 | Feito | metadados.identificador; criado localmente, estável e renovado ao duplicar; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | ID FeatherSlide sem rede; gerado para documentos novos/legados e renovado na duplicação; não equivalente ao GUID original do PowerPoint. |
| FTS-0069 | Feito | metadados.estendidos; chave/valor e dados de origem da importação; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Mapa de até 100 chaves, nomes únicos; valores novos/removidos pelo painel; Unicode em campos de comprimento prefixado no Arrow, sem JSON. |
| FTS-0070 | Feito | metadados::estatisticas; slides, elementos, notas, textos, palavras etc. atualizados; core/src/metadados.rs; gui/src/propriedades.rs; io/src/fsdz.rs; io/src/metadados_externos.rs | Contagens em tempo real de slides, elementos, imagens, caixas de texto, notas, palavras/caracteres e comentários; desconsidera elementos de fundo. |
| FTS-0085 | Parcial | pacote.rs: abrir/parte; xml.rs: ler; tests/importar.rs | Há mensagens de erro e testes; limites de recursos/fuzzing não comprovados. |
| FTS-0086 | Parcial | pacote.rs: parte | Leitura em memória, sem extração em disco; isso não equivale a auditoria de todos os leitores da suíte. |
| FTS-0087 | Pendente | pacote.rs: parte | read_to_end não impõe orçamento de descompressão por recurso. |
| FTS-0089 | Feito | core/src/slides.rs: slide_novo; gui/src/faixa.rs; gui/src/miniaturas.rs | Inclui com layout escolhido; undo no motor. |
| FTS-0090 | Feito | core/src/operacoes_slides.rs: slide_inserir_em; gui/src/menu_contexto.rs | Insere inclusive antes do primeiro slide. |
| FTS-0091 | Feito | core/src/operacoes_slides.rs: slide_inserir_em; gui/src/menu_contexto.rs | Novo slide em posição atual+1. |
| FTS-0092 | Feito | core/src/operacoes_slides.rs: slides_copiar/slides_colar; gui/src/app.rs | Vários slides em uma única transação, IDs independentes. |
| FTS-0093 | Feito | core/src/operacoes_slides.rs: slides_copiar; gui/src/app.rs; gui/src/faixa.rs | Área de transferência interna da aplicação; não exporta formato para outros programas. |
| FTS-0094 | Feito | core/src/operacoes_slides.rs: slides_recortar; gui/src/app.rs | Recusa recortar o último slide; um passo de desfazer; mantém cópia interna. |
| FTS-0095 | Feito | core/src/operacoes_slides.rs: slides_colar; gui/src/app.rs | Remapeia IDs e links entre os slides copiados. Referências a slides não copiados são descartadas. |
| FTS-0096 | Feito | core/src/operacoes_slides.rs: slides_excluir; gui/src/app.rs | Suporta seleção múltipla; um desfazer; impede documento sem slide. |
| FTS-0097 | Feito | core/src/slides.rs: slide_mover; core/src/operacoes_slides.rs: slides_reordenar | Movimentação para posição escolhida; uma operação no histórico. |
| FTS-0098 | Feito | core/src/operacoes_slides.rs: slides_reordenar; gui/src/app.rs: Miniatura::Soltou | Arraste de seleção múltipla preserva ordem do grupo; seções associadas às posições, como no motor antigo. |
| FTS-0099 | Feito | gui/src/app.rs: clicar_slide; slides_sel | Ctrl alterna item, Shift seleciona intervalo, clique escolhe um. |
| FTS-0100 | Feito | gui/src/app.rs: SlidesSelecionarTodos; miniaturas; menu_contexto.rs | Ctrl+A nas miniaturas ou opção do menu; não confundir com selecionar objetos. |
| FTS-0101 | Feito | core/src/operacoes_slides.rs: slides_alternar_ocultacao; gui/src/app.rs | Aplica a todos os marcados em um único desfazer. |
| FTS-0102 | Feito | core/src/operacoes_slides.rs: slides_alternar_ocultacao; gui/src/app.rs | Exibe slides ocultos selecionados sem apagá-los. |
| FTS-0103 | Feito | gui/src/reutilizar_slides.rs; gui/src/app.rs: ReutilizarArquivoEscolhido/Confirmar | Escolhe páginas de .fsdz/.pptx/.odp, aplica Manter original ou Adaptar tema; importadores externos permanecem parciais. |
| FTS-0104 | Feito | gui/src/arquivos.rs: Importar/ImportarEscolhido; core/src/slides.rs: apresentacao_importar | Importa todos os slides de arquivo; converte ao formato nativo e salva .fsdz. Compatibilidade PPTX/ODP parcial. |
| FTS-0105 | Feito | gui/src/app.rs: SlideExportarPdf/PdfEscolhido/exportar_pdf; gui/src/menu_contexto.rs | Exporta apenas o slide atual em PDF; não oferece exportação PPTX nem outras mídias por slide. |
| FTS-0106 | Feito | gui/src/app.rs: NumeroSlideTexto/Confirmar; gui/src/menu_contexto.rs | Diálogo aceita posição 1..N, valida antes de navegar; não modifica o documento. |
| FTS-0107 | Feito | core/src/slides.rs: slide_ir; gui/src/app.rs: clicar_slide | Navega sem alterar o histórico. |
| FTS-0108 | Feito | core/src/busca.rs; gui/src/app.rs: Pesquisa/PesquisaProximo | Pesquisa textos, conforme limites atuais do índice. |
| FTS-0109 | Feito | core/src/operacoes_slides.rs: slides_restaurar_layout; gui/src/menu_contexto.rs | Reposiciona e reestiliza espaços reservados; mantém seu conteúdo e formas livres. |
| FTS-0110 | Feito | core/src/operacoes_slides.rs: slides_layout; gui/src/app.rs: EscolherLayout | Vários slides selecionados, uma ação de desfazer. |
| FTS-0111 | Feito | core/src/operacoes_slides.rs: EstiloColagem::ManterOrigem; gui/src/menu_contexto.rs | Preserva estilo dos elementos e imagens; adapta escala ao tamanho do documento. |
| FTS-0112 | Feito | core/src/operacoes_slides.rs: EstiloColagem::AdaptarAoTema; core/src/temas.rs | Paleta/fontes/decor do destino; personalizações fora do tema podem permanecer. |
| FTS-0113 | Feito | secoes.rs; app.rs: SecaoNova/SecaoRenomear/SecoesAlternar; miniaturas.rs | Validação visual não realizada neste levantamento. |
| FTS-0114 | Feito | secoes.rs; app.rs: SecaoNova/SecaoRenomear/SecoesAlternar; miniaturas.rs | Validação visual não realizada neste levantamento. |
| FTS-0116 | Feito | secoes.rs; app.rs: SecaoNova/SecaoRenomear/SecoesAlternar; miniaturas.rs | Validação visual não realizada neste levantamento. |
| FTS-0117 | Feito | secoes.rs; app.rs: SecaoNova/SecaoRenomear/SecoesAlternar; miniaturas.rs | Validação visual não realizada neste levantamento. |
| FTS-0118 | Feito | secoes.rs; app.rs: SecaoNova/SecaoRenomear/SecoesAlternar; miniaturas.rs | Validação visual não realizada neste levantamento. |
| FTS-0115 | Feito | core/src/secoes.rs: secao_mover; gui/src/menu_contexto.rs e app.rs | Move a seção inteira entre as adjacentes com ID e ordem interna preservados; desfazer unitário. |
| FTS-0119 | Feito | core/src/secoes.rs: secao_mover, testes | Reordenação por comandos subir/descer; ainda não há arrasto direto do cabeçalho. |
| FTS-0120 | Feito | core/src/secoes.rs: slides_para_secao; menu do cabeçalho; gui/src/app.rs | Transfere slides marcados para o fim da seção alvo, preserva notas, IDs, animações; não clona. |
| FTS-0121 | Feito | core/src/personalizadas.rs; gui/src/personalizadas.rs; io/src/fsdz.rs | Cria roteiro nomeado dos slides marcados, persistido em Arrow v9; máximo 50 roteiros e 1000 slides por roteiro. |
| FTS-0122 | Feito | core/src/personalizadas.rs: personalizada_mover/adicionar/remover_slide/ordem; gui/src/app.rs | Ordenação independente por IDs, reprodução e sequência editável; páginas originais excluídas são ignoradas na reprodução. |
| FTS-0123 | Feito | core/src/personalizadas.rs: personalizada_ocultar/ordem; gui/src/personalizadas.rs | Cada roteiro oculta suas próprias entradas sem esconder slides na apresentação geral; não permite todos ocultos. |
| FTS-0139 | Feito | app.rs: view_editor/view_classificacao/comecar_apresentacao | Validação visual não realizada neste levantamento. |
| FTS-0124 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Presets e operações existentes preservados; aplicar a todos copia também a aparência completa do fundo. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0125 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Presets e operações existentes preservados; aplicar a todos copia também a aparência completa do fundo. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0126 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Dimensões de 36 a 7200 pt na aba Design. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0127 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Presets e operações existentes preservados; aplicar a todos copia também a aparência completa do fundo. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0128 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Troca entre retrato e paisagem no painel. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0129 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Esticar, Ajustar proporcionalmente ou Maximizar; altera todos os slides. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0130 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Escala proporcional para preencher, com possível corte nas bordas. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0131 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Presets e operações existentes preservados; aplicar a todos copia também a aparência completa do fundo. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0132 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Imagem local incorporada, esticada à área do slide. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0133 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Gradiente horizontal entre duas cores. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0134 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Textura pontilhada vetorial, determinística. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0135 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Padrão xadrez vetorial em duas cores. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0136 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Opacidade 0–100% para todos os tipos de fundo. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0137 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Presets e operações existentes preservados; aplicar a todos copia também a aparência completa do fundo. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0138 | Feito | core/src/configuracao_slide.rs; gui/src/configuracao_slide.rs; gui/src/faixa_abas.rs; io/src/fsdz.rs; testes core/IO/GUI | Oculta as formas com Papel::Fundo, sem apagar o mestre/layout. Histórico e persistência cobertos; validação visual no desktop pendente. |
| FTS-0140 | Feito | app.rs: view_editor/view_classificacao/comecar_apresentacao | Validação visual não realizada neste levantamento. |
| FTS-0143 | Pendente | faixa_abas.rs: exibir | Botão explicitamente Em breve. |
| FTS-0144 | Feito | app.rs: view_editor/view_classificacao/comecar_apresentacao | Validação visual não realizada neste levantamento. |
| FTS-0145 | Parcial | app.rs: view_apresentador/abrir_plateia/escolher_monitor_plateia; cronometro.rs; faixa_abas.rs | Acesso pela faixa, janela da plateia, notas, próximo slide, cronômetro e seleção de monitor implementados; validação visual com dois monitores pendente. |
| FTS-0149 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0151 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0152 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0153 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0154 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0155 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0156 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0157 | Parcial | faixa_abas.rs: imagem/formato_icone | Faixa contextual; não há confirmação de painel lateral equivalente a todos os controles do PowerPoint. |
| FTS-0160 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0161 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0162 | Pendente | app.rs: Pesquisa | A pesquisa atual procura texto de slides, não comandos. |
| FTS-0164 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0166 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0167 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0169 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0170 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0171 | Feito | app.rs: view_editor/view_painel_selecao; faixa.rs; painel_animacao.rs; configuracao.rs; palco.rs | Validação visual não realizada neste levantamento. |
| FTS-0172 | Parcial | app.rs: view_apresentador/abrir_plateia/escolher_monitor_plateia; cronometro.rs; faixa_abas.rs | Acesso pela faixa, janela da plateia, notas, próximo slide, cronômetro e seleção de monitor implementados; validação visual com dois monitores pendente. |
| FTS-0174 | Feito | regua.rs; palco.rs; elementos.rs: alinhar_entre_si/distribuir/agrupar; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0175 | Feito | regua.rs; palco.rs; elementos.rs: alinhar_entre_si/distribuir/agrupar; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0176 | Feito | regua.rs; palco.rs; elementos.rs: alinhar_entre_si/distribuir/agrupar; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0178 | Feito | regua.rs; palco.rs; elementos.rs: alinhar_entre_si/distribuir/agrupar; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0182 | Feito | regua.rs; palco.rs; elementos.rs: alinhar_entre_si/distribuir/agrupar; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0184 | Feito | regua.rs; palco.rs; elementos.rs: alinhar_entre_si/distribuir/agrupar; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0185 | Feito | regua.rs; palco.rs; elementos.rs: alinhar_entre_si/distribuir/agrupar; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0189 | Feito | regua.rs; palco.rs; elementos.rs: alinhar_entre_si/distribuir/agrupar; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0190 | Feito | regua.rs; palco.rs; elementos.rs: alinhar_entre_si/distribuir/agrupar; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0193 | Feito | regua.rs; palco.rs; elementos.rs: alinhar_entre_si/distribuir/agrupar; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0194 | Feito | regua.rs; palco.rs; elementos.rs: alinhar_entre_si/distribuir/agrupar; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0195 | Feito | modelo.rs: Tipo/EstiloDoTexto; elementos.rs; layouts.rs; simbolos.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0196 | Feito | modelo.rs: Tipo/EstiloDoTexto; elementos.rs; layouts.rs; simbolos.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0197 | Feito | modelo.rs: Tipo/EstiloDoTexto; elementos.rs; layouts.rs; simbolos.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0198 | Parcial | core/src/wordart.rs; gui/src/texto_artistico.rs; app.rs: exportar_pdf | Texto editável, quatro estilos, deformações, contorno e sombra; PDF usa contornos vetoriais. Sem equivalência PPTX nem validação visual completa. |
| FTS-0200 | Feito | modelo.rs: Tipo/EstiloDoTexto; elementos.rs; layouts.rs; simbolos.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0201 | Feito | modelo.rs: Tipo/EstiloDoTexto; elementos.rs; layouts.rs; simbolos.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0203 | Feito | modelo.rs: Tipo/EstiloDoTexto; elementos.rs; layouts.rs; simbolos.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0206 | Feito | modelo.rs: Tipo/EstiloDoTexto; elementos.rs; layouts.rs; simbolos.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0207 | Feito | modelo.rs: Tipo/EstiloDoTexto; elementos.rs; layouts.rs; simbolos.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0208 | Parcial | faixa_abas.rs: EQUACOES; app.rs: InserirEquacao | Presets em texto Unicode; sem editor matemático estruturado ou OMML. |
| FTS-0209 | Feito | modelo.rs: EstiloDoTexto; elementos.rs: formatar; desenho.rs; tests/sessao.rs | Formatação vale para a caixa inteira; texto rico por trecho continua pendente. |
| FTS-0210 | Feito | modelo.rs: EstiloDoTexto; elementos.rs: formatar; desenho.rs; tests/sessao.rs | Formatação vale para a caixa inteira; texto rico por trecho continua pendente. |
| FTS-0211 | Feito | modelo.rs: EstiloDoTexto; elementos.rs: formatar; desenho.rs; tests/sessao.rs | Formatação vale para a caixa inteira; texto rico por trecho continua pendente. |
| FTS-0212 | Feito | modelo.rs: EstiloDoTexto; elementos.rs: formatar; desenho.rs; tests/sessao.rs | Formatação vale para a caixa inteira; texto rico por trecho continua pendente. |
| FTS-0213 | Feito | modelo.rs: EstiloDoTexto; elementos.rs: formatar; desenho.rs; tests/sessao.rs | Formatação vale para a caixa inteira; texto rico por trecho continua pendente. |
| FTS-0214 | Feito | modelo.rs: EstiloDoTexto; elementos.rs: formatar; desenho.rs; tests/sessao.rs | Formatação vale para a caixa inteira; texto rico por trecho continua pendente. |
| FTS-0215 | Pendente | modelo.rs: EstiloDoTexto | Sem propriedades correspondentes no estilo nativo de edição. |
| FTS-0216 | Feito | modelo.rs: EstiloDoTexto; elementos.rs: formatar; desenho.rs; tests/sessao.rs | Formatação vale para a caixa inteira; texto rico por trecho continua pendente. |
| FTS-0217 | Pendente | modelo.rs: EstiloDoTexto | Sem propriedades correspondentes no estilo nativo de edição. |
| FTS-0218 | Pendente | modelo.rs: EstiloDoTexto | Sem propriedades correspondentes no estilo nativo de edição. |
| FTS-0219 | Pendente | modelo.rs: EstiloDoTexto | Sem propriedades correspondentes no estilo nativo de edição. |
| FTS-0223 | Pendente | modelo.rs: EstiloDoTexto | Sem propriedades correspondentes no estilo nativo de edição. |
| FTS-0224 | Pendente | modelo.rs: EstiloDoTexto | Sem propriedades correspondentes no estilo nativo de edição. |
| FTS-0225 | Feito | modelo.rs: EstiloDoTexto; elementos.rs: formatar; desenho.rs; tests/sessao.rs | Formatação vale para a caixa inteira; texto rico por trecho continua pendente. |
| FTS-0227 | Feito | modelo.rs: EstiloDoTexto; elementos.rs: formatar; desenho.rs; tests/sessao.rs | Formatação vale para a caixa inteira; texto rico por trecho continua pendente. |
| FTS-0230 | Pendente | modelo.rs: EstiloDoTexto | Sem propriedades correspondentes no estilo nativo de edição. |
| FTS-0232 | Feito | modelo.rs: EstiloDoTexto; elementos.rs: formatar; desenho.rs; tests/sessao.rs | Formatação vale para a caixa inteira; texto rico por trecho continua pendente. |
| FTS-0234 | Parcial | modelo.rs: FonteEmbutida; pptx/fontes.rs; fsdz.rs; app.rs | Leitura e armazenamento de fontes incorporadas existem; incorporar uma fonte nova pela interface e exportá-la em PPTX não foi comprovado. |
| FTS-0235 | Feito | modelo.rs: Alinhamento/Vertical/EstiloDoTexto; elementos.rs: lista_alternar; quadro.rs; faixa.rs | Validação visual não realizada neste levantamento. |
| FTS-0236 | Feito | modelo.rs: Alinhamento/Vertical/EstiloDoTexto; elementos.rs: lista_alternar; quadro.rs; faixa.rs | Validação visual não realizada neste levantamento. |
| FTS-0237 | Feito | modelo.rs: Alinhamento/Vertical/EstiloDoTexto; elementos.rs: lista_alternar; quadro.rs; faixa.rs | Validação visual não realizada neste levantamento. |
| FTS-0238 | Feito | modelo.rs: Alinhamento/Vertical/EstiloDoTexto; elementos.rs: lista_alternar; quadro.rs; faixa.rs | Validação visual não realizada neste levantamento. |
| FTS-0245 | Feito | modelo.rs: Alinhamento/Vertical/EstiloDoTexto; elementos.rs: lista_alternar; quadro.rs; faixa.rs | Validação visual não realizada neste levantamento. |
| FTS-0247 | Feito | modelo.rs: Alinhamento/Vertical/EstiloDoTexto; elementos.rs: lista_alternar; quadro.rs; faixa.rs | Validação visual não realizada neste levantamento. |
| FTS-0248 | Feito | modelo.rs: Alinhamento/Vertical/EstiloDoTexto; elementos.rs: lista_alternar; quadro.rs; faixa.rs | Validação visual não realizada neste levantamento. |
| FTS-0252 | Feito | modelo.rs: Alinhamento/Vertical/EstiloDoTexto; elementos.rs: lista_alternar; quadro.rs; faixa.rs | Validação visual não realizada neste levantamento. |
| FTS-0253 | Feito | modelo.rs: Alinhamento/Vertical/EstiloDoTexto; elementos.rs: lista_alternar; quadro.rs; faixa.rs | Validação visual não realizada neste levantamento. |
| FTS-0255 | Feito | modelo.rs: Alinhamento/Vertical/EstiloDoTexto; elementos.rs: lista_alternar; quadro.rs; faixa.rs | Validação visual não realizada neste levantamento. |
| FTS-0261 | Feito | quadro.rs; medida.rs; modelo.rs: EstiloDoTexto; desenho.rs | Quebra de linha usa medida aproximada, sem garantia de layout idêntico ao PowerPoint. |
| FTS-0262 | Feito | quadro.rs; medida.rs; modelo.rs: EstiloDoTexto; desenho.rs | Quebra de linha usa medida aproximada, sem garantia de layout idêntico ao PowerPoint. |
| FTS-0263 | Feito | quadro.rs; medida.rs; modelo.rs: EstiloDoTexto; desenho.rs | Quebra de linha usa medida aproximada, sem garantia de layout idêntico ao PowerPoint. |
| FTS-0265 | Feito | quadro.rs; medida.rs; modelo.rs: EstiloDoTexto; desenho.rs | Quebra de linha usa medida aproximada, sem garantia de layout idêntico ao PowerPoint. |
| FTS-0267 | Feito | busca.rs; app.rs: Pesquisa; tests/sessao.rs: localizar_e_substituir | Validação visual não realizada neste levantamento. |
| FTS-0268 | Feito | busca.rs; app.rs: Pesquisa; tests/sessao.rs: localizar_e_substituir | Validação visual não realizada neste levantamento. |
| FTS-0269 | Feito | busca.rs; app.rs: Pesquisa; tests/sessao.rs: localizar_e_substituir | Validação visual não realizada neste levantamento. |
| FTS-0270 | Feito | gui/src/ferramentas_texto.rs; gui/src/app.rs; gui/src/faixa.rs; gui/src/faixa_abas.rs; ortografia/src/lib.rs; gui/src/ortografia.rs; gui/src/configuracoes.rs | Revisão F7 de caixas de texto, notas e células de tabelas; requer dicionário Hunspell local. Validação visual pendente. |
| FTS-0271 | Feito | gui/src/ferramentas_texto.rs; gui/src/app.rs; gui/src/faixa.rs; gui/src/faixa_abas.rs; ortografia/src/lib.rs; gui/src/ortografia.rs; gui/src/configuracoes.rs | Consulta ortográfica, sugestões, dicionário pessoal e instalação de Hunspell; não é um dicionário de definições. Validação visual pendente. |
| FTS-0272 | Parcial | gui/src/ferramentas_texto.rs; gui/src/app.rs; gui/src/faixa.rs; gui/src/faixa_abas.rs | Vocabulário local inicial em português, editável; aplicação no trecho selecionado com desfazer. Sem léxico amplo instalado. Validação visual pendente. |
| FTS-0273 | Feito | gui/src/ferramentas_texto.rs; gui/src/app.rs; gui/src/faixa.rs; gui/src/faixa_abas.rs | Regras locais editáveis, opcionais; corrige ao terminar palavras na digitação de caixas e células. Validação visual pendente. |
| FTS-0274 | Feito | gui/src/ferramentas_texto.rs; gui/src/app.rs; gui/src/faixa.rs; gui/src/faixa_abas.rs | Opção para capitalizar início de frase durante a digitação; usa regras de pontuação. Validação visual pendente. |
| FTS-0275 | Parcial | gui/src/ferramentas_texto.rs; gui/src/app.rs; gui/src/faixa.rs; gui/src/faixa_abas.rs; ortografia/src/lib.rs; gui/src/ortografia.rs; gui/src/configuracoes.rs | Idioma principal e dicionários locais configuráveis; escolha global, sem idioma por trecho. Validação visual pendente. |
| FTS-0276 | Parcial | gui/src/ferramentas_texto.rs; gui/src/app.rs; gui/src/faixa.rs; gui/src/faixa_abas.rs | Tradução exata por glossário local editável e par de idiomas; sem modelo para frases livres. Validação visual pendente. |
| FTS-0277 | Feito | gui/src/ferramentas_texto.rs; gui/src/app.rs; gui/src/faixa.rs; gui/src/faixa_abas.rs | Palavras e caracteres dos slides, tabelas, rótulos de gráficos e trecho selecionado; notas, fundos e comentários ficam fora desta contagem. Validação visual pendente. |
| FTS-0278 | Feito | busca.rs; app.rs: Pesquisa; tests/sessao.rs: localizar_e_substituir | Validação visual não realizada neste levantamento. |
| FTS-0279 | Feito | gui/src/ferramentas_texto.rs; gui/src/app.rs; gui/src/faixa.rs; gui/src/faixa_abas.rs; core/src/ferramentas_texto.rs; core/src/elementos.rs | Texto simples em caixa nova ou no texto/célula em edição; Ctrl+Shift+V. Validação visual pendente. |
| FTS-0280 | Parcial | gui/src/ferramentas_texto.rs; gui/src/app.rs; gui/src/faixa.rs; gui/src/faixa_abas.rs; core/src/ferramentas_texto.rs; core/src/elementos.rs | Preserva estilo e elementos da cópia interna; formatos ricos de outros programas não são importados. Validação visual pendente. |
| FTS-0281 | Feito | gui/src/ferramentas_texto.rs; gui/src/app.rs; gui/src/faixa.rs; gui/src/faixa_abas.rs; core/src/ferramentas_texto.rs; core/src/elementos.rs | Captura estilo do texto, preenchimento, contorno e sombra; clique no destino aplica sem alterar conteúdo ou geometria; Esc cancela. Validação visual pendente. |
| FTS-0282 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0283 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0284 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0285 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Dimensões iguais garantidas na inserção e no redimensionamento; estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0286 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0287 | Feito | core/src/modelo.rs: Tipo; core/src/formas.rs: pontos/testes; gui/src/galeria_figuras.rs (ADICIONAIS); io/src/fsdz.rs; io/src/pdf.rs | Forma vetorial nativa, selecionável, redimensionável, com estilo; round-trip coberto pelos testes de ADICIONAIS; conferência visual ainda pendente. |
| FTS-0288 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0289 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0290 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0291 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0292 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0293 | Feito | core/src/modelo.rs: Tipo; core/src/formas.rs: pontos/testes; gui/src/galeria_figuras.rs (ADICIONAIS); io/src/fsdz.rs; io/src/pdf.rs | Forma vetorial nativa, selecionável, redimensionável, com estilo; round-trip coberto pelos testes de ADICIONAIS; conferência visual ainda pendente. |
| FTS-0294 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0295 | Feito | core/src/modelo.rs: Tipo; core/src/formas.rs: pontos/testes; gui/src/galeria_figuras.rs (ADICIONAIS); io/src/fsdz.rs; io/src/pdf.rs | Forma vetorial nativa, selecionável, redimensionável, com estilo; round-trip coberto pelos testes de ADICIONAIS; conferência visual ainda pendente. |
| FTS-0296 | Feito | core/src/modelo.rs: Tipo; core/src/formas.rs: pontos/testes; gui/src/galeria_figuras.rs (ADICIONAIS); io/src/fsdz.rs; io/src/pdf.rs | Forma vetorial nativa, selecionável, redimensionável, com estilo; round-trip coberto pelos testes de ADICIONAIS; conferência visual ainda pendente. |
| FTS-0297 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0298 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Contornos compostos com vazios reais; estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0299 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Curva aberta, apenas contorno, sem preenchimento e sem mesclagem booleana; estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0300 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Geometria vetorial por segmentos e contornos; estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0301 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0302 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Geometria vetorial por segmentos e contornos; estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0303 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Geometria vetorial por segmentos e contornos; estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0304 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Contornos compostos com vazios reais; estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0305 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Geometria vetorial por segmentos e contornos; estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0306 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Geometria vetorial por segmentos e contornos; estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0307 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Geometria vetorial por segmentos e contornos; estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0308 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Geometria vetorial por segmentos e contornos; estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0309 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Geometria vetorial por segmentos e contornos; estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0310 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Geometria vetorial por segmentos e contornos; estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0311 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Geometria vetorial por segmentos e contornos; estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0312 | Feito | core/src/novas_formas.rs; core/src/modelo.rs; core/src/quadro.rs; gui/src/galeria_figuras.rs; mcp/src/renderizar.rs; io/src/pdf.rs; io/tests/fsdz.rs; core/tests/sessao.rs | Variações fixas de 4, 5, 6, 8, 10 e 12 pontas (sem ajuste livre das pontas); estilo e transformações de elementos; curvas discretizadas e validação visual manual ainda necessárias. |
| FTS-0313 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs | Validação visual não realizada neste levantamento. |
| FTS-0314 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs | Validação visual não realizada neste levantamento. |
| FTS-0315 | Pendente | modelo.rs: Tipo; formas.rs; galeria_figuras.rs | Variação não implementada como forma na galeria nativa. |
| FTS-0316 | Pendente | modelo.rs: Tipo; formas.rs; galeria_figuras.rs | Variação não implementada como forma na galeria nativa. |
| FTS-0317 | Pendente | modelo.rs: Tipo; formas.rs; galeria_figuras.rs | Variação não implementada como forma na galeria nativa. |
| FTS-0318 | Pendente | modelo.rs: Tipo; formas.rs; galeria_figuras.rs | Variação não implementada como forma na galeria nativa. |
| FTS-0319 | Pendente | modelo.rs: Tipo; formas.rs; galeria_figuras.rs | Variação não implementada como forma na galeria nativa. |
| FTS-0320 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs | Validação visual não realizada neste levantamento. |
| FTS-0321 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs | Validação visual não realizada neste levantamento. |
| FTS-0322 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs | Validação visual não realizada neste levantamento. |
| FTS-0323 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs | Validação visual não realizada neste levantamento. |
| FTS-0324 | Feito | modelo.rs: Tipo; formas.rs; galeria_figuras.rs | Validação visual não realizada neste levantamento. |
| FTS-0325 | Parcial | formas.rs; galeria_figuras.rs | Há setas preenchidas básicas; não a galeria completa de setas de bloco. |
| FTS-0326 | Pendente | modelo.rs: Tipo; formas.rs; galeria_figuras.rs | Variação não implementada como forma na galeria nativa. |
| FTS-0341 | Feito | elementos.rs; palco.rs; faixa_abas.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0342 | Feito | elementos.rs; palco.rs; faixa_abas.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0343 | Feito | elementos.rs; palco.rs; faixa_abas.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0344 | Feito | elementos.rs; palco.rs; faixa_abas.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0345 | Feito | elementos.rs; palco.rs; faixa_abas.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0346 | Feito | elementos.rs; palco.rs; faixa_abas.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0349 | Pendente | modelo.rs: Elemento/Tipo; palco.rs | Não há modelo ou ferramenta para editar nós e segmentos de formas livres. |
| FTS-0350 | Pendente | modelo.rs: Elemento/Tipo; palco.rs | Não há modelo ou ferramenta para editar nós e segmentos de formas livres. |
| FTS-0351 | Pendente | modelo.rs: Elemento/Tipo; palco.rs | Não há modelo ou ferramenta para editar nós e segmentos de formas livres. |
| FTS-0352 | Pendente | modelo.rs: Elemento/Tipo; palco.rs | Não há modelo ou ferramenta para editar nós e segmentos de formas livres. |
| FTS-0353 | Pendente | modelo.rs: Elemento/Tipo; palco.rs | Não há modelo ou ferramenta para editar nós e segmentos de formas livres. |
| FTS-0354 | Pendente | modelo.rs: Elemento/Tipo; palco.rs | Não há modelo ou ferramenta para editar nós e segmentos de formas livres. |
| FTS-0356 | Feito | elementos.rs; palco.rs; faixa_abas.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0357 | Feito | elementos.rs; palco.rs; faixa_abas.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0358 | Feito | elementos.rs; palco.rs; faixa_abas.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0359 | Feito | elementos.rs; palco.rs; faixa_abas.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0361 | Feito | elementos.rs; palco.rs; faixa_abas.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0363 | Feito | core/src/booleanas.rs; modelo.rs: Tipo::Caminho; quadro.rs; gui/src/faixa_abas.rs + app.rs; core/tests (módulo booleanas); io/tests/fsdz.rs | União de formas fechadas via contornos vetoriais; curvas discretizadas; desfazer em um passo; arquivos .fsdz v5. Testes do motor OK; validação visual em desktop pendente. |
| FTS-0364 | Feito | core/src/booleanas.rs; modelo.rs: Tipo::Caminho; quadro.rs; gui/src/faixa_abas.rs + app.rs; core/tests (módulo booleanas); io/tests/fsdz.rs | Interseção de formas fechadas via contornos vetoriais; curvas discretizadas; desfazer em um passo; arquivos .fsdz v5. Testes do motor OK; validação visual em desktop pendente. |
| FTS-0365 | Feito | core/src/booleanas.rs; modelo.rs: Tipo::Caminho; quadro.rs; gui/src/faixa_abas.rs + app.rs; core/tests (módulo booleanas); io/tests/fsdz.rs | Subtração de formas fechadas via contornos vetoriais; curvas discretizadas; desfazer em um passo; arquivos .fsdz v5. Testes do motor OK; validação visual em desktop pendente. |
| FTS-0366 | Feito | core/src/booleanas.rs; modelo.rs: Tipo::Caminho; quadro.rs; gui/src/faixa_abas.rs + app.rs; core/tests (módulo booleanas); io/tests/fsdz.rs | Combinação de formas fechadas via contornos vetoriais; curvas discretizadas; desfazer em um passo; arquivos .fsdz v5. Testes do motor OK; validação visual em desktop pendente. |
| FTS-0367 | Feito | core/src/booleanas.rs; modelo.rs: Tipo::Caminho; quadro.rs; gui/src/faixa_abas.rs + app.rs; core/tests (módulo booleanas); io/tests/fsdz.rs | Fragmentação de formas fechadas via contornos vetoriais; curvas discretizadas; desfazer em um passo; arquivos .fsdz v5. Testes do motor OK; validação visual em desktop pendente. |
| FTS-0368 | Feito | modelo.rs: Cor; seletor_cor.rs; app.rs: cor_no_slide/aplicar_hsva; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0369 | Feito | modelo.rs: Cor; seletor_cor.rs; app.rs: cor_no_slide/aplicar_hsva; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0370 | Feito | modelo.rs: Cor; seletor_cor.rs; app.rs: cor_no_slide/aplicar_hsva; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0371 | Pendente | modelo.rs: Elemento/Cor; desenho.rs | Preenchimento nativo é uma cor; sem modelo destas propriedades. |
| FTS-0372 | Pendente | modelo.rs: Elemento/Cor; desenho.rs | Preenchimento nativo é uma cor; sem modelo destas propriedades. |
| FTS-0373 | Pendente | modelo.rs: Elemento/Cor; desenho.rs | Preenchimento nativo é uma cor; sem modelo destas propriedades. |
| FTS-0374 | Pendente | modelo.rs: Elemento/Cor; desenho.rs | Preenchimento nativo é uma cor; sem modelo destas propriedades. |
| FTS-0375 | Pendente | modelo.rs: Elemento/Cor; desenho.rs | Preenchimento nativo é uma cor; sem modelo destas propriedades. |
| FTS-0377 | Pendente | modelo.rs: Elemento/Cor; desenho.rs | Preenchimento nativo é uma cor; sem modelo destas propriedades. |
| FTS-0378 | Pendente | modelo.rs: Elemento/Cor; desenho.rs | Preenchimento nativo é uma cor; sem modelo destas propriedades. |
| FTS-0380 | Feito | modelo.rs: Cor; seletor_cor.rs; app.rs: cor_no_slide/aplicar_hsva; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0381 | Feito | modelo.rs: Cor; seletor_cor.rs; app.rs: cor_no_slide/aplicar_hsva; desenho.rs | Validação visual não realizada neste levantamento. |
| FTS-0382 | Feito | modelo.rs: Elemento; elementos.rs: formatar; desenho.rs; faixa_abas.rs: formato_icone | Validação visual não realizada neste levantamento. |
| FTS-0383 | Feito | modelo.rs: Elemento; elementos.rs: formatar; desenho.rs; faixa_abas.rs: formato_icone | Validação visual não realizada neste levantamento. |
| FTS-0385 | Feito | modelo.rs: Elemento; elementos.rs: formatar; desenho.rs; faixa_abas.rs: formato_icone | Validação visual não realizada neste levantamento. |
| FTS-0394 | Parcial | modelo.rs: Sombra; desenho.rs | Sombra nativa básica; não sai na exportação PDF e não equivale a todos os presets do PowerPoint. |
| FTS-0406 | Feito | app.rs: InserirImagem/ImagemLida; icones.rs; fsdz.rs; galeria_figuras.rs | Validação visual não realizada neste levantamento. |
| FTS-0407 | Cancelado | Decisão do usuário no inventário | Não implementar. |
| FTS-0408 | Cancelado | Decisão do usuário no inventário | Não implementar. |
| FTS-0412 | Cancelado | Decisão do usuário no inventário | Não implementar. |
| FTS-0413 | Feito | app.rs: InserirImagem/ImagemLida; icones.rs; fsdz.rs; galeria_figuras.rs | Validação visual não realizada neste levantamento. |
| FTS-0414 | Parcial | core/src/icones.rs/svgpath.rs; gui/src/icones.rs | SVG Tabler embutido; isso não comprova importação genérica de arquivos SVG. |
| FTS-0416 | Feito | app.rs: InserirImagem/ImagemLida; icones.rs; fsdz.rs; galeria_figuras.rs | Validação visual não realizada neste levantamento. |
| FTS-0417 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Redimensionamento nativo com alças sem recompressão. Validação visual no Ubuntu pendente. |
| FTS-0418 | Feito | core/src/elementos.rs: imagem_cortar_margem; gui/src/desenho.rs; testes io/tests/fsdz.rs | Corte por quatro margens, arraste e zoom, sem destruir o arquivo fonte. Corte por margens acessível na faixa → Recorte; arraste continua no slide. Validação visual no Ubuntu pendente. |
| FTS-0419 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Recorte para forma com máscara alfa. Validação visual no Ubuntu pendente. |
| FTS-0420 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Proporção da caixa com enquadramento da imagem. Validação visual no Ubuntu pendente. |
| FTS-0421 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Modo Preencher ocupa a caixa cortando excedentes. Validação visual no Ubuntu pendente. |
| FTS-0422 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Modo Ajustar exibe toda a imagem com espaços transparentes. Validação visual no Ubuntu pendente. |
| FTS-0423 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Rotação do elemento de imagem. Validação visual no Ubuntu pendente. |
| FTS-0424 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Espelhamento horizontal e vertical não destrutivo. Validação visual no Ubuntu pendente. |
| FTS-0425 | Parcial | core/src/elementos.rs: imagem_remover_fundo; gui/src/desenho.rs; testes io/tests/fsdz.rs | Cinco modos de remover fundo (cor, borda, LAB, watershed e marcas); sem segmentação semântica IA. Fundo: técnica logo sob Remover/Restaurar e + para tolerância/suavidade; remover semântico por IA permanece parcial. Validação visual no Ubuntu pendente. |
| FTS-0426 | Feito | core/src/elementos.rs: imagem_modo_fundo; gui/src/faixa_abas.rs; gui/src/desenho.rs: modo 5; io/src/fsdz.rs | Cor transparente exata: pixels com RGB idêntico à cor escolhida tornam-se transparentes; a remoção com tolerância permanece disponível. Teste visual no Ubuntu pendente. |
| FTS-0427 | Feito | core/src/elementos.rs: imagem_substituir; gui/src/app.rs; testes io/tests/fsdz.rs | Substituir imagem preservando posição, tamanho, máscara e efeitos. Validação visual no Ubuntu pendente. |
| FTS-0428 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Restaurar enquadramento e redefinir ajustes de foto sem descartar original. Validação visual no Ubuntu pendente. |
| FTS-0429 | Feito | gui/src/imagem_arquivo.rs: recomprimir; gui/src/app.rs; testes io/tests/fsdz.rs | Compactar JPEG qualidade 60/85 ou PNG com alfa; Ctrl+Z recupera a anterior. Validação visual no Ubuntu pendente. |
| FTS-0430 | Feito | gui/src/imagem_arquivo.rs: recomprimir; gui/src/app.rs; testes io/tests/fsdz.rs | Reduzir para até 1280 ou 800 px com Lanczos3. Validação visual no Ubuntu pendente. |
| FTS-0431 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Bytes originais preservados até redução ou compactação voluntária. Validação visual no Ubuntu pendente. |
| FTS-0432 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Opacidade 0..100% na tela e PDF. Opacidade em entrada numérica 0–100% (Enter/✓), sem botões fixos; não altera o comportamento do motor. Validação visual no Ubuntu pendente. |
| FTS-0433 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Brilho RGB -100..100%. Validação visual no Ubuntu pendente. |
| FTS-0434 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Contraste RGB -100..100%. Validação visual no Ubuntu pendente. |
| FTS-0435 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Máscara unsharp de nitidez 0..100%. Validação visual no Ubuntu pendente. |
| FTS-0436 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Saturação 0..200% sobre RGBA. Validação visual no Ubuntu pendente. |
| FTS-0437 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Temperatura fria/quente por balanço de cor. Validação visual no Ubuntu pendente. |
| FTS-0438 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Recolorir por matiz YIQ. Validação visual no Ubuntu pendente. |
| FTS-0439 | Feito | core/src/imagem.rs/elementos.rs; gui/src/imagem_processamento.rs/faixa_abas.rs/desenho.rs; io/src/fsdz.rs; testes io/tests/fsdz.rs | Cinza, sépia, negativo, posterizar e solarizar. Validação visual no Ubuntu pendente. |
| FTS-0440 | Feito | gui/src/faixa_abas.rs: grupo_contorno_formato; core/src/elementos.rs; testes io/tests/fsdz.rs | Borda colorida e espessura na aba Formato da Imagem. Amostra de cor e espessura reunidas no mesmo grupo Contorno, sem divisor. Validação visual no Ubuntu pendente. |
| FTS-0441 | Feito | core/src/elementos.rs: imagem_sombra; gui/src/desenho.rs; gui/src/app.rs: exportar_pdf; testes io/tests/fsdz.rs | Sombra suavizada, deslocada, renderizada no Iced e PDF. Validação visual no Ubuntu pendente. |
| FTS-0442 | Feito | gui/src/imagem_processamento.rs: gerar_reflexo; gui/src/desenho.rs; gui/src/app.rs: exportar_pdf; io/src/fsdz.rs | Reflexo projetado abaixo da imagem com desvanecimento, na tela Iced e no PDF. Teste visual no Ubuntu pendente. |
| FTS-0443 | Feito | gui/src/imagem_processamento.rs: gerar_brilho; gui/src/desenho.rs; gui/src/app.rs: exportar_pdf; io/src/fsdz.rs | Halo suavizado projetado fora do retângulo da foto, na tela Iced e no PDF. Teste visual no Ubuntu pendente. |
| FTS-0444 | Feito | core/src/imagem.rs: perspectiva; gui/src/imagem_processamento.rs: efeitos; io/src/fsdz.rs | Perspectiva trapezoidal 2D com ajuste reversível; não é rotação de câmera 3D. Teste visual no Ubuntu pendente. |
| FTS-0445 | Feito | core/src/elementos.rs: imagem_texto_alternativo; io/src/fsdz.rs; testes io/tests/fsdz.rs | Texto alternativo editável e salvo no .fsdz; PDF sem tags estruturadas. Texto alternativo abre campo inline por ALT +, aplica Enter/✓. Validação visual no Ubuntu pendente. |
| FTS-0446 | Parcial | io/src/svg.rs; gui/src/vetores.rs; faixa_abas.rs: inserir | Diálogo local; paths, formas, grupos/use, viewBox e transformações pelo usvg. Desativa resolução externa. Recusa recursos sem conversão: gradientes, padrões, filtros, máscaras, texto/imagens, opacidade de grupo e traços especiais. |
| FTS-0447 | Parcial | core/src/vetores.rs; gui/src/desenho.rs; io/src/pdf.rs; mcp/src/renderizar.rs | Curvas Bézier nativas, caminhos abertos, alfa e preenchimento nonzero/evenodd em Iced, PDF e MCP. Recursos recusados pelo importador ainda pendentes. Sem validação visual nesta etapa. |
| FTS-0448 | Feito | core/src/vetores.rs; io/tests/svg.rs | Tipo Caminho guarda geometria SVG1 em dados; comandos M/L/Q/C/Z e regra de preenchimento preservados no .fsdz, sem rasterização. Teste de ida/volta após edição. |
| FTS-0449 | Parcial | core/src/vetores.rs: svg_inserir/svg_converter; gui/src/vetores.rs | Importa grupo de peças vetoriais; Converter SVG libera peças individualmente. IDs, seleção e um passo de desfazer/refazer; conversão genérica dos recursos recusados permanece pendente. |
| FTS-0450 | Feito | core/src/vetores.rs: primitivas; elementos.rs: formatar; io/tests/svg.rs | Cores sólidas editadas em Formato da Forma → Preenchimento, por peça ou seleção; persistência e histórico testados. |
| FTS-0451 | Feito | core/src/vetores.rs: primitivas; elementos.rs: formatar; io/tests/svg.rs | Cor e espessura de contorno editáveis na faixa existente; caminhos abertos não são fechados ao traçar. Traços especiais do SVG original ainda não são importáveis. |
| FTS-0452 | Feito | core/src/vetores.rs: decodificar; io/tests/svg.rs | Coordenadas normalizadas; alças/tamanho transformam a caixa sem alterar os dados originais. Teste amplia 100× mantendo curva e número de segmentos. |
| FTS-0453 | Feito | io/src/svg.rs: exportar; gui/src/vetores.rs; io/tests/svg.rs | Exporta vetores selecionados (SVG importado, ícones e caminhos booleanos) como SVG com cores, curvas, contornos, rotações e fill-rule. Usa coordenadas do slide; não exporta todo slide com textos e fotos. Ida/volta SVG e PDF com curvas e evenodd testados. |
| FTS-0454 | Feito | elementos.rs: tabela_inserir/tabela_estrutura; dados.rs; app.rs: editando_celula; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0455 | Feito | elementos.rs: tabela_inserir/tabela_estrutura; dados.rs; app.rs: editando_celula; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0456 | Feito | elementos.rs: tabela_inserir/tabela_estrutura; dados.rs; app.rs: editando_celula; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0457 | Feito | elementos.rs: tabela_inserir/tabela_estrutura; dados.rs; app.rs: editando_celula; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0458 | Feito | elementos.rs: tabela_inserir/tabela_estrutura; dados.rs; app.rs: editando_celula; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0459 | Feito | elementos.rs: tabela_inserir/tabela_estrutura; dados.rs; app.rs: editando_celula; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0460 | Pendente | dados.rs: Grade; elementos.rs: tabela_estrutura | Grade retangular de textos; sem spans ou medidas independentes de linha/coluna. |
| FTS-0461 | Pendente | dados.rs: Grade; elementos.rs: tabela_estrutura | Grade retangular de textos; sem spans ou medidas independentes de linha/coluna. |
| FTS-0462 | Pendente | dados.rs: Grade; elementos.rs: tabela_estrutura | Grade retangular de textos; sem spans ou medidas independentes de linha/coluna. |
| FTS-0463 | Pendente | dados.rs: Grade; elementos.rs: tabela_estrutura | Grade retangular de textos; sem spans ou medidas independentes de linha/coluna. |
| FTS-0466 | Feito | elementos.rs: tabela_inserir/tabela_estrutura; dados.rs; app.rs: editando_celula; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0467 | Feito | core/src/dados.rs: primitivas_tabela; gui/src/faixa_abas.rs: inserir; core/tests/sessao.rs; io/tests/fsdz.rs | Espessura do elemento aplicada às bordas na tela e no PDF; controles de 0 a 50 pt em passos de 0,5 pt. Zero remove o contorno das primitivas. Histórico e persistência testados; validação visual pendente. |
| FTS-0468 | Feito | elementos.rs: tabela_inserir/tabela_estrutura; dados.rs; app.rs: editando_celula; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0471 | Feito | elementos.rs: tabela_inserir/tabela_estrutura; dados.rs; app.rs: editando_celula; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0472 | Feito | elementos.rs: tabela_inserir/tabela_estrutura; dados.rs; app.rs: editando_celula; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0473 | Feito | elementos.rs: tabela_inserir/tabela_estrutura; dados.rs; app.rs: editando_celula; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-0477 | Parcial | dados.rs: primitivas_tabela; modelo.rs: EstiloDoTexto | Aparência básica e estilo do elemento; não estilos individuais completos por célula. |
| FTS-0481 | Parcial | dados.rs: primitivas_tabela; modelo.rs: EstiloDoTexto | Aparência básica e estilo do elemento; não estilos individuais completos por célula. |
| FTS-0492 | Parcial | dados.rs: primitivas_tabela; modelo.rs: EstiloDoTexto | Aparência básica e estilo do elemento; não estilos individuais completos por célula. |
| FTS-0500 | Feito | dados.rs: primitivas_grafico; galeria_figuras.rs; tests/sessao.rs: inserir_tabela_e_grafico | Validação visual não realizada neste levantamento. |
| FTS-0501 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0502 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0503 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0504 | Feito | dados.rs: primitivas_grafico; galeria_figuras.rs; tests/sessao.rs: inserir_tabela_e_grafico | Validação visual não realizada neste levantamento. |
| FTS-0505 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0506 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0507 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0508 | Feito | dados.rs: primitivas_grafico; galeria_figuras.rs; tests/sessao.rs: inserir_tabela_e_grafico | Validação visual não realizada neste levantamento. |
| FTS-0509 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0510 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0511 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0512 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0513 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0514 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0515 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0516 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0517 | Feito | dados.rs: primitivas_grafico; galeria_figuras.rs; tests/sessao.rs: inserir_tabela_e_grafico | Validação visual não realizada neste levantamento. |
| FTS-0518 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0519 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0520 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0521 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0522 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0523 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0524 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0525 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0526 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0527 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0528 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0529 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0530 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0531 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0532 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0533 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0534 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0535 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0536 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0537 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0538 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0539 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0540 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0541 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0542 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0543 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0544 | Pendente | dados.rs: Grafico/primitivas_grafico | Renderizador nativo tem colunas, barras, linhas e pizza simples, sem as variantes listadas. |
| FTS-0545 | Parcial | dados.rs: Grafico; elementos.rs: grafico_definir; dados_janela.rs | Grade/título/tipo básicos; não recursos completos de gráfico ou importação PPTX. |
| FTS-0546 | Parcial | dados.rs: Grafico; elementos.rs: grafico_definir; dados_janela.rs | Grade/título/tipo básicos; não recursos completos de gráfico ou importação PPTX. |
| FTS-0547 | Parcial | dados.rs: Grafico; elementos.rs: grafico_definir; dados_janela.rs | Grade/título/tipo básicos; não recursos completos de gráfico ou importação PPTX. |
| FTS-0548 | Parcial | dados.rs: Grafico; elementos.rs: grafico_definir; dados_janela.rs | Grade/título/tipo básicos; não recursos completos de gráfico ou importação PPTX. |
| FTS-0549 | Parcial | dados_janela.rs | Janela de grade embutida com valores; integração viva com FeatherSheet não comprovada. |
| FTS-0553 | Parcial | dados.rs: Grafico; elementos.rs: grafico_definir; dados_janela.rs | Grade/título/tipo básicos; não recursos completos de gráfico ou importação PPTX. |
| FTS-0561 | Parcial | dados.rs: Grafico; elementos.rs: grafico_definir; dados_janela.rs | Grade/título/tipo básicos; não recursos completos de gráfico ou importação PPTX. |
| FTS-0564 | Parcial | dados.rs: Grafico; elementos.rs: grafico_definir; dados_janela.rs | Grade/título/tipo básicos; não recursos completos de gráfico ou importação PPTX. |
| FTS-0581 | Pendente | dados.rs: Grafico/primitivas_grafico | Sem ajuste ou equações de tendência. |
| FTS-0582 | Pendente | dados.rs: Grafico/primitivas_grafico | Sem ajuste ou equações de tendência. |
| FTS-0583 | Pendente | dados.rs: Grafico/primitivas_grafico | Sem ajuste ou equações de tendência. |
| FTS-0584 | Pendente | dados.rs: Grafico/primitivas_grafico | Sem ajuste ou equações de tendência. |
| FTS-0585 | Pendente | dados.rs: Grafico/primitivas_grafico | Sem ajuste ou equações de tendência. |
| FTS-0586 | Pendente | dados.rs: Grafico/primitivas_grafico | Sem ajuste ou equações de tendência. |
| FTS-0587 | Feito | core/src/diagramas.rs: Categoria::Listas / modelo; gui/src/galeria_figuras.rs: Diagramas; gui/src/faixa_abas.rs: Inserir; io/tests/fsdz.rs | Categoria disponível com modelo vetorial de nós/conectores editáveis, agrupamento e undo; não é SmartArt OOXML. |
| FTS-0588 | Feito | core/src/diagramas.rs: Categoria::Processos / modelo; gui/src/galeria_figuras.rs: Diagramas; gui/src/faixa_abas.rs: Inserir; io/tests/fsdz.rs | Categoria disponível com modelo vetorial de nós/conectores editáveis, agrupamento e undo; não é SmartArt OOXML. |
| FTS-0589 | Feito | core/src/diagramas.rs: Categoria::Ciclos / modelo; gui/src/galeria_figuras.rs: Diagramas; gui/src/faixa_abas.rs: Inserir; io/tests/fsdz.rs | Categoria disponível com modelo vetorial de nós/conectores editáveis, agrupamento e undo; não é SmartArt OOXML. |
| FTS-0590 | Feito | core/src/diagramas.rs: Categoria::Hierarquias / modelo; gui/src/galeria_figuras.rs: Diagramas; gui/src/faixa_abas.rs: Inserir; io/tests/fsdz.rs | Categoria disponível com modelo vetorial de nós/conectores editáveis, agrupamento e undo; não é SmartArt OOXML. |
| FTS-0591 | Feito | core/src/diagramas.rs: Categoria::Organogramas / modelo; gui/src/galeria_figuras.rs: Diagramas; gui/src/faixa_abas.rs: Inserir; io/tests/fsdz.rs | Categoria disponível com modelo vetorial de nós/conectores editáveis, agrupamento e undo; não é SmartArt OOXML. |
| FTS-0592 | Feito | core/src/diagramas.rs: Categoria::Relacoes / modelo; gui/src/galeria_figuras.rs: Diagramas; gui/src/faixa_abas.rs: Inserir; io/tests/fsdz.rs | Categoria disponível com modelo vetorial de nós/conectores editáveis, agrupamento e undo; não é SmartArt OOXML. |
| FTS-0593 | Feito | core/src/diagramas.rs: Categoria::Matrizes / modelo; gui/src/galeria_figuras.rs: Diagramas; gui/src/faixa_abas.rs: Inserir; io/tests/fsdz.rs | Categoria disponível com modelo vetorial de nós/conectores editáveis, agrupamento e undo; não é SmartArt OOXML. |
| FTS-0594 | Feito | core/src/diagramas.rs: Categoria::Piramides / modelo; gui/src/galeria_figuras.rs: Diagramas; gui/src/faixa_abas.rs: Inserir; io/tests/fsdz.rs | Categoria disponível com modelo vetorial de nós/conectores editáveis, agrupamento e undo; não é SmartArt OOXML. |
| FTS-0595 | Parcial | core/src/diagramas.rs: Categoria::Imagens / modelo; gui/src/galeria_figuras.rs: Diagramas; gui/src/faixa_abas.rs: Inserir; io/tests/fsdz.rs | Modelo com molduras vetoriais ilustrativas; imagens reais em nós ainda dependem de FTS-0617. |
| FTS-0596 | Feito | core/src/diagramas.rs: Categoria::Venn / modelo; gui/src/galeria_figuras.rs: Diagramas; gui/src/faixa_abas.rs: Inserir; io/tests/fsdz.rs | Categoria disponível com modelo vetorial de nós/conectores editáveis, agrupamento e undo; não é SmartArt OOXML. |
| FTS-0597 | Feito | core/src/diagramas.rs: Categoria::Radiais / modelo; gui/src/galeria_figuras.rs: Diagramas; gui/src/faixa_abas.rs: Inserir; io/tests/fsdz.rs | Categoria disponível com modelo vetorial de nós/conectores editáveis, agrupamento e undo; não é SmartArt OOXML. |
| FTS-0598 | Feito | core/src/diagramas.rs: Categoria::Engrenagens / modelo; gui/src/galeria_figuras.rs: Diagramas; gui/src/faixa_abas.rs: Inserir; io/tests/fsdz.rs | Categoria disponível com modelo vetorial de nós/conectores editáveis, agrupamento e undo; não é SmartArt OOXML. |
| FTS-0599 | Feito | core/src/diagramas.rs: Categoria::Fluxo / modelo; gui/src/galeria_figuras.rs: Diagramas; gui/src/faixa_abas.rs: Inserir; io/tests/fsdz.rs | Categoria disponível com modelo vetorial de nós/conectores editáveis, agrupamento e undo; não é SmartArt OOXML. |
| FTS-0600 | Feito | core/src/diagramas.rs: Categoria::Comparacao / modelo; gui/src/galeria_figuras.rs: Diagramas; gui/src/faixa_abas.rs: Inserir; io/tests/fsdz.rs | Categoria disponível com modelo vetorial de nós/conectores editáveis, agrupamento e undo; não é SmartArt OOXML. |
| FTS-0601 | Pendente | modelo.rs: Tipo; faixa_abas.rs | Não há modelo ou editor de SmartArt/diagramas. |
| FTS-0602 | Parcial | core/src/editor_diagrama.rs; gui/src/editor_diagrama.rs; app.rs; io/tests/fsdz.rs | Layouts personalizados de lista, processo, ciclo e hierarquia; não converte para todos os layouts SmartArt. Funciona sobre diagramas nativos, sem objetos SmartArt OOXML. Validação visual pendente. |
| FTS-0603 | Pendente | modelo.rs: Tipo; faixa_abas.rs | Não há modelo ou editor de SmartArt/diagramas. |
| FTS-0604 | Pendente | modelo.rs: Tipo; faixa_abas.rs | Não há modelo ou editor de SmartArt/diagramas. |
| FTS-0605 | Feito | core/src/editor_diagrama.rs; gui/src/editor_diagrama.rs; app.rs; io/tests/fsdz.rs | Adicionar item, irmão ou filho no painel; até 64 nós, com redistribuição e conectores vinculados. Funciona sobre diagramas nativos, sem objetos SmartArt OOXML. Validação visual pendente. |
| FTS-0606 | Feito | core/src/editor_diagrama.rs; gui/src/editor_diagrama.rs; app.rs; io/tests/fsdz.rs | Excluir nó no painel; filhos são promovidos ao pai do nó removido. Funciona sobre diagramas nativos, sem objetos SmartArt OOXML. Validação visual pendente. |
| FTS-0607 | Feito | core/src/editor_diagrama.rs; gui/src/editor_diagrama.rs; app.rs; io/tests/fsdz.rs | Promover nó com sua subárvore na hierarquia personalizada. Funciona sobre diagramas nativos, sem objetos SmartArt OOXML. Validação visual pendente. |
| FTS-0608 | Feito | core/src/editor_diagrama.rs; gui/src/editor_diagrama.rs; app.rs; io/tests/fsdz.rs | Rebaixar nó para filho do irmão anterior na hierarquia personalizada. Funciona sobre diagramas nativos, sem objetos SmartArt OOXML. Validação visual pendente. |
| FTS-0609 | Feito | core/src/editor_diagrama.rs; gui/src/editor_diagrama.rs; app.rs; io/tests/fsdz.rs | Mover para cima/baixo entre irmãos, preservando os filhos. Funciona sobre diagramas nativos, sem objetos SmartArt OOXML. Validação visual pendente. |
| FTS-0610 | Feito | core/src/editor_diagrama.rs; gui/src/editor_diagrama.rs; app.rs; io/tests/fsdz.rs | Reordenar subárvores com atualização de layout, relações e desfazer. Funciona sobre diagramas nativos, sem objetos SmartArt OOXML. Validação visual pendente. |
| FTS-0611 | Pendente | modelo.rs: Tipo; faixa_abas.rs | Não há modelo ou editor de SmartArt/diagramas. |
| FTS-0612 | Feito | core/src/editor_diagrama.rs; gui/src/editor_diagrama.rs; app.rs; io/tests/fsdz.rs | Painel de diagrama personalizado com níveis reais; rascunho, adicionar/excluir, promover/rebaixar, ordenar e aplicar em uma transação. Funciona sobre diagramas nativos, sem objetos SmartArt OOXML. Validação visual pendente. |
| FTS-0613 | Pendente | modelo.rs: Tipo; faixa_abas.rs | Não há modelo ou editor de SmartArt/diagramas. |
| FTS-0614 | Pendente | modelo.rs: Tipo; faixa_abas.rs | Não há modelo ou editor de SmartArt/diagramas. |
| FTS-0615 | Pendente | modelo.rs: Tipo; faixa_abas.rs | Não há modelo ou editor de SmartArt/diagramas. |
| FTS-0616 | Pendente | modelo.rs: Tipo; faixa_abas.rs | Não há modelo ou editor de SmartArt/diagramas. |
| FTS-0617 | Pendente | modelo.rs: Tipo; faixa_abas.rs | Não há modelo ou editor de SmartArt/diagramas. |
| FTS-0618 | Pendente | modelo.rs: Tipo; faixa_abas.rs | Não há modelo ou editor de SmartArt/diagramas. |
| FTS-0619 | Pendente | modelo.rs: Tipo; faixa_abas.rs | Não há modelo ou editor de SmartArt/diagramas. |
| FTS-0620 | Pendente | modelo.rs: Tipo; faixa_abas.rs | Não há modelo ou editor de SmartArt/diagramas. |
| FTS-0621 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0622 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0623 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0624 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0625 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0626 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0627 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0628 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0629 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0630 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0631 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0632 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0633 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0634 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0635 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0636 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0637 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0638 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0639 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0640 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0641 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0642 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0643 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0644 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0645 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0646 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0647 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0648 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0649 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0650 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0651 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0652 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0653 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0654 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0655 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Vídeo permanece desabilitado na faixa; não há objeto de mídia no modelo. |
| FTS-0656 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0657 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0658 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0659 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0660 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0661 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0662 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0663 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0664 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0665 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0666 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0667 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0668 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0669 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0670 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0671 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0672 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0673 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0674 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0675 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0676 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0677 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0678 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0679 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0680 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0681 | Pendente | modelo.rs: Tipo; faixa_abas.rs: inserir | Áudio permanece desabilitado; não há reprodução ou narração no modelo. |
| FTS-0682 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0683 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0684 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0685 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0686 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0687 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0688 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0689 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0690 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0691 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0692 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0693 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0694 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0695 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0696 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0697 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0698 | Pendente | modelo.rs: Tipo | O modelo de objetos é 2D; não há recursos 3D. |
| FTS-0699 | Parcial | modelo.rs: TipoTransicao::Nenhuma | Troca direta via Nenhuma; não um efeito Cortar configurável separado. |
| FTS-0700 | Feito | modelo.rs: TipoTransicao; apresentacao.rs; previa.rs | Validação visual não realizada neste levantamento. |
| FTS-0701 | Feito | modelo.rs: TipoTransicao; apresentacao.rs; previa.rs | Validação visual não realizada neste levantamento. |
| FTS-0702 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0703 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0704 | Feito | modelo.rs: TipoTransicao; apresentacao.rs; previa.rs | Validação visual não realizada neste levantamento. |
| FTS-0705 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0706 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0707 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0708 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0709 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0710 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0711 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0712 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0713 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0714 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0715 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0716 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0717 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0718 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0719 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0720 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0721 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0722 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0723 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0724 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0725 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0726 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0727 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0728 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0729 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0730 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0731 | Pendente | modelo.rs: TipoTransicao | Efeito não consta do enum nativo. |
| FTS-0732 | Feito | apresentacao.rs; previa.rs; modelo.rs: TipoTransicao::Zoom | Validação visual não realizada neste levantamento. |
| FTS-0733 | Pendente | modelo.rs: Transicao; apresentacao.rs | Sem Morph ou correspondência/interpolação entre objetos de slides. |
| FTS-0734 | Pendente | modelo.rs: Transicao; apresentacao.rs | Sem Morph ou correspondência/interpolação entre objetos de slides. |
| FTS-0735 | Pendente | modelo.rs: Transicao; apresentacao.rs | Sem Morph ou correspondência/interpolação entre objetos de slides. |
| FTS-0736 | Pendente | modelo.rs: Transicao; apresentacao.rs | Sem Morph ou correspondência/interpolação entre objetos de slides. |
| FTS-0737 | Pendente | modelo.rs: Transicao; apresentacao.rs | Sem Morph ou correspondência/interpolação entre objetos de slides. |
| FTS-0738 | Pendente | modelo.rs: Transicao; apresentacao.rs | Sem Morph ou correspondência/interpolação entre objetos de slides. |
| FTS-0739 | Pendente | modelo.rs: Transicao; apresentacao.rs | Sem Morph ou correspondência/interpolação entre objetos de slides. |
| FTS-0740 | Pendente | modelo.rs: Transicao; apresentacao.rs | Sem Morph ou correspondência/interpolação entre objetos de slides. |
| FTS-0741 | Pendente | modelo.rs: Transicao; apresentacao.rs | Sem Morph ou correspondência/interpolação entre objetos de slides. |
| FTS-0742 | Pendente | modelo.rs: Transicao; apresentacao.rs | Sem Morph ou correspondência/interpolação entre objetos de slides. |
| FTS-0743 | Pendente | modelo.rs: Transicao; apresentacao.rs | Sem Morph ou correspondência/interpolação entre objetos de slides. |
| FTS-0744 | Pendente | modelo.rs: Transicao; apresentacao.rs | Sem Morph ou correspondência/interpolação entre objetos de slides. |
| FTS-0745 | Pendente | modelo.rs: Transicao; apresentacao.rs | Sem Morph ou correspondência/interpolação entre objetos de slides. |
| FTS-0746 | Pendente | modelo.rs: Transicao; apresentacao.rs | Sem Morph ou correspondência/interpolação entre objetos de slides. |
| FTS-0747 | Feito | efeitos.rs: transicao_definir; modelo.rs: Transicao; app.rs; previa.rs; fsdz.rs | Preservação no formato nativo; não leitura/gravação de transições PPTX. |
| FTS-0748 | Feito | efeitos.rs: transicao_definir; modelo.rs: Transicao; app.rs; previa.rs; fsdz.rs | Preservação no formato nativo; não leitura/gravação de transições PPTX. |
| FTS-0749 | Feito | efeitos.rs: transicao_definir; modelo.rs: Transicao; app.rs; previa.rs; fsdz.rs | Preservação no formato nativo; não leitura/gravação de transições PPTX. |
| FTS-0750 | Feito | efeitos.rs: transicao_definir; modelo.rs: Transicao; app.rs; previa.rs; fsdz.rs | Preservação no formato nativo; não leitura/gravação de transições PPTX. |
| FTS-0751 | Feito | efeitos.rs: transicao_definir; modelo.rs: Transicao; app.rs; previa.rs; fsdz.rs | Preservação no formato nativo; não leitura/gravação de transições PPTX. |
| FTS-0752 | Pendente | modelo.rs: Transicao | Não há campo ou recurso de som. |
| FTS-0753 | Feito | efeitos.rs: transicao_definir; modelo.rs: Transicao; app.rs; previa.rs; fsdz.rs | Preservação no formato nativo; não leitura/gravação de transições PPTX. |
| FTS-0754 | Feito | efeitos.rs: transicao_definir; modelo.rs: Transicao; app.rs; previa.rs; fsdz.rs | Preservação no formato nativo; não leitura/gravação de transições PPTX. |
| FTS-0755 | Feito | efeitos.rs: transicao_definir; modelo.rs: Transicao; app.rs; previa.rs; fsdz.rs | Preservação no formato nativo; não leitura/gravação de transições PPTX. |
| FTS-0756 | Feito | efeitos.rs: transicao_definir; modelo.rs: Transicao; app.rs; previa.rs; fsdz.rs | Preservação no formato nativo; não leitura/gravação de transições PPTX. |
| FTS-0757 | Feito | efeitos.rs: transicao_definir; modelo.rs: Transicao; app.rs; previa.rs; fsdz.rs | Preservação no formato nativo; não leitura/gravação de transições PPTX. |
| FTS-0758 | Feito | modelo.rs: TipoAnimacao; desenho.rs; previa.rs | Validação visual não realizada neste levantamento. |
| FTS-0759 | Feito | modelo.rs: TipoAnimacao; desenho.rs; previa.rs | Validação visual não realizada neste levantamento. |
| FTS-0760 | Feito | modelo.rs: TipoAnimacao; desenho.rs: efeito_do_elemento/faixas_visiveis; faixa_abas.rs: Animações; previa.rs; io/src/fsdz.rs | Entrada nativa com reprodução/preview e serialização .fsdz v10. Máscaras dependem de recorte do Canvas Iced; validação visual em imagens e no Ubuntu pendente. |
| FTS-0761 | Feito | modelo.rs: TipoAnimacao; desenho.rs: efeito_do_elemento/faixas_visiveis; faixa_abas.rs: Animações; previa.rs; io/src/fsdz.rs | Entrada nativa com reprodução/preview e serialização .fsdz v10. Máscaras dependem de recorte do Canvas Iced; validação visual em imagens e no Ubuntu pendente. |
| FTS-0762 | Feito | modelo.rs: TipoAnimacao; desenho.rs: efeito_do_elemento/faixas_visiveis; faixa_abas.rs: Animações; previa.rs; io/src/fsdz.rs | Entrada nativa com reprodução/preview e serialização .fsdz v10. Máscaras dependem de recorte do Canvas Iced; validação visual em imagens e no Ubuntu pendente. |
| FTS-0763 | Feito | modelo.rs: TipoAnimacao; desenho.rs: efeito_do_elemento/faixas_visiveis; faixa_abas.rs: Animações; previa.rs; io/src/fsdz.rs | Entrada nativa com reprodução/preview e serialização .fsdz v10. Máscaras dependem de recorte do Canvas Iced; validação visual em imagens e no Ubuntu pendente. |
| FTS-0764 | Feito | modelo.rs: TipoAnimacao; desenho.rs: efeito_do_elemento/faixas_visiveis; faixa_abas.rs: Animações; previa.rs; io/src/fsdz.rs | Entrada nativa com reprodução/preview e serialização .fsdz v10. Máscaras dependem de recorte do Canvas Iced; validação visual em imagens e no Ubuntu pendente. |
| FTS-0765 | Feito | modelo.rs: TipoAnimacao; desenho.rs: efeito_do_elemento/faixas_visiveis; faixa_abas.rs: Animações; previa.rs; io/src/fsdz.rs | Entrada nativa com reprodução/preview e serialização .fsdz v10. Máscaras dependem de recorte do Canvas Iced; validação visual em imagens e no Ubuntu pendente. |
| FTS-0766 | Feito | modelo.rs: TipoAnimacao; desenho.rs: efeito_do_elemento/faixas_visiveis; faixa_abas.rs: Animações; previa.rs; io/src/fsdz.rs | Entrada nativa com reprodução/preview e serialização .fsdz v10. Máscaras dependem de recorte do Canvas Iced; validação visual em imagens e no Ubuntu pendente. |
| FTS-0767 | Feito | modelo.rs: TipoAnimacao; desenho.rs; previa.rs | Validação visual não realizada neste levantamento. |
| FTS-0768 | Feito | modelo.rs: TipoAnimacao; desenho.rs: efeito_do_elemento/faixas_visiveis; faixa_abas.rs: Animações; previa.rs; io/src/fsdz.rs | Entrada nativa com reprodução/preview e serialização .fsdz v10. Máscaras dependem de recorte do Canvas Iced; validação visual em imagens e no Ubuntu pendente. |
| FTS-0769 | Feito | modelo.rs: TipoAnimacao; desenho.rs: efeito_do_elemento/faixas_visiveis; faixa_abas.rs: Animações; previa.rs; io/src/fsdz.rs | Entrada nativa com reprodução/preview e serialização .fsdz v10. Máscaras dependem de recorte do Canvas Iced; validação visual em imagens e no Ubuntu pendente. |
| FTS-0770 | Feito | modelo.rs: TipoAnimacao; desenho.rs: efeito_do_elemento/faixas_visiveis; faixa_abas.rs: Animações; previa.rs; io/src/fsdz.rs | Entrada nativa com reprodução/preview e serialização .fsdz v10. Máscaras dependem de recorte do Canvas Iced; validação visual em imagens e no Ubuntu pendente. |
| FTS-0771 | Feito | modelo.rs: TipoAnimacao; desenho.rs: efeito_do_elemento/faixas_visiveis; faixa_abas.rs: Animações; previa.rs; io/src/fsdz.rs | Entrada nativa com reprodução/preview e serialização .fsdz v10. Máscaras dependem de recorte do Canvas Iced; validação visual em imagens e no Ubuntu pendente. |
| FTS-0772 | Feito | modelo.rs: TipoAnimacao; desenho.rs: efeito_do_elemento/faixas_visiveis; faixa_abas.rs: Animações; previa.rs; io/src/fsdz.rs | Entrada nativa com reprodução/preview e serialização .fsdz v10. Máscaras dependem de recorte do Canvas Iced; validação visual em imagens e no Ubuntu pendente. |
| FTS-0773 | Feito | modelo.rs: TipoAnimacao::Pulsar; desenho.rs; previa.rs | Validação visual não realizada neste levantamento. |
| FTS-0774 | Pendente | modelo.rs: TipoAnimacao | Sem efeito correspondente no modelo de animação. |
| FTS-0775 | Pendente | modelo.rs: TipoAnimacao | Sem efeito correspondente no modelo de animação. |
| FTS-0776 | Pendente | modelo.rs: TipoAnimacao | Sem efeito correspondente no modelo de animação. |
| FTS-0777 | Pendente | modelo.rs: TipoAnimacao | Sem efeito correspondente no modelo de animação. |
| FTS-0778 | Pendente | modelo.rs: TipoAnimacao | Sem efeito correspondente no modelo de animação. |
| FTS-0779 | Pendente | modelo.rs: TipoAnimacao | Sem efeito correspondente no modelo de animação. |
| FTS-0780 | Pendente | modelo.rs: TipoAnimacao | Sem efeito correspondente no modelo de animação. |
| FTS-0781 | Pendente | modelo.rs: TipoAnimacao | Sem efeito correspondente no modelo de animação. |
| FTS-0782 | Pendente | modelo.rs: TipoAnimacao | Sem efeito correspondente no modelo de animação. |
| FTS-0783 | Pendente | modelo.rs: TipoAnimacao | Sem efeito correspondente no modelo de animação. |
| FTS-0784 | Pendente | modelo.rs: TipoAnimacao | Sem efeito correspondente no modelo de animação. |
| FTS-0785 | Pendente | modelo.rs: TipoAnimacao | Sem efeito correspondente no modelo de animação. |
| FTS-0786 | Pendente | modelo.rs: TipoAnimacao | Sem efeito correspondente no modelo de animação. |
| FTS-0787 | Pendente | modelo.rs: TipoAnimacao; desenho.rs: pintar_com_efeitos | Os efeitos atuais são entrada/ênfase; não há categoria de saída. |
| FTS-0788 | Pendente | modelo.rs: TipoAnimacao; desenho.rs: pintar_com_efeitos | Os efeitos atuais são entrada/ênfase; não há categoria de saída. |
| FTS-0789 | Pendente | modelo.rs: TipoAnimacao; desenho.rs: pintar_com_efeitos | Os efeitos atuais são entrada/ênfase; não há categoria de saída. |
| FTS-0790 | Pendente | modelo.rs: TipoAnimacao; desenho.rs: pintar_com_efeitos | Os efeitos atuais são entrada/ênfase; não há categoria de saída. |
| FTS-0791 | Pendente | modelo.rs: TipoAnimacao; desenho.rs: pintar_com_efeitos | Os efeitos atuais são entrada/ênfase; não há categoria de saída. |
| FTS-0792 | Pendente | modelo.rs: TipoAnimacao; desenho.rs: pintar_com_efeitos | Os efeitos atuais são entrada/ênfase; não há categoria de saída. |
| FTS-0793 | Pendente | modelo.rs: TipoAnimacao; desenho.rs: pintar_com_efeitos | Os efeitos atuais são entrada/ênfase; não há categoria de saída. |
| FTS-0794 | Pendente | modelo.rs: TipoAnimacao; desenho.rs: pintar_com_efeitos | Os efeitos atuais são entrada/ênfase; não há categoria de saída. |
| FTS-0795 | Pendente | modelo.rs: TipoAnimacao; desenho.rs: pintar_com_efeitos | Os efeitos atuais são entrada/ênfase; não há categoria de saída. |
| FTS-0796 | Pendente | modelo.rs: Animacao | Sem caminhos ou trajetórias de animação no modelo. |
| FTS-0797 | Pendente | modelo.rs: Animacao | Sem caminhos ou trajetórias de animação no modelo. |
| FTS-0798 | Pendente | modelo.rs: Animacao | Sem caminhos ou trajetórias de animação no modelo. |
| FTS-0799 | Pendente | modelo.rs: Animacao | Sem caminhos ou trajetórias de animação no modelo. |
| FTS-0800 | Pendente | modelo.rs: Animacao | Sem caminhos ou trajetórias de animação no modelo. |
| FTS-0801 | Pendente | modelo.rs: Animacao | Sem caminhos ou trajetórias de animação no modelo. |
| FTS-0802 | Pendente | modelo.rs: Animacao | Sem caminhos ou trajetórias de animação no modelo. |
| FTS-0803 | Pendente | modelo.rs: Animacao | Sem caminhos ou trajetórias de animação no modelo. |
| FTS-0804 | Pendente | modelo.rs: Animacao | Sem caminhos ou trajetórias de animação no modelo. |
| FTS-0805 | Pendente | modelo.rs: Animacao | Sem caminhos ou trajetórias de animação no modelo. |
| FTS-0806 | Pendente | modelo.rs: Animacao | Sem caminhos ou trajetórias de animação no modelo. |
| FTS-0807 | Pendente | modelo.rs: Animacao | Sem caminhos ou trajetórias de animação no modelo. |
| FTS-0808 | Feito | modelo.rs: InicioAnimacao/Animacao; efeitos.rs: passos_de_animacao/animacao_ordem; painel_animacao.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0809 | Feito | modelo.rs: InicioAnimacao/Animacao; efeitos.rs: passos_de_animacao/animacao_ordem; painel_animacao.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0810 | Feito | modelo.rs: InicioAnimacao/Animacao; efeitos.rs: passos_de_animacao/animacao_ordem; painel_animacao.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0811 | Feito | modelo.rs: InicioAnimacao/Animacao; efeitos.rs: passos_de_animacao/animacao_ordem; painel_animacao.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0812 | Feito | modelo.rs: InicioAnimacao/Animacao; efeitos.rs: passos_de_animacao/animacao_ordem; painel_animacao.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0813 | Pendente | modelo.rs: Animacao | Sem propriedades de repetição, texto por trecho ou vínculo de mídia. |
| FTS-0814 | Pendente | modelo.rs: Animacao | Sem propriedades de repetição, texto por trecho ou vínculo de mídia. |
| FTS-0817 | Feito | modelo.rs: InicioAnimacao/Animacao; efeitos.rs: passos_de_animacao/animacao_ordem; painel_animacao.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0818 | Feito | modelo.rs: InicioAnimacao/Animacao; efeitos.rs: passos_de_animacao/animacao_ordem; painel_animacao.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0820 | Feito | modelo.rs: InicioAnimacao/Animacao; efeitos.rs: passos_de_animacao/animacao_ordem; painel_animacao.rs; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0821 | Pendente | modelo.rs: Animacao | Sem propriedades de repetição, texto por trecho ou vínculo de mídia. |
| FTS-0822 | Pendente | modelo.rs: Animacao | Sem propriedades de repetição, texto por trecho ou vínculo de mídia. |
| FTS-0823 | Pendente | modelo.rs: Animacao | Sem propriedades de repetição, texto por trecho ou vínculo de mídia. |
| FTS-0829 | Pendente | modelo.rs: Animacao | Sem propriedades de repetição, texto por trecho ou vínculo de mídia. |
| FTS-0830 | Pendente | modelo.rs: Animacao | Sem propriedades de repetição, texto por trecho ou vínculo de mídia. |
| FTS-0834 | Feito | painel_animacao.rs; efeitos.rs: animacoes_do_slide/animacao_ordem; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0835 | Feito | painel_animacao.rs; efeitos.rs: animacoes_do_slide/animacao_ordem; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0837 | Feito | painel_animacao.rs; efeitos.rs: animacoes_do_slide/animacao_ordem; app.rs | Validação visual não realizada neste levantamento. |
| FTS-0841 | Parcial | app.rs: avancar/efeitos_atuais; previa.rs | Apresentação executa sequência; hover pré-visualiza o efeito selecionado, não uma linha do tempo completa. |
| FTS-0842 | Feito | app.rs: comecar_apresentacao/avancar/efeitos_atuais; apresentacao.rs | Validação visual não realizada neste levantamento. |
| FTS-0843 | Feito | app.rs: comecar_apresentacao/avancar/efeitos_atuais; apresentacao.rs | Validação visual não realizada neste levantamento. |
| FTS-0846 | Feito | app.rs: comecar_apresentacao/avancar/efeitos_atuais; apresentacao.rs | Validação visual não realizada neste levantamento. |
| FTS-0853 | Feito | app.rs: comecar_apresentacao/avancar/efeitos_atuais; apresentacao.rs | Validação visual não realizada neste levantamento. |
| FTS-0855 | Feito | app.rs: proximo_slide/voltar/sair_da_apresentacao; apresentacao.rs: Acao/alvo | Validação visual não realizada neste levantamento. |
| FTS-0856 | Feito | app.rs: proximo_slide/voltar/sair_da_apresentacao; apresentacao.rs: Acao/alvo | Validação visual não realizada neste levantamento. |
| FTS-0861 | Feito | app.rs: proximo_slide/voltar/sair_da_apresentacao; apresentacao.rs: Acao/alvo | Validação visual não realizada neste levantamento. |
| FTS-0862 | Feito | app.rs: proximo_slide/voltar/sair_da_apresentacao; apresentacao.rs: Acao/alvo | Validação visual não realizada neste levantamento. |
| FTS-0863 | Feito | app.rs: proximo_slide/voltar/sair_da_apresentacao; apresentacao.rs: Acao/alvo | Validação visual não realizada neste levantamento. |
| FTS-0864 | Feito | app.rs: proximo_slide/voltar/sair_da_apresentacao; apresentacao.rs: Acao/alvo | Validação visual não realizada neste levantamento. |
| FTS-0870 | Feito | apresentacao.rs: Reproducao::preta/branca; app.rs: teclado | Validação visual não realizada neste levantamento. |
| FTS-0871 | Feito | apresentacao.rs: Reproducao::preta/branca; app.rs: teclado | Validação visual não realizada neste levantamento. |
| FTS-0873 | Parcial | app.rs: view_apresentador/abrir_plateia/escolher_monitor_plateia; cronometro.rs; faixa_abas.rs | Acesso pela faixa, janela da plateia, notas, próximo slide, cronômetro e seleção de monitor implementados; validação visual com dois monitores pendente. |
| FTS-0874 | Parcial | app.rs: view_apresentador/abrir_plateia/escolher_monitor_plateia; cronometro.rs; faixa_abas.rs | Acesso pela faixa, janela da plateia, notas, próximo slide, cronômetro e seleção de monitor implementados; validação visual com dois monitores pendente. |
| FTS-0875 | Parcial | app.rs: view_apresentador/abrir_plateia/escolher_monitor_plateia; cronometro.rs; faixa_abas.rs | Acesso pela faixa, janela da plateia, notas, próximo slide, cronômetro e seleção de monitor implementados; validação visual com dois monitores pendente. |
| FTS-0878 | Parcial | app.rs: view_apresentador/abrir_plateia/escolher_monitor_plateia; cronometro.rs; faixa_abas.rs | Acesso pela faixa, janela da plateia, notas, próximo slide, cronômetro e seleção de monitor implementados; validação visual com dois monitores pendente. |
| FTS-0881 | Feito | links.rs; elementos.rs: gatilho_inserir; apresentacao.rs: Acao::Ir/Link | Validação visual não realizada neste levantamento. |
| FTS-0882 | Feito | links.rs; elementos.rs: gatilho_inserir; apresentacao.rs: Acao::Ir/Link | Validação visual não realizada neste levantamento. |
| FTS-0883 | Feito | links.rs; elementos.rs: gatilho_inserir; apresentacao.rs: Acao::Ir/Link | Validação visual não realizada neste levantamento. |
| FTS-0890 | Pendente | modelo.rs: Animacao/Elemento::destino | destino é navegação para slide; não gatilho de efeito em objeto específico. |
| FTS-0891 | Feito | links.rs; elementos.rs: gatilho_inserir; apresentacao.rs: Acao::Ir/Link | Validação visual não realizada neste levantamento. |
| FTS-0892 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0893 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0894 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0895 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0896 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0897 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0898 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0899 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0900 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0901 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0902 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0903 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0904 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0905 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0906 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0907 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0908 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0909 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0910 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0911 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0912 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0913 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0914 | Pendente | faixa_abas.rs: apresentacao; modelo.rs | Ensaio e gravação desabilitados; não há pipeline de vídeo/câmera. |
| FTS-0915 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0916 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0917 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0918 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0919 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0920 | Parcial | pptx/mod.rs: contexto/formas; modelo.rs: Papel::Fundo | Formas herdadas viram objetos de fundo no slide, sem mestre editável preservado. |
| FTS-0921 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0922 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0923 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0924 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0925 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0926 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0927 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0928 | Parcial | pptx/mod.rs: contexto/formas; modelo.rs: Papel::Fundo | Formas herdadas viram objetos de fundo no slide, sem mestre editável preservado. |
| FTS-0929 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0930 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0931 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0932 | Pendente | modelo.rs: Apresentacao/Slide; layouts.rs | Não existe modelo persistente/editor de mestres; importar herança achatada é parcial. |
| FTS-0937 | Parcial | layouts.rs; slides.rs: slide_layout; modelo.rs: Papel | Layouts predefinidos e placeholders; sem editor de layout personalizado/herança integral. |
| FTS-0938 | Parcial | layouts.rs; slides.rs: slide_layout; modelo.rs: Papel | Layouts predefinidos e placeholders; sem editor de layout personalizado/herança integral. |
| FTS-0940 | Parcial | layouts.rs; slides.rs: slide_layout; modelo.rs: Papel | Layouts predefinidos e placeholders; sem editor de layout personalizado/herança integral. |
| FTS-0941 | Parcial | layouts.rs; slides.rs: slide_layout; modelo.rs: Papel | Layouts predefinidos e placeholders; sem editor de layout personalizado/herança integral. |
| FTS-0953 | Feito | temas.rs; efeitos.rs: tema_aplicar; faixa_abas.rs: design | Validação visual não realizada neste levantamento. |
| FTS-0972 | Parcial | app.rs: InserirNumeroSlide/InserirDataHora | Caixas de texto estáticas; não campos que se atualizam ao reordenar slides ou mudar a data. |
| FTS-0975 | Parcial | app.rs: InserirNumeroSlide/InserirDataHora | Caixas de texto estáticas; não campos que se atualizam ao reordenar slides ou mudar a data. |
| FTS-0986 | Feito | links.rs: comentario_adicionar/comentario_excluir/comentario_resolver; revisao.rs; app.rs; fsdz.rs | Validação visual não realizada neste levantamento. |
| FTS-0988 | Feito | links.rs: comentario_adicionar/comentario_excluir/comentario_resolver; revisao.rs; app.rs; fsdz.rs | Validação visual não realizada neste levantamento. |
| FTS-0991 | Feito | links.rs: comentario_adicionar/comentario_excluir/comentario_resolver; revisao.rs; app.rs; fsdz.rs | Validação visual não realizada neste levantamento. |
| FTS-0992 | Feito | links.rs: comentario_adicionar/comentario_excluir/comentario_resolver; revisao.rs; app.rs; fsdz.rs | Validação visual não realizada neste levantamento. |
| FTS-0995 | Feito | links.rs: comentario_adicionar/comentario_excluir/comentario_resolver; revisao.rs; app.rs; fsdz.rs | Validação visual não realizada neste levantamento. |
| FTS-1005 | Parcial | ortografia/src/lib.rs; configuracoes.rs; gui/src/ortografia.rs | Dicionários no disco; revisão limitada a caixas de texto, não todos os conteúdos. |
| FTS-1006 | Parcial | ortografia/src/lib.rs; configuracoes.rs; gui/src/ortografia.rs | Dicionários no disco; revisão limitada a caixas de texto, não todos os conteúdos. |
| FTS-1019 | Feito | app.rs: salvar_automaticamente; arquivos.rs: salvar_documentos_em_segundo_plano | Automático por tempo real em sessões ativa e suspensas; rascunhos sem nome e falhas preservam o estado de edição. |
| FTS-1042 | Parcial | app.rs: teclado; configuracao.rs; widgets Iced | Atalhos e foco existem; não comprovam leitor de tela ou conformidade completa. |
| FTS-1043 | Parcial | app.rs: teclado; configuracao.rs; widgets Iced | Atalhos e foco existem; não comprovam leitor de tela ou conformidade completa. |
| FTS-1074 | Pendente | modelo.rs: Tipo::Texto; app.rs: InserirEquacao; faixa_abas.rs: EQUACOES | Não há AST matemática/editor estruturado, LaTeX ou OMML. |
| FTS-1075 | Pendente | modelo.rs: Tipo::Texto; app.rs: InserirEquacao; faixa_abas.rs: EQUACOES | Não há AST matemática/editor estruturado, LaTeX ou OMML. |
| FTS-1076 | Parcial | simbolos.rs; faixa_abas.rs: EQUACOES; app.rs: InserirEquacao; desenho.rs | Símbolos/presets Unicode; não composição de equações estruturadas. Alguns caracteres relatados ainda precisam de exemplo e validação visual. |
| FTS-1077 | Parcial | simbolos.rs; faixa_abas.rs: EQUACOES; app.rs: InserirEquacao; desenho.rs | Símbolos/presets Unicode; não composição de equações estruturadas. Alguns caracteres relatados ainda precisam de exemplo e validação visual. |
| FTS-1078 | Parcial | simbolos.rs; faixa_abas.rs: EQUACOES; app.rs: InserirEquacao; desenho.rs | Símbolos/presets Unicode; não composição de equações estruturadas. Alguns caracteres relatados ainda precisam de exemplo e validação visual. |
| FTS-1079 | Parcial | simbolos.rs; faixa_abas.rs: EQUACOES; app.rs: InserirEquacao; desenho.rs | Símbolos/presets Unicode; não composição de equações estruturadas. Alguns caracteres relatados ainda precisam de exemplo e validação visual. |
| FTS-1080 | Parcial | simbolos.rs; faixa_abas.rs: EQUACOES; app.rs: InserirEquacao; desenho.rs | Símbolos/presets Unicode; não composição de equações estruturadas. Alguns caracteres relatados ainda precisam de exemplo e validação visual. |
| FTS-1081 | Pendente | modelo.rs: Tipo::Texto; app.rs: InserirEquacao; faixa_abas.rs: EQUACOES | Não há AST matemática/editor estruturado, LaTeX ou OMML. |
| FTS-1082 | Pendente | modelo.rs: Tipo::Texto; app.rs: InserirEquacao; faixa_abas.rs: EQUACOES | Não há AST matemática/editor estruturado, LaTeX ou OMML. |
| FTS-1083 | Parcial | simbolos.rs; faixa_abas.rs: EQUACOES; app.rs: InserirEquacao; desenho.rs | Símbolos/presets Unicode; não composição de equações estruturadas. Alguns caracteres relatados ainda precisam de exemplo e validação visual. |
| FTS-1084 | Parcial | simbolos.rs; faixa_abas.rs: EQUACOES; app.rs: InserirEquacao; desenho.rs | Símbolos/presets Unicode; não composição de equações estruturadas. Alguns caracteres relatados ainda precisam de exemplo e validação visual. |
| FTS-1085 | Parcial | simbolos.rs; faixa_abas.rs: EQUACOES; app.rs: InserirEquacao; desenho.rs | Símbolos/presets Unicode; não composição de equações estruturadas. Alguns caracteres relatados ainda precisam de exemplo e validação visual. |
| FTS-1086 | Parcial | simbolos.rs; faixa_abas.rs: EQUACOES; app.rs: InserirEquacao; desenho.rs | Símbolos/presets Unicode; não composição de equações estruturadas. Alguns caracteres relatados ainda precisam de exemplo e validação visual. |
| FTS-1087 | Parcial | simbolos.rs; faixa_abas.rs: EQUACOES; app.rs: InserirEquacao; desenho.rs | Símbolos/presets Unicode; não composição de equações estruturadas. Alguns caracteres relatados ainda precisam de exemplo e validação visual. |
| FTS-1088 | Pendente | modelo.rs: Tipo::Texto; app.rs: InserirEquacao; faixa_abas.rs: EQUACOES | Não há AST matemática/editor estruturado, LaTeX ou OMML. |
| FTS-1089 | Parcial | simbolos.rs; faixa_abas.rs: EQUACOES; app.rs: InserirEquacao; desenho.rs | Símbolos/presets Unicode; não composição de equações estruturadas. Alguns caracteres relatados ainda precisam de exemplo e validação visual. |
| FTS-1090 | Parcial | simbolos.rs; faixa_abas.rs: EQUACOES; app.rs: InserirEquacao; desenho.rs | Símbolos/presets Unicode; não composição de equações estruturadas. Alguns caracteres relatados ainda precisam de exemplo e validação visual. |
| FTS-1091 | Pendente | modelo.rs: Tipo::Texto; app.rs: InserirEquacao; faixa_abas.rs: EQUACOES | Não há AST matemática/editor estruturado, LaTeX ou OMML. |
| FTS-1092 | Pendente | modelo.rs: Tipo::Texto; app.rs: InserirEquacao; faixa_abas.rs: EQUACOES | Não há AST matemática/editor estruturado, LaTeX ou OMML. |
| FTS-1093 | Pendente | modelo.rs: Tipo::Texto; app.rs: InserirEquacao; faixa_abas.rs: EQUACOES | Não há AST matemática/editor estruturado, LaTeX ou OMML. |
| FTS-1094 | Pendente | modelo.rs: Tipo::Texto; app.rs: InserirEquacao; faixa_abas.rs: EQUACOES | Não há AST matemática/editor estruturado, LaTeX ou OMML. |
| FTS-1095 | Pendente | modelo.rs: Tipo::Texto; app.rs: InserirEquacao; faixa_abas.rs: EQUACOES | Não há AST matemática/editor estruturado, LaTeX ou OMML. |
| FTS-1096 | Parcial | simbolos.rs; faixa_abas.rs: EQUACOES; app.rs: InserirEquacao; desenho.rs | Símbolos/presets Unicode; não composição de equações estruturadas. Alguns caracteres relatados ainda precisam de exemplo e validação visual. |
| FTS-1097 | Feito | links.rs; revisao.rs; elementos.rs: gatilho_inserir; apresentacao.rs: alvo | Validação visual não realizada neste levantamento. |
| FTS-1099 | Feito | links.rs; revisao.rs; elementos.rs: gatilho_inserir; apresentacao.rs: alvo | Validação visual não realizada neste levantamento. |
| FTS-1100 | Parcial | links.rs; Elemento::destino; apresentacao.rs | Link pertence ao elemento inteiro; navegação de slide usa destino. Sem link por trecho ou sistema genérico de ações. |
| FTS-1103 | Parcial | links.rs; Elemento::destino; apresentacao.rs | Link pertence ao elemento inteiro; navegação de slide usa destino. Sem link por trecho ou sistema genérico de ações. |
| FTS-1104 | Feito | links.rs; revisao.rs; elementos.rs: gatilho_inserir; apresentacao.rs: alvo | Validação visual não realizada neste levantamento. |
| FTS-1105 | Feito | links.rs; revisao.rs; elementos.rs: gatilho_inserir; apresentacao.rs: alvo | Validação visual não realizada neste levantamento. |
| FTS-1106 | Feito | links.rs; revisao.rs; elementos.rs: gatilho_inserir; apresentacao.rs: alvo | Validação visual não realizada neste levantamento. |
| FTS-1107 | Feito | links.rs; revisao.rs; elementos.rs: gatilho_inserir; apresentacao.rs: alvo | Validação visual não realizada neste levantamento. |
| FTS-1109 | Parcial | links.rs; Elemento::destino; apresentacao.rs | Link pertence ao elemento inteiro; navegação de slide usa destino. Sem link por trecho ou sistema genérico de ações. |
| FTS-1112 | Feito | links.rs; revisao.rs; elementos.rs: gatilho_inserir; apresentacao.rs: alvo | Validação visual não realizada neste levantamento. |
| FTS-1118 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1119 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1120 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1121 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1122 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1123 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1124 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1125 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1126 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1127 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1128 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1129 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1130 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1131 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1132 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1133 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1134 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1135 | Pendente | modelo.rs: Elemento; pptx/mod.rs: Leitor::formas | Sem modelo de OLE, embeddings ou vínculos atualizáveis. |
| FTS-1136 | Pendente | modelo.rs: Apresentacao; io/src/lib.rs; mcp/src/ferramentas.rs | Sem projeto ou executor VBA; MCP não executa VBA. |
| FTS-1137 | Pendente | modelo.rs: Apresentacao; io/src/lib.rs; mcp/src/ferramentas.rs | Sem projeto ou executor VBA; MCP não executa VBA. |
| FTS-1138 | Pendente | modelo.rs: Apresentacao; io/src/lib.rs; mcp/src/ferramentas.rs | Sem projeto ou executor VBA; MCP não executa VBA. |
| FTS-1139 | Pendente | modelo.rs: Apresentacao; io/src/lib.rs; mcp/src/ferramentas.rs | Sem projeto ou executor VBA; MCP não executa VBA. |
| FTS-1140 | Pendente | modelo.rs: Apresentacao; io/src/lib.rs; mcp/src/ferramentas.rs | Sem projeto ou executor VBA; MCP não executa VBA. |
| FTS-1141 | Pendente | modelo.rs: Apresentacao; io/src/lib.rs; mcp/src/ferramentas.rs | Sem projeto ou executor VBA; MCP não executa VBA. |
| FTS-1142 | Pendente | modelo.rs: Apresentacao; io/src/lib.rs; mcp/src/ferramentas.rs | Sem projeto ou executor VBA; MCP não executa VBA. |
| FTS-1143 | Pendente | modelo.rs: Apresentacao; io/src/lib.rs; mcp/src/ferramentas.rs | Sem projeto ou executor VBA; MCP não executa VBA. |
| FTS-1144 | Pendente | modelo.rs: Apresentacao; io/src/lib.rs; mcp/src/ferramentas.rs | Sem projeto ou executor VBA; MCP não executa VBA. |
| FTS-1149 | Parcial | mcp/src/ferramentas.rs; ponte/src/lib.rs | Comandos locais existem; não são suplemento compatível com a API oficial do PowerPoint. |
| FTS-1150 | Parcial | mcp/src/ferramentas.rs; ponte/src/lib.rs | Comandos locais existem; não são suplemento compatível com a API oficial do PowerPoint. |
| FTS-1151 | Parcial | mcp/src/ferramentas.rs; ponte/src/lib.rs | Comandos locais existem; não são suplemento compatível com a API oficial do PowerPoint. |
| FTS-1154 | Parcial | mcp/src/ferramentas.rs; ponte/src/lib.rs | Comandos locais existem; não são suplemento compatível com a API oficial do PowerPoint. |
| FTS-1199 | Parcial | ortografia/src/gramatica.rs; app.rs: ondulados_atuais | Regras/listas limitadas de português; não análise gramatical completa por modelo. |
| FTS-1209 | Parcial | mcp/src/ferramentas.rs; core/src/sessao.rs/edicao.rs | Ferramentas para agentes externos e desfazer local; não provedor/modelo de IA embutido ou histórico dedicado por agente. |
| FTS-1210 | Parcial | mcp/src/ferramentas.rs; core/src/sessao.rs/edicao.rs | Ferramentas para agentes externos e desfazer local; não provedor/modelo de IA embutido ou histórico dedicado por agente. |
| FTS-1211 | Parcial | mcp/src/ferramentas.rs; core/src/sessao.rs/edicao.rs | Ferramentas para agentes externos e desfazer local; não provedor/modelo de IA embutido ou histórico dedicado por agente. |
| FTS-1212 | Parcial | mcp/src/ferramentas.rs; core/src/sessao.rs/edicao.rs | Ferramentas para agentes externos e desfazer local; não provedor/modelo de IA embutido ou histórico dedicado por agente. |
| FTS-1218 | Parcial | pptx/mod.rs; pacote.rs; pptx/cores.rs/fontes.rs | Leitura das partes necessárias aos recursos suportados; não preservação integral do pacote. |
| FTS-1219 | Parcial | pptx/mod.rs; pacote.rs; pptx/cores.rs/fontes.rs | Leitura das partes necessárias aos recursos suportados; não preservação integral do pacote. |
| FTS-1220 | Parcial | pptx/mod.rs; pacote.rs; pptx/cores.rs/fontes.rs | Leitura das partes necessárias aos recursos suportados; não preservação integral do pacote. |
| FTS-1221 | Parcial | pptx/mod.rs; pacote.rs; pptx/cores.rs/fontes.rs | Leitura das partes necessárias aos recursos suportados; não preservação integral do pacote. |
| FTS-1222 | Parcial | pptx/mod.rs; pacote.rs; pptx/cores.rs/fontes.rs | Leitura das partes necessárias aos recursos suportados; não preservação integral do pacote. |
| FTS-1223 | Parcial | pptx/mod.rs; pacote.rs; pptx/cores.rs/fontes.rs | Leitura das partes necessárias aos recursos suportados; não preservação integral do pacote. |
| FTS-1224 | Parcial | pptx/mod.rs; pacote.rs; pptx/cores.rs/fontes.rs | Leitura das partes necessárias aos recursos suportados; não preservação integral do pacote. |
| FTS-1225 | Parcial | pptx/mod.rs; pacote.rs; pptx/cores.rs/fontes.rs | Leitura das partes necessárias aos recursos suportados; não preservação integral do pacote. |
| FTS-1228 | Parcial | pptx/mod.rs; pacote.rs; pptx/cores.rs/fontes.rs | Leitura das partes necessárias aos recursos suportados; não preservação integral do pacote. |
| FTS-1237 | Feito | pacote.rs; xml.rs; pptx/mod.rs; tests/importar.rs | Validação visual não realizada neste levantamento. |
| FTS-1238 | Parcial | pacote.rs; xml.rs; pptx/mod.rs; pptx/cores.rs | Parser compara nomes/prefixos literais em vários pontos; herança é convertida ao modelo nativo, sem preservar todas as partes. |
| FTS-1239 | Feito | pacote.rs; xml.rs; pptx/mod.rs; tests/importar.rs | Validação visual não realizada neste levantamento. |
| FTS-1240 | Parcial | pacote.rs; xml.rs; pptx/mod.rs; pptx/cores.rs | Parser compara nomes/prefixos literais em vários pontos; herança é convertida ao modelo nativo, sem preservar todas as partes. |
| FTS-1241 | Feito | pacote.rs; xml.rs; pptx/mod.rs; tests/importar.rs | Validação visual não realizada neste levantamento. |
| FTS-1242 | Feito | pacote.rs; xml.rs; pptx/mod.rs; tests/importar.rs | Validação visual não realizada neste levantamento. |
| FTS-1243 | Parcial | pacote.rs; xml.rs; pptx/mod.rs; pptx/cores.rs | Parser compara nomes/prefixos literais em vários pontos; herança é convertida ao modelo nativo, sem preservar todas as partes. |
| FTS-1244 | Parcial | pacote.rs; xml.rs; pptx/mod.rs; pptx/cores.rs | Parser compara nomes/prefixos literais em vários pontos; herança é convertida ao modelo nativo, sem preservar todas as partes. |
| FTS-1245 | Parcial | pacote.rs; xml.rs; pptx/mod.rs; pptx/cores.rs | Parser compara nomes/prefixos literais em vários pontos; herança é convertida ao modelo nativo, sem preservar todas as partes. |
| FTS-1246 | Feito | pacote.rs; xml.rs; pptx/mod.rs; tests/importar.rs | Validação visual não realizada neste levantamento. |
| FTS-1247 | Parcial | pacote.rs; xml.rs; pptx/mod.rs; pptx/cores.rs | Parser compara nomes/prefixos literais em vários pontos; herança é convertida ao modelo nativo, sem preservar todas as partes. |
| FTS-1248 | Parcial | pacote.rs; xml.rs; pptx/mod.rs; pptx/cores.rs | Parser compara nomes/prefixos literais em vários pontos; herança é convertida ao modelo nativo, sem preservar todas as partes. |
| FTS-1249 | Parcial | pacote.rs; xml.rs; pptx/mod.rs; pptx/cores.rs | Parser compara nomes/prefixos literais em vários pontos; herança é convertida ao modelo nativo, sem preservar todas as partes. |
| FTS-1250 | Feito | pacote.rs; xml.rs; pptx/mod.rs; tests/importar.rs | Validação visual não realizada neste levantamento. |
| FTS-1251 | Pendente | pptx/mod.rs: ler/Leitor::formas; modelo.rs: Apresentacao; io/src/lib.rs | No contexto PPTX: não lê estas partes nem grava/reconstrói o pacote original. |
| FTS-1252 | Pendente | pptx/mod.rs: ler/Leitor::formas; modelo.rs: Apresentacao; io/src/lib.rs | No contexto PPTX: não lê estas partes nem grava/reconstrói o pacote original. |
| FTS-1253 | Pendente | pptx/mod.rs: ler/Leitor::formas; modelo.rs: Apresentacao; io/src/lib.rs | No contexto PPTX: não lê estas partes nem grava/reconstrói o pacote original. |
| FTS-1254 | Pendente | pptx/mod.rs: ler/Leitor::formas; modelo.rs: Apresentacao; io/src/lib.rs | No contexto PPTX: não lê estas partes nem grava/reconstrói o pacote original. |
| FTS-1255 | Pendente | pptx/mod.rs: ler/Leitor::formas; modelo.rs: Apresentacao; io/src/lib.rs | No contexto PPTX: não lê estas partes nem grava/reconstrói o pacote original. |
| FTS-1256 | Pendente | pptx/mod.rs: ler/Leitor::formas; modelo.rs: Apresentacao; io/src/lib.rs | No contexto PPTX: não lê estas partes nem grava/reconstrói o pacote original. |
| FTS-1257 | Feito | pacote.rs; xml.rs; pptx/mod.rs; tests/importar.rs | Validação visual não realizada neste levantamento. |
| FTS-1258 | Pendente | pptx/mod.rs: ler/Leitor::formas; modelo.rs: Apresentacao; io/src/lib.rs | No contexto PPTX: não lê estas partes nem grava/reconstrói o pacote original. |
| FTS-1259 | Pendente | pptx/mod.rs: ler/Leitor::formas; modelo.rs: Apresentacao; io/src/lib.rs | No contexto PPTX: não lê estas partes nem grava/reconstrói o pacote original. |
| FTS-1260 | Pendente | pptx/mod.rs: ler/Leitor::formas; modelo.rs: Apresentacao; io/src/lib.rs | No contexto PPTX: não lê estas partes nem grava/reconstrói o pacote original. |
| FTS-1261 | Pendente | pptx/mod.rs: ler/Leitor::formas; modelo.rs: Apresentacao; io/src/lib.rs | No contexto PPTX: não lê estas partes nem grava/reconstrói o pacote original. |
| FTS-1262 | Pendente | pptx/mod.rs: ler/Leitor::formas; modelo.rs: Apresentacao; io/src/lib.rs | No contexto PPTX: não lê estas partes nem grava/reconstrói o pacote original. |
| FTS-1264 | Pendente | pptx/mod.rs: ler/Leitor::formas; modelo.rs: Apresentacao; io/src/lib.rs | No contexto PPTX: não lê estas partes nem grava/reconstrói o pacote original. |
| FTS-1265 | Pendente | pptx/mod.rs: ler/Leitor::formas; modelo.rs: Apresentacao; io/src/lib.rs | No contexto PPTX: não lê estas partes nem grava/reconstrói o pacote original. |
| FTS-1266 | Pendente | pptx/mod.rs: ler/Leitor::formas; modelo.rs: Apresentacao; io/src/lib.rs | No contexto PPTX: não lê estas partes nem grava/reconstrói o pacote original. |
| FTS-1267 | Parcial | pptx/mod.rs; pptx/texto.rs; pptx/cores.rs | Leitura de subconjunto; grupos achatados, estilo de trecho predominante e efeitos limitados. |
| FTS-1268 | Parcial | pptx/mod.rs; pptx/texto.rs; pptx/cores.rs | Leitura de subconjunto; grupos achatados, estilo de trecho predominante e efeitos limitados. |
| FTS-1269 | Parcial | pptx/mod.rs; pptx/texto.rs; pptx/cores.rs | Leitura de subconjunto; grupos achatados, estilo de trecho predominante e efeitos limitados. |
| FTS-1270 | Parcial | pptx/mod.rs; pptx/texto.rs; pptx/cores.rs | Leitura de subconjunto; grupos achatados, estilo de trecho predominante e efeitos limitados. |
| FTS-1271 | Parcial | pptx/mod.rs; pptx/texto.rs; pptx/cores.rs | Leitura de subconjunto; grupos achatados, estilo de trecho predominante e efeitos limitados. |
| FTS-1272 | Parcial | pptx/mod.rs; pptx/texto.rs; pptx/cores.rs | Leitura de subconjunto; grupos achatados, estilo de trecho predominante e efeitos limitados. |
| FTS-1273 | Pendente | pptx/mod.rs: Leitor::formas; pptx/cores.rs | Não interpretado pelo importador atual. |
| FTS-1274 | Parcial | pptx/mod.rs; pptx/texto.rs; pptx/cores.rs | Leitura de subconjunto; grupos achatados, estilo de trecho predominante e efeitos limitados. |
| FTS-1275 | Parcial | pptx/mod.rs; pptx/texto.rs; pptx/cores.rs | Leitura de subconjunto; grupos achatados, estilo de trecho predominante e efeitos limitados. |
| FTS-1276 | Pendente | pptx/mod.rs: Leitor::formas; pptx/cores.rs | Não interpretado pelo importador atual. |
| FTS-1277 | Pendente | pptx/mod.rs: Leitor::formas; pptx/cores.rs | Não interpretado pelo importador atual. |
| FTS-1278 | Pendente | pptx/mod.rs: Leitor::formas; pptx/cores.rs | Não interpretado pelo importador atual. |
| FTS-1279 | Parcial | pptx/mod.rs; pptx/texto.rs; pptx/cores.rs | Leitura de subconjunto; grupos achatados, estilo de trecho predominante e efeitos limitados. |
| FTS-1280 | Parcial | pptx/mod.rs; pptx/texto.rs; pptx/cores.rs | Leitura de subconjunto; grupos achatados, estilo de trecho predominante e efeitos limitados. |
| FTS-1281 | Parcial | pptx/mod.rs; pptx/texto.rs; pptx/cores.rs | Leitura de subconjunto; grupos achatados, estilo de trecho predominante e efeitos limitados. |
| FTS-1282 | Parcial | pptx/mod.rs; pptx/texto.rs; pptx/cores.rs | Leitura de subconjunto; grupos achatados, estilo de trecho predominante e efeitos limitados. |
| FTS-1283 | Pendente | pptx/mod.rs: Leitor::formas; pptx/cores.rs | Não interpretado pelo importador atual. |
| FTS-1284 | Parcial | pptx/mod.rs; pptx/texto.rs; pptx/cores.rs | Leitura de subconjunto; grupos achatados, estilo de trecho predominante e efeitos limitados. |
| FTS-1285 | Parcial | pptx/mod.rs; pptx/texto.rs; pptx/cores.rs | Leitura de subconjunto; grupos achatados, estilo de trecho predominante e efeitos limitados. |
| FTS-1286 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1287 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1288 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1289 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1291 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1292 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1293 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1294 | Pendente | pptx/mod.rs: tipo_da_forma/Leitor::forma; modelo.rs: Tipo | Sem importação de geometria personalizada e destas propriedades; SVG de ícones é outro caminho. |
| FTS-1295 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1296 | Pendente | pptx/mod.rs: tipo_da_forma/Leitor::forma; modelo.rs: Tipo | Sem importação de geometria personalizada e destas propriedades; SVG de ícones é outro caminho. |
| FTS-1297 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1298 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1299 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1300 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1301 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1302 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1304 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1305 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1308 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1309 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1310 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1311 | Pendente | pptx/mod.rs: tipo_da_forma/Leitor::forma; modelo.rs: Tipo | Sem importação de geometria personalizada e destas propriedades; SVG de ícones é outro caminho. |
| FTS-1312 | Pendente | pptx/mod.rs: tipo_da_forma/Leitor::forma; modelo.rs: Tipo | Sem importação de geometria personalizada e destas propriedades; SVG de ícones é outro caminho. |
| FTS-1313 | Parcial | pptx/mod.rs: Transformacao/Leitor::forma; pptx/texto.rs/cores.rs; quadro.rs | Geometria suportada é limitada, grupos achatados, dimensões normalizadas a 16:9/4:3 e texto com estilo predominante/métricas aproximadas. |
| FTS-1314 | Pendente | pptx/mod.rs: tipo_da_forma/Leitor::forma; modelo.rs: Tipo | Sem importação de geometria personalizada e destas propriedades; SVG de ícones é outro caminho. |
| FTS-1315 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1316 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1317 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1318 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1319 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1320 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1321 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1322 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1323 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1324 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1325 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1326 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1327 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1328 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1329 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1330 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1331 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1332 | Pendente | pptx/mod.rs: Leitor::slide; modelo.rs: Transicao/Animacao | O importador não lê p:timing/p:transition; efeitos nativos não comprovam compatibilidade PPTX. |
| FTS-1333 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1334 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1335 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1336 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1337 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1338 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1339 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1340 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1341 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1342 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1343 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1344 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1345 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1346 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1347 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1348 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1349 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1350 | Pendente | pptx/mod.rs: Leitor::formas | p:graphicFrame é ignorado; gráfico nativo não equivale a importar gráfico PPTX. |
| FTS-1351 | Pendente | pptx/mod.rs: Leitor::formas; modelo.rs: Tipo | Não lê partes de diagramas nem preserva seu modelo editável. |
| FTS-1352 | Pendente | pptx/mod.rs: Leitor::formas; modelo.rs: Tipo | Não lê partes de diagramas nem preserva seu modelo editável. |
| FTS-1353 | Pendente | pptx/mod.rs: Leitor::formas; modelo.rs: Tipo | Não lê partes de diagramas nem preserva seu modelo editável. |
| FTS-1354 | Pendente | pptx/mod.rs: Leitor::formas; modelo.rs: Tipo | Não lê partes de diagramas nem preserva seu modelo editável. |
| FTS-1355 | Pendente | pptx/mod.rs: Leitor::formas; modelo.rs: Tipo | Não lê partes de diagramas nem preserva seu modelo editável. |
| FTS-1356 | Pendente | pptx/mod.rs: Leitor::formas; modelo.rs: Tipo | Não lê partes de diagramas nem preserva seu modelo editável. |
| FTS-1357 | Pendente | pptx/mod.rs: Leitor::formas; modelo.rs: Tipo | Não lê partes de diagramas nem preserva seu modelo editável. |
| FTS-1358 | Pendente | pptx/mod.rs: Leitor::formas; modelo.rs: Tipo | Não lê partes de diagramas nem preserva seu modelo editável. |
| FTS-1359 | Pendente | pptx/mod.rs: Leitor::formas; modelo.rs: Tipo | Não lê partes de diagramas nem preserva seu modelo editável. |
| FTS-1360 | Pendente | pptx/mod.rs: Leitor::formas; modelo.rs: Tipo | Não lê partes de diagramas nem preserva seu modelo editável. |
| FTS-1361 | Pendente | pptx/mod.rs: Leitor::formas; modelo.rs: Tipo | Não lê partes de diagramas nem preserva seu modelo editável. |
| FTS-1362 | Pendente | pptx/mod.rs: Leitor::formas; modelo.rs: Tipo | Não lê partes de diagramas nem preserva seu modelo editável. |
| FTS-1363 | Pendente | pptx/mod.rs: Leitor::formas; modelo.rs: Tipo | Não lê partes de diagramas nem preserva seu modelo editável. |
| FTS-1365 | Parcial | pptx/fontes.rs; app.rs; desenho.rs: shaping; medida.rs/quadro.rs | Fontes carregadas/shaping avançado para alguns símbolos; métricas e layout do motor continuam aproximados, sem garantia entre sistemas. |
| FTS-1367 | Parcial | pptx/fontes.rs; app.rs; desenho.rs: shaping; medida.rs/quadro.rs | Fontes carregadas/shaping avançado para alguns símbolos; métricas e layout do motor continuam aproximados, sem garantia entre sistemas. |
| FTS-1368 | Parcial | pptx/fontes.rs; app.rs; desenho.rs: shaping; medida.rs/quadro.rs | Fontes carregadas/shaping avançado para alguns símbolos; métricas e layout do motor continuam aproximados, sem garantia entre sistemas. |
| FTS-1369 | Parcial | pptx/fontes.rs; app.rs; desenho.rs: shaping; medida.rs/quadro.rs | Fontes carregadas/shaping avançado para alguns símbolos; métricas e layout do motor continuam aproximados, sem garantia entre sistemas. |
| FTS-1371 | Parcial | pptx/fontes.rs; app.rs; desenho.rs: shaping; medida.rs/quadro.rs | Fontes carregadas/shaping avançado para alguns símbolos; métricas e layout do motor continuam aproximados, sem garantia entre sistemas. |
| FTS-1372 | Parcial | pptx/fontes.rs; app.rs; desenho.rs: shaping; medida.rs/quadro.rs | Fontes carregadas/shaping avançado para alguns símbolos; métricas e layout do motor continuam aproximados, sem garantia entre sistemas. |
| FTS-1383 | Parcial | pptx/fontes.rs; app.rs; desenho.rs: shaping; medida.rs/quadro.rs | Fontes carregadas/shaping avançado para alguns símbolos; métricas e layout do motor continuam aproximados, sem garantia entre sistemas. |
| FTS-1384 | Parcial | pptx/fontes.rs; app.rs; desenho.rs: shaping; medida.rs/quadro.rs | Fontes carregadas/shaping avançado para alguns símbolos; métricas e layout do motor continuam aproximados, sem garantia entre sistemas. |
| FTS-1385 | Parcial | pptx/fontes.rs; app.rs; desenho.rs: shaping; medida.rs/quadro.rs | Fontes carregadas/shaping avançado para alguns símbolos; métricas e layout do motor continuam aproximados, sem garantia entre sistemas. |
| FTS-1386 | Pendente | medida.rs; modelo.rs: EstiloDoTexto; desenho.rs | Não há fidelidade certificada nem modelo de texto em caminho. |
| FTS-1387 | Pendente | medida.rs; modelo.rs: EstiloDoTexto; desenho.rs | Não há fidelidade certificada nem modelo de texto em caminho. |
| FTS-1388 | Parcial | pptx/mod.rs: ler; odp.rs: ler; tests/importar.rs | Importadores de subconjuntos; sem ida e volta sem perda. |
| FTS-1389 | Pendente | app.rs: abrir_dialogo_abrir/AbrirCaminho; io/src/lib.rs; modelo.rs: Tipo | Fluxo de abrir filtra fsdz/pptx/odp; não há importadores dedicados dos demais formatos. |
| FTS-1390 | Pendente | app.rs: abrir_dialogo_abrir/AbrirCaminho; io/src/lib.rs; modelo.rs: Tipo | Fluxo de abrir filtra fsdz/pptx/odp; não há importadores dedicados dos demais formatos. |
| FTS-1391 | Pendente | app.rs: abrir_dialogo_abrir/AbrirCaminho; io/src/lib.rs; modelo.rs: Tipo | Fluxo de abrir filtra fsdz/pptx/odp; não há importadores dedicados dos demais formatos. |
| FTS-1392 | Parcial | pptx/mod.rs: ler; odp.rs: ler; tests/importar.rs | Importadores de subconjuntos; sem ida e volta sem perda. |
| FTS-1393 | Pendente | app.rs: abrir_dialogo_abrir/AbrirCaminho; io/src/lib.rs; modelo.rs: Tipo | Fluxo de abrir filtra fsdz/pptx/odp; não há importadores dedicados dos demais formatos. |
| FTS-1394 | Pendente | app.rs: abrir_dialogo_abrir/AbrirCaminho; io/src/lib.rs; modelo.rs: Tipo | Fluxo de abrir filtra fsdz/pptx/odp; não há importadores dedicados dos demais formatos. |
| FTS-1396 | Feito | app.rs: InserirImagem/ImagemLida; desenho.rs: decodificar | Validação visual não realizada neste levantamento. |
| FTS-1397 | Feito | app.rs: InserirImagem/ImagemLida; desenho.rs: decodificar | Validação visual não realizada neste levantamento. |
| FTS-1403 | Pendente | app.rs: abrir_dialogo_abrir/AbrirCaminho; io/src/lib.rs; modelo.rs: Tipo | Fluxo de abrir filtra fsdz/pptx/odp; não há importadores dedicados dos demais formatos. |
| FTS-1404 | Pendente | app.rs: abrir_dialogo_abrir/AbrirCaminho; io/src/lib.rs; modelo.rs: Tipo | Fluxo de abrir filtra fsdz/pptx/odp; não há importadores dedicados dos demais formatos. |
| FTS-1406 | Pendente | app.rs: abrir_dialogo_abrir/AbrirCaminho; io/src/lib.rs; modelo.rs: Tipo | Fluxo de abrir filtra fsdz/pptx/odp; não há importadores dedicados dos demais formatos. |
| FTS-1407 | Pendente | io/src/lib.rs; app.rs: exportar_pdf; mcp/src/renderizar.rs | Saída de apresentação disponível é fsdz/PDF; PNG existe no renderizador MCP, não estes exportadores. |
| FTS-1408 | Pendente | io/src/lib.rs; app.rs: exportar_pdf; mcp/src/renderizar.rs | Saída de apresentação disponível é fsdz/PDF; PNG existe no renderizador MCP, não estes exportadores. |
| FTS-1409 | Pendente | io/src/lib.rs; app.rs: exportar_pdf; mcp/src/renderizar.rs | Saída de apresentação disponível é fsdz/PDF; PNG existe no renderizador MCP, não estes exportadores. |
| FTS-1410 | Pendente | io/src/lib.rs; app.rs: exportar_pdf; mcp/src/renderizar.rs | Saída de apresentação disponível é fsdz/PDF; PNG existe no renderizador MCP, não estes exportadores. |
| FTS-1411 | Parcial | io/src/pdf.rs; app.rs: exportar_pdf | Texto WinAnsi/Helvetica, sem incorporação de fontes próprias e sem sombras. |
| FTS-1412 | Parcial | mcp/src/renderizar.rs; app.rs: gerar_visao_slide | Renderização PNG via MCP/miniatura; não interface completa de exportação de imagens por slide. |
| FTS-1413 | Pendente | io/src/lib.rs; app.rs: exportar_pdf; mcp/src/renderizar.rs | Saída de apresentação disponível é fsdz/PDF; PNG existe no renderizador MCP, não estes exportadores. |
| FTS-1414 | Pendente | io/src/lib.rs; app.rs: exportar_pdf; mcp/src/renderizar.rs | Saída de apresentação disponível é fsdz/PDF; PNG existe no renderizador MCP, não estes exportadores. |
| FTS-1415 | Pendente | io/src/lib.rs; app.rs: exportar_pdf; mcp/src/renderizar.rs | Saída de apresentação disponível é fsdz/PDF; PNG existe no renderizador MCP, não estes exportadores. |
| FTS-1416 | Pendente | io/src/lib.rs; app.rs: exportar_pdf; mcp/src/renderizar.rs | Saída de apresentação disponível é fsdz/PDF; PNG existe no renderizador MCP, não estes exportadores. |
| FTS-1417 | Pendente | io/src/lib.rs; app.rs: exportar_pdf; mcp/src/renderizar.rs | Saída de apresentação disponível é fsdz/PDF; PNG existe no renderizador MCP, não estes exportadores. |
| FTS-1418 | Parcial | mcp/src/renderizar.rs; app.rs: gerar_visao_slide | Renderização PNG via MCP/miniatura; não interface completa de exportação de imagens por slide. |
| FTS-1419 | Pendente | io/src/lib.rs; app.rs: exportar_pdf; mcp/src/renderizar.rs | Saída de apresentação disponível é fsdz/PDF; PNG existe no renderizador MCP, não estes exportadores. |
| FTS-1420 | Pendente | io/src/lib.rs; app.rs: exportar_pdf; mcp/src/renderizar.rs | Saída de apresentação disponível é fsdz/PDF; PNG existe no renderizador MCP, não estes exportadores. |
| FTS-1422 | Feito | fsdz.rs: gravar; pdf.rs: exportar; app.rs | Validação visual não realizada neste levantamento. |
| FTS-1436 | Feito | modelo.rs; sessao.rs; fsdz.rs; tests/fsdz.rs | Validação visual não realizada neste levantamento. |
| FTS-1437 | Feito | modelo.rs; sessao.rs; fsdz.rs; tests/fsdz.rs | Validação visual não realizada neste levantamento. |
| FTS-1438 | Feito | modelo.rs; sessao.rs; fsdz.rs; tests/fsdz.rs | Validação visual não realizada neste levantamento. |
| FTS-1439 | Feito | modelo.rs; sessao.rs; fsdz.rs; tests/fsdz.rs | Validação visual não realizada neste levantamento. |
| FTS-1440 | Parcial | modelo.rs: grupo/Papel/destino; fsdz.rs; pptx/mod.rs | Grupos planos e leitura compatível das versões fsdz; sem árvore completa de mestres/grupos aninhados ou migrações arbitrárias. |
| FTS-1441 | Parcial | modelo.rs: grupo/Papel/destino; fsdz.rs; pptx/mod.rs | Grupos planos e leitura compatível das versões fsdz; sem árvore completa de mestres/grupos aninhados ou migrações arbitrárias. |
| FTS-1443 | Parcial | modelo.rs: grupo/Papel/destino; fsdz.rs; pptx/mod.rs | Grupos planos e leitura compatível das versões fsdz; sem árvore completa de mestres/grupos aninhados ou migrações arbitrárias. |
| FTS-1446 | Feito | modelo.rs; sessao.rs; fsdz.rs; tests/fsdz.rs | Validação visual não realizada neste levantamento. |
| FTS-1447 | Feito | modelo.rs; sessao.rs; fsdz.rs; tests/fsdz.rs | Validação visual não realizada neste levantamento. |
| FTS-1448 | Parcial | modelo.rs: grupo/Papel/destino; fsdz.rs; pptx/mod.rs | Grupos planos e leitura compatível das versões fsdz; sem árvore completa de mestres/grupos aninhados ou migrações arbitrárias. |
| FTS-1449 | Feito | edicao.rs; sessao.rs: editar; elementos.rs: formatar_previa; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1450 | Feito | edicao.rs; sessao.rs: editar; elementos.rs: formatar_previa; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1451 | Feito | edicao.rs; sessao.rs: editar; elementos.rs: formatar_previa; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1452 | Feito | edicao.rs; sessao.rs: editar; elementos.rs: formatar_previa; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1453 | Feito | edicao.rs; sessao.rs: editar; elementos.rs: formatar_previa; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1455 | Feito | edicao.rs; sessao.rs: editar; elementos.rs: formatar_previa; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1457 | Feito | edicao.rs; sessao.rs: editar; elementos.rs: formatar_previa; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1458 | Feito | edicao.rs; sessao.rs: editar; elementos.rs: formatar_previa; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1460 | Feito | io/src/arquivo.rs: gravar_atomico; io/src/fsdz.rs: salvar; gui/src/app.rs; mcp/src/ferramentas.rs | GUI e MCP compartilham temporário exclusivo, sincronização dos dados e renomeação no mesmo diretório. Falhas removem o temporário e não marcam o documento como salvo. Sem garantia de persistência dos metadados do diretório após queda de energia. |
| FTS-1462 | Feito | elementos.rs; palco.rs; app.rs: slides_sel/editando_celula | Validação visual não realizada neste levantamento. |
| FTS-1463 | Feito | elementos.rs; palco.rs; app.rs: slides_sel/editando_celula | Validação visual não realizada neste levantamento. |
| FTS-1464 | Feito | elementos.rs; palco.rs; app.rs: slides_sel/editando_celula | Validação visual não realizada neste levantamento. |
| FTS-1465 | Pendente | cursor.rs; modelo.rs: Elemento; palco.rs | Sem seleção de nós vetoriais/árvore de objetos; seleção de texto tem implementação própria. |
| FTS-1467 | Pendente | cursor.rs; modelo.rs: Elemento; palco.rs | Sem seleção de nós vetoriais/árvore de objetos; seleção de texto tem implementação própria. |
| FTS-1468 | Feito | core/src/cursor.rs: intervalo/substituir; gui/src/app.rs: ancora_edicao/atalho_de_texto; palco.rs: TextoArrastou | Seleção por Shift/arraste e área de transferência no texto; formatação ainda vale para a caixa inteira. |
| FTS-1469 | Feito | elementos.rs; palco.rs; app.rs: slides_sel/editando_celula | Validação visual não realizada neste levantamento. |
| FTS-1470 | Feito | elementos.rs; palco.rs; app.rs: slides_sel/editando_celula | Validação visual não realizada neste levantamento. |
| FTS-1472 | Feito | elementos.rs; escala.rs; palco.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1473 | Feito | elementos.rs; escala.rs; palco.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1474 | Feito | elementos.rs; escala.rs; palco.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1476 | Feito | elementos.rs; escala.rs; palco.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1477 | Feito | elementos.rs; escala.rs; palco.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1478 | Feito | elementos.rs; escala.rs; palco.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1479 | Feito | elementos.rs; escala.rs; palco.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1482 | Feito | elementos.rs; escala.rs; palco.rs; tests/sessao.rs | Validação visual não realizada neste levantamento. |
| FTS-1484 | Feito | desenho.rs; svgpath.rs; dados.rs; palco.rs; Cargo.toml | Bézier/curvas via caminhos dos ícones, não editor de pontos; texto e gráficos têm as limitações descritas nas áreas próprias. |
| FTS-1485 | Feito | desenho.rs; svgpath.rs; dados.rs; palco.rs; Cargo.toml | Bézier/curvas via caminhos dos ícones, não editor de pontos; texto e gráficos têm as limitações descritas nas áreas próprias. |
| FTS-1486 | Feito | desenho.rs; svgpath.rs; dados.rs; palco.rs; Cargo.toml | Bézier/curvas via caminhos dos ícones, não editor de pontos; texto e gráficos têm as limitações descritas nas áreas próprias. |
| FTS-1487 | Feito | desenho.rs; svgpath.rs; dados.rs; palco.rs; Cargo.toml | Bézier/curvas via caminhos dos ícones, não editor de pontos; texto e gráficos têm as limitações descritas nas áreas próprias. |
| FTS-1488 | Feito | desenho.rs; svgpath.rs; dados.rs; palco.rs; Cargo.toml | Bézier/curvas via caminhos dos ícones, não editor de pontos; texto e gráficos têm as limitações descritas nas áreas próprias. |
| FTS-1489 | Pendente | modelo.rs: Tipo/Elemento; desenho.rs | Sem representação ou renderização nativa destas propriedades. |
| FTS-1490 | Pendente | modelo.rs: Tipo/Elemento; desenho.rs | Sem representação ou renderização nativa destas propriedades. |
| FTS-1491 | Feito | desenho.rs; svgpath.rs; dados.rs; palco.rs; Cargo.toml | Bézier/curvas via caminhos dos ícones, não editor de pontos; texto e gráficos têm as limitações descritas nas áreas próprias. |
| FTS-1492 | Parcial | desenho.rs; modelo.rs: EfeitosImagem/Sombra; apresentacao.rs | Sombra básica e recorte/máscara de imagens; não todas as operações DrawingML. |
| FTS-1493 | Parcial | desenho.rs; modelo.rs: EfeitosImagem/Sombra; apresentacao.rs | Sombra básica e recorte/máscara de imagens; não todas as operações DrawingML. |
| FTS-1494 | Parcial | desenho.rs; modelo.rs: EfeitosImagem/Sombra; apresentacao.rs | Sombra básica e recorte/máscara de imagens; não todas as operações DrawingML. |
| FTS-1495 | Pendente | modelo.rs: Tipo/Elemento; desenho.rs | Sem representação ou renderização nativa destas propriedades. |
| FTS-1496 | Pendente | modelo.rs: Tipo/Elemento; desenho.rs | Sem representação ou renderização nativa destas propriedades. |
| FTS-1498 | Feito | desenho.rs; svgpath.rs; dados.rs; palco.rs; Cargo.toml | Bézier/curvas via caminhos dos ícones, não editor de pontos; texto e gráficos têm as limitações descritas nas áreas próprias. |
| FTS-1499 | Feito | desenho.rs; svgpath.rs; dados.rs; palco.rs; Cargo.toml | Bézier/curvas via caminhos dos ícones, não editor de pontos; texto e gráficos têm as limitações descritas nas áreas próprias. |
| FTS-1500 | Feito | desenho.rs; svgpath.rs; dados.rs; palco.rs; Cargo.toml | Bézier/curvas via caminhos dos ícones, não editor de pontos; texto e gráficos têm as limitações descritas nas áreas próprias. |
| FTS-1501 | Feito | desenho.rs; svgpath.rs; dados.rs; palco.rs; Cargo.toml | Bézier/curvas via caminhos dos ícones, não editor de pontos; texto e gráficos têm as limitações descritas nas áreas próprias. |
| FTS-1502 | Feito | desenho.rs; svgpath.rs; dados.rs; palco.rs; Cargo.toml | Bézier/curvas via caminhos dos ícones, não editor de pontos; texto e gráficos têm as limitações descritas nas áreas próprias. |
| FTS-1503 | Feito | desenho.rs; svgpath.rs; dados.rs; palco.rs; Cargo.toml | Bézier/curvas via caminhos dos ícones, não editor de pontos; texto e gráficos têm as limitações descritas nas áreas próprias. |
| FTS-1504 | Feito | desenho.rs; svgpath.rs; dados.rs; palco.rs; Cargo.toml | Bézier/curvas via caminhos dos ícones, não editor de pontos; texto e gráficos têm as limitações descritas nas áreas próprias. |
| FTS-1505 | Pendente | modelo.rs: Tipo/Elemento; desenho.rs | Sem representação ou renderização nativa destas propriedades. |
| FTS-1506 | Feito | desenho.rs; apresentacao.rs; previa.rs; mcp/src/renderizar.rs; pdf.rs | Validação visual não realizada neste levantamento. |
| FTS-1507 | Feito | desenho.rs; apresentacao.rs; previa.rs; mcp/src/renderizar.rs; pdf.rs | Validação visual não realizada neste levantamento. |
| FTS-1508 | Feito | desenho.rs; apresentacao.rs; previa.rs; mcp/src/renderizar.rs; pdf.rs | Validação visual não realizada neste levantamento. |
| FTS-1509 | Feito | desenho.rs; apresentacao.rs; previa.rs; mcp/src/renderizar.rs; pdf.rs | Validação visual não realizada neste levantamento. |
| FTS-1510 | Feito | desenho.rs; apresentacao.rs; previa.rs; mcp/src/renderizar.rs; pdf.rs | Validação visual não realizada neste levantamento. |
| FTS-1513 | Feito | desenho.rs; apresentacao.rs; previa.rs; mcp/src/renderizar.rs; pdf.rs | Validação visual não realizada neste levantamento. |
| FTS-1514 | Pendente | io/src/lib.rs; modelo.rs: Tipo; apresentacao.rs | Sem exportador de vídeo ou mecanismo de mídia. |
| FTS-1516 | Pendente | io/src/lib.rs; modelo.rs: Tipo; apresentacao.rs | Sem exportador de vídeo ou mecanismo de mídia. |
| FTS-1517 | Pendente | io/src/lib.rs; modelo.rs: Tipo; apresentacao.rs | Sem exportador de vídeo ou mecanismo de mídia. |
| FTS-1521 | Feito | app.rs: quadro_em_cache; miniaturas.rs: canvas::Cache; iced::Task | Validação visual não realizada neste levantamento. |
| FTS-1526 | Feito | app.rs: quadro_em_cache; miniaturas.rs: canvas::Cache; iced::Task | Validação visual não realizada neste levantamento. |
| FTS-1530 | Parcial | imprimir.rs; app.rs: tratar_impressao; pdf.rs; app.rs: teste ctrl_p_abre_a_caixa_de_impressao_e_limita_as_copias | Fluxo/PDF testados; impressão física não validada. CUPS no Linux; outros sistemas têm limitações. |
| FTS-1531 | Pendente | imprimir.rs; app.rs: exportar_pdf/tratar_impressao | Caixa oferece todos/atual e PDF; sem intervalos, notas, folhetos ou prévia dedicada. |
| FTS-1532 | Parcial | imprimir.rs; app.rs: tratar_impressao; pdf.rs; app.rs: teste ctrl_p_abre_a_caixa_de_impressao_e_limita_as_copias | Fluxo/PDF testados; impressão física não validada. CUPS no Linux; outros sistemas têm limitações. |
| FTS-1533 | Pendente | imprimir.rs; app.rs: exportar_pdf/tratar_impressao | Caixa oferece todos/atual e PDF; sem intervalos, notas, folhetos ou prévia dedicada. |
| FTS-1534 | Pendente | imprimir.rs; app.rs: exportar_pdf/tratar_impressao | Caixa oferece todos/atual e PDF; sem intervalos, notas, folhetos ou prévia dedicada. |
| FTS-1535 | Pendente | imprimir.rs; app.rs: exportar_pdf/tratar_impressao | Caixa oferece todos/atual e PDF; sem intervalos, notas, folhetos ou prévia dedicada. |
| FTS-1536 | Pendente | imprimir.rs; app.rs: exportar_pdf/tratar_impressao | Caixa oferece todos/atual e PDF; sem intervalos, notas, folhetos ou prévia dedicada. |
| FTS-1537 | Parcial | imprimir.rs; app.rs: tratar_impressao; pdf.rs; app.rs: teste ctrl_p_abre_a_caixa_de_impressao_e_limita_as_copias | Fluxo/PDF testados; impressão física não validada. CUPS no Linux; outros sistemas têm limitações. |
| FTS-1538 | Pendente | imprimir.rs; app.rs: exportar_pdf/tratar_impressao | Caixa oferece todos/atual e PDF; sem intervalos, notas, folhetos ou prévia dedicada. |
| FTS-1542 | Parcial | imprimir.rs; app.rs: tratar_impressao; pdf.rs; app.rs: teste ctrl_p_abre_a_caixa_de_impressao_e_limita_as_copias | Fluxo/PDF testados; impressão física não validada. CUPS no Linux; outros sistemas têm limitações. |
| FTS-1549 | Pendente | imprimir.rs; app.rs: exportar_pdf/tratar_impressao | Caixa oferece todos/atual e PDF; sem intervalos, notas, folhetos ou prévia dedicada. |
| FTS-1550 | Parcial | imprimir.rs; app.rs: tratar_impressao; pdf.rs; app.rs: teste ctrl_p_abre_a_caixa_de_impressao_e_limita_as_copias | Fluxo/PDF testados; impressão física não validada. CUPS no Linux; outros sistemas têm limitações. |
| FTS-1551 | Pendente | configuracao.rs; configuracoes.rs | Sem estas opções/infraestrutura na interface nativa atual. |
| FTS-1552 | Parcial | configuracao.rs: idiomas_ortografia; configuracoes.rs | Escolha/instalação de dicionários; não traduz a interface. |
| FTS-1553 | Feito | configuracao.rs; configuracoes.rs; app.rs: tratar_config | Validação visual não realizada neste levantamento. |
| FTS-1554 | Feito | configuracao.rs; configuracoes.rs; app.rs: tratar_config | Validação visual não realizada neste levantamento. |
| FTS-1559 | Feito | configuracao.rs; configuracoes.rs; app.rs: tratar_config | Validação visual não realizada neste levantamento. |
| FTS-1568 | Feito | configuracao.rs; configuracoes.rs; app.rs: tratar_config | Validação visual não realizada neste levantamento. |
| FTS-1571 | Pendente | configuracao.rs; configuracoes.rs | Sem estas opções/infraestrutura na interface nativa atual. |
| FTS-1572 | Pendente | configuracao.rs; configuracoes.rs | Sem estas opções/infraestrutura na interface nativa atual. |
| FTS-1574 | Pendente | gui/src/*.rs; configuracao.rs: ATALHOS | Interface nativa tem textos pt-BR; dicionário en_US não é tradução da interface. |
| FTS-1575 | Feito | gui/src/*.rs; desenho.rs; modelo.rs | Validação visual não realizada neste levantamento. |
| FTS-1576 | Pendente | gui/src/*.rs; configuracao.rs: ATALHOS | Interface nativa tem textos pt-BR; dicionário en_US não é tradução da interface. |
| FTS-1577 | Pendente | gui/src/*.rs; configuracao.rs: ATALHOS | Interface nativa tem textos pt-BR; dicionário en_US não é tradução da interface. |
| FTS-1578 | Pendente | gui/src/*.rs; configuracao.rs: ATALHOS | Interface nativa tem textos pt-BR; dicionário en_US não é tradução da interface. |
| FTS-1582 | Feito | gui/src/*.rs; desenho.rs; modelo.rs | Validação visual não realizada neste levantamento. |
| FTS-1584 | Parcial | ortografia/src/lib.rs; configuracao.rs: idiomas_ortografia | Dicionários encontrados no disco; instalação e cobertura variam por sistema. |
| FTS-1589 | Pendente | gui/src/*.rs; configuracao.rs: ATALHOS | Interface nativa tem textos pt-BR; dicionário en_US não é tradução da interface. |
| FTS-1590 | Feito | core/tests/sessao.rs; io/tests/importar.rs/fsdz.rs; testes internos dos módulos | Validação visual não realizada neste levantamento. |
| FTS-1591 | Feito | core/tests/sessao.rs; io/tests/importar.rs/fsdz.rs; testes internos dos módulos | Validação visual não realizada neste levantamento. |
| FTS-1592 | Feito | core/tests/sessao.rs; io/tests/importar.rs/fsdz.rs; testes internos dos módulos | Validação visual não realizada neste levantamento. |
| FTS-1593 | Feito | core/tests/sessao.rs; io/tests/importar.rs/fsdz.rs; testes internos dos módulos | Validação visual não realizada neste levantamento. |
| FTS-1594 | Feito | core/tests/sessao.rs; io/tests/importar.rs/fsdz.rs; testes internos dos módulos | Validação visual não realizada neste levantamento. |
| FTS-1595 | Feito | core/tests/sessao.rs; io/tests/importar.rs/fsdz.rs; testes internos dos módulos | Validação visual não realizada neste levantamento. |
| FTS-1596 | Feito | core/tests/sessao.rs; io/tests/importar.rs/fsdz.rs; testes internos dos módulos | Validação visual não realizada neste levantamento. |
| FTS-1597 | Feito | core/tests/sessao.rs; io/tests/importar.rs/fsdz.rs; testes internos dos módulos | Validação visual não realizada neste levantamento. |
| FTS-1598 | Feito | core/tests/sessao.rs; io/tests/importar.rs/fsdz.rs; testes internos dos módulos | Validação visual não realizada neste levantamento. |
| FTS-1599 | Feito | core/tests/sessao.rs; io/tests/importar.rs/fsdz.rs; testes internos dos módulos | Validação visual não realizada neste levantamento. |
| FTS-1600 | Feito | core/tests/sessao.rs; io/tests/importar.rs/fsdz.rs; testes internos dos módulos | Validação visual não realizada neste levantamento. |
| FTS-1601 | Feito | core/tests/sessao.rs; io/tests/importar.rs/fsdz.rs; testes internos dos módulos | Validação visual não realizada neste levantamento. |
| FTS-1603 | Feito | io/tests/importar.rs; io/tests/pdf.rs | Validação visual não realizada neste levantamento. |
| FTS-1604 | Pendente | io/src/lib.rs; io/tests/ | Não existem exportadores para estes formatos. |
| FTS-1605 | Pendente | io/src/lib.rs; io/tests/ | Não existem exportadores para estes formatos. |
| FTS-1608 | Feito | io/tests/importar.rs; io/tests/pdf.rs | Validação visual não realizada neste levantamento. |
| FTS-1610 | Pendente | io/src/lib.rs; io/tests/ | Não existem exportadores para estes formatos. |
| FTS-1614 | A verificar | Validação visual | Testes automatizados existentes não substituem comparação de screenshots com PowerPoint. |
| FTS-1615 | A verificar | Validação visual | Testes automatizados existentes não substituem comparação de screenshots com PowerPoint. |
| FTS-1616 | A verificar | Validação visual | Testes automatizados existentes não substituem comparação de screenshots com PowerPoint. |
| FTS-1617 | A verificar | Validação visual | Testes automatizados existentes não substituem comparação de screenshots com PowerPoint. |
| FTS-1618 | A verificar | Validação visual | Testes automatizados existentes não substituem comparação de screenshots com PowerPoint. |
| FTS-1619 | A verificar | Validação visual | Testes automatizados existentes não substituem comparação de screenshots com PowerPoint. |
| FTS-1620 | A verificar | Validação visual | Testes automatizados existentes não substituem comparação de screenshots com PowerPoint. |
| FTS-1621 | A verificar | Validação visual | Testes automatizados existentes não substituem comparação de screenshots com PowerPoint. |
| FTS-1622 | A verificar | Validação visual | Testes automatizados existentes não substituem comparação de screenshots com PowerPoint. |
| FTS-1623 | A verificar | Validação visual | Testes automatizados existentes não substituem comparação de screenshots com PowerPoint. |
| FTS-1624 | A verificar | Validação visual | Testes automatizados existentes não substituem comparação de screenshots com PowerPoint. |
| FTS-1640 | Feito | core/src/modelo.rs; io/src/pptx/mod.rs; crates/*/Cargo.toml; gui/src/main.rs; docs/ | Validação visual não realizada neste levantamento. |
| FTS-1641 | Feito | core/src/modelo.rs; io/src/pptx/mod.rs; crates/*/Cargo.toml; gui/src/main.rs; docs/ | Validação visual não realizada neste levantamento. |
| FTS-1642 | Feito | core/src/modelo.rs; io/src/pptx/mod.rs; crates/*/Cargo.toml; gui/src/main.rs; docs/ | Validação visual não realizada neste levantamento. |
| FTS-1651 | Feito | core/src/modelo.rs; io/src/pptx/mod.rs; crates/*/Cargo.toml; gui/src/main.rs; docs/ | Validação visual não realizada neste levantamento. |
| FTS-1654 | Feito | core/src/modelo.rs; io/src/pptx/mod.rs; crates/*/Cargo.toml; gui/src/main.rs; docs/ | Validação visual não realizada neste levantamento. |
| FTS-1655 | A verificar | Proposta de organização | Crates atuais usam manifestos independentes; workspace é candidato, não requisito de suporte. |
| FTS-1656 | Feito | gui/Cargo.toml; io/Cargo.toml; core/Cargo.toml | Indica dependência adotada, não conclusão de todas as capacidades dessa biblioteca. |
| FTS-1657 | A verificar | Avaliação de dependências | Lista de candidatos; não instalar nem substituir bibliotecas automaticamente. |
| FTS-1658 | Feito | gui/Cargo.toml; io/Cargo.toml; core/Cargo.toml | Indica dependência adotada, não conclusão de todas as capacidades dessa biblioteca. |
| FTS-1659 | Feito | gui/Cargo.toml; io/Cargo.toml; core/Cargo.toml | Indica dependência adotada, não conclusão de todas as capacidades dessa biblioteca. |
| FTS-1660 | Feito | gui/Cargo.toml; io/Cargo.toml; core/Cargo.toml | Indica dependência adotada, não conclusão de todas as capacidades dessa biblioteca. |
| FTS-1661 | Feito | gui/Cargo.toml; io/Cargo.toml; core/Cargo.toml | Indica dependência adotada, não conclusão de todas as capacidades dessa biblioteca. |
| FTS-1662 | A verificar | Avaliação de dependências | Lista de candidatos; não instalar nem substituir bibliotecas automaticamente. |
| FTS-1663 | A verificar | Avaliação de dependências | Lista de candidatos; não instalar nem substituir bibliotecas automaticamente. |
| FTS-1664 | A verificar | Avaliação de dependências | Lista de candidatos; não instalar nem substituir bibliotecas automaticamente. |
| FTS-1665 | A verificar | Avaliação de dependências | Lista de candidatos; não instalar nem substituir bibliotecas automaticamente. |
| FTS-1666 | A verificar | Avaliação de dependências | Lista de candidatos; não instalar nem substituir bibliotecas automaticamente. |
| FTS-1667 | Feito | gui/Cargo.toml; io/Cargo.toml; core/Cargo.toml | Indica dependência adotada, não conclusão de todas as capacidades dessa biblioteca. |
| FTS-1668 | A verificar | Avaliação de dependências | Lista de candidatos; não instalar nem substituir bibliotecas automaticamente. |
| FTS-1669 | A verificar | Avaliação de dependências | Lista de candidatos; não instalar nem substituir bibliotecas automaticamente. |
| FTS-1670 | A verificar | Avaliação de dependências | Lista de candidatos; não instalar nem substituir bibliotecas automaticamente. |
| FTS-1671 | Parcial | Inventário e código do FeatherSlide | Fases contêm recursos feitos e pendentes; não inferir fase concluída por um subconjunto. |
| FTS-1672 | Parcial | Inventário e código do FeatherSlide | Fases contêm recursos feitos e pendentes; não inferir fase concluída por um subconjunto. |
| FTS-1673 | Parcial | Inventário e código do FeatherSlide | Fases contêm recursos feitos e pendentes; não inferir fase concluída por um subconjunto. |
| FTS-1674 | Parcial | Inventário e código do FeatherSlide | Fases contêm recursos feitos e pendentes; não inferir fase concluída por um subconjunto. |
| FTS-1675 | Parcial | Inventário e código do FeatherSlide | Fases contêm recursos feitos e pendentes; não inferir fase concluída por um subconjunto. |
| FTS-1676 | Feito | core/src/modelo.rs; io/src/pptx/mod.rs | Validação visual não realizada neste levantamento. |
| FTS-1677 | Pendente | modelo.rs: Apresentacao; pptx/mod.rs: ler/Leitor::formas | Sem armazenamento das partes desconhecidas do pacote original. |
| FTS-1678 | Referência | Links fornecidos pelo usuário | Material de consulta, não tarefa de implementação ou alegação de suporte. |
| FTS-1679 | Referência | Links fornecidos pelo usuário | Material de consulta, não tarefa de implementação ou alegação de suporte. |
| FTS-1680 | Referência | Links fornecidos pelo usuário | Material de consulta, não tarefa de implementação ou alegação de suporte. |
| FTS-1681 | Referência | Links fornecidos pelo usuário | Material de consulta, não tarefa de implementação ou alegação de suporte. |
| FTS-1682 | Referência | Links fornecidos pelo usuário | Material de consulta, não tarefa de implementação ou alegação de suporte. |
| FTS-1683 | Referência | Links fornecidos pelo usuário | Material de consulta, não tarefa de implementação ou alegação de suporte. |
| FTS-1684 | Referência | Links fornecidos pelo usuário | Material de consulta, não tarefa de implementação ou alegação de suporte. |

## Arquivos para conferir

- [Modelo do documento](../crates/featherslide-core/src/modelo.rs)
- [Operações sobre elementos](../crates/featherslide-core/src/elementos.rs)
- [Teste do motor](../crates/featherslide-core/tests/sessao.rs)
- [Importador PPTX](../crates/featherslide-io/src/pptx/mod.rs)
- [Pacote e relacionamentos](../crates/featherslide-io/src/pacote.rs)
- [Testes de importação](../crates/featherslide-io/tests/importar.rs)
- [Arquivo nativo](../crates/featherslide-io/src/fsdz.rs)
- [Exportador PDF](../crates/featherslide-io/src/pdf.rs)
- [Aplicativo Iced](../crates/featherslide-gui/src/app.rs)
- [Faixa e botões ainda desabilitados](../crates/featherslide-gui/src/faixa_abas.rs)
- [Prévia dos efeitos](../crates/featherslide-gui/src/previa.rs)
- [Apresentação](../crates/featherslide-gui/src/apresentacao.rs)
- [Configurações](../crates/featherslide-gui/src/configuracao.rs)
- [Impressão](../crates/featherslide-gui/src/imprimir.rs)
- [Regras de gramática](../crates/featherslide-ortografia/src/gramatica.rs)

## Próximas verificações

Conferir os registros A verificar por área, registrar testes específicos e
validar os fluxos visuais na aplicação. Para compatibilidade, usar arquivos
PPTX reais com fontes, grupos, tabelas, gráficos e efeitos, medindo perdas
separadamente da edição nativa. Não marcar ausência ou conclusão apenas
pela presença de um botão ou de uma dependência.

## Continuação — WordArt e apresentador

- FTS-0876 e FTS-0877: **Feito**. Pausar/retomar e zerar disponíveis no
  apresentador; teste determinístico de pausa, retomada e reinício em `cronometro.rs`.
- WordArt no PDF: teste `pdf_wordart_preserva_transformacao_sem_texto_duplicado`
  verifica que a deformação altera o arquivo e não duplica o texto comum.
- Item 5.5 (operações booleanas): delegado pelo usuário a outra pessoa.

## Seção 1.1 — operações básicas concluídas

Acesso pelos botões **Arquivos** do Início e do editor. As 26 entradas estão
implementadas no escopo nativo descrito no mapa acima; validação visual do
painel e do arraste pelo desktop ainda não foi realizada.

Testes cobrem proteção de somente leitura, múltiplas sessões e desfazer,
fechar/cancelar/recuperar, recuperação em uma nova instância, versões anteriores,
reverter com rascunho, duplicar, automático em segundo plano, renomear/excluir,
local/favoritos e linha de comando. O teste do motor cobre importação com IDs
únicos, gatilhos remapeados e desfazer/refazer do conjunto.

O Clippy do motor continua bloqueado por quatro funções preexistentes acima
do limite de linhas (`dados.rs`, `svgpath.rs` e `modelos.rs`).

Validação desta continuação: `cargo check` e testes do motor, interface e IO
aprovados. `cargo fmt --check` aponta diferenças de formatação anteriores em
vários arquivos da interface; os novos módulos foram formatados com rustfmt.

## Seção 4.2 — formatação de caracteres (14/10/2026)

- **FTS-0215/0217** (sublinhado e tachado duplos): `EstiloDoTexto::sublinhado_duplo`/
  `tachado_duplo`, `Formatacao::SublinhadoDuplo`/`TachadoDuplo` (liga os dois campos;
  o botão simples volta ao traço único). Colunas `sublinhado_duplo`/`tachado_duplo` no
  `.fsdz` (arquivo antigo lê como falso). Palco (`desenho.rs`) e PDF desenham dois traços.
  Leitura: `.pptx` `u="dbl…"` e `strike` (o tachado simples também passou a ser lido),
  `.odp` `style:text-underline-type`/`text-line-through-type = double`.
  Testes: `traco_duplo_liga_e_desliga` (GUI), `traco_duplo_persiste` (IO).
- **FTS-0218/0219/0223** (x², x₂, espaçamento): feitos em 13/10, ver
  `docs/PERSONALIZACAO-FORMAS-TEXTO.md`.
- **FTS-0220–0222** (maiúsculas e minúsculas): `texto_formato::Caixa` e
  `Sessao::texto_caixa` — frase, minúsculas, MAIÚSCULAS, Cada Palavra, aLTERNAR; no
  trecho em edição ou no texto inteiro da seleção; um passo de desfazer; trechos de
  x²/x₂ remapeados quando a letra muda de tamanho ("ß" → "SS"). Menu **Aa** na Fonte.
  Testes: `caixa_modos_e_trecho`, `caixa_desfaz_e_preserva_indices` (motor),
  `caixa_no_trecho_em_edicao_acompanha_a_selecao` (GUI).
- Validação: motor 116, IO 86, GUI 112 testes aprovados; ponte e MCP compilam.
  Aceite visual pendente.
- **FTS-0251** (nível de lista): **defeito corrigido** — Aumentar/Diminuir recuo
  chamavam `Formatacao::TamanhoPasso` e mudavam o tamanho da fonte. Agora
  `Sessao::texto_recuo` / `texto_formato::recuar` acrescentam ou tiram um nível
  (quatro espaços, ou um tab) das linhas do trecho em edição (ou da linha do cursor)
  ou da caixa inteira; linhas vazias não mudam; x²/x₂ acompanham. Testes:
  `recuo_por_linha_e_trecho`, `recuo_desfaz_e_preserva_indices` (motor),
  `recuo_muda_linhas_e_nao_a_fonte` (GUI). GUI 113 testes aprovados.
