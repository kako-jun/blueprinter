//! SVG text → glyph path 展開ヘルパ。
//!
//! blueprinter は最終的に PNG/WebP のラスター画像を出す。accessibility や
//! copy/paste は最初から成立していないので、`<text>` 要素を glyph outline まで
//! 展開してしまえば、既存の path jitter (`jitter.rs`) をそのまま当てて文字を
//! 手書き感のある揺れた path として描けるようになる。
//!
//! 使い方: `transform_svg` の前段で本ヘルパを呼び、戻り値の SVG 文字列を後段の
//! roxmltree-based パイプラインに流す。後段は text/tspan がもう来ない前提で動く。
//!
//! フォントが必要な場合（日本語など system fontdb で見つからないとき）は、
//! `--font-dir` 経由でフォントを供給する。フォントが見つからない場合の挙動は
//! usvg 側に委ねており、警告で空 path に展開されるかエラーが返るかは usvg の
//! 仕様に従う。

use std::path::Path;

use resvg::usvg;

/// `<text>` を glyph path に展開した SVG 文字列を返す。
///
/// `font_dir` には追加のフォントディレクトリを指定できる。`None` の場合は
/// system fontdb のみを利用する。
pub fn flatten_text_to_paths(svg: &str, font_dir: Option<&Path>) -> Result<String, String> {
    let mut options = usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    if let Some(dir) = font_dir {
        options.fontdb_mut().load_fonts_dir(dir);
    }

    let tree = usvg::Tree::from_str(svg, &options)
        .map_err(|e| format!("Failed to parse SVG for text-to-path flattening: {e}"))?;

    // WriteOptions::default() は preserve_text = false なので、`<text>` は
    // glyph path に展開された状態でシリアライズされる。
    Ok(tree.to_string(&usvg::WriteOptions::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_text_to_paths_removes_text_element() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="80">
            <rect width="200" height="80" fill="#ffffff"/>
            <text x="20" y="50" font-size="24" fill="#000000">Hi</text>
        </svg>"##;

        let out = flatten_text_to_paths(svg, None).expect("flatten");
        // text element must not survive in the output (either expanded to paths
        // or, when no font matches, dropped silently by usvg).
        assert!(!out.contains("<text "), "text element should be removed: {out}");
    }

    #[test]
    fn flatten_text_to_paths_returns_err_on_invalid_svg() {
        let result = flatten_text_to_paths("not valid svg", None);
        assert!(result.is_err());
    }

    #[test]
    fn flatten_text_to_paths_preserves_non_text_elements() {
        // Shapes other than <text> must round-trip through usvg without being
        // lost — they're the canvas the glyph paths get composed onto.
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="30" height="20" fill="blue"/>
            <circle cx="50" cy="50" r="20" fill="red"/>
        </svg>"##;

        let out = flatten_text_to_paths(svg, None).expect("flatten");
        // usvg may normalize rect/circle into <path>, but the visual primitives
        // must still produce some path data in the output.
        assert!(out.contains("<path") || out.contains("<rect") || out.contains("<circle"));
    }
}
