# blueprinter

Hand-drawn style embedded-visual renderer CLI.

blueprinter renders structured diagrams (Mermaid today via the external `mmdc`, draw.io direct input planned) as hand-drawn raster images. The primary outputs are **PNG** and **WebP**; SVG output is kept as a debug-only intermediate so you can inspect the styling pipeline before rasterization. The Markdown pipeline currently batch-renders embedded `mermaid` blocks and is planned to expand into a general embedded-visual compiler, starting with `latex-render` blocks for editorial cards, lists, and tables.

## Installation

```bash
cargo install blueprinter
```

Or download a prebuilt binary from GitHub Releases once `v0.1.0+` tags are published.

## Usage

```bash
# Render a Mermaid diagram to PNG (requires mmdc on PATH)
blueprinter render -i flowchart.mmd -o flowchart.png --theme manga --seed 42

# Batch-render every supported embedded visual block in a Markdown file
# (PNG is the default format)
blueprinter md -i README.md -o ./diagrams --theme manga --width 800

# Transform an existing SVG into a hand-drawn PNG
blueprinter transform -i input.svg -o output.png --theme blueprint --seed 42

# Export to PNG with an explicit scale factor
blueprinter transform -i input.svg -o output.png \
  --scale 2.0

# Export to PNG with explicit dimensions (maintains aspect ratio)
blueprinter transform -i input.svg -o output.png \
  --width 800

# Export to lossless WebP (smaller than PNG for diagram content)
blueprinter transform -i input.svg -o output.webp \
  --width 800

# Tune the hand-drawn intensity
blueprinter transform -i input.svg -o output.png \
  --seed 42 \
  --jitter-amplitude 3.5 \
  --jitter-frequency 7 \
  --jitter-stroke-width-var 0.4

# Supply a font directory so glyph paths use the face you want
blueprinter transform -i input.svg -o output.png \
  --seed 42 \
  --font-dir ./fonts

# Debug-only: dump the intermediate styled SVG before rasterization
blueprinter transform -i input.svg -o debug.svg --format svg --seed 42
```

> Note: `--format svg` (and writing to a `.svg` path) keeps working, but SVG
> output is treated as a debug aid — the styling pipeline is moving toward
> raster-only effects (watercolor bleed, text-to-path, etc.) that cannot be
> faithfully represented in SVG.

