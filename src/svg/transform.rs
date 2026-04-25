use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use roxmltree::{Document, Node};

use crate::jitter::{jitter_primitive_path_with_seed, JitterConfig, JitteredPath};
use crate::svg::primitive::Primitive;
use crate::svg::theme::theme_style;

const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";
const SVG_NS: &str = "http://www.w3.org/2000/svg";
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    #[default]
    None,
    Blueprint,
    Sumi,
    Watercolor,
    Chalk,
    Marker,
    Manga,
}

#[derive(Debug, PartialEq)]
pub enum TransformError {
    XmlParseError(String),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TransformOptions {
    pub seed: Option<u64>,
    pub font_family_override: Option<String>,
    pub theme: Theme,
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
            return serialize_jittered_path(node, &path, options, seed_state);
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
        "line" | "polyline" | "path" | "text" | "circle" | "ellipse" | "polygon" => true,
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

fn serialize_jittered_path(
    source: Node<'_, '_>,
    path: &JitteredPath,
    options: &TransformOptions,
    seed_state: &mut Option<u64>,
) -> String {
    let tag = qualified_replacement_path_name(source);
    let source_tag = source.tag_name().name();
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

    let mut rng = next_rng(seed_state);
    let style = theme_style(options.theme);

    for attr in source.attributes() {
        if path.stroke_width.is_some() && attr.name() == "stroke-width" {
            continue;
        }
        if !is_geometry_attr(source_tag, attr.name()) {
            let attr_name = qualified_attr_name(source, &attr);
            let attr_value = match attr.name() {
                "stroke" => style.stroke_random(attr.value(), &mut rng),
                "fill" => style.fill_random(attr.value(), source_tag, &mut rng),
                "style" => style.style(attr.value(), source_tag),
                _ => attr.value().to_string(),
            };
            out.push_str(&format_attr(&attr_name, &attr_value));
        }
    }
    if let Some(stroke_width) = path.stroke_width {
        out.push_str(&format_attr("stroke-width", &format!("{stroke_width:.3}")));
    }

    if let Some(opacity) = style.stroke_opacity(&mut rng) {
        out.push_str(&format_attr("stroke-opacity", &format!("{opacity:.3}")));
    }

    let has_stroke = source.attribute("stroke").is_some() || source.attribute("style").is_some();
    if !has_stroke {
        if let Some(stroke) = style.default_stroke_random(&mut rng) {
            out.push_str(&format_attr("stroke", &stroke));
        }
    }

    out.push_str(&format!(r#" filter="url(#{})""#, style.filter_id()));
    out.push_str(" />");

    let extra = style.extra_replicas(source_tag);
    if extra > 0 {
        let replicas = emit_stroke_replicas(&out, extra, seed_state);
        out.push_str(&replicas);
    }

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
    let style = theme_style(options.theme);
    if should_jitter_text(&node) {
        serialize_text_attrs(node, config, options, seed_state, &mut out);
    } else {
        let mut has_stroke = false;
        for attr in node.attributes() {
            let attr_name = qualified_attr_name(node, &attr);
            let attr_value = match attr.name() {
                "stroke" => {
                    has_stroke = true;
                    style.stroke_static(attr.value())
                }
                "fill" => style.fill_static(attr.value(), &tag),
                "style" => style.style(attr.value(), &tag),
                _ => attr.value().to_string(),
            };
            out.push_str(&format_attr(&attr_name, &attr_value));
        }
        if !has_stroke && !is_inside_non_visual_container(&node) {
            if let Some(stroke) = style.default_stroke_static() {
                out.push_str(&format_attr("stroke", &stroke));
            }
        }
    }

    let children: Vec<_> = node.children().collect();
    if children.is_empty() {
        out.push_str(" />");
        return out;
    }

    out.push('>');
    if tag == "svg" {
        if !has_defs_child(&node) {
            insert_svg_defs(&mut out, seed_state.unwrap_or(42), options.theme);
        }
        if let Some(color) = style.background() {
            if let Some(bg) = theme_background(&node, color) {
                out.push_str(&bg);
            }
        }
    }
    if tag == "text" {
        serialize_text_content(node, config, options, seed_state, &mut out);
    } else if tag == "defs" {
        // Serialize defs children, then inject blueprinter filters
        for child in children {
            out.push_str(&serialize_node(child, config, options, seed_state));
        }
        out.push_str(&bp_filter_defs_content(
            seed_state.unwrap_or(42),
            options.theme,
        ));
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
            | ("polygon", "points")
            | ("circle", "cx")
            | ("circle", "cy")
            | ("circle", "r")
            | ("ellipse", "cx")
            | ("ellipse", "cy")
            | ("ellipse", "rx")
            | ("ellipse", "ry")
            | ("path", "d")
    )
}

fn should_jitter_text(node: &Node<'_, '_>) -> bool {
    if is_inside_non_visual_container(node) || !is_svg_element(node) {
        return false;
    }

    matches!(node.tag_name().name(), "text" | "tspan")
}

fn has_defs_child(node: &Node<'_, '_>) -> bool {
    node.children()
        .any(|child| child.is_element() && child.tag_name().name() == "defs")
}

fn bp_filter_defs_content(seed: u64, theme: Theme) -> String {
    let text_grunge = r#"<filter id="text-grunge" x="-20%" y="-20%" width="140%" height="140%"><feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="4" result="noise" seed="{seed}"/><feDisplacementMap in="SourceGraphic" in2="noise" scale="0.8" xChannelSelector="R" yChannelSelector="G"/></filter>"#
        .replace("{seed}", &seed.to_string());

    let subtle_bleed = r#"<filter id="subtle-bleed" x="-25%" y="-25%" width="150%" height="150%"><feGaussianBlur in="SourceGraphic" stdDeviation="3.0" result="blurred1"/><feOffset in="blurred1" dx="1.0" dy="1.0" result="offset1"/><feGaussianBlur in="offset1" stdDeviation="1.5" result="blurred2"/><feComponentTransfer in="blurred2" result="faded"><feFuncA type="linear" slope="0.4"/></feComponentTransfer><feComposite in="faded" in2="SourceGraphic" operator="darken"/></filter>"#;

    let mut out = format!("{text_grunge}{subtle_bleed}");
    if let Some(extra) = theme_style(theme).extra_defs(seed) {
        out.push_str(&extra);
    }
    out
}

fn insert_svg_defs(out: &mut String, seed: u64, theme: Theme) {
    out.push_str(r#"<defs>"#);
    out.push_str(&bp_filter_defs_content(seed, theme));
    out.push_str(r#"</defs>"#);
}

fn remove_stroke_opacity(s: &str) -> String {
    // Remove stroke-opacity="0.###" pattern
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == ' ' && chars.peek() == Some(&'s') {
            // Possible start of " stroke-opacity"
            let rest: String = chars.clone().collect();
            if rest.starts_with("stroke-opacity=\"") {
                // Skip until closing quote
                let prefix = "stroke-opacity=\"";
                for _ in 0..prefix.len() {
                    chars.next();
                }
                let mut found_close = false;
                for c in chars.by_ref() {
                    if c == '"' {
                        found_close = true;
                        break;
                    }
                }
                if !found_close {
                    // Malformed, just append the space back
                    result.push(' ');
                    result.push_str("stroke-opacity=\"");
                }
                // Skip the attribute, don't append
                continue;
            }
        }
        result.push(ch);
    }
    result
}

fn emit_stroke_replicas(main_element: &str, extra: usize, seed_state: &mut Option<u64>) -> String {
    let mut rng = next_rng(seed_state);
    let mut result = String::new();

    for i in 1..=extra {
        let dx = (rng.gen::<f32>() - 0.5) * 1.5;
        let dy = (rng.gen::<f32>() - 0.5) * 1.5;
        let opacity = (0.5 - (i as f64 * 0.15)).max(0.1);

        let replica = if main_element.contains(" />") {
            let base = main_element.trim_end_matches(" />");
            let base_no_opacity = remove_stroke_opacity(base);
            format!(
                r#"{} transform="translate({:.2}, {:.2})" stroke-opacity="{:.2}" />"#,
                base_no_opacity, dx, dy, opacity
            )
        } else {
            main_element.to_string()
        };

        result.push_str(&replica);
    }

    result
}

fn theme_background(svg_node: &Node<'_, '_>, color: &str) -> Option<String> {
    let viewbox = svg_node.attribute("viewBox").and_then(parse_viewbox);
    let width = svg_node
        .attribute("width")
        .and_then(|w| w.parse::<f64>().ok())
        .or_else(|| viewbox.map(|(_, _, w, _)| w))
        .unwrap_or(100.0);
    let height = svg_node
        .attribute("height")
        .and_then(|h| h.parse::<f64>().ok())
        .or_else(|| viewbox.map(|(_, _, _, h)| h))
        .unwrap_or(100.0);
    let (x, y) = viewbox.map(|(x, y, _, _)| (x, y)).unwrap_or((0.0, 0.0));

    Some(format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
        x, y, width, height, color
    ))
}

fn parse_viewbox(value: &str) -> Option<(f64, f64, f64, f64)> {
    let parts: Vec<f64> = value
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();
    if parts.len() == 4 {
        Some((parts[0], parts[1], parts[2], parts[3]))
    } else {
        None
    }
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
                out.push_str(&format!(
                    r#" transform="rotate({:.2} {:.3} {:.3})""#,
                    angle, x, y
                ));
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
            theme: Theme::None,
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
            ..Default::default()
        };

        let result = transform_svg(svg, &config, &options).unwrap();
        assert!(result.contains(r#"font-family="Georgia""#));
    }

    #[test]
    fn test_blueprint_theme_stroke_color() {
        // Use a non-jittered element (text) so we can test the stroke color transformation
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <g stroke="black"><circle cx="50" cy="50" r="20"/></g>
        </svg>"#;

        let config = JitterConfig::default();
        let options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme: Theme::Blueprint,
        };

        let result = transform_svg(svg, &config, &options).unwrap();
        assert!(result.contains(r##"stroke="#e8e8e8""##));
        assert!(!result.contains(r##"stroke="black""##));
    }

    #[test]
    fn test_blueprint_theme_fill_closed_shapes() {
        // circle and ellipse are now jittered to paths, so they will have fill="none" in the path output
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20" fill="blue"/>
          <ellipse cx="70" cy="70" rx="20" ry="10" fill="green"/>
        </svg>"#;

        let config = JitterConfig::default();
        let options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme: Theme::Blueprint,
        };

        let result = transform_svg(svg, &config, &options).unwrap();
        // circle and ellipse are converted to paths with jitter, fill should be "none" in blueprint theme
        assert!(result.contains(r##"fill="none""##));
    }

    #[test]
    fn test_blueprint_theme_line_fill_unchanged() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <line x1="0" y1="0" x2="100" y2="100" stroke="black" fill="red"/>
        </svg>"#;

        let config = JitterConfig::default();
        let options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme: Theme::Blueprint,
        };

        let result = transform_svg(svg, &config, &options).unwrap();
        // line elements should not have fill changed to "none" (they don't match the closed shapes)
        assert!(result.contains(r##"fill="red""##));
    }

    #[test]
    fn test_no_theme_preserves_colors() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <rect x="10" y="10" width="50" height="50" fill="red" stroke="blue"/>
        </svg>"#;

        let config = JitterConfig::default();
        let options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme: Theme::None,
        };

        let result = transform_svg(svg, &config, &options).unwrap();
        assert!(result.contains(r##"fill="red""##));
        assert!(result.contains(r##"stroke="blue""##));
    }

    #[test]
    fn test_blueprint_theme_style_stroke_fill() {
        // circle is now jittered to path, so style attribute won't be present - path attributes will be set directly
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20" style="stroke:red;fill:blue"/>
        </svg>"#;

        let config = JitterConfig::default();
        let options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme: Theme::Blueprint,
        };

        let result = transform_svg(svg, &config, &options).unwrap();
        // circle is converted to path with jitter, so output will have fill and stroke set directly in path element
        assert!(result.contains(r##"stroke="##));
        assert!(result.contains(r##"fill="##));
    }

    #[test]
    fn test_blueprint_theme_default_stroke_added() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20" fill="blue"/>
        </svg>"#;

        let config = JitterConfig::default();
        let options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme: Theme::Blueprint,
        };

        let result = transform_svg(svg, &config, &options).unwrap();
        // circle should have default stroke added in blueprint theme
        assert!(result.contains(r##"stroke="#e8e8e8""##));
    }

    #[test]
    fn test_sumi_theme_produces_grayscale() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20" stroke="blue"/>
        </svg>"#;

        let config = JitterConfig::default();
        let options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme: Theme::Sumi,
        };

        let result = transform_svg(svg, &config, &options).unwrap();
        // sumi theme should use grayscale color
        assert!(result.contains("rgba(50, 50, 50, 0.8)"));
    }

    #[test]
    fn test_watercolor_theme_produces_colors() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20" stroke="blue"/>
        </svg>"#;

        let config = JitterConfig::default();
        let options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme: Theme::Watercolor,
        };

        let result = transform_svg(svg, &config, &options).unwrap();
        // watercolor theme should use a pastel color from the palette
        let palette_colors = [
            "#FFB3BA", "#FFDFBA", "#FFFFBA", "#BAFFC9", "#BAE1FF", "#E0BBE4", "#FFC7F5",
        ];
        let has_palette_color = palette_colors.iter().any(|color| result.contains(color));
        assert!(
            has_palette_color,
            "Result should contain at least one watercolor palette color"
        );
    }

    #[test]
    fn test_both_themes_add_blur_filter() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20"/>
        </svg>"#;

        let config = JitterConfig::default();

        // Test sumi theme
        let sumi_options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme: Theme::Sumi,
        };
        let sumi_result = transform_svg(svg, &config, &sumi_options).unwrap();
        assert!(sumi_result.contains("feGaussianBlur"));
        assert!(sumi_result.contains("sumi-ink-bleed"));

        // Test watercolor theme
        let watercolor_options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme: Theme::Watercolor,
        };
        let watercolor_result = transform_svg(svg, &config, &watercolor_options).unwrap();
        assert!(watercolor_result.contains("feGaussianBlur"));
        assert!(watercolor_result.contains("watercolor-bleed"));
    }
}

#[test]
fn test_chalk_theme_uses_palette_color_for_stroke() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20" stroke="black"/>
        </svg>"#;

    let config = JitterConfig::default();
    let options = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Chalk,
    };

    let result = transform_svg(svg, &config, &options).unwrap();
    let palette_colors = ["#f5f5f5", "#fff5b8", "#ffd0d0", "#cfe7ff", "#d8ffd0"];
    let has_palette = palette_colors.iter().any(|c| result.contains(c));
    assert!(has_palette, "chalk theme should pick from palette");
    assert!(!result.contains(r#"stroke="black""#));
}

#[test]
fn test_chalk_theme_emits_blackboard_background() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20"/>
        </svg>"#;

    let config = JitterConfig::default();
    let options = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Chalk,
    };

    let result = transform_svg(svg, &config, &options).unwrap();
    assert!(
        result.contains(r##"fill="#1f2a25""##),
        "chalk theme should emit chalkboard background"
    );
}

#[test]
fn test_chalk_theme_default_stroke_added() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20" fill="blue"/>
        </svg>"#;

    let config = JitterConfig::default();
    let options = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Chalk,
    };

    let result = transform_svg(svg, &config, &options).unwrap();
    assert!(
        result.contains("stroke="),
        "chalk theme should add a default stroke when missing"
    );
}

#[test]
fn test_chalk_theme_filter_id() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20" stroke="black"/>
        </svg>"#;

    let config = JitterConfig::default();
    let options = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Chalk,
    };

    let result = transform_svg(svg, &config, &options).unwrap();
    assert!(result.contains(r##"filter="url(#chalk-dust)""##));
    assert!(result.contains(r#"id="chalk-dust""#));
}

#[test]
fn test_chalk_theme_closed_shape_fill_none() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <rect x="10" y="10" width="50" height="50" fill="red" stroke="black"/>
        </svg>"#;

    let config = JitterConfig::default();
    let options = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Chalk,
    };

    let result = transform_svg(svg, &config, &options).unwrap();
    assert!(result.contains(r#"fill="none""#));
}

#[test]
fn test_theme_filter_ids_are_applied() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20" stroke="black"/>
        </svg>"#;

    let config = JitterConfig::default();

    // Test Blueprint theme
    let blueprint_options = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Blueprint,
    };
    let blueprint_result = transform_svg(svg, &config, &blueprint_options).unwrap();
    assert!(blueprint_result.contains(r##"filter="url(#subtle-bleed)""##));

    // Test Sumi theme
    let sumi_options = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Sumi,
    };
    let sumi_result = transform_svg(svg, &config, &sumi_options).unwrap();
    assert!(sumi_result.contains(r##"filter="url(#sumi-ink-bleed)""##));

    // Test Watercolor theme
    let watercolor_options = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Watercolor,
    };
    let watercolor_result = transform_svg(svg, &config, &watercolor_options).unwrap();
    assert!(watercolor_result.contains(r##"filter="url(#watercolor-bleed)""##));
}

#[test]
fn test_watercolor_opacity_randomization() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20" stroke="black"/>
        </svg>"#;

    let config = JitterConfig::default();
    let options = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Watercolor,
    };

    let result = transform_svg(svg, &config, &options).unwrap();
    // Check that stroke-opacity is set for watercolor theme
    assert!(
        result.contains(r##"stroke-opacity=""##),
        "Watercolor should have randomized stroke-opacity"
    );
}

#[test]
fn test_sumi_opacity_randomization() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20" stroke="black"/>
        </svg>"#;

    let config = JitterConfig::default();
    let options = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Sumi,
    };

    let result = transform_svg(svg, &config, &options).unwrap();
    // Check that stroke-opacity is set for sumi theme
    assert!(
        result.contains(r##"stroke-opacity=""##),
        "Sumi should have randomized stroke-opacity"
    );
}

#[test]
fn test_watercolor_color_randomization_varies_with_seed() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20" stroke="black"/>
          <circle cx="30" cy="30" r="15" stroke="black"/>
          <circle cx="70" cy="70" r="18" stroke="black"/>
        </svg>"#;

    let config = JitterConfig::default();
    let options = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Watercolor,
    };

    let result1 = transform_svg(svg, &config, &options).unwrap();
    let result2 = transform_svg(
        svg,
        &config,
        &TransformOptions {
            seed: Some(43),
            font_family_override: None,
            theme: Theme::Watercolor,
        },
    )
    .unwrap();

    // Different seeds should produce different color combinations
    // (though we can't guarantee they'll be completely different due to randomness,
    // with 3 circles and 7 palette colors, different seeds have high probability of variation)
    assert_ne!(result1, result2);
}
