/// Sumi (墨) theme implementation — ink bleed effect.

use rand::Rng;

/// Applies sumi theme to a stroke color.
/// Sumi uses grayscale colors ranging from black to light gray.
pub fn apply_sumi_stroke(_stroke: &str) -> String {
    // Default to a light gray that mimics ink
    "rgba(50, 50, 50, 0.8)".to_string()
}

/// Applies sumi theme to fill color.
/// Sumi typically has no fill on closed shapes.
pub fn apply_sumi_fill(fill: &str, tag: &str) -> String {
    if matches!(tag, "rect" | "circle" | "ellipse" | "polygon") {
        "none".to_string()
    } else {
        fill.to_string()
    }
}

/// Generates randomized opacity for sumi ink effect.
pub fn sumi_random_opacity<R: Rng + ?Sized>(rng: &mut R) -> f64 {
    // Ink opacity varies between 0.6 and 1.0
    let base = 0.6;
    let variance = rng.gen::<f64>() * 0.4;
    (base + variance).min(1.0)
}

/// Generates blur radius for sumi effect.
/// Returns a value between 2.0 and 4.0 pixels.
pub fn sumi_blur_radius<R: Rng + ?Sized>(rng: &mut R) -> f32 {
    2.0 + rng.gen::<f32>() * 2.0
}

/// Creates SVG filter definitions for sumi theme.
pub fn sumi_filter_defs(_seed: u64) -> String {
    let blur_radius = 3.0;
    format!(
        r#"<filter id="sumi-ink-bleed" x="-15%" y="-15%" width="130%" height="130%"><feGaussianBlur stdDeviation="{blur_radius}" result="blurred"/><feOffset in="blurred" dx="0.1" dy="0.1" result="offset"/><feComponentTransfer in="offset" result="faded"><feFuncA type="linear" slope="0.2"/></feComponentTransfer><feComposite in="faded" in2="SourceGraphic" operator="lighten"/></filter>"#,
        blur_radius = blur_radius
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_sumi_stroke_returns_grayscale() {
        let stroke = apply_sumi_stroke("blue");
        assert!(stroke.contains("50") || stroke.contains("50"));
    }

    #[test]
    fn test_sumi_fill_none_for_closed_shapes() {
        assert_eq!(apply_sumi_fill("red", "rect"), "none");
        assert_eq!(apply_sumi_fill("red", "circle"), "none");
        assert_eq!(apply_sumi_fill("red", "ellipse"), "none");
        assert_eq!(apply_sumi_fill("red", "polygon"), "none");
    }

    #[test]
    fn test_sumi_fill_preserved_for_open_shapes() {
        assert_eq!(apply_sumi_fill("red", "line"), "red");
        assert_eq!(apply_sumi_fill("blue", "path"), "blue");
    }

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
    fn test_sumi_blur_radius_in_range() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..10 {
            let radius = sumi_blur_radius(&mut rng);
            assert!(radius >= 2.0);
            assert!(radius <= 4.0);
        }
    }

    #[test]
    fn test_sumi_filter_defs_contains_required_elements() {
        let defs = sumi_filter_defs(42);
        assert!(defs.contains("sumi-ink-bleed"));
        assert!(defs.contains("feGaussianBlur"));
    }
}
