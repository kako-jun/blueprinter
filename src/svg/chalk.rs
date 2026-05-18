/// Chalk theme implementation — chalkboard with dusty chalk strokes.
use rand::{Rng, RngCore};

use crate::svg::theme::{rewrite_style, ThemeStyle};

pub struct ChalkStyle;

impl ThemeStyle for ChalkStyle {
    fn stroke_random(&self, _original: &str, rng: &mut dyn RngCore) -> String {
        apply_chalk_stroke(rng).to_string()
    }
    fn stroke_static(&self, _original: &str) -> String {
        CHALK_DEFAULT_STROKE.to_string()
    }
    fn fill_static(&self, original: &str, tag: &str) -> String {
        apply_chalk_fill(original, tag)
    }
    fn style(&self, style: &str, tag: &str) -> String {
        rewrite_style(style, tag, CHALK_DEFAULT_STROKE, "none")
    }
    fn stroke_opacity(&self, rng: &mut dyn RngCore) -> Option<f64> {
        Some(chalk_random_opacity(rng))
    }
    fn default_stroke_random(&self, rng: &mut dyn RngCore) -> Option<String> {
        Some(apply_chalk_stroke(rng).to_string())
    }
    fn default_stroke_static(&self) -> Option<String> {
        Some(CHALK_DEFAULT_STROKE.to_string())
    }
    fn filter_id(&self) -> Option<&'static str> {
        Some("chalk-dust")
    }
    fn extra_defs(&self, seed: u64) -> Option<String> {
        Some(chalk_filter_defs(seed))
    }
    fn background(&self) -> Option<&'static str> {
        Some(CHALK_BACKGROUND)
    }
    fn extra_replicas(&self, tag: &str) -> usize {
        if matches!(
            tag,
            "path" | "text" | "rect" | "circle" | "ellipse" | "line" | "polyline"
        ) {
            1
        } else {
            0
        }
    }
}

/// White is repeated to bias selection — most chalks are plain white,
/// with the occasional pale-color stick.
const CHALK_PALETTE: &[&str] = &[
    "#f5f5f5", "#f5f5f5", "#f5f5f5", "#f5f5f5", "#f5f5f5", "#f5f5f5", "#fff5b8", "#ffd0d0",
    "#cfe7ff", "#d8ffd0",
];

pub const CHALK_BACKGROUND: &str = "#1f2a25";
pub const CHALK_DEFAULT_STROKE: &str = "#f5f5f5";

pub fn apply_chalk_stroke<R: Rng + ?Sized>(rng: &mut R) -> &'static str {
    CHALK_PALETTE[rng.gen_range(0..CHALK_PALETTE.len())]
}

pub fn apply_chalk_fill(fill: &str, tag: &str) -> String {
    if matches!(tag, "rect" | "circle" | "ellipse" | "polygon") {
        "none".to_string()
    } else {
        fill.to_string()
    }
}

pub fn chalk_random_opacity<R: Rng + ?Sized>(rng: &mut R) -> f64 {
    let base = 0.7;
    let variance = rng.gen::<f64>() * 0.25;
    (base + variance).min(0.95)
}

/// Filter that gives chalk strokes a dusty, broken-up look:
/// turbulence-driven displacement breaks the line, a small blur softens edges.
pub fn chalk_filter_defs(seed: u64) -> String {
    format!(
        r#"<filter id="chalk-dust" x="-15%" y="-15%" width="130%" height="130%"><feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="2" seed="{seed}" result="noise"/><feDisplacementMap in="SourceGraphic" in2="noise" scale="1.4" xChannelSelector="R" yChannelSelector="G" result="displaced"/><feGaussianBlur in="displaced" stdDeviation="0.35"/></filter>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn chalk_stroke_returns_palette_color() {
        let mut rng = StdRng::seed_from_u64(42);
        let stroke = apply_chalk_stroke(&mut rng);
        assert!(CHALK_PALETTE.contains(&stroke));
    }

    #[test]
    fn chalk_fill_none_for_closed_shapes() {
        assert_eq!(apply_chalk_fill("red", "rect"), "none");
        assert_eq!(apply_chalk_fill("red", "circle"), "none");
        assert_eq!(apply_chalk_fill("red", "ellipse"), "none");
        assert_eq!(apply_chalk_fill("red", "polygon"), "none");
    }

    #[test]
    fn chalk_fill_preserved_for_open_shapes() {
        assert_eq!(apply_chalk_fill("red", "line"), "red");
        assert_eq!(apply_chalk_fill("blue", "path"), "blue");
    }

    #[test]
    fn chalk_random_opacity_in_range() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..32 {
            let opacity = chalk_random_opacity(&mut rng);
            assert!(opacity >= 0.7);
            assert!(opacity <= 0.95);
        }
    }

    #[test]
    fn chalk_filter_defs_contains_required_elements() {
        let defs = chalk_filter_defs(42);
        assert!(defs.contains("chalk-dust"));
        assert!(defs.contains("feTurbulence"));
        assert!(defs.contains("feDisplacementMap"));
        assert!(defs.contains("feGaussianBlur"));
        assert!(defs.contains(r#"seed="42""#));
    }

    #[test]
    fn chalk_stroke_picks_white_more_than_half_the_time() {
        let mut rng = StdRng::seed_from_u64(42);
        let samples = 1000;
        let white_picks = (0..samples)
            .filter(|_| apply_chalk_stroke(&mut rng) == "#f5f5f5")
            .count();
        assert!(
            white_picks * 2 > samples,
            "white should dominate: got {white_picks}/{samples}"
        );
    }
}
