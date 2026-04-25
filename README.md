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

- `blueprint` — accepted today; full technical drawing styling is planned
- `sumi` — planned Japanese ink wash painting
- `chalk` — planned white chalk on a blackboard
- `marker` — planned bold neon marker strokes
- `watercolor` — planned soft pigment bleeding
- `manga` — planned screentone patterns and speed lines

## Current Status

### Implemented
- `transform` command: SVG → hand-drawn SVG transformation
- PNG output: `--format png`, with `--scale`, `--width`, `--height` options
- Blueprint theme: complete with stroke/fill styling and background
- Jitter controls: `--jitter-amplitude`, `--jitter-frequency`, `--jitter-stroke-width-var`
- Text overrides: `--font-family` for font replacement
- Reproducible output: `--seed` for deterministic jitter
- Shape jittering: `rect`, `line`, `polyline`, `path`, `circle`, `ellipse`, `polygon` (latter three via Bezier approximation)

### Planned
- WebP output (currently PNG only)
- Additional themes: `sumi` (ink wash), `chalk`, `marker`, `watercolor`, `manga`
- Full theme styling for blueprint (currently basic)
- Text outline conversion for advanced effects
- `render` command (Mermaid/draw.io → SVG → hand-drawn SVG)
- `convert` command (general format conversion)

### Known Limitations
- XML declarations, comments, processing instructions, doctypes, and CDATA boundaries are not preserved
- Symbols and definitions under `defs`/`symbol`/`marker` are preserved without jitter

## License

MIT
