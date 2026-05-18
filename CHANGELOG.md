# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [0.2.0] - 2026-05-19

### Added
- `aquarelle = "0.2"` 統合: sumi / watercolor の bleed をラスター pass で合成（#25）
  - sumi: `radius=3.0, intensity=0.3, halo=0.0`
  - watercolor: `radius=6.0, intensity=0.5, halo=0.4`
- text → glyph path flattening: `usvg::Tree::to_string(&WriteOptions::default())` で text を path 化して既存 jitter で輪郭を揺らす（#4）
- `path_is_closed(d)` helper: usvg canonicalize 後の path を fill 処理上 closed shape として認識（Mermaid 入力でも blueprint テーマの `fill="none"` 化を維持）
- `DEFAULT_SEED` 定数: seed 未指定時の fallback 値を 1 箇所に集約

### Changed
- 出力デフォルトを **ラスター主軸（PNG）** に変更（#31）。`--format` 拡張子推定の fallback も `svg` → `png`
- `transform_svg` のシグネチャに `font_dir: Option<&Path>` を追加（text→path flatten 用）
- `ThemeStyle::filter_id()` を `Option<&'static str>` に変更（bleed 系テーマで shape filter 属性を出さない）

### Removed
- SVG-filter bleed prototypes: `watercolor_filter_defs`, `sumi_filter_defs`, `subtle-bleed` filter, `src/svg/filter.rs`
- text 仮実装: `serialize_text_content`（tspan jitter）, `serialize_text_attrs`, `should_jitter_text`, `text-grunge` filter
- per-shape SVG filter attribute injection for sumi / watercolor

### Notes
- `--font-family` は #4 glyph path flatten 以降 no-op。API は破壊変更回避で残置。再有効化は #35 で追跡
- marker-glow / chalk-dust は per-shape SVG filter のまま残置。aquarelle 化は #36 で追跡

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
