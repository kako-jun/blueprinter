use std::fs;

use blueprinter::jitter::JitterConfig;
use blueprinter::svg::{transform_svg, TransformOptions};

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
