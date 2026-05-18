use aquarelle::AquarelleBleedParams;
use rand::RngCore;

use crate::svg::transform::Theme;

/// Per-theme styling surface. One implementation per `Theme` variant; transform
/// dispatch goes through `theme_style(theme)` so adding a new theme touches a
/// single new module rather than every dispatch site.
///
/// Two method families:
/// - `*_random` is used for jittered shapes — one shape, one seeded outcome.
/// - `*_static` is used for non-jittered elements (groups, rounded rects, the
///   SVG root) and must NOT advance the shared seed stream.
pub trait ThemeStyle: Sync {
    fn stroke_random(&self, original: &str, rng: &mut dyn RngCore) -> String {
        let _ = rng;
        self.stroke_static(original)
    }
    fn stroke_static(&self, original: &str) -> String {
        original.to_string()
    }

    fn fill_random(&self, original: &str, tag: &str, rng: &mut dyn RngCore) -> String {
        let _ = rng;
        self.fill_static(original, tag)
    }
    fn fill_static(&self, original: &str, tag: &str) -> String {
        let _ = tag;
        original.to_string()
    }

    fn style(&self, style: &str, tag: &str) -> String {
        let _ = tag;
        style.to_string()
    }

    fn stroke_opacity(&self, rng: &mut dyn RngCore) -> Option<f64> {
        let _ = rng;
        None
    }

    fn default_stroke_random(&self, rng: &mut dyn RngCore) -> Option<String> {
        let _ = rng;
        self.default_stroke_static()
    }
    fn default_stroke_static(&self) -> Option<String> {
        None
    }

    /// Per-shape SVG filter id applied via `filter="url(#...)"`. Themes that
    /// emit ink/dust glyph effects (chalk, marker) override this; themes that
    /// do bleed via aquarelle raster pass return `None` from
    /// [`Self::bleed_pass_params`] and skip the shape-level filter attribute.
    fn filter_id(&self) -> Option<&'static str> {
        None
    }

    /// SVG fragments to inject into the document's `<defs>` (filters,
    /// patterns, gradients, etc.). `None` = nothing extra.
    fn extra_defs(&self, seed: u64) -> Option<String> {
        let _ = seed;
        None
    }

    /// Aquarelle raster bleed pass parameters. `Some` = run
    /// `render_aquarelle_bleed_pass` on the rasterized output; the SVG-level
    /// shape filter attribute is suppressed to avoid double-bleed.
    fn bleed_pass_params(&self) -> Option<AquarelleBleedParams> {
        None
    }

    fn background(&self) -> Option<&'static str> {
        None
    }

    fn extra_replicas(&self, tag: &str) -> usize {
        let _ = tag;
        0
    }
}

struct NoneStyle;
struct BlueprintStyle;

impl ThemeStyle for NoneStyle {}

impl ThemeStyle for BlueprintStyle {
    fn stroke_static(&self, _original: &str) -> String {
        "#e8e8e8".to_string()
    }
    fn fill_static(&self, original: &str, tag: &str) -> String {
        if is_closed_shape(tag) {
            "none".to_string()
        } else {
            original.to_string()
        }
    }
    fn style(&self, style: &str, tag: &str) -> String {
        rewrite_style(style, tag, "#e8e8e8", "none")
    }
    fn default_stroke_static(&self) -> Option<String> {
        Some("#e8e8e8".to_string())
    }
    fn background(&self) -> Option<&'static str> {
        Some("#1a3a5c")
    }
}

static NONE: NoneStyle = NoneStyle;
static BLUEPRINT: BlueprintStyle = BlueprintStyle;
static SUMI: crate::svg::sumi::SumiStyle = crate::svg::sumi::SumiStyle;
static WATERCOLOR: crate::svg::watercolor::WatercolorStyle =
    crate::svg::watercolor::WatercolorStyle;
static CHALK: crate::svg::chalk::ChalkStyle = crate::svg::chalk::ChalkStyle;
static MARKER: crate::svg::marker::MarkerStyle = crate::svg::marker::MarkerStyle;
static MANGA: crate::svg::manga::MangaStyle = crate::svg::manga::MangaStyle;

pub fn theme_style(theme: Theme) -> &'static dyn ThemeStyle {
    match theme {
        Theme::None => &NONE,
        Theme::Blueprint => &BLUEPRINT,
        Theme::Sumi => &SUMI,
        Theme::Watercolor => &WATERCOLOR,
        Theme::Chalk => &CHALK,
        Theme::Marker => &MARKER,
        Theme::Manga => &MANGA,
    }
}

pub fn is_closed_shape(tag: &str) -> bool {
    matches!(tag, "rect" | "circle" | "ellipse" | "polygon")
}

/// Returns true when `d` contains a path-close command (`Z` or `z`).
/// Used to recognise paths emitted from rect/circle/ellipse/polygon after
/// usvg canonicalization (which `flatten_text_to_paths` triggers whenever
/// the input SVG contains `<text>`) as closed shapes for fill styling.
/// Without this, themes like blueprint would skip the `fill="none"` rewrite
/// on a Mermaid rect that came through the text-flatten path.
pub fn path_is_closed(d: &str) -> bool {
    d.contains('Z') || d.contains('z')
}

/// Style attribute rewriter shared by all themes that recolor strokes/fills.
pub fn rewrite_style(style: &str, tag: &str, stroke_repl: &str, closed_fill_repl: &str) -> String {
    let is_closed = is_closed_shape(tag);
    let mut result = String::new();
    for part in style.split(';') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "stroke" => result.push_str(&format!("stroke:{stroke_repl};")),
                "fill" => {
                    let v = if is_closed { closed_fill_repl } else { value };
                    result.push_str(&format!("fill:{v};"));
                }
                _ => result.push_str(&format!("{key}:{value};")),
            }
        } else {
            result.push_str(trimmed);
            result.push(';');
        }
    }
    if result.ends_with(';') {
        result.pop();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_is_closed_detects_uppercase_z() {
        assert!(path_is_closed("M 0 0 L 10 0 L 10 10 Z"));
    }

    #[test]
    fn path_is_closed_detects_lowercase_z() {
        assert!(path_is_closed("M 0 0 l 10 0 l 0 10 z"));
    }

    #[test]
    fn path_is_closed_open_path_returns_false() {
        assert!(!path_is_closed("M 0 0 L 10 0"));
    }

    #[test]
    fn path_is_closed_empty_string_returns_false() {
        assert!(!path_is_closed(""));
    }
}
