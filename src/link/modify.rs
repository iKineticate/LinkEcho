use super::{
    list::{LinkList, LinkProp, Status},
    utils::{initialize_com_and_create_shell_link, process_icon},
};
use crate::image::base64::get_img_base64_by_path;

use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use dioxus::signals::{Readable, Signal, Writable};
use glob::glob;
use log::*;
use rfd::FileDialog;
use rust_i18n::t;
use winsafe::{IPersistFile, co, prelude::*};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum MatchPriority {
    FullContains,   // 完全包含匹配（最高优先级）
    PartialOverlap, // 部分重叠匹配（次要优先级）
}

struct MatchScore {
    path: PathBuf,
    priority: MatchPriority,
    length_diff: usize,
}

pub fn partial_match_icon(icon_map: &HashMap<String, PathBuf>, link_name: &str) -> Option<PathBuf> {
    let mut candidates: Vec<MatchScore> = icon_map
        .iter()
        .filter_map(|(icon_name, icon_path)| {
            let (priority, diff) =
                match (link_name.contains(icon_name), icon_name.contains(link_name)) {
                    (true, _) => (
                        MatchPriority::FullContains,
                        link_name.len() - icon_name.len(),
                    ),
                    (_, true) => (
                        MatchPriority::FullContains,
                        icon_name.len() - link_name.len(),
                    ),
                    _ => {
                        // 部分重叠检测，仅检查名称重叠差1个字符
                        let overlap = calculate_overlap(link_name, icon_name);
                        if overlap >= link_name.len() - 1 {
                            (MatchPriority::PartialOverlap, overlap)
                        } else {
                            return None;
                        }
                    }
                };

            Some(MatchScore {
                path: icon_path.clone(),
                priority,
                length_diff: diff,
            })
        })
        .collect();

    // 排序规则：先按匹配类型优先级，相同类型中按长度差匹配
    candidates.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then(a.length_diff.cmp(&b.length_diff))
    });

    candidates.first().map(|m| m.path.clone())
}

// 重叠计算
fn calculate_overlap(a: &str, b: &str) -> usize {
    let min_len = std::cmp::min(a.len(), b.len());
    (0..min_len)
        .filter(|i| a.chars().nth(*i) == b.chars().nth(*i))
        .count()
}

pub fn change_all_shortcuts_icons(mut link_list: Signal<LinkList>) -> Result<bool> {
    let start = std::time::Instant::now();

    let select_icons_folder_path = match FileDialog::new()
        .set_title(t!("SELECT_ICONS_FOLDER"))
        .pick_folder()
    {
        Some(path_buf) => path_buf
            .join("**/*.*")
            .to_str()
            .map(str::to_owned)
            .with_context(|| "Failed to get the path")?,
        None => return Ok(false),
    };

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
                        // 图标同名时优先添加'.ico'至'icon_map'
                        if ext == "ico" {
                            *existing = file_path.clone();
                        }
                    })
                    .or_insert(file_path);
            }

            icon_map
        });

    let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;
    let link_items = link_list.read().items.clone();
    for (index, link_prop) in link_items.iter().enumerate() {
        let link_name = link_prop.name.trim().to_lowercase();
        let link_path = &link_prop.path;
        let link_target_path = Path::new(&link_prop.target_path);
        let link_icon_path = Path::new(&link_prop.icon_path);

        let matched_icon = icon_map
            .remove(&link_name) // 完全匹配
            .or(partial_match_icon(&icon_map, &link_name)) // 部分匹配
            .and_then(|p| process_icon(&p).inspect_err(|e| error!("{e}")).ok());

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
            Ok(()) => {
                let mut link_list_write = link_list.write();
                link_list_write.items[index].icon_path = icon_path.clone();
                link_list_write.items[index].status = Status::Changed;
                link_list_write.items[index].icon_base64 = get_img_base64_by_path(&icon_path);

                info!("{}:\n{link_path}\n{icon_path}", t!("SHORTCUT"));
            }
            Err(e) => {
                error!("Failed to save the shortcut:\n{link_path}\n{e}");
                continue;
            }
        }
    }

    let end = std::time::Instant::now();
    let duration = end.duration_since(start);
    info!("Duration: {:?}", duration);
    Ok(true)
}

