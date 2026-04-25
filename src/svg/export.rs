/// Export SVG to raster formats (PNG, WebP)
use resvg::{tiny_skia, usvg};

pub fn export_to_png(
    svg: &str,
    dimensions: Option<(Option<u32>, Option<u32>)>,
    scale: f32,
) -> Result<Vec<u8>, String> {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default())
        .map_err(|e| format!("Failed to parse SVG: {e}"))?;

    let (width, height) = calculate_dimensions(&tree, dimensions, scale)?;

    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or("Failed to create pixmap")?;

    let render_ts = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, render_ts, &mut pixmap.as_mut());

    pixmap
        .encode_png()
        .map_err(|e| format!("Failed to encode PNG: {e}"))
}

pub fn export_to_webp(
    svg: &str,
    dimensions: Option<(Option<u32>, Option<u32>)>,
    scale: f32,
) -> Result<Vec<u8>, String> {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default())
        .map_err(|e| format!("Failed to parse SVG: {e}"))?;

    let (width, height) = calculate_dimensions(&tree, dimensions, scale)?;

    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or("Failed to create pixmap")?;

    let render_ts = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, render_ts, &mut pixmap.as_mut());

    // Encode to WebP using webp::Encoder
    // Note: webp crate is not available in Cargo.toml, so we use tiny-skia's png output instead
    // and would need an external tool or additional dependency for WebP support
    Err("WebP encoding requires additional dependencies. Use PNG format or convert with external tools.".to_string())
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
}
