# blueprinter

Hand-drawn style diagram renderer CLI.

Turn SVG into sketchy SVG today. Rasterize to PNG. Mermaid and draw.io direct input are planned.

## Installation

```bash
cargo install blueprinter
```

## Usage

```bash
# Transform an existing SVG with the default blueprint theme
blueprinter transform -i input.svg -o output.svg --theme blueprint --seed 42

# Tune the hand-drawn intensity
blueprinter transform -i input.svg -o output.svg \
  --seed 42 \
  --jitter-amplitude 3.5 \
  --jitter-frequency 7 \
  --jitter-stroke-width-var 0.4

# Override text font-family while keeping layout intact
blueprinter transform -i input.svg -o output.svg \
  --seed 42 \
  --font-family "Virgil"

# Export to PNG (2x scale)
blueprinter transform -i input.svg -o output.png \
  --format png \
  --scale 2.0

# Export to PNG with explicit dimensions (maintains aspect ratio)
blueprinter transform -i input.svg -o output.png \
  --format png \
  --width 800
```

`--jitter-amplitude` controls how far coordinates can wobble, `--jitter-frequency`
controls how densely strokes are subdivided before wobble is applied, and
`--jitter-stroke-width-var` controls relative stroke-thickness variation. Omitting
them preserves today's defaults. `--font-family` overrides SVG text `font-family`;
if omitted, existing text fonts and stylesheet-driven fonts are left as authored.

## Themes

- `blueprint` — accepted; full technical drawing styling with Gaussian blur
- `sumi` — Japanese ink wash painting with grayscale ink bleed effect (now implemented)
- `watercolor` — soft pigment bleeding and color mixing (now implemented)
- `chalk` — white chalk on a blackboard, with dust/breakup filter (now implemented)
- `marker` — bold neon marker strokes on a dark sketchbook (now implemented)
- `manga` — planned screentone patterns and speed lines

### Sumi Theme

The sumi (墨) theme mimics traditional Japanese ink painting with grayscale strokes and a soft bleed effect.

```bash
blueprinter transform -i input.svg -o output.svg --theme sumi --seed 42
```

**Features:**
- Grayscale color palette (black to light gray)
- Gaussian blur filter with 2–4px standard deviation
- Semi-transparent stroke opacity (0.6–1.0) for ink wash effect

### Watercolor Theme

The watercolor theme simulates soft pigment mixing and color bleeding with pastel colors.

```bash
blueprinter transform -i input.svg -o output.svg --theme watercolor --seed 42
```

**Features:**
- Pastel color palette (#FFB3BA, #FFDFBA, #FFFFBA, #BAFFC9, #BAE1FF, #E0BBE4, #FFC7F5)
- Gaussian blur filter with 4–8px standard deviation for diffuse bleed
- Color matrix saturation adjustment (90%) for soft, washed appearance
- Semi-transparent fills (0.5–0.9) for transparency effect

### Marker Theme

The marker theme renders strokes in saturated neon highlighter colors on a dark navy background, with a soft halo behind each shape.

```bash
blueprinter transform -i input.svg -o output.svg --theme marker --seed 42
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
blueprinter transform -i input.svg -o output.svg --theme chalk --seed 42
```

**Features:**
- Chalkboard background (#1f2a25) inserted automatically
- White-dominated palette with pale yellow / pink / blue / green chalk accents
- `chalk-dust` filter combining `feTurbulence` + `feDisplacementMap` + light Gaussian blur for a powdery, broken-line look
- Semi-transparent stroke opacity (0.7–0.95) per stroke for uneven pressure
- Closed shapes get `fill="none"` — chalk is treated as a line medium

## Current Status

### Implemented
- `transform` command: SVG → hand-drawn SVG transformation
- PNG output: `--format png`, with `--scale`, `--width`, `--height` options
- Blueprint theme: complete with stroke/fill styling and background
- **Sumi theme**: ink wash effect with grayscale colors and blur filters
- **Watercolor theme**: pastel color palette with diffuse bleed effect
- **Chalk theme**: white chalk strokes on a chalkboard with dust/breakup filter
- **Marker theme**: bold neon highlighter strokes on a dark navy background with halo
- Jitter controls: `--jitter-amplitude`, `--jitter-frequency`, `--jitter-stroke-width-var`
- Text overrides: `--font-family` for font replacement
- Reproducible output: `--seed` for deterministic jitter
- Shape jittering: `rect`, `line`, `polyline`, `path`, `circle`, `ellipse`, `polygon` (latter three via Bezier approximation)

### Planned
- WebP output (currently PNG only)
- Additional themes: `manga`
- Full theme styling for blueprint (currently basic)
- Text outline conversion for advanced effects
- `render` command (Mermaid/draw.io → SVG → hand-drawn SVG)
- `convert` command (general format conversion)

### Known Limitations
- XML declarations, comments, processing instructions, doctypes, and CDATA boundaries are not preserved
- Symbols and definitions under `defs`/`symbol`/`marker` are preserved without jitter

## License

MIT
