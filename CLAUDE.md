# blueprinter - Hand-drawn Style Diagram Renderer

図の手書き風レンダラー CLI。最終出力はラスター画像（PNG / WebP）。入力は SVG / Mermaid (mmdc 経由) / Markdown 埋め込みブロックで、draw.io 直接入力は予定。SVG 出力モードはパイプライン途中の中間 SVG をダンプする debug-only 用途として残してある。

## ドキュメント

| ファイル | 内容 | 言語 |
|---|---|---|
| `README.md` | エンドユーザー向けの使い方 | 英語（マスター） |
| `docs/overview.md` | 設計思想、アーキテクチャ、使用場面 | 英語 |
| `docs/roadmap.md` | 完了済み・残タスク（内部運用メモ） | 日本語 |
| `CLAUDE.md` | AI向け内部ドキュメント | 日本語 |

### 言語ルール

- README は英語マスターのみ（日本語版は現時点では不要）
- docs/ は英語のみ
- docs/roadmap.md と CLAUDE.md は内部ドキュメントのため日本語のまま

## ソース構成（予定）

```
src/
├── main.rs           # CLI パース（clap）、サブコマンドディスパッチ
├── render.rs         # render サブコマンドの実装
├── transform.rs      # transform サブコマンドの実装
├── convert.rs        # convert サブコマンドの実装
├── theme/            # テーマ定義（blueprint, sumi, chalk, marker, watercolor, manga 等）
│   ├── mod.rs
│   └── ...
├── filter/           # SVG→SVG の見た目変換フィルター
│   ├── mod.rs
│   └── ...
└── util.rs           # 共通ユーティリティ
```

## 主要な設計判断

- **レイアウト計算はしない** — 入力 SVG の座標をそのまま使い、見た目（stroke, fill, filter）のみを変換する
- **ラスター主軸** — ユーザー向けの最終出力は PNG / WebP。内部的には styled SVG を中間表現として作り resvg でラスタライズする。中間 SVG は `--format svg` または `.svg` 出力パスでダンプ可能だが debug-only 扱いで、aquarelle (#25) や text path 化 (#4) のような SVG では表現しきれない加工を今後追加していく前提
- **毎回違う出力** — 手書き風のランダム性を持たせる。`--seed` で再現可能（transform 実装済み）
- **エディタは作らない** — CLI で変換するだけ。入力は既存のエディタ・ツールで作成する
- **mmdc 連携** — Mermaid 入力対応時は mermaid-cli（mmdc）を外部コマンドとして呼び出す予定
