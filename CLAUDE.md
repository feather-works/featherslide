# FeatherSlide — estado do projeto

Apresentações da suíte (sétimo app na contagem antiga; hoje um dos cinco). Nasceu já com o
nome novo em **07/10/2026** (a pasta `openslide/` só tinha o plano, movido para
`docs/ARQUITETURA.md`). Regras da suíte: `../CLAUDE.md` e `../RECEITA.md` — valem todas.

**Foco atual: 100% Rust com Iced, motor e interface.** Estimativa do usuário:
**~40% das funcionalidades prontas**, em 07/10/2026. Não é o avanço da migração.
Continuar no FeatherSlide; depois FeatherDocs, FeatherSheet e, se houver tempo,
FeatherCAD. Checklist completa: `docs/INVENTARIO-FUNCIONALIDADES.md` (44 áreas,
1.684 entradas). Evidências: `docs/VERIFICACAO-FUNCIONALIDADES.md`.

## Porta para o pelicano, lado a lado com o Iced (21/10/2026)

- `crates/featherslide-pelicano` (binário `featherslide-pelicano`) é a cópia da interface
  portada para o pelicano, para comparar com o `featherslide-gui` (Iced), que segue intacto.
  Mudança feita num lado precisa ser repetida no outro até escolher um.
- Depende do pelicano v0.1.9 e do feather-ui v0.3.0 (a versão sobre o pelicano) por git +
  tag; o `featherslide-gui` segue no feather-ui v0.2.x (Iced).
- Canvas: o `Renderer` usa coordenadas da janela; só caminhos, `fill_text` e imagens seguem
  `push_transform`.
- Igual ao Iced, medido por captura: mistura de cores linear (texto), sombra, trilho da
  barra de rolagem, setas das listas. `src/rastros.rs` compara redesenho parcial com o cheio.
  Capturas: `FEATHERSLIDE_CAPTURAR_ABA=inserir|design|…` e `FEATHERSLIDE_CAPTURAR_TELA=inicio`
  (nos dois apps). Testes com limite: `systemd-run --user --scope -p MemoryMax=8G
  -p MemorySwapMax=0 -p CPUQuota=400% cargo test --release -j 4 …`.
- Falta: lista de fontes cinza como no Iced, polegar da barra sob o mouse, "Selecionar tudo"
  não cabe em 1280 px (nos dois), picos de 1,4–1,7 s com milhares de elementos.

## Formas comuns no `feather-shapes` (14/10/2026)

- A parte pura de formas, ícones e diagramas saiu do core para o crate comum
  `feather-shapes` (repositório `../feather-shapes`, tag local v0.1.0, ainda não
  publicada: cada raiz de build tem `[patch]` com o caminho local). Geometria de
  `formas`/`novas_formas`/`outras_galerias`, recorte das `booleanas`, `wordart`,
  `svgpath`, catálogo e busca de `icones` (dados Tabler e `tools/gerar_tabler.py`
  foram junto), `diagramas` (modelos), arranjos do `editor_diagrama` e a aparência
  de `estilo_forma` (com `Sombra`).
- Os módulos do core continuam com os mesmos caminhos públicos (reexportam) e
  ficam com `Elemento`, `Sessao`, desfazer e `Primitiva`. A ponte é
  `core/src/formas_comuns.rs` (`Tipo` ↔ `TipoForma`, item de diagrama →
  `Elemento`, geometria → `Primitiva`). `Tipo::TODOS` e o `.fsdz` não mudaram;
  a saída das funções de geometria foi comparada byte a byte antes e depois.

## Vetores SVG — seção 6.3 (08/10/2026)

- **Inserir → SVG…** importa geometria de cores sólidas como um grupo vetorial;
  **Formato da Forma → Vetores → Converter SVG** libera suas peças para edição.
  Preenchimento, contorno e tamanho usam os controles existentes.
- `core/src/vetores.rs` preserva M/L/Q/C/Z e fill-rule em `Tipo::Caminho`, campo
  `dados` com prefixo `SVG1`. Curvas são normalizadas na caixa e permanecem
  vetoriais no `.fsdz`, no canvas Iced e no PDF. A prévia MCP usa os mesmos segmentos.
- `io/src/svg.rs` usa usvg, com resolução externa desativada. Máscaras, filtros,
  gradientes, padrões, texto/imagens, opacidade de grupo e traços especiais
  ainda são recusados explicitamente. FTS-0446/0447/0449 permanecem `[/]`.
- **Exportar SVG…** exporta a seleção de vetores, ícones e caminhos; usa o tamanho
  do slide como viewBox. Não é exportação completa de slides com fotos/texto.
- Testes: importação de curvas/transformações/alfa/furos, edição e desfazer,
  ampliação sem perda, persistência `.fsdz`, SVG ida/volta, Bézier/evenodd no PDF
  e renderização da prévia com furo transparente ao fundo do slide.

## Comandos atuais (de dentro de `featherslide/`)

```bash
cargo run --release --manifest-path crates/featherslide-gui/Cargo.toml
cargo check --manifest-path crates/featherslide-gui/Cargo.toml
cargo test --manifest-path crates/featherslide-core/Cargo.toml
cargo test --manifest-path crates/featherslide-io/Cargo.toml
cargo test --manifest-path crates/featherslide-ortografia/Cargo.toml
```

