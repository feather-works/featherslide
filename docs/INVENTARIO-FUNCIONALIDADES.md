# Inventário completo de funcionalidades — FeatherSlide

Catálogo fornecido pelo usuário em 07/10/2026, convertido em checklist de
funcionalidades e requisitos de compatibilidade. Mantém as **44 áreas** e
as seis partes do inventário. Formatos, efeitos e recursos dependentes de
versão ou plataforma continuam sendo candidatos, não promessas de suporte.

**Direção do projeto: motor e interface 100% Rust, com Iced.** Ordem de foco:
**FeatherSlide → FeatherDocs → FeatherSheet**, depois FeatherCAD se houver tempo.
Os **~40%** informados pelo usuário são uma **estimativa de funcionalidades
já prontas no FeatherSlide**; não são uma medida da migração nem o resultado
de dividir itens marcados pelo total deste catálogo.

## Como acompanhar

- Cada entrada tem um ID estável `FTS-0001` para referências em tarefas e testes.
- `[x]` **Feito**: implementação identificada no código, com evidência registrada.
- `[/]` **Parcial**: há implementação com limitações explicitadas.
- `[ ]` **Pendente**: funcionalidade ainda ausente no escopo examinado.
- `[ ]` **A verificar**: falta conferir implementação, acesso na interface ou testes.
- `[ ]` **Cancelado**: não é para implementar
- **Fora do escopo atual**: decisões da suíte podem adiar ou excluir itens do catálogo.

Uma marca de implementação não comprova fidelidade ao PowerPoint nem validação
visual. Motor, interface, leitura PPTX e gravação PPTX devem ser avaliados
separadamente. Testes e evidências ficam em
[`VERIFICACAO-FUNCIONALIDADES.md`](VERIFICACAO-FUNCIONALIDADES.md).
IDs existentes não devem ser renumerados; novos itens recebem IDs novos.
Não marcar um item como pendente apenas porque sua verificação ainda falta.

As áreas 41–44 são propostas de organização, avaliação e referências; não
representam funcionalidades do PowerPoint nem exigem instalar todas as bibliotecas.

# PARTE I — FUNCIONALIDADES DO EDITOR


## 1. Arquivos e gerenciamento de apresentações


### 1.1. Operações básicas

- [x] **FTS-0001** — Criar apresentação. **Feito**.
- [x] **FTS-0002** — Criar apresentação em branco. **Feito**.
- [x] **FTS-0003** — Criar apresentação baseada em modelo. **Feito**.
- [x] **FTS-0004** — Abrir apresentação. **Feito**.
- [x] **FTS-0005** — Abrir arquivos recentes. **Feito**.
- [x] **FTS-0006** — Abrir arquivos somente para leitura. **Feito**.
- [x] **FTS-0007** — Fechar apresentação. **Feito**.
- [x] **FTS-0008** — Fechar todas as apresentações. **Feito**.
- [x] **FTS-0009** — Salvar. **Feito**.
- [x] **FTS-0010** — Salvar como. **Feito**.
- [x] **FTS-0011** — Salvar cópia. **Feito**.
- [x] **FTS-0012** — Salvar automaticamente. **Feito**.
- [x] **FTS-0013** — Recuperar documentos não salvos. **Feito**.
- [x] **FTS-0014** — Recuperar versões após encerramento inesperado. **Feito**.
- [x] **FTS-0015** — Reverter para versão salva. **Feito**.
- [x] **FTS-0016** — Visualizar histórico de versões. **Feito**.
- [x] **FTS-0017** — Duplicar apresentação. **Feito**.
- [x] **FTS-0018** — Renomear apresentação. **Feito**.
- [x] **FTS-0019** — Excluir apresentação. **Feito**.
- [x] **FTS-0020** — Abrir múltiplas apresentações simultaneamente. **Feito**.
- [x] **FTS-0021** — Alternar entre apresentações. **Feito**.
- [x] **FTS-0022** — Pesquisar arquivos. **Feito**.
- [x] **FTS-0023** — Gerenciar locais favoritos. **Feito**.
- [x] **FTS-0024** — Abrir arquivos pela linha de comando. **Feito**.
- [x] **FTS-0025** — Arrastar arquivos para abrir. **Feito**.
- [x] **FTS-0026** — Importar arquivos para uma apresentação existente. **Feito**.

### 1.2. Formatos de arquivo — Microsoft

- [/] **FTS-0027** — PPTX — apresentação Open XML. **Parcial**.
- [ ] **FTS-0028** — PPTM — apresentação com macros. **A verificar**.
- [ ] **FTS-0029** — PPT — formato binário legado. **A verificar**.
- [ ] **FTS-0030** — POTX — modelo Open XML. **A verificar**.
- [ ] **FTS-0031** — POTM — modelo com macros. **A verificar**.
- [ ] **FTS-0032** — POT — modelo legado. **A verificar**.
- [ ] **FTS-0033** — PPSX — apresentação de slides. **A verificar**.
- [ ] **FTS-0034** — PPSM — apresentação de slides com macros. **A verificar**.
- [ ] **FTS-0035** — PPS — apresentação de slides legada. **A verificar**.
- [ ] **FTS-0036** — PPAM — suplemento com macros. **A verificar**.
- [ ] **FTS-0037** — THMX — tema. **A verificar**.

### 1.2. Formatos de arquivo — outros

- [/] **FTS-0038** — ODP — OpenDocument Presentation. **Parcial**.
- [/] **FTS-0039** — PDF — exportação. **Parcial**.
- [ ] **FTS-0040** — SVG — exportação de elementos vetoriais. **A verificar**.
- [ ] **FTS-0041** — PNG. **A verificar**.
- [ ] **FTS-0042** — JPEG. **A verificar**.
- [ ] **FTS-0043** — GIF. **A verificar**.
- [ ] **FTS-0044** — TIFF. **A verificar**.
- [ ] **FTS-0045** — BMP. **A verificar**.
- [ ] **FTS-0046** — EMF. **A verificar**.
- [ ] **FTS-0047** — WMF. **A verificar**.
- [ ] **FTS-0048** — MP4. **A verificar**.
- [ ] **FTS-0049** — WMV — compatibilidade de exportação legada. **A verificar**.
- [ ] **FTS-0050** — RTF — importação de estruturas de texto. **A verificar**.
- [ ] **FTS-0051** — TXT — estruturas de tópicos. **A verificar**.
- [ ] **FTS-0052** — HTML — exportação alternativa. **A verificar**.

### 1.3. Propriedades do documento

- [x] **FTS-0053** — Título. **Feito** (Apresentacao::titulo; painel Iced; importação de dc:title do PPTX/ODP; ver limitações na verificação).
- [x] **FTS-0054** — Assunto. **Feito** (metadados.assunto; importação de dc:subject; ver limitações na verificação).
- [x] **FTS-0055** — Autor. **Feito** (Apresentacao::autor; painel Iced; importação de dc:creator / meta:initial-creator; ver limitações na verificação).
- [x] **FTS-0056** — Empresa. **Feito** (metadados.empresa; docProps/app.xml: Company; ver limitações na verificação).
- [x] **FTS-0057** — Gerente. **Feito** (metadados.gerente; docProps/app.xml: Manager; ver limitações na verificação).
- [x] **FTS-0058** — Categoria. **Feito** (metadados.categoria; cp:category em PPTX; ver limitações na verificação).
- [x] **FTS-0059** — Palavras-chave. **Feito** (metadados.palavras_chave; cp:keywords/meta:keyword; ver limitações na verificação).
- [x] **FTS-0060** — Comentários. **Feito** (metadados.comentarios, diferente de comentários por slide; ver limitações na verificação).
- [x] **FTS-0061** — Data de criação. **Feito** (metadados.criado_em_utc; ISO de PPTX/ODP quando informado; ver limitações na verificação).
- [x] **FTS-0062** — Data de modificação. **Feito** (metadados.modificado_em_utc; carimbo automático da última alteração; ver limitações na verificação).
- [x] **FTS-0063** — Último autor. **Feito** (metadados.ultimo_autor; cp:lastModifiedBy e dc:creator; ver limitações na verificação).
- [x] **FTS-0064** — Número de revisões. **Feito** (metadados.revisoes; incremento a cada edição inclusive desfazer/refazer; ver limitações na verificação).
- [x] **FTS-0065** — Tempo total de edição. **Feito** (metadados.tempo_edicao_segundos; períodos ativos estimados e importação do original; ver limitações na verificação).
- [x] **FTS-0066** — Propriedades personalizadas. **Feito** (metadados.propriedades_personalizadas; chave/valor com suporte a Unicode; ver limitações na verificação).
- [x] **FTS-0067** — Idioma. **Feito** (metadados.idioma; pt-BR por padrão; metadado distinto do corretor ortográfico; ver limitações na verificação).
- [x] **FTS-0068** — Identificador do documento. **Feito** (metadados.identificador; criado localmente, estável e renovado ao duplicar; ver limitações na verificação).
- [x] **FTS-0069** — Metadados estendidos. **Feito** (metadados.estendidos; chave/valor e dados de origem da importação; ver limitações na verificação).
- [x] **FTS-0070** — Estatísticas da apresentação. **Feito** (metadados::estatisticas; slides, elementos, notas, textos, palavras etc. atualizados; ver limitações na verificação).

### 1.4. Segurança

- [ ] **FTS-0071** — Proteger com senha. **A verificar**.
- [ ] **FTS-0072** — Criptografar apresentação. **A verificar**.
- [ ] **FTS-0073** — Abrir em modo somente leitura. **A verificar**.
- [ ] **FTS-0074** — Restringir edição. **A verificar**.
- [ ] **FTS-0075** — Marcar como final. **A verificar**.
- [ ] **FTS-0076** — Verificar assinatura digital. **A verificar**.
- [ ] **FTS-0077** — Assinar digitalmente. **A verificar**.
- [ ] **FTS-0078** — Verificar integridade de arquivos. **A verificar**.
- [ ] **FTS-0079** — Inspecionar metadados. **A verificar**.
- [ ] **FTS-0080** — Remover informações pessoais. **A verificar**.
- [ ] **FTS-0081** — Detectar conteúdo externo. **A verificar**.
- [ ] **FTS-0082** — Controlar vínculos externos. **A verificar**.
- [ ] **FTS-0083** — Bloquear macros não confiáveis. **A verificar**.
- [ ] **FTS-0084** — Isolar objetos incorporados potencialmente perigosos. **A verificar**.
- [/] **FTS-0085** — Tratar arquivos malformados. **Parcial**.
- [/] **FTS-0086** — Impedir extração fora do diretório ao abrir pacotes ZIP. **Parcial**.
- [ ] **FTS-0087** — Limitar tamanho de recursos incorporados. **Pendente**.
- [ ] **FTS-0088** — Detectar referências externas não confiáveis. **A verificar**.

## 2. Gerenciamento de slides


### 2.1. Operações

- [x] **FTS-0089** — Adicionar slide. **Feito**.
- [x] **FTS-0090** — Inserir slide antes. **Feito** (motor: slide_inserir_em(posição atual)).
- [x] **FTS-0091** — Inserir slide depois. **Feito**.
- [x] **FTS-0092** — Duplicar slide. **Feito**.
- [x] **FTS-0093** — Copiar slide. **Feito** (transferência interna de slides com notas, mídia e formatação).
- [x] **FTS-0094** — Recortar slide. **Feito** (uma transação com desfazer; recusa remover todos os slides).
- [x] **FTS-0095** — Colar slide. **Feito** (insere após o atual, com novos IDs e remapeamento de links internos).
- [x] **FTS-0096** — Excluir slide. **Feito**.
- [x] **FTS-0097** — Mover slide. **Feito**.
- [x] **FTS-0098** — Reordenar slides. **Feito**.
- [x] **FTS-0099** — Selecionar múltiplos slides. **Feito**.
- [x] **FTS-0100** — Selecionar todos os slides. **Feito** (Ctrl+A com foco nas miniaturas; menu contextual).
- [x] **FTS-0101** — Ocultar slide. **Feito**.
- [x] **FTS-0102** — Exibir slide oculto. **Feito**.
- [x] **FTS-0103** — Reutilizar slides de outra apresentação. **Feito** (painel de seleção de páginas de .fsdz/.pptx/.odp).
- [x] **FTS-0104** — Importar slides. **Feito** (painel Arquivos → Importar, adiciona slides de .fsdz/.pptx/.odp).
- [x] **FTS-0105** — Exportar slide individual. **Feito** (PDF do slide atual, acessível no menu da miniatura).
- [x] **FTS-0106** — Localizar slide pelo número. **Feito** (diálogo Ir para slide; valida número antes da navegação).
- [x] **FTS-0107** — Navegar entre slides. **Feito**.
- [x] **FTS-0108** — Localizar slides pelo conteúdo. **Feito**.
- [x] **FTS-0109** — Restaurar layout de slide. **Feito** (reposição de posições/estilos dos espaços reservados, preservando textos).
- [x] **FTS-0110** — Aplicar layout a múltiplos slides. **Feito** (galeria Layout aplicada a todos os slides marcados, um desfazer).
- [x] **FTS-0111** — Preservar formatação original ao colar. **Feito** (opção Manter original com cores/fontes/imagens).
- [x] **FTS-0112** — Adaptar formatação ao tema de destino. **Feito** (opção Adaptar tema com paleta, fontes e decoração de destino).

### 2.2. Estrutura

- [x] **FTS-0113** — Criar seção. **Feito**.
- [x] **FTS-0114** — Renomear seção. **Feito**.
- [x] **FTS-0115** — Mover seção. **Feito** (Menu do cabeçalho: mover seção inteira para cima/baixo, preservando slides e limites; um desfazer.)
- [x] **FTS-0116** — Excluir seção. **Feito**.
- [x] **FTS-0117** — Recolher seção. **Feito**.
- [x] **FTS-0118** — Expandir seção. **Feito**.
- [x] **FTS-0119** — Reordenar seções. **Feito** (Reordenação de blocos inteiros pelo menu da seção, sem alterar os IDs dos slides.)
- [x] **FTS-0120** — Mover slides entre seções. **Feito** (Miniaturas marcadas → menu do cabeçalho da seção destino; preserva IDs e conteúdo.)
- [x] **FTS-0121** — Criar apresentação personalizada. **Feito** (Aba Apresentação → Apresentações personalizadas, criar com slides marcados; .fsdz v9.)
- [x] **FTS-0122** — Definir sequência personalizada de slides. **Feito** (Lista ordenada de IDs persistentes, botões ↑/↓, acrescentar/remover e reprodução na ordem definida.)
- [x] **FTS-0123** — Definir slides ocultos em apresentações personalizadas. **Feito** (Ocultar/exibir somente no roteiro, sem alterar o atributo de ocultação do slide original.)

### 2.3. Configuração

- [x] **FTS-0124** — Tamanho 16:9. **Feito**.
- [x] **FTS-0125** — Tamanho 4:3. **Feito**.
- [x] **FTS-0126** — Tamanho personalizado. **Feito** (Dimensões de 36 a 7200 pt na aba Design.).
- [x] **FTS-0127** — Orientação paisagem. **Feito**.
- [x] **FTS-0128** — Orientação retrato. **Feito** (Troca entre retrato e paisagem no painel.).
- [x] **FTS-0129** — Ajustar conteúdo ao alterar dimensões. **Feito** (Esticar, Ajustar proporcionalmente ou Maximizar; altera todos os slides.).
- [x] **FTS-0130** — Maximizar conteúdo. **Feito** (Escala proporcional para preencher, com possível corte nas bordas.).
- [x] **FTS-0131** — Cor de fundo. **Feito**.
- [x] **FTS-0132** — Imagem de fundo. **Feito** (Imagem local incorporada, esticada à área do slide.).
- [x] **FTS-0133** — Gradiente de fundo. **Feito** (Gradiente horizontal entre duas cores.).
- [x] **FTS-0134** — Textura de fundo. **Feito** (Textura pontilhada vetorial, determinística.).
- [x] **FTS-0135** — Padrão de fundo. **Feito** (Padrão xadrez vetorial em duas cores.).
- [x] **FTS-0136** — Transparência do fundo. **Feito** (Opacidade 0–100% para todos os tipos de fundo.).
- [x] **FTS-0137** — Aplicar fundo a todos os slides. **Feito**.
- [x] **FTS-0138** — Ocultar elementos herdados do mestre. **Feito** (Oculta as formas com Papel::Fundo, sem apagar o mestre/layout.).

## 3. Área de trabalho e interface


### 3.1. Modos de visualização

- [x] **FTS-0139** — Exibição normal. **Feito**.
- [x] **FTS-0140** — Classificador de slides. **Feito**.
- [ ] **FTS-0141** — Estrutura de tópicos. **A verificar**.
- [ ] **FTS-0142** — Página de anotações. **A verificar**.
- [ ] **FTS-0143** — Modo de leitura. **Pendente**.
- [x] **FTS-0144** — Apresentação em tela cheia. **Feito**.
- [/] **FTS-0145** — Exibição do apresentador. **Parcial**.
- [ ] **FTS-0146** — Slide Mestre. **A verificar**.
- [ ] **FTS-0147** — Folheto Mestre. **A verificar**.
- [ ] **FTS-0148** — Anotações Mestras. **A verificar**.

### 3.2. Interface

- [x] **FTS-0149** — Faixa de opções. **Feito**.
- [ ] **FTS-0150** — Barra de ferramentas de acesso rápido. **A verificar**.
- [x] **FTS-0151** — Barra de título. **Feito**.
- [x] **FTS-0152** — Barra de status. **Feito**.
- [x] **FTS-0153** — Painel de miniaturas. **Feito**.
- [x] **FTS-0154** — Painel de anotações. **Feito**.
- [x] **FTS-0155** — Painel de seleção. **Feito**.
- [x] **FTS-0156** — Painel de animações. **Feito**.
- [/] **FTS-0157** — Painel de formatação. **Parcial**.
- [ ] **FTS-0158** — Painel de comentários. **A verificar**.
- [ ] **FTS-0159** — Painel de acessibilidade. **A verificar**.
- [x] **FTS-0160** — Menu contextual. **Feito**.
- [x] **FTS-0161** — Menus de ferramentas contextuais. **Feito**.
- [ ] **FTS-0162** — Pesquisa de comandos. **Pendente**.
- [ ] **FTS-0163** — Personalização da barra de ferramentas. **A verificar**.
- [x] **FTS-0164** — Personalização de atalhos. **Feito**.
- [ ] **FTS-0165** — Interface compacta. **A verificar**.
- [x] **FTS-0166** — Modo claro. **Feito**.
- [x] **FTS-0167** — Modo escuro. **Feito**.
- [ ] **FTS-0168** — Redimensionamento de painéis. **A verificar**.
- [x] **FTS-0169** — Zoom da área de edição. **Feito**.
- [x] **FTS-0170** — Ajustar slide à janela. **Feito**.
- [x] **FTS-0171** — Rolagem horizontal e vertical. **Feito**.
- [/] **FTS-0172** — Exibição em múltiplos monitores. **Parcial**.
- [ ] **FTS-0173** — Múltiplas janelas para o mesmo arquivo. **A verificar**.

