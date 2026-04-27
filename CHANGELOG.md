# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [0.1.0] - 2026-04-27

### Added
- `transform` command for SVG -> hand-drawn SVG conversion
- `render` command for Mermaid -> `mmdc` -> blueprinter pipeline
- `md` command for batch extraction of embedded `mermaid` blocks from Markdown
- PNG export via `resvg`
- Lossless WebP export
- Themes: `blueprint`, `sumi`, `watercolor`, `chalk`, `marker`, `manga`
- Jitter controls: `--seed`, `--jitter-amplitude`, `--jitter-frequency`, `--jitter-stroke-width-var`
- Font controls: `--font-family`, `--font-dir`
- GitHub Actions CI and release workflow

### Notes
- `md` currently processes `mermaid` blocks only.
- Expansion to `latex-render` blocks is planned in Issue #30.
