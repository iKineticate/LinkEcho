use crate::env;
use anyhow::{Context, Result};

use std::fs::OpenOptions;
use std::io::{ErrorKind::AlreadyExists, Write};
use std::path::{Path, PathBuf};
use win_toast_notify::{CropCircle, WinToastNotify};

pub const LOGO_IMAGE: &[u8] = include_bytes!("../resources/logo.png");

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
