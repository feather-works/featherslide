# Registro de tarefas

> Era o README de `featherslide/` enquanto o produto morava no repositório de desenvolvimento da suíte (até 15/10/2026). Fica como histórico das entregas e validações; links para `featheroffice/` e outros produtos apontam para pastas que não existem neste repositório.


> **Produto independente — repositório planejado:** `FeatherOffice/featherslide`.
> O projeto ainda está nesta pasta do repositório de desenvolvimento; a separação Git não foi realizada.

**Apresentações:** formato nativo `.fsdz`; desenvolvimento e versões próprios.

**Arquitetura compartilhada:** utiliza a base de motores Rust em [`featheroffice/`](../../featheroffice/) e os componentes de interface Rust/Iced de [`feather-ui/`](../../feather-ui/) quando aplicáveis. As dependências compartilhadas serão versionadas sem acoplar um produto ao código interno de outro.

**Publicação prevista:** versão **alpha em 30/10/2026**. O produto terá distribuição própria e também poderá ser selecionado isoladamente ou junto aos demais em um **instalador geral planejado**. Isso não significa que a funcionalidade de instalação já esteja pronta.


## IO-014 — Base64 consolidado em Image (compilação pendente)

O antigo `featherslide-io::base64` mantém seus quatro símbolos públicos,
agora como reexportações de `featheroffice-image::base64`. O algoritmo duplicado
foi removido; a serialização FSDZ, PPTX e ODP **não foi modificada**.
Compatibilidade de assets e de abertura dos arquivos precisa de Cargo/smoke.
[Guia e comandos](../../featheroffice/IO-014-DEDUP-BASE64.md).

## APP-003 — Tabelas e exportação de gráficos comuns (em validação)

Ao selecionar um gráfico, a guia contextual oferece **Exportar gráfico SVG…**
por Charts/Drawing comuns; o diálogo salva um SVG vetorial da grade nativa,
sem modificar `Elemento::dados` ou o formato `.fsdz`. As operações de
estrutura das tabelas continuam no motor compartilhado
`featheroffice-table-adapters`. O exemplo `app003_intercambio` grava
um gráfico SVG e um FSDZ com tabela/gráfico, conferindo a leitura de volta.
**Cargo e inspeção visual pendentes; APP-003 `[/]`.**

[API, roteiro GUI e comandos](../../featheroffice/APP-003-CHARTS-TABLES-FEATHERSLIDE.md).


## UI-011 — alinhamento compartilhado

A faixa Página Inicial → Parágrafo agora usa o seletor horizontal
`feather_ui::alinhamento` (esquerda, centro, direita, justificado),
com seleção visual do estilo da forma e botões inativos sem seleção.
O comando ainda é `FaixaMensagem::Alinhar`; `.fsdz` não foi alterado.

```bash
cargo test --manifest-path featherslide/crates/featherslide-gui/Cargo.toml ui011_alinhamento_horizontal_slide_tem_estado_e_undo -j 2
```

[Integração e roteiro de teste](../../featheroffice/UI-011-ALINHAMENTO-COMUM.md).
**Status `[/]`, validação Cargo/GUI pendente.**


## IMG-013 — Image compartilhado no Canvas e PDF (em validação)

O `desenho::decodificar` e a prévia do Canvas usam agora
`featherslide_io::imagem_compartilhada::decodificar_raster`: verificam
64 MiB/40 milhões de pixels antes de descomprimir, aplicam o mesmo
redimensionamento Triangle do Slide, e seguem com os filtros e efeitos
compartilhados. A exportação PDF reutiliza a prévia RGBA do mesmo caminho.
Perspectiva não clona o buffer original e restaura os pixels em caso de erro;
`.fsdz` não muda. Testes e exemplo IMG-013 foram publicados, mas Cargo e
inspeção visual continuam pendentes (`[/]`).

[Guia técnico, matriz real e comandos](../../featheroffice/IMG-013-FEATHERSLIDE-IMAGE.md).


## APP-004 — Ribbon compartilhada (em validação)

