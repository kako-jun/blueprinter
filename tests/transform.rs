use std::fs;

use blueprinter::jitter::JitterConfig;
use blueprinter::svg::transform::TransformError;
use blueprinter::svg::{theme_style, transform_svg, Theme, TransformOptions};

fn options(seed: u64) -> TransformOptions {
    TransformOptions {
        seed: Some(seed),
        font_family_override: None,
        theme: Default::default(),
    }
}

#[test]
fn transform_svg_preserves_root_and_writes_jittered_paths() {
    let svg = fs::read_to_string("tests/fixtures/simple.svg").unwrap();
    let transformed = transform_svg(&svg, &JitterConfig::default(), &options(42), None).unwrap();

    // #4 で text を含む SVG は usvg で path に正規化される。root の <svg> 開始タグと
    // jittered path data だけ確認する（usvg は width/height 属性順序を変えるため
    // starts_with は使わない）。
    assert!(transformed.contains("<svg"));
    assert!(transformed.contains(r#"xmlns="http://www.w3.org/2000/svg""#));
    assert!(transformed.contains("<path"));
    assert!(transformed.contains("d=\"M"));
    // text 要素は glyph path に展開されるので、もう <text> としては存在しない
    assert!(!transformed.contains("<text "));
}

#[test]
fn transform_svg_with_same_seed_is_reproducible() {
    let svg = fs::read_to_string("tests/fixtures/simple.svg").unwrap();
    let config = JitterConfig::default();
    let out1 = transform_svg(&svg, &config, &options(42), None).unwrap();
    let out2 = transform_svg(&svg, &config, &options(42), None).unwrap();
    assert_eq!(out1, out2);
}

#[test]
fn transform_svg_with_different_seed_changes_jitter() {
    let svg = fs::read_to_string("tests/fixtures/simple.svg").unwrap();
    let config = JitterConfig::default();
    let out1 = transform_svg(&svg, &config, &options(42), None).unwrap();
    let out2 = transform_svg(&svg, &config, &options(43), None).unwrap();
    assert_ne!(out1, out2);
}

#[test]
fn transform_svg_with_same_seed_and_custom_config_is_reproducible() {
    let svg = fs::read_to_string("tests/fixtures/simple.svg").unwrap();
    let config = JitterConfig {
        amplitude: 3.5,
        frequency: 7.0,
        stroke_width_var: 0.4,
    };
    let out1 = transform_svg(&svg, &config, &options(42), None).unwrap();
    let out2 = transform_svg(&svg, &config, &options(42), None).unwrap();
    assert_eq!(out1, out2);
}

#[test]
fn transform_svg_changes_when_jitter_config_changes() {
    let svg = fs::read_to_string("tests/fixtures/simple.svg").unwrap();
    let subtle = JitterConfig {
        amplitude: 0.5,
        frequency: 2.0,
        stroke_width_var: 0.05,
    };
    let rough = JitterConfig {
        amplitude: 4.0,
        frequency: 9.0,
        stroke_width_var: 0.6,
    };

    let subtle_out = transform_svg(&svg, &subtle, &options(42), None).unwrap();
    let rough_out = transform_svg(&svg, &rough, &options(42), None).unwrap();

    assert_ne!(subtle_out, rough_out);
}

#[test]
fn transform_svg_preserves_non_jittered_structure_and_extra_attrs() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">
  <defs><linearGradient id="g"><stop offset="0%" stop-color="#fff"/></linearGradient></defs>
  <g id="layer1" class="node" transform="translate(1 2)">
    <rect id="rounded" x="1" y="2" width="3" height="4" rx="1" style="opacity:0.5"/>
    <line id="edge" class="wire" x1="0" y1="0" x2="5" y2="5" transform="scale(2)" style="stroke:red"/>
  </g>
</svg>"##;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42), None).unwrap();

    // #4: text を含まないので usvg 経由の正規化はかからず、blueprinter の
    // roxmltree pipeline が原文の構造をほぼ温存する。text-grunge filter は廃止された
    // ので defs には何も追加されない（テーマ別 extra_defs のみが入る）。
    assert!(transformed.contains(
        r##"<linearGradient id="g"><stop offset="0%" stop-color="#fff" /></linearGradient>"##
    ));
    assert!(!transformed.contains("text-grunge"));
    assert!(transformed.contains(r##"<g id="layer1" class="node" transform="translate(1 2)""##));
    assert!(transformed.contains(
        r#"<rect id="rounded" x="1" y="2" width="3" height="4" rx="1" style="opacity:0.5" />"#
    ));
    assert!(transformed.contains(r#"<path d=""#));
    assert!(transformed.contains(r#"id="edge""#));
    assert!(transformed.contains(r#"class="wire""#));
    assert!(transformed.contains(r#"transform="scale(2)""#));
    assert!(transformed.contains(r#"style="stroke:red""#));
}

// #4 で <text>/<tspan> は usvg で glyph path に展開されるため、テキスト要素や
// tspan を直接 assert する以下の旧テストは削除した:
//   - transform_svg_escapes_decoded_text_and_attributes
//     (tspan が出ない、text content は glyph path に展開される)
//   - transform_svg_preserves_existing_font_family_when_override_is_absent
//   - transform_svg_keeps_text_layout_and_only_jitters_rotation_and_opacity
//   - transform_svg_appends_rotation_to_existing_text_transform
//   - transform_svg_preserves_text_position_and_font_size_exactly
//   - transform_svg_jitters_existing_text_opacity
//   - test_text_filter_only_text_grunge (text-grunge filter 廃止)

#[test]
fn transform_svg_handles_compact_path_data() {
    let svg =
        r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M0-1L2.5.5Z" stroke="black"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42), None).unwrap();

    assert!(transformed.contains("<path"));
    assert!(transformed.contains(" L "));
    assert!(transformed.contains("Z"));
}

#[test]
fn transform_svg_normalizes_relative_path_commands_to_absolute() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="m 10 10 l 20 0" stroke="black"/></svg>"#;
    let transformed = transform_svg(
        svg,
        &JitterConfig {
            amplitude: 0.0,
            frequency: 1.0,
            stroke_width_var: 0.0,
        },
        &options(42),
        None,
    )
    .unwrap();

    assert!(transformed.contains(r#"d="M 10.000 10.000 L 30.000 10.000""#));
}

#[test]
fn transform_svg_relative_and_absolute_paths_match_without_noise() {
    let relative =
        r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="m 10 10 l 20 0 l 0 20"/></svg>"#;
    let absolute =
        r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M 10 10 L 30 10 L 30 30"/></svg>"#;
    let config = JitterConfig {
        amplitude: 0.0,
        frequency: 1.0,
        stroke_width_var: 0.0,
    };

    assert_eq!(
        transform_svg(relative, &config, &options(42), None).unwrap(),
        transform_svg(absolute, &config, &options(42), None).unwrap()
    );
}

#[test]
fn transform_svg_leaves_malformed_path_data_unchanged() {
    let svg =
        r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M 0 0 L 1" stroke="black"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42), None).unwrap();

    assert!(transformed.contains(r#"<path d="M 0 0 L 1" stroke="black" />"#));
}

#[test]
fn transform_svg_leaves_degenerate_polyline_unchanged() {
    let svg =
        r#"<svg xmlns="http://www.w3.org/2000/svg"><polyline points="0 0" stroke="black"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42), None).unwrap();

    assert!(transformed.contains(r#"<polyline points="0 0" stroke="black" />"#));
}

#[test]
fn transform_svg_applies_seeded_stroke_width_jitter() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><line x1="0" y1="0" x2="10" y2="0" stroke-width="2"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42), None).unwrap();

    assert!(transformed.contains("stroke-width="));
    assert!(!transformed.contains(r#"stroke-width="2""#));
}

#[test]
fn transform_svg_can_disable_stroke_width_variation() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><line x1="0" y1="0" x2="10" y2="0" stroke-width="2"/></svg>"#;
    let transformed = transform_svg(
        svg,
        &JitterConfig {
            amplitude: 0.0,
            frequency: 1.0,
            stroke_width_var: 0.0,
        },
        &options(42),
        None,
    )
    .unwrap();

    assert!(transformed.contains(r#"stroke-width="2.000""#));
}

#[test]
fn transform_svg_preserves_namespace_prefixes() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><defs><path id="icon" d="M0 0 L1 1"/></defs><use xlink:href="#icon"/></svg>"##;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42), None).unwrap();

    assert!(transformed.contains(r#"xmlns:xlink="http://www.w3.org/1999/xlink""#));
    assert!(transformed.contains(r##"xlink:href="#icon""##));
}

#[test]
fn transform_svg_preserves_nested_namespace_declarations() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg"><g xmlns:bp="https://example.com/bp"><bp:meta bp:key="edge"/><line x1="0" y1="0" x2="1" y2="1"/></g></svg>"##;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42), None).unwrap();

    assert!(transformed.contains(r#"xmlns:bp="https://example.com/bp""#));
    assert!(transformed.contains(r#"<bp:meta bp:key="edge" />"#));
}

#[test]
fn transform_svg_does_not_jitter_non_svg_namespace_elements() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:bp="https://example.com/bp"><bp:path d="M0 0 L1 1" bp:key="edge"/><line x1="0" y1="0" x2="1" y2="1"/></svg>"##;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42), None).unwrap();

    assert!(transformed.contains(r##"<bp:path d="M0 0 L1 1" bp:key="edge" />"##));
    assert!(transformed.contains("<path"));
}

#[test]
fn transform_svg_does_not_jitter_defs_content() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><defs><clipPath id="clip"><path d="M0 0 L1 1"/></clipPath></defs><path d="M0 0 L5 5"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42), None).unwrap();

    assert!(transformed.contains(r#"<path d="M0 0 L1 1" />"#));
    assert!(!transformed.contains(r#"<path d="M0 0 L5 5" />"#));
}

// #4: font_family_override は廃止（usvg が SVG をパースする時点でフォントが
// 確定し、text は glyph path に展開される）。CLI フラグは残しているが、
// transform_svg レイヤでの font-family 書き換えロジックは存在しないので
// 対応するテストは削除した。フォント差し替えは `--font-dir` 経由で行う。
// 同様に「text 要素を tspan に分割して位置/回転/不透明度をジッターする」旧仕様は
// glyph path jitter に置き換わったため、tspan ベースのレイアウト保持テストも削除した。

#[test]
fn transform_svg_converts_circle_to_jittered_path() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
      <circle cx="50" cy="50" r="20" stroke="red" stroke-width="2"/>
    </svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42), None).unwrap();

    assert!(!transformed.contains("<circle"));
    assert!(transformed.contains("<path"));
    assert!(transformed.contains("d=\"M"));
    assert!(transformed.contains(r#"stroke="red""#));
    // Default theme (None) does not emit any per-shape filter attribute.
    assert!(!transformed.contains("filter=\"url(#"));
}

#[test]
fn transform_svg_converts_ellipse_to_jittered_path() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
      <ellipse cx="50" cy="50" rx="30" ry="20" stroke="blue" stroke-width="1.5"/>
    </svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42), None).unwrap();

    assert!(!transformed.contains("<ellipse"));
    assert!(transformed.contains("<path"));
    assert!(transformed.contains("d=\"M"));
    assert!(transformed.contains(r#"stroke="blue""#));
}

#[test]
fn transform_svg_converts_polygon_to_jittered_path() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
      <polygon points="10,10 20,20 30,10" stroke="green" fill="yellow"/>
    </svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42), None).unwrap();

    assert!(!transformed.contains("<polygon"));
    assert!(transformed.contains("<path"));
    assert!(transformed.contains("d=\"M"));
    assert!(transformed.contains("L"));
    assert!(transformed.contains("Z"));
    assert!(transformed.contains(r#"stroke="green""#));
}

#[test]
fn transform_svg_circle_ellipse_polygon_reproducible_with_same_seed() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
      <circle cx="50" cy="50" r="20"/>
      <ellipse cx="30" cy="30" rx="15" ry="10"/>
      <polygon points="10,10 20,20 30,10"/>
    </svg>"#;
    let config = JitterConfig::default();
    let out1 = transform_svg(svg, &config, &options(42), None).unwrap();
    let out2 = transform_svg(svg, &config, &options(42), None).unwrap();
    assert_eq!(out1, out2);
}

#[test]
fn transform_svg_circle_ellipse_polygon_different_with_different_seed() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
      <circle cx="50" cy="50" r="20"/>
    </svg>"#;
    let config = JitterConfig::default();
    let out1 = transform_svg(svg, &config, &options(42), None).unwrap();
    let out2 = transform_svg(svg, &config, &options(43), None).unwrap();
    assert_ne!(out1, out2);
}

// #4: 旧 text 仕様 (rotation 累積, x/y/font-size 保持, opacity ジッター, tspan 展開)
// は廃止。text は usvg で glyph path に展開され、既存 path jitter が適用される。
// 新仕様の text→path 統合テストは tests/text_to_path_integration.rs に置く。

/// Sumi/Watercolor の bleed_pass_params のマジックナンバーを仕様として固定する。
/// この値を緩く変えると、aquarelle ラスター pass の見た目が黙って変わるので
/// 数値ピンを 1 箇所に置く。値を変えたいときは意図的にこのテストを更新する。
#[test]
fn test_theme_bleed_pass_params_values_are_pinned() {
    let sumi = theme_style(Theme::Sumi)
        .bleed_pass_params()
        .expect("sumi must enable bleed pass");
    assert_eq!(sumi.radius, 3.0);
    assert_eq!(sumi.intensity, 0.3);
    assert_eq!(sumi.halo, 0.0);

    let watercolor = theme_style(Theme::Watercolor)
        .bleed_pass_params()
        .expect("watercolor must enable bleed pass");
    assert_eq!(watercolor.radius, 6.0);
    assert_eq!(watercolor.intensity, 0.5);
    assert_eq!(watercolor.halo, 0.4);
}

/// aquarelle bleed pass は sumi / watercolor 限定。他テーマは None を返し、
/// raster pass を走らせない（export 側で None 分岐に落ちる）。
#[test]
fn test_themes_without_bleed_pass_return_none() {
    for theme in [
        Theme::None,
        Theme::Blueprint,
        Theme::Chalk,
        Theme::Marker,
        Theme::Manga,
    ] {
        assert!(
            theme_style(theme).bleed_pass_params().is_none(),
            "{:?} must not enable bleed pass",
            theme
        );
    }
}

/// 旧 SVG フィルタ実装の残骸検出。subtle-bleed / sumi-ink-bleed / watercolor-bleed
/// の id と、それを参照する filter="url(#subtle-bleed)" が全テーマで出力に
/// 残っていないことを担保する。
#[test]
fn test_legacy_bleed_filter_defs_removed_for_all_themes() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <circle cx="50" cy="50" r="20" stroke="black"/>
    </svg>"#;
    let config = JitterConfig::default();

    for theme in [
        Theme::None,
        Theme::Blueprint,
        Theme::Sumi,
        Theme::Watercolor,
        Theme::Chalk,
        Theme::Marker,
        Theme::Manga,
    ] {
        let opts = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme,
        };
        let out = transform_svg(svg, &config, &opts, None).unwrap();
        assert!(
            !out.contains("<filter id=\"subtle-bleed"),
            "{:?} leaks subtle-bleed filter def",
            theme
        );
        assert!(
            !out.contains("id=\"sumi-ink-bleed\""),
            "{:?} leaks sumi-ink-bleed filter def",
            theme
        );
        assert!(
            !out.contains("id=\"watercolor-bleed\""),
            "{:?} leaks watercolor-bleed filter def",
            theme
        );
        assert!(
            !out.contains("url(#subtle-bleed)"),
            "{:?} still references subtle-bleed",
            theme
        );
    }
}

/// #4 廃止: 旧仕様では <text>/<tspan> に text-grunge filter が付与されていたが、
/// usvg で glyph path に展開する方式に置き換わったため、tspan も text-grunge も
/// 出力には現れない。代替検証は tests/text_to_path_integration.rs を参照。
#[test]
fn test_text_grunge_filter_is_removed_for_all_themes() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="80">
        <text x="20" y="50" font-size="24">Hi</text>
    </svg>"#;
    let config = JitterConfig::default();

    for theme in [
        Theme::None,
        Theme::Blueprint,
        Theme::Sumi,
        Theme::Watercolor,
        Theme::Chalk,
        Theme::Marker,
        Theme::Manga,
    ] {
        let opts = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme,
        };
        let out = transform_svg(svg, &config, &opts, None).unwrap();

        assert!(
            !out.contains("text-grunge"),
            "{:?} still emits text-grunge filter (deleted in #4)",
            theme
        );
        assert!(
            !out.contains("<tspan"),
            "{:?} still emits tspan (deleted in #4)",
            theme
        );
        assert!(
            !out.contains("<text "),
            "{:?} still emits <text> element",
            theme
        );
    }
}

/// chalk / marker は「per-shape filter_id を持つ」が「bleed_pass_params は None」のクラス。
/// この組み合わせが崩れると、aquarelle raster pass が掛かったうえに glyph effect も
/// 二重に乗ってしまうので、トレイト戻り値レベルで固定しておく。
#[test]
fn test_chalk_marker_have_filter_id_but_no_bleed_pass() {
    let chalk = theme_style(Theme::Chalk);
    assert_eq!(chalk.filter_id(), Some("chalk-dust"));
    assert!(chalk.bleed_pass_params().is_none());

    let marker = theme_style(Theme::Marker);
    assert_eq!(marker.filter_id(), Some("marker-glow"));
    assert!(marker.bleed_pass_params().is_none());
}

// ---------------------------------------------------------------------------
// #4 text→glyph path flatten 周りの異常系・境界値・テーマ横断テスト群
// ---------------------------------------------------------------------------

/// 観点: 異常系・エラー種別分離。
/// `<text>` を含む不正な SVG は flatten_text_to_paths の usvg パースで落ち、
/// transform_svg は TextFlattenError として包んで返す（XmlParseError と分離）。
#[test]
fn test_transform_svg_returns_text_flatten_error_for_broken_svg_with_text() {
    let result = transform_svg(
        "<text>broken without root",
        &JitterConfig::default(),
        &options(42),
        None,
    );
    match result {
        Err(TransformError::TextFlattenError(_)) => {}
        other => panic!("expected TextFlattenError, got {other:?}"),
    }
}

/// 観点: 異常系・責務分離。
/// `<text>` を含まない不正な SVG は flatten_text_to_paths を素通しし、
/// 後段の roxmltree::Document::parse が拾って XmlParseError を返す。
#[test]
fn test_transform_svg_returns_xml_parse_error_for_broken_svg_without_text() {
    let result = transform_svg("<rect>broken", &JitterConfig::default(), &options(42), None);
    match result {
        Err(TransformError::XmlParseError(_)) => {}
        other => panic!("expected XmlParseError, got {other:?}"),
    }
}

/// 観点: docs follow-through。
/// `font_family_override` は #4 glyph-path flatten 以降は transform 層では
/// 使われない（フォント差し替えは `--font-dir` 経由）。指定しても出力に
/// 反映されないことを担保する。
#[test]
fn test_transform_svg_font_family_override_is_noop_after_flatten() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="80">
        <text x="20" y="50" font-size="24">Hi</text>
    </svg>"##;
    let opts = TransformOptions {
        seed: Some(42),
        font_family_override: Some("Virgil".to_string()),
        theme: Theme::None,
    };
    let out = transform_svg(svg, &JitterConfig::default(), &opts, None).unwrap();
    assert!(
        !out.contains("Virgil"),
        "font_family_override is a no-op after #4 flatten, but output mentions Virgil: {out}"
    );
}

/// 観点: テーマ横断（既存 blueprint 版の chalk 版）。
/// text を含むことで rect が usvg で path に正規化されても、chalk テーマの
/// closed-shape fill ルールが effective_tag_for_fill 経由で効き、fill="none" が出る。
#[test]
fn test_transform_svg_closed_path_fill_rewrite_works_for_chalk_theme() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="80">
        <rect x="10" y="10" width="50" height="40" fill="red" stroke="black"/>
        <text x="60" y="50" font-size="14">Hi</text>
    </svg>"##;
    let opts = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Chalk,
    };
    let out = transform_svg(svg, &JitterConfig::default(), &opts, None).unwrap();
    assert!(
        out.contains(r#"fill="none""#),
        "chalk closed-shape fill rewrite did not apply: {out}"
    );
}

/// 観点: 誤検出経路の安全性。
/// XML コメント内の `<text>` は contains_text_element が拾う可能性があるが、
/// その場合でも usvg パースで panic せず Ok を返すことを担保する。
#[test]
fn test_transform_svg_handles_comment_with_text_tag_without_panic() {
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\"><!--<text>x</text>--></svg>";
    let result = transform_svg(svg, &JitterConfig::default(), &options(42), None);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

/// 観点: path 以外の素通り（effective_tag_for_fill 統合経由）。
/// text を含むことで usvg が走るが、`<line>` は usvg を通っても `<line>` のまま
/// （path-with-close ではない）か、line→path 変換でも path_is_closed=false に
/// なる。いずれにせよ blueprint テーマで fill="none" 強制は起きず、元の fill
/// 指定（"red"）が温存されること（あるいは line が消えていないこと）を確認する。
#[test]
fn test_effective_tag_for_fill_via_line_with_close_command_in_d() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <line x1="0" y1="0" x2="50" y2="50" stroke="black" fill="red"/>
        <text x="20" y="80" font-size="14">x</text>
    </svg>"##;
    let opts = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Blueprint,
    };
    let out = transform_svg(svg, &JitterConfig::default(), &opts, None).unwrap();
    // line は open path にしかならないので、fill="red" が "none" に書き換えられない
    // ことを担保する（line→path 変換後も path_is_closed=false で素通り）。
    // usvg を経由すると "red" → "#ff0000" に hex 展開される可能性があるので、
    // どちらの形でも line の fill 指定が温存されていることを担保する。
    let preserved_red = out.contains(r##"fill="red""##) || out.contains(r##"fill="#ff0000""##);
    assert!(
        preserved_red,
        "line's fill should not be forced to none in blueprint theme: {out}"
    );
}

/// 観点: null / 境界値。
/// `<path>` の `d` 属性が空・欠落でも panic しない（jitter pipeline が
/// degenerate path として処理し、blueprint テーマ適用も破綻しない）。
#[test]
fn test_transform_svg_path_with_empty_d_does_not_panic() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <text x="10" y="20" font-size="14">x</text>
        <path d="" stroke="red"/>
    </svg>"##;
    let opts = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Blueprint,
    };
    let result = transform_svg(svg, &JitterConfig::default(), &opts, None);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

/// 観点: Display 実装の固定。
/// TransformError::TextFlattenError は Display 経由で内包文字列を漏らす
/// （ログに inner エラー情報が乗ること）。フォーマット細則は実装に従う。
#[test]
fn test_transform_svg_text_flatten_error_display_format() {
    let err = TransformError::TextFlattenError("oops".to_string());
    let formatted = format!("{err}");
    assert!(
        formatted.contains("oops"),
        "Display should include inner message, got {formatted}"
    );
}

/// 観点: text 以外の text-like 要素 smoke。
/// `<textPath>` は contains_text_element に拾われる可能性があるが、
/// transform_svg として panic せず Ok を返すこと（usvg が解釈・正規化できる
/// か、できなくても TextFlattenError として表面化するだけで panic しない）。
#[test]
fn test_transform_svg_handles_textpath_without_panic() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="200" height="80">
        <defs><path id="p" d="M 10 40 L 190 40"/></defs>
        <text><textPath xlink:href="#p">foo</textPath></text>
    </svg>"##;
    let result = transform_svg(svg, &JitterConfig::default(), &options(42), None);
    // panic しなければ Ok でも Err でもよい。実機では Ok を期待するが、usvg の
    // textPath 対応に依存するため、いずれにせよ catch_unwind なしで返ること
    // だけを担保する。
    let _ = result;
}
