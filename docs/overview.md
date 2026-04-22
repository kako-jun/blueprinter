# blueprinter Overview

Last updated: 2026-04-22

## What is blueprinter?

**blueprinter** is a CLI tool that renders diagrams in a hand-drawn, sketchy style.
It takes Mermaid definitions, draw.io SVG exports, or any arbitrary SVG as input,
and produces stylized output in SVG, PNG, or WebP format.

The core idea: **do not recompute layout**. Instead, take an already-laid-out SVG
and transform its visual appearance — strokes, fills, and filters — to mimic
imperfection, human handwriting, and analog media.

## Intended Use Case

You have a clean, precise diagram made in Mermaid, draw.io, or another tool.
You want to present it with personality: a blueprint draft, a chalkboard sketch,
a watercolor painting, a manga panel. blueprinter applies the aesthetic filter
without forcing you to redraw anything.

## Design Philosophy

### Layout is Input, Appearance is Output

blueprinter does not calculate positions, box sizes, or edge routes.
It assumes the input SVG already encodes a valid layout.
Its job is purely visual transformation: replace straight strokes with wobbly ones,
apply texture filters, swap color palettes, and add subtle random offsets.

This constraint keeps the architecture simple and makes the tool composable:
any SVG-producing tool can be a front-end.

### SVG-first Pipeline

The internal pipeline is always SVG → SVG → raster.
Even when the final output is PNG or WebP, the styling pass produces an intermediate SVG.
This preserves vector quality for downstream editing and makes the transformation
inspectable and debuggable.

### Randomness with Reproducibility

Hand-drawn style requires variation. Every run produces a slightly different result.
However, `--seed` locks the random number generator, making output deterministic
for documentation builds, CI snapshots, or collaborative reviews.

### No Editor

blueprinter is a filter, not an editor. There is no GUI, no canvas, no drag-and-drop.
You create diagrams in the tools you already know, then run blueprinter to stylize them.
This keeps the scope bounded and the codebase maintainable.

## Architecture

```
Input (Mermaid / draw.io SVG / any SVG)
    │
    ▼
[ Mermaid parser / SVG loader ]
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
    ▼
[ Rasterizer (resvg) ]  ──optional──►  PNG / WebP
    │
    ▼
Output SVG
```

## Themes (Planned)

| Theme | Description |
|---|---|
| `blueprint` | Default. Technical drawing on blue grid paper. |
| `sumi` | Japanese ink wash painting on washi paper. |
| `chalk` | White chalk on a blackboard. |
| `marker` | Bold neon marker strokes on dark background. |
| `watercolor` | Soft pigment bleeding and paper grain. |
| `manga` | Screentone patterns and speed lines. |

## Technology Stack

- **Rust** — CLI and pipeline
- **clap** — CLI argument parsing and subcommands
- **SVG parsing** — roxmltree or similar for SVG DOM manipulation
- **resvg** — SVG rasterization for PNG/WebP output
- **mmdc** — External Mermaid CLI for Mermaid → SVG conversion