## Categorias de diagramas (FTS-0587–0600, 08/10/2026)

- `core/src/diagramas.rs`: 14 modelos gerados com formas, conectores e
  textos nativos, agrupados por `grupo` e inseridos em uma edição.
- `gui/src/faixa_abas.rs`: **Inserir → Diagramas**; a galeria
  `gui/src/galeria_figuras.rs` desenha prévias da mesma geometria do motor.
- 13 categorias completas como modelos editáveis; **diagramas com imagens
  permanecem parciais**, pois os espaços de foto são placeholders vetoriais.
  Adicionar fotos reais a nós, editar hierarquia semântica ou fazer round-trip
  de SmartArt de PPTX são etapas distintas ainda pendentes (9.2/32).
- `.fsdz` utiliza os mesmos elementos/grupos persistentes já existentes;
  testes no core e IO. Conferir visualmente no Ubuntu.

## Animações de entrada (FTS-0760–0772, 08/10/2026)

- `modelo.rs`: 12 novas variantes anexadas a `TipoAnimacao::TODOS` sem mudar
  códigos antigos. `fsdz.rs` passa a v10 (novos códigos 7..18), mantendo leitura
  dos arquivos v9. `faixa_abas.rs` oferece seletor de 19 efeitos e três atalhos.
- `desenho.rs`: máscaras por elemento (Surgir, Dividir, Apagar, Forma,
  Barras), rotação (Roda/Girar), interpolação (Flutuar/Deslizar), rebote e
  escalas (Expandir/Contrair). Motor continua com undo/ordem e prévia no Iced.
- FTS-0773 Pulso segue efeito de ênfase, não de entrada. Máscaras em imagens
  podem variar conforme renderizador; conferir animações no Ubuntu, em especial
  sobre fotografias.

## Arrastar e soltar seções (FTS-0115/0119, 08/10/2026)

- No painel de miniaturas, clicar no cabeçalho continua recolhendo/expandindo.
  Arrastar pelo cabeçalho e soltar em **outra seção ou slide dela** move a seção
  completa com todos os slides em uma operação de desfazer.
- O alvo mostra linha azul de inserção; um cartão flutuante identifica a
  seção arrastada. Arrastar para cima insere antes do alvo; para baixo,
  depois dele. Funciona entre seções não adjacentes e recolhidas.
- O motor identifica seções por limites/índices, não por nome; admite
  seções homônimas e padrão implícito. Os IDs dos slides são preservados.
- A interface preserva quais seções estavam recolhidas por ID do cabeçalho
  e pode desfazer com Ctrl+Z. Testes cobrem os casos de motor e fluxo Iced;
  validar o movimento do mouse no Ubuntu permanece necessário.

## UI compartilhada (08/10/2026)

- O tema anteriormente em `gui/src/tema.rs` foi extraído para
  `/feather-ui/src/tema.rs`; no gui, o módulo antigo apenas reexporta a
  mesma API, evitando mexer em dezenas de chamadas já testadas.
- `faixa_abas.rs` usa `feather_ui::componentes` para botão grande,
  botão pequeno e grupo da faixa, mantendo as mensagens FeatherSlide locais.
- FeatherDocs/FeatherSheet ainda não usam Iced; sua integração acontecerá
  durante a migração dos próprios front-ends e não afeta os executáveis Tauri.
- Não existe Workspace global nem ligação dinâmica; as dependências são
  compiladas em cada aplicativo nativo.

## Arquitetura

### Outras galerias — FTS-0327–0340 (08/10/2026)

- Inserir → Formas organiza fluxogramas, balões, faixas/fitas, ondas/explosões,
  matemática, ação, conectores e desenho livre. São 24 modelos vetoriais em
  `core/src/outras_galerias.rs`, com prévias Iced, seleção, formatação,
  redimensionamento, rotação e desfazer. Não são ícones bitmap.
- Conectores reto, cotovelo e curvo conectam duas formas selecionadas ou pedem
  cliques na origem e no destino; Esc cancela. Com uma selecionada, basta
  clicar no destino. As âncoras
  seguem as bordas das caixas após edições. Cópias remapeiam os IDs quando
  incluem as duas formas; sem elas, a cópia é um traço independente.
- Desenho livre: arrastar no palco, com prévia imediata; soltar insere um caminho
  aberto. Esc cancela. Limite de 10.000 pontos; uma transação no histórico.
- Formas de ação Início/Anterior/Próximo recebem destino de slide na inserção;
  Informação é uma forma para configurar um hiperlink pelos comandos existentes.
- Persistência `.fsdz` v11: caminhos `SVG1` e âncoras `#CON` dentro da coluna
  `dados`; os formatos anteriores continuam legíveis. Tela/PDF/MCP reutilizam
  primitivas vetoriais. As curvas dos modelos preservam Bézier.
