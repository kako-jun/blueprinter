use blueprinter::jitter::{jitter_primitive, JitterConfig};
use blueprinter::svg::Primitive;

#[test]
fn test_jitter_rect_outputs_path() {
    let rect = Primitive::Rect {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
        fill: Some("none".to_string()),
        stroke: Some("black".to_string()),
        stroke_width: Some(2.0),
    };
    let config = JitterConfig::default();
    let output = jitter_primitive(&rect, &config);
    assert!(output.starts_with("<path"));
    assert!(output.contains("d="));
}

#[test]
fn test_jitter_line_outputs_path() {
    let line = Primitive::Line {
        x1: 0.0,
        y1: 0.0,
        x2: 100.0,
        y2: 100.0,
        stroke: Some("black".to_string()),
        stroke_width: Some(1.0),
    };
    let config = JitterConfig::default();
    let output = jitter_primitive(&line, &config);
    assert!(output.starts_with("<path"));
    assert!(output.contains("d="));
}

#[test]
fn test_jitter_path_changes_d() {
    let path = Primitive::Path {
        d: "M 10 10 L 50 50 C 60 60 70 70 80 80 Z".to_string(),
        fill: None,
        stroke: Some("red".to_string()),
        stroke_width: Some(1.5),
    };
    let config = JitterConfig::default();
    let output = jitter_primitive(&path, &config);
    assert!(output.starts_with("<path"));
    // 元の d と異なるはず
    assert!(!output.contains(r#"d="M 10 10 L 50 50 C 60 60 70 70 80 80 Z""#));
}

#[test]
fn test_jitter_randomness() {
    let rect = Primitive::Rect {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
        fill: None,
        stroke: Some("black".to_string()),
        stroke_width: None,
    };
    let config = JitterConfig::default();
    let out1 = jitter_primitive(&rect, &config);
    let out2 = jitter_primitive(&rect, &config);
    assert_ne!(out1, out2, "jitter output should be random");
}
