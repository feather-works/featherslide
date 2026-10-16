# OpenSlide — o contrato

**Direção atual — 07/10/2026: motor e interface 100% Rust, com Iced.**
Prioridade: **FeatherSlide → FeatherDocs → FeatherSheet**; FeatherCAD
depois, se houver tempo. O inventário abaixo registra funcionalidades e
decisões anteriores; a migração da interface deste aplicativo ainda deve
ser conferida no código, sem presumir que os recursos já existem em Iced.

Decidido em 24/09/2026, pelo usuário. Ainda não existe código — mesmo
papel que `feathersheet/docs/arquitetura/ARQUITETURA.md`, `openpdf/ARQUITETURA.md` e
`opendocs/ARQUITETURA.md` cumprem: **o motor é Rust, a interface desenha**
(regra 2 do `CLAUDE.md` da raiz). É o plano, não a implementação.

## O problema que resolve

PowerPoint/Google Slides tratam o slide como caixa de texto solta em
posição de pixel absoluto — distorce em projetor 4:3, mídia pesada corrompe
o arquivo, gráfico colado é print estático que fica desatualizado no dia
seguinte. O OpenSlide é um grafo de cena 2D com coordenada relativa, dado
vivo do FeatherSheet, e um motor de animação físico em vez de "efeito de
PowerPoint".

## O contêiner

```
apresentacao.oslz (ZIP/zstd — contrato geral em FORMATO.md)
├── manifest.json              # metadados, formato base (16:9), slide inicial
├── .git/                      # cada versão relevante é um commit (FORMATO.md)
├── presentation/
│   ├── deck.json               # ordem dos slides, transição global
│   ├── theme.json              # tokens de cor/tipografia/sombra do tema
│   └── slides/
│       ├── slide_001.json      # grafo de cena do slide (nó, transform)
│       └── slide_001_notes.md  # notas do apresentador, Markdown puro
├── data/
│   └── queries.json            # ligação viva com pipeline do OpenQuery/.fshz
└── assets/                     # imagem, SVG, vídeo local, fonte embutida
```

## Grafo de cena: coordenada relativa, não pixel

Cada slide tem espaço de coordenada independente (`0.0`–`1.0` ou canvas
lógico `1920×1080`) — nunca pixel de tela física, que é o que distorce em
projetor antigo ou monitor ultrawide. Todo nó carrega transform vetorial:

```json
{
  "id": "node_card_kpi",
  "type": "container",
  "transform": { "x": 0.05, "y": 0.20, "width": 0.40, "height": 0.35, "rotation": 0.0, "z_index": 2 },
  "style": { "fill": "#1E1E24", "radius": 8, "border": "1px solid #333" },
  "children": [ ]
}
```

**Hierarquia explícita pai-filho** — `Slide` contém `Container` (card,
seção) contém `Elemento` (texto, gráfico do FeatherSheet, vetor do OpenDraw);
mover o container move os filhos juntos. A interface expõe isso num painel
de camadas com ícone por tipo de nó (`[T]` texto, `[#]` código/Markdown,
`[fx]` equação, `[▤]` tabela/query viva, `[◇]` vetor, `[🖼]` imagem),
trava/ocultar por linha, e um *breadcrumb* no topo do canvas (`Slide 04 >
Seção Métricas > Card Esquerdo`) para navegar sem errar o clique.

## Auto-layout e grade magnética

Container com regra de fluxo (vertical/horizontal), `gap` paramétrico,
*padding* e alinhamento — acrescentar um quarto card num grupo de três
redistribui largura e espaçamento sozinho, sem sobrepor. *Smart snapping*
detecta alinhamento de borda/centro contra outro nó e avisa quando o
espaçamento entre três elementos é equidistante. Nó pode alternar entre
fluxo automático e posição livre (para elemento decorativo solto).

## Conteúdo

- **Markdown rápido**: `- item`, `## Título`, `**ênfase**` viram nó da
  cena na hora — mesmo princípio de `opendocs/ARQUITETURA.md`.
- **Bloco de código com destaque de sintaxe** (Go, Rust, Python, SQL, JS…)
  — ver "Pontos em aberto" abaixo, porque a biblioteca de gramática mais
  usada em Rust (`syntect`) tem uma pegadinha com a mesma classe de
  problema de dependência C que já apareceu duas vezes na suíte.