### 3.3. Auxiliares de edição

- [x] **FTS-0174** — Régua horizontal. **Feito**.
- [x] **FTS-0175** — Régua vertical. **Feito**.
- [x] **FTS-0176** — Guias. **Feito**.
- [ ] **FTS-0177** — Guias inteligentes. **A verificar**.
- [x] **FTS-0178** — Grade. **Feito**.
- [ ] **FTS-0179** — Ajustar à grade. **A verificar**.
- [ ] **FTS-0180** — Ajustar às guias. **A verificar**.
- [ ] **FTS-0181** — Ajustar aos objetos. **A verificar**.
- [x] **FTS-0182** — Exibir limites dos objetos. **Feito**.
- [ ] **FTS-0183** — Exibir marcadores de posição. **A verificar**.
- [x] **FTS-0184** — Alinhar objetos. **Feito**.
- [x] **FTS-0185** — Distribuir objetos. **Feito**.
- [ ] **FTS-0186** — Mostrar distância entre objetos. **A verificar**.
- [ ] **FTS-0187** — Mostrar dimensões. **A verificar**.
- [ ] **FTS-0188** — Coordenadas de posição. **A verificar**.
- [x] **FTS-0189** — Seleção por área. **Feito**.
- [x] **FTS-0190** — Seleção múltipla. **Feito**.
- [ ] **FTS-0191** — Seleção de objetos sobrepostos. **A verificar**.
- [ ] **FTS-0192** — Bloqueio de objetos. **A verificar**.
- [x] **FTS-0193** — Agrupamento. **Feito**.
- [x] **FTS-0194** — Desagrupamento. **Feito**.

## 4. Edição de texto


### 4.1. Inserção

- [x] **FTS-0195** — Caixa de texto. **Feito**.
- [x] **FTS-0196** — Texto em formas. **Feito**.
- [x] **FTS-0197** — Texto em marcadores de posição. **Feito**.
- [/] **FTS-0198** — WordArt. **Parcial**.
- [ ] **FTS-0199** — Texto vertical. **A verificar**.
- [x] **FTS-0200** — Texto horizontal. **Feito**.
- [x] **FTS-0201** — Texto rotacionado. **Feito**.
- [ ] **FTS-0202** — Texto empilhado. **A verificar**.
- [x] **FTS-0203** — Texto em tabelas. **Feito**.
- [ ] **FTS-0204** — Texto em SmartArt. **A verificar**.
- [ ] **FTS-0205** — Texto em gráficos. **A verificar**.
- [x] **FTS-0206** — Inserção de símbolos. **Feito**.
- [x] **FTS-0207** — Inserção de caracteres Unicode. **Feito**.
- [/] **FTS-0208** — Inserção de equações. **Parcial**.

### 4.2. Formatação de caracteres

- [x] **FTS-0209** — Fonte. **Feito**.
- [x] **FTS-0210** — Família tipográfica. **Feito**.
- [x] **FTS-0211** — Tamanho da fonte. **Feito**.
- [x] **FTS-0212** — Negrito. **Feito**.
- [x] **FTS-0213** — Itálico. **Feito**.
- [x] **FTS-0214** — Sublinhado. **Feito**.
- [/] **FTS-0215** — Sublinhado duplo. **Parcial** (botão na Fonte; motor, .fsdz, palco, PDF e leitura .pptx/.odp; aceite visual pendente).
- [x] **FTS-0216** — Tachado. **Feito**.
- [/] **FTS-0217** — Tachado duplo. **Parcial** (botão na Fonte; motor, .fsdz, palco, PDF e leitura .pptx/.odp; aceite visual pendente).
- [/] **FTS-0218** — Sobrescrito. **Parcial** (por trecho em edição ou caixa inteira; .fsdz e PDF; aceite visual pendente).
- [/] **FTS-0219** — Subscrito. **Parcial** (por trecho em edição ou caixa inteira; .fsdz e PDF; aceite visual pendente).
- [/] **FTS-0220** — Maiúsculas. **Parcial** (menu Aa: trecho em edição ou caixa inteira, um passo de desfazer; aceite visual pendente).
- [/] **FTS-0221** — Minúsculas. **Parcial** (menu Aa; aceite visual pendente).
- [/] **FTS-0222** — Capitalização. **Parcial** (menu Aa: primeira letra da frase, cada palavra e alternar; aceite visual pendente).
- [/] **FTS-0223** — Espaçamento entre caracteres. **Parcial** (cinco níveis (-2 a 4 pt) por caixa; sem kerning; aceite visual pendente).
- [ ] **FTS-0224** — Kerning. **Pendente**.
- [x] **FTS-0225** — Cor. **Feito**.
- [ ] **FTS-0226** — Transparência. **A verificar**.
- [x] **FTS-0227** — Realce. **Feito**.
- [ ] **FTS-0228** — Sombra. **A verificar**.
- [ ] **FTS-0229** — Contorno. **A verificar**.
- [ ] **FTS-0230** — Preenchimento gradiente. **Pendente**.
- [ ] **FTS-0231** — Efeitos de texto. **A verificar**.
- [x] **FTS-0232** — Limpar formatação. **Feito**.
- [ ] **FTS-0233** — Substituir fontes. **A verificar**.
- [/] **FTS-0234** — Incorporar fontes ao documento. **Parcial**.

### 4.3. Formatação de parágrafos

- [x] **FTS-0235** — Alinhamento à esquerda. **Feito**.
- [x] **FTS-0236** — Centralizar. **Feito**.
- [x] **FTS-0237** — Alinhamento à direita. **Feito**.
- [x] **FTS-0238** — Justificar. **Feito**.
- [ ] **FTS-0239** — Recuo esquerdo. **A verificar**.
- [ ] **FTS-0240** — Recuo direito. **A verificar**.
- [ ] **FTS-0241** — Recuo da primeira linha. **A verificar**.
- [ ] **FTS-0242** — Recuo suspenso. **A verificar**.
- [ ] **FTS-0243** — Espaçamento antes do parágrafo. **A verificar**.
- [ ] **FTS-0244** — Espaçamento depois do parágrafo. **A verificar**.
- [x] **FTS-0245** — Espaçamento entre linhas. **Feito**.
- [ ] **FTS-0246** — Tabulações. **A verificar**.
- [x] **FTS-0247** — Marcadores. **Feito**.
- [x] **FTS-0248** — Numeração. **Feito**.
- [ ] **FTS-0249** — Marcadores personalizados. **A verificar**.
- [ ] **FTS-0250** — Imagens como marcadores. **A verificar**.
- [/] **FTS-0251** — Alterar nível de lista. **Parcial** (Aumentar/Diminuir recuo na faixa: linhas do trecho em edição ou caixa inteira, quatro espaços por nível; aceite visual pendente).
- [x] **FTS-0252** — Dividir em colunas. **Feito**.
- [x] **FTS-0253** — Direção do texto. **Feito**.
- [ ] **FTS-0254** — Espaçamento entre colunas. **A verificar**.
- [x] **FTS-0255** — Alinhamento vertical. **Feito**.

### 4.4. Comportamento da caixa de texto

- [ ] **FTS-0256** — Ajustar texto automaticamente. **A verificar**.
- [ ] **FTS-0257** — Reduzir texto quando ultrapassa os limites. **A verificar**.
- [ ] **FTS-0258** — Redimensionar forma para ajustar ao texto. **A verificar**.
- [ ] **FTS-0259** — Não ajustar automaticamente. **A verificar**.
- [ ] **FTS-0260** — Definir margens internas. **A verificar**.
- [x] **FTS-0261** — Quebra automática de linha. **Feito**.
- [x] **FTS-0262** — Rotação do texto. **Feito**.
- [x] **FTS-0263** — Alinhamento interno. **Feito**.
- [ ] **FTS-0264** — Controle de transbordamento. **A verificar**.
- [x] **FTS-0265** — Controle de espaçamento entre linhas. **Feito**.
- [ ] **FTS-0266** — Controle de layout para escrita bidirecional. **A verificar**.

### 4.5. Ferramentas

- [x] **FTS-0267** — Localizar. **Feito**.
- [x] **FTS-0268** — Substituir. **Feito**.
- [x] **FTS-0269** — Substituir todas as ocorrências. **Feito**.
- [x] **FTS-0270** — Verificação ortográfica. **Feito** (Revisão F7 de caixas de texto, notas e células de tabelas; requer dicionário Hunspell local).
- [x] **FTS-0271** — Dicionário. **Feito** (Consulta ortográfica, sugestões, dicionário pessoal e instalação de Hunspell; não é um dicionário de definições).
- [/] **FTS-0272** — Sinônimos. **Parcial** (Vocabulário local inicial em português, editável; aplicação no trecho selecionado com desfazer. Sem léxico amplo instalado).
- [x] **FTS-0273** — Autocorreção. **Feito** (Regras locais editáveis, opcionais; corrige ao terminar palavras na digitação de caixas e células).
- [x] **FTS-0274** — Correção automática de capitalização. **Feito** (Opção para capitalizar início de frase durante a digitação; usa regras de pontuação).
- [/] **FTS-0275** — Idioma de revisão. **Parcial** (Idioma principal e dicionários locais configuráveis; escolha global, sem idioma por trecho).
- [/] **FTS-0276** — Tradução. **Parcial** (Tradução exata por glossário local editável e par de idiomas; sem modelo para frases livres).
- [x] **FTS-0277** — Contagem de palavras. **Feito** (Palavras e caracteres dos slides, tabelas, rótulos de gráficos e trecho selecionado; notas, fundos e comentários ficam fora desta contagem).
- [x] **FTS-0278** — Pesquisa de texto na apresentação. **Feito**.
- [x] **FTS-0279** — Colar sem formatação. **Feito** (Texto simples em caixa nova ou no texto/célula em edição; Ctrl+Shift+V).
- [/] **FTS-0280** — Colar com formatação original. **Parcial** (Preserva estilo e elementos da cópia interna; formatos ricos de outros programas não são importados).
- [x] **FTS-0281** — Pincel de formatação. **Feito** (Captura estilo do texto, preenchimento, contorno e sombra; clique no destino aplica sem alterar conteúdo ou geometria; Esc cancela).

## 5. Formas geométricas e desenho vetorial


### 5.1. Formas básicas

- [x] **FTS-0282** — Retângulo. **Feito**.
- [x] **FTS-0283** — Retângulo arredondado. **Feito**.
- [x] **FTS-0284** — Elipse. **Feito**.
- [x] **FTS-0285** — Círculo. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).
- [x] **FTS-0286** — Triângulo. **Feito**.
- [x] **FTS-0287** — Triângulo retângulo. **Feito** (forma vetorial nativa, editável e exportável).
- [x] **FTS-0288** — Losango. **Feito**.
- [x] **FTS-0289** — Paralelogramo. **Feito**.
- [x] **FTS-0290** — Trapézio. **Feito**.
- [x] **FTS-0291** — Pentágono. **Feito**.
- [x] **FTS-0292** — Hexágono. **Feito**.
- [x] **FTS-0293** — Heptágono. **Feito** (forma vetorial nativa, editável e exportável).
- [x] **FTS-0294** — Octógono. **Feito**.
- [x] **FTS-0295** — Decágono. **Feito** (forma vetorial nativa, editável e exportável).
- [x] **FTS-0296** — Dodecágono. **Feito** (forma vetorial nativa, editável e exportável).
- [x] **FTS-0297** — Cruz. **Feito**.
- [x] **FTS-0298** — Anel. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).
- [x] **FTS-0299** — Arco. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).
- [x] **FTS-0300** — Corda. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).
- [x] **FTS-0301** — Pizza. **Feito**.
- [x] **FTS-0302** — Cubo. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).
- [x] **FTS-0303** — Cilindro. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).
- [x] **FTS-0304** — Moldura. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).
- [x] **FTS-0305** — Canto em L. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).
- [x] **FTS-0306** — Placa. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).
- [x] **FTS-0307** — Sol. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).
- [x] **FTS-0308** — Lua. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).
- [x] **FTS-0309** — Coração. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).
- [x] **FTS-0310** — Raio. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).
- [x] **FTS-0311** — Nuvem. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).
- [x] **FTS-0312** — Estrelas com diferentes números de pontas. **Feito** (vetor nativo Rust, inserção na galeria, arquivo .fsdz e exportação PDF).

### 5.2. Setas

- [x] **FTS-0313** — Seta simples. **Feito**.
- [x] **FTS-0314** — Seta dupla. **Feito**.
- [ ] **FTS-0315** — Seta quádrupla. **Pendente**.
- [ ] **FTS-0316** — Seta circular. **Pendente**.
- [ ] **FTS-0317** — Seta curva. **Pendente**.
- [ ] **FTS-0318** — Seta em U. **Pendente**.
- [ ] **FTS-0319** — Seta em L. **Pendente**.
- [x] **FTS-0320** — Seta em chevron. **Feito**.
- [x] **FTS-0321** — Seta para cima. **Feito**.
- [x] **FTS-0322** — Seta para baixo. **Feito**.
- [x] **FTS-0323** — Seta para esquerda. **Feito**.
- [x] **FTS-0324** — Seta para direita. **Feito**.
- [/] **FTS-0325** — Setas de bloco. **Parcial**.
- [ ] **FTS-0326** — Setas de fluxo. **Pendente**.

### 5.3. Outras galerias

- [x] **FTS-0327** — Fluxogramas. **Feito** — galeria vetorial nativa; ver evidências em `VERIFICACAO-FUNCIONALIDADES.md`.
- [x] **FTS-0328** — Símbolos de decisão. **Feito** — galeria vetorial nativa; ver evidências em `VERIFICACAO-FUNCIONALIDADES.md`.
- [x] **FTS-0329** — Símbolos de processamento. **Feito** — galeria vetorial nativa; ver evidências em `VERIFICACAO-FUNCIONALIDADES.md`.
- [x] **FTS-0330** — Símbolos de armazenamento. **Feito** — galeria vetorial nativa; ver evidências em `VERIFICACAO-FUNCIONALIDADES.md`.
- [x] **FTS-0331** — Conectores. **Feito** — galeria vetorial nativa; ver evidências em `VERIFICACAO-FUNCIONALIDADES.md`.
- [x] **FTS-0332** — Balões de fala. **Feito** — galeria vetorial nativa; ver evidências em `VERIFICACAO-FUNCIONALIDADES.md`.
- [x] **FTS-0333** — Balões de pensamento. **Feito** — galeria vetorial nativa; ver evidências em `VERIFICACAO-FUNCIONALIDADES.md`.
- [x] **FTS-0334** — Faixas. **Feito** — galeria vetorial nativa; ver evidências em `VERIFICACAO-FUNCIONALIDADES.md`.
- [x] **FTS-0335** — Fitas. **Feito** — galeria vetorial nativa; ver evidências em `VERIFICACAO-FUNCIONALIDADES.md`.
- [x] **FTS-0336** — Ondas. **Feito** — galeria vetorial nativa; ver evidências em `VERIFICACAO-FUNCIONALIDADES.md`.
- [x] **FTS-0337** — Explosões. **Feito** — galeria vetorial nativa; ver evidências em `VERIFICACAO-FUNCIONALIDADES.md`.
- [x] **FTS-0338** — Formas matemáticas. **Feito** — galeria vetorial nativa; ver evidências em `VERIFICACAO-FUNCIONALIDADES.md`.
- [x] **FTS-0339** — Formas de ação. **Feito** — galeria vetorial nativa; ver evidências em `VERIFICACAO-FUNCIONALIDADES.md`.
- [x] **FTS-0340** — Formas livres. **Feito** — galeria vetorial nativa; ver evidências em `VERIFICACAO-FUNCIONALIDADES.md`.

### 5.4. Manipulação

- [x] **FTS-0341** — Criar forma. **Feito**.
- [x] **FTS-0342** — Excluir forma. **Feito**.
- [x] **FTS-0343** — Duplicar forma. **Feito**.
- [x] **FTS-0344** — Mover. **Feito**.
- [x] **FTS-0345** — Redimensionar. **Feito**.
- [x] **FTS-0346** — Rotacionar. **Feito**.
- [ ] **FTS-0347** — Inverter horizontalmente. **A verificar**.
- [ ] **FTS-0348** — Inverter verticalmente. **A verificar**.
- [ ] **FTS-0349** — Editar pontos. **Pendente**.
- [ ] **FTS-0350** — Adicionar ponto. **Pendente**.
- [ ] **FTS-0351** — Remover ponto. **Pendente**.
- [ ] **FTS-0352** — Converter segmento em curva. **Pendente**.
- [ ] **FTS-0353** — Converter segmento em linha reta. **Pendente**.
- [ ] **FTS-0354** — Ajustar alças Bézier. **Pendente**.
- [ ] **FTS-0355** — Alterar tipo de forma. **A verificar**.
- [x] **FTS-0356** — Modificar proporção. **Feito**.
- [x] **FTS-0357** — Ajustar cantos arredondados. **Feito**.
- [x] **FTS-0358** — Agrupar formas. **Feito**.
- [x] **FTS-0359** — Desagrupar formas. **Feito**.
- [ ] **FTS-0360** — Reagrupar formas. **A verificar**.
- [x] **FTS-0361** — Ordenar camadas. **Feito**.
- [ ] **FTS-0362** — Definir forma padrão. **A verificar**.

### 5.5. Operações booleanas

- [x] **FTS-0363** — União. **Feito** (motor Rust; formas fechadas, contornos vetoriais e curvas aproximadas).
- [x] **FTS-0364** — Interseção. **Feito** (motor Rust; formas fechadas, contornos vetoriais e curvas aproximadas).
- [x] **FTS-0365** — Subtração. **Feito** (motor Rust; formas fechadas, contornos vetoriais e curvas aproximadas).
- [x] **FTS-0366** — Combinação. **Feito** (motor Rust; formas fechadas, contornos vetoriais e curvas aproximadas).
- [x] **FTS-0367** — Fragmentação. **Feito** (motor Rust; formas fechadas, contornos vetoriais e curvas aproximadas).

### 5.6. Preenchimento