Os grupos da aba Página Inicial, botões de recortar/copiar/duplicar,
operações de slides, ações de fonte, dicas e botões de Ajuda/Configurações
passam a reutilizar `feather-ui::componentes` e
`feather-ui::formatacao`. As mensagens `FaixaMensagem` e os atalhos
F1/F12/Ctrl+F1 continuam sob controle do Slide; modelos e `.fsdz`
não foram alterados.

```bash
cargo run --manifest-path featherslide/crates/featherslide-gui/Cargo.toml -j 2
cargo test --manifest-path featherslide/crates/featherslide-gui/Cargo.toml app004_ribbon_migrada_preserva_aba_atalhos_e_documento -j 2
```

**Status `[/]`:** integração publicada, Cargo e aceitação visual pendentes.
[Mapa de controles e roteiro completo](../../featheroffice/APP004-RIBBON-FEATHERSLIDE.md).



## Paridade de formas Drawing → Iced/SVG/PDF (DRW-012 — em validação)

O Canvas nativo passa pelos testes de uma `Cena` com formas
sobrepostas idênticas às do snapshot SVG/PDF do motor compartilhado.
As quatro âncoras do `Viewport` testam escalonamento de pontos para
pixels lógicos e físicos (DPR), sem dupla conversão. Um gradiente que o
Iced ainda não consegue reproduzir causa erro explícito e permanece
no fallback existente, em vez de desaparecer. **Teste de GPU e captura
raster real pendentes, tarefa `[/]`.**

```bash
cargo test --manifest-path featherslide/crates/featherslide-gui/Cargo.toml drw012_ -j 2 -- --nocapture
cargo run --manifest-path ../feather-draw/Cargo.toml --example drw012_snapshots -j 2 -- /tmp/featheroffice-drw012
```

[Corpus e limites de tolerância](../../featheroffice/fixtures/drw012/README.md).



## Regressão dos arquivos nativos (IO-009 / IO-013)

O formato `.fsdz` mantém slides, textos, imagens, notas e metadados sob
controle do codec próprio, sem substituir a serialização durante a migração
Rust/Iced. **IO-009** verifica salvar → ler → salvar → ler num corpus compacto.
**IO-013** amplia a regressão para apresentações extensas, Unicode, formas,
seções, comentários e arquivo truncado. O estado dos dois IDs permanece
`[/]` até execução efetiva dos testes.

```bash
cargo run --manifest-path featherslide/crates/featherslide-io/Cargo.toml --example io009_roundtrip -j 2 -- /tmp/io009-slide.fsdz
cargo run --manifest-path featherslide/crates/featherslide-io/Cargo.toml --example io013_extremos -j 2 -- /tmp/io013-slide.fsdz
```

[Corpus IO-009](../../featheroffice/fixtures/io009/README.md) ·
[Testes de estresse IO-013](../../featheroffice/fixtures/io013/README.md)


## Image único no Iced e PDF — APP-002

O processamento visual de imagens está centralizado no módulo
`featherslide-io::imagem_compartilhada`, independente de Iced.
`imagem_processamento` na GUI apenas reexporta os métodos compatíveis,
preservando testes de regressão pixel a pixel. A montagem de camadas
raster na exportação PDF usa diretamente a mesma ponte, sem alterar as
imagens originais do `.fsdz`. Efeitos preservam alfa.

```bash
cargo test --manifest-path featherslide/crates/featherslide-io/Cargo.toml imagem_compartilhada::testes -j 2
cargo run --manifest-path featherslide/crates/featherslide-io/Cargo.toml --example app002_pdf_imagem -j 2 -- /tmp/app002
cargo test --manifest-path featherslide/crates/featherslide-gui/Cargo.toml imagem_processamento::testes -j 2
```

[Detalhes, limitações e aceite](../../featheroffice/APP002-IMAGE-SLIDE-PDF.md).
**APP-002 permanece `[/]` até testes de GUI, PDF e round-trip FSDZ.**


## Migração de formas nativas ao Drawing — APP-001 (em validação)

