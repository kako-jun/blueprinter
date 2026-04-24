# blueprinter Mermaid PoC

This document exists to answer one question before deeper feature work:

**Can blueprinter make Mermaid-generated diagrams look attractive enough to be worth pursuing?**

The PoC deliberately avoids implementing full Mermaid direct-input support.
Instead, it gives us a repeatable visual workflow for producing baseline SVGs
with Mermaid CLI and then running `blueprinter transform` on top.

`mmdc` is therefore a **workflow dependency for this PoC today**, even though
Mermaid direct-input support inside the CLI remains a planned feature.

## Included Fixtures

- `tests/fixtures/mermaid/flowchart.mmd`
- `tests/fixtures/mermaid/sequence.mmd`
- `tests/fixtures/mermaid/er-diagram.mmd`

These three fixture types were chosen because they stress different failure
modes:

- **Flowchart**: box + arrow charm. Good for checking whether wobble adds warmth.
- **Sequence diagram**: many parallel lines. Good for checking whether jitter turns into clutter.
- **ER diagram**: node/edge density + labels. Good for checking structural readability.

## Run the PoC

Prerequisites:

- `mmdc` must be available on `PATH`
- Rust toolchain installed

Run:

```bash
scripts/mermaid-poc.sh
```

Outputs are written to:

```text
target/poc/
├── baseline/
│   ├── er-diagram.svg
│   ├── flowchart.svg
│   └── sequence.svg
└── blueprinter/
    ├── er-diagram.svg
    ├── flowchart.svg
    └── sequence.svg
```

You can override the output directory:

```bash
scripts/mermaid-poc.sh /tmp/blueprinter-poc
```

You can also override the seed:

```bash
BLUEPRINTER_POC_SEED=7 scripts/mermaid-poc.sh
```

## What to Look For

When comparing baseline vs transformed SVG, evaluate these points:

1. **Charm**: does the transformed version feel more human or handcrafted?
2. **Legibility**: are labels, arrows, and node boundaries still easy to read?
3. **Density tolerance**: which diagram types get better, and which get muddy?
4. **Theme confidence**: is the current blueprint treatment interesting enough to justify more theme investment?

## Expected Outcome

The goal is not for every diagram type to look perfect.
The goal is to identify:

- which Mermaid diagrams already look promising with current jitter
- where blueprint styling is too weak
- whether theme work should proceed before Mermaid direct input

If the transformed outputs do not look compelling, we should improve visual
treatment before spending much more effort on additional format support.