- [x] **FTS-0368** — Sem preenchimento. **Feito**.
- [x] **FTS-0369** — Cor sólida. **Feito**.
- [x] **FTS-0370** — Transparência. **Feito**.
- [ ] **FTS-0371** — Gradiente linear. **Pendente**.
- [ ] **FTS-0372** — Gradiente radial. **Pendente**.
- [ ] **FTS-0373** — Gradiente retangular. **Pendente**.
- [ ] **FTS-0374** — Gradiente de caminho. **Pendente**.
- [ ] **FTS-0375** — Múltiplas paradas de gradiente. **Pendente**.
- [ ] **FTS-0376** — Imagem como preenchimento. **A verificar**.
- [ ] **FTS-0377** — Textura. **Pendente**.
- [ ] **FTS-0378** — Padrões. **Pendente**.
- [ ] **FTS-0379** — Cor do tema. **A verificar**.
- [x] **FTS-0380** — Cor personalizada. **Feito**.
- [x] **FTS-0381** — Conta-gotas. **Feito**.

### 5.7. Contornos

- [x] **FTS-0382** — Cor. **Feito**.
- [x] **FTS-0383** — Espessura. **Feito**.
- [ ] **FTS-0384** — Transparência. **A verificar**.
- [x] **FTS-0385** — Linha contínua. **Feito**.
- [ ] **FTS-0386** — Linha tracejada. **A verificar**.
- [ ] **FTS-0387** — Linha pontilhada. **A verificar**.
- [ ] **FTS-0388** — Estilos compostos. **A verificar**.
- [ ] **FTS-0389** — Extremidades arredondadas. **A verificar**.
- [ ] **FTS-0390** — Extremidades quadradas. **A verificar**.
- [ ] **FTS-0391** — Junções de linha. **A verificar**.
- [ ] **FTS-0392** — Setas nas extremidades. **A verificar**.
- [ ] **FTS-0393** — Estilos de conectores. **A verificar**.

### 5.8. Efeitos

- [/] **FTS-0394** — Sombra externa. **Parcial**.
- [ ] **FTS-0395** — Sombra interna. **A verificar**.
- [ ] **FTS-0396** — Reflexo. **A verificar**.
- [ ] **FTS-0397** — Brilho. **A verificar**.
- [ ] **FTS-0398** — Bordas suaves. **A verificar**.
- [ ] **FTS-0399** — Chanfro. **A verificar**.
- [ ] **FTS-0400** — Extrusão 3D. **A verificar**.
- [ ] **FTS-0401** — Rotação 3D. **A verificar**.
- [ ] **FTS-0402** — Materiais. **A verificar**.
- [ ] **FTS-0403** — Iluminação. **A verificar**.
- [ ] **FTS-0404** — Profundidade. **A verificar**.
- [ ] **FTS-0405** — Perspectiva. **A verificar**.

## 6. Imagens e gráficos vetoriais


### 6.1. Inserção

- [x] **FTS-0406** — Imagem local. **Feito**.
- [ ] **FTS-0407** — Imagem online. **Cancelado**.
- [ ] **FTS-0408** — Imagem de biblioteca. **Cancelado**.
- [ ] **FTS-0409** — Imagem da área de transferência. **A verificar**.
- [ ] **FTS-0410** — Captura de tela. **A verificar**.
- [ ] **FTS-0411** — Recorte de tela. **A verificar**.
- [ ] **FTS-0412** — Álbum de fotografias. **Cancelado**.
- [x] **FTS-0413** — Ícones. **Feito**.
- [/] **FTS-0414** — SVG. **Parcial**.
- [ ] **FTS-0415** — Imagens vinculadas. **A verificar**.
- [x] **FTS-0416** — Imagens incorporadas. **Feito**.

### 6.2. Manipulação

- [x] **FTS-0417** — Redimensionar. **Feito** (Redimensionamento nativo com alças sem recompressão.).
- [x] **FTS-0418** — Cortar. **Feito** (Corte por quatro margens, arraste e zoom, sem destruir o arquivo fonte.).
- [x] **FTS-0419** — Cortar para forma. **Feito** (Recorte para forma com máscara alfa.).
- [x] **FTS-0420** — Ajustar proporção. **Feito** (Proporção da caixa com enquadramento da imagem.).
- [x] **FTS-0421** — Preencher área. **Feito** (Modo Preencher ocupa a caixa cortando excedentes.).
- [x] **FTS-0422** — Ajustar imagem à área. **Feito** (Modo Ajustar exibe toda a imagem com espaços transparentes.).
- [x] **FTS-0423** — Girar. **Feito** (Rotação do elemento de imagem.).
- [x] **FTS-0424** — Inverter. **Feito** (Espelhamento horizontal e vertical não destrutivo.).
- [/] **FTS-0425** — Remover fundo. **Parcial** (Cinco modos de remover fundo (cor, borda, LAB, watershed e marcas); sem segmentação semântica IA.).
- [x] **FTS-0426** — Definir cor transparente. **Feito** (Cor transparente exata: pixels com RGB idêntico à cor escolhida tornam-se transparentes; a remoção com tolerância permanece disponível.).
- [x] **FTS-0427** — Substituir imagem. **Feito** (Substituir imagem preservando posição, tamanho, máscara e efeitos.).
- [x] **FTS-0428** — Redefinir imagem. **Feito** (Restaurar enquadramento e redefinir ajustes de foto sem descartar original.).
- [x] **FTS-0429** — Compactar imagem. **Feito** (Compactar JPEG qualidade 60/85 ou PNG com alfa; Ctrl+Z recupera a anterior.).
- [x] **FTS-0430** — Reduzir resolução. **Feito** (Reduzir para até 1280 ou 800 px com Lanczos3.).
- [x] **FTS-0431** — Preservar qualidade original. **Feito** (Bytes originais preservados até redução ou compactação voluntária.).
- [x] **FTS-0432** — Ajustar transparência. **Feito** (Opacidade 0..100% na tela e PDF.).
- [x] **FTS-0433** — Ajustar brilho. **Feito** (Brilho RGB -100..100%.).
- [x] **FTS-0434** — Ajustar contraste. **Feito** (Contraste RGB -100..100%.).
- [x] **FTS-0435** — Ajustar nitidez. **Feito** (Máscara unsharp de nitidez 0..100%.).
- [x] **FTS-0436** — Ajustar saturação. **Feito** (Saturação 0..200% sobre RGBA.).
- [x] **FTS-0437** — Ajustar temperatura de cor. **Feito** (Temperatura fria/quente por balanço de cor.).
- [x] **FTS-0438** — Recolorir. **Feito** (Recolorir por matiz YIQ.).
- [x] **FTS-0439** — Aplicar filtros artísticos. **Feito** (Cinza, sépia, negativo, posterizar e solarizar.).
- [x] **FTS-0440** — Adicionar borda. **Feito** (Borda colorida e espessura na aba Formato da Imagem.).
- [x] **FTS-0441** — Adicionar sombra. **Feito** (Sombra suavizada, deslocada, renderizada no Iced e PDF.).
- [x] **FTS-0442** — Adicionar reflexo. **Feito** (Reflexo projetado abaixo da imagem com desvanecimento, na tela Iced e no PDF.).
- [x] **FTS-0443** — Adicionar brilho. **Feito** (Halo suavizado projetado fora do retângulo da foto, na tela Iced e no PDF.).
- [x] **FTS-0444** — Adicionar perspectiva. **Feito** (Perspectiva trapezoidal 2D com ajuste reversível; não é rotação de câmera 3D.).
- [x] **FTS-0445** — Inserir texto alternativo. **Feito** (Texto alternativo editável e salvo no .fsdz; PDF sem tags estruturadas.).

### 6.3. Vetores

- [/] **FTS-0446** — Importar SVG. **Parcial**.
- [/] **FTS-0447** — Renderizar SVG. **Parcial**.
- [x] **FTS-0448** — Preservar vetores. **Feito**.
- [/] **FTS-0449** — Converter SVG em elementos editáveis. **Parcial**.
- [x] **FTS-0450** — Editar preenchimentos vetoriais. **Feito**.
- [x] **FTS-0451** — Editar contornos. **Feito**.
- [x] **FTS-0452** — Redimensionar sem perda. **Feito**.
- [x] **FTS-0453** — Exportar para SVG. **Feito**.

Importação e conversão implementadas para geometria SVG com cores sólidas, curvas,
grupos, transformações e regras de preenchimento. Gradientes, padrões, filtros,
máscaras, imagens, texto, opacidade de grupo e traços especiais ainda são recusados
com mensagem explícita; por isso FTS-0446/0447/0449 continuam `[/]`.
Inserir → SVG; Formato da Forma → Vetores libera as peças e exporta a seleção.

## 7. Tabelas


### 7.1. Estrutura

- [x] **FTS-0454** — Inserir tabela. **Feito**.
- [x] **FTS-0455** — Excluir tabela. **Feito**.
- [x] **FTS-0456** — Inserir linha. **Feito**.
- [x] **FTS-0457** — Inserir coluna. **Feito**.
- [x] **FTS-0458** — Excluir linha. **Feito**.
- [x] **FTS-0459** — Excluir coluna. **Feito**.
- [ ] **FTS-0460** — Mesclar células. **Pendente**.
- [ ] **FTS-0461** — Dividir células. **Pendente**.
- [ ] **FTS-0462** — Redimensionar linhas. **Pendente**.
- [ ] **FTS-0463** — Redimensionar colunas. **Pendente**.
- [ ] **FTS-0464** — Distribuir linhas. **A verificar**.
- [ ] **FTS-0465** — Distribuir colunas. **A verificar**.
- [x] **FTS-0466** — Ajustar dimensões da tabela. **Feito**.
- [x] **FTS-0467** — Alterar largura das bordas. **Feito** (Inserir → Bordas da tabela: passos de 0,5 pt entre 0 e 50 pt; zero oculta as bordas. Desfazer/refazer e persistência .fsdz.).
- [x] **FTS-0468** — Selecionar célula. **Feito**.
- [ ] **FTS-0469** — Selecionar linha. **A verificar**.
- [ ] **FTS-0470** — Selecionar coluna. **A verificar**.
- [x] **FTS-0471** — Selecionar tabela inteira. **Feito**.
- [x] **FTS-0472** — Mover tabela. **Feito**.
- [x] **FTS-0473** — Duplicar tabela. **Feito**.
- [ ] **FTS-0474** — Copiar tabela de outro documento. **A verificar**.

### 7.2. Formatação

- [ ] **FTS-0475** — Estilos predefinidos. **A verificar**.
- [ ] **FTS-0476** — Estilos personalizados. **A verificar**.
- [/] **FTS-0477** — Linha de cabeçalho. **Parcial**.
- [ ] **FTS-0478** — Linha de totais. **A verificar**.
- [ ] **FTS-0479** — Primeira coluna destacada. **A verificar**.
- [ ] **FTS-0480** — Última coluna destacada. **A verificar**.
- [/] **FTS-0481** — Linhas alternadas. **Parcial**.
- [ ] **FTS-0482** — Colunas alternadas. **A verificar**.
- [ ] **FTS-0483** — Preenchimento de células. **A verificar**.
- [ ] **FTS-0484** — Bordas individuais. **A verificar**.
- [ ] **FTS-0485** — Bordas externas. **A verificar**.
- [ ] **FTS-0486** — Bordas internas. **A verificar**.
- [ ] **FTS-0487** — Remover bordas. **A verificar**.
- [ ] **FTS-0488** — Alinhamento horizontal. **A verificar**.
- [ ] **FTS-0489** — Alinhamento vertical. **A verificar**.
- [ ] **FTS-0490** — Margens internas. **A verificar**.
- [ ] **FTS-0491** — Direção do texto. **A verificar**.
- [/] **FTS-0492** — Formatação de fonte. **Parcial**.
- [ ] **FTS-0493** — Efeitos de célula. **A verificar**.

### 7.3. Integração

- [ ] **FTS-0494** — Colar tabelas do Excel. **A verificar**.
- [ ] **FTS-0495** — Incorporar planilhas Excel. **A verificar**.
- [ ] **FTS-0496** — Vincular tabelas externas. **A verificar**.
- [ ] **FTS-0497** — Atualizar conteúdo vinculado. **A verificar**.
- [ ] **FTS-0498** — Converter tabelas em formas. **A verificar**.
- [ ] **FTS-0499** — Exportar tabelas como imagens. **A verificar**.

## 8. Gráficos de dados


### 8.1. Tipos — colunas

- [x] **FTS-0500** — Colunas agrupadas. **Feito**.
- [ ] **FTS-0501** — Colunas empilhadas. **Pendente**.
- [ ] **FTS-0502** — Colunas 100% empilhadas. **Pendente**.
- [ ] **FTS-0503** — Colunas 3D. **Pendente**.

### 8.1. Tipos — barras

- [x] **FTS-0504** — Barras agrupadas. **Feito**.
- [ ] **FTS-0505** — Barras empilhadas. **Pendente**.
- [ ] **FTS-0506** — Barras 100% empilhadas. **Pendente**.
- [ ] **FTS-0507** — Barras 3D. **Pendente**.

### 8.1. Tipos — linhas

- [x] **FTS-0508** — Linhas simples. **Feito**.
- [ ] **FTS-0509** — Linhas com marcadores. **Pendente**.
- [ ] **FTS-0510** — Linhas empilhadas. **Pendente**.
- [ ] **FTS-0511** — Linhas 100% empilhadas. **Pendente**.
- [ ] **FTS-0512** — Linhas 3D. **Pendente**.

### 8.1. Tipos — áreas

- [ ] **FTS-0513** — Área simples. **Pendente**.
- [ ] **FTS-0514** — Área empilhada. **Pendente**.
- [ ] **FTS-0515** — Área 100% empilhada. **Pendente**.
- [ ] **FTS-0516** — Área 3D. **Pendente**.

### 8.1. Tipos — pizza e rosca

- [x] **FTS-0517** — Pizza. **Feito**.
- [ ] **FTS-0518** — Pizza 3D. **Pendente**.
- [ ] **FTS-0519** — Pizza destacada. **Pendente**.
- [ ] **FTS-0520** — Pizza de pizza. **Pendente**.
- [ ] **FTS-0521** — Barra de pizza. **Pendente**.
- [ ] **FTS-0522** — Rosca. **Pendente**.

### 8.1. Tipos — dispersão

- [ ] **FTS-0523** — Dispersão XY. **Pendente**.
- [ ] **FTS-0524** — Dispersão com linhas. **Pendente**.
- [ ] **FTS-0525** — Dispersão com marcadores. **Pendente**.
- [ ] **FTS-0526** — Dispersão com linhas suaves. **Pendente**.

### 8.1. Tipos — estatísticos

- [ ] **FTS-0527** — Histograma. **Pendente**.
- [ ] **FTS-0528** — Pareto. **Pendente**.
- [ ] **FTS-0529** — Caixa e bigodes. **Pendente**.
- [ ] **FTS-0530** — Cascata. **Pendente**.

### 8.1. Tipos — hierárquicos

- [ ] **FTS-0531** — Mapa de árvore. **Pendente**.
- [ ] **FTS-0532** — Explosão solar. **Pendente**.

### 8.1. Tipos — financeiros

- [ ] **FTS-0533** — Máxima, mínima e fechamento. **Pendente**.
- [ ] **FTS-0534** — Abertura, máxima, mínima e fechamento. **Pendente**.
- [ ] **FTS-0535** — Variações com volume. **Pendente**.

### 8.1. Tipos — outros

- [ ] **FTS-0536** — Bolhas. **Pendente**.
- [ ] **FTS-0537** — Bolhas 3D. **Pendente**.
- [ ] **FTS-0538** — Radar. **Pendente**.
- [ ] **FTS-0539** — Radar preenchido. **Pendente**.
- [ ] **FTS-0540** — Superfície. **Pendente**.
- [ ] **FTS-0541** — Contorno. **Pendente**.
- [ ] **FTS-0542** — Funil. **Pendente**.
- [ ] **FTS-0543** — Combinação. **Pendente**.
- [ ] **FTS-0544** — Mapa geográfico. **Pendente**.

### 8.2. Configurações dos gráficos

- [/] **FTS-0545** — Adicionar séries. **Parcial**.
- [/] **FTS-0546** — Remover séries. **Parcial**.
- [/] **FTS-0547** — Renomear séries. **Parcial**.
- [/] **FTS-0548** — Alterar dados. **Parcial**.
- [/] **FTS-0549** — Editar dados em planilha. **Parcial**.
- [ ] **FTS-0550** — Vincular dados externos. **A verificar**.
- [ ] **FTS-0551** — Atualizar vínculos. **A verificar**.
- [ ] **FTS-0552** — Alternar linhas e colunas. **A verificar**.
- [/] **FTS-0553** — Alterar tipo de gráfico. **Parcial**.
- [ ] **FTS-0554** — Criar gráfico combinado. **A verificar**.
- [ ] **FTS-0555** — Configurar eixo primário. **A verificar**.
- [ ] **FTS-0556** — Configurar eixo secundário. **A verificar**.
- [ ] **FTS-0557** — Configurar escala. **A verificar**.
- [ ] **FTS-0558** — Configurar limites dos eixos. **A verificar**.
- [ ] **FTS-0559** — Configurar unidades. **A verificar**.
- [ ] **FTS-0560** — Configurar eixo logarítmico. **A verificar**.
- [/] **FTS-0561** — Configurar categorias. **Parcial**.
- [ ] **FTS-0562** — Configurar rótulos. **A verificar**.
- [ ] **FTS-0563** — Configurar legenda. **A verificar**.
- [/] **FTS-0564** — Configurar título. **Parcial**.
- [ ] **FTS-0565** — Configurar títulos dos eixos. **A verificar**.
- [ ] **FTS-0566** — Configurar linhas de grade. **A verificar**.
- [ ] **FTS-0567** — Configurar barras de erro. **A verificar**.
- [ ] **FTS-0568** — Configurar linhas de tendência. **A verificar**.
- [ ] **FTS-0569** — Configurar equações de tendência. **A verificar**.
- [ ] **FTS-0570** — Exibir coeficiente R². **A verificar**.
- [ ] **FTS-0571** — Configurar marcadores. **A verificar**.
- [ ] **FTS-0572** — Configurar preenchimentos. **A verificar**.
- [ ] **FTS-0573** — Configurar bordas. **A verificar**.
- [ ] **FTS-0574** — Configurar área de plotagem. **A verificar**.
- [ ] **FTS-0575** — Configurar área do gráfico. **A verificar**.
- [ ] **FTS-0576** — Configurar profundidade 3D. **A verificar**.
- [ ] **FTS-0577** — Configurar rotação 3D. **A verificar**.
- [ ] **FTS-0578** — Configurar perspectiva. **A verificar**.
- [ ] **FTS-0579** — Configurar espaçamento entre colunas. **A verificar**.
- [ ] **FTS-0580** — Configurar sobreposição de séries. **A verificar**.

