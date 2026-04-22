# blueprinter ロードマップ

最終更新: 2026-04-22

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

## 残タスク

### Phase 1: SVGパース基盤（#2）

- [ ] SVG読み込みモジュール（roxmltree or xml-rs選定）
- [ ] SVG要素の走査（path, rect, circle, line, text 等）
- [ ] 基本フィルター: stroke の手書き風変換（直線→ベジェ曲線）
- [ ] `--seed` 対応（rand クレート導入）

### Phase 2: テーマシステム（#3）

- [ ] Theme trait / enum 設計
- [ ] blueprint テーマ（デフォルト）実装
- [ ] カラーパレット定義
- [ ] fill / stroke のテーマ適用

### Phase 3: 入出力拡張（#4）

- [ ] Mermaid 入力対応（mmdc 外部コマンド呼び出し）
- [ ] PNG 出力（resvg 導入）
- [ ] WebP 出力

### Phase 4: 追加テーマ（#5）

- [ ] sumi テーマ
- [ ] chalk テーマ
- [ ] marker テーマ
- [ ] watercolor テーマ
- [ ] manga テーマ

### Phase 5: 公開準備

- [ ] crates.io 公開
- [ ] Homebrew _formula 検討
- [ ] README にインストール方法・使用例を追加
