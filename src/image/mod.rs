pub mod background;
pub mod corner;
pub mod icon;
pub mod surface;
pub mod svg;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
