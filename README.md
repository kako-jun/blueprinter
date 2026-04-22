# blueprinter

Hand-drawn style diagram renderer CLI.

Turn Mermaid, draw.io, and any SVG into sketchy SVG/PNG/WebP.

## Installation

```bash
cargo install blueprinter
```

## Usage

```bash
# Render a Mermaid or SVG file with the default blueprint theme
blueprinter render -i diagram.mmd -o output.svg

# Transform an existing SVG with a specific theme and seed
blueprinter transform -i input.svg -o output.svg --theme chalk --seed 42

# Convert SVG to PNG
blueprinter convert -i input.svg -o output.png
```

## Themes

- `blueprint` — Technical drawing on blue grid paper (default)
- `sumi` — Japanese ink wash painting
- `chalk` — White chalk on a blackboard
- `marker` — Bold neon marker strokes
- `watercolor` — Soft pigment bleeding
- `manga` — Screentone patterns and speed lines

## License

MIT