### 8.3. Linhas de tendência

- [ ] **FTS-0581** — Linear. **Pendente**.
- [ ] **FTS-0582** — Exponencial. **Pendente**.
- [ ] **FTS-0583** — Logarítmica. **Pendente**.
- [ ] **FTS-0584** — Polinomial. **Pendente**.
- [ ] **FTS-0585** — Potência. **Pendente**.
- [ ] **FTS-0586** — Média móvel. **Pendente**.

## 9. SmartArt e diagramas


### 9.1. Categorias

- [x] **FTS-0587** — Listas. **Feito**.
- [x] **FTS-0588** — Processos. **Feito**.
- [x] **FTS-0589** — Ciclos. **Feito**.
- [x] **FTS-0590** — Hierarquias. **Feito**.
- [x] **FTS-0591** — Organogramas. **Feito**.
- [x] **FTS-0592** — Relações. **Feito**.
- [x] **FTS-0593** — Matrizes. **Feito**.
- [x] **FTS-0594** — Pirâmides. **Feito**.
- [/] **FTS-0595** — Diagramas com imagens. **Parcial**.
- [x] **FTS-0596** — Diagramas de Venn. **Feito**.
- [x] **FTS-0597** — Diagramas radiais. **Feito**.
- [x] **FTS-0598** — Diagramas de engrenagens. **Feito**.
- [x] **FTS-0599** — Diagramas de fluxo. **Feito**.
- [x] **FTS-0600** — Diagramas de comparação. **Feito**.

### 9.2. Operações

- [ ] **FTS-0601** — Inserir SmartArt. **Pendente**.
- [/] **FTS-0602** — Alterar layout. **Parcial** (Layouts personalizados de lista, processo, ciclo e hierarquia; não converte para todos os layouts SmartArt).
- [ ] **FTS-0603** — Alterar estilo. **Pendente**.
- [ ] **FTS-0604** — Alterar cores. **Pendente**.
- [x] **FTS-0605** — Adicionar item. **Feito** (Adicionar item, irmão ou filho no painel; até 64 nós, com redistribuição e conectores vinculados).
- [x] **FTS-0606** — Excluir item. **Feito** (Excluir nó no painel; filhos são promovidos ao pai do nó removido).
- [x] **FTS-0607** — Promover item. **Feito** (Promover nó com sua subárvore na hierarquia personalizada).
- [x] **FTS-0608** — Rebaixar item. **Feito** (Rebaixar nó para filho do irmão anterior na hierarquia personalizada).
- [x] **FTS-0609** — Mover item. **Feito** (Mover para cima/baixo entre irmãos, preservando os filhos).
- [x] **FTS-0610** — Reordenar hierarquia. **Feito** (Reordenar subárvores com atualização de layout, relações e desfazer).
- [ ] **FTS-0611** — Adicionar assistente em organograma. **Pendente**.
- [x] **FTS-0612** — Abrir painel de texto. **Feito** (Painel de diagrama personalizado com níveis reais; rascunho, adicionar/excluir, promover/rebaixar, ordenar e aplicar em uma transação).
- [ ] **FTS-0613** — Converter texto em SmartArt. **Pendente**.
- [ ] **FTS-0614** — Converter SmartArt em texto. **Pendente**.
- [ ] **FTS-0615** — Converter SmartArt em formas. **Pendente**.
- [ ] **FTS-0616** — Aplicar efeitos 3D. **Pendente**.
- [ ] **FTS-0617** — Inserir imagens em nós. **Pendente**.
- [ ] **FTS-0618** — Alterar tamanho dos nós. **Pendente**.
- [ ] **FTS-0619** — Alterar distribuição. **Pendente**.
- [ ] **FTS-0620** — Alterar direção do diagrama. **Pendente**.

## 10. Vídeo


### 10.1. Inserção

- [ ] **FTS-0621** — Inserir vídeo local. **Pendente**.
- [ ] **FTS-0622** — Inserir vídeo online. **Pendente**.
- [ ] **FTS-0623** — Inserir vídeo incorporado. **Pendente**.
- [ ] **FTS-0624** — Inserir vídeo vinculado. **Pendente**.
- [ ] **FTS-0625** — Inserir vídeo de biblioteca. **Pendente**.
- [ ] **FTS-0626** — Inserir vídeo da área de transferência. **Pendente**.
- [ ] **FTS-0627** — Inserir captura de tela gravada. **Pendente**.

### 10.2. Reprodução

- [ ] **FTS-0628** — Reproduzir. **Pendente**.
- [ ] **FTS-0629** — Pausar. **Pendente**.
- [ ] **FTS-0630** — Parar. **Pendente**.
- [ ] **FTS-0631** — Reiniciar. **Pendente**.
- [ ] **FTS-0632** — Ajustar volume. **Pendente**.
- [ ] **FTS-0633** — Silenciar. **Pendente**.
- [ ] **FTS-0634** — Reproduzir automaticamente. **Pendente**.
- [ ] **FTS-0635** — Reproduzir ao clicar. **Pendente**.
- [ ] **FTS-0636** — Reproduzir em sequência de cliques. **Pendente**.
- [ ] **FTS-0637** — Reproduzir em tela cheia. **Pendente**.
- [ ] **FTS-0638** — Repetir continuamente. **Pendente**.
- [ ] **FTS-0639** — Retroceder ao terminar. **Pendente**.
- [ ] **FTS-0640** — Ocultar quando não estiver reproduzindo. **Pendente**.
- [ ] **FTS-0641** — Mostrar controles de mídia. **Pendente**.

### 10.3. Edição

- [ ] **FTS-0642** — Cortar início. **Pendente**.
- [ ] **FTS-0643** — Cortar final. **Pendente**.
- [ ] **FTS-0644** — Definir imagem de capa. **Pendente**.
- [ ] **FTS-0645** — Adicionar marcador temporal. **Pendente**.
- [ ] **FTS-0646** — Remover marcador. **Pendente**.
- [ ] **FTS-0647** — Configurar fade-in. **Pendente**.
- [ ] **FTS-0648** — Configurar fade-out. **Pendente**.
- [ ] **FTS-0649** — Alterar tamanho. **Pendente**.
- [ ] **FTS-0650** — Cortar enquadramento. **Pendente**.
- [ ] **FTS-0651** — Aplicar bordas e efeitos. **Pendente**.
- [ ] **FTS-0652** — Aplicar estilos de moldura. **Pendente**.
- [ ] **FTS-0653** — Configurar brilho e contraste. **Pendente**.
- [ ] **FTS-0654** — Adicionar legendas. **Pendente**.
- [ ] **FTS-0655** — Sincronizar animações com marcadores. **Pendente**.

## 11. Áudio


### 11.1. Inserção

- [ ] **FTS-0656** — Inserir arquivo de áudio. **Pendente**.
- [ ] **FTS-0657** — Gravar áudio. **Pendente**.
- [ ] **FTS-0658** — Importar narração. **Pendente**.
- [ ] **FTS-0659** — Inserir áudio vinculado. **Pendente**.
- [ ] **FTS-0660** — Incorporar áudio. **Pendente**.

### 11.2. Reprodução

- [ ] **FTS-0661** — Reproduzir automaticamente. **Pendente**.
- [ ] **FTS-0662** — Reproduzir ao clicar. **Pendente**.
- [ ] **FTS-0663** — Reproduzir entre slides. **Pendente**.
- [ ] **FTS-0664** — Repetir áudio. **Pendente**.
- [ ] **FTS-0665** — Executar em segundo plano. **Pendente**.
- [ ] **FTS-0666** — Ocultar ícone durante apresentação. **Pendente**.
- [ ] **FTS-0667** — Ajustar volume. **Pendente**.
- [ ] **FTS-0668** — Silenciar. **Pendente**.
- [ ] **FTS-0669** — Cortar áudio. **Pendente**.
- [ ] **FTS-0670** — Aplicar fade-in. **Pendente**.
- [ ] **FTS-0671** — Aplicar fade-out. **Pendente**.
- [ ] **FTS-0672** — Definir ponto de início. **Pendente**.
- [ ] **FTS-0673** — Definir ponto de término. **Pendente**.
- [ ] **FTS-0674** — Sincronizar com animações. **Pendente**.
- [ ] **FTS-0675** — Sincronizar com transições. **Pendente**.

### 11.3. Narração

- [ ] **FTS-0676** — Gravar narração por slide. **Pendente**.
- [ ] **FTS-0677** — Gravar narração contínua. **Pendente**.
- [ ] **FTS-0678** — Regravar narração. **Pendente**.
- [ ] **FTS-0679** — Excluir narração. **Pendente**.
- [ ] **FTS-0680** — Gerenciar tempos associados à narração. **Pendente**.
- [ ] **FTS-0681** — Exportar apresentação com áudio. **Pendente**.

## 12. Modelos e objetos 3D

- [ ] **FTS-0682** — Importar modelo 3D. **Pendente**.
- [ ] **FTS-0683** — Inserir modelo 3D de biblioteca. **Pendente**.
- [ ] **FTS-0684** — Manipular rotação nos eixos X, Y e Z. **Pendente**.
- [ ] **FTS-0685** — Ajustar escala. **Pendente**.
- [ ] **FTS-0686** — Ajustar enquadramento. **Pendente**.
- [ ] **FTS-0687** — Alterar ângulo de visualização. **Pendente**.
- [ ] **FTS-0688** — Aplicar visualização predefinida. **Pendente**.
- [ ] **FTS-0689** — Alterar perspectiva. **Pendente**.
- [ ] **FTS-0690** — Aplicar animação de rotação. **Pendente**.
- [ ] **FTS-0691** — Aplicar animação de entrada. **Pendente**.
- [ ] **FTS-0692** — Aplicar animação de saída. **Pendente**.
- [ ] **FTS-0693** — Aplicar animação de oscilação. **Pendente**.
- [ ] **FTS-0694** — Aplicar animação de salto. **Pendente**.
- [ ] **FTS-0695** — Preservar dados tridimensionais. **Pendente**.
- [ ] **FTS-0696** — Gerar visualização estática para exportações. **Pendente**.
- [ ] **FTS-0697** — Gerenciar texturas. **Pendente**.
- [ ] **FTS-0698** — Gerenciar materiais. **Pendente**.

# PARTE II — ANIMAÇÕES E APRESENTAÇÕES


## 13. Transições entre slides


### 13.1. Transições sutis

- [/] **FTS-0699** — Cortar. **Parcial**.
- [x] **FTS-0700** — Esmaecer. **Feito**.
- [x] **FTS-0701** — Empurrar. **Feito**.
- [ ] **FTS-0702** — Apagar. **Pendente**.
- [ ] **FTS-0703** — Dividir. **Pendente**.
- [x] **FTS-0704** — Revelar. **Feito**.
- [ ] **FTS-0705** — Cobrir. **Pendente**.
- [ ] **FTS-0706** — Descobrir. **Pendente**.
- [ ] **FTS-0707** — Barras aleatórias. **Pendente**.
- [ ] **FTS-0708** — Forma. **Pendente**.
- [ ] **FTS-0709** — Flash. **Pendente**.

### 13.2. Transições dinâmicas

- [ ] **FTS-0710** — Dissolver. **Pendente**.
- [ ] **FTS-0711** — Quadriculado. **Pendente**.
- [ ] **FTS-0712** — Persianas. **Pendente**.
- [ ] **FTS-0713** — Relógio. **Pendente**.
- [ ] **FTS-0714** — Ondulação. **Pendente**.
- [ ] **FTS-0715** — Favo de mel. **Pendente**.
- [ ] **FTS-0716** — Brilho. **Pendente**.
- [ ] **FTS-0717** — Vórtice. **Pendente**.
- [ ] **FTS-0718** — Fragmentar. **Pendente**.
- [ ] **FTS-0719** — Alternar. **Pendente**.
- [ ] **FTS-0720** — Virar. **Pendente**.
- [ ] **FTS-0721** — Galeria. **Pendente**.
- [ ] **FTS-0722** — Cubo. **Pendente**.
- [ ] **FTS-0723** — Portas. **Pendente**.
- [ ] **FTS-0724** — Caixa. **Pendente**.
- [ ] **FTS-0725** — Janela. **Pendente**.
- [ ] **FTS-0726** — Órbita. **Pendente**.
- [ ] **FTS-0727** — Roda-gigante. **Pendente**.
- [ ] **FTS-0728** — Transportar. **Pendente**.
- [ ] **FTS-0729** — Rolagem. **Pendente**.
- [ ] **FTS-0730** — Girar. **Pendente**.
- [ ] **FTS-0731** — Panorâmica. **Pendente**.
- [x] **FTS-0732** — Zoom. **Feito**.

### 13.3. Morph / Transformar

- [ ] **FTS-0733** — Interpolação de posição. **Pendente**.
- [ ] **FTS-0734** — Interpolação de tamanho. **Pendente**.
- [ ] **FTS-0735** — Interpolação de rotação. **Pendente**.
- [ ] **FTS-0736** — Interpolação de formas. **Pendente**.
- [ ] **FTS-0737** — Interpolação de cores. **Pendente**.
- [ ] **FTS-0738** — Interpolação de transparência. **Pendente**.
- [ ] **FTS-0739** — Interpolação de propriedades compatíveis. **Pendente**.
- [ ] **FTS-0740** — Correspondência automática de objetos. **Pendente**.
- [ ] **FTS-0741** — Correspondência por identificadores. **Pendente**.
- [ ] **FTS-0742** — Correspondência de texto. **Pendente**.
- [ ] **FTS-0743** — Interpolação por palavras. **Pendente**.
- [ ] **FTS-0744** — Interpolação por caracteres. **Pendente**.
- [ ] **FTS-0745** — Transformação de imagens. **Pendente**.
- [ ] **FTS-0746** — Transformação de modelos 3D compatíveis. **Pendente**.

### 13.4. Configurações

- [x] **FTS-0747** — Aplicar transição. **Feito**.
- [x] **FTS-0748** — Remover transição. **Feito**.
- [x] **FTS-0749** — Pré-visualizar. **Feito**.
- [x] **FTS-0750** — Alterar direção. **Feito**.
- [x] **FTS-0751** — Alterar duração. **Feito**.
- [ ] **FTS-0752** — Adicionar som. **Pendente**.
- [x] **FTS-0753** — Avançar ao clicar. **Feito**.
- [x] **FTS-0754** — Avançar automaticamente. **Feito**.
- [x] **FTS-0755** — Configurar atraso. **Feito**.
- [x] **FTS-0756** — Aplicar a todos os slides. **Feito**.
- [x] **FTS-0757** — Preservar transições individuais. **Feito**.

## 14. Animações de objetos


### 14.1. Entrada

- [x] **FTS-0758** — Aparecer. **Feito**.
- [x] **FTS-0759** — Esmaecer. **Feito**.
- [x] **FTS-0760** — Surgir. **Feito**.
- [x] **FTS-0761** — Flutuar. **Feito**.
- [x] **FTS-0762** — Dividir. **Feito**.
- [x] **FTS-0763** — Apagar. **Feito**.
- [x] **FTS-0764** — Forma. **Feito**.
- [x] **FTS-0765** — Roda. **Feito**.
- [x] **FTS-0766** — Barras. **Feito**.
- [x] **FTS-0767** — Zoom. **Feito**.
- [x] **FTS-0768** — Girar. **Feito**.
- [x] **FTS-0769** — Rebote. **Feito**.
- [x] **FTS-0770** — Deslizar. **Feito**.
- [x] **FTS-0771** — Expandir. **Feito**.
- [x] **FTS-0772** — Contrair. **Feito**.

### 14.2. Ênfase

- [x] **FTS-0773** — Pulso. **Feito**.
- [ ] **FTS-0774** — Girar. **Pendente**.
- [ ] **FTS-0775** — Aumentar. **Pendente**.
- [ ] **FTS-0776** — Diminuir. **Pendente**.
- [ ] **FTS-0777** — Alterar cor. **Pendente**.
- [ ] **FTS-0778** — Alterar transparência. **Pendente**.
- [ ] **FTS-0779** — Oscilar. **Pendente**.
- [ ] **FTS-0780** — Ondular. **Pendente**.
- [ ] **FTS-0781** — Clarear. **Pendente**.
- [ ] **FTS-0782** — Escurecer. **Pendente**.
- [ ] **FTS-0783** — Descolorir. **Pendente**.
- [ ] **FTS-0784** — Alterar preenchimento. **Pendente**.
- [ ] **FTS-0785** — Alterar tamanho da fonte. **Pendente**.
- [ ] **FTS-0786** — Alterar cor da fonte. **Pendente**.

### 14.3. Saída

- [ ] **FTS-0787** — Desaparecer. **Pendente**.
- [ ] **FTS-0788** — Esmaecer. **Pendente**.
- [ ] **FTS-0789** — Flutuar para fora. **Pendente**.
- [ ] **FTS-0790** — Dividir. **Pendente**.
- [ ] **FTS-0791** — Apagar. **Pendente**.
- [ ] **FTS-0792** — Zoom. **Pendente**.
- [ ] **FTS-0793** — Roda. **Pendente**.
- [ ] **FTS-0794** — Encolher. **Pendente**.
- [ ] **FTS-0795** — Deslizar para fora. **Pendente**.

### 14.4. Trajetórias

- [ ] **FTS-0796** — Linha. **Pendente**.
- [ ] **FTS-0797** — Arco. **Pendente**.
- [ ] **FTS-0798** — Curva. **Pendente**.
- [ ] **FTS-0799** — Círculo. **Pendente**.
- [ ] **FTS-0800** — Quadrado. **Pendente**.
- [ ] **FTS-0801** — Retângulo. **Pendente**.
- [ ] **FTS-0802** — Estrela. **Pendente**.
- [ ] **FTS-0803** — Espiral. **Pendente**.
- [ ] **FTS-0804** — Loop. **Pendente**.
- [ ] **FTS-0805** — Onda. **Pendente**.
- [ ] **FTS-0806** — Trajetória personalizada. **Pendente**.
- [ ] **FTS-0807** — Trajetória desenhada à mão. **Pendente**.

### 14.5. Configurações avançadas