pub fn change_single_shortcut_icon(mut link_list: Signal<LinkList>) -> Result<Option<String>> {
    let index = link_list
        .read()
        .state
        .select
        .with_context(|| "LinkList's State prompt does not have a selection icon")?;

    let link_prop = link_list.read().items[index].clone();
    let link_name = &link_prop.name;
    let link_path = &link_prop.path;
    let link_target_path = &link_prop.target_path;
    let link_icon_path = &link_prop.icon_path;

    let icon_ext = ["ico", "png", "svg", "bmp", "webp", "tiff", "exe"];
    let select_icon_path = match FileDialog::new()
        .set_title(t!("SELECT_ONE_ICON"))
        .add_filter("ICON", &icon_ext)
        .pick_file()
    {
        Some(p) => p,
        None => return Ok(None),
    };

    {
        let select_icon_path = select_icon_path.to_string_lossy().to_lowercase();
        let link_icon_path = link_icon_path.to_lowercase();
        if select_icon_path == link_icon_path || &select_icon_path == link_target_path {
            return Ok(None);
        };
    }

    let icon_path = process_icon(&select_icon_path)?;
    let icon_base64 = get_img_base64_by_path(&icon_path);
    let icon_path = icon_path.to_string_lossy().into_owned();

    let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;
    persist_file.Load(link_path, co::STGM::WRITE)?;
    shell_link.SetIconLocation(&icon_path, 0)?;
    persist_file.Save(None, true)?;

    info!("{}:\n{link_path}\n{icon_path}", t!("SHORTCUT"));

    let mut link_list_write = link_list.write();
    link_list_write.items[index].icon_base64 = icon_base64;
    link_list_write.items[index].icon_path = icon_path;
    link_list_write.items[index].status = Status::Changed;

    Ok(Some(link_name.to_owned()))
}

pub fn restore_all_shortcuts_icons(mut link_list: Signal<LinkList>) -> Result<()> {
    let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;

    let items = link_list.read().items.clone();
    for (index, link_prop) in items.iter().enumerate() {
        // Skip shortcuts that are not replaced or extend to uwp|app - 跳过未被更换图标或扩展为uwp|app的快捷方式
        if link_prop.status == Status::Unchanged || link_prop.target_ext == *"uwp|app" {
            continue;
        }

        if let Err(e) = restore_shortcut_icon(link_prop, &shell_link, &persist_file) {
            error!("{}: {e}", t!("ERROR_RESTORE_ONE"));
            continue;
        }

        let link_path = &link_prop.path;
        let link_target_path = &link_prop.target_path;

        let mut link_list_write = link_list.write();
        link_list_write.items[index].icon_path = link_target_path.clone();
        link_list_write.items[index].status = Status::Unchanged;
        link_list_write.items[index].icon_base64 = get_img_base64_by_path(link_target_path);

        info!(
            "{}:\n{link_path}\n{link_target_path}",
            t!("SUCCESS_RESTORE_ONE")
        );
    }

    Ok(())
}

pub fn restore_single_shortcut_icon(mut link_list: Signal<LinkList>) -> Result<Option<String>> {
    let index = link_list
        .read()
        .state
        .select
        .with_context(|| "LinkList's State prompt does not have a selection icon")?;

    let link_prop = link_list.read().items[index].clone();
    let link_name = &link_prop.name;
    let link_path = &link_prop.path;
    let link_target_path = &link_prop.target_path;

    if link_prop.status == Status::Unchanged || link_prop.target_ext == *"uwp|app" {
        return Ok(None);
    };

    let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;

    restore_shortcut_icon(&link_prop, &shell_link, &persist_file)
        .map_err(|e| anyhow!("{}: {link_name}\n{e}", t!("ERROR_RESTORE_ONE")))?;

    let mut link_list_write = link_list.write();
    link_list_write.items[index].icon_path = link_target_path.clone();
    link_list_write.items[index].status = Status::Unchanged;
    link_list_write.items[index].icon_base64 = get_img_base64_by_path(link_target_path);

    info!(
        "{}:\n{link_path}\n{link_target_path}",
        t!("SUCCESS_RESTORE_ONE")
    );

    Ok(Some(link_name.to_owned()))
}

fn restore_shortcut_icon(
    link_prop: &LinkProp,
    shell_link: &winsafe::IShellLink,
    persist_file: &IPersistFile,
) -> Result<()> {
    persist_file.Load(&link_prop.path, co::STGM::WRITE)?;
    shell_link.SetIconLocation(&link_prop.target_path, 0)?;
    persist_file.Save(None, true)?;
    Ok(())
}
