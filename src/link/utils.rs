use crate::{image::icongen::image_to_ico, utils::ensure_local_app_folder_exists};

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use log::*;
use rust_i18n::t;
use winsafe::{IPersistFile, co, prelude::*};

pub fn initialize_com_and_create_shell_link() -> Result<(winsafe::IShellLink, IPersistFile)> {
    let _com_lib =
        winsafe::CoInitializeEx(co::COINIT::APARTMENTTHREADED | co::COINIT::DISABLE_OLE1DDE)
            .context("Failed to initialize com library")?;

    let shell_link = winsafe::CoCreateInstance::<winsafe::IShellLink>(
        &co::CLSID::ShellLink,
        None,
        co::CLSCTX::INPROC_SERVER,
    )
    .context("Failed to create an IUnknown-derived COM object - ShellLink")?;

    let persist_file: IPersistFile = shell_link
        .QueryInterface()
        .context("Failed to query for ShellLink's IPersistFile interface")?;

    Ok((shell_link, persist_file))
}

pub fn process_icon(icon_path: &Path) -> Result<PathBuf> {
    let ext = icon_path
        .extension()
        .and_then(OsStr::to_str)
        .with_context(|| anyhow!("Not an icon: {icon_path:?}"))?;

    let icon_path = match ext {
        "ico" | "exe" => icon_path.to_path_buf(),
        _ => {
            // 配置文件
            // 1.保存到软件转换图标目录（默认）
            // 2.保存到图标目录```if let Some(convert_icon_path) = icon_path.with_extension("ico")```
            let app_data_path = ensure_local_app_folder_exists()?;
            let icon_data_path = app_data_path.join("icons");
            std::fs::create_dir_all(&icon_data_path)?;
            let icon_name = icon_path
                .file_stem()
                .and_then(OsStr::to_str)
                .with_context(|| anyhow!("Failed to get icon name: {icon_path:?}"))?;
            let convert_icon_path = icon_data_path.join(format!("{icon_name}.ico"));
            if !convert_icon_path.is_file() {
                image_to_ico(icon_path, &convert_icon_path, icon_name)?;
                info!("{}: {icon_name}.{ext}", t!("SUCCESS_IMG_TO_ICO"));
            };
            convert_icon_path
        }
    };

    Ok(icon_path)
}
