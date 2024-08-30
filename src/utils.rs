use crate::{env, PathBuf};
use chrono::Local;
use color_eyre::eyre::Result;
use std::fs::{write, File, OpenOptions};
use std::io::{Error, Write};
use win_toast_notify::{CropCircle, WinToastNotify};

pub fn read_log() -> Result<File, Error> {
    let log_path = env::temp_dir().join("LinkEcho.log");
    match log_path.try_exists() {
        Ok(_) => OpenOptions::new().create(true).append(true).open(log_path),
        Err(err) => Err(err),
    }
}

pub fn write_log(log_file: &mut File, text: String) -> Result<(), Error> {
    let now_time = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    writeln!(log_file, "{}\n{}\n", now_time, text)?;
    Ok(())
}

pub fn show_notify(messages: Vec<&str>) {
    let logo_path = env::temp_dir().join("linkecho.png");
    let logo_path = logo_path.to_string_lossy();

    WinToastNotify::new()
        // .set_app_id("Link.Echo.Test")
        .set_title("LinkEcho")
        .set_messages(messages)
        .set_logo(&logo_path, CropCircle::False)
        .show()
        .expect("Failed to show toast notification");
}

pub fn ensure_image_exists(logo_path: PathBuf, LOGO_IMAGE: &[u8]) {
    match logo_path.try_exists() {
        Ok(true) => {
            if !logo_path.is_file() {
                write(&logo_path, LOGO_IMAGE).expect("Unable to write file");
            }
        }
        Ok(false) => {
            write(&logo_path, LOGO_IMAGE).expect("Unable to write file");
        }
        _ => (),
    };
}
