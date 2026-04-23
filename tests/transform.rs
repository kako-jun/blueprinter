use std::fs;

use blueprinter::jitter::JitterConfig;
use blueprinter::svg::transform_svg;

#[test]
fn transform_svg_preserves_root_and_writes_jittered_paths() {
    let svg = fs::read_to_string("tests/fixtures/simple.svg").unwrap();
    let transformed = transform_svg(&svg, &JitterConfig::default(), Some(42)).unwrap();

    assert!(transformed
        .starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">"#));
    assert!(transformed.contains("<path"));
    assert!(transformed.contains("<text x=\"50\" y=\"50\""));
    assert!(transformed.contains("<g>"));
    assert!(transformed.contains("<circle"));
}

#[test]
fn transform_svg_with_same_seed_is_reproducible() {
    let svg = fs::read_to_string("tests/fixtures/simple.svg").unwrap();
    let config = JitterConfig::default();
    let out1 = transform_svg(&svg, &config, Some(42)).unwrap();
    let out2 = transform_svg(&svg, &config, Some(42)).unwrap();
    assert_eq!(out1, out2);
}

#[test]
fn transform_svg_with_different_seed_changes_jitter() {
    let svg = fs::read_to_string("tests/fixtures/simple.svg").unwrap();
    let config = JitterConfig::default();
    let out1 = transform_svg(&svg, &config, Some(42)).unwrap();
    let out2 = transform_svg(&svg, &config, Some(43)).unwrap();
    assert_ne!(out1, out2);
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
    let transformed = transform_svg(svg, &JitterConfig::default(), Some(42)).unwrap();

    assert!(transformed.contains(
        r##"<defs><linearGradient id="g"><stop offset="0%" stop-color="#fff" /></linearGradient></defs>"##
    ));
    assert!(transformed.contains(r#"<g id="layer1" class="node" transform="translate(1 2)">"#));
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
    let transformed = transform_svg(svg, &JitterConfig::default(), Some(42)).unwrap();

    assert!(transformed.contains(r#"title="A &amp; B""#));
    assert!(transformed.contains(">A &amp; B</text>"));
    assert!(!transformed.contains(">A & B</text>"));
}

#[test]
fn transform_svg_handles_compact_path_data() {
    let svg =
        r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M0-1L2.5.5Z" stroke="black"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), Some(42)).unwrap();

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
        Some(42),
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
        transform_svg(relative, &config, Some(42)).unwrap(),
        transform_svg(absolute, &config, Some(42)).unwrap()
    );
}

#[test]
fn transform_svg_leaves_malformed_path_data_unchanged() {
    let svg =
        r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M 0 0 L 1" stroke="black"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), Some(42)).unwrap();

    assert!(transformed.contains(r#"<path d="M 0 0 L 1" stroke="black" />"#));
}

#[test]
fn transform_svg_leaves_degenerate_polyline_unchanged() {
    let svg =
        r#"<svg xmlns="http://www.w3.org/2000/svg"><polyline points="0 0" stroke="black"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), Some(42)).unwrap();

    assert!(transformed.contains(r#"<polyline points="0 0" stroke="black" />"#));
}

#[test]
fn transform_svg_applies_seeded_stroke_width_jitter() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><line x1="0" y1="0" x2="10" y2="0" stroke-width="2"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), Some(42)).unwrap();

    assert!(transformed.contains("stroke-width="));
    assert!(!transformed.contains(r#"stroke-width="2""#));
}

#[test]
fn transform_svg_preserves_namespace_prefixes() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><defs><path id="icon" d="M0 0 L1 1"/></defs><use xlink:href="#icon"/></svg>"##;
    let transformed = transform_svg(svg, &JitterConfig::default(), Some(42)).unwrap();

    assert!(transformed.contains(r#"xmlns:xlink="http://www.w3.org/1999/xlink""#));
    assert!(transformed.contains(r##"xlink:href="#icon""##));
}

#[test]
fn transform_svg_preserves_nested_namespace_declarations() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg"><g xmlns:bp="https://example.com/bp"><bp:meta bp:key="edge"/><line x1="0" y1="0" x2="1" y2="1"/></g></svg>"##;
    let transformed = transform_svg(svg, &JitterConfig::default(), Some(42)).unwrap();

    assert!(transformed.contains(r#"xmlns:bp="https://example.com/bp""#));
    assert!(transformed.contains(r#"<bp:meta bp:key="edge" />"#));
}

#[test]
fn transform_svg_does_not_jitter_non_svg_namespace_elements() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:bp="https://example.com/bp"><bp:path d="M0 0 L1 1" bp:key="edge"/><line x1="0" y1="0" x2="1" y2="1"/></svg>"##;
    let transformed = transform_svg(svg, &JitterConfig::default(), Some(42)).unwrap();

    assert!(transformed.contains(r##"<bp:path d="M0 0 L1 1" bp:key="edge" />"##));
    assert!(transformed.contains("<path"));
}

#[test]
fn transform_svg_does_not_jitter_defs_content() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><defs><clipPath id="clip"><path d="M0 0 L1 1"/></clipPath></defs><path d="M0 0 L5 5"/></svg>"#;
    let transformed = transform_svg(svg, &JitterConfig::default(), Some(42)).unwrap();

    assert!(transformed.contains(r#"<path d="M0 0 L1 1" />"#));
    assert!(!transformed.contains(r#"<path d="M0 0 L5 5" />"#));
}
