use std::{fs::File, io::Read, path::Path};

use anyhow::{Context, Result, anyhow};
use base64::prelude::{BASE64_STANDARD, Engine};
use image::ImageFormat;
use log::{error, warn};

const FILE_NOT_EXIST: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAMAAAD04JH5AAAAVFBMVEUAAACXl5eamprg4ODf39+Xl5ejo6PU1NSZmZmampqYmJiZmZnLy8u/v7+qqqqsrKzQ0NDX19eZmZn////g4OCzs7Ompqby8vLMzMzl5eXZ2dm/v79UJhU+AAAAEnRSTlMAQL/gICDjw99gn4C208vnz7cEViRYAAACDUlEQVR42u3b3W6jMBCG4fwsWX7S7nYcj8dw//fZk6qTZgDJlf21auc99EmeAMYGiYPn2bppGk91ev7ziZ8/Ub0u52LBQDW73EoFR6oMKBX0tQGFgoFqAwoFp+qAQgFVBxgBGlAm6OoDygTHRgAV4AFWgAeoAA+gmxGAAWcjAANebkaABTzdjAAL6M9GgAXQxQjAAPpnBGAA9f/PRoABKOH56Xr9+961wwJsRwc4wAEOcIADfg6AOcqyRNKQAJYc3kqiCBSA5/ChmbEACY8lAQI4h5USowCcgmYEAEAOGyUMIIfNZgRgCTstAEC6v/IlMsv9SEOAnYDyNsRmrCVA/25cmxepNWC5/32NdbgxYNYLfn1c2gJ0DjLR6iHIbQF6CWzJUmPA5i1HwADZnJ6gU7B8FUC2DvQMAnDemO2oWUAsOc28fYOaWwH2Y7MeggB2j0YaDsDJbEmQgChBY9JaA/ThRBOCAjg8lAkJsNvTFMGA8FAkLCDu/X88IJMJCUgLaXhAFtLwAL34HOAAMIDNEggGUDI7VDAgmjMABlBMZgnAADSmjX7Hy2oHELGYt/VQgJidMBYg5mEEDFh/KfKLFiMH+HL8DZZjzr4ckwMc4AAHOMABDqgK6AhS930BB4K0+6kXoHEHMBCgYQfQ9dS8HvCxV/kk1CZq3ID86NXW2ylo64ZxPDVonKbu4HmmV5qQRJe+53ewAAAAAElFTkSuQmCC";

#[derive(Debug, Clone, Copy)]
enum ImageMediaType {
    Ico,
    Png,
    Svg,
    Bmp,
    Tiff,
    Webp,
}

impl ImageMediaType {
    fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "ico" => Some(Self::Ico),
            "png" => Some(Self::Png),
            "svg" => Some(Self::Svg),
            "bmp" => Some(Self::Bmp),
            "tiff" | "tif" => Some(Self::Tiff),
            "webp" => Some(Self::Webp),
            _ => None,
        }
    }

    fn mime_type(&self) -> &'static str {
        match self {
            Self::Ico => "image/x-icon",
            Self::Png => "image/png",
            Self::Svg => "image/svg+xml",
            Self::Bmp => "image/bmp",
            Self::Tiff => "image/tiff",
            Self::Webp => "image/webp",
        }
    }

    fn image_format(&self) -> Option<ImageFormat> {
        match self {
            Self::Png => Some(ImageFormat::Png),
            Self::Bmp => Some(ImageFormat::Bmp),
            Self::Tiff => Some(ImageFormat::Tiff),
            Self::Ico => Some(ImageFormat::Ico),
            Self::Webp => Some(ImageFormat::WebP),
            Self::Svg => None,
        }
    }
}

pub fn get_img_base64_by_path(path: impl AsRef<Path>) -> String {
    match try_get_img_base64(path) {
        Ok(base64) => base64,
        Err(e) => {
            error!("Failed to process image: {e:?}");
            FILE_NOT_EXIST.to_owned()
        }
    }
}

fn try_get_img_base64(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let exists = path
        .try_exists()
        .with_context(|| format!("File existence check failed: {path:?}"))?;

    if !exists {
        if !path.to_string_lossy().is_empty() {
            warn!("File not found: {path:?}");
        }
        return Ok(FILE_NOT_EXIST.to_owned());
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();

    match ImageMediaType::from_extension(ext) {
        Some(ImageMediaType::Svg) => read_and_encode(path, "image/svg+xml", read_svg_file),
        Some(img_type) => {
            let mime = img_type.mime_type();
            read_and_encode(path, mime, |p| read_binary_file(p, img_type))
        }
        None => handle_unknown_type(path),
    }
}

fn read_and_encode<F>(path: &Path, mime: &str, reader: F) -> Result<String>
where
    F: Fn(&Path) -> Result<Vec<u8>>,
{
    let data = reader(path).with_context(|| format!("Failed to read file: {}", path.display()))?;
    Ok(format!(
        "data:{};base64,{}",
        mime,
        BASE64_STANDARD.encode(data)
    ))
}

fn read_svg_file(path: &Path) -> Result<Vec<u8>> {
    let mut content = String::new();
    File::open(path)
        .with_context(|| format!("Failed to open SVG file: {}", path.display()))?
        .read_to_string(&mut content)
        .context("Failed to read SVG content")?;
    Ok(content.into_bytes())
}

fn read_binary_file(path: &Path, img_type: ImageMediaType) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    File::open(path)
        .with_context(|| format!("Failed to open image file: {}", path.display()))?
        .read_to_end(&mut data)
        .context("Failed to read image data")?;

    if let Some(format) = img_type.image_format() {
        image::load_from_memory_with_format(&data, format).context("Invalid image format")?;
    }

    Ok(data)
}

fn handle_unknown_type(path: &Path) -> Result<String> {
    windows_icons::get_icon_base64_by_path(path)
        .map(|icon| format!("data:image/png;base64,{icon}"))
        .map_err(|e| {
            warn!("Unknown file type: {path:?}");
            anyhow!("{e}")
        })
}