A tela Iced agora desenha `retangulo`, `elipse`, `triangulo` e `linha`
usando `feather-draw`, além dos caminhos vetoriais migrados
em DRW-008. A conversão usa `featherslide-io::drawing::converter_forma_basica`;
o Iced mantém rotação, transformações de animações, coordenadas em pontos
e arrastos. Estilos e formas com ajustes especiais usam a pintura anterior.
**Sem mudanças no `.fsdz`, IDs, modelo ou histórico.**

[Escopo, testes e limitações](../../featheroffice/APP001-MIGRACAO-FORMAS.md).

```bash
cargo test --manifest-path featherslide/crates/featherslide-io/Cargo.toml drawing::testes -j 2
cargo run --manifest-path featherslide/crates/featherslide-io/Cargo.toml --example app001_formas -j 2 -- /tmp/app001-formas.svg
cargo check --manifest-path featherslide/crates/featherslide-gui/Cargo.toml -j 2
```

**APP-001 permanece `[/]` até testes e comparação visual.**


## Edição de tabelas via motor compartilhado (TBL-011 — em validação)

O FeatherSlide passa a consumir o motor real `featheroffice-tables` nos
comandos de edição de célula, adição/exclusão de linhas e colunas e navegação
de teclado (Tab, Shift+Tab, Enter e setas). O crate
`featheroffice-table-adapters` fornece
`slide_editar_celula`, `slide_editar_estrutura` e
`slide_navegar_celula`. O binário Iced usa a feature Cargo `slide` para
**não compilar FeatherDocs como dependência da interface**.

A GUI mantém a UX de edição dentro da célula; os eventos passam pelo modelo
esparso comum e o retorno atualiza somente `Elemento::dados`. O comando
`Sessao::tabela_dados_definir` do core salva a mudança como um **único passo**
de Ctrl+Z/Ctrl+Y. Identificadores, posição, animações, estilo do elemento,
renderização e persistência do `.fsdz` permanecem sob responsabilidade dos
módulos nativos do Slide. As dimensões máximas do Ribbon (30×15) continuam
iguais. Inputs e comandos incompatíveis geram mensagens visíveis no status.

A migração **não** transfere para o motor compartilhado a renderização das
células, o conteúdo de texto com atributos por célula ou todas as funções de
copiar/colar. Isso exige testes visuais e futuras tarefas TBL-010/APP-003.
O crate `featheroffice-table-adapters/tests/edicao_slide_real.rs` adiciona
10 testes usando `Sessao` real: compara comandos antigo/novo, valida
Undo/Redo, Unicode, teclas, limites e ausência de alterações após erro.
Não foi possível executá-los com Cargo neste ambiente.

Validação a partir da raiz do repositório:

```bash
git pull --ff-only origin main
cargo test --manifest-path featheroffice/Cargo.toml -p featheroffice-table-adapters --test edicao_slide_real -j 2 -- --nocapture
cargo test --manifest-path featheroffice/Cargo.toml -p featheroffice-table-adapters -j 2
cargo check --manifest-path featherslide/crates/featherslide-gui/Cargo.toml -j 2
cargo test --manifest-path featheroffice/Cargo.toml --workspace --all-targets -j 2
```

## Canvas Iced com geometria comum (DRW-008 — validação pendente)

Além da exportação SVG comum, o próprio Canvas do Slide reutiliza o
`feather-draw` para pintar caminhos SVG editáveis e compostos. O
módulo `featherslide-gui/src/drawing_iced.rs` recebe `Cena/Figura/Caminho`,
mapeia pontos → pixels lógicos via `Viewport` e preserva curvas Q/C,
fill-rule, opacidade e ordem Z. Para estilos sem equivalência segura, o
desenho atual é mantido como fallback — sem mudar os bytes `.fsdz`.

Teste direcionado:
```bash
cargo test --manifest-path featherslide/crates/featherslide-gui/Cargo.toml drawing_iced::testes -j 2
cargo check --manifest-path featherslide/crates/featherslide-gui/Cargo.toml -j 2
```

Documentação: [DRW008-ICED.md](../../featheroffice/DRW008-ICED.md).
A migração está `[/]` até regressões e conferência visual na GUI.

## Exportar SVG com Drawing compartilhado (DRW-011 — em validação)

