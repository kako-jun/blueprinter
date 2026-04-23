use roxmltree::{Document, Node};

use crate::jitter::{jitter_primitive_path_with_seed, JitterConfig, JitteredPath};
use crate::svg::primitive::Primitive;

const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";
const SVG_NS: &str = "http://www.w3.org/2000/svg";
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

#[derive(Debug, PartialEq)]
pub enum TransformError {
    XmlParseError(String),
}

impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransformError::XmlParseError(msg) => write!(f, "XML parse error: {msg}"),
        }
    }
}

impl std::error::Error for TransformError {}

pub fn transform_svg(
    input: &str,
    config: &JitterConfig,
    seed: Option<u64>,
) -> Result<String, TransformError> {
    let doc = Document::parse(input).map_err(|e| TransformError::XmlParseError(e.to_string()))?;
    let mut state = seed;
    Ok(serialize_node(doc.root_element(), config, &mut state))
}

fn serialize_node(
    node: Node<'_, '_>,
    config: &JitterConfig,
    seed_state: &mut Option<u64>,
) -> String {
    if node.is_text() {
        return escape_text(node.text().unwrap_or_default());
    }
    if !node.is_element() {
        return String::new();
    }

    if should_jitter(&node) {
        let primitive = crate::svg::parser::parse_node(&node);
        if let Some(path) = jittered_path_data(&primitive, config, seed_state) {
            return serialize_jittered_path(node, &path);
        }
    }

    serialize_original_element(node, config, seed_state)
}

fn should_jitter(node: &Node<'_, '_>) -> bool {
    if is_inside_non_visual_container(node) {
        return false;
    }
    if !is_svg_element(node) {
        return false;
    }

    match node.tag_name().name() {
        "line" | "polyline" | "path" => true,
        "rect" => node.attribute("rx").is_none() && node.attribute("ry").is_none(),
        _ => false,
    }
}

fn is_svg_element(node: &Node<'_, '_>) -> bool {
    matches!(node.tag_name().namespace(), Some(SVG_NS) | None)
}

fn is_inside_non_visual_container(node: &Node<'_, '_>) -> bool {
    node.ancestors().skip(1).any(|ancestor| {
        if !ancestor.is_element() {
            return false;
        }
        matches!(
            ancestor.tag_name().name(),
            "defs"
                | "clipPath"
                | "mask"
                | "marker"
                | "pattern"
                | "linearGradient"
                | "radialGradient"
                | "symbol"
        )
    })
}

fn jittered_path_data(
    primitive: &Primitive,
    config: &JitterConfig,
    seed_state: &mut Option<u64>,
) -> Option<JitteredPath> {
    jitter_primitive_path_with_seed(primitive, config, seed_state)
}

fn serialize_jittered_path(source: Node<'_, '_>, path: &JitteredPath) -> String {
    let tag = qualified_replacement_path_name(source);
    let mut out = String::from("<");
    out.push_str(&tag);
    for namespace in source.namespaces() {
        if namespace_is_inherited(source, namespace.name(), namespace.uri()) {
            continue;
        }
        match namespace.name() {
            Some(prefix) => {
                out.push_str(&format_attr(&format!("xmlns:{prefix}"), namespace.uri()));
            }
            None => out.push_str(&format_attr("xmlns", namespace.uri())),
        }
    }
    out.push_str(&format_attr("d", &path.d));
    for attr in source.attributes() {
        if path.stroke_width.is_some() && attr.name() == "stroke-width" {
            continue;
        }
        if !is_geometry_attr(source.tag_name().name(), attr.name()) {
            out.push_str(&format_attr(
                &qualified_attr_name(source, &attr),
                attr.value(),
            ));
        }
    }
    if let Some(stroke_width) = path.stroke_width {
        out.push_str(&format_attr("stroke-width", &format!("{stroke_width:.3}")));
    }
    out.push_str(" />");
    out
}

fn serialize_original_element(
    node: Node<'_, '_>,
    config: &JitterConfig,
    seed_state: &mut Option<u64>,
) -> String {
    let tag = qualified_tag_name(node);
    let mut out = String::new();
    out.push('<');
    out.push_str(&tag);
    for namespace in node.namespaces() {
        if namespace_is_inherited(node, namespace.name(), namespace.uri()) {
            continue;
        }
        match namespace.name() {
            Some(prefix) => {
                out.push_str(&format_attr(&format!("xmlns:{prefix}"), namespace.uri()));
            }
            None => out.push_str(&format_attr("xmlns", namespace.uri())),
        }
    }
    for attr in node.attributes() {
        out.push_str(&format_attr(
            &qualified_attr_name(node, &attr),
            attr.value(),
        ));
    }

    let children: Vec<_> = node.children().collect();
    if children.is_empty() {
        out.push_str(" />");
        return out;
    }

    out.push('>');
    for child in children {
        out.push_str(&serialize_node(child, config, seed_state));
    }
    out.push_str("</");
    out.push_str(&tag);
    out.push('>');
    out
}

fn qualified_tag_name(node: Node<'_, '_>) -> String {
    let name = node.tag_name();
    if let Some(namespace) = name.namespace() {
        if let Some(prefix) = node.lookup_prefix(namespace) {
            if !prefix.is_empty() {
                return format!("{prefix}:{}", name.name());
            }
        }
    }
    name.name().to_string()
}

fn qualified_replacement_path_name(source: Node<'_, '_>) -> String {
    if source.tag_name().namespace() == Some(SVG_NS) {
        if let Some(prefix) = source.lookup_prefix(SVG_NS) {
            if !prefix.is_empty() {
                return format!("{prefix}:path");
            }
        }
    }
    "path".to_string()
}

fn namespace_is_inherited(node: Node<'_, '_>, prefix: Option<&str>, uri: &str) -> bool {
    node.parent_element()
        .and_then(|parent| parent.lookup_namespace_uri(prefix))
        == Some(uri)
}

fn qualified_attr_name(node: Node<'_, '_>, attr: &roxmltree::Attribute<'_, '_>) -> String {
    if let Some(namespace) = attr.namespace() {
        let prefix = node
            .lookup_prefix(namespace)
            .or_else(|| (namespace == XML_NS).then_some("xml"))
            .or_else(|| (namespace == XLINK_NS).then_some("xlink"));
        if let Some(prefix) = prefix {
            return format!("{prefix}:{}", attr.name());
        }
    }
    attr.name().to_string()
}

fn is_geometry_attr(tag: &str, name: &str) -> bool {
    matches!(
        (tag, name),
        ("rect", "x")
            | ("rect", "y")
            | ("rect", "width")
            | ("rect", "height")
            | ("line", "x1")
            | ("line", "y1")
            | ("line", "x2")
            | ("line", "y2")
            | ("polyline", "points")
            | ("path", "d")
    )
}

fn format_attr(name: &str, value: &str) -> String {
    format!(r#" {name}="{}""#, escape_attr(value))
}

fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(value: &str) -> String {
    escape_text(value).replace('"', "&quot;")
}
