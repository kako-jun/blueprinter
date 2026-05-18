use std::path::Path;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use roxmltree::{Document, Node};

use crate::jitter::{jitter_primitive_path_with_seed, JitterConfig, JitteredPath};
use crate::svg::primitive::Primitive;
use crate::svg::text_to_path::flatten_text_to_paths;
use crate::svg::theme::{path_is_closed, theme_style};

const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";
const SVG_NS: &str = "http://www.w3.org/2000/svg";
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

/// Fallback seed used when the caller passes `seed: None`. Centralized so the
/// SVG-defs path, the aquarelle bleed pass, and the CLI's `--seed` fallback
/// stay in lockstep — changing the default in one place must not silently
/// diverge from the others.
pub const DEFAULT_SEED: u64 = 42;

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
    TextFlattenError(String),
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
            TransformError::TextFlattenError(msg) => write!(f, "Text flatten error: {msg}"),
        }
    }
}

impl std::error::Error for TransformError {}

pub fn transform_svg(
    input: &str,
    config: &JitterConfig,
    options: &TransformOptions,
    font_dir: Option<&Path>,
) -> Result<String, TransformError> {
    // text → glyph path 展開を最初に通す。後段の roxmltree pipeline は
    // <text>/<tspan> が来ない前提で動く（path jitter が glyph path に直接掛かる）。
    let flattened =
        flatten_text_to_paths(input, font_dir).map_err(TransformError::TextFlattenError)?;
    let doc =
        Document::parse(&flattened).map_err(|e| TransformError::XmlParseError(e.to_string()))?;
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
        "line" | "polyline" | "path" | "circle" | "ellipse" | "polygon" => true,
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
    let fill_tag = effective_tag_for_fill(source_tag, Some(path.d.as_str()));

    for attr in source.attributes() {
        if path.stroke_width.is_some() && attr.name() == "stroke-width" {
            continue;
        }
        if !is_geometry_attr(source_tag, attr.name()) {
            let attr_name = qualified_attr_name(source, &attr);
            let attr_value = match attr.name() {
                "stroke" => style.stroke_random(attr.value(), &mut rng),
                "fill" => style.fill_random(attr.value(), fill_tag, &mut rng),
                "style" => style.style(attr.value(), fill_tag),
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

    if let Some(filter_id) = style.filter_id() {
        out.push_str(&format!(r#" filter="url(#{filter_id})""#));
    }
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
    let mut has_stroke = false;
    let fill_tag = effective_tag_for_fill(&tag, node.attribute("d")).to_string();
    for attr in node.attributes() {
        let attr_name = qualified_attr_name(node, &attr);
        let attr_value = match attr.name() {
            "stroke" => {
                has_stroke = true;
                style.stroke_static(attr.value())
            }
            "fill" => style.fill_static(attr.value(), &fill_tag),
            "style" => style.style(attr.value(), &fill_tag),
            _ => attr.value().to_string(),
        };
        out.push_str(&format_attr(&attr_name, &attr_value));
    }
    if !has_stroke && !is_inside_non_visual_container(&node) {
        if let Some(stroke) = style.default_stroke_static() {
            out.push_str(&format_attr("stroke", &stroke));
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
            insert_svg_defs(&mut out, seed_state.unwrap_or(DEFAULT_SEED), options.theme);
        }
        if let Some(color) = style.background() {
            if let Some(bg) = theme_background(&node, color) {
                out.push_str(&bg);
            }
        }
    }
    if tag == "defs" {
        // Serialize defs children, then inject blueprinter filters
        for child in children {
            out.push_str(&serialize_node(child, config, options, seed_state));
        }
        out.push_str(&blueprinter_defs_content(
            seed_state.unwrap_or(DEFAULT_SEED),
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

/// When `flatten_text_to_paths` runs (any input SVG containing `<text>`),
/// `usvg::Tree::to_string` canonicalizes every rect/circle/ellipse/polygon
/// into a `<path>`. The downstream fill/style theme helpers use
/// `is_closed_shape(tag)` to decide whether to rewrite fills (e.g. blueprint
/// turning a Mermaid rect fill into `"none"`), and `"path"` is not in that
/// set — so a Mermaid rect that round-tripped through usvg would lose its
/// fill rewrite. We detect a closed path via its `d` close command and map
/// the tag to `"rect"` for the fill/style call only. Stroke decisions stay
/// on the original tag.
fn effective_tag_for_fill<'a>(tag: &'a str, d: Option<&str>) -> &'a str {
    if tag == "path" {
        if let Some(d) = d {
            if path_is_closed(d) {
                return "rect";
            }
        }
    }
    tag
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

fn has_defs_child(node: &Node<'_, '_>) -> bool {
    node.children()
        .any(|child| child.is_element() && child.tag_name().name() == "defs")
}

fn blueprinter_defs_content(seed: u64, theme: Theme) -> String {
    // text-grunge filter は廃止（#4: usvg で glyph path に展開し path jitter を当てる）。
    // テーマ固有の defs（chalk-dust など）だけ残す。
    theme_style(theme).extra_defs(seed).unwrap_or_default()
}

fn insert_svg_defs(out: &mut String, seed: u64, theme: Theme) {
    out.push_str(r#"<defs>"#);
    out.push_str(&blueprinter_defs_content(seed, theme));
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

fn next_rng(seed_state: &mut Option<u64>) -> StdRng {
    let seed = *seed_state;
    if let Some(seed) = seed {
        *seed_state = Some(seed.wrapping_add(1));
        StdRng::seed_from_u64(seed)
    } else {
        StdRng::from_entropy()
    }
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

    // Text-related tests removed in #4: <text> is now expanded to glyph paths
    // upstream via flatten_text_to_paths, so transform_svg never sees text
    // elements. Glyph rendering and font resolution are usvg's responsibility,
    // and there's no longer a tspan/font-family-override layer to test here.

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

        let result = transform_svg(svg, &config, &options, None).unwrap();
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

        let result = transform_svg(svg, &config, &options, None).unwrap();
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

        let result = transform_svg(svg, &config, &options, None).unwrap();
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

        let result = transform_svg(svg, &config, &options, None).unwrap();
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

        let result = transform_svg(svg, &config, &options, None).unwrap();
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

        let result = transform_svg(svg, &config, &options, None).unwrap();
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

        let result = transform_svg(svg, &config, &options, None).unwrap();
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

        let result = transform_svg(svg, &config, &options, None).unwrap();
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
    fn transform_svg_treats_closed_path_as_fill_target_for_blueprint() {
        // Mermaid 風: <text> を含むため flatten_text_to_paths が usvg canonicalize
        // を走らせ、rect は <path d="... Z"> に化ける。それでも blueprint の
        // closed-shape fill 処理 (fill="none") が効くことを保証する。
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100" viewBox="0 0 200 100">
          <rect x="10" y="10" width="80" height="40" fill="red" stroke="black"/>
          <text x="20" y="35" font-size="12">node</text>
        </svg>"#;

        let config = JitterConfig {
            amplitude: 0.0,
            ..JitterConfig::default()
        };
        let options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme: Theme::Blueprint,
        };

        let result = transform_svg(svg, &config, &options, None).unwrap();
        // closed path として認識され、blueprint の fill="none" が掛かっていること
        assert!(
            result.contains(r##"fill="none""##),
            "closed path from usvg canonicalize must be treated as fill target for blueprint; got: {result}"
        );
        assert!(
            !result.contains(r##"fill="red""##),
            "original red fill must be rewritten by blueprint theme; got: {result}"
        );
    }

    #[test]
    fn transform_svg_keeps_open_path_fill_for_blueprint() {
        // 開いた path（Z なし）には fill="none" を強制しない。
        // 元の fill 属性が保持されることを確認する。
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
          <path d="M 10 10 L 90 10 L 90 50" fill="red" stroke="black"/>
          <text x="20" y="80" font-size="12">node</text>
        </svg>"#;

        let config = JitterConfig {
            amplitude: 0.0,
            ..JitterConfig::default()
        };
        let options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme: Theme::Blueprint,
        };

        let result = transform_svg(svg, &config, &options, None).unwrap();
        // open path は fill="none" 化されない。usvg は色名を hex (#ff0000) に
        // 正規化することがあるため、両方を許容して closed-shape 用の "none"
        // 強制が掛かっていない（元の赤系 fill が残っている）ことを確認する。
        // glyph path などの fill="none" は SVG 内に別途現れるため、特定の
        // open path 要素に対する fill チェックでなく、赤系 fill の残存を見る。
        let kept_red = result.contains(r##"fill="red""##)
            || result.contains(r##"fill="#ff0000""##)
            || result.contains(r##"fill="#FF0000""##);
        assert!(
            kept_red,
            "open path fill must not be force-rewritten to none by blueprint; got: {result}"
        );
    }

    #[test]
    fn test_sumi_and_watercolor_skip_shape_filter_attribute() {
        // sumi/watercolor use aquarelle raster bleed pass, so they must not
        // emit per-shape SVG filter="url(#...)" attributes (which would
        // double-bleed against the raster pass).
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20"/>
        </svg>"#;

        let config = JitterConfig::default();

        let sumi_options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme: Theme::Sumi,
        };
        let sumi_result = transform_svg(svg, &config, &sumi_options, None).unwrap();
        assert!(!sumi_result.contains("sumi-ink-bleed"));
        assert!(!sumi_result.contains("filter=\"url(#"));

        let watercolor_options = TransformOptions {
            seed: Some(42),
            font_family_override: None,
            theme: Theme::Watercolor,
        };
        let watercolor_result = transform_svg(svg, &config, &watercolor_options, None).unwrap();
        assert!(!watercolor_result.contains("watercolor-bleed"));
        assert!(!watercolor_result.contains("filter=\"url(#"));
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

    let result = transform_svg(svg, &config, &options, None).unwrap();
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

    let result = transform_svg(svg, &config, &options, None).unwrap();
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

    let result = transform_svg(svg, &config, &options, None).unwrap();
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

    let result = transform_svg(svg, &config, &options, None).unwrap();
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

    let result = transform_svg(svg, &config, &options, None).unwrap();
    assert!(result.contains(r#"fill="none""#));
}

#[test]
fn test_theme_filter_ids_are_applied() {
    // Only themes that override `filter_id` (chalk, marker) should emit
    // per-shape filter attributes. Blueprint, sumi, and watercolor either use
    // no per-shape filter (blueprint) or rely on the aquarelle raster bleed
    // pass instead (sumi, watercolor).
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <circle cx="50" cy="50" r="20" stroke="black"/>
        </svg>"#;

    let config = JitterConfig::default();

    let chalk_options = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Chalk,
    };
    let chalk_result = transform_svg(svg, &config, &chalk_options, None).unwrap();
    assert!(chalk_result.contains(r##"filter="url(#chalk-dust)""##));

    let marker_options = TransformOptions {
        seed: Some(42),
        font_family_override: None,
        theme: Theme::Marker,
    };
    let marker_result = transform_svg(svg, &config, &marker_options, None).unwrap();
    assert!(marker_result.contains(r##"filter="url(#marker-glow)""##));
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

    let result = transform_svg(svg, &config, &options, None).unwrap();
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

    let result = transform_svg(svg, &config, &options, None).unwrap();
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

    let result1 = transform_svg(svg, &config, &options, None).unwrap();
    let result2 = transform_svg(
        svg,
        &config,
        &TransformOptions {
            seed: Some(43),
            font_family_override: None,
            theme: Theme::Watercolor,
        },
        None,
    )
    .unwrap();

    // Different seeds should produce different color combinations
    // (though we can't guarantee they'll be completely different due to randomness,
    // with 3 circles and 7 palette colors, different seeds have high probability of variation)
    assert_ne!(result1, result2);
}