- Limites: não há editor de vértices ou roteamento que evite obstáculos;
  ancoragem usa a caixa, não o contorno exato de uma forma girada. Excluir um
  alvo mantém o traço na última posição. As formas não possuem texto interno;
  pode-se sobrepor e agrupar uma caixa de texto. Validação visual ainda pendente.
- Testes: `core/src/outras_galerias.rs`, `io/tests/fsdz.rs` e testes do app Iced.

- `crates/featherslide-core` — o motor, só `serde`: `modelo` (Apresentação, Slide, Elemento,
  coordenadas em pontos, 16:9 = 960×540), `layouts` (título, título e conteúdo, seção, dois
  conteúdos, somente título, em branco), `medida` (largura **aproximada** do texto),
  `quadro` (o slide pronto para pintar: linhas já quebradas e posicionadas), `sessao`
  (estado, versão por slide), `slides`, `elementos`, `edicao` (desfazer por retrato da
  apresentação inteira, até 200 passos).
- `crates/featherslide-io` — `.fsdz` (ZIP + `data/apresentacao.arrow`, `slides.arrow`,
  `elementos.arrow`, `assets/` para imagens). Sem JSON no arquivo, como o `.fdoz`.
- `crates/featherslide-ponte` — despacho Rust legado (`COMANDOS_DA_PONTE`),
  preservado com seus testes; não é dependência da interface Iced.
- `crates/featherslide-gui` — interface nativa atual em Iced; chama core/io diretamente.

## Limpeza da interface web (08/10/2026)

O código TypeScript, seus testes, páginas HTML e configurações npm/Vite/Tauri
foram retirados após a migração da interface para Iced. Execute e valide o
aplicativo com Cargo, usando os comandos acima. A Ajuda preserva seu conteúdo
em `crates/featherslide-gui/dados/pt-BR.json`. Recursos de fontes e ícones são
preservados para reutilização; a limpeza não implica que todas as funcionalidades
do inventário estejam concluídas.

## Decisões não óbvias

- **Casca copiada, não importada** (regra 3): menu, faixa, tema, paletas, configurações e
  Ajuda vieram do FeatherDocs; Monitor e console do FeatherCAD. Mudança na casca de um
  app precisa ser repetida nos outros até virar pacote comum.
- **Quadro em JSON**, não binário: um slide tem dezenas de elementos, e a versão por slide
  já evita pedir de novo o que não mudou. Rever se aparecer slide pesado.
- **Seleção mora no motor** (como o cursor no FeatherDocs): os comandos agem sobre ela.
- **Edição de texto nativa**: cursor e alteração do conteúdo são tratados pelo
  motor Rust; a interface Iced recebe as teclas e desenha o cursor. A medida
  do texto ainda é aproximada; métricas reais da fonte continuam pendentes.
- O que não tem motor aparece na faixa como "Em breve", desabilitado, com a dica do que
  fará (mesma decisão do FeatherDocs).

## Operações booleanas de formas (FTS-0363–0367, 07/10/2026)

- Implementadas em `crates/featherslide-core/src/booleanas.rs`, expostas em
  **Formato da Forma → Mesclar formas** (com duas ou mais formas selecionadas) no Iced. União, interseção,
  subtração (primeira selecionada menos as demais), combinação (XOR)
  e fragmentação em áreas separadas.
- Segmentos de forma são recortados e classificados geometricamente no Rust,
  **sem rasterizar**, produzindo `Tipo::Caminho` com anéis normalizados em
  `Elemento::dados`. Suporta retângulos, elipses, triângulos, setas,
  formas adicionais da galeria e novos caminhos; círculos e cantos arredondados
  são aproximados por segmentos vetoriais.
- O caminho é desenhado pelas `Primitiva::Caminho`, preserva buracos,
  aceita edição de cor/contorno/transformação e usa o mesmo histórico de
  desfazer/refazer. `.fsdz` formato 5 preserva o código novo sem alterar os
  códigos dos tipos antigos.
- Testes do motor cobrem operações, áreas, furos, fragmentação, desfazer,
  e interseção vazia; teste do formato confirma o round-trip. Restrições:
  apenas formas fechadas sem imagens ou texto especial; limites geométricos
  e precisão dependem de tolerâncias para curvas e casos degenerados.
  **Validação visual no desktop ainda é necessária**.

## Formas adicionais e faixa compacta (07/10/2026)

- FTS-0287 (triângulo retângulo), FTS-0293 (heptágono), FTS-0295 (decágono) e
  FTS-0296 (dodecágono): `Tipo` nativo, contorno em `formas.rs`,
  inserção pela galeria existente de Formas, renderização Iced e PDF via
  `formas::pontos`, persistência `.fsdz` e uso nas operações booleanas.
- Compatibilidade: variantes novas de `Tipo::TODOS` acrescentadas **no final**
  para preservar os códigos posicionais dos arquivos existentes.
- Faixa de opções: Mesclar formas, distribuir, centralizar e agrupar usam
  ícones Tabler **já existentes**, com nome completo e explicação no tooltip;
  não introduzir ícone duplicado nem comando sem funcionalidade.

## Formas básicas completas (5.1; FTS-0282 a FTS-0312)

