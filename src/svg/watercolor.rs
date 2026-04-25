/// Watercolor theme implementation — color bleed and mixing effect.
use rand::{Rng, RngCore};

use crate::svg::theme::{rewrite_style, ThemeStyle};

pub struct WatercolorStyle;

impl ThemeStyle for WatercolorStyle {
    fn stroke_random(&self, _original: &str, rng: &mut dyn RngCore) -> String {
        apply_watercolor_stroke(rng)
    }
    fn stroke_static(&self, _original: &str) -> String {
        "#FFB3BA".to_string()
    }
    fn fill_random(&self, original: &str, tag: &str, rng: &mut dyn RngCore) -> String {
        apply_watercolor_fill(original, tag, rng)
    }
    fn fill_static(&self, original: &str, tag: &str) -> String {
        if matches!(tag, "rect" | "circle" | "ellipse" | "polygon") {
            "#FFB3BACC".to_string()
        } else {
            original.to_string()
        }
    }
    fn style(&self, style: &str, tag: &str) -> String {
        rewrite_style(style, tag, "#FFB3BA", "#FFB3BACC")
    }
    fn stroke_opacity(&self, rng: &mut dyn RngCore) -> Option<f64> {
        Some(watercolor_random_opacity(rng))
    }
    fn filter_id(&self) -> &'static str {
        "watercolor-bleed"
    }
    fn extra_defs(&self, seed: u64) -> Option<String> {
        Some(watercolor_filter_defs(seed))
    }
    fn extra_replicas(&self, tag: &str) -> usize {
        if matches!(
            tag,
            "path" | "text" | "rect" | "circle" | "ellipse" | "line" | "polyline"
        ) {
            2
        } else {
            0
        }
    }
}

/// Watercolor color palette — soft pastel colors
const WATERCOLOR_PALETTE: &[&str] = &[
    "#FFB3BA", // soft red
    "#FFDFBA", // soft orange
    "#FFFFBA", // soft yellow
    "#BAFFC9", // soft green
    "#BAE1FF", // soft blue
    "#E0BBE4", // soft purple
    "#FFC7F5", // soft pink
];

/// Applies watercolor theme to a stroke color.
/// Selects a pastel color from the palette.
pub fn apply_watercolor_stroke<R: Rng + ?Sized>(rng: &mut R) -> String {
    let idx = (rng.gen::<usize>()) % WATERCOLOR_PALETTE.len();
    WATERCOLOR_PALETTE[idx].to_string()
}

/// Applies watercolor theme to fill color.
/// Watercolor typically has lighter, semi-transparent fills.
pub fn apply_watercolor_fill<R: Rng + ?Sized>(fill: &str, tag: &str, rng: &mut R) -> String {
    if matches!(tag, "rect" | "circle" | "ellipse" | "polygon") {
        // Use a pastel color with transparency
        let idx = (rng.gen::<usize>()) % WATERCOLOR_PALETTE.len();
        let color = WATERCOLOR_PALETTE[idx];
        format!("{}CC", color) // Add alpha channel
    } else {
        fill.to_string()
    }
}

/// Generates randomized opacity for watercolor bleed effect.
pub fn watercolor_random_opacity<R: Rng + ?Sized>(rng: &mut R) -> f64 {
    // Watercolor opacity varies between 0.5 and 0.9 for transparency
    let base = 0.5;
    let variance = rng.gen::<f64>() * 0.4;
    (base + variance).min(1.0)
}

/// Generates blur radius for watercolor effect.
/// Returns a value between 4.0 and 8.0 pixels for more diffuse bleed.
pub fn watercolor_blur_radius<R: Rng + ?Sized>(rng: &mut R) -> f32 {
    4.0 + rng.gen::<f32>() * 4.0
}

/// Creates SVG filter definitions for watercolor theme.
pub fn watercolor_filter_defs(_seed: u64) -> String {
    let blur_radius = 6.0;
    format!(
        r#"<filter id="watercolor-bleed" x="-25%" y="-25%" width="150%" height="150%"><feGaussianBlur stdDeviation="{blur_radius}" result="blurred"/><feColorMatrix in="blurred" type="saturate" values="0.9" result="saturated"/><feOffset in="saturated" dx="0.2" dy="0.2" result="offset"/><feComponentTransfer in="offset" result="faded"><feFuncA type="linear" slope="0.3"/></feComponentTransfer><feComposite in="faded" in2="SourceGraphic" operator="lighten"/></filter>"#,
        blur_radius = blur_radius
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_watercolor_stroke_returns_pastel_color() {
        let mut rng = StdRng::seed_from_u64(42);
        let stroke = apply_watercolor_stroke(&mut rng);
        assert!(stroke.starts_with('#'));
        assert_eq!(stroke.len(), 7); // #RRGGBB
    }

    #[test]
    fn test_watercolor_fill_with_transparency_for_closed_shapes() {
        let mut rng = StdRng::seed_from_u64(42);
        let fill = apply_watercolor_fill("red", "rect", &mut rng);
        assert!(fill.starts_with('#'));
        assert_eq!(fill.len(), 9); // #RRGGBBAA
    }

    #[test]
    fn test_watercolor_fill_preserved_for_open_shapes() {
        let mut rng = StdRng::seed_from_u64(42);
        assert_eq!(apply_watercolor_fill("red", "line", &mut rng), "red");
        assert_eq!(apply_watercolor_fill("blue", "path", &mut rng), "blue");
    }

    #[test]
    fn test_watercolor_random_opacity_in_range() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..10 {
            let opacity = watercolor_random_opacity(&mut rng);
            assert!(opacity >= 0.5);
            assert!(opacity <= 1.0);
        }
    }

    #[test]
    fn test_watercolor_blur_radius_in_range() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..10 {
            let radius = watercolor_blur_radius(&mut rng);
            assert!(radius >= 4.0);
            assert!(radius <= 8.0);
        }
    }

    #[test]
    fn test_watercolor_filter_defs_contains_required_elements() {
        let defs = watercolor_filter_defs(42);
        assert!(defs.contains("watercolor-bleed"));
        assert!(defs.contains("feGaussianBlur"));
        assert!(defs.contains("feColorMatrix"));
    }

    #[test]
    fn test_watercolor_palette_not_empty() {
        assert!(!WATERCOLOR_PALETTE.is_empty());
        for color in WATERCOLOR_PALETTE {
            assert!(color.starts_with('#'));
        }
    }
}
