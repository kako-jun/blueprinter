use std::fs;

use blueprinter::svg::{parse_svg, Primitive};

#[test]
fn test_parse_simple_svg() {
    let svg = fs::read_to_string("tests/fixtures/simple.svg").unwrap();
    let primitives = parse_svg(&svg).unwrap();

    assert_eq!(primitives.len(), 3);

    // rect
    assert_eq!(
        primitives[0],
        Primitive::Rect {
            x: 10.0,
            y: 10.0,
            width: 30.0,
            height: 20.0,
            fill: Some("blue".to_string()),
            stroke: Some("black".to_string()),
            stroke_width: Some(2.0),
        }
    );

    // text
    assert_eq!(
        primitives[1],
        Primitive::Text {
            x: 50.0,
            y: 50.0,
            content: "Hello".to_string(),
            font_family: Some("Arial".to_string()),
            font_size: Some(12.0),
            fill: Some("red".to_string()),
        }
    );

    // group -> circle
    match &primitives[2] {
        Primitive::Group { children } => {
            assert_eq!(children.len(), 1);
            assert_eq!(
                children[0],
                Primitive::Circle {
                    cx: 80.0,
                    cy: 80.0,
                    r: 10.0,
                    fill: Some("green".to_string()),
                    stroke: None,
                    stroke_width: None,
                }
            );
        }
        _ => panic!("Expected Group, got {:?}", primitives[2]),
    }
}

#[test]
fn test_unknown_element_passes_through() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
        <custom-tag foo="bar" baz="123"/>
    </svg>"#;
    let primitives = parse_svg(svg).unwrap();

    assert_eq!(primitives.len(), 1);
    assert_eq!(
        primitives[0],
        Primitive::Unknown {
            tag: "custom-tag".to_string(),
            attrs: vec![
                ("foo".to_string(), "bar".to_string()),
                ("baz".to_string(), "123".to_string()),
            ],
        }
    );
}