- Implementadas as formas pendentes FTS-0298–0300 e FTS-0302–0311:
  **anel, arco, corda, cubo, cilindro, moldura, canto em L, placa, sol, lua,
  coração, raio e nuvem**, além de círculo próprio (FTS-0285) e estrelas
  de **4, 5, 6, 8, 10 e 12 pontas** (FTS-0312).
- Modelo `Tipo` contém novos identificadores **no fim de `Tipo::TODOS`**,
  preservando os valores dos 32 tipos anteriores. A geometria fica em
  `core/src/novas_formas.rs`, em pontos vetoriais, nunca em bitmap.
- Integração: galeria de Formas gera previews da própria geometria; ao
  inserir, o elemento é editável, aceita mover, redimensionar, girar, formatar,
  desfazer/refazer e persistir no `.fsdz`. Círculo mantém lados iguais;
  anel e moldura têm vazios reais para preenchimento e seleção; arco é aberto,
  sem preenchimento; cubo e cilindro são ícones **2D**, não sólidos 3D.
- Iced e PDF consomem primitivas vetoriais do motor; a prévia MCP também
  desenha os mesmos contornos. A união/subtração aceita as regiões fechadas,
  mas não o arco aberto. Curvas são **aproximadas por vértices**; não há
  alças Bézier ou controle paramétrico individual de cada desenho.
- Evidências: testes de geometria e furos em `novas_formas.rs`, seleção/
  proporção em `core/tests/sessao.rs`, round-trip em `io/tests/fsdz.rs`,
  e CI Rust/Iced. **A conferência visual no Ubuntu continua necessária**.
- `.fsdz` formato **6**: identifica novos tipos de formas sem deixar binários
  antigos interpretarem códigos desconhecidos como caixas de texto; arquivos
  formato 5 continuam legíveis na versão atual.

## Propriedades do documento (FTS-0053–FTS-0070; 07/10/2026)

- Campos gerais **Título, Assunto, Autor, Empresa, Gerente, Categoria,
  Palavras-chave, Comentários e Idioma** editáveis pelo painel
  **Propriedades** (ícone já existente na barra superior). A janela trabalha
  com rascunho: **Aplicar** registra uma edição única no histórico,
  **Cancelar** não altera o documento. Respeita arquivo somente leitura.
- Campos automáticos: data de criação, última modificação e último autor,
  revisões (monotônicas mesmo depois de desfazer/refazer), identificador
  estável e tempo de edição **estimado** pelo intervalo de atividade entre
  ações (até 300 s por intervalo). Datas ausentes: "Não disponível".
- Propriedades personalizadas e metadados estendidos são mapas editáveis,
  com chaves únicas, exclusão, limites de tamanho, validação e Unicode.
  Estatísticas computadas no Rust em tempo real: slides, slides ocultos,
  elementos, formas, textos, imagens, tabelas, gráficos, animações,
  notas, seções, comentários, palavras/caracteres e fontes embutidas.
- Persistência **.fsdz v7**, novas colunas `data/apresentacao.arrow`,
  sem JSON no arquivo; leitor aceita versões anteriores com valores padrão.
  Importação de PPTX: `docProps/core.xml`, `app.xml` e `custom.xml`;
  de ODP: `meta.xml`. Metadados que a origem não possui permanecem
  desconhecidos, sem inventar uma data histórica.
- Ao duplicar uma apresentação, o conteúdo e as propriedades editáveis
  permanecem, mas nasce outro **identificador, criação e histórico de edição**.
  O idioma informativo do documento não substitui o idioma do corretor.
  Não promete round-trip PPTX/ODP nem paridade com todos os atributos
  externos; validação visual do painel no desktop ainda é necessária.
- Testes: `core/src/metadados.rs`, `core/tests/sessao.rs`,
  `io/tests/fsdz.rs`, `io/src/metadados_externos.rs` e
  `gui/src/propriedades.rs`; GitHub Actions verifica core/IO/Iced.

## Operações de slides — seção 2.1 (FTS-0089–0112)

- Operações nativas de slides implementadas em core/src/operacoes_slides.rs,
  reutilizando estruturas de Slide, FonteEmbutida e tema já existentes.
  Inserir slide antes/depois, duplicar, copiar, recortar, colar, excluir,
  mover/reordenar e ocultar/exibir múltiplos slides em uma única transação.
- Área de transferência interna vive no app Iced (gui/src/app.rs) e funciona
  entre documentos abertos na mesma instância; Ctrl+C/X/V/A age sobre
  miniaturas quando estas têm foco, sem interferir na edição de texto/objetos.
  Preservar formatação mantém o visual; adaptar aplica paleta/fontes e decoração
  do tema de destino. Links internos entre slides copiados são remapeados;
  links que apontam fora da seleção são descartados.
- Reutilização seletiva: menu da miniatura → "Reutilizar slides de outro
  arquivo…" aceita .fsdz/.pptx/.odp; painel gui/src/reutilizar_slides.rs
  permite marcar páginas e escolher o estilo da colagem. "Importar todos
  os slides…" reutiliza Arquivos → Importar já existente (adiciona ao fim).
  A interpretação de formatos externos continua parcial.
