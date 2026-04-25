/// Manga theme implementation — black ink lines on white paper with screentone fills.
use rand::{Rng, RngCore};

use crate::svg::theme::{rewrite_style, ThemeStyle};

pub const MANGA_BACKGROUND: &str = "#ffffff";
pub const MANGA_INK: &str = "#000000";

/// IDs of the screentone <pattern>s injected via [`manga_pattern_defs`].
/// Order matters — kept in sync with the array used by [`pick_pattern`].
const MANGA_PATTERNS: &[&str] = &["manga-dots-light", "manga-dots-dark", "manga-lines-diag"];

const MANGA_DEFAULT_PATTERN: &str = "manga-dots-light";

pub struct MangaStyle;

impl ThemeStyle for MangaStyle {
    fn stroke_random(&self, _original: &str, _rng: &mut dyn RngCore) -> String {
        MANGA_INK.to_string()
    }
    fn stroke_static(&self, _original: &str) -> String {
        MANGA_INK.to_string()
    }
    fn fill_random(&self, original: &str, tag: &str, rng: &mut dyn RngCore) -> String {
        if matches!(tag, "rect" | "circle" | "ellipse" | "polygon") {
            pattern_url(pick_pattern(rng))
        } else {
            original.to_string()
        }
    }
    fn fill_static(&self, original: &str, tag: &str) -> String {
        if matches!(tag, "rect" | "circle" | "ellipse" | "polygon") {
            pattern_url(MANGA_DEFAULT_PATTERN)
        } else {
            original.to_string()
        }
    }
    fn style(&self, style: &str, tag: &str) -> String {
        let pattern_fill = pattern_url(MANGA_DEFAULT_PATTERN);
        rewrite_style(style, tag, MANGA_INK, &pattern_fill)
    }
    fn default_stroke_static(&self) -> Option<String> {
        Some(MANGA_INK.to_string())
    }
    fn extra_defs(&self, _seed: u64) -> Option<String> {
        Some(manga_pattern_defs())
    }
    fn background(&self) -> Option<&'static str> {
        Some(MANGA_BACKGROUND)
    }
}

fn pick_pattern(rng: &mut dyn RngCore) -> &'static str {
    MANGA_PATTERNS[rng.gen_range(0..MANGA_PATTERNS.len())]
}

fn pattern_url(id: &str) -> String {
    format!("url(#{id})")
}

/// Three screentone patterns: sparse dots, dense dots, and diagonal lines.
/// Together they cover most "value" needs (light shading, dark shading, motion).
fn manga_pattern_defs() -> String {
    let dots_light = r##"<pattern id="manga-dots-light" x="0" y="0" width="6" height="6" patternUnits="userSpaceOnUse"><circle cx="3" cy="3" r="0.7" fill="#000"/></pattern>"##;
    let dots_dark = r##"<pattern id="manga-dots-dark" x="0" y="0" width="4" height="4" patternUnits="userSpaceOnUse"><circle cx="2" cy="2" r="1.0" fill="#000"/></pattern>"##;
    let lines_diag = r##"<pattern id="manga-lines-diag" x="0" y="0" width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><line x1="0" y1="0" x2="0" y2="6" stroke="#000" stroke-width="0.8"/></pattern>"##;
    format!("{dots_light}{dots_dark}{lines_diag}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn manga_pick_pattern_returns_known_id() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..16 {
            let id = pick_pattern(&mut rng);
            assert!(MANGA_PATTERNS.contains(&id));
        }
    }

    #[test]
    fn manga_pattern_url_format() {
        assert_eq!(pattern_url("manga-dots-light"), "url(#manga-dots-light)");
    }

    #[test]
    fn manga_extra_defs_contains_all_patterns() {
        let style = MangaStyle;
        let defs = style.extra_defs(0).unwrap();
        for id in MANGA_PATTERNS {
            assert!(
                defs.contains(&format!(r#"id="{id}""#)),
                "missing pattern {id}"
            );
        }
        assert!(defs.contains("<pattern"));
        assert!(defs.contains("patternUnits=\"userSpaceOnUse\""));
    }

    #[test]
    fn manga_closed_shape_fill_is_pattern_url() {
        let style = MangaStyle;
        let mut rng = StdRng::seed_from_u64(42);
        let fill = style.fill_random("red", "rect", &mut rng);
        assert!(fill.starts_with("url(#manga-"));
        assert!(fill.ends_with(')'));
    }

    #[test]
    fn manga_open_shape_fill_preserved() {
        let style = MangaStyle;
        let mut rng = StdRng::seed_from_u64(42);
        assert_eq!(style.fill_random("red", "line", &mut rng), "red");
    }

    #[test]
    fn manga_stroke_is_pure_black() {
        let style = MangaStyle;
        let mut rng = StdRng::seed_from_u64(42);
        assert_eq!(style.stroke_random("anything", &mut rng), "#000000");
        assert_eq!(style.stroke_static("anything"), "#000000");
    }
}
