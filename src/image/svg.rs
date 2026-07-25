use image::{DynamicImage, RgbaImage};
use resvg::{tiny_skia, usvg};

pub fn open_svg<P: AsRef<std::path::Path>>(
    image_path: P,
    size: u32,
) -> super::Result<DynamicImage> {
    let mut opt = usvg::Options::default();
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    *opt.fontdb_mut() = fontdb;

    let svg_data = std::fs::read(image_path)?;
    let rtree = usvg::Tree::from_data(&svg_data, &opt)?;

    let rtree_size = rtree.size();
    let transform = tiny_skia::Transform::from_scale(
        size as f32 / rtree_size.width(),
        size as f32 / rtree_size.height(),
    );

    let mut pixmap = tiny_skia::Pixmap::new(size, size).ok_or("failed to create SVG Pixmap")?;

    resvg::render(&rtree, transform, &mut pixmap.as_mut());

    RgbaImage::from_raw(size, size, pixmap.take())
        .map(|buffer| buffer.into())
        .ok_or("failed to convert SVG to image".into())
}