- **LaTeX inline/bloco** — **é o mesmo ponto em aberto de
  `opendocs/ARQUITETURA.md`** ("tipografia matemática"), não um segundo a
  decidir: os dois apps usam o mesmo motor, decisão única.
- **Componente vivo do FeatherSheet**: tabela/gráfico ligado a um `.fshz` ou
  a uma consulta do OpenQuery; "Atualizar" relê a fonte. Zero-cópia de
  verdade — o slide não copia a tabela inteira para a memória da
  apresentação, lê fatia contígua do Arrow IPC do arquivo de origem, então
  uma agregação de 1 milhão de linhas abre instantânea. Precisa do mesmo
  **crate comum de leitura entre apps** que `opendocs/ARQUITETURA.md` já
  apontou como faltando (ponto 3 de lá) — os dois consumidores validam o
  desenho junto.
- **Vetor nativo do OpenDraw**: curva matemática de verdade, não
  rasterizado — nítido em projeção gigante.

## Tema: tokens, não template engessado

`theme.json` guarda cor, tipografia, raio e sombra como *design tokens*
nomeados (`bg_primary`, `accent`, `font_heading`, `scale_ratio`…), nunca
valor solto por slide. Trocar de tema é remapear os tokens — os 50 slides
mudam de uma vez, sem quebrar layout, porque layout não depende de cor
nenhuma diretamente. Tema exporta/importa como arquivo próprio
(`.otheme`), e componente reutilizável (card de comparação, prancha de
esquema) salva na biblioteca local para arrastar em slide novo.

## Animação: física, não "efeito de PowerPoint"

- **Magic Move**: duplicar um slide e mudar posição/tamanho/rotação de um
  nó — o motor casa o mesmo `id` nos dois slides e interpola (curva Bézier
  ou *spring physics*) automaticamente. É comparação estrutural de dois
  grafos de cena, não gravação de vídeo.
- **Gatilho por passo**: "ao clicar", "com o anterior" (simultâneo),
  "após o anterior" (cascata); presets prontos (fade + subida, escala
  sutil, "escrita" tipográfica para código/Markdown, pulso em métrica).
- **Timeline na base da tela**: faixa por animação, duração e atraso
  ajustados arrastando — visual, não campo numérico only.

## Execução: quem computa, quem desenha

- **Rust**: matriz de transformação 2D, curva de interpolação da animação,
  cálculo de auto-layout — produz **um buffer de coordenada pronto por
  quadro**, nunca HTML nem instrução de desenho.

## Exportação

- **PDF via crate do `openpdf`** (dependência de crate, regra 3 — não
  código duplicado): cada slide vira página 16:9, texto e vetor sem
  rasterizar.
- **Mídia otimizada localmente** — ver "Pontos em aberto": imagem tem
  caminho resolvido (mesma reamostragem de `opendocs/ARQUITETURA.md`),
  vídeo não.

## Pontos que precisam de decisão antes de alguém começar a implementar

1. **Destaque de sintaxe**: `syntect` é a biblioteca Rust mais usada para
   isso, mas o *backend* de regex padrão dela (`onig`) é um binding C para
   Oniguruma — a mesma classe de problema que já apareceu com
   `harfbuzz`/`hunspell`. Ela tem um backend alternativo em Rust puro
   (`fancy-regex`); confirmar que dá pra forçar esse backend antes de
   aceitar a dependência, para não abrir uma exceção silenciosa à regra
   "sem lib C atrás".
2. **Tipografia matemática**: não decidir aqui — é o mesmo ponto em
   aberto do `opendocs/ARQUITETURA.md`, resolvido uma vez para os dois
   apps.
3. **Vídeo embutido não tem resposta em Rust puro.** Ao contrário de
   texto/ortografia/imagem, transcodificação e otimização de vídeo de
   verdade praticamente sempre se apoia em `ffmpeg` (C) por trás — não há
   equivalente maduro em Rust puro para o pipeline completo (decodificar
   contêiner, recodificar, remuxar). Precisa de decisão explícita do
   usuário: aceitar uma dependência C isolada só nesta fatia, limitar o
   suporte a formatos que codecs Rust puro já cobrem (ex. AV1 via
   `rav1e`/`dav1d`, mais restrito), ou adiar vídeo embutido para depois do
   essencial.
4. **O crate comum de leitura entre apps** (tabela Arrow do FeatherSheet,
   cena do OpenDraw) é pré-requisito daqui também, não só do OpenDoc — vale
   desenhar pensando nos dois consumidores ao mesmo tempo, não um depois
   do outro.
