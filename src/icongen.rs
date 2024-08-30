// Copyright 2022 Kenton Hamaluik
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

use crate::{utils::show_notify, PathBuf};
use anyhow::{Context, Result};
use image::codecs::ico::{IcoEncoder, IcoFrame};
use image::io::Reader as ImageReader;
use image::{DynamicImage, Rgba, RgbaImage};
use rayon::prelude::*;
use std::ffi::OsStr;

pub fn convert_ico(image: PathBuf, output: PathBuf, name: &str) -> Result<()> {
    let sizes = vec![16, 32, 48, 64, 128, 256];

    let filter = image::imageops::FilterType::CatmullRom;

    let im: DynamicImage = if image
        .extension()
        .map(OsStr::to_str)
        .flatten()
        .map(str::to_lowercase)
        == Some("svg".to_owned())
    {
        let mut opt = usvg::Options::default();
        opt.resources_dir = std::fs::canonicalize(&image)
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));
        opt.fontdb.load_system_fonts();

        let svg = std::fs::read(&image)
            .with_context(|| format!("Failed to read file '{}'", image.display()))?;
        let rtree = usvg::Tree::from_data(&svg, &opt.to_ref())
            .with_context(|| "Failed to parse SVG contents")?;

        let pixmap_size = rtree.svg_node().size.to_screen_size();

        if pixmap_size.width() != pixmap_size.height() {
            show_notify(vec![
                "Warning: your {name} is not square, and will appear squished!",
            ])
        }

        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
            .with_context(|| "Failed to create SVG Pixmap!")?;

        let size = *sizes.iter().max().unwrap();
        resvg::render(
            &rtree,
            usvg::FitTo::Size(size, size),
            tiny_skia::Transform::default(),
            pixmap.as_mut(),
        )
        .with_context(|| "Failed to render SVG!")?;

        // copy it into an image buffer translating types as we go
        // I'm sure there's faster ways of doing this but ¯\_(ツ)_/¯
        let mut image = RgbaImage::new(size, size);
        for y in 0..size {
            for x in 0..size {
                let pixel = pixmap.pixel(x, y).unwrap();
                let pixel = Rgba([pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]);
                image.put_pixel(x, y, pixel);
            }
        }

        image.into()
    } else {
        ImageReader::open(&image)
            .with_context(|| format!("Failed to open file '{}'", image.display()))?
            .decode()
            .with_context(|| "Failed to decode image!")?
    };

    if im.width() != im.height() {
        show_notify(vec![&format!(
            "Warning: {name} is not square, and will appear squished!"
        )]);
    }

    if im.width() < sizes.iter().max().map(|&v| v).unwrap_or_default() {
        show_notify(
            vec![&format!("Warning: You've requested sizes bigger than your input, your {name} will be scaled up!")]
        )
    }

    let frames: Vec<Vec<u8>> = sizes
        .par_iter()
        .map(|&sz| {
            let im = im.resize_exact(sz, sz, filter.into());
            im.to_rgba8().to_vec()
        })
        .collect();

    let frames: Result<Vec<IcoFrame>> = frames
        .par_iter()
        .zip(sizes.par_iter())
        .map(|(buf, &sz)| {
            IcoFrame::as_png(buf.as_slice(), sz, sz, im.color())
                .with_context(|| "Failed to encode frame")
        })
        .collect();
    let frames = frames?;

    let file = std::fs::File::create_new(&output)
        .with_context(|| format!("Failed to create file '{}'", output.display()))?;
    let encoder = IcoEncoder::new(file);
    encoder
        .encode_images(frames.as_slice())
        .with_context(|| "Failed to encode .ico file")?;

    Ok(())
}