- [x] **FTS-0808** — Iniciar ao clicar. **Feito**.
- [x] **FTS-0809** — Iniciar com o anterior. **Feito**.
- [x] **FTS-0810** — Iniciar após o anterior. **Feito**.
- [x] **FTS-0811** — Definir atraso. **Feito**.
- [x] **FTS-0812** — Definir duração. **Feito**.
- [ ] **FTS-0813** — Repetir animação. **Pendente**.
- [ ] **FTS-0814** — Reverter automaticamente. **Pendente**.
- [ ] **FTS-0815** — Configurar aceleração. **A verificar**.
- [ ] **FTS-0816** — Configurar desaceleração. **A verificar**.
- [x] **FTS-0817** — Definir sequência. **Feito**.
- [x] **FTS-0818** — Reordenar animações. **Feito**.
- [ ] **FTS-0819** — Copiar animação. **A verificar**.
- [x] **FTS-0820** — Remover animação. **Feito**.
- [ ] **FTS-0821** — Animar texto por parágrafo. **Pendente**.
- [ ] **FTS-0822** — Animar texto por palavra. **Pendente**.
- [ ] **FTS-0823** — Animar texto por caractere. **Pendente**.
- [ ] **FTS-0824** — Animar elementos de gráficos. **A verificar**.
- [ ] **FTS-0825** — Animar SmartArt por nível. **A verificar**.
- [ ] **FTS-0826** — Animar grupos. **A verificar**.
- [ ] **FTS-0827** — Definir gatilho de animação. **A verificar**.
- [ ] **FTS-0828** — Disparar ao clicar em objeto específico. **A verificar**.
- [ ] **FTS-0829** — Disparar em marcador de vídeo. **Pendente**.
- [ ] **FTS-0830** — Reproduzir som durante animação. **Pendente**.
- [ ] **FTS-0831** — Ocultar objeto após animação. **A verificar**.
- [ ] **FTS-0832** — Retornar à aparência original. **A verificar**.
- [ ] **FTS-0833** — Definir comportamento ao concluir. **A verificar**.

### 14.6. Linha do tempo

- [x] **FTS-0834** — Visualizar todas as animações. **Feito**.
- [x] **FTS-0835** — Selecionar animação. **Feito**.
- [ ] **FTS-0836** — Alterar duração graficamente. **A verificar**.
- [x] **FTS-0837** — Reordenar efeitos. **Feito**.
- [ ] **FTS-0838** — Visualizar dependências. **A verificar**.
- [ ] **FTS-0839** — Inspecionar gatilhos. **A verificar**.
- [ ] **FTS-0840** — Visualizar animações simultâneas. **A verificar**.
- [/] **FTS-0841** — Pré-visualizar sequência completa. **Parcial**.

## 15. Apresentação de slides


### 15.1. Execução

- [x] **FTS-0842** — Iniciar do começo. **Feito**.
- [x] **FTS-0843** — Iniciar do slide atual. **Feito**.
- [ ] **FTS-0844** — Executar apresentação personalizada. **A verificar**.
- [ ] **FTS-0845** — Executar em janela. **A verificar**.
- [x] **FTS-0846** — Executar em tela cheia. **Feito**.
- [ ] **FTS-0847** — Executar em modo quiosque. **A verificar**.
- [ ] **FTS-0848** — Repetir continuamente. **A verificar**.
- [ ] **FTS-0849** — Usar tempos gravados. **A verificar**.
- [ ] **FTS-0850** — Ignorar tempos gravados. **A verificar**.
- [ ] **FTS-0851** — Usar narrações. **A verificar**.
- [ ] **FTS-0852** — Desativar narrações. **A verificar**.
- [x] **FTS-0853** — Usar animações. **Feito**.
- [ ] **FTS-0854** — Desativar animações. **A verificar**.

### 15.2. Navegação

- [x] **FTS-0855** — Próximo slide. **Feito**.
- [x] **FTS-0856** — Slide anterior. **Feito**.
- [ ] **FTS-0857** — Ir ao primeiro slide. **A verificar**.
- [ ] **FTS-0858** — Ir ao último slide. **A verificar**.
- [ ] **FTS-0859** — Ir ao slide por número. **A verificar**.
- [ ] **FTS-0860** — Abrir grade de slides. **A verificar**.
- [x] **FTS-0861** — Navegar por hiperlinks. **Feito**.
- [x] **FTS-0862** — Navegar por botões de ação. **Feito**.
- [x] **FTS-0863** — Voltar ao slide anterior. **Feito**.
- [x] **FTS-0864** — Encerrar apresentação. **Feito**.

### 15.3. Ferramentas do apresentador

- [ ] **FTS-0865** — Ponteiro laser. **A verificar**.
- [ ] **FTS-0866** — Caneta. **A verificar**.
- [ ] **FTS-0867** — Marcador. **A verificar**.
- [ ] **FTS-0868** — Borracha. **A verificar**.
- [ ] **FTS-0869** — Apagar anotações. **A verificar**.
- [x] **FTS-0870** — Tela preta. **Feito**.
- [x] **FTS-0871** — Tela branca. **Feito**.
- [ ] **FTS-0872** — Ampliar região. **A verificar**.
- [/] **FTS-0873** — Visualizar anotações. **Parcial**.
- [/] **FTS-0874** — Visualizar próximo slide. **Parcial**.
- [/] **FTS-0875** — Visualizar tempo decorrido. **Parcial**.
- [x] **FTS-0876** — Pausar cronômetro. **Feito**.
- [x] **FTS-0877** — Reiniciar cronômetro. **Feito**.
- [/] **FTS-0878** — Configurar monitor. **Parcial**.
- [ ] **FTS-0879** — Alternar monitores. **A verificar**.
- [ ] **FTS-0880** — Ocultar interface do apresentador. **A verificar**.

### 15.4. Apresentações interativas

- [x] **FTS-0881** — Hiperlinks. **Feito**.
- [x] **FTS-0882** — Botões de ação. **Feito**.
- [x] **FTS-0883** — Navegação para slide específico. **Feito**.
- [ ] **FTS-0884** — Navegação para seção. **A verificar**.
- [ ] **FTS-0885** — Zoom de resumo. **A verificar**.
- [ ] **FTS-0886** — Zoom de seção. **A verificar**.
- [ ] **FTS-0887** — Zoom de slide. **A verificar**.
- [ ] **FTS-0888** — Retorno automático ao slide de origem. **A verificar**.
- [ ] **FTS-0889** — Execução de mídia por ação. **A verificar**.
- [ ] **FTS-0890** — Gatilhos de animação. **Pendente**.
- [x] **FTS-0891** — Navegação não linear. **Feito**.

## 16. Gravação, ensaio e exportação de apresentações

- [ ] **FTS-0892** — Testar intervalos. **Pendente**.
- [ ] **FTS-0893** — Registrar tempo por slide. **Pendente**.
- [ ] **FTS-0894** — Gravar apresentação. **Pendente**.
- [ ] **FTS-0895** — Gravar áudio. **Pendente**.
- [ ] **FTS-0896** — Gravar câmera. **Pendente**.
- [ ] **FTS-0897** — Gravar anotações manuscritas. **Pendente**.
- [ ] **FTS-0898** — Gravar ponteiro. **Pendente**.
- [ ] **FTS-0899** — Regravar slide. **Pendente**.
- [ ] **FTS-0900** — Excluir gravação. **Pendente**.
- [ ] **FTS-0901** — Usar teleprompter. **Pendente**.
- [ ] **FTS-0902** — Visualizar anotações durante gravação. **Pendente**.
- [ ] **FTS-0903** — Inserir câmera ao vivo com Cameo. **Pendente**.
- [ ] **FTS-0904** — Alterar formato da câmera. **Pendente**.
- [ ] **FTS-0905** — Aplicar moldura à câmera. **Pendente**.
- [ ] **FTS-0906** — Exportar apresentação gravada. **Pendente**.
- [ ] **FTS-0907** — Exportar para vídeo. **Pendente**.
- [ ] **FTS-0908** — Exportar para GIF animado. **Pendente**.
- [ ] **FTS-0909** — Configurar resolução. **Pendente**.
- [ ] **FTS-0910** — Configurar quadros por segundo. **Pendente**.
- [ ] **FTS-0911** — Configurar tempos de slides. **Pendente**.
- [ ] **FTS-0912** — Incorporar áudio no vídeo. **Pendente**.
- [ ] **FTS-0913** — Incorporar animações no vídeo. **Pendente**.
- [ ] **FTS-0914** — Incorporar transições no vídeo. **Pendente**.

# PARTE III — ESTRUTURA E RECURSOS AVANÇADOS


## 17. Slide Mestre, layouts e temas


### 17.1. Slide Mestre

- [ ] **FTS-0915** — Criar mestre. **Pendente**.
- [ ] **FTS-0916** — Excluir mestre. **Pendente**.
- [ ] **FTS-0917** — Duplicar mestre. **Pendente**.
- [ ] **FTS-0918** — Renomear mestre. **Pendente**.
- [ ] **FTS-0919** — Preservar mestre. **Pendente**.
- [/] **FTS-0920** — Importar mestre de outra apresentação. **Parcial**.
- [ ] **FTS-0921** — Associar layouts ao mestre. **Pendente**.
- [ ] **FTS-0922** — Definir estilos globais de texto. **Pendente**.
- [ ] **FTS-0923** — Definir estilos de títulos. **Pendente**.
- [ ] **FTS-0924** — Definir estilos de corpo. **Pendente**.
- [ ] **FTS-0925** — Definir cores globais. **Pendente**.
- [ ] **FTS-0926** — Definir fontes globais. **Pendente**.
- [ ] **FTS-0927** — Definir fundo global. **Pendente**.
- [/] **FTS-0928** — Definir objetos persistentes. **Parcial**.
- [ ] **FTS-0929** — Configurar número do slide. **Pendente**.
- [ ] **FTS-0930** — Configurar rodapé. **Pendente**.
- [ ] **FTS-0931** — Configurar data e hora. **Pendente**.
- [ ] **FTS-0932** — Configurar marcadores de posição. **Pendente**.

### 17.2. Layouts

- [ ] **FTS-0933** — Criar layout. **A verificar**.
- [ ] **FTS-0934** — Excluir layout. **A verificar**.
- [ ] **FTS-0935** — Duplicar layout. **A verificar**.
- [ ] **FTS-0936** — Renomear layout. **A verificar**.
- [/] **FTS-0937** — Aplicar layout. **Parcial**.
- [/] **FTS-0938** — Redefinir layout. **Parcial**.
- [ ] **FTS-0939** — Preservar layout. **A verificar**.
- [/] **FTS-0940** — Criar marcador de título. **Parcial**.
- [/] **FTS-0941** — Criar marcador de corpo. **Parcial**.
- [ ] **FTS-0942** — Criar marcador de imagem. **A verificar**.
- [ ] **FTS-0943** — Criar marcador de tabela. **A verificar**.
- [ ] **FTS-0944** — Criar marcador de gráfico. **A verificar**.
- [ ] **FTS-0945** — Criar marcador de mídia. **A verificar**.
- [ ] **FTS-0946** — Criar marcador de SmartArt. **A verificar**.
- [ ] **FTS-0947** — Reposicionar marcadores. **A verificar**.
- [ ] **FTS-0948** — Redimensionar marcadores. **A verificar**.
- [ ] **FTS-0949** — Configurar herança de estilos. **A verificar**.

### 17.3. Temas

- [ ] **FTS-0950** — Criar tema. **A verificar**.
- [ ] **FTS-0951** — Importar tema. **A verificar**.
- [ ] **FTS-0952** — Exportar tema. **A verificar**.
- [x] **FTS-0953** — Aplicar tema. **Feito**.
- [ ] **FTS-0954** — Configurar cores. **A verificar**.
- [ ] **FTS-0955** — Configurar fontes. **A verificar**.
- [ ] **FTS-0956** — Configurar efeitos. **A verificar**.
- [ ] **FTS-0957** — Criar variantes. **A verificar**.
- [ ] **FTS-0958** — Definir cores de destaque. **A verificar**.
- [ ] **FTS-0959** — Definir cores de hiperlink. **A verificar**.
- [ ] **FTS-0960** — Definir fontes alternativas. **A verificar**.
- [ ] **FTS-0961** — Definir estilos padrão de objetos. **A verificar**.
- [ ] **FTS-0962** — Preservar tema original ao importar slides. **A verificar**.

### 17.4. Folhetos e anotações mestras

- [ ] **FTS-0963** — Criar layout de folheto. **A verificar**.
- [ ] **FTS-0964** — Configurar slides por página. **A verificar**.
- [ ] **FTS-0965** — Configurar cabeçalho. **A verificar**.
- [ ] **FTS-0966** — Configurar rodapé. **A verificar**.
- [ ] **FTS-0967** — Configurar orientação. **A verificar**.
- [ ] **FTS-0968** — Configurar margens. **A verificar**.
- [ ] **FTS-0969** — Definir aparência das anotações. **A verificar**.
- [ ] **FTS-0970** — Configurar numeração de páginas. **A verificar**.
- [ ] **FTS-0971** — Configurar impressão de notas. **A verificar**.

## 18. Cabeçalhos, rodapés e elementos dinâmicos

- [/] **FTS-0972** — Inserir número do slide. **Parcial**.
- [ ] **FTS-0973** — Ocultar número no slide de título. **A verificar**.
- [ ] **FTS-0974** — Definir numeração inicial. **A verificar**.
- [/] **FTS-0975** — Inserir data fixa. **Parcial**.
- [ ] **FTS-0976** — Inserir data automática. **A verificar**.
- [ ] **FTS-0977** — Inserir rodapé. **A verificar**.
- [ ] **FTS-0978** — Inserir texto de cabeçalho em anotações e folhetos. **A verificar**.
- [ ] **FTS-0979** — Aplicar a um slide. **A verificar**.
- [ ] **FTS-0980** — Aplicar a todos. **A verificar**.
- [ ] **FTS-0981** — Excluir elementos dinâmicos. **A verificar**.
- [ ] **FTS-0982** — Definir estilos herdados. **A verificar**.
- [ ] **FTS-0983** — Definir posicionamento. **A verificar**.
- [ ] **FTS-0984** — Definir formatação. **A verificar**.
- [ ] **FTS-0985** — Atualizar campos dinâmicos. **A verificar**.

## 19. Comentários e revisão

- [x] **FTS-0986** — Criar comentário. **Feito**.
- [ ] **FTS-0987** — Editar comentário. **A verificar**.
- [x] **FTS-0988** — Excluir comentário. **Feito**.
- [ ] **FTS-0989** — Responder comentário. **A verificar**.
- [ ] **FTS-0990** — Criar conversa encadeada. **A verificar**.
- [x] **FTS-0991** — Resolver comentário. **Feito**.
- [x] **FTS-0992** — Reabrir comentário. **Feito**.
- [ ] **FTS-0993** — Mencionar usuário. **A verificar**.
- [ ] **FTS-0994** — Associar comentário a objeto. **A verificar**.
- [x] **FTS-0995** — Associar comentário a slide. **Feito**.
- [ ] **FTS-0996** — Navegar entre comentários. **A verificar**.
- [ ] **FTS-0997** — Filtrar comentários. **A verificar**.
- [ ] **FTS-0998** — Mostrar comentários por autor. **A verificar**.
- [ ] **FTS-0999** — Comparar apresentações. **A verificar**.
- [ ] **FTS-1000** — Inspecionar alterações. **A verificar**.
- [ ] **FTS-1001** — Aceitar alterações compatíveis. **A verificar**.
- [ ] **FTS-1002** — Rejeitar alterações compatíveis. **A verificar**.
- [ ] **FTS-1003** — Mesclar revisões. **A verificar**.
- [ ] **FTS-1004** — Exibir informações dos autores. **A verificar**.
- [/] **FTS-1005** — Verificar ortografia. **Parcial**.
- [/] **FTS-1006** — Definir idioma. **Parcial**.
- [ ] **FTS-1007** — Traduzir conteúdo. **A verificar**.
- [ ] **FTS-1008** — Verificar acessibilidade. **A verificar**.

## 20. Colaboração

- [ ] **FTS-1009** — Compartilhar apresentação. **A verificar**.
- [ ] **FTS-1010** — Criar link de compartilhamento. **A verificar**.
- [ ] **FTS-1011** — Definir permissões. **A verificar**.
- [ ] **FTS-1012** — Acesso somente leitura. **A verificar**.
- [ ] **FTS-1013** — Acesso de edição. **A verificar**.
- [ ] **FTS-1014** — Coautoria em tempo real. **A verificar**.
- [ ] **FTS-1015** — Identificação de colaboradores. **A verificar**.
- [ ] **FTS-1016** — Indicadores de presença. **A verificar**.
- [ ] **FTS-1017** — Sincronização de alterações. **A verificar**.
- [ ] **FTS-1018** — Resolução de conflitos. **A verificar**.
- [x] **FTS-1019** — Salvamento automático. **Feito**.
- [ ] **FTS-1020** — Histórico de versões. **A verificar**.
- [ ] **FTS-1021** — Restauração de versões. **A verificar**.
- [ ] **FTS-1022** — Comentários colaborativos. **A verificar**.
- [ ] **FTS-1023** — Notificações de comentários. **A verificar**.
- [ ] **FTS-1024** — Integração com armazenamento remoto. **A verificar**.
- [ ] **FTS-1025** — Compartilhamento por organização. **A verificar**.
- [ ] **FTS-1026** — Gerenciamento de sessões simultâneas. **A verificar**.

## 21. Acessibilidade

- [ ] **FTS-1027** — Texto alternativo em imagens. **A verificar**.
- [ ] **FTS-1028** — Texto alternativo em formas. **A verificar**.
- [ ] **FTS-1029** — Texto alternativo em gráficos. **A verificar**.
- [ ] **FTS-1030** — Marcação de objetos decorativos. **A verificar**.
- [ ] **FTS-1031** — Ordem de leitura. **A verificar**.
- [ ] **FTS-1032** — Verificação de contraste. **A verificar**.
- [ ] **FTS-1033** — Verificação de títulos de slides. **A verificar**.
- [ ] **FTS-1034** — Verificação de texto alternativo. **A verificar**.
- [ ] **FTS-1035** — Verificação de nomes de tabelas. **A verificar**.
- [ ] **FTS-1036** — Verificação de estruturas de tabelas. **A verificar**.
- [ ] **FTS-1037** — Verificação de legibilidade. **A verificar**.
- [ ] **FTS-1038** — Verificação de hyperlinks. **A verificar**.
- [ ] **FTS-1039** — Verificação de identificação do idioma. **A verificar**.
- [ ] **FTS-1040** — Legendas. **A verificar**.
- [ ] **FTS-1041** — Suporte a leitores de tela. **A verificar**.
- [/] **FTS-1042** — Navegação por teclado. **Parcial**.
- [/] **FTS-1043** — Indicadores de foco. **Parcial**.
- [ ] **FTS-1044** — Escala de interface. **A verificar**.
- [ ] **FTS-1045** — Suporte a alto contraste. **A verificar**.
- [ ] **FTS-1046** — Verificação automática de acessibilidade. **A verificar**.
- [ ] **FTS-1047** — Relatório de problemas. **A verificar**.
- [ ] **FTS-1048** — Sugestões de correção. **A verificar**.
- [ ] **FTS-1049** — Exportação acessível para PDF. **A verificar**.

