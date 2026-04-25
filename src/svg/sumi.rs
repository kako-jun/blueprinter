/// Sumi (墨) theme implementation — ink bleed effect.
use rand::{Rng, RngCore};

use crate::svg::theme::{is_closed_shape, rewrite_style, ThemeStyle};

const SUMI_STROKE: &str = "rgba(50, 50, 50, 0.8)";

pub struct SumiStyle;

impl ThemeStyle for SumiStyle {
    fn stroke_static(&self, _original: &str) -> String {
        SUMI_STROKE.to_string()
    }
    fn fill_static(&self, original: &str, tag: &str) -> String {
        if is_closed_shape(tag) {
            "none".to_string()
        } else {
            original.to_string()
        }
    }
    fn style(&self, style: &str, tag: &str) -> String {
        rewrite_style(style, tag, SUMI_STROKE, "none")
    }
    fn stroke_opacity(&self, rng: &mut dyn RngCore) -> Option<f64> {
        Some(sumi_random_opacity(rng))
    }
    fn filter_id(&self) -> &'static str {
        "sumi-ink-bleed"
    }
    fn extra_defs(&self, seed: u64) -> Option<String> {
        Some(sumi_filter_defs(seed))
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

/// Generates randomized opacity for sumi ink effect (0.6-1.0).
pub fn sumi_random_opacity<R: Rng + ?Sized>(rng: &mut R) -> f64 {
    let base = 0.6;
    let variance = rng.gen::<f64>() * 0.4;
    (base + variance).min(1.0)
}

/// Creates SVG filter definitions for sumi theme.
pub fn sumi_filter_defs(_seed: u64) -> String {
    r#"<filter id="sumi-ink-bleed" x="-15%" y="-15%" width="130%" height="130%"><feGaussianBlur stdDeviation="3.0" result="blurred"/><feOffset in="blurred" dx="0.1" dy="0.1" result="offset"/><feComponentTransfer in="offset" result="faded"><feFuncA type="linear" slope="0.2"/></feComponentTransfer><feComposite in="faded" in2="SourceGraphic" operator="lighten"/></filter>"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_sumi_random_opacity_in_range() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..10 {
            let opacity = sumi_random_opacity(&mut rng);
            assert!(opacity >= 0.6);
            assert!(opacity <= 1.0);
        }
    }

    #[test]
    fn test_sumi_filter_defs_contains_required_elements() {
        let defs = sumi_filter_defs(42);
        assert!(defs.contains("sumi-ink-bleed"));
        assert!(defs.contains("feGaussianBlur"));
    }

    #[test]
    fn sumi_style_static_methods_match_palette() {
        let style = SumiStyle;
        assert_eq!(style.stroke_static("anything"), SUMI_STROKE);
        assert_eq!(style.fill_static("red", "rect"), "none");
        assert_eq!(style.fill_static("red", "line"), "red");
    }
}
