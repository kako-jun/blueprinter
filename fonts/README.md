# fonts/

This directory is reserved for fonts that future versions of blueprinter may
bundle into the binary so output is reproducible without relying on the host's
system fonts.

The directory is currently empty. The CLI already supports `--font-dir <path>`,
which loads every `.ttf` / `.otf` from a user-supplied directory into the
rasterizer's font database. That covers the same use case without touching
the binary size, and is the recommended approach today.

If you do want to bundle fonts:

1. Pick fonts under a redistributable license. Recommended:
   - **SIL Open Font License (OFL)** — most "handwriting" / "comic" fonts on
     Google Fonts ship under OFL.
   - **Apache 2.0** — Roboto, Noto, etc.
   - **MIT** — rare for fonts but acceptable.

2. Recommended faces by theme (each ~30–200 KB):
   - General handwriting: **Caveat**, **Patrick Hand**, **Architects Daughter**
   - Technical hand-drawn: **Excalifont** (the font Excalidraw ships), **Virgil**
   - Bold marker: **Permanent Marker**
   - Brush ink (for `sumi`): **Zen Brush 2** style fonts are usually proprietary;
     consider **Yuji Mai** or **Hina Mincho** under OFL
   - Manga / speech bubbles: **Bangers**, **Comic Neue**, **Mochiy Pop One**
     (CJK)

3. Drop the `.ttf` / `.otf` files in this directory and add a license-tracking
   note (e.g. `Caveat-Regular.OFL.txt`) verbatim from the upstream.

4. The bundling code itself (compile-time `include_bytes!` + fontdb
   registration) is not yet wired up — see `src/render.rs` / `src/svg/export.rs`
   for the registration call sites we'd hook into.