## 22. Recursos de tinta e desenho manual

- [ ] **FTS-1050** — Caneta. **A verificar**.
- [ ] **FTS-1051** — Marcador. **A verificar**.
- [ ] **FTS-1052** — Borracha. **A verificar**.
- [ ] **FTS-1053** — Borracha por segmento. **A verificar**.
- [ ] **FTS-1054** — Borracha por traço. **A verificar**.
- [ ] **FTS-1055** — Seleção por laço. **A verificar**.
- [ ] **FTS-1056** — Régua. **A verificar**.
- [ ] **FTS-1057** — Desenho livre. **A verificar**.
- [ ] **FTS-1058** — Desenho com mouse. **A verificar**.
- [ ] **FTS-1059** — Desenho com touchscreen. **A verificar**.
- [ ] **FTS-1060** — Desenho com caneta digital. **A verificar**.
- [ ] **FTS-1061** — Sensibilidade à pressão. **A verificar**.
- [ ] **FTS-1062** — Personalização de cor. **A verificar**.
- [ ] **FTS-1063** — Personalização de espessura. **A verificar**.
- [ ] **FTS-1064** — Canetas com efeitos especiais. **A verificar**.
- [ ] **FTS-1065** — Converter tinta em forma. **A verificar**.
- [ ] **FTS-1066** — Converter tinta em texto. **A verificar**.
- [ ] **FTS-1067** — Converter tinta em equação. **A verificar**.
- [ ] **FTS-1068** — Reproduzir sequência de traços. **A verificar**.
- [ ] **FTS-1069** — Editar traços. **A verificar**.
- [ ] **FTS-1070** — Mover traços. **A verificar**.
- [ ] **FTS-1071** — Agrupar traços. **A verificar**.
- [ ] **FTS-1072** — Excluir todos os traços. **A verificar**.
- [ ] **FTS-1073** — Preservar tinta na apresentação. **A verificar**.

## 23. Equações e símbolos matemáticos

- [ ] **FTS-1074** — Editor de equações. **Pendente**.
- [ ] **FTS-1075** — Frações. **Pendente**.
- [/] **FTS-1076** — Potências. **Parcial**.
- [/] **FTS-1077** — Índices. **Parcial**.
- [/] **FTS-1078** — Radicais. **Parcial**.
- [/] **FTS-1079** — Integrais. **Parcial**.
- [/] **FTS-1080** — Somatórios. **Parcial**.
- [ ] **FTS-1081** — Produtórios. **Pendente**.
- [ ] **FTS-1082** — Matrizes. **Pendente**.
- [/] **FTS-1083** — Limites. **Parcial**.
- [/] **FTS-1084** — Operadores. **Parcial**.
- [/] **FTS-1085** — Letras gregas. **Parcial**.
- [/] **FTS-1086** — Operadores lógicos. **Parcial**.
- [/] **FTS-1087** — Símbolos geométricos. **Parcial**.
- [ ] **FTS-1088** — Estruturas de alinhamento. **Pendente**.
- [/] **FTS-1089** — Chaves e delimitadores. **Parcial**.
- [/] **FTS-1090** — Equações lineares. **Parcial**.
- [ ] **FTS-1091** — Equações profissionais. **Pendente**.
- [ ] **FTS-1092** — Entrada matemática compatível com sintaxe LaTeX. **Pendente**.
- [ ] **FTS-1093** — Compatibilidade com OMML. **Pendente**.
- [ ] **FTS-1094** — Importação de fórmulas. **Pendente**.
- [ ] **FTS-1095** — Exportação de fórmulas. **Pendente**.
- [/] **FTS-1096** — Conversão de fórmulas para representação visual. **Parcial**.

## 24. Hiperlinks e ações

- [x] **FTS-1097** — Link para URL. **Feito**.
- [ ] **FTS-1098** — Link para arquivo. **A verificar**.
- [x] **FTS-1099** — Link para e-mail. **Feito**.
- [/] **FTS-1100** — Link para slide. **Parcial**.
- [ ] **FTS-1101** — Link para seção. **A verificar**.
- [ ] **FTS-1102** — Link para apresentação externa. **A verificar**.
- [/] **FTS-1103** — Link em texto. **Parcial**.
- [x] **FTS-1104** — Link em imagem. **Feito**.
- [x] **FTS-1105** — Link em forma. **Feito**.
- [x] **FTS-1106** — Remover link. **Feito**.
- [x] **FTS-1107** — Editar link. **Feito**.
- [ ] **FTS-1108** — Definir tooltip. **A verificar**.
- [/] **FTS-1109** — Ação ao clicar. **Parcial**.
- [ ] **FTS-1110** — Ação ao passar o mouse. **A verificar**.
- [ ] **FTS-1111** — Reproduzir som por ação. **A verificar**.
- [x] **FTS-1112** — Navegar para slide específico. **Feito**.
- [ ] **FTS-1113** — Voltar. **A verificar**.
- [ ] **FTS-1114** — Avançar. **A verificar**.
- [ ] **FTS-1115** — Abrir documento externo. **A verificar**.
- [ ] **FTS-1116** — Executar macro. **A verificar**.
- [ ] **FTS-1117** — Executar programa. **A verificar**.

## 25. Objetos incorporados e vínculos externos

- [ ] **FTS-1118** — Incorporar documento. **Pendente**.
- [ ] **FTS-1119** — Incorporar planilha. **Pendente**.
- [ ] **FTS-1120** — Incorporar gráfico. **Pendente**.
- [ ] **FTS-1121** — Incorporar arquivo PDF como objeto. **Pendente**.
- [ ] **FTS-1122** — Vincular planilha. **Pendente**.
- [ ] **FTS-1123** — Vincular gráfico. **Pendente**.
- [ ] **FTS-1124** — Vincular imagem. **Pendente**.
- [ ] **FTS-1125** — Vincular vídeo. **Pendente**.
- [ ] **FTS-1126** — Atualizar vínculo. **Pendente**.
- [ ] **FTS-1127** — Quebrar vínculo. **Pendente**.
- [ ] **FTS-1128** — Alterar origem do vínculo. **Pendente**.
- [ ] **FTS-1129** — Editar objeto incorporado. **Pendente**.
- [ ] **FTS-1130** — Visualizar objeto como ícone. **Pendente**.
- [ ] **FTS-1131** — Exibir nome do objeto. **Pendente**.
- [ ] **FTS-1132** — Preservar dados binários. **Pendente**.
- [ ] **FTS-1133** — Preservar relacionamentos. **Pendente**.
- [ ] **FTS-1134** — Preservar objetos não reconhecidos. **Pendente**.
- [ ] **FTS-1135** — Bloquear execução de conteúdo externo inseguro. **Pendente**.

## 26. Macros, suplementos e extensibilidade


### 26.1. VBA

- [ ] **FTS-1136** — Preservar projeto VBA. **Pendente**.
- [ ] **FTS-1137** — Identificar macros. **Pendente**.
- [ ] **FTS-1138** — Listar macros. **Pendente**.
- [ ] **FTS-1139** — Executar macros em ambiente compatível. **Pendente**.
- [ ] **FTS-1140** — Criar módulos. **Pendente**.
- [ ] **FTS-1141** — Editar módulos. **Pendente**.
- [ ] **FTS-1142** — Depurar código. **Pendente**.
- [ ] **FTS-1143** — Responder a eventos. **Pendente**.
- [ ] **FTS-1144** — Automatizar objetos da apresentação. **Pendente**.

### 26.2. Suplementos

- [ ] **FTS-1145** — Carregar suplemento. **A verificar**.
- [ ] **FTS-1146** — Desativar suplemento. **A verificar**.
- [ ] **FTS-1147** — Remover suplemento. **A verificar**.
- [ ] **FTS-1148** — Configurar suplemento. **A verificar**.
- [/] **FTS-1149** — API para manipular slides. **Parcial**.
- [/] **FTS-1150** — API para manipular formas. **Parcial**.
- [/] **FTS-1151** — API para manipular textos. **Parcial**.
- [ ] **FTS-1152** — API para manipular tabelas. **A verificar**.
- [ ] **FTS-1153** — API para manipular gráficos. **A verificar**.
- [/] **FTS-1154** — API para manipular arquivos. **Parcial**.
- [ ] **FTS-1155** — API para exportação. **A verificar**.
- [ ] **FTS-1156** — API para eventos. **A verificar**.

### 26.3. Extensões em Rust — proposta

- [ ] **FTS-1157** — Sistema de plugins. **A verificar**.
- [ ] **FTS-1158** — Registro de comandos. **A verificar**.
- [ ] **FTS-1159** — Registro de tipos de objetos. **A verificar**.
- [ ] **FTS-1160** — Registro de importadores. **A verificar**.
- [ ] **FTS-1161** — Registro de exportadores. **A verificar**.
- [ ] **FTS-1162** — Registro de renderizadores. **A verificar**.
- [ ] **FTS-1163** — Registro de ferramentas. **A verificar**.
- [ ] **FTS-1164** — Eventos de edição. **A verificar**.
- [ ] **FTS-1165** — Eventos de seleção. **A verificar**.
- [ ] **FTS-1166** — Eventos de salvamento. **A verificar**.
- [ ] **FTS-1167** — Eventos de apresentação. **A verificar**.
- [ ] **FTS-1168** — API de scripting. **A verificar**.
- [ ] **FTS-1169** — Sandbox de plugins. **A verificar**.
- [ ] **FTS-1170** — Permissões por extensão. **A verificar**.
- [ ] **FTS-1171** — Versionamento da API. **A verificar**.

## 27. Inteligência artificial e funcionalidades conectadas


### 27.1. Assistência à criação

- [ ] **FTS-1172** — Criar apresentação a partir de texto. **A verificar**.
- [ ] **FTS-1173** — Criar apresentação a partir de documentos. **A verificar**.
- [ ] **FTS-1174** — Gerar roteiro. **A verificar**.
- [ ] **FTS-1175** — Gerar títulos. **A verificar**.
- [ ] **FTS-1176** — Gerar resumos. **A verificar**.
- [ ] **FTS-1177** — Criar estrutura de slides. **A verificar**.
- [ ] **FTS-1178** — Reorganizar conteúdo. **A verificar**.
- [ ] **FTS-1179** — Reescrever textos. **A verificar**.
- [ ] **FTS-1180** — Reduzir quantidade de texto. **A verificar**.
- [ ] **FTS-1181** — Expandir explicações. **A verificar**.
- [ ] **FTS-1182** — Transformar listas em diagramas. **A verificar**.
- [ ] **FTS-1183** — Gerar anotações do apresentador. **A verificar**.
- [ ] **FTS-1184** — Sugerir layouts. **A verificar**.
- [ ] **FTS-1185** — Sugerir imagens. **A verificar**.
- [ ] **FTS-1186** — Sugerir gráficos. **A verificar**.
- [ ] **FTS-1187** — Sugerir temas. **A verificar**.
- [ ] **FTS-1188** — Corrigir inconsistências visuais. **A verificar**.

### 27.2. Assistência ao apresentador

- [ ] **FTS-1189** — Analisar velocidade da fala. **A verificar**.
- [ ] **FTS-1190** — Identificar palavras repetidas. **A verificar**.
- [ ] **FTS-1191** — Detectar vícios de linguagem. **A verificar**.
- [ ] **FTS-1192** — Identificar leitura literal dos slides. **A verificar**.
- [ ] **FTS-1193** — Analisar ritmo. **A verificar**.
- [ ] **FTS-1194** — Avaliar duração. **A verificar**.
- [ ] **FTS-1195** — Fornecer relatório de ensaio. **A verificar**.
- [ ] **FTS-1196** — Sugerir melhorias. **A verificar**.

### 27.3. Tradução e linguagem

- [ ] **FTS-1197** — Traduzir textos. **A verificar**.
- [ ] **FTS-1198** — Identificar idioma. **A verificar**.
- [/] **FTS-1199** — Corrigir gramática. **Parcial**.
- [ ] **FTS-1200** — Ajustar formalidade. **A verificar**.
- [ ] **FTS-1201** — Gerar legendas. **A verificar**.
- [ ] **FTS-1202** — Transcrever narração. **A verificar**.
- [ ] **FTS-1203** — Resumir apresentações. **A verificar**.

### 27.4. Integração com modelos locais

- [ ] **FTS-1204** — Suporte a Ollama. **A verificar**.
- [ ] **FTS-1205** — Modelos locais de linguagem. **A verificar**.
- [ ] **FTS-1206** — Modelos remotos. **A verificar**.
- [ ] **FTS-1207** — Gerenciamento de provedores. **A verificar**.
- [ ] **FTS-1208** — Cache de respostas. **A verificar**.
- [/] **FTS-1209** — Geração estruturada de slides. **Parcial**.
- [/] **FTS-1210** — Geração de layouts por comandos. **Parcial**.
- [/] **FTS-1211** — Validação de conteúdo gerado. **Parcial**.
- [/] **FTS-1212** — Histórico das alterações realizadas pela IA. **Parcial**.

# PARTE IV — COMPATIBILIDADE REAL COM PPTX


## 28. Estrutura de arquivos Open XML


### 28.1. Estrutura principal — partes do pacote

- [ ] **FTS-1213** — [Content_Types].xml. **A verificar**.
- [ ] **FTS-1214** — _rels/.rels. **A verificar**.
- [ ] **FTS-1215** — docProps/core.xml. **A verificar**.
- [ ] **FTS-1216** — docProps/app.xml. **A verificar**.
- [ ] **FTS-1217** — docProps/custom.xml. **A verificar**.
- [/] **FTS-1218** — ppt/presentation.xml. **Parcial**.
- [/] **FTS-1219** — ppt/_rels/presentation.xml.rels. **Parcial**.
- [/] **FTS-1220** — ppt/slides/slideN.xml. **Parcial**.
- [/] **FTS-1221** — ppt/slides/_rels/. **Parcial**.
- [/] **FTS-1222** — ppt/slideLayouts/. **Parcial**.
- [/] **FTS-1223** — ppt/slideMasters/. **Parcial**.
- [/] **FTS-1224** — ppt/theme/. **Parcial**.
- [/] **FTS-1225** — ppt/notesSlides/. **Parcial**.
- [ ] **FTS-1226** — ppt/notesMasters/. **A verificar**.
- [ ] **FTS-1227** — ppt/handoutMasters/. **A verificar**.
- [/] **FTS-1228** — ppt/media/. **Parcial**.
- [ ] **FTS-1229** — ppt/charts/. **A verificar**.
- [ ] **FTS-1230** — ppt/embeddings/. **A verificar**.
- [ ] **FTS-1231** — ppt/diagrams/. **A verificar**.
- [ ] **FTS-1232** — ppt/comments/. **A verificar**.
- [ ] **FTS-1233** — ppt/commentAuthors.xml. **A verificar**.
- [ ] **FTS-1234** — ppt/tableStyles.xml. **A verificar**.
- [ ] **FTS-1235** — ppt/presProps.xml. **A verificar**.
- [ ] **FTS-1236** — ppt/viewProps.xml. **A verificar**.

### 28.2. Funcionalidades do parser

- [x] **FTS-1237** — Abrir contêiner ZIP. **Feito**.
- [/] **FTS-1238** — Validar estrutura do pacote. **Parcial**.
- [x] **FTS-1239** — Ler XML. **Feito**.
- [/] **FTS-1240** — Interpretar namespaces. **Parcial**.
- [x] **FTS-1241** — Interpretar relacionamentos. **Feito**.
- [x] **FTS-1242** — Resolver referências relativas. **Feito**.
- [/] **FTS-1243** — Identificar partes obrigatórias. **Parcial**.
- [/] **FTS-1244** — Identificar partes opcionais. **Parcial**.
- [/] **FTS-1245** — Carregar apresentações. **Parcial**.
- [x] **FTS-1246** — Carregar slides. **Feito**.
- [/] **FTS-1247** — Carregar layouts. **Parcial**.
- [/] **FTS-1248** — Carregar mestres. **Parcial**.
- [/] **FTS-1249** — Carregar temas. **Parcial**.
- [x] **FTS-1250** — Carregar imagens. **Feito**.
- [ ] **FTS-1251** — Carregar vídeos. **Pendente**.
- [ ] **FTS-1252** — Carregar áudios. **Pendente**.
- [ ] **FTS-1253** — Carregar gráficos. **Pendente**.
- [ ] **FTS-1254** — Carregar tabelas. **Pendente**.
- [ ] **FTS-1255** — Carregar animações. **Pendente**.
- [ ] **FTS-1256** — Carregar transições. **Pendente**.
- [x] **FTS-1257** — Carregar anotações. **Feito**.
- [ ] **FTS-1258** — Carregar comentários. **Pendente**.
- [ ] **FTS-1259** — Carregar propriedades. **Pendente**.
- [ ] **FTS-1260** — Carregar objetos incorporados. **Pendente**.
- [ ] **FTS-1261** — Preservar partes desconhecidas. **Pendente**.
- [ ] **FTS-1262** — Reconstruir relacionamentos. **Pendente**.
- [ ] **FTS-1263** — Validar integridade referencial. **A verificar**.
- [ ] **FTS-1264** — Salvar alterações. **Pendente**.
- [ ] **FTS-1265** — Recompactar arquivo. **Pendente**.
- [ ] **FTS-1266** — Preservar compatibilidade. **Pendente**.

### 28.3. Elementos XML importantes

- [/] **FTS-1267** — p:presentation — raiz da apresentação. **Parcial**.
- [/] **FTS-1268** — p:sld — slide. **Parcial**.
- [/] **FTS-1269** — p:sldMaster — slide mestre. **Parcial**.
- [/] **FTS-1270** — p:sldLayout — layout. **Parcial**.
- [/] **FTS-1271** — p:sp — forma. **Parcial**.
- [/] **FTS-1272** — p:pic — imagem. **Parcial**.
- [ ] **FTS-1273** — p:graphicFrame — contêiner gráfico. **Pendente**.
- [/] **FTS-1274** — p:grpSp — grupo de formas. **Parcial**.
- [/] **FTS-1275** — p:cxnSp — conector. **Parcial**.
- [ ] **FTS-1276** — a:tbl — tabela. **Pendente**.
- [ ] **FTS-1277** — p:timing — animações. **Pendente**.
- [ ] **FTS-1278** — p:transition — transição. **Pendente**.
- [/] **FTS-1279** — a:r — trecho de texto. **Parcial**.
- [/] **FTS-1280** — a:p — parágrafo. **Parcial**.
- [/] **FTS-1281** — a:xfrm — transformações geométricas. **Parcial**.
- [/] **FTS-1282** — a:solidFill — preenchimento sólido. **Parcial**.
- [ ] **FTS-1283** — a:gradFill — preenchimento gradiente. **Pendente**.
- [/] **FTS-1284** — a:ln — propriedades de linha. **Parcial**.
- [/] **FTS-1285** — a:effectLst — lista de efeitos. **Parcial**.