- Exportar um slide individual usa o exportador PDF do slide atual; "Ir para
  o slide número…" aceita 1..N e não altera o documento.
  Restaurar layout recompõe as caixas do layout e preserva textos/formas
  livres. Aplicar layout aos slides selecionados é uma edição única.
- Operações de grupo respeitam a exigência de ao menos um slide por arquivo.
  Desfazer/refazer é um passo por lote; IDs de slides/elementos são novos na
  duplicação, colagem e reutilização; proporções são ajustadas ao destino.
- Testes do motor em core/src/operacoes_slides.rs validam inserção antes,
  transferências e IDs, tema, lote e undo; io/tests/fsdz.rs valida
  imagem/notas persistentes após colagem e reabertura. Validação de uso visual
  com mouse/arrasto no Ubuntu segue pendente. Não há integração do
  clipboard de slides com outros programas (somente entre abas do app).

## Estrutura — seção 2.2 (FTS-0113–0123)

- **Seções:** criar/renomear/excluir e recolher/expandir já existiam.
  Mover e reordenar seções agora usa o menu do botão direito no cabeçalho
  (subir/descer), movendo o bloco inteiro em um único desfazer.
  Com várias miniaturas marcadas, botão direito no cabeçalho da seção de
  destino → **Mover slides marcados para esta seção**. O motor mantém IDs,
  elementos, notas, animações, conteúdo e limites; não copia nem recria slides.
- **Apresentações personalizadas:** aba Apresentação → **Apresentações
  personalizadas**. Criar a partir dos slides marcados; acrescentar, remover
  e reordenar com ↑/↓. Omitir slides em um roteiro não altera o ocultamento
  global da apresentação. Reprodução segue a ordem personalizada, inclusive
  navegação anterior/próximo e repetição. Os roteiros usam IDs estáveis:
  reordenar a apresentação geral não altera o roteiro; slides excluídos
  aparecem como ausentes no editor e são pulados na reprodução.
- Dados tipados no core/src/personalizadas.rs, UI no
  gui/src/personalizadas.rs, gui/src/app.rs e menu_contexto.rs.
  Persistência .fsdz **v9** no arquivo Arrow data/personalizadas.arrow;
  leitura de arquivos antigos mantém os roteiros vazios, sem inventar dados.
  Não há JSON no contêiner. Limites: 50 roteiros e 1.000 entradas por roteiro.
- Testes em core/src/secoes.rs, core/src/personalizadas.rs e io/tests/fsdz.rs
  cobrem mover grupos, IDs, desfazer, ordenação/ocultamento, ida e volta
  de arquivo. Validação visual no Ubuntu ainda pendente. Arrasto de
  cabeçalho não foi implementado; a reordenação é por botões subir/descer.

## Manipulação de imagens — seção 6.2 (FTS-0417–0445)

- Toda em Rust. Ajustes não destrutivos em core/src/imagem.rs,
  copiados ao quadro e gui/src/desenho.rs, utilizados no renderizador Iced
  e na exportação PDF. Persistência .fsdz **v8** com colunas Arrow
  visual_imagem (parâmetros separados por vírgula, sem JSON) e
  texto_alternativo. Compatível com leitura de versões anteriores.
- **Interface compacta da faixa Formato da Imagem (08/10/2026):**
  TODOS os comandos principais da seção 6.2 agora são acessíveis
  **na própria faixa**, sem precisar abrir o antigo painel modal.
  Recorte como figura e enquadrar estão empilhados; as setas de
  mover foto saíram da faixa (arraste e roda continuam no palco).
  O corte por bordas tem seletor e ajuste +/-; o grupo Fundo contém
  Remover/Restaurar, cor/conta-gotas, técnica logo abaixo e um botão
  **+ Ajustes** que expande tolerância, suavidade, borda e pena,
  mantendo refino e marcas do watershed.
- Opacidade é um **campo percentual livre (0–100, Enter ou ✓)**, sem
  presets. Contorno reúne **amostra de cor + espessura no mesmo grupo**,
  sem divisor. Os grupos Cor e Efeitos usam seletores compactos de
  parâmetros e controles -/+, filtro artístico, sombra e espelhamento.
  Arquivo tem Substituir, menus Reduzir/Compactar e ALT expandível com
  campo de texto. A faixa de imagem é um pouco mais alta que as demais
  e tem **rolagem horizontal** em janelas estreitas.
- Estado de UI em gui/src/app.rs (seletores, expandir, rascunhos);
  layout em gui/src/faixa_abas.rs; mensagens em gui/src/faixa.rs.
  O novo desenho não altera estruturas de arquivo ou algoritmos
  do motor. O painel gui/src/imagem_painel.rs permanece no código por
  compatibilidade, mas não é necessário para ajustar a imagem.
- Compactação efetiva só ao clicar: JPEG 60/85 para foto opaca e PNG
  para foto com alfa; redução 1280/800 px por Lanczos3. Ambas modificam
  a imagem incorporada, são reversíveis por Ctrl+Z, preservando
  original durante todos os outros ajustes.
