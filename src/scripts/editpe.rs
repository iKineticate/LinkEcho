use crate::t;
use anyhow::{Result, anyhow};
use editpe::Image;
use rfd::FileDialog;
use std::ffi::OsStr;

pub fn modify_exe_icon() -> Result<Option<()>> {
    let exe_path = match FileDialog::new()
        .set_title(t!("SELECT_EXE_FILE"))
        .add_filter("EXE", &["exe"])
        .pick_file()
    {
        Some(path_buf) => path_buf,
        None => return Ok(None),
    };

    let icon_path = match FileDialog::new()
        .set_title(t!("SELECT_ICON_FILE"))
        .add_filter("ICON", &["png", "ico", "exe"])
        .pick_file()
    {
        Some(path_buf) => path_buf,
        None => return Ok(None),
    };

    let icon_ext = icon_path
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_lowercase)
        .filter(|ext| ["ico", "png", "exe"].contains(&ext.as_str()));

    match icon_ext.as_deref() {
        Some("exe") => {
            let image = Image::parse_file(&icon_path)?;
            // get the resource directory from the source
            let resources = image.resource_directory().cloned().unwrap_or_default();

            let mut image = Image::parse_file(&exe_path)?;
            // copy the resource directory to the target
            image.set_resource_directory(resources).map_err(|e| {
                anyhow!("Failed to set the directory: {} - {e}", exe_path.display())
            })?;

            // write an executable image with all changes applied
            image
                .write_file(&exe_path)
                .map_err(|e| anyhow!("Failed to write exe: {} - {e}", exe_path.display()))?;
        }
        _ => {
            let mut image = Image::parse_file(&exe_path)
                .map_err(|e| anyhow!("Failed to parse exe: {} - {e}", exe_path.display()))?;

            // get the resource directory
            let mut resources = image.resource_directory().cloned().unwrap_or_default();

            // set the icon file
            resources
                .set_main_icon_file(&icon_path.to_string_lossy().into_owned())
                .map_err(|e| anyhow!("Failed to set the exe icon: {} - {e}", exe_path.display()))?;

            // set the resource directory in the image
            image.set_resource_directory(resources).map_err(|e| {
                anyhow!("Failed to set the directory: {} - {e}", exe_path.display())
            })?;

            // write an executable image with all changes applied
            image
                .write_file(&exe_path)
                .map_err(|e| anyhow!("Failed to write exe: {} - {e}", exe_path.display()))?;
        }
    }

    Ok(Some(()))
}