## 29. Compatibilidade de layout

- [/] **FTS-1286** — Interpretar coordenadas EMU. **Parcial**.
- [/] **FTS-1287** — Interpretar dimensões. **Parcial**.
- [/] **FTS-1288** — Interpretar posições. **Parcial**.
- [/] **FTS-1289** — Interpretar rotações. **Parcial**.
- [ ] **FTS-1290** — Interpretar matrizes de transformação. **A verificar**.
- [/] **FTS-1291** — Interpretar escala. **Parcial**.
- [/] **FTS-1292** — Interpretar grupos. **Parcial**.
- [/] **FTS-1293** — Interpretar coordenadas relativas. **Parcial**.
- [ ] **FTS-1294** — Interpretar formas personalizadas. **Pendente**.
- [/] **FTS-1295** — Interpretar geometria predefinida. **Parcial**.
- [ ] **FTS-1296** — Interpretar caminhos Bézier. **Pendente**.
- [/] **FTS-1297** — Interpretar caixas de texto. **Parcial**.
- [/] **FTS-1298** — Interpretar margens internas. **Parcial**.
- [/] **FTS-1299** — Interpretar quebra de linha. **Parcial**.
- [/] **FTS-1300** — Interpretar alinhamento. **Parcial**.
- [/] **FTS-1301** — Interpretar herança de estilos. **Parcial**.
- [/] **FTS-1302** — Interpretar estilos de tema. **Parcial**.
- [ ] **FTS-1303** — Interpretar fontes substitutas. **A verificar**.
- [/] **FTS-1304** — Interpretar estilos de parágrafos. **Parcial**.
- [/] **FTS-1305** — Interpretar ordem de sobreposição. **Parcial**.
- [ ] **FTS-1306** — Interpretar clipping. **A verificar**.
- [ ] **FTS-1307** — Interpretar máscaras. **A verificar**.
- [/] **FTS-1308** — Interpretar transparência. **Parcial**.
- [/] **FTS-1309** — Interpretar efeitos. **Parcial**.
- [/] **FTS-1310** — Interpretar imagens. **Parcial**.
- [ ] **FTS-1311** — Interpretar gradientes. **Pendente**.
- [ ] **FTS-1312** — Interpretar padrões. **Pendente**.
- [/] **FTS-1313** — Interpretar sombras. **Parcial**.
- [ ] **FTS-1314** — Interpretar perspectiva. **Pendente**.

## 30. Compatibilidade de animações

- [ ] **FTS-1315** — Interpretar árvore temporal. **Pendente**.
- [ ] **FTS-1316** — Interpretar sequências. **Pendente**.
- [ ] **FTS-1317** — Interpretar grupos paralelos. **Pendente**.
- [ ] **FTS-1318** — Interpretar grupos sequenciais. **Pendente**.
- [ ] **FTS-1319** — Interpretar durações. **Pendente**.
- [ ] **FTS-1320** — Interpretar atrasos. **Pendente**.
- [ ] **FTS-1321** — Interpretar repetições. **Pendente**.
- [ ] **FTS-1322** — Interpretar gatilhos. **Pendente**.
- [ ] **FTS-1323** — Interpretar estados. **Pendente**.
- [ ] **FTS-1324** — Interpretar efeitos. **Pendente**.
- [ ] **FTS-1325** — Interpretar propriedades animadas. **Pendente**.
- [ ] **FTS-1326** — Interpretar trajetórias. **Pendente**.
- [ ] **FTS-1327** — Interpretar referências a objetos. **Pendente**.
- [ ] **FTS-1328** — Interpretar referências a parágrafos. **Pendente**.
- [ ] **FTS-1329** — Interpretar transições. **Pendente**.
- [ ] **FTS-1330** — Preservar animações desconhecidas. **Pendente**.
- [ ] **FTS-1331** — Salvar animações existentes sem perda. **Pendente**.
- [ ] **FTS-1332** — Renderizar efeitos compatíveis. **Pendente**.

## 31. Compatibilidade de gráficos

- [ ] **FTS-1333** — Ler partes de gráficos. **Pendente**.
- [ ] **FTS-1334** — Ler dados incorporados. **Pendente**.
- [ ] **FTS-1335** — Ler dados em cache. **Pendente**.
- [ ] **FTS-1336** — Ler planilhas incorporadas. **Pendente**.
- [ ] **FTS-1337** — Ler vínculos externos. **Pendente**.
- [ ] **FTS-1338** — Interpretar séries. **Pendente**.
- [ ] **FTS-1339** — Interpretar eixos. **Pendente**.
- [ ] **FTS-1340** — Interpretar escalas. **Pendente**.
- [ ] **FTS-1341** — Interpretar títulos. **Pendente**.
- [ ] **FTS-1342** — Interpretar legendas. **Pendente**.
- [ ] **FTS-1343** — Interpretar cores. **Pendente**.
- [ ] **FTS-1344** — Interpretar estilos. **Pendente**.
- [ ] **FTS-1345** — Interpretar dados numéricos. **Pendente**.
- [ ] **FTS-1346** — Interpretar categorias. **Pendente**.
- [ ] **FTS-1347** — Interpretar fórmulas. **Pendente**.
- [ ] **FTS-1348** — Preservar informações desconhecidas. **Pendente**.
- [ ] **FTS-1349** — Reconstruir arquivos ao salvar. **Pendente**.
- [ ] **FTS-1350** — Gerar gráficos editáveis. **Pendente**.

## 32. Compatibilidade de SmartArt

- [ ] **FTS-1351** — Interpretar diagramas. **Pendente**.
- [ ] **FTS-1352** — Interpretar dados de nós. **Pendente**.
- [ ] **FTS-1353** — Interpretar relacionamentos. **Pendente**.
- [ ] **FTS-1354** — Interpretar layouts. **Pendente**.
- [ ] **FTS-1355** — Interpretar estilos. **Pendente**.
- [ ] **FTS-1356** — Interpretar cores. **Pendente**.
- [ ] **FTS-1357** — Interpretar hierarquias. **Pendente**.
- [ ] **FTS-1358** — Interpretar posicionamento. **Pendente**.
- [ ] **FTS-1359** — Interpretar geometria. **Pendente**.
- [ ] **FTS-1360** — Renderizar diagrama. **Pendente**.
- [ ] **FTS-1361** — Editar dados do diagrama. **Pendente**.
- [ ] **FTS-1362** — Regenerar layout. **Pendente**.
- [ ] **FTS-1363** — Preservar recursos não suportados. **Pendente**.

## 33. Compatibilidade de fontes

- [ ] **FTS-1364** — Descoberta de fontes instaladas. **A verificar**.
- [/] **FTS-1365** — Carregamento de fontes. **Parcial**.
- [ ] **FTS-1366** — Substituição de fontes. **A verificar**.
- [/] **FTS-1367** — Fallback de fontes. **Parcial**.
- [/] **FTS-1368** — Suporte a OpenType. **Parcial**.
- [/] **FTS-1369** — Suporte a TrueType. **Parcial**.
- [ ] **FTS-1370** — Suporte a fontes variáveis. **A verificar**.
- [/] **FTS-1371** — Suporte a fontes incorporadas, respeitando licenças. **Parcial**.
- [/] **FTS-1372** — Suporte a Unicode. **Parcial**.
- [ ] **FTS-1373** — Suporte a ligaduras. **A verificar**.
- [ ] **FTS-1374** — Suporte a kerning. **A verificar**.
- [ ] **FTS-1375** — Suporte a escrita bidirecional. **A verificar**.
- [ ] **FTS-1376** — Suporte a escrita da direita para a esquerda. **A verificar**.
- [ ] **FTS-1377** — Suporte a idiomas asiáticos. **A verificar**.
- [ ] **FTS-1378** — Suporte a caracteres combinados. **A verificar**.
- [ ] **FTS-1379** — Suporte a emoji. **A verificar**.
- [ ] **FTS-1380** — Cálculo de métricas. **A verificar**.
- [ ] **FTS-1381** — Cálculo de altura de linha. **A verificar**.
- [ ] **FTS-1382** — Posicionamento de glifos. **A verificar**.
- [/] **FTS-1383** — Quebra de linha. **Parcial**.
- [/] **FTS-1384** — Quebra de palavra. **Parcial**.
- [/] **FTS-1385** — Renderização de texto rotacionado. **Parcial**.
- [ ] **FTS-1386** — Renderização de texto curvo. **Pendente**.
- [ ] **FTS-1387** — Preservação de layout entre sistemas. **Pendente**.

## 34. Importação e exportação


### 34.1. Importação

- [/] **FTS-1388** — PPTX. **Parcial**.
- [ ] **FTS-1389** — PPTM. **Pendente**.
- [ ] **FTS-1390** — PPSX. **Pendente**.
- [ ] **FTS-1391** — POTX. **Pendente**.
- [/] **FTS-1392** — ODP. **Parcial**.
- [ ] **FTS-1393** — PPT legado, com conversor específico. **Pendente**.
- [ ] **FTS-1394** — PDF. **Pendente**.
- [ ] **FTS-1395** — SVG. **A verificar**.
- [x] **FTS-1396** — PNG. **Feito**.
- [x] **FTS-1397** — JPEG. **Feito**.
- [ ] **FTS-1398** — GIF. **A verificar**.
- [ ] **FTS-1399** — TIFF. **A verificar**.
- [ ] **FTS-1400** — BMP. **A verificar**.
- [ ] **FTS-1401** — EMF. **A verificar**.
- [ ] **FTS-1402** — WMF. **A verificar**.
- [ ] **FTS-1403** — Áudio. **Pendente**.
- [ ] **FTS-1404** — Vídeo. **Pendente**.
- [ ] **FTS-1405** — Dados tabulares. **A verificar**.
- [ ] **FTS-1406** — Modelos 3D compatíveis. **Pendente**.

### 34.2. Exportação

- [ ] **FTS-1407** — PPTX. **Pendente**.
- [ ] **FTS-1408** — PPSX. **Pendente**.
- [ ] **FTS-1409** — POTX. **Pendente**.
- [ ] **FTS-1410** — ODP. **Pendente**.
- [/] **FTS-1411** — PDF. **Parcial**.
- [/] **FTS-1412** — PNG. **Parcial**.
- [ ] **FTS-1413** — JPEG. **Pendente**.
- [ ] **FTS-1414** — SVG. **Pendente**.
- [ ] **FTS-1415** — GIF animado. **Pendente**.
- [ ] **FTS-1416** — MP4. **Pendente**.
- [ ] **FTS-1417** — HTML. **Pendente**.
- [/] **FTS-1418** — Imagens por slide. **Parcial**.
- [ ] **FTS-1419** — Notas do apresentador. **Pendente**.
- [ ] **FTS-1420** — Folhetos. **Pendente**.
- [ ] **FTS-1421** — Slides selecionados. **A verificar**.
- [x] **FTS-1422** — Apresentação inteira. **Feito**.

### 34.3. Configurações

- [ ] **FTS-1423** — Resolução. **A verificar**.
- [ ] **FTS-1424** — Qualidade. **A verificar**.
- [ ] **FTS-1425** — Compressão. **A verificar**.
- [ ] **FTS-1426** — Transparência. **A verificar**.
- [ ] **FTS-1427** — Dimensões. **A verificar**.
- [ ] **FTS-1428** — Páginas selecionadas. **A verificar**.
- [ ] **FTS-1429** — Inclusão de notas. **A verificar**.
- [ ] **FTS-1430** — Inclusão de slides ocultos. **A verificar**.
- [ ] **FTS-1431** — Incorporação de fontes. **A verificar**.
- [ ] **FTS-1432** — Incorporação de mídia. **A verificar**.
- [ ] **FTS-1433** — Inclusão de transições. **A verificar**.
- [ ] **FTS-1434** — Inclusão de animações. **A verificar**.
- [ ] **FTS-1435** — Inclusão de narração. **A verificar**.

# PARTE V — ARQUITETURA DO PROJETO EM RUST


## 35. Núcleo de edição


### 35.1. Modelo de documentos

- [x] **FTS-1436** — Representação interna da apresentação. **Feito**.
- [x] **FTS-1437** — Representação interna dos slides. **Feito**.
- [x] **FTS-1438** — Representação interna dos objetos. **Feito**.
- [x] **FTS-1439** — Identificadores estáveis para objetos. **Feito**.
- [/] **FTS-1440** — Hierarquia de objetos. **Parcial**.
- [/] **FTS-1441** — Herança de propriedades. **Parcial**.
- [ ] **FTS-1442** — Gerenciamento de recursos compartilhados. **A verificar**.
- [/] **FTS-1443** — Gerenciamento de relações entre objetos. **Parcial**.
- [ ] **FTS-1444** — Gerenciamento de referências externas. **A verificar**.
- [ ] **FTS-1445** — Validação estrutural. **A verificar**.
- [x] **FTS-1446** — Serialização. **Feito**.
- [x] **FTS-1447** — Desserialização. **Feito**.
- [/] **FTS-1448** — Migração entre versões do modelo. **Parcial**.

### 35.2. Sistema de edição

- [x] **FTS-1449** — Desfazer. **Feito**.
- [x] **FTS-1450** — Refazer. **Feito**.
- [x] **FTS-1451** — Histórico de operações. **Feito**.
- [x] **FTS-1452** — Agrupamento de operações. **Feito**.
- [x] **FTS-1453** — Transações de edição. **Feito**.
- [ ] **FTS-1454** — Cancelamento de operações. **A verificar**.
- [x] **FTS-1455** — Operações reversíveis. **Feito**.
- [ ] **FTS-1456** — Controle de alterações. **A verificar**.
- [x] **FTS-1457** — Histórico por documento. **Feito**.
- [x] **FTS-1458** — Detecção de modificações. **Feito**.
- [ ] **FTS-1459** — Salvamento incremental. **A verificar**.
- [x] **FTS-1460** — Salvamento atômico. **Feito**.
- [ ] **FTS-1461** — Recuperação de sessão. **A verificar**.

### 35.3. Sistema de seleção

- [x] **FTS-1462** — Seleção individual. **Feito**.
- [x] **FTS-1463** — Seleção múltipla. **Feito**.
- [x] **FTS-1464** — Seleção por área. **Feito**.
- [ ] **FTS-1465** — Seleção por hierarquia. **Pendente**.
- [ ] **FTS-1466** — Seleção dentro de grupos. **A verificar**.
- [ ] **FTS-1467** — Seleção de pontos vetoriais. **Pendente**.
- [x] **FTS-1468** — Seleção de texto. **Feito**.
- [x] **FTS-1469** — Seleção de células. **Feito**.
- [x] **FTS-1470** — Seleção de slides. **Feito**.
- [ ] **FTS-1471** — Seleção de elementos sobrepostos. **A verificar**.

### 35.4. Transformações

- [x] **FTS-1472** — Translação. **Feito**.
- [x] **FTS-1473** — Rotação. **Feito**.
- [x] **FTS-1474** — Escala. **Feito**.
- [ ] **FTS-1475** — Espelhamento. **A verificar**.
- [x] **FTS-1476** — Alinhamento. **Feito**.
- [x] **FTS-1477** — Distribuição. **Feito**.
- [x] **FTS-1478** — Agrupamento. **Feito**.
- [x] **FTS-1479** — Desagrupamento. **Feito**.
- [ ] **FTS-1480** — Transformação de coordenadas. **A verificar**.
- [ ] **FTS-1481** — Transformação de grupos aninhados. **A verificar**.
- [x] **FTS-1482** — Ajuste de proporções. **Feito**.
- [ ] **FTS-1483** — Ajuste magnético. **A verificar**.

## 36. Renderização gráfica


### 36.1. Renderização 2D

- [x] **FTS-1484** — Formas vetoriais. **Feito**.
- [x] **FTS-1485** — Caminhos Bézier. **Feito**.
- [x] **FTS-1486** — Linhas. **Feito**.
- [x] **FTS-1487** — Curvas. **Feito**.
- [x] **FTS-1488** — Preenchimentos. **Feito**.
- [ ] **FTS-1489** — Gradientes. **Pendente**.
- [ ] **FTS-1490** — Padrões. **Pendente**.
- [x] **FTS-1491** — Transparência. **Feito**.
- [/] **FTS-1492** — Máscaras. **Parcial**.
- [/] **FTS-1493** — Clipping. **Parcial**.
- [/] **FTS-1494** — Sombras. **Parcial**.
- [ ] **FTS-1495** — Reflexos. **Pendente**.
- [ ] **FTS-1496** — Brilhos. **Pendente**.
- [ ] **FTS-1497** — Desfoques. **A verificar**.
- [x] **FTS-1498** — Composição de camadas. **Feito**.
- [x] **FTS-1499** — Transformações. **Feito**.
- [x] **FTS-1500** — Antialiasing. **Feito**.
- [x] **FTS-1501** — Renderização de imagens. **Feito**.
- [x] **FTS-1502** — Renderização de texto. **Feito**.
- [x] **FTS-1503** — Renderização de tabelas. **Feito**.
- [x] **FTS-1504** — Renderização de gráficos. **Feito**.
- [ ] **FTS-1505** — Renderização de diagramas. **Pendente**.

### 36.2. Renderização de apresentações

- [x] **FTS-1506** — Renderização de slide estático. **Feito**.
- [x] **FTS-1507** — Renderização de transições. **Feito**.
- [x] **FTS-1508** — Renderização de animações. **Feito**.
- [x] **FTS-1509** — Renderização em tempo real. **Feito**.
- [x] **FTS-1510** — Renderização fora da tela. **Feito**.
- [ ] **FTS-1511** — Renderização em alta resolução. **A verificar**.
- [ ] **FTS-1512** — Exportação de quadros. **A verificar**.
- [x] **FTS-1513** — Renderização para impressão. **Feito**.
- [ ] **FTS-1514** — Renderização para vídeo. **Pendente**.
- [ ] **FTS-1515** — Controle de FPS. **A verificar**.
- [ ] **FTS-1516** — Sincronização com áudio. **Pendente**.
- [ ] **FTS-1517** — Sincronização com vídeo. **Pendente**.

### 36.3. Desempenho