- Sombra é camada raster com blur/deslocamento, atrás da foto na
  interface e no PDF. Reflexo projetado abaixo da foto e halo externo
  são camadas temporárias com alfa, renderizadas em Iced/PDF.
  Perspectiva é uma transformação trapezoidal 2D, não câmera 3D.
  Cor transparente exata (modo 5) apaga pixels do RGB escolhido;
  também existe o modo por tolerância. Remoção de fundo possui cinco
  técnicas locais mas não segmentação semântica por IA.
  Texto alternativo não produz PDF acessível com tags estruturadas.
- Testes: core/src/imagem.rs; gui/src/imagem_processamento.rs e
  imagem_arquivo.rs; io/tests/fsdz.rs. Comparação visual no Ubuntu
  ainda pendente.

## Bordas de tabela (08/10/2026; FTS-0467)

- Inserir → Bordas da tabela ajusta a espessura de todas as células da
  tabela selecionada em passos de 0,5 pt (0–50 pt). Zero oculta as bordas.
- `dados::primitivas_tabela` usa `Elemento::espessura`, compartilhada pelo
  desenho Iced e pela exportação PDF. A cor vazia continua usando o cinza
  padrão das tabelas existentes. Sem mudança no formato `.fsdz`.
- Testes cobrem primitivas, desfazer/refazer e persistência da formatação.
  Validação visual no desktop ainda pendente.

## Configuração do slide (2.3; 08/10/2026)

- Design → Configurar slide…: largura/altura em pontos (36–7200), retrato ou
  paisagem, modos Esticar, Ajustar e Maximizar. Ajustar mantém proporções e
  centraliza; Maximizar preenche a área e pode cortar conteúdo. Os modos
  proporcionais também escalam fonte e espessura. Colar slides considera
  as dimensões personalizadas da origem.
- Fundo sólido, imagem local esticada à área, gradiente horizontal de duas
  cores, textura pontilhada e padrão xadrez vetoriais. Opacidade 0–100%.
  Aplicar a todos copia toda a aparência. Ocultar mestre/layout filtra as
  formas `Papel::Fundo` apenas no desenho, sem apagá-las.
- Painel guarda rascunhos; aplicar tamanho e aplicar fundo são transações
  independentes de desfazer. Cancelar/fechar não aplica os campos pendentes.
- `.fsdz` formato **9** acrescenta dimensões em `apresentacao.arrow` e
  configuração do fundo em `slides.arrow`. Imagens continuam em `assets/`;
  memória compartilhada evita copiar seus bytes a cada passo do histórico.
  Leitura de versões anteriores usa dimensões e fundo padrão.
- Testes cobrem ajuste, validação, ocultação, aplicar a todos, persistência,
  compatibilidade de colunas antigas, PDF e comandos da interface. Inspeção
  visual do painel e dos fundos ainda pendente. O Clippy do motor mantém os
  erros anteriores em dados.rs, svgpath.rs e modelos.rs.

## Fila

3. **Ferramentas da Página inicial** (pedido de 07/10/2026) — **feito e testado**:
   Ativadas as 10 ferramentas que estavam cinzas com `breve(...)` na faixa de opções:
   - **Fonte:** `tachado` (strikethrough no motor, salvo no `.fsdz` e desenhado no canvas), `limpar` (reverte formatação para o estilo padrão da caixa).
   - **Parágrafo:** `lista_marcadores` (alterna marcadores `• ` no motor), `lista_numerada` (alterna numeração ordenada `1. `, `2. ` etc. no motor), `entrelinha` (1.0, 1.15, 1.5, 2.0 com espaçamento proporcional no `quadro.rs`), `colunas_texto` (1 e 2 colunas distribuídas no motor), `direcao_texto` (rotação em passos de 90°), `alinhar_justificado` (`Alinhamento::Justificado` com distribuição visual no canvas e no motor, salvo no `.fsdz` e suportado no `.odp`).
   - **Edição:** `localizar` (Ctrl+F) e `substituir` (Ctrl+H) com painel flutuante `PainelDeBusca` integrado aos atalhos; motor de busca 100% Rust puro em `featherslide-core/src/busca.rs` (diferenciação de maiúsculas, palavra inteira com limites Unicode, navegação próximo/anterior com seleção e troca de slide no motor, substituição unitária e substituir tudo com registro atômico no histórico de desfazer).
4. Medida real do texto (as métricas da fonte, como o FeatherDocs) — hoje é aproximada.
5. Texto rico por trecho (negrito numa palavra), listas complexas, tabela, gráfico do FeatherSheet.
   **Dados de gráfico/tabela (decisão do usuário, 07/10):** editam-se no FeatherSheet (fórmula
   só lá); sem o FeatherSheet instalado, uma grade embutida só com valores fixos, sem fórmula.
   Detalhes em `docs/ARQUITETURA.md` ("Editar os dados de gráfico e tabela").
6. Exportar PDF e `.pptx`; transições e animações; modo do apresentador.
7. Histórico `.git` dentro do `.fsdz` (o FeatherDocs já tem); ícones da suíte revistos.

## Interface nativa em Iced (`crates/featherslide-gui`) — estado de 07/10/2026

