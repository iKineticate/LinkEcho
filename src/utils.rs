use crate::env;
use anyhow::{Context, Result};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::Local;
use image::ImageFormat;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, ErrorKind::AlreadyExists, Read, Write};
use std::path::{Path, PathBuf};
use win_toast_notify::{CropCircle, WinToastNotify};
use windows_icons::get_icon_base64_by_path;

pub const LOGO_IMAGE: &[u8] = include_bytes!("../resources/logo.png");
const FILE_NOT_EXIST: &str = "iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAMAAAD04JH5AAAAVFBMVEUAAACXl5eamprg4ODf39+Xl5ejo6PU1NSZmZmampqYmJiZmZnLy8u/v7+qqqqsrKzQ0NDX19eZmZn////g4OCzs7Ompqby8vLMzMzl5eXZ2dm/v79UJhU+AAAAEnRSTlMAQL/gICDjw99gn4C208vnz7cEViRYAAACDUlEQVR42u3b3W6jMBCG4fwsWX7S7nYcj8dw//fZk6qTZgDJlf21auc99EmeAMYGiYPn2bppGk91ev7ziZ8/Ub0u52LBQDW73EoFR6oMKBX0tQGFgoFqAwoFp+qAQgFVBxgBGlAm6OoDygTHRgAV4AFWgAeoAA+gmxGAAWcjAANebkaABTzdjAAL6M9GgAXQxQjAAPpnBGAA9f/PRoABKOH56Xr9+961wwJsRwc4wAEOcIADfg6AOcqyRNKQAJYc3kqiCBSA5/ChmbEACY8lAQI4h5USowCcgmYEAEAOGyUMIIfNZgRgCTstAEC6v/IlMsv9SEOAnYDyNsRmrCVA/25cmxepNWC5/32NdbgxYNYLfn1c2gJ0DjLR6iHIbQF6CWzJUmPA5i1HwADZnJ6gU7B8FUC2DvQMAnDemO2oWUAsOc28fYOaWwH2Y7MeggB2j0YaDsDJbEmQgChBY9JaA/ThRBOCAjg8lAkJsNvTFMGA8FAkLCDu/X88IJMJCUgLaXhAFtLwAL34HOAAMIDNEggGUDI7VDAgmjMABlBMZgnAADSmjX7Hy2oHELGYt/VQgJidMBYg5mEEDFh/KfKLFiMH+HL8DZZjzr4ckwMc4AAHOMABDqgK6AhS930BB4K0+6kXoHEHMBCgYQfQ9dS8HvCxV/kk1CZq3ID86NXW2ylo64ZxPDVonKbu4HmmV5qQRJe+53ewAAAAAElFTkSuQmCC";

pub fn ensure_local_app_folder_exists() -> Result<PathBuf> {
    let local_app_data_path = env::var("LOCALAPPDATA")
        .context("Failed to fetches the environment 'LOCALAPPDATA' variable")?;

    let local_link_echo_path = Path::new(&local_app_data_path).join("LinkEcho");

    std::fs::create_dir_all(&local_link_echo_path)
        .context("Failed to create LinkEcho directory at ../Users/MyUser/Appdata/Local")?;

    Ok(local_link_echo_path)
}

pub fn ensure_logo_exists() -> Result<PathBuf> {
    let local_app_folder_path = ensure_local_app_folder_exists()?;
    let logo_path = local_app_folder_path.join("logo.png");

    match OpenOptions::new()
        .write(true)
        .create(true)
        .create_new(true) // 如果文件已存在，返回错误(错误类型为AlreadyExists)
        .open(&logo_path)
    {
        Ok(mut file) => {
            file.write_all(LOGO_IMAGE)?;
            Ok(logo_path)
        }
        Err(e) if e.kind() == AlreadyExists => Ok(logo_path),
        Err(e) => Err(e.into()),
    }
}

