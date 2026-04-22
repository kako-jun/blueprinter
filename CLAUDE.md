# blueprinter - Hand-drawn Style Diagram Renderer

図の手書き風レンダラー CLI。Mermaid / draw.io SVG / 任意の SVG を手書き風の SVG/PNG/WebP に変換する。

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
- **SVG→SVG が核** — フォーマット変換（PNG/WebP）は resvg によるレンダリングの後工程
- **毎回違う出力** — 手書き風のランダム性を持たせる。`--seed` で再現可能
- **エディタは作らない** — CLI で変換するだけ。入力は既存のエディタ・ツールで作成する
- **mmdc 連携** — Mermaid 入力の場合は mermaid-cli（mmdc）を外部コマンドとして呼び出す