**Ícones da interface:** as 109 funções de `gui/src/icones.rs` usam agora os SVGs
Tabler Icons 3.49.0 já embutidos no core, em vez dos desenhos manuais. A galeria
e os botões compartilham a biblioteca; traço 2 e viewBox 24×24 originais,
cores adaptadas ao tema. Origem: https://github.com/tabler/tabler-icons (MIT,
© Paweł Kuna). Crédito no README e em Configurações → Sobre; licença em
`dados/TABLER-LICENSE.txt` do `feather-shapes` (exposta como `featherslide_core::icones::LICENCA_TABLER`). Símbolos inseríveis no texto e a marca “F”
do aplicativo continuam sendo conteúdo textual e identidade do produto.

- **Janelas** (`iced::daemon`): principal + Monitor (F12), Ajuda (F1), Configurações, Dados do gráfico e
  Painel de animação, cada uma uma janela do sistema (`Janela` em `app.rs`).
- **Tabela e gráfico de verdade**: `Tipo::Tabela`/`Tipo::Grafico`, dados em texto compacto em
  `Elemento::dados` (`core/src/dados.rs`, que também calcula o desenho em *primitivas* — retângulos,
  setores, linhas, textos — que a interface só pinta). Coluna `dados` no `.fsdz` formato **4**.
- **Design/Transições/Animações**: `core/src/temas.rs` (8 temas, `Apresentacao::tema`), `Slide::transicao`,
  `Elemento::animacao`, `core/src/efeitos.rs` (comandos + `passos_de_animacao`); colunas novas no `.fsdz` 4.
  A reprodução (tempo, transição entre slides, passos por clique) é de `gui/src/apresentacao.rs`.
- **Abas da faixa**: Início em `faixa.rs`; Inserir/Design/Transições/Animações/Apresentação/Exibir em
  `faixa_abas.rs`. Menu de contexto (botão direito) em `menu_contexto.rs`. Tema claro/escuro e cor de
  destaque em `tema.rs` (`definir_tema`); preferências e atalhos regraváveis em `configuracao.rs`.
- **Formas do fundo do layout** (`Papel::Fundo`, vindas do mestre do `.pptx`) não se selecionam:
  `fundo_liberar` (Design → "Editar formas do fundo") as solta; `selecao_para_fundo` faz o inverso.
- **Seções (pastas de slides)**: `Slide::secao` (não vazio = o slide abre uma seção), `core/src/secoes.rs`
  (`secoes`, `secao_adicionar/renomear/remover`, `secoes_remover_todas`), coluna `secao` no `.fsdz` 4. Recolher/expandir é só da
  interface (`secoes_recolhidas` em `app.rs`); painel em `gui/src/miniaturas.rs` (cabeçalho em canvas, renomear no próprio painel).
  Slide oculto aparece apagado com um olho riscado. Miniaturas usam `canvas::Cache` + `quadro_em_cache`; o tique de 1 s só liga com
  salvamento automático ou Monitor aberto (antes reconstruía a tela inteira todo segundo).
- **Seleção múltipla e gatilho (07/10)**: Ctrl/Shift+clique marca vários slides (`slides_sel`; excluir/duplicar/ocultar agem em todos).
  Arrastar uma miniatura: mostra a prévia flutuante do slide sendo arrastado junto ao cursor; sobre outra miniatura reordena;
  sobre o slide solta a **visão do slide** salva (miniatura PNG real do slide de destino via `tiny-skia`, com contorno e sombra suave,
  sem botão textual "Ir para o slide X"; atualiza as visões automaticamente ao salvar a apresentação) com gatilho (`Elemento::destino`,
  coluna `destino` do `.fsdz`, `Sessao::gatilho_inserir`) que na apresentação leva ao slide (`Acao::Ir`). Toda ação vai às Métricas
  (`[ação] Palco::Clicou`…, `[quadro] montar a tela`, `[abrir] …`). `[profile.dev]` otimizado no gui: o build sem otimização era a causa dos 266 ms ao abrir.
- **Zoom**: com zoom > 100 %, roda do mouse rola a tela (Shift = lado) e botão do meio arrasta (`pan` em `app.rs`, limitado em `palco.rs`).
- **Ideia adiada (07/10)**: extrair texto de imagem com `ocrs` (Rust puro, modelos ~10 MB, só sob pedido); fórmulas ficam de fora (modelos pesados).
- **Falta/Próximo**: Monitor de CPU "estranho" (usuário reclamou) — falta gráfico de CPU e conferir a leitura; testar tudo visualmente
  (não há captura de tela no ambiente); commitar (nada foi commitado); importar seções/tabelas/transições do `.pptx`.
- **Link e Comentário (07/10)**: `Elemento::link` (só `http`/`https`/`mailto`, validado em `core/src/links.rs`; `exemplo.com` vira `https://`) e `Slide::comentarios` (autor, texto, resolvido). Colunas `link` e `comentarios` no `.fsdz` sem subir a versão (arquivo antigo lê como vazio). Na apresentação, clicar no elemento com link abre com `xdg-open` (só por clique); gatilho de slide tem prioridade. Caixas em `gui/src/revisao.rs` (Inserir → Link / Comentário). Testes no core, no io e no app. Falta: link em trecho de texto (hoje é do elemento), importar link/comentário do `.pptx`, MCP.

