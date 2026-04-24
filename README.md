# blueprinter

Hand-drawn style diagram renderer CLI.

Turn SVG into sketchy SVG today. Mermaid, draw.io direct input, and raster export are planned.

## Installation

```bash
cargo install --git https://github.com/kako-jun/blueprinter blueprinter
```

`blueprinter` is not published to crates.io yet.

## Usage

```bash
# Transform an existing SVG with the default blueprint theme
blueprinter transform -i input.svg -o output.svg --theme blueprint --seed 42
```

## Mermaid PoC

If you want to evaluate whether blueprinter is visually compelling on real
diagram shapes today, run the Mermaid proof-of-concept workflow:

```bash
scripts/mermaid-poc.sh
```

This renders a small Mermaid fixture set to baseline SVG, then runs
`blueprinter transform` so you can compare baseline vs stylized output side by
side. See [docs/poc.md](docs/poc.md) for details.

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
- XML declarations, comments, processing instructions, doctypes, and CDATA boundaries are not preserved yet
- symbols and definitions under `defs`/`symbol`/`marker` are preserved without jitter, including shapes later referenced by `use`
- `render` and `convert` are not implemented yet

## License

MIT
