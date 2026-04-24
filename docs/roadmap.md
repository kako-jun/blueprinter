# blueprinter ロードマップ

最終更新: 2026-04-24

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

### Phase 2.5: Mermaid見た目PoC（#20）

- [x] Mermaid fixture 3本（flowchart / sequence / ER）追加
- [x] `mmdc` → baseline SVG → `blueprinter transform` の PoC スクリプト追加
- [x] baseline / transformed の見比べ手順を docs 化
- [x] fixtures / script / docs の整合を守る軽量テスト追加

## 残タスク

### Phase 3: テーマシステム（#6〜#8）

- [ ] Theme trait / enum 設計
- [ ] blueprint テーマ（デフォルト）実装
- [ ] marker テーマ実装
- [ ] chalk テーマ実装
- [ ] カラーパレット定義
- [ ] fill / stroke のテーマ適用

### Phase 4: テキスト・フォント（#4, #12）

- [ ] text のフォント差し替え + ジッター
- [ ] フォント選定と同梱（OFLライセンス確認）

### Phase 5: 入出力拡張（#9〜#11）

- [ ] Mermaid 入力対応（mmdc 外部コマンド呼び出し）
- [ ] md 一括変換モード
- [ ] PNG 出力（resvg 導入）
- [ ] WebP 出力

### Phase 6: 追加テーマ・図形（#13, #14）

- [ ] sumi / watercolor テーマ（にじみエンジン）
- [ ] circle / ellipse / polygon の揺らし変換
- [ ] manga テーマ

### Phase 7: 公開準備（#15）

- [ ] crates.io 公開
- [ ] Homebrew formula 検討
- [ ] 宣伝記事作成
- [ ] README にインストール方法・使用例を追加
