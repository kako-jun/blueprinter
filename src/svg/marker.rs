/// Marker theme implementation — bold neon strokes on a dark sketchbook.
use rand::{Rng, RngCore};

use crate::svg::theme::{rewrite_style, ThemeStyle};

const MARKER_PALETTE: &[&str] = &[
    "#ff3ec9", // hot pink
    "#00e5ff", // cyan
    "#c6ff00", // lime
    "#ff8400", // orange
    "#ffeb00", // yellow
    "#ff00d4", // magenta
];

pub const MARKER_BACKGROUND: &str = "#1a1a2e";
pub const MARKER_DEFAULT_STROKE: &str = "#ff3ec9";
const MARKER_FILL_ALPHA: &str = "33"; // ~20% — highlighter-style translucency

pub struct MarkerStyle;

impl ThemeStyle for MarkerStyle {
    fn stroke_random(&self, _original: &str, rng: &mut dyn RngCore) -> String {
        pick_palette(rng).to_string()
    }
    fn stroke_static(&self, _original: &str) -> String {
        MARKER_DEFAULT_STROKE.to_string()
    }
    fn fill_random(&self, original: &str, tag: &str, rng: &mut dyn RngCore) -> String {
        if matches!(tag, "rect" | "circle" | "ellipse" | "polygon") {
            format!("{}{}", pick_palette(rng), MARKER_FILL_ALPHA)
        } else {
            original.to_string()
        }
    }
    fn fill_static(&self, original: &str, tag: &str) -> String {
        if matches!(tag, "rect" | "circle" | "ellipse" | "polygon") {
            format!("{}{}", MARKER_DEFAULT_STROKE, MARKER_FILL_ALPHA)
        } else {
            original.to_string()
        }
    }
    fn style(&self, style: &str, tag: &str) -> String {
        let closed_fill = format!("{}{}", MARKER_DEFAULT_STROKE, MARKER_FILL_ALPHA);
        rewrite_style(style, tag, MARKER_DEFAULT_STROKE, &closed_fill)
    }
    fn stroke_opacity(&self, rng: &mut dyn RngCore) -> Option<f64> {
        Some(0.85 + rng.gen::<f64>() * 0.15)
    }
    fn default_stroke_random(&self, rng: &mut dyn RngCore) -> Option<String> {
        Some(pick_palette(rng).to_string())
    }
    fn default_stroke_static(&self) -> Option<String> {
        Some(MARKER_DEFAULT_STROKE.to_string())
    }
    fn filter_id(&self) -> &'static str {
        "marker-glow"
    }
    fn filter_defs(&self, _seed: u64) -> Option<String> {
        Some(marker_filter_defs())
    }
    fn background(&self) -> Option<&'static str> {
        Some(MARKER_BACKGROUND)
    }
}

fn pick_palette(rng: &mut dyn RngCore) -> &'static str {
    MARKER_PALETTE[rng.gen_range(0..MARKER_PALETTE.len())]
}

/// Subtle halo around the source: a soft blur layered behind the original
/// pixels mimics marker pigment glowing slightly under dim light.
fn marker_filter_defs() -> String {
    r#"<filter id="marker-glow" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur in="SourceGraphic" stdDeviation="1.6" result="halo"/><feMerge><feMergeNode in="halo"/><feMergeNode in="SourceGraphic"/></feMerge></filter>"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn marker_stroke_returns_palette_color() {
        let mut rng = StdRng::seed_from_u64(42);
        let stroke = pick_palette(&mut rng);
        assert!(MARKER_PALETTE.contains(&stroke));
    }

    #[test]
    fn marker_palette_all_neon_hex() {
        for color in MARKER_PALETTE {
            assert!(color.starts_with('#'));
            assert_eq!(color.len(), 7);
        }
    }

    #[test]
    fn marker_filter_defs_contains_required_elements() {
        let defs = marker_filter_defs();
        assert!(defs.contains("marker-glow"));
        assert!(defs.contains("feGaussianBlur"));
        assert!(defs.contains("feMerge"));
    }

    #[test]
    fn marker_stroke_opacity_in_range() {
        let style = MarkerStyle;
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..32 {
            let opacity = style.stroke_opacity(&mut rng).unwrap();
            assert!(opacity >= 0.85);
            assert!(opacity <= 1.0);
        }
    }

    #[test]
    fn marker_closed_fill_has_alpha() {
        let style = MarkerStyle;
        let mut rng = StdRng::seed_from_u64(42);
        let fill = style.fill_random("red", "rect", &mut rng);
        assert_eq!(fill.len(), 9, "fill should be #RRGGBBAA");
        assert!(fill.ends_with(MARKER_FILL_ALPHA));
    }

    #[test]
    fn marker_open_shape_fill_preserved() {
        let style = MarkerStyle;
        let mut rng = StdRng::seed_from_u64(42);
        assert_eq!(style.fill_random("red", "line", &mut rng), "red");
    }
}
