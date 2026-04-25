//! Filter definitions for various themes.

/// Creates a Gaussian blur SVG filter definition.
pub fn create_blur_filter(id: &str, radius: f32) -> String {
    format!(
        r#"<filter id="{id}" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="{radius}"/></filter>"#
    )
}

/// Creates a color blend filter for watercolor theme.
pub fn create_color_blend_filter(id: &str) -> String {
    format!(
        r#"<filter id="{id}" x="-20%" y="-20%" width="140%" height="140%"><feColorMatrix type="saturate" values="0.9"/></feColorMatrix></filter>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_blur_filter() {
        let filter = create_blur_filter("test-blur", 3.5);
        assert!(filter.contains("id=\"test-blur\""));
        assert!(filter.contains("feGaussianBlur"));
        assert!(filter.contains("3.5"));
    }

    #[test]
    fn test_create_color_blend_filter() {
        let filter = create_color_blend_filter("test-blend");
        assert!(filter.contains("id=\"test-blend\""));
        assert!(filter.contains("feColorMatrix"));
    }
}
