# blueprinter Overview

Last updated: 2026-04-27

## What is blueprinter?

**blueprinter** is a CLI tool for turning embedded visuals into a hand-drawn, sketchy style.
It accepts arbitrary SVG via `transform`, Mermaid (through external `mmdc`) via `render`,
and Markdown documents containing one or more supported embedded visual blocks via `md`,
and produces stylized SVG, PNG, or WebP output. Today the Markdown path handles `mermaid`
blocks; the next planned expansion is `latex-render` blocks so lists, tables, and editorial
layouts can be authored beside Markdown and rendered as static visual cards. draw.io direct
input is a planned follow-up phase.

The core idea: **do not recompute layout unless a front-end format requires it**.
When blueprinter receives SVG, it preserves the existing geometry and transforms
visual appearance — strokes, fills, and filters — to mimic imperfection, human
handwriting, and analog media. When blueprinter receives higher-level embedded
formats through the Markdown pipeline, those formats are expected to compile into
an SVG first, then flow through the same styling stages.

## Intended Use Case

You have a clean, precise diagram made in Mermaid, draw.io, or another tool.
Or you have a block of structured prose — a feature list, a comparison table,
an editorial callout — that looks flat in plain Markdown. You want to present
it with personality: a blueprint draft, a chalkboard sketch, a watercolor
painting, a manga panel, or eventually a newspaper-like card authored in
`latex-render`. blueprinter applies the aesthetic filter without forcing you
to redraw anything.

## Design Philosophy

### Layout is Input, Appearance is Output

blueprinter does not calculate positions, box sizes, or edge routes for raw SVG input.
It assumes the input SVG already encodes a valid layout.
Its job is purely visual transformation: replace straight strokes with wobbly ones,
apply texture filters, swap color palettes, and add subtle random offsets.

This constraint keeps the architecture simple and makes the tool composable:
any SVG-producing tool can be a front-end. For higher-level embedded formats such
as Mermaid or planned `latex-render`, layout belongs to the upstream compiler, not
to blueprinter's styling stage.

### SVG-first Pipeline

The internal pipeline is SVG → SVG first.
PNG and WebP export are planned later, and will rasterize from the transformed SVG.
This preserves vector quality for downstream editing and makes the transformation
inspectable and debuggable.

The current serializer preserves non-jittered element structure, attributes,
namespaces, and text, but it does not preserve XML declarations, comments,
processing instructions, doctypes, or CDATA boundaries yet. Non-visual
definition containers such as `defs`, `symbol`, and `marker` are intentionally
left unchanged; shapes referenced via `use` therefore remain as authored until
symbol-level styling is implemented.

### Randomness with Reproducibility

Hand-drawn style requires variation. Every run produces a slightly different result.
However, `--seed` locks the random number generator, making output deterministic
for documentation builds, CI snapshots, or collaborative reviews. Determinism is
defined for the same SVG structure; adding or removing earlier jittered elements
can change the seeded jitter applied to later elements. The current CLI also exposes
`--jitter-amplitude`, `--jitter-frequency`, and `--jitter-stroke-width-var` so
users can compare subtle and rough variants intentionally instead of relying on
a single hardcoded style. Text can also be overridden with `--font-family`; if no
override is provided, blueprinter preserves the font choice already encoded in the
input SVG and applies only subtle seeded `rotation` and `opacity` jitter while
preserving the original text layout box.

### No Editor

blueprinter is a filter, not an editor. There is no GUI, no canvas, no drag-and-drop.
You create diagrams in the tools you already know, then run blueprinter to stylize them.
This keeps the scope bounded and the codebase maintainable.

## Architecture

```
Input format
    │
    ├── SVG ───────────────► [ SVG loader ]
    │
    ├── Mermaid ───────────► [ mmdc ]
    │                         │
    │                         ▼
    └── latex-render (planned)► [ TeX/DSL compiler ]
                              │
                              ▼
                          Intermediate SVG
    │
    ▼
[ Layout-preserving SVG filter ]
    │    ├── Stroke wobble
    │    ├── Fill texture
    │    ├── Color palette swap (theme)
    │    └── Random offset / jitter
    ▼
Intermediate SVG
    │
    ├──► Output SVG
    │
    └──► [ Rasterizer (resvg) ]  ──optional──►  PNG / WebP (lossless)
```

For Markdown input, `md` acts as an orchestrator: find supported fenced blocks,
render each block into SVG or raster output, place the assets in a sibling
directory, and eventually emit a companion Markdown file with the original block
replaced by image references.

## Themes

| Theme | Status | Description |
|---|---|---|
| `blueprint` | Implemented | Technical-drawing aesthetic: dark blue background with light line strokes. |
| `sumi` | Implemented | Japanese ink wash painting with grayscale strokes and Gaussian bleed. |
| `watercolor` | Implemented | Soft pastel palette, color-mixing bleed, and stroke replicas for diffuse pigment. |
| `chalk` | Implemented | White (and pale color) chalk on a slate-green chalkboard, with a turbulence-driven dust filter that breaks each stroke up. |
| `marker` | Implemented | Six-color neon highlighter palette on a dark navy sketchbook, with a Gaussian-blur halo behind each shape and translucent palette fills. |
| `manga` | Implemented | Pure black ink on white paper, with three SVG `<pattern>` screentones (sparse dots / dense dots / diagonal lines) sampled per closed shape. Speed lines are out of scope (would require layout). |

## Technology Stack

- **Rust** — CLI and pipeline
- **clap** — CLI argument parsing and subcommands
- **SVG parsing** — roxmltree or similar for SVG DOM manipulation
- **resvg** — planned SVG rasterization for PNG/WebP output
- **mmdc** — external Mermaid CLI invoked by the `render` subcommand for Mermaid → SVG conversion