// pub fn ensure_temp_app_folder_exists() -> Result<PathBuf> {
//     let temp_path = env::var("TEMP")
//         .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "TEMP not found"))?;
//
//     let temp_link_echo_path = Path::new(&temp_path).join("LinkEcho");
//
//     std::fs::create_dir_all(&temp_link_echo_path)
//         .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Failed to create LinkEcho directory at Local/Temp"))?;
//
//     Ok(temp_link_echo_path)
// }

pub fn write_log(text: String) -> Result<()> {
    let local_app_path = ensure_local_app_folder_exists()?;
    let app_log_path = local_app_path.join("LinkEcho.log");
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(app_log_path)
        .context("Failed to read the log file")?;
    let now_time = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    writeln!(log_file, "{now_time}\n{text}\n")?;
    Ok(())
}

pub fn notify(messages: &str) {
    let logo_path = ensure_logo_exists().expect("The Logo file does not exist");
    let logo_path = logo_path.to_string_lossy();

    WinToastNotify::new()
        // .set_app_id("Link.Echo")
        .set_title("LinkEcho")
        .set_messages(vec![messages])
        .set_logo(&logo_path, CropCircle::False)
        .show()
        .expect("Failed to show toast notification");
}

enum ImageType {
    Ico,
    Png,
    Svg,
    Bmp,
    Tiff,
    Webp,
}

pub fn get_img_base64_by_path(path: &str) -> String {
    let img_path = Path::new(path);
    match img_path.try_exists() {
        Ok(true) => match img_path.extension().and_then(|ext| ext.to_str()) {
            Some("ico") => image_to_base64(img_path, ImageType::Ico),
            Some("png") => image_to_base64(img_path, ImageType::Png),
            Some("svg") => image_to_base64(img_path, ImageType::Svg),
            Some("bmp") => image_to_base64(img_path, ImageType::Bmp),
            Some("tiff") => image_to_base64(img_path, ImageType::Tiff),
            Some("webp") => image_to_base64(img_path, ImageType::Webp),
            _ => {
                let base64 = get_icon_base64_by_path(path).unwrap_or(FILE_NOT_EXIST.to_owned());
                format!("data:image/png;base64,{base64}")
            }
        },
        _ => format!("data:image/png;base64,{}", FILE_NOT_EXIST),
    }
}

fn image_to_base64(path: &Path, image_type: ImageType) -> String {
    match image_type {
        ImageType::Svg => {
            let mut file = match File::open(path) {
                Ok(file) => file,
                Err(_) => return format!("data:image/png;base64,{}", FILE_NOT_EXIST),
            };
            let mut svg_content = String::new();
            if file.read_to_string(&mut svg_content).is_err() {
                return format!("data:image/png;base64,{}", FILE_NOT_EXIST);
            }
            let base64_string = BASE64_STANDARD.encode(svg_content.as_bytes());
            return format!("data:image/svg+xml;base64,{}", base64_string);
        }
        _ => {
            let file = match File::open(path) {
                Ok(file) => file,
                Err(_) => return format!("data:image/png;base64,{}", FILE_NOT_EXIST),
            };
            let mut reader = BufReader::new(file);
            let mut buffer = Vec::new();
            if reader.read_to_end(&mut buffer).is_err() {
                return format!("data:image/png;base64,{}", FILE_NOT_EXIST);
            }

            let format = match image::guess_format(&buffer) {
                Ok(format) => format,
                Err(_) => return format!("data:image/png;base64,{}", FILE_NOT_EXIST),
            };
            let media_type = match format {
                ImageFormat::Png => "image/png",
                ImageFormat::Bmp => "image/bmp",
                ImageFormat::Tiff => "image/itiff",
                ImageFormat::Ico => "image/x-icon",
                ImageFormat::WebP => "image/webp",
                _ => return format!("data:image/png;base64,{}", FILE_NOT_EXIST),
            };

            let base64_string = BASE64_STANDARD.encode(&buffer);
            format!("data:{media_type};base64,{base64_string}")
        }
    }
}