A ferramenta **Exportar SVG** no app Iced agora usa
`featherslide-io::drawing::exportar_svg`, que transforma elementos suportados
em `feather-draw::Cena` com medidas em pontos. O exportador antigo
`featherslide-io::svg::exportar` e a importação SVG continuam disponíveis.
O core, os IDs e a serialização `.fsdz` não foram modificados.

O adaptador inclui formas geométricas, formas adicionais com furos, caminhos
SVG editáveis e ícones Tabler. Texto, imagem, tabelas, gráficos e formas que
não possuem equivalência de renderização devolvem erro explícito, sem
descartar silenciosamente seus atributos. Consulte
[DRW011-MIGRACAO-SLIDE.md](../../featheroffice/DRW011-MIGRACAO-SLIDE.md)
para o mapa de formatos, limitações e testes.

```bash
cargo test --manifest-path featherslide/crates/featherslide-io/Cargo.toml drawing::testes -j 2
cargo run --manifest-path featherslide/crates/featherslide-io/Cargo.toml --example drw011_formas -j 2 -- /tmp/drw011-formas.svg
cargo check --manifest-path featherslide/crates/featherslide-gui/Cargo.toml -j 2
```

**DRW-011 permanece `[/]` até Cargo, round-trip FSDZ e inspeção visual.**

# FeatherSlide

Apresentações da suíte FeatherOffice (ver o `README.md` da raiz). Formato próprio
**`.fsdz`** (ZIP com a apresentação em colunas Arrow e as imagens como arquivos).

**Motor e interface 100% Rust, com Iced.** Foco atual da suíte, com **~40% das
funcionalidades prontas**, na estimativa do usuário (07/10/2026). Ordem de trabalho:
**FeatherSlide → FeatherDocs → FeatherSheet**, depois FeatherCAD se houver tempo.

A interface nativa chama diretamente os crates do motor.
O frontend TypeScript e a configuração npm/Vite/Tauri foram removidos.
Builds e testes do aplicativo usam Cargo.

```bash
cd crates/featherslide-gui
cargo run --release  # aplicativo nativo usado atualmente
```

## Adaptador de gráficos compartilhados (CHT-014 — em validação)

O crate `featherslide-io` oferece o módulo público
`grafico_compartilhado`, que converte gráficos existentes (colunas,
barras, linhas e pizza) do `Elemento::dados` em
`featheroffice-charts::Grafico` e exporta SVG/PDF individuais, sem
alterar o formato `.fsdz` nem introduzir dependências no
`featherslide-core`. A GUI e o PDF de apresentações continuam no
renderizador legado até comparação visual.

Na raiz do repositório:

```bash
cargo test --manifest-path featherslide/crates/featherslide-io/Cargo.toml \
  --lib grafico_compartilhado -j 2 -- --nocapture
cargo run --manifest-path featherslide/crates/featherslide-io/Cargo.toml \
  --example cht014_grafico_slide -j 2 -- /tmp/featheroffice-cht014
```

Detalhes: [migração CHT-014](../../featheroffice/ADAPTADOR-CHARTS-FEATHERSLIDE.md).

O plano de longo prazo (grafo de cena, auto-layout, dado vivo do FeatherSheet,
modo do apresentador) está em `docs/ARQUITETURA.md`; o estado e a fila, em `CLAUDE.md`.

O catálogo completo enviado pelo usuário está na
[`checklist de funcionalidades`](../docs/INVENTARIO-FUNCIONALIDADES.md): 44 áreas,
1.684 entradas, IDs estáveis e estados de acompanhamento. As evidências e
limitações encontradas no código ficam em
[`Verificação de funcionalidades`](../docs/VERIFICACAO-FUNCIONALIDADES.md).

Os ícones SVG da interface e da galeria são do [Tabler Icons](https://github.com/tabler/tabler-icons),
versão 3.49.0, © Paweł Kuna, licença MIT. Os caminhos vetoriais ficam embutidos no
aplicativo e funcionam sem rede. Os dados e a licença
(`dados/TABLER-LICENSE.txt`) ficam no crate comum
[`feather-shapes`](https://github.com/FeatherOffice/feather-shapes) e o crédito
também aparece em Configurações → Sobre.
