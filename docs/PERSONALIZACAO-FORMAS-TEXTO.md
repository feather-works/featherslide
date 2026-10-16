# Personalização de formas e texto no Iced

Selecione uma forma e clique em **Formato da Forma**. A aparência fica
somente no painel lateral, sem controles duplicados na Ribbon. O painel
permite preenchimento sólido, sem preenchimento, gradiente linear ou radial.
No linear, o ângulo é horário: 0° vai da esquerda para a direita e 90° de
cima para baixo. O radial parte do centro. Gradientes aceitam de 2 a 16
pontos, cada um com cor hexadecimal e posição de 0 a 100%; Aplicar ordena
os pontos pela posição. Inverter cores também inverte as posições.

O mesmo painel oferece transparência de 0 a 100%, cor e espessura do
contorno, cor e transparência da sombra, direção, distância, desfoque de 0 a 14 pt
e deslocamentos X/Y.
Cores aceitam `#RRGGBB` e `#RRGGBBAA`; deixe o campo de contorno ou sombra
vazio para removê-los. As alterações aparecem diretamente no slide. **Aplicar** confirma a edição
em uma transação; Ctrl+Z desfaz. Cancelar, × ou Esc restauram a aparência
original sem criar um passo de desfazer.
Escolher uma cor sólida na faixa remove o gradiente anterior.

Em **Página Inicial → Fonte**, os botões vetoriais **x²** e **x₂** aplicam
sobrescrito e subscrito. Durante a edição há dois jeitos: selecionar o
trecho e clicar (digite `x2`, selecione `2` e clique em x²), ou clicar em
x² sem nada marcado e digitar: as letras saem sobrescritas até clicar de
novo ou mover o cursor. Digitar logo depois de um trecho em x² continua
nele. Um segundo clique sobre um trecho marcado remove o índice. Fora da edição, o comando vale para todo o texto
das caixas selecionadas. Os caracteres originais permanecem no documento;
o motor guarda os intervalos em bytes UTF-8 e resolve tamanho e posição
para desenho, seleção e PDF. Edições deslocam os intervalos restantes.

Os botões **Espaçamento entre letras** e **Maiúsculas e minúsculas** (Aa)
abrem um menu logo embaixo, com ícone e a opção atual marcada. O Espaçamento ajusta a distância entre letras: muito justo
(-2 pt), justo (-1 pt), normal (0), amplo (2 pt) e muito amplo (4 pt).
Limpar formatação também remove índices e espaçamento.

**Todos os slides**, no rodapé, abre a classificação em grade, também
acessível por **Exibir → Classificação**. Clicar em um cartão abre esse
slide no editor. O botão Voltar ao editor retorna ao slide atual.

A janela **Dados do gráfico** usa cartões e campos do tema compartilhado,
cabeçalho com contagem de categorias/séries, linhas alternadas e rolagem
horizontal e vertical. As alterações continuam atualizando o gráfico
imediatamente. Os botões de remoção respeitam a categoria/série mínima.

## Persistência e renderização

`data/elementos.arrow` acrescenta as colunas opcionais `gradiente_forma`
(texto JSON dos parâmetros), `indices_texto` (texto JSON dos intervalos)
e `espacamento_letras` (Float32). Arquivos antigos recebem os padrões
sem gradiente, sem índices e espaçamento zero; o contêiner `.fsdz` mantém
sua estrutura. O leitor valida gradientes e intervalos UTF-8.

Formas com gradiente, transparência ou sombra usam uma camada RGBA
compartilhada por Canvas, miniaturas e PDF, com limite de 1024 px no maior
lado da forma, margem para contorno e cache na GUI. O PDF mantém o texto
separado e usa máscara alfa para essa camada. A aparência original das
formas sem esses efeitos continua no desenho vetorial existente.

## Validação em 13/10/2026

- `cargo check` da GUI aprovado.
- Testes completos de Core, IO e GUI aprovados, incluindo desfazer/refazer,
  intervalos UTF-8, edição de texto, round-trip `.fsdz`, direção/radial/
  transparência do gradiente no PDF e comandos da interface.
- `cargo tree` do workspace compartilhado aprovado. `cargo check
  --workspace --all-targets` e `cargo test --workspace` foram executados,
  mas encontraram erros de compilação em testes fora desta implementação:
  chamadas de `gravar_tabela` com array em `featheroffice-native-io`,
  empréstimo de catálogo temporário em `featheroffice-io/tests/ooxml.rs`
  e atributo interno fora do início em `featheroffice-table-adapters/tests/edicao_slide_real.rs`.
- Aceite visual nas janelas reais permanece pendente. A reserva do
  checklist permanece `[/]`.

## Edição direta no palco — 13/10/2026

O painel ocupa 320 px do editor. Não há imagem de prévia separada:
a forma do documento mostra imediatamente as alterações.

Combinações prontas, pontos, cor e posição do gradiente ficam no mesmo
grupo. Na forma selecionada, arraste os pontos coloridos ao longo da linha
para mudar suas posições. A alça branca depois do fim da linha muda o
ângulo. As alças acompanham zoom e rotação. O ponto selecionado no palco
também fica selecionado no painel.

Sombra oferece direção, distância, transparência, desfoque e ajustes X/Y.
Contorno e sombra preservam a cor quando desligados e religados. Ao
selecionar outra forma, a edição anterior é confirmada e o painel passa a
mostrar a nova seleção. Aplicar confirma as alterações em um único passo
de desfazer; Cancelar restaura o retrato e o estado limpo anteriores.
Os caches raster de formas/sombras retêm oito entradas por cache para
limitar o acúmulo de versões em edições contínuas.

## Ajuda F1

O Slide usa `feather-ui::ajuda`, o mesmo widget do Sheet: menu lateral,
cartões, pesquisa e paginação. Mantém todos os tópicos e atalhos do Slide,
com janela de 1190×760 px e Ctrl+F para focalizar a pesquisa. A página
inicial não mostra comandos de fórmulas/planilha para apresentações.

## Validação deste incremento

Core: 114 testes; IO: 85; GUI: 110; componente de ajuda: 5 aprovados.
Incluem edição ao vivo, Cancelar sem mudar documento limpo, confirmação
em um passo de undo, arrasto real do Canvas, sombra e navegação do F1.

A suíte completa do `feather-ui` também foi executada: 49 testes passaram
e o hash do catálogo de ícones falhou; o catálogo já estava alterado no
workspace antes deste incremento. O baseline não foi regenerado.
Check/test de `featheroffice` continuam bloqueados por chamadas de
`gravar_tabela` com arrays em testes de adaptadores fora desta alteração.
Aceite visual na janela real permanece pendente; manter `[/]`.
