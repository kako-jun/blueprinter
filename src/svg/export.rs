/// Export SVG to raster formats (PNG, WebP)
use resvg::{tiny_skia, usvg};

/// usvg defaults to an empty font database, so any `<text>` in the input is
/// silently dropped from raster output. We load system fonts so common faces
/// (Arial, Helvetica, the macOS / Linux defaults) resolve.
fn options_with_fonts() -> usvg::Options<'static> {
    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    opt
}

pub fn export_to_png(
    svg: &str,
    dimensions: Option<(Option<u32>, Option<u32>)>,
    scale: f32,
) -> Result<Vec<u8>, String> {
    let tree = usvg::Tree::from_str(svg, &options_with_fonts())
        .map_err(|e| format!("Failed to parse SVG: {e}"))?;

    let (width, height) = calculate_dimensions(&tree, dimensions, scale)?;

    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or("Failed to create pixmap")?;
    pixmap.fill(tiny_skia::Color::WHITE);

    let render_ts = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, render_ts, &mut pixmap.as_mut());

    pixmap
        .encode_png()
        .map_err(|e| format!("Failed to encode PNG: {e}"))
}

/// Encodes the rasterized SVG as lossless WebP. Lossless preserves the sharp
/// edges of strokes and screentone patterns, which the lossy encoder smears.
pub fn export_to_webp(
    svg: &str,
    dimensions: Option<(Option<u32>, Option<u32>)>,
    scale: f32,
) -> Result<Vec<u8>, String> {
    let tree = usvg::Tree::from_str(svg, &options_with_fonts())
        .map_err(|e| format!("Failed to parse SVG: {e}"))?;

    let (width, height) = calculate_dimensions(&tree, dimensions, scale)?;

    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or("Failed to create pixmap")?;
    pixmap.fill(tiny_skia::Color::WHITE);

    let render_ts = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, render_ts, &mut pixmap.as_mut());

    let encoder = webp::Encoder::from_rgba(pixmap.data(), width, height);
    Ok(encoder.encode_lossless().to_vec())
}

fn calculate_dimensions(
    tree: &usvg::Tree,
    dimensions: Option<(Option<u32>, Option<u32>)>,
    scale: f32,
) -> Result<(u32, u32), String> {
    let svg_size = tree.size();
    let svg_aspect_ratio = svg_size.width() / svg_size.height();

    let (width, height) = match dimensions {
        // Both width and height specified
        Some((Some(w), Some(h))) => (w, h),

        // Width only specified → preserve aspect ratio, calculate height
        Some((Some(w), None)) => {
            let h = (w as f32 / svg_aspect_ratio) as u32;
            (w, h)
        }

        // Height only specified → preserve aspect ratio, calculate width
        Some((None, Some(h))) => {
            let w = (h as f32 * svg_aspect_ratio) as u32;
            (w, h)
        }

        // Neither specified → apply scale
        None => (
            (svg_size.width() * scale) as u32,
            (svg_size.height() * scale) as u32,
        ),

        // This case is impossible (type system ensures it)
        Some((None, None)) => unreachable!(),
    };

    if width == 0 || height == 0 {
        return Err("Invalid dimensions: width and height must be greater than 0".to_string());
    }

    Ok((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_to_png_simple_svg() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <circle cx="50" cy="50" r="40" fill="red"/>
        </svg>"#;

        let result = export_to_png(svg, None, 1.0);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty());
        // PNG magic number
        assert_eq!(&data[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_export_to_png_with_scale() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="blue"/>
        </svg>"#;

        let result = export_to_png(svg, None, 2.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_to_png_with_both_dimensions() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <line x1="0" y1="0" x2="100" y2="100" stroke="black"/>
        </svg>"#;

        let result = export_to_png(svg, Some((Some(200), Some(200))), 1.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_to_png_with_width_only() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="blue"/>
        </svg>"#;

        let result = export_to_png(svg, Some((Some(200), None)), 1.0);
        assert!(result.is_ok());
        // Aspect ratio should be preserved (200 x 200 for square SVG)
    }

    #[test]
    fn test_export_to_png_with_height_only() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
            <rect x="10" y="10" width="180" height="80" fill="green"/>
        </svg>"#;

        let result = export_to_png(svg, Some((None, Some(100))), 1.0);
        assert!(result.is_ok());
        // Aspect ratio should be preserved (200 x 100 for 2:1 SVG)
    }

    #[test]
    fn test_export_invalid_svg() {
        let svg = "not valid svg";
        let result = export_to_png(svg, None, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_to_webp_simple_svg() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <circle cx="50" cy="50" r="40" fill="red"/>
        </svg>"#;

        let data = export_to_webp(svg, None, 1.0).expect("webp encode");
        assert!(!data.is_empty());
        // RIFF container header + WEBP fourcc
        assert_eq!(&data[0..4], b"RIFF");
        assert_eq!(&data[8..12], b"WEBP");
    }

    #[test]
    fn test_export_to_webp_with_scale() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="blue"/>
        </svg>"#;

        let data = export_to_webp(svg, None, 2.0).expect("webp encode");
        assert_eq!(&data[0..4], b"RIFF");
    }

    #[test]
    fn test_export_to_webp_with_explicit_dimensions() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <line x1="0" y1="0" x2="100" y2="100" stroke="black"/>
        </svg>"#;

        let data = export_to_webp(svg, Some((Some(200), Some(200))), 1.0).expect("webp encode");
        assert_eq!(&data[0..4], b"RIFF");
    }

    #[test]
    fn test_export_to_webp_invalid_svg() {
        assert!(export_to_webp("not valid svg", None, 1.0).is_err());
    }

    /// Regression: usvg's default options use an empty font db, so any text in
    /// the source SVG silently disappears in raster output. The exporter must
    /// load system fonts so common faces resolve. We can't assert the bytes
    /// for "Hello" rendered, but we can assert the rasterized image differs
    /// from one rendered without a fontdb (which would render no glyphs).
    #[test]
    fn test_export_renders_text_glyphs() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="80">
            <rect width="200" height="80" fill="#ffffff"/>
            <text x="20" y="50" font-size="32" fill="#000000">Hello</text>
        </svg>"##;

        let with_fonts = export_to_png(svg, None, 1.0).unwrap();

        // Render the same SVG with a deliberately empty fontdb to capture the
        // "no glyphs" baseline.
        let opt = usvg::Options::default(); // empty fontdb
        let tree = usvg::Tree::from_str(svg, &opt).unwrap();
        let size = tree.size();
        let mut pixmap = tiny_skia::Pixmap::new(size.width() as u32, size.height() as u32).unwrap();
        pixmap.fill(tiny_skia::Color::WHITE);
        resvg::render(
            &tree,
            tiny_skia::Transform::identity(),
            &mut pixmap.as_mut(),
        );
        let no_fonts = pixmap.encode_png().unwrap();

        assert_ne!(
            with_fonts, no_fonts,
            "system fonts must be loaded so text actually renders"
        );
    }
}
