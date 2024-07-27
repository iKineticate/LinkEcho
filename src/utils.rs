use std::fs::{OpenOptions, File};
use std::env;
use std::io::{Error, Write};
use chrono::Local;
use color_eyre::eyre::Result;

pub fn read_log() -> Result<File, Error> {
    let log_path = env::temp_dir().join("LinkEcho.log");
    match log_path.try_exists() {
        Ok(true) => {
            OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
        },
        Ok(false) => Err(Error::new(std::io::ErrorKind::NotFound, "Log file does not exist and cannot be created")),
        Err(err) => Err(err), 
    }
}

pub fn write_log(log_file: &mut File, text: String) -> Result<(), Error> {
    let now_time = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    writeln!(log_file, "{}\n{}", now_time, text)?;
    Ok(())
}
