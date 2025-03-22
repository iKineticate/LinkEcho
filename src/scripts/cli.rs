use crate::link::{
    info::ManageLinkProp,
    list::LinkList,
    modify::partial_match_icon,
    utils::{initialize_com_and_create_shell_link, process_icon},
};

use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use glob::glob;
use log::*;
use rust_i18n::t;
use winsafe::{co, prelude::*};

pub fn change_all_shortcuts_icons(links_path: Option<PathBuf>, icons_path: &Path) -> Result<bool> {
    let select_icons_folder_path = icons_path
        .join("**\\*.*")
        .to_str()
        .map(str::to_owned)
        .ok_or(anyhow!("Failed to get the path"))?;

    let match_icon_ext = ["ico", "png", "svg", "bmp", "webp", "tiff", "exe"];
    let mut icon_map = glob(&select_icons_folder_path)
        .map_err(|e| anyhow!("Glob failed for {select_icons_folder_path}: {e}"))?
        .filter_map(Result::ok)
        .fold(HashMap::new(), |mut icon_map, file_path| {
            if let Some((name, ext)) = file_path
                .file_stem()
                .and_then(OsStr::to_str)
                .zip(file_path.extension().and_then(OsStr::to_str))
                .map(|(stem, ext)| (stem.trim().to_lowercase(), ext.to_lowercase()))
                .filter(|(_, ext)| match_icon_ext.contains(&ext.as_str()))
            {
                icon_map
                    .entry(name)
                    .and_modify(|existing| {
                        if ext == "ico" {
                            *existing = file_path.clone();
                        }
                    })
                    .or_insert(file_path);
            }

            icon_map
        });

    let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;
    let link_list = links_path.map(LinkList::other).unwrap_or_default();
    for link_prop in link_list.items.iter() {
        let link_name = link_prop.name.trim().to_lowercase();
        let link_path = &link_prop.path;
        let link_target_path = Path::new(&link_prop.target_path);
        let link_icon_path = Path::new(&link_prop.icon_path);

        let matched_icon = icon_map
            .remove(&link_name)
            .or(partial_match_icon(&icon_map, &link_name))
            .and_then(|p| process_icon(&p).ok());

        let icon_path = match matched_icon {
            Some(p) => {
                if p == link_target_path || p == link_icon_path {
                    continue;
                }
                p.to_string_lossy().into_owned()
            }
            None => continue,
        };

        if let Err(e) = persist_file.Load(link_path, co::STGM::WRITE) {
            error!("Failed to load the shortcut:\n{link_path}\n{e}");
            continue;
        }

        if let Err(e) = shell_link.SetIconLocation(&icon_path, 0) {
            error!("Failed to set icon:\n{link_path}\n{icon_path}\n{e}");
            continue;
        }

        // Save a copy of the object to the specified file - 将对象的副本保存到指定文件
        match persist_file.Save(None, true) {
            Ok(()) => info!("{}:\n{link_path}\n{icon_path}", t!("SHORTCUT")),
            Err(e) => {
                error!("Failed to save the shortcut:\n{link_path}\n{e}");
                continue;
            }
        }
    }
    Ok(true)
}

pub fn change_single_shortcut_icon(link_path: &Path, icon_path: &Path) -> Result<bool> {
    let match_icon_ext = ["ico", "png", "svg", "bmp", "webp", "tiff", "exe"];

    let _is_icon = icon_path
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_lowercase)
        .filter(|ext| match_icon_ext.contains(&ext.as_str()))
        .with_context(|| anyhow!("the file is not an icon: {icon_path:?}"))?;

    let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;
    let link_prop = ManageLinkProp::get_info(link_path, &shell_link, &persist_file)?;
    let link_path = &link_prop.path;
    let link_target_path = &link_prop.target_path;
    let link_icon_path = &link_prop.icon_path;

    {
        let select_icon_path = icon_path.to_string_lossy().to_lowercase();
        let link_icon_path = link_icon_path.to_lowercase();
        if select_icon_path == link_icon_path || &select_icon_path == link_target_path {
            return Ok(false);
        };
    }

    let icon_path = process_icon(icon_path)?;

    persist_file.Load(link_path, co::STGM::WRITE)?;
    shell_link.SetIconLocation(&icon_path.to_string_lossy(), 0)?;
    persist_file.Save(None, true)?;

    info!("{}:\n{link_path}\n{icon_path:?}", t!("SHORTCUT"));

    Ok(true)
}
