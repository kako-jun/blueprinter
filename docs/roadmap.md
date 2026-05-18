# blueprinter ロードマップ

最終更新: 2026-05-19

## 完了済み

### Phase 0: プロジェクト初期化（#1）

- [x] crates.io で "blueprinter" クレート名の空き確認
- [x] `cargo init` でプロジェクト作成
- [x] Cargo.toml の基本設定（name, version, description, license, repository 等）
- [x] clap 導入と CLI 雛形（render, transform, convert サブコマンド）
- [x] `.gitignore` 設定
- [x] `cargo build` が通る状態まで
- [x] `--help` で使い方が表示される
- [x] CLAUDE.md 作成
- [x] docs/overview.md 作成
- [x] docs/roadmap.md 作成
- [x] README.md 作成
- [x] LICENSE（MIT）作成
- [x] .github/workflows/ci.yml 作成

## 完了済み（追加）

### Phase 1: SVGパース基盤（#2）

- [x] SVG読み込みモジュール（roxmltree 選定・導入）
- [x] SVG要素の走査（rect, line, polyline, path, circle, ellipse, polygon, text, group）
- [x] Primitive enum による内部表現定義
- [x] グループ `<g>` の階層構造保持

### Phase 2: 揺らし変換（#3）

- [x] JitterConfig 構造体（amplitude, frequency, stroke_width_var）
- [x] rect → 揺れた path 変換
- [x] line / polyline → 揺れた path 変換
- [x] path → 制御点にノイズを重畳（絶対・相対コマンド両対応）
- [x] ノイズ生成関数（rand クレート）
- [x] 線の太さ揺れ
- [x] transform CLI から SVG 入力 → SVG 出力まで接続
- [x] `--seed` による transform 出力の再現性

## 残タスク

### Phase 5.5: Markdown 埋め込みビジュアル拡張（new）

- [ ] `md` サブコマンドを「`mermaid` 専用バッチ」から「埋め込みビジュアル用パイプライン」へ拡張
- [ ] ` ```latex-render ` ブロック検出を追加
- [ ] `latex-render` ブロックを SVG/PNG 化する内部レンダラを実装
- [ ] 生成物を `<stem>.assets/` に連番出力
- [ ] 置換後 Markdown を `<stem>.generated.md` に出力
- [ ] 将来 `--in-place` オプションを検討（初期実装では非破壊優先）
- [ ] Blueprinter 向けの薄い TeX/DSL マクロ層を設計（見出し、箇条書き、表、引用、2カラム）
- [ ] Mermaid と `latex-render` を同一テーマ・同一出力フラグで扱えるよう CLI 仕様を整理

### Phase 3: テーマシステム（#6〜#8）

- [x] Theme trait / enum 設計
- [x] blueprint テーマ（デフォルト）実装
- [x] marker テーマ実装（暗いネイビー背景・蛍光6色パレット・marker-glow filter・低 alpha 塗り）
- [x] chalk テーマ実装（黒板緑 #1f2a25 背景・白主体パレット・chalk-dust filter）
- [x] カラーパレット定義（blueprint: 背景 #1a3a5c、stroke #e8e8e8、fill none）
- [x] fill / stroke のテーマ適用

### Phase 4: テキスト・フォント（#4, #12）

- [x] text → glyph path 展開（#4）— usvg の `Tree::to_string(&WriteOptions::default())`（preserve_text = false）で `<text>` を glyph outline path に展開し、既存 `jitter.rs` の path jitter をそのまま字形に当てる構成。旧 text-grunge filter / tspan 分割 / 文字単位の rotation・opacity ジッターは廃止
- [~] フォント選定と同梱（OFLライセンス確認）— `--font-dir <path>` で任意ディレクトリのフォントを fontdb に追加するインフラは実装済み (`fonts/README.md` に推奨フォントを記載)。binary 同梱 (`include_bytes!` + 既定参照) は未着手 — 実フォントファイルを repo にコミットする工程が必要

### Phase 5: 入出力拡張（#9〜#11）

- [x] Mermaid 入力対応（mmdc 外部コマンド呼び出し）— `render` サブコマンドが mmdc を呼び出して Mermaid → SVG → transform → 任意フォーマットでパイプライン化
- [x] md 一括変換モード — `md` サブコマンドが `.md` から ` ```mermaid ` ブロックを抽出 (line-by-line state machine、依存追加なし) し、`<out_dir>/<stem>-<n>.<ext>` で連番出力
- [x] PNG 出力（resvg 導入、#11 完了）— `--format png`, `--scale`, `--width`, `--height` 対応
- [x] WebP 出力（lossless）— `webp` クレート導入、PNG と同じフラグで動作。Diagram 用途では PNG より大幅に小さくなる
- [x] 出力デフォルトをラスター主軸へ切替（#31）— `transform` / `render` の拡張子推定既定を `png` に、`md` サブコマンドの `--format` 既定も `png` に変更。SVG 出力は `--format svg` または `.svg` 拡張子で引き続き利用可能だが debug-only 扱い

### Phase 6: 追加テーマ・図形（#13, #14）

- [x] sumi / watercolor テーマ（にじみエンジン）— #13 完了
- [x] circle / ellipse / polygon の揺らし変換
- [x] manga テーマ（黒インク + 白背景 + 3 種スクリーントーン pattern）
- [x] [#25] sumi / watercolor の bleed を SVG filter から aquarelle ラスター pass に置換（sumi: radius 3.0 / intensity 0.3 / halo 0.0、watercolor: radius 6.0 / intensity 0.5 / halo 0.4）— 完了

### Phase 7: 公開準備（#15）

- [x] GitHub Releases workflow（tag `v*` で Linux/macOS/Windows artifact を生成）
- [x] CHANGELOG.md 作成
- [x] `v0.2.0` tag + GitHub Release + `cargo publish`（出力ラスター主軸・aquarelle bleed・glyph path text 化を含む）
- [ ] Homebrew formula 検討
- [ ] 宣伝記事作成
- [ ] `freeza/output/blueprinter-*` の Mermaid fixture 比較画像リフレッシュ（#25 / #4 でラスター出力が変わったため）

## Known Limitations / 暫定実装

セッション間で忘れず把握しておくべき制限事項。解消・実装決定時に削除またはアーカイブする。

### 機能の暫定状態

- **`--font-family` フラグが no-op (#4 glyph path 化以降)**
  - API は破壊変更回避のため残置
  - glyph path 化後は font 属性が SVG に存在しないため当たらない
  - 再有効化検討: #35

- **usvg canonicalize の副作用**
  - `<text>` を含む SVG では rect / circle / ellipse / polygon が path に化け、色名が hex 化される（`red` → `#ff0000`）
  - `is_closed_shape("path")` で fill 処理が走らない問題は `path_is_closed(d)` の Z/z 検出で補強済
  - 形状固有情報（`rx`, `ry`, polygon の頂点リスト等）は実質損失するが、視覚出力には影響なし

- **`contains_text_element` の検出が素朴**
  - コメント内 / CDATA 内 / 属性値内の `<text>` を誤検出しうる
  - 誤検出時に usvg を経由するだけで壊れないが、不要な canonicalize が走る

### 残置 SVG filter

- **`marker-glow` (marker テーマ)** — per-shape Gaussian blur halo
- **`chalk-dust` (chalk テーマ)** — feTurbulence + feDisplacementMap による粉砕風
- aquarelle の Pixmap 全体 pass とは性質が異なる（per-shape 効果）
- aquarelle 化検討: #36