`--jitter-amplitude` controls how far coordinates can wobble, `--jitter-frequency`
controls how densely strokes are subdivided before wobble is applied, and
`--jitter-stroke-width-var` controls relative stroke-thickness variation. Omitting
them preserves today's defaults. Text in the input SVG is flattened to glyph
outline paths via `usvg` before transformation (#4), so the same coordinate
jitter that wobbles strokes and shapes also wobbles letter outlines — there is
no longer a separate text-grunge filter or per-character rotation. To pin a
specific face, pass `--font-dir <path>` (see the **Fonts** section below).

## Themes

- `blueprint` — accepted; full technical drawing styling with Gaussian blur
- `sumi` — Japanese ink wash painting with grayscale ink bleed effect (now implemented)
- `watercolor` — soft pigment bleeding and color mixing (now implemented)
- `chalk` — white chalk on a blackboard, with dust/breakup filter (now implemented)
- `marker` — bold neon marker strokes on a dark sketchbook (now implemented)
- `manga` — black ink lines on white paper with screentone fills (now implemented)

### Sumi Theme

The sumi (墨) theme mimics traditional Japanese ink painting with grayscale strokes and a soft bleed effect.

```bash
blueprinter transform -i input.svg -o output.png --theme sumi --seed 42
```

**Features:**
- Grayscale color palette (black to light gray)
- Raster bleed via the aquarelle compositor (`radius=3.0`, `intensity=0.3`, `halo=0.0`) applied after rasterization for a soft ink-wash spread
- Semi-transparent stroke opacity (0.6–1.0) for ink wash effect

### Watercolor Theme

The watercolor theme simulates soft pigment mixing and color bleeding with pastel colors.

```bash
blueprinter transform -i input.svg -o output.png --theme watercolor --seed 42
```

**Features:**
- Pastel color palette (#FFB3BA, #FFDFBA, #FFFFBA, #BAFFC9, #BAE1FF, #E0BBE4, #FFC7F5)
- Raster bleed via the aquarelle compositor (`radius=6.0`, `intensity=0.5`, `halo=0.4`) for diffuse pigment spread and soft halo
- Semi-transparent fills (0.5–0.9) for transparency effect

### Manga Theme

The manga theme renders crisp black ink lines on white paper, with closed shapes filled by `<pattern>`-based screentones (sparse dots, dense dots, or diagonal lines) sampled per shape.

```bash
blueprinter transform -i input.svg -o output.png --theme manga --seed 42
```

**Features:**
- White paper background (#ffffff)
- Pure black strokes (#000000) — no per-shape color randomization
- Three SVG `<pattern>` screentones injected into `<defs>` and referenced via `fill="url(#manga-...)"`
- Closed shapes get a screentone picked from `manga-dots-light`, `manga-dots-dark`, or `manga-lines-diag`

### Marker Theme

The marker theme renders strokes in saturated neon highlighter colors on a dark navy background, with a soft halo behind each shape.

```bash
blueprinter transform -i input.svg -o output.png --theme marker --seed 42
```

**Features:**
- Dark navy background (#1a1a2e) inserted automatically
- Six-color neon palette (hot pink, cyan, lime, orange, yellow, magenta) — sampled per shape
- Closed shapes get a translucent palette-colored fill (~20% alpha) for highlighter-style overlap
- `marker-glow` filter (Gaussian blur halo merged behind source) for a slight bleed
- Stroke opacity 0.85–1.0 — marker ink is consistent

### Chalk Theme

The chalk theme renders strokes as white (and occasional pale color) chalk on a slate-green chalkboard, with a dust filter that breaks each stroke up.

```bash
blueprinter transform -i input.svg -o output.png --theme chalk --seed 42
```

**Features:**
- Chalkboard background (#1f2a25) inserted automatically
- White-dominated palette with pale yellow / pink / blue / green chalk accents
- `chalk-dust` filter combining `feTurbulence` + `feDisplacementMap` + light Gaussian blur for a powdery, broken-line look
- Semi-transparent stroke opacity (0.7–0.95) per stroke for uneven pressure
- Closed shapes get `fill="none"` — chalk is treated as a line medium

## Current Status

### Implemented
- `transform` command: SVG → hand-drawn raster (PNG/WebP) — SVG-output mode is preserved as a debug aid
- PNG output (default): `--scale`, `--width`, `--height` options
- WebP output (lossless): same flags as PNG, via `--format webp` or a `.webp` output path
- `render` command: Mermaid (`.mmd` / `.mermaid`) → mmdc → blueprinter pipeline. Supports the same theme / output-format / jitter / font flags as `transform`. Requires [mermaid-cli](https://github.com/mermaid-js/mermaid-cli): `npm install -g @mermaid-js/mermaid-cli`
- `md` command: currently extracts every ` ```mermaid ` block from a Markdown file and writes them to an output directory as `<stem>-<n>.<ext>`. This command is intended to grow into the general pipeline for embedded visual blocks such as future `latex-render`.
- Blueprint theme: complete with stroke/fill styling and background
- **Sumi theme**: ink wash effect with grayscale colors and a raster bleed pass (aquarelle compositor)
- **Watercolor theme**: pastel color palette with a diffuse raster bleed pass (aquarelle compositor)
- **Chalk theme**: white chalk strokes on a chalkboard with dust/breakup filter
- **Marker theme**: bold neon highlighter strokes on a dark navy background with halo
- **Manga theme**: black ink lines on white paper with three screentone patterns for fills
- Jitter controls: `--jitter-amplitude`, `--jitter-frequency`, `--jitter-stroke-width-var`
- Text → glyph path flattening via `usvg` (#4): letter outlines wobble through the same path jitter as strokes and shapes
- Reproducible output: `--seed` for deterministic jitter
- Shape jittering: `rect`, `line`, `polyline`, `path`, `circle`, `ellipse`, `polygon` (latter three via Bezier approximation)

### Planned
- `md` pipeline expansion: support ` ```latex-render ` blocks and replace them with generated SVG/PNG cards in a companion Markdown output
- More screentone variants and (eventually) speed-line layout for manga
- Full theme styling for blueprint (currently basic)
- draw.io input path for `render`
- `convert` command (general format conversion)

## Release

`blueprinter` is prepared as a Rust CLI crate with:
- `cargo install blueprinter`
- GitHub Actions CI on pushes and pull requests
- a tag-driven GitHub Releases workflow for Linux, macOS, and Windows artifacts

The first public crate/release target is `v0.1.0`.

### Font Resolution

Text in the input SVG is flattened to glyph outline paths via `usvg` (#4) before any styling runs, so fonts are resolved up-front. blueprinter loads the host's system fonts so any `font-family` referenced in the input SVG (e.g. `Arial`, `Helvetica`) resolves. If the requested face is not installed, `usvg` falls back to whatever the fontdb can match.

Note: with #4 glyph-path flattening, the `--font-family` CLI flag no longer applies at the SVG attribute level. Text rendering uses the fonts loaded into the rasterizer's font database (via `--font-dir`, or the system font database). `--font-family` is retained for API stability and may be re-enabled by a future text styling pass.

For cross-platform reproducibility, pass `--font-dir <path>` to load every `.ttf` / `.otf` in a directory into the fontdb. This is the recommended way to pin specific fonts:

```bash
blueprinter transform -i input.svg -o output.png \
  --font-dir ./fonts
```

The repo-level `fonts/` directory is reserved for future built-in bundling; see `fonts/README.md` for license-compatible OFL fonts that fit each theme.

### Known Limitations
- The debug-only SVG output does not preserve XML declarations, comments, processing instructions, doctypes, or CDATA boundaries
- Symbols and definitions under `defs`/`symbol`/`marker` are preserved without jitter

## License

MIT
