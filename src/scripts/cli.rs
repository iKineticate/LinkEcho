use crate::{
    link::{
        link_info::{ManageLinkProp, initialize_com_and_create_shell_link},
        link_list::LinkList,
    },
    link_modify::process_icon,
    t,
};

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use glob::glob;
use log::*;
use winsafe::{co, prelude::*};

pub fn change_all_shortcuts_icons(links_path: Option<PathBuf>, icons_path: &Path) -> Result<bool> {
    let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;

    let mut match_same_vec: Vec<String> = Vec::new();

    let select_icons_folder_path = icons_path
        .join("**\\*.*")
        .to_str()
        .map(str::to_owned)
        .ok_or(anyhow!("Failed to get the path"))?;

    let link_list = links_path
        .map(|p| LinkList::other(p))
        .unwrap_or_else(LinkList::default);

    for path_buf in glob(&select_icons_folder_path)
        .map_err(|e| anyhow!("Glob failed for {select_icons_folder_path}: {e}"))?
        .filter_map(Result::ok)
    {
        if let Some((icon_path, icon_name)) = process_icon(path_buf)? {
            let items = link_list.items.clone();
            for link_prop in items.iter() {
                if match_same_vec.contains(&link_prop.path) {
                    continue;
                }

                let link_name_lowercase = &link_prop.name.trim().to_lowercase();
                match (
                    link_name_lowercase.contains(&icon_name),
                    icon_name.contains(link_name_lowercase),
                ) {
                    (false, false) => continue,
                    (true, true) => match_same_vec.push(link_prop.path.clone()),
                    _ => (),
                }

                if link_prop.icon_path == icon_path {
                    continue;
                }

                if let Err(e) = persist_file.Load(&link_prop.path, co::STGM::WRITE) {
                    error!("{}:\n{}\n{e}", t!("ERROR_LOAD_LNK_FILE"), link_prop.path);
                    continue;
                }

                if let Err(e) = shell_link.SetIconLocation(&icon_path, 0) {
                    error!(
                        "{}:\n{}\n{icon_path}\n{e}",
                        t!("ERROR_SET_ICON_LOCATION"),
                        link_prop.path
                    );
                    continue;
                }

                match persist_file.Save(None, true) {
                    Ok(()) => info!("{}:\n{}\n{icon_path}", t!("SHORTCUT"), link_prop.path),
                    Err(e) => {
                        error!(
                            "{}:\n{}\n{e}",
                            t!("ERROR_SAVE_OBJECT_COPY_TO_FILE"),
                            link_prop.path
                        );
                        continue;
                    }
                }
            }
        }
    }
    Ok(true)
}

pub fn change_single_shortcut_icon(link_path: &Path, icon_path: &Path) -> Result<bool> {
    let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;
    let link_prop = ManageLinkProp::get_info(link_path, &shell_link, &persist_file)?;
    let link_path = link_prop.path.clone();
    let link_target_path = link_prop.target_path.clone();
    let link_icon_path = link_prop.icon_path.clone();

    let icon_path_buf = icon_path.to_path_buf();
    let icon_path = icon_path.to_string_lossy().to_string();

    if icon_path == link_icon_path || icon_path == link_target_path {
        return Ok(false);
    };

    persist_file.Load(&link_path, co::STGM::WRITE)?;

    let (icon_path, _) = match process_icon(icon_path_buf)? {
        Some((icon_path, icon_name)) => (icon_path, icon_name),
        None => return Ok(false),
    };

    shell_link.SetIconLocation(&icon_path, 0)?;
    persist_file.Save(None, true)?;

    info!("{}:\n{link_path}\n{icon_path}", t!("SHORTCUT"));

    Ok(true)
}
