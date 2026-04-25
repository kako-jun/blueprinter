/// Chalk theme implementation — chalkboard with dusty chalk strokes.
use rand::Rng;

/// Chalk palette — white-dominated with pale color chalks.
/// White is repeated to bias selection, mimicking a classroom chalk tray.
const CHALK_PALETTE: &[&str] = &[
    "#f5f5f5", // white
    "#f5f5f5", // white
    "#f5f5f5", // white
    "#f5f5f5", // white
    "#f5f5f5", // white
    "#f5f5f5", // white
    "#fff5b8", // pale yellow
    "#ffd0d0", // pale pink
    "#cfe7ff", // pale blue
    "#d8ffd0", // pale green
];

/// Chalkboard background color — dark slate green.
pub const CHALK_BACKGROUND: &str = "#1f2a25";

/// Default chalk stroke color when a stroke is added because the source had none.
pub const CHALK_DEFAULT_STROKE: &str = "#f5f5f5";

/// Selects a chalk color from the palette, biased toward white.
pub fn apply_chalk_stroke<R: Rng + ?Sized>(rng: &mut R) -> String {
    let idx = rng.gen::<usize>() % CHALK_PALETTE.len();
    CHALK_PALETTE[idx].to_string()
}

/// Closed shapes get no fill; chalk is a line medium.
pub fn apply_chalk_fill(fill: &str, tag: &str) -> String {
    if matches!(tag, "rect" | "circle" | "ellipse" | "polygon") {
        "none".to_string()
    } else {
        fill.to_string()
    }
}

/// Chalk strokes vary in opacity to mimic uneven pressure and dust.
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
        assert!(CHALK_PALETTE.contains(&stroke.as_str()));
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
    fn chalk_palette_white_is_majority() {
        let white_count = CHALK_PALETTE.iter().filter(|c| **c == "#f5f5f5").count();
        assert!(
            white_count * 2 > CHALK_PALETTE.len(),
            "white should dominate the chalk palette"
        );
    }
}
