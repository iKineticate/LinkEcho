use super::{Result, svg::open_svg};

use std::path::Path;

use image::{
    DynamicImage, GenericImageView,
    codecs::ico::{IcoEncoder, IcoFrame},
    imageops::FilterType,
};
use rayon::prelude::*;
use resvg::usvg;

const SIZES: [u32; 6] = [16, 32, 48, 64, 128, 256];

#[derive(Debug, Clone, PartialEq)]
pub struct IconImage {
    input_path: String,
    sizes: Vec<u32>,
    pub image: DynamicImage,
    pub width: u32,
    pub height: u32,
}

impl IconImage {
    pub fn new(input_path: &str, sizes: Option<Vec<u32>>) -> Result<Self> {
        let image = Self::open_image(&input_path)?;
        let (width, height) = image.dimensions();

        Ok(Self {
            input_path: input_path.to_owned(),
            sizes: sizes.unwrap_or_else(|| SIZES.to_vec()),
            image,
            width,
            height,
        })
    }

    pub fn path(&self) -> String {
        self.input_path.clone()
    }

    fn open_image<P: AsRef<Path>>(path: P) -> Result<DynamicImage> {
        let image_date = std::fs::read(&path)?;

        let extension = infer::get(&image_date)
            .map_or_else(
                || {
                    // infer lib can't detect svg, so we use usvg to detect svg
                    usvg::Tree::from_data(&image_date, &usvg::Options::default())
                        .ok()
                        .map(|_| "svg")
                },
                |r#type| Some(r#type.extension()),
            )
            .ok_or_else(|| "failed to get file extension".to_owned())?;

        if extension == "svg" {
            open_svg(path, *SIZES.iter().max().unwrap_or(&256))
                .map_err(|e| format!("failed to open svg: {e}").into())
        } else if extension == "exe" {
            windows_icons::get_icon_by_path(path).map(|i| i.into())
        } else {
            image::load_from_memory(&image_date)
                .map_err(|e| format!("failed to open image: {e}").into())
        }
    }

    pub fn save_to_ico<P: AsRef<Path>>(&self, ouput_path: P) -> Result<()> {
        let image = &self.image;
        let color_type = image.color();
        let output_file = std::fs::File::create(ouput_path)?;

        if image.width() != image.height() {
            log::warn!("non-square image will appear squished");
        }
        if image.width() < 32 {
            log::warn!("image smaller than 32x32 may appear blurry");
        }

        let frames = self
            .sizes
            .par_iter()
            .filter_map(|&sz| {
                let resized_image = image.resize_to_fill(sz, sz, FilterType::CatmullRom);

                let rgba_img = resized_image.to_rgba8();

                IcoFrame::as_png(&rgba_img, sz, sz, color_type.into()).ok()
            })
            .collect::<Vec<_>>();

        IcoEncoder::new(output_file)
            .encode_images(&frames)
            .map_err(|e| format!("failed to encode frames to ico: {e}").into())
    }
}
