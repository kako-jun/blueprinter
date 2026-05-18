use std::fs;

use blueprinter::jitter::JitterConfig;
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
    let transformed = transform_svg(&svg, &JitterConfig::default(), &options(42)).unwrap();

    assert!(transformed
        .starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">"#));
    assert!(transformed.contains("<path"));
    assert!(transformed.contains("<text x="));
    assert!(transformed.contains("<g>"));
    // circle elements are now converted to paths with jitter, so they no longer appear as <circle
    assert!(transformed.contains("d=\"M"));
}

#[test]
fn transform_svg_with_same_seed_is_reproducible() {
    let svg = fs::read_to_string("tests/fixtures/simple.svg").unwrap();
    let config = JitterConfig::default();
    let out1 = transform_svg(&svg, &config, &options(42)).unwrap();
    let out2 = transform_svg(&svg, &config, &options(42)).unwrap();
    assert_eq!(out1, out2);
}

#[test]
fn transform_svg_with_different_seed_changes_jitter() {
    let svg = fs::read_to_string("tests/fixtures/simple.svg").unwrap();
    let config = JitterConfig::default();
    let out1 = transform_svg(&svg, &config, &options(42)).unwrap();
    let out2 = transform_svg(&svg, &config, &options(43)).unwrap();
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
    let out1 = transform_svg(&svg, &config, &options(42)).unwrap();
    let out2 = transform_svg(&svg, &config, &options(42)).unwrap();
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

    let subtle_out = transform_svg(&svg, &subtle, &options(42)).unwrap();
    let rough_out = transform_svg(&svg, &rough, &options(42)).unwrap();

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
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

    // defs now includes blueprinter filter content, so check components separately
    assert!(transformed.contains(
        r##"<linearGradient id="g"><stop offset="0%" stop-color="#fff" /></linearGradient>"##
    ));
    assert!(transformed.contains("<filter id=\"text-grunge\""));
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

#[test]
fn transform_svg_escapes_decoded_text_and_attributes() {
    let svg =
        r#"<svg xmlns="http://www.w3.org/2000/svg"><text title="A &amp; B">A &amp; B</text></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

    assert!(transformed.contains(r#"title="A &amp; B""#));
    // text は tspan で分割されるため、各文字が別の tspan に入る
    assert!(transformed.contains(">A</tspan>"));
    assert!(transformed.contains(">&amp;</tspan>"));
    assert!(transformed.contains(">B</tspan>"));
    assert!(!transformed.contains(">A & B</text>"));
}

#[test]
fn transform_svg_handles_compact_path_data() {
    let svg =
        r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M0-1L2.5.5Z" stroke="black"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

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
        transform_svg(relative, &config, &options(42)).unwrap(),
        transform_svg(absolute, &config, &options(42)).unwrap()
    );
}

#[test]
fn transform_svg_leaves_malformed_path_data_unchanged() {
    let svg =
        r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M 0 0 L 1" stroke="black"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

    assert!(transformed.contains(r#"<path d="M 0 0 L 1" stroke="black" />"#));
}

#[test]
fn transform_svg_leaves_degenerate_polyline_unchanged() {
    let svg =
        r#"<svg xmlns="http://www.w3.org/2000/svg"><polyline points="0 0" stroke="black"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

    assert!(transformed.contains(r#"<polyline points="0 0" stroke="black" />"#));
}

#[test]
fn transform_svg_applies_seeded_stroke_width_jitter() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><line x1="0" y1="0" x2="10" y2="0" stroke-width="2"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

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
    )
    .unwrap();

    assert!(transformed.contains(r#"stroke-width="2.000""#));
}

#[test]
fn transform_svg_preserves_namespace_prefixes() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><defs><path id="icon" d="M0 0 L1 1"/></defs><use xlink:href="#icon"/></svg>"##;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

    assert!(transformed.contains(r#"xmlns:xlink="http://www.w3.org/1999/xlink""#));
    assert!(transformed.contains(r##"xlink:href="#icon""##));
}

#[test]
fn transform_svg_preserves_nested_namespace_declarations() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg"><g xmlns:bp="https://example.com/bp"><bp:meta bp:key="edge"/><line x1="0" y1="0" x2="1" y2="1"/></g></svg>"##;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

    assert!(transformed.contains(r#"xmlns:bp="https://example.com/bp""#));
    assert!(transformed.contains(r#"<bp:meta bp:key="edge" />"#));
}

#[test]
fn transform_svg_does_not_jitter_non_svg_namespace_elements() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:bp="https://example.com/bp"><bp:path d="M0 0 L1 1" bp:key="edge"/><line x1="0" y1="0" x2="1" y2="1"/></svg>"##;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

    assert!(transformed.contains(r##"<bp:path d="M0 0 L1 1" bp:key="edge" />"##));
    assert!(transformed.contains("<path"));
}

#[test]
fn transform_svg_does_not_jitter_defs_content() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><defs><clipPath id="clip"><path d="M0 0 L1 1"/></clipPath></defs><path d="M0 0 L5 5"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

    assert!(transformed.contains(r#"<path d="M0 0 L1 1" />"#));
    assert!(!transformed.contains(r#"<path d="M0 0 L5 5" />"#));
}

#[test]
fn transform_svg_preserves_existing_font_family_when_override_is_absent() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><text x="10" y="20" font-family="Arial" font-size="12">Hello</text></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

    assert!(transformed.contains(r#"font-family="Arial""#));
    assert!(!transformed.contains(r#"font-family="Virgil""#));
}

#[test]
fn transform_svg_can_override_font_family_for_text() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><text x="10" y="20" font-family="Arial" font-size="12">Hello</text></svg>"#;
    let transformed = transform_svg(
        svg,
        &JitterConfig::default(),
        &TransformOptions {
            seed: Some(42),
            font_family_override: Some("Virgil".to_string()),
            theme: Default::default(),
        },
    )
    .unwrap();

    assert!(transformed.contains(r#"font-family="Virgil""#));
    assert!(!transformed.contains(r#"font-family="Arial""#));
}

#[test]
fn transform_svg_adds_font_family_override_when_input_relies_on_stylesheet() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><text x="10" y="20">Hello</text></svg>"#;
    let transformed = transform_svg(
        svg,
        &JitterConfig::default(),
        &TransformOptions {
            seed: Some(42),
            font_family_override: Some("Virgil".to_string()),
            theme: Default::default(),
        },
    )
    .unwrap();

    assert!(transformed.contains(r#"font-family="Virgil""#));
}

#[test]
fn transform_svg_keeps_text_layout_and_only_jitters_rotation_and_opacity() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><text x="10" y="20" font-size="12">Hello</text></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

    assert!(transformed.contains(r#"font-size="12""#));
    // text は tspan に分割され、各文字に位置・傾きジッターが適用される
    assert!(transformed.contains("<tspan"));
    assert!(transformed.contains(r#"x="9."#) || transformed.contains(r#"x="10"#));
    assert!(transformed.contains(r#"y="19."#) || transformed.contains(r#"y="20"#));
    assert!(transformed.contains("transform=\"rotate("));
}

#[test]
fn transform_svg_converts_circle_to_jittered_path() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
      <circle cx="50" cy="50" r="20" stroke="red" stroke-width="2"/>
    </svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

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
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

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
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

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
    let out1 = transform_svg(svg, &config, &options(42)).unwrap();
    let out2 = transform_svg(svg, &config, &options(42)).unwrap();
    assert_eq!(out1, out2);
}

#[test]
fn transform_svg_circle_ellipse_polygon_different_with_different_seed() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
      <circle cx="50" cy="50" r="20"/>
    </svg>"#;
    let config = JitterConfig::default();
    let out1 = transform_svg(svg, &config, &options(42)).unwrap();
    let out2 = transform_svg(svg, &config, &options(43)).unwrap();
    assert_ne!(out1, out2);
}

#[test]
fn transform_svg_appends_rotation_to_existing_text_transform() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><text x="10" y="20" transform="translate(1 2)">Hello</text></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

    assert!(transformed.contains(r#"transform="translate(1 2) rotate("#));
}

#[test]
fn transform_svg_preserves_text_position_and_font_size_exactly() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><text x="40" y="70" font-size="28">Hello</text></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

    assert!(transformed.contains(r#"x="40""#));
    assert!(transformed.contains(r#"y="70""#));
    assert!(transformed.contains(r#"font-size="28""#));
}

#[test]
fn transform_svg_jitters_existing_text_opacity() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><text x="10" y="20" opacity="0.8">Hello</text></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), &options(42)).unwrap();

    // text 要素の opacity はジッターされ、各 tspan にも個別のジッター opacity が適用される
    assert!(transformed.contains(r#"opacity="0.8"#)); // opacity 属性が存在する
    assert!(!transformed.contains(r#"opacity="0.800""#)); // ジッターされているため、元の値は保持されない
    assert!(transformed.contains("<tspan"));
    // 複数の tspan が存在
    assert!(transformed.matches("<tspan").count() > 1);
}

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
        let out = transform_svg(svg, &config, &opts).unwrap();
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

/// 残置仕様: <text>/<tspan> には text-grunge フィルタだけが付き、廃止された
/// subtle-bleed への参照は一切残らない。全テーマで成立する。
#[test]
fn test_text_filter_only_text_grunge() {
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
        let out = transform_svg(svg, &config, &opts).unwrap();

        // 文字ごとに tspan が出る前提
        assert!(out.contains("<tspan"), "{:?} dropped tspan output", theme);
        assert!(
            out.contains(r#"filter="url(#text-grunge)""#),
            "{:?} tspan must keep text-grunge filter",
            theme
        );
        assert!(
            !out.contains("url(#subtle-bleed)"),
            "{:?} tspan must not reference subtle-bleed",
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
