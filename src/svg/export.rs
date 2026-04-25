/// Export SVG to raster formats (PNG, WebP)
use resvg::{tiny_skia, usvg};

pub fn export_to_png(
    svg: &str,
    dimensions: Option<(u32, u32)>,
    scale: f32,
) -> Result<Vec<u8>, String> {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default())
        .map_err(|e| format!("Failed to parse SVG: {e}"))?;

    let (width, height) = calculate_dimensions(&tree, dimensions, scale)?;

    let mut pixmap =
        tiny_skia::Pixmap::new(width, height).ok_or("Failed to create pixmap")?;

    let render_ts = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, render_ts, &mut pixmap.as_mut());

    pixmap
        .encode_png()
        .map_err(|e| format!("Failed to encode PNG: {e}"))
}

pub fn export_to_webp(
    svg: &str,
    dimensions: Option<(u32, u32)>,
    scale: f32,
) -> Result<Vec<u8>, String> {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default())
        .map_err(|e| format!("Failed to parse SVG: {e}"))?;

    let (width, height) = calculate_dimensions(&tree, dimensions, scale)?;

    let mut pixmap =
        tiny_skia::Pixmap::new(width, height).ok_or("Failed to create pixmap")?;

    let render_ts = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, render_ts, &mut pixmap.as_mut());

    // Encode to WebP using webp::Encoder
    // Note: webp crate is not available in Cargo.toml, so we use tiny-skia's png output instead
    // and would need an external tool or additional dependency for WebP support
    Err("WebP encoding requires additional dependencies. Use PNG format or convert with external tools.".to_string())
}

fn calculate_dimensions(
    tree: &usvg::Tree,
    dimensions: Option<(u32, u32)>,
    scale: f32,
) -> Result<(u32, u32), String> {
    let (width, height) = if let Some((w, h)) = dimensions {
        (w, h)
    } else {
        let root_size = tree.size();
        (
            (root_size.width() * scale) as u32,
            (root_size.height() * scale) as u32,
        )
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
    fn test_export_to_png_with_explicit_dimensions() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <line x1="0" y1="0" x2="100" y2="100" stroke="black"/>
        </svg>"#;

        let result = export_to_png(svg, Some((200, 200)), 1.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_invalid_svg() {
        let svg = "not valid svg";
        let result = export_to_png(svg, None, 1.0);
        assert!(result.is_err());
    }
}
