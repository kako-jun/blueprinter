pub mod parser;
pub mod primitive;
pub mod transform;

pub use parser::parse_svg;
pub use primitive::Primitive;
pub use transform::{transform_svg, TransformOptions, Theme};
