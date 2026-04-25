use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TransformOptions {
    pub seed: Option<u64>,
    pub font_family_override: Option<String>,
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
    options: &TransformOptions,
) -> Result<String, TransformError> {
    let doc = Document::parse(input).map_err(|e| TransformError::XmlParseError(e.to_string()))?;
    let mut state = options.seed;
    Ok(serialize_node(
        doc.root_element(),
        config,
        options,
        &mut state,
    ))
}

fn serialize_node(
    node: Node<'_, '_>,
    config: &JitterConfig,
    options: &TransformOptions,
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

    serialize_original_element(node, config, options, seed_state)
}

fn should_jitter(node: &Node<'_, '_>) -> bool {
    if is_inside_non_visual_container(node) {
        return false;
    }
    if !is_svg_element(node) {
        return false;
    }

    match node.tag_name().name() {
        "line" | "polyline" | "path" | "text" => true,
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
    out.push_str(r#" filter="url(#subtle-bleed)""#);
    out.push_str(" />");
    out
}

fn serialize_original_element(
    node: Node<'_, '_>,
    config: &JitterConfig,
    options: &TransformOptions,
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
    if should_jitter_text(&node) {
        serialize_text_attrs(node, config, options, seed_state, &mut out);
    } else {
        for attr in node.attributes() {
            out.push_str(&format_attr(
                &qualified_attr_name(node, &attr),
                attr.value(),
            ));
        }
    }

    let children: Vec<_> = node.children().collect();
    if children.is_empty() {
        out.push_str(" />");
        return out;
    }

    out.push('>');
    if tag == "svg" {
        insert_svg_defs(&mut out);
    }
    if tag == "text" {
        serialize_text_content(node, config, options, seed_state, &mut out);
    } else {
        for child in children {
            out.push_str(&serialize_node(child, config, options, seed_state));
        }
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

fn should_jitter_text(node: &Node<'_, '_>) -> bool {
    if is_inside_non_visual_container(node) || !is_svg_element(node) {
        return false;
    }

    matches!(node.tag_name().name(), "text" | "tspan")
}

fn insert_svg_defs(out: &mut String) {
    out.push_str(r#"<defs>"#);
    // Text grunge texture filter
    out.push_str(r#"<filter id="text-grunge" x="-20%" y="-20%" width="140%" height="140%"><feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="4" result="noise" seed="42"/><feDisplacementMap in="SourceGraphic" in2="noise" scale="0.8" xChannelSelector="R" yChannelSelector="G"/></filter>"#);
    // Subtle bleed for both lines and text
    out.push_str(r#"<filter id="subtle-bleed" x="-10%" y="-10%" width="120%" height="120%"><feGaussianBlur in="SourceGraphic" stdDeviation="0.3" result="blurred"/><feOffset in="blurred" dx="0.2" dy="0.2" result="offset"/><feComponentTransfer in="offset" result="faded"><feFuncA type="linear" slope="0.15"/></feComponentTransfer><feComposite in="faded" in2="SourceGraphic" operator="lighten"/></filter>"#);
    out.push_str(r#"</defs>"#);
}

fn serialize_text_content(
    node: Node<'_, '_>,
    config: &JitterConfig,
    _options: &TransformOptions,
    seed_state: &mut Option<u64>,
    out: &mut String,
) {
    let text_content = node.text().unwrap_or_default();
    if text_content.trim().is_empty() {
        return;
    }

    let mut rng = next_rng(seed_state);
    let rotation_amplitude = (config.amplitude * 0.3).clamp(0.0, 1.5);
    let opacity_amplitude = (config.stroke_width_var * 0.2).clamp(0.0, 0.08);
    let position_amplitude = config.amplitude * 0.2;

    let base_x = node.attribute("x").and_then(|s| s.parse::<f64>().ok());
    let base_y = node.attribute("y").and_then(|s| s.parse::<f64>().ok());

    for (i, ch) in text_content.chars().enumerate() {
        let char_x = base_x.map(|x| x + i as f64 * 6.0);
        let char_y = base_y;

        out.push_str("<tspan");

        if let Some(x) = char_x {
            let jx = uniform_noise(&mut rng, position_amplitude);
            out.push_str(&format!(r#" x="{:.3}""#, x + jx));
        }

        if let Some(y) = char_y {
            let jy = uniform_noise(&mut rng, position_amplitude);
            out.push_str(&format!(r#" y="{:.3}""#, y + jy));
        }

        let opacity = jittered_opacity(1.0, &mut rng, opacity_amplitude);
        if (opacity - 1.0).abs() > 0.001 {
            out.push_str(&format!(r#" opacity="{:.3}""#, opacity));
        }

        if let (Some(x), Some(y)) = (char_x, char_y) {
            let angle = uniform_noise(&mut rng, rotation_amplitude);
            if angle.abs() > 0.01 {
                out.push_str(&format!(r#" transform="rotate({:.2} {:.3} {:.3})""#, angle, x, y));
            }
        }

        out.push_str(r#" filter="url(#text-grunge) url(#subtle-bleed)""#);

        out.push('>');
        out.push_str(&escape_text(&ch.to_string()));
        out.push_str("</tspan>");
    }
}

fn serialize_text_attrs(
    node: Node<'_, '_>,
    config: &JitterConfig,
    options: &TransformOptions,
    seed_state: &mut Option<u64>,
    out: &mut String,
) {
    let mut rng = next_rng(seed_state);
    let rotation_amplitude = (config.amplitude * 0.3).clamp(0.0, 1.5);
    let opacity_amplitude = (config.stroke_width_var * 0.2).clamp(0.0, 0.08);
    let mut saw_font_family = false;
    let mut saw_transform = false;
    let mut saw_opacity = false;
    let mut anchor_x = None;
    let mut anchor_y = None;

    for attr in node.attributes() {
        let qualified_name = qualified_attr_name(node, &attr);
        match attr.name() {
            "x" | "y" => {
                if let Ok(value) = attr.value().parse::<f64>() {
                    if attr.name() == "x" {
                        anchor_x = Some(value);
                    } else {
                        anchor_y = Some(value);
                    }
                    out.push_str(&format_attr(&qualified_name, attr.value()));
                } else {
                    out.push_str(&format_attr(&qualified_name, attr.value()));
                }
            }
            "font-family" => {
                saw_font_family = true;
                let value = options
                    .font_family_override
                    .as_deref()
                    .unwrap_or(attr.value());
                out.push_str(&format_attr(&qualified_name, value));
            }
            "transform" => {
                saw_transform = true;
                let value = append_text_rotation(
                    attr.value(),
                    &mut rng,
                    rotation_amplitude,
                    anchor_x,
                    anchor_y,
                );
                out.push_str(&format_attr(&qualified_name, &value));
            }
            "opacity" => {
                saw_opacity = true;
                if let Ok(value) = attr.value().parse::<f64>() {
                    let jittered = jittered_opacity(value, &mut rng, opacity_amplitude);
                    out.push_str(&format_attr(&qualified_name, &format!("{jittered:.3}")));
                } else {
                    out.push_str(&format_attr(&qualified_name, attr.value()));
                }
            }
            _ => out.push_str(&format_attr(&qualified_name, attr.value())),
        }
    }

    if !saw_font_family {
        if let Some(font_family) = &options.font_family_override {
            out.push_str(&format_attr("font-family", font_family));
        }
    }

    if !saw_transform {
        if let Some(value) = text_rotation(&mut rng, rotation_amplitude, anchor_x, anchor_y) {
            out.push_str(&format_attr("transform", &value));
        }
    }

    if !saw_opacity {
        let jittered = jittered_opacity(1.0, &mut rng, opacity_amplitude);
        if jittered < 0.999 {
            out.push_str(&format_attr("opacity", &format!("{jittered:.3}")));
        }
    }
}

fn next_rng(seed_state: &mut Option<u64>) -> StdRng {
    let seed = *seed_state;
    if let Some(seed) = seed {
        *seed_state = Some(seed.wrapping_add(1));
        StdRng::seed_from_u64(seed)
    } else {
        StdRng::from_entropy()
    }
}

fn uniform_noise<R: Rng + ?Sized>(rng: &mut R, amplitude: f64) -> f64 {
    if amplitude == 0.0 {
        return 0.0;
    }
    (rng.gen::<f64>() - 0.5) * 2.0 * amplitude
}

fn append_text_rotation<R: Rng + ?Sized>(
    existing: &str,
    rng: &mut R,
    amplitude: f64,
    x: Option<f64>,
    y: Option<f64>,
) -> String {
    match text_rotation(rng, amplitude, x, y) {
        Some(rotation) if !existing.trim().is_empty() => {
            format!("{} {}", existing.trim(), rotation)
        }
        Some(rotation) => rotation,
        None => existing.to_string(),
    }
}

fn text_rotation<R: Rng + ?Sized>(
    rng: &mut R,
    amplitude: f64,
    x: Option<f64>,
    y: Option<f64>,
) -> Option<String> {
    let x = x?;
    let y = y?;
    let angle = uniform_noise(rng, amplitude);
    Some(format!("rotate({angle:.3} {x:.3} {y:.3})"))
}

fn jittered_opacity<R: Rng + ?Sized>(base: f64, rng: &mut R, amplitude: f64) -> f64 {
    (base + uniform_noise(rng, amplitude)).clamp(0.2, 1.0)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_with_jitter() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
          <text x="10" y="20" font-family="Arial">Hi</text>
        </svg>"#;

        let config = JitterConfig::default();
        let options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
        };

        let result = transform_svg(svg, &config, &options).unwrap();
        assert!(result.contains("<tspan"));
        assert!(result.contains("</tspan>"));
        assert!(result.contains("H"));
        assert!(result.contains("i"));
    }

    #[test]
    fn test_text_with_font_family_override() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
          <text x="10" y="20" font-family="Arial">Test</text>
        </svg>"#;

        let config = JitterConfig::default();
        let options = TransformOptions {
            seed: Some(42),
            font_family_override: Some("Georgia".to_string()),
        };

        let result = transform_svg(svg, &config, &options).unwrap();
        assert!(result.contains(r#"font-family="Georgia""#));
    }
}