- [ ] **FTS-1518** — Cache de texturas. **A verificar**.
- [ ] **FTS-1519** — Cache de glifos. **A verificar**.
- [ ] **FTS-1520** — Cache de objetos. **A verificar**.
- [x] **FTS-1521** — Cache de slides. **Feito**.
- [ ] **FTS-1522** — Renderização parcial. **A verificar**.
- [ ] **FTS-1523** — Invalidação de regiões modificadas. **A verificar**.
- [ ] **FTS-1524** — Carregamento sob demanda. **A verificar**.
- [ ] **FTS-1525** — Pré-carregamento do próximo slide. **A verificar**.
- [x] **FTS-1526** — Processamento assíncrono. **Feito**.
- [ ] **FTS-1527** — Uso opcional de GPU. **A verificar**.
- [ ] **FTS-1528** — Limitação de consumo de memória. **A verificar**.
- [ ] **FTS-1529** — Liberação de recursos não utilizados. **A verificar**.

## 37. Impressão

- [/] **FTS-1530** — Imprimir apresentação completa. **Parcial**.
- [ ] **FTS-1531** — Imprimir slides selecionados. **Pendente**.
- [/] **FTS-1532** — Imprimir slide atual. **Parcial**.
- [ ] **FTS-1533** — Imprimir slides ocultos. **Pendente**.
- [ ] **FTS-1534** — Imprimir anotações. **Pendente**.
- [ ] **FTS-1535** — Imprimir folhetos. **Pendente**.
- [ ] **FTS-1536** — Imprimir estrutura de tópicos. **Pendente**.
- [/] **FTS-1537** — Imprimir um slide por página. **Parcial**.
- [ ] **FTS-1538** — Imprimir múltiplos slides por página. **Pendente**.
- [ ] **FTS-1539** — Configurar orientação. **A verificar**.
- [ ] **FTS-1540** — Configurar papel. **A verificar**.
- [ ] **FTS-1541** — Configurar margens. **A verificar**.
- [/] **FTS-1542** — Ajustar ao papel. **Parcial**.
- [ ] **FTS-1543** — Impressão colorida. **A verificar**.
- [ ] **FTS-1544** — Impressão em escala de cinza. **A verificar**.
- [ ] **FTS-1545** — Impressão em preto e branco. **A verificar**.
- [ ] **FTS-1546** — Adicionar moldura. **A verificar**.
- [ ] **FTS-1547** — Imprimir número do slide. **A verificar**.
- [ ] **FTS-1548** — Imprimir comentários. **A verificar**.
- [ ] **FTS-1549** — Visualizar impressão. **Pendente**.
- [/] **FTS-1550** — Exportar impressão para PDF. **Parcial**.

## 38. Configurações e preferências

- [ ] **FTS-1551** — Idioma da interface. **Pendente**.
- [/] **FTS-1552** — Idioma de revisão. **Parcial**.
- [x] **FTS-1553** — Tema claro. **Feito**.
- [x] **FTS-1554** — Tema escuro. **Feito**.
- [ ] **FTS-1555** — Tema de alto contraste. **A verificar**.
- [ ] **FTS-1556** — Escala da interface. **A verificar**.
- [ ] **FTS-1557** — Configuração de fontes padrão. **A verificar**.
- [ ] **FTS-1558** — Diretório padrão. **A verificar**.
- [x] **FTS-1559** — Intervalo de salvamento automático. **Feito**.
- [ ] **FTS-1560** — Quantidade de operações de desfazer. **A verificar**.
- [ ] **FTS-1561** — Configurações de desempenho. **A verificar**.
- [ ] **FTS-1562** — Configurações de renderização. **A verificar**.
- [ ] **FTS-1563** — Configurações de GPU. **A verificar**.
- [ ] **FTS-1564** — Configurações de cache. **A verificar**.
- [ ] **FTS-1565** — Configurações de acessibilidade. **A verificar**.
- [ ] **FTS-1566** — Configurações de privacidade. **A verificar**.
- [ ] **FTS-1567** — Configurações de telemetria. **A verificar**.
- [x] **FTS-1568** — Personalização de atalhos. **Feito**.
- [ ] **FTS-1569** — Personalização de menus. **A verificar**.
- [ ] **FTS-1570** — Personalização de ferramentas. **A verificar**.
- [ ] **FTS-1571** — Gerenciamento de extensões. **Pendente**.
- [ ] **FTS-1572** — Gerenciamento de atualizações. **Pendente**.
- [ ] **FTS-1573** — Gerenciamento de modelos. **A verificar**.

## 39. Internacionalização

- [ ] **FTS-1574** — Interface multilíngue. **Pendente**.
- [x] **FTS-1575** — Português brasileiro. **Feito**.
- [ ] **FTS-1576** — Inglês. **Pendente**.
- [ ] **FTS-1577** — Espanhol. **Pendente**.
- [ ] **FTS-1578** — Outros idiomas. **Pendente**.
- [ ] **FTS-1579** — Formatação regional. **A verificar**.
- [ ] **FTS-1580** — Direção do texto. **A verificar**.
- [ ] **FTS-1581** — Suporte RTL. **A verificar**.
- [x] **FTS-1582** — Unicode. **Feito**.
- [ ] **FTS-1583** — Fontes alternativas por idioma. **A verificar**.
- [/] **FTS-1584** — Corretor ortográfico por idioma. **Parcial**.
- [ ] **FTS-1585** — Separadores decimais. **A verificar**.
- [ ] **FTS-1586** — Formatos de data. **A verificar**.
- [ ] **FTS-1587** — Formatos de hora. **A verificar**.
- [ ] **FTS-1588** — Ordenação linguística. **A verificar**.
- [ ] **FTS-1589** — Atalhos adaptados ao idioma. **Pendente**.

## 40. Testes e qualidade


### 40.1. Testes unitários

- [x] **FTS-1590** — Parser XML. **Feito**.
- [x] **FTS-1591** — Parser PPTX. **Feito**.
- [x] **FTS-1592** — Serialização. **Feito**.
- [x] **FTS-1593** — Geometria. **Feito**.
- [x] **FTS-1594** — Transformações. **Feito**.
- [x] **FTS-1595** — Texto. **Feito**.
- [x] **FTS-1596** — Layout. **Feito**.
- [x] **FTS-1597** — Tabelas. **Feito**.
- [x] **FTS-1598** — Gráficos. **Feito**.
- [x] **FTS-1599** — Animações. **Feito**.
- [x] **FTS-1600** — Transições. **Feito**.
- [x] **FTS-1601** — Temas. **Feito**.
- [ ] **FTS-1602** — Mestres. **A verificar**.

### 40.2. Testes de integração

- [x] **FTS-1603** — Abrir PPTX. **Feito**.
- [ ] **FTS-1604** — Salvar PPTX. **Pendente**.
- [ ] **FTS-1605** — Reabrir PPTX salvo. **Pendente**.
- [ ] **FTS-1606** — Importar slides. **A verificar**.
- [ ] **FTS-1607** — Exportar slides. **A verificar**.
- [x] **FTS-1608** — Exportar PDF. **Feito**.
- [ ] **FTS-1609** — Exportar imagens. **A verificar**.
- [ ] **FTS-1610** — Exportar vídeo. **Pendente**.
- [ ] **FTS-1611** — Preservar recursos. **A verificar**.
- [ ] **FTS-1612** — Verificar relacionamentos internos. **A verificar**.
- [ ] **FTS-1613** — Verificar arquivos incorporados. **A verificar**.

### 40.3. Testes visuais

- [ ] **FTS-1614** — Comparar renderização com PowerPoint. **A verificar**.
- [ ] **FTS-1615** — Comparar posicionamento. **A verificar**.
- [ ] **FTS-1616** — Comparar fontes. **A verificar**.
- [ ] **FTS-1617** — Comparar cores. **A verificar**.
- [ ] **FTS-1618** — Comparar gradientes. **A verificar**.
- [ ] **FTS-1619** — Comparar tabelas. **A verificar**.
- [ ] **FTS-1620** — Comparar gráficos. **A verificar**.
- [ ] **FTS-1621** — Comparar imagens. **A verificar**.
- [ ] **FTS-1622** — Comparar sombras. **A verificar**.
- [ ] **FTS-1623** — Comparar animações. **A verificar**.
- [ ] **FTS-1624** — Comparar transições. **A verificar**.

### 40.4. Testes de robustez

- [ ] **FTS-1625** — Arquivos corrompidos. **A verificar**.
- [ ] **FTS-1626** — ZIP malformado. **A verificar**.
- [ ] **FTS-1627** — XML inválido. **A verificar**.
- [ ] **FTS-1628** — Referências quebradas. **A verificar**.
- [ ] **FTS-1629** — Arquivos grandes. **A verificar**.
- [ ] **FTS-1630** — Apresentações com milhares de slides. **A verificar**.
- [ ] **FTS-1631** — Imagens de alta resolução. **A verificar**.
- [ ] **FTS-1632** — Muitas animações simultâneas. **A verificar**.
- [ ] **FTS-1633** — Objetos profundamente aninhados. **A verificar**.
- [ ] **FTS-1634** — Fontes ausentes. **A verificar**.
- [ ] **FTS-1635** — Recursos desconhecidos. **A verificar**.
- [ ] **FTS-1636** — Arquivos contendo macros. **A verificar**.
- [ ] **FTS-1637** — Consumo excessivo de memória. **A verificar**.
- [ ] **FTS-1638** — Fuzz testing dos importadores. **A verificar**.
- [ ] **FTS-1639** — Testes de regressão. **A verificar**.

# PARTE VI — ORGANIZAÇÃO RECOMENDADA EM RUST


## 41. Estrutura do projeto

- [x] **FTS-1640** — Separar modelo do documento e formato PPTX. **Feito**.
- [x] **FTS-1641** — Organizar crates independentes. **Feito**.
- [x] **FTS-1642** — Núcleo de documento, slides, objetos, temas e histórico. **Feito**.
- [ ] **FTS-1643** — Pacote PPTX: OPC, parser, serialização, relacionamentos e compatibilidade. **A verificar**.
- [ ] **FTS-1644** — Renderização: geometria, texto, imagens, efeitos e composição. **A verificar**.
- [ ] **FTS-1645** — Animação: linha do tempo, transições, interpolação e gatilhos. **A verificar**.
- [ ] **FTS-1646** — Gráficos: dados, eixos, séries e desenho. **A verificar**.
- [ ] **FTS-1647** — Mídia: áudio, vídeo e sincronização. **A verificar**.
- [ ] **FTS-1648** — Exportação: PDF, SVG, imagens e vídeo. **A verificar**.
- [ ] **FTS-1649** — Editor: comandos, seleção, ferramentas e área de transferência. **A verificar**.
- [ ] **FTS-1650** — Plugins: API, registro e sandbox. **A verificar**.
- [x] **FTS-1651** — Aplicativo desktop. **Feito**.
- [ ] **FTS-1652** — Aplicativo CLI. **A verificar**.
- [ ] **FTS-1653** — Fixtures, testes de compatibilidade e testes visuais. **A verificar**.
- [x] **FTS-1654** — Documentação versionada. **Feito**.
- [ ] **FTS-1655** — Avaliar workspace Cargo. **A verificar**.

Estrutura de referência enviada pelo usuário (proposta; os nomes não exigem
renomear os crates existentes):

```text
rust-presentation/
├── Cargo.toml
├── crates/
│   ├── presentation-core/       document, slides, objects, themes, history
│   ├── presentation-pptx/       opc, parser, serializer, relationships, compatibility
│   ├── presentation-render/     geometry, text, images, effects, compositing
│   ├── presentation-animation/  timeline, transitions, interpolation, triggers
│   ├── presentation-charts/     data, axes, series, rendering
│   ├── presentation-media/      audio, video, synchronization
│   ├── presentation-export/     pdf, svg, image, video
│   ├── presentation-editor/     commands, selection, tools, clipboard
│   └── presentation-plugins/    api, registry, sandbox
├── apps/
│   ├── desktop/
│   └── cli/
├── tests/
│   ├── fixtures/
│   ├── compatibility/
│   └── visual/
└── docs/
```

## 42. Bibliotecas Rust para avaliação

- [x] **FTS-1656** — Interface desktop — Iced (escolha atual). **Feito**.
- [ ] **FTS-1657** — Renderização 2D — Vello, tiny-skia ou Skia via bindings. **A verificar**.
- [x] **FTS-1658** — GPU — wgpu. **Feito**.
- [x] **FTS-1659** — XML — quick-xml. **Feito**.
- [x] **FTS-1660** — ZIP — zip. **Feito**.
- [x] **FTS-1661** — Serialização — serde. **Feito**.
- [ ] **FTS-1662** — Fontes — fontdb. **A verificar**.
- [ ] **FTS-1663** — Formatação de texto — cosmic-text. **A verificar**.
- [ ] **FTS-1664** — Shaping de texto — rustybuzz. **A verificar**.
- [ ] **FTS-1665** — SVG — resvg e usvg. **A verificar**.
- [ ] **FTS-1666** — Geometria — kurbo. **A verificar**.
- [x] **FTS-1667** — Imagens — image. **Feito**.
- [ ] **FTS-1668** — PDF — printpdf, conforme requisitos. **A verificar**.
- [ ] **FTS-1669** — Processamento assíncrono — tokio. **A verificar**.
- [ ] **FTS-1670** — IDs de objetos — uuid. **A verificar**.

## 43. Prioridades de implementação

- [/] **FTS-1671** — Fase 1 — MVP: slides, texto, formas, imagens, temas básicos e PPTX. **Parcial**.
- [/] **FTS-1672** — Fase 2 — edição avançada: tabelas, gráficos, mestres, gradientes e efeitos. **Parcial**.
- [/] **FTS-1673** — Fase 3 — apresentação: transições, animações, áudio, vídeo e modo apresentador. **Parcial**.
- [/] **FTS-1674** — Fase 4 — compatibilidade: SmartArt, OLE, Morph, diagramas e objetos desconhecidos. **Parcial**.
- [/] **FTS-1675** — Fase 5 — ecossistema: plugins, colaboração, APIs, IA e automações. **Parcial**.
- [x] **FTS-1676** — Modelo interno independente do XML da Microsoft. **Feito**.
- [ ] **FTS-1677** — Preservar dados e relacionamentos de recursos não interpretados. **Pendente**.

Exemplo de separação do modelo interno apresentado no inventário. É apenas
um esqueleto ilustrativo, não uma substituição do modelo atual do FeatherSlide:

```rust
pub struct Presentation {
    pub slides: Vec<Slide>,
    pub themes: Vec<Theme>,
}

pub struct Slide {
    pub id: SlideId,
    pub elements: Vec<Element>,
}

pub enum Element {
    Text(TextElement),
    Shape(ShapeElement),
    Image(ImageElement),
    Table(TableElement),
    Chart(ChartElement),
    Group(GroupElement),
}
```

O modelo completo também precisa representar mestres, layouts, relacionamentos,
animações, notas, recursos compartilhados e metadados, além de conservar partes
originais ainda não interpretadas pelo editor.

## 44. Referências técnicas essenciais

- [ ] **FTS-1678** — ISO/IEC 29500 — especificação Office Open XML. **Referência**.
- [ ] **FTS-1679** — Estrutura do PresentationML — organização dos arquivos PPTX. **Referência**.
- [ ] **FTS-1680** — Animações em PresentationML — animações e transições. **Referência**.
- [ ] **FTS-1681** — Slides mestres — herança e layouts. **Referência**.
- [ ] **FTS-1682** — API oficial do PowerPoint — modelo de objetos. **Referência**.
- [ ] **FTS-1683** — ECMA-376 — especificação Office Open XML. **Referência**.
- [ ] **FTS-1684** — Microsoft Support — funcionalidades para usuários. **Referência**.

## Notas de compatibilidade e escopo

- O pacote PPTX combina XML, recursos e relacionamentos. Prefixos XML podem
  variar; a interpretação deve respeitar os namespaces.
- Nem todo formato listado é uma apresentação completa; alguns representam
  recursos inseridos, modelos, temas ou formatos apenas de exportação.
- SmartArt deve preservar nós, dados e layout editável; uma imagem não equivale
  a suporte completo. Renderização estática com preservação dos dados pode ser uma etapa.
- Animações e transições importadas precisam preservar `p:timing` e `p:transition`.
- Mestres e layouts são estruturas do documento, além de ferramentas visuais.
- OLE e ActiveX podem ser preservados sem execução. Macros e objetos incorporados
  não são equivalentes à API local MCP nem ao sistema de plugins proposto.
- Colaboração e serviços conectados são candidatos opcionais. A suíte mantém
  uso local, sem conta e sem telemetria; este inventário não altera essas decisões.
- Equações estruturadas, LaTeX e OMML não equivalem a inserir símbolos Unicode.
- Métricas, fontes e quebra de linha precisam de validação para fidelidade visual.
- Preservar partes e relacionamentos desconhecidos é requisito próprio; importar
  conteúdo visível para o modelo interno não comprova ida e volta PPTX sem perda.
- A proposta enviada também citava egui e Slint como alternativas de interface.
  A escolha vigente do FeatherOffice é Iced. Outras bibliotecas permanecem apenas
  referências para avaliação, sem adoção automática.

## Links de referência fornecidos no inventário

Links preservados como referência; não foram usados para certificar o suporte
do FeatherSlide nesta verificação do código local.

- [ISO/IEC 29500](https://www.iso.org/standard/71691.html)
- [Estrutura do PresentationML](https://learn.microsoft.com/en-us/office/open-xml/presentation/structure-of-a-presentationml-document)
- [Animações em PresentationML](https://learn.microsoft.com/en-us/office/open-xml/presentation/working-with-animation)
- [Slides mestres](https://learn.microsoft.com/en-us/office/open-xml/presentation/working-with-slide-masters)
- [API oficial do PowerPoint](https://learn.microsoft.com/en-us/javascript/api/powerpoint)
- [Conjuntos de funcionalidades da API](https://learn.microsoft.com/pt-br/javascript/api/requirement-sets/powerpoint/powerpoint-api-requirement-sets)
- [ECMA-376](https://ecma-international.org/publications-and-standards/standards/ecma-376/)
- [Microsoft Support — PowerPoint](https://support.microsoft.com/powerpoint)

## Registro por funcionalidade

Para detalhar uma entrada, usar: ID; categoria/subcategoria; funcionalidade;
subfuncionalidade; origem (PowerPoint, requisito de formato ou proposta do
FeatherSlide); plataforma; leitura PPTX; gravação PPTX; prioridade; estado;
implementação; teste; validação visual; limitações e data da verificação.

Nenhum total deste inventário define automaticamente um percentual de conclusão:
há itens duplicados por contexto, requisitos de tamanhos diferentes e propostas
opcionais. A estimativa funcional de ~40% permanece a avaliação do usuário.
