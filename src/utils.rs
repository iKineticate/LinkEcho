use anyhow::{Context, Result};
use std::env;
use std::fs::OpenOptions;
use std::io::{ErrorKind::AlreadyExists, Write};
use std::path::{Path, PathBuf};
use tauri_winrt_notification::{IconCrop, Toast};
use log::error;

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
    let logo_path = ensure_logo_exists()
        .map_err(|e| error!("Logo file does not exist - {e}"))
        .expect("Logo file does not exist");

    Toast::new(Toast::POWERSHELL_APP_ID)
        .icon(&logo_path, IconCrop::Square, "none")
        .title("LinkEcho")
        .text1(messages)
        .show()
        .map_err(|e| error!("Unable to toast: {e}"))
        .expect("Unable to toast")
}
