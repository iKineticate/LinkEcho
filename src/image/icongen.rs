// Copyright 2022 Kenton Hamaluik
// Modified by iKineticate on 2024
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   //http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Note: This file has been modified from the original version.

use crate::{PathBuf, utils::notify};
use anyhow::{Context, Result};
use image::codecs::ico::{IcoEncoder, IcoFrame};
use image::{DynamicImage, Rgba, RgbaImage};
use rayon::prelude::*;
use resvg::tiny_skia;
use std::ffi::OsStr;
use std::path::Path;

pub fn image_to_ico(image_path: PathBuf, output_path: PathBuf, name: &str) -> Result<()> {
    let sizes = vec![16, 32, 48, 64, 128, 256];
    let filter = image::imageops::FilterType::CatmullRom;

    let image = load_image(&image_path, &sizes)?;
    check_image_dimensions(&image, name);

    let frames = create_frames(&image, sizes, filter)?;
    save_ico(frames, &output_path)?;

    Ok(())
}

fn load_image(image_path: &PathBuf, sizes: &[u32]) -> Result<DynamicImage> {
    if image_path
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_lowercase)
        .is_some_and(|ext| &ext == "svg")
    {
        load_svg(image_path, sizes)
    } else {
        image::open(image_path)
            .with_context(|| format!("Failed to open file '{}'", image_path.display()))
    }
}

fn load_svg(image_path: &PathBuf, sizes: &[u32]) -> Result<DynamicImage> {
    let mut opt = resvg::usvg::Options::default();
    opt.resources_dir = std::fs::canonicalize(image_path)
        .ok()
        .as_deref()
        .and_then(Path::parent)
        .map(Path::to_path_buf);

    let mut fontdb = resvg::usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    opt.fontdb = fontdb.into();

    let svg_data = std::fs::read(image_path)
        .with_context(|| format!("Failed to read file '{}'", image_path.display()))?;
    let rtree = resvg::usvg::Tree::from_data(&svg_data, &opt)
        .with_context(|| "Failed to parse SVG contents")?;

    let pixmap_size = rtree.size();
    let max_size = *sizes.iter().max().unwrap_or(&256);

    let mut pixmap = resvg::tiny_skia::Pixmap::new(max_size, max_size)
        .ok_or_else(|| anyhow::anyhow!("Failed to create SVG Pixmap!"))?;

    let transform = resvg::tiny_skia::Transform::from_scale(
        max_size as f32 / pixmap_size.width(),
        max_size as f32 / pixmap_size.height(),
    );
    resvg::render(&rtree, transform, &mut pixmap.as_mut());

    let mut image = RgbaImage::new(max_size, max_size);
    for y in 0..max_size {
        for x in 0..max_size {
            let pixel = pixmap
                .pixel(x, y)
                .unwrap_or(tiny_skia::PremultipliedColorU8::TRANSPARENT);
            let rgba = Rgba([pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]);
            image.put_pixel(x, y, rgba);
        }
    }

    Ok(image.into())
}

fn check_image_dimensions(image: &DynamicImage, name: &str) {
    if image.width() != image.height() {
        notify(&format!(
            "Warning: {name} is not square, and will appear squished!"
        ));
    }

    if image.width() < 64 {
        notify(&format!(
            "Warning: You've requested sizes bigger than your input, your image: {name} will be scaled up!"
        ));
    }
}

pub fn create_frames(
    image: &DynamicImage,
    sizes: Vec<u32>,
    filter: image::imageops::FilterType,
) -> Result<Vec<IcoFrame>> {
    let frames: Vec<Vec<u8>> = sizes
        .par_iter()
        .map(|&sz| {
            let resized_image = image.resize_exact(sz, sz, filter);
            resized_image.to_rgba8().to_vec()
        })
        .collect();

    frames
        .par_iter()
        .zip(sizes.par_iter())
        .map(|(buf, &sz)| {
            IcoFrame::as_png(buf.as_slice(), sz, sz, image.color().into())
                .with_context(|| "Failed to encode frame")
        })
        .collect()
}

pub fn save_ico(frames: Vec<IcoFrame>, output_path: &PathBuf) -> Result<()> {
    let file = std::fs::File::create(output_path)
        .with_context(|| format!("Failed to create file '{}'", output_path.display()))?;
    let encoder = IcoEncoder::new(file);
    encoder
        .encode_images(frames.as_slice())
        .with_context(|| "Failed to encode .ico file")
}
