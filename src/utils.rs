use std::fs::{OpenOptions, File};
use std::env;
use std::io::{Error, Write};
use chrono::Local;
use color_eyre::eyre::Result;

pub fn open_log_file() -> Result<File, Error> {
    let mut log_path = env::temp_dir();

    log_path.push("LinkEcho.log");

    OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
}

pub fn write_log(log_file: &mut File, text: String) -> Result<(), Error> {
    let now_time = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    writeln!(log_file, "{}\n{}", now_time, text)?;
    Ok(())
}
