# FeatherSlide

**Presentations for the FeatherWorks suite, written in Rust with a native
interface on [pelicano](https://github.com/feather-works/pelicano)** — light,
fast and without WebKit. Opens `.pptx` and `.odp`, saves to its own `.fsdz`
format and runs entirely on your machine.

> **Status:** in active development, with about **40% of the features** in the
> inventory done.

---

## What it does

| Area | Highlights |
|---|---|
| **Slides** | slide and element editing with undo, layouts with drawn backgrounds, formatted text |
| **Presenting** | full screen, presenter view on a second screen, reading mode in a window, rehearse timings |
| **Shapes** | shape catalog, icons, SmartArt and WordArt through the shared [`feather-shapes`](https://github.com/feather-works/feather-shapes) engine |
| **Charts and tables** | charts through [`feather-charts`](https://github.com/feather-works/feather-charts), tables with editable structure, chart export as SVG |
| **Files** | opens `.pptx` and `.odp` at their own slide size; saves `.fsdz`; exports PDF and image. Autosave and recovery points are written off the window thread |
| **Spelling** | Hunspell dictionaries read from disk, all local |
| **Local AI** | MCP server (`featherslide-mcp`) so an AI can view, edit and create presentations, over stdio or live in the open app |

Full inventory: [`docs/INVENTARIO-FUNCIONALIDADES.md`](docs/INVENTARIO-FUNCIONALIDADES.md) ·
verification evidence: [`docs/VERIFICACAO-FUNCIONALIDADES.md`](docs/VERIFICACAO-FUNCIONALIDADES.md).

## The `.fsdz` format

A ZIP container with the presentation as Arrow tables (`data/*.arrow`) and the images
and fonts under `assets/`. No JSON inside. Each save keeps the previous version in
`.<name>.versoes/` next to the file (the 20 most recent).

## Layout

| Crate | Role |
|---|---|
| [`featherslide`](crates/featherslide) | the app: native interface on pelicano and [`feather-ui`](https://github.com/feather-works/feather-ui) |
| [`featherslide-core`](crates/featherslide-core) | the presentation (slides and elements), editing with undo and the frame of each slide |
| [`featherslide-io`](crates/featherslide-io) | `.fsdz`, `.pptx` and `.odp` reading, PDF and image |
| [`featherslide-tabelas`](crates/featherslide-tabelas) | table editing through the shared `feather-tables` engine |
| [`featherslide-ortografia`](crates/featherslide-ortografia) | Hunspell spell checking |
| [`featherslide-mcp`](crates/featherslide-mcp) | MCP server for a local AI |

## Build

The suite libraries come from GitHub by tag or revision; the repository builds on its own.

```sh
cd crates/featherslide
cargo run --release
```

Installers (`.deb`, `.tar.gz` and Windows): `pacote/gerar.sh`.

## Privacy

Everything runs locally. No presentation leaves the machine; there is no account,
login, telemetry or server.

## Documentation

- [Architecture](docs/ARQUITETURA.md)
- [Shape and text customization](docs/PERSONALIZACAO-FORMAS-TEXTO.md)
- [Task log](docs/REGISTRO-DE-TAREFAS.md) — delivery history up to the repository split

## License

[GPL-3.0](LICENSE). The Liberation fonts in `assets/fontes/` are under the SIL Open Font License.
