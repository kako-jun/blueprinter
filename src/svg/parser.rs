use roxmltree::{Document, Node};

use crate::svg::primitive::Primitive;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    XmlParseError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::XmlParseError(msg) => write!(f, "XML parse error: {msg}"),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse_svg(input: &str) -> Result<Vec<Primitive>, ParseError> {
    let doc = Document::parse(input).map_err(|e| ParseError::XmlParseError(e.to_string()))?;
    let root = doc.root_element();
    parse_children(&root)
}

fn parse_children(node: &Node<'_, '_>) -> Result<Vec<Primitive>, ParseError> {
    let mut primitives = Vec::new();
    for child in node.children().filter(|n| n.is_element()) {
        primitives.push(parse_node(&child));
    }
    Ok(primitives)
}

pub fn parse_node(node: &Node<'_, '_>) -> Primitive {
    let tag = node.tag_name().name();
    match tag {
        "rect" => Primitive::Rect {
            x: attr_f64(node, "x").unwrap_or(0.0),
            y: attr_f64(node, "y").unwrap_or(0.0),
            width: attr_f64(node, "width").unwrap_or(0.0),
            height: attr_f64(node, "height").unwrap_or(0.0),
            fill: attr_string(node, "fill"),
            stroke: attr_string(node, "stroke"),
            stroke_width: attr_f64(node, "stroke-width"),
        },
        "line" => Primitive::Line {
            x1: attr_f64(node, "x1").unwrap_or(0.0),
            y1: attr_f64(node, "y1").unwrap_or(0.0),
            x2: attr_f64(node, "x2").unwrap_or(0.0),
            y2: attr_f64(node, "y2").unwrap_or(0.0),
            stroke: attr_string(node, "stroke"),
            stroke_width: attr_f64(node, "stroke-width"),
        },
        "polyline" => Primitive::Polyline {
            points: parse_points(node.attribute("points").unwrap_or("")),
            stroke: attr_string(node, "stroke"),
            stroke_width: attr_f64(node, "stroke-width"),
        },
        "path" => Primitive::Path {
            d: attr_string(node, "d").unwrap_or_default(),
            fill: attr_string(node, "fill"),
            stroke: attr_string(node, "stroke"),
            stroke_width: attr_f64(node, "stroke-width"),
        },
        "circle" => Primitive::Circle {
            cx: attr_f64(node, "cx").unwrap_or(0.0),
            cy: attr_f64(node, "cy").unwrap_or(0.0),
            r: attr_f64(node, "r").unwrap_or(0.0),
            fill: attr_string(node, "fill"),
            stroke: attr_string(node, "stroke"),
            stroke_width: attr_f64(node, "stroke-width"),
        },
        "ellipse" => Primitive::Ellipse {
            cx: attr_f64(node, "cx").unwrap_or(0.0),
            cy: attr_f64(node, "cy").unwrap_or(0.0),
            rx: attr_f64(node, "rx").unwrap_or(0.0),
            ry: attr_f64(node, "ry").unwrap_or(0.0),
            fill: attr_string(node, "fill"),
            stroke: attr_string(node, "stroke"),
            stroke_width: attr_f64(node, "stroke-width"),
        },
        "polygon" => Primitive::Polygon {
            points: parse_points(node.attribute("points").unwrap_or("")),
            fill: attr_string(node, "fill"),
            stroke: attr_string(node, "stroke"),
            stroke_width: attr_f64(node, "stroke-width"),
        },
        "text" => Primitive::Text {
            x: attr_f64(node, "x").unwrap_or(0.0),
            y: attr_f64(node, "y").unwrap_or(0.0),
            content: node.children().filter_map(|n| n.text()).collect::<String>(),
            font_family: attr_string(node, "font-family"),
            font_size: attr_f64(node, "font-size"),
            fill: attr_string(node, "fill"),
        },
        "g" => Primitive::Group {
            children: parse_children(node).unwrap_or_default(),
        },
        _ => Primitive::Unknown {
            tag: tag.to_string(),
            attrs: node
                .attributes()
                .map(|a| (a.name().to_string(), a.value().to_string()))
                .collect(),
        },
    }
}

fn attr_f64(node: &Node<'_, '_>, name: &str) -> Option<f64> {
    node.attribute(name)?.parse::<f64>().ok()
}

fn attr_string(node: &Node<'_, '_>, name: &str) -> Option<String> {
    Some(node.attribute(name)?.to_string())
}

fn parse_points(s: &str) -> Vec<(f64, f64)> {
    let values: Vec<f64> = s
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<f64>().ok())
        .collect();
    values
        .chunks_exact(2)
        .map(|pair| (pair[0], pair[1]))
        .collect()
}