## Ajustes da interface nativa — 07/10/2026 (Codex)

- Parágrafo: alinhamento vertical (topo/meio/base) na terceira linha; ícones Tabler
  sem moldura (`layout-align-*`) também no Organizar. Recompilar release para vê-los.
- Formas: 14 opções adicionais (losango, pentágono, hexágono, octógono, trapézio,
  paralelogramo, estrela, cruz, setas esquerda/dupla/cima/baixo, chevron e documento).
  Geometria comum em `core/src/formas.rs`, usada pela galeria, palco, PDF e PNG do MCP.
  Códigos de tipo novos no fim de `Tipo::TODOS`, sem mudar os anteriores; ida e volta `.fsdz` testada.
- Mouse sobre Transições/Animações: prévia imediata no palco, uma vez por entrada.
  Passar o mouse não muda documento, efeito escolhido, seleção nem histórico. Sair
  do botão deixa o efeito terminar; clique aplica e reproduz novamente. O fundo
  externo é a mesa do editor, com o mesmo zoom/pan; preto só na tela cheia. `gui/src/previa.rs` reaproveita o renderizador da apresentação.
- Redimensionar seleção múltipla: um fator comum para tamanho, posição relativa,
  fonte e traço; um único Ctrl+Z. Shift ao arrastar mantém proporção de objeto único,
  inclusive girado; Ctrl continua sendo modificador de seleção.
- Ícones: aba contextual “Formato do Ícone” e menu do botão direito. Cor, espessura,
  tamanho proporcional, rotação e trocar desenho sem perder formatação/posição.
- Equações: texto Unicode com busca de fontes alternativas habilitada no palco e
  menu. Não há parser LaTeX nem editor de equações estruturadas; falta validar o
  exemplo concreto dos caracteres que o usuário relatou como não reconhecidos.

## Salvamento — continuação (Codex)

- FTS-0011: Salvar cópia em Configurações → Salvamento e Ctrl+Alt+S (regravável).
  Grava .fsdz e preserva o documento original, nome, caminho e alterações pendentes.
- FTS-0012: automático usa tempo real após a última alteração, com intervalo configurável.
  Sem caminho, mantém rascunho exclusivo em `~/.config/featherslide/rascunhos/`
  (respeita XDG_CONFIG_HOME; pasta temporária como fallback). O caminho aparece na
  barra de status; pode ser aberto normalmente. Rascunho não limpa o estado sujo.
  Recuperação ao iniciar agora está disponível em Arquivos (FTS-0013/0014).
- Testes: cópia preserva original; automático cobre intervalo, documento nomeado,
  rascunho sem nome, desativação e falha de gravação.

## Operações básicas 1.1 — concluídas (Codex)

- Botão **Arquivos** no Início e no editor: abrir somente leitura, em branco,
  salvar/cópia, fechar/todas, duplicar, renomear, excluir, reverter, importar slides,
  pesquisar no local e gerenciar favoritos.
- Várias apresentações ficam abertas com sessões e desfazer independentes;
  barra de documentos para alternar, inclusive salvamento automático de sessões
  em segundo plano. Abrir/criar não descarta a anterior.
- Rascunhos periódicos a cada 30 s, mesmo sem automático ligado, e antes de
  fechar/reverter. São oferecidos em Recuperação ao iniciar; abrem como cópia
  editável sem sobrescrever o original. Confirmar fechamento conserva rascunhos.
- Histórico de versões salva o conteúdo anterior em `.<arquivo>.versoes/`
  ao lado do .fsdz antes da substituição atômica. O painel abre versões como cópias;
  renomear leva o histórico junto. Git dentro do .fsdz continua sendo outro item.
- Excluir pede confirmação no app e mantém uma cópia recuperável. Renomear
  preserva a extensão do arquivo e recusa destino existente.
- `featherslide caminho1.fsdz "caminho com espaços.pptx"` abre múltiplas sessões;
  arrastar arquivos para a janela passa pelo mesmo fluxo.
- Importar slides aceita .fsdz/.pptx/.odp com remapeamento de IDs, grupos e
  gatilhos e incorporação de fontes; segue as limitações dos importadores.
- Itens Cancelados do inventário (imagem online, biblioteca e álbum de fotos)
  permanecem fora da implementação.
- Testes automatizados em diretórios temporários; gravação das preferências
  reais fica desativada no binário de testes. Validação visual do painel pendente.
- Clippy: erros preexistentes de comprimento de função no motor, registrados
  em VERIFICACAO-FUNCIONALIDADES.md.

## Conferir a tela sem o usuário (14/10/2026)

`FEATHERSLIDE_CAPTURAR=/caminho/tela.png ./target/release/featherslide` abre o app,
insere e seleciona uma caixa de texto, fotografa a janela principal (1280×800) e fecha.
`FEATHERSLIDE_CAPTURAR_MENU=caixa|espacamento|tracos` abre também um menu da Fonte.
A primeira foto às vezes sai antes do menu aparecer: repetir. Usar antes de devolver
mudança visual ao usuário.
