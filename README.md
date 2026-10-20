# FeatherSlide

**Apresentações da suíte FeatherWorks, escritas em Rust com interface nativa
Iced** — leve, rápida e sem WebKit. Abre `.pptx` e `.odp`, grava no formato
próprio `.fsdz` e roda inteira na sua máquina.

> **Estado:** em desenvolvimento ativo, com cerca de **40% das funcionalidades**
> do inventário prontas. Versão alpha prevista para **30/10/2026**.

---

## O que já faz

| Área | Destaques |
|---|---|
| **Slides** | edição de slides e elementos com desfazer, layouts com fundo desenhado, texto com formatação |
| **Formas** | catálogo de formas, ícones, SmartArt e WordArt pelo motor comum [`feather-shapes`](https://github.com/feather-works/feather-shapes) |
| **Gráficos e tabelas** | gráficos pelo [`feather-charts`](https://github.com/feather-works/feather-charts), tabelas com estrutura editável, exportar gráfico em SVG |
| **Arquivos** | abre `.pptx` e `.odp`; grava `.fsdz`; exporta PDF e imagem |
| **Ortografia** | verificação com dicionários Hunspell lidos do disco, tudo local |
| **IA local** | servidor MCP (`featherslide-mcp`) para uma IA ver, editar e criar apresentações, por stdio ou ao vivo no aplicativo |

Inventário completo: [`docs/INVENTARIO-FUNCIONALIDADES.md`](docs/INVENTARIO-FUNCIONALIDADES.md) ·
evidências da verificação: [`docs/VERIFICACAO-FUNCIONALIDADES.md`](docs/VERIFICACAO-FUNCIONALIDADES.md).

## Formato `.fsdz`

Contêiner compactado com **histórico Git embutido** (feito com
[`gix`](https://github.com/Byron/gitoxide), Rust puro): as versões da
apresentação ficam dentro do arquivo, sem servidor.

## Estrutura

| Crate | Papel |
|---|---|
| [`featherslide-core`](crates/featherslide-core) | a apresentação (slides e elementos), a edição com desfazer e o quadro de cada slide |
| [`featherslide-io`](crates/featherslide-io) | `.fsdz`, leitura de `.pptx` e `.odp`, PDF e imagem |
| [`featherslide-gui`](crates/featherslide-gui) | a interface nativa em Iced |
| [`featherslide-tabelas`](crates/featherslide-tabelas) | edição das tabelas pelo motor comum `feather-tables` |
| [`featherslide-ortografia`](crates/featherslide-ortografia) | verificação ortográfica Hunspell |
| [`featherslide-mcp`](crates/featherslide-mcp) | servidor MCP para IA local |
| [`featherslide-ponte`](crates/featherslide-ponte) | despacho legado de comandos |

## Compilar

As bibliotecas da suíte vêm do GitHub por tag; o repositório compila sozinho.

```sh
cd crates/featherslide-gui
cargo run --release
```

Instaladores (`.deb`, `.tar.gz` e Windows): `pacote/gerar.sh`.

## Privacidade

Tudo roda localmente. Nenhuma apresentação sai da máquina; não há conta, login,
telemetria nem servidor.

## Documentação

- [Arquitetura](docs/ARQUITETURA.md)
- [Personalização de formas e texto](docs/PERSONALIZACAO-FORMAS-TEXTO.md)
- [Registro de tarefas](docs/REGISTRO-DE-TAREFAS.md) — histórico das entregas até a separação do repositório

## Licença

[GPL-3.0](LICENSE). As fontes Liberation em `assets/fontes/` seguem a licença SIL Open Font License.
