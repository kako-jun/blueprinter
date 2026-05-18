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
///
/// 入力 SVG に `<text` 要素が含まれない場合は **入力をそのまま返す**。
/// usvg は SVG をパースする際に rect/circle/ellipse/polygon を path に
/// 正規化したり、色名を hex に展開したりするので、text が無いのに usvg を
/// 通してしまうと後段の shape-tag ベースの theme 判定（is_closed_shape 等）が
/// 崩れてしまう。text が無いケースを早期 return することで、既存の SVG
/// パイプラインへの影響を text を含むドキュメントだけに局所化する。
pub fn flatten_text_to_paths(svg: &str, font_dir: Option<&Path>) -> Result<String, String> {
    // 安価な substring 判定で text が無いことが分かれば usvg をスキップする。
    // 厳密な XML パースは後段の roxmltree-based transform_svg に委ねる。
    if !contains_text_element(svg) {
        return Ok(svg.to_string());
    }

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

/// `<text` で始まる開始タグが含まれるかを判定する。
///
/// XML コメント・CDATA 内の `<text` も誤検出しうるが、その場合は usvg を
/// 通すだけで挙動が壊れることはない（usvg 側で何もしないか正規化されるだけ）。
fn contains_text_element(svg: &str) -> bool {
    // ` <text ` または `<text>` または `<text\n` のように、tag を続ける文字
    // （空白・改行・>・/）が直後に来る場合のみ「text 要素」と判定する。
    // 単に文字列 "text" が含まれているだけのケース（attribute 名等）を除外。
    let bytes = svg.as_bytes();
    let needle = b"<text";
    let mut i = 0;
    while i + needle.len() < bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            let next = bytes[i + needle.len()];
            if next == b' '
                || next == b'>'
                || next == b'\t'
                || next == b'\n'
                || next == b'\r'
                || next == b'/'
            {
                return true;
            }
        }
        i += 1;
    }
    false
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
        assert!(
            !out.contains("<text "),
            "text element should be removed: {out}"
        );
    }

    #[test]
    fn flatten_text_to_paths_returns_err_on_invalid_svg_with_text() {
        // text を含むため early-return がスキップされ、usvg のパース失敗が
        // 表面化する。text を含まない不正 SVG は (text 用処理が走らないので)
        // そのまま pass-through される — それは後段 transform_svg の roxmltree が
        // 拾うので本ヘルパの責務外。
        let result = flatten_text_to_paths("<text>broken without root", None);
        assert!(result.is_err(), "got: {result:?}");
    }

    #[test]
    fn flatten_text_to_paths_passes_text_free_svg_through_unchanged() {
        // text を含まない SVG は usvg を通さず原文をそのまま返す。usvg は
        // rect/circle 等を path に正規化したり色名を hex に展開したりするので、
        // text が無いのに正規化が走ると後段の theme 判定が壊れる。
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><circle cx="50" cy="50" r="20" fill="blue"/></svg>"#;
        let out = flatten_text_to_paths(svg, None).expect("flatten");
        assert_eq!(out, svg);
    }

    #[test]
    fn contains_text_element_matches_real_text_tags() {
        assert!(contains_text_element("<text>foo</text>"));
        assert!(contains_text_element("<text x=\"0\">foo</text>"));
        assert!(contains_text_element("<text\nx=\"0\">foo</text>"));
        assert!(contains_text_element("<text/>"));
    }

    #[test]
    fn contains_text_element_ignores_attribute_substrings() {
        // "text" がアトリビュート名や値に含まれていても、開始タグでなければ false
        assert!(!contains_text_element(r#"<rect id="textbox"/>"#));
        assert!(!contains_text_element(r#"<g class="text-layer"/>"#));
        assert!(!contains_text_element("<svg></svg>"));
    }

    /// 観点: 境界値・空文字。
    /// 空の text 要素や空白だけの text 要素を含む SVG でも panic せず Ok を返す。
    #[test]
    fn flatten_text_to_paths_does_not_panic_on_empty_or_whitespace_text() {
        let svgs = [
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"><text x="10" y="20"></text></svg>"##,
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"><text x="10" y="20">   </text></svg>"##,
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"><text/></svg>"##,
        ];
        for svg in svgs {
            let result = flatten_text_to_paths(svg, None);
            assert!(
                result.is_ok(),
                "empty/whitespace text panicked or errored for {svg}: {result:?}"
            );
        }
    }

    /// 観点: API 失敗 / ファイル系。
    /// 存在しない font_dir を渡しても panic せず、Ok か Err のどちらかに収まる。
    #[test]
    fn flatten_text_to_paths_does_not_panic_with_nonexistent_font_dir() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"><text x="10" y="20" font-size="14">Hi</text></svg>"##;
        let result =
            flatten_text_to_paths(svg, Some(Path::new("/nonexistent-path-xyz-blueprinter")));
        // どちらでもよいが panic しないこと。
        let _ = result;
    }

    /// 観点: 境界値・事故パターン。
    /// `<text` 5 バイトちょうど、`<tex` などの切り詰め入力で out-of-bounds せず false を返す。
    #[test]
    fn contains_text_element_handles_truncated_input_at_text_tag_boundary() {
        // 末尾で続き文字 (空白・>・/ 等) が無いので false。
        assert!(!contains_text_element("<text"));
        // tag 名にも届かない切り詰め。
        assert!(!contains_text_element("<tex"));
        assert!(!contains_text_element("<t"));
        assert!(!contains_text_element("<"));
        assert!(!contains_text_element(""));
    }

    #[test]
    fn flatten_text_to_paths_preserves_non_text_elements_alongside_text() {
        // text と一緒に居る shape は usvg で正規化される（rect/circle が path に
        // 展開されるなど）。これは usvg の仕様で、glyph path 化と引き換えに
        // 受け入れる canonicalization。最低限、何らかの path data が
        // 出力に含まれていること（glyph or shape）だけを確認する。
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="80">
            <rect x="10" y="10" width="30" height="20" fill="blue"/>
            <text x="50" y="50" font-size="14">Hi</text>
        </svg>"##;

        let out = flatten_text_to_paths(svg, None).expect("flatten");
        assert!(
            out.contains("<path"),
            "should contain at least one path: {out}"
        );
        assert!(!out.contains("<text "), "text should be flattened: {out}");
    }
}
