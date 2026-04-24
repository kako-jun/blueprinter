# blueprinter

Hand-drawn style diagram renderer CLI.

Turn SVG into sketchy SVG today. Mermaid, draw.io direct input, and raster export are planned.

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

- `transform` works for SVG input and writes SVG output
- only the `blueprint` theme name is accepted; full theme styling is still planned
- `--seed` is supported for reproducible SVG jitter on the same SVG structure; changing earlier jittered elements can change later seeded jitter
- `transform` exposes `--jitter-amplitude`, `--jitter-frequency`, and `--jitter-stroke-width-var` for line-style tuning
- `transform` can override text with `--font-family`, and otherwise keeps the original SVG font choice
- `text` and `tspan` currently preserve their original `x`/`y`/`font-size` layout and only get subtle seeded `rotation` and `opacity` jitter; outline conversion is still planned
- XML declarations, comments, processing instructions, doctypes, and CDATA boundaries are not preserved yet
- symbols and definitions under `defs`/`symbol`/`marker` are preserved without jitter, including shapes later referenced by `use`
- `render` and `convert` are not implemented yet

## License

MIT
