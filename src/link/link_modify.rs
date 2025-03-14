use crate::{
    FileDialog, LinkList, LinkProp, PathBuf, Status, glob, icongen,
    image::base64::get_img_base64_by_path, link::link_info::initialize_com_and_create_shell_link,
    t, utils::ensure_local_app_folder_exists,
};
use anyhow::{Context, Result, anyhow};
use dioxus::signals::{Readable, Signal, Writable};
use log::*;
use std::ffi::OsStr;
use winsafe::{IPersistFile, co, prelude::*};

pub fn change_all_shortcuts_icons(mut link_list: Signal<LinkList>) -> Result<bool> {
    // Initialize COM library and create IShellLink object
    let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;

    // Vec of matched objects - 存储已匹配的对象们
    let mut match_same_vec: Vec<String> = Vec::new();

    // Select the folder with the icons - 选择存放图标文件夹
    let select_icons_folder_path = match FileDialog::new()
        .set_title(t!("SELECT_ICONS_FOLDER"))
        .pick_folder()
    {
        Some(path_buf) => path_buf
            .join("**\\*.*")
            .to_str()
            .map(str::to_owned)
            .ok_or(anyhow!("Failed to get the path"))?,
        None => return Ok(false),
    };

    // Iterate through the folder of icons - 遍历快捷方式目录中的图标（包括子目录）
    for path_buf in glob(&select_icons_folder_path)
        .map_err(|e| anyhow!("Glob failed for {select_icons_folder_path}: {e}"))?
        .filter_map(Result::ok)
    {
        if let Some((icon_path, icon_name)) = process_icon(path_buf)? {
            let items = link_list.read().items.clone();
            // Iterate over the vec that stores the shortcut properties - 遍历快捷方式的属性
            for (index, link_prop) in items.iter().enumerate() {
                // Skip matched objects - 跳过已匹配的对象
                if match_same_vec.contains(&link_prop.path) {
                    continue;
                }

                // Compare the containment relationship between two strings - 比较图标名称与LNK名称的包含关系
                let link_name_lowercase = &link_prop.name.trim().to_lowercase();
                match (
                    link_name_lowercase.contains(&icon_name),
                    icon_name.contains(link_name_lowercase),
                ) {
                    (false, false) => continue,
                    (true, true) => match_same_vec.push(link_prop.path.clone()), // 需排在let ink_prop.icon_path == icon_path上方
                    _ => (),
                }

                // Skip cases with identical icons - 跳过图标相同的情况
                if link_prop.icon_path == icon_path {
                    continue;
                }

                // Load the shortcut file (LNK file) - 载入快捷方式的文件
                if let Err(e) = persist_file.Load(&link_prop.path, co::STGM::WRITE) {
                    error!("{}:\n{}\n{e}", t!("ERROR_LOAD_LNK_FILE"), link_prop.path);
                    continue;
                }

                // Set the icon location - 设置图标位置
                if let Err(e) = shell_link.SetIconLocation(&icon_path, 0) {
                    error!(
                        "{}:\n{}\n{icon_path}\n{e}",
                        t!("ERROR_SET_ICON_LOCATION"),
                        link_prop.path
                    );
                    continue;
                }

                // Save a copy of the object to the specified file - 将对象的副本保存到指定文件
                match persist_file.Save(None, true) {
                    Ok(()) => {
                        let mut link_list_write = link_list.write();
                        link_list_write.items[index].icon_path = icon_path.clone();
                        link_list_write.items[index].status = Status::Changed;
                        link_list_write.items[index].icon_base64 =
                            get_img_base64_by_path(&icon_path);

                        info!("{}:\n{}\n{icon_path}", t!("SHORTCUT"), link_prop.path);
                    }
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

pub fn change_single_shortcut_icon(mut link_list: Signal<LinkList>) -> Result<Option<String>> {
    let index = link_list.read().state.select.ok_or(anyhow!(
        "LinkList's State prompt does not have a selection icon"
    ))?;
    let link_prop = link_list.read().items[index].clone();
    let link_name = link_prop.name.clone();
    let link_path = link_prop.path.clone();
    let link_target_path = link_prop.target_path.clone();
    let link_icon_path = link_prop.icon_path.clone();

    let icon_path_buf = match FileDialog::new()
        .set_title(t!("SELECT_ONE_ICON"))
        .add_filter(
            "ICONs",
            &["ico", "png", "bmp", "svg", "tiff", "exe", "webp"],
        )
        .pick_file()
    {
        Some(path_buf) => path_buf,
        None => return Ok(None),
    };
    let icon_path = icon_path_buf.to_string_lossy().to_string();

    if icon_path == link_icon_path || icon_path == link_target_path {
        return Ok(None);
    };

    let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;
    persist_file.Load(&link_path, co::STGM::WRITE)?;

    let (icon_path, _) = match process_icon(icon_path_buf)? {
        Some((icon_path, icon_name)) => (icon_path, icon_name),
        None => return Ok(None),
    };

    let icon_base64 = get_img_base64_by_path(&icon_path);

    shell_link.SetIconLocation(&icon_path, 0)?;
    persist_file.Save(None, true)?;

    let mut link_list_write = link_list.write();
    link_list_write.items[index].icon_base64 = icon_base64;
    link_list_write.items[index].icon_path = icon_path.clone();
    link_list_write.items[index].status = Status::Changed;

    info!("{}:\n{link_path}\n{icon_path}", t!("SHORTCUT"));

    Ok(Some(link_name))
}

pub fn process_icon(path_buf: PathBuf) -> Result<Option<(String, String)>> {
    // Get data folder path - 获取软件的图标目录路径
    let app_data_path = ensure_local_app_folder_exists().expect("Failed to get the app data path");
    let icon_data_path = app_data_path.join("icons");
    std::fs::create_dir_all(&icon_data_path)
        .context("Failed to create LinkEcho's icons directory at Appdata/Local")?;

    let ext = path_buf
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_lowercase)
        .filter(|ext| ["ico", "png", "bmp", "svg", "tiff", "exe", "webp"].contains(&ext.as_str()));

    if let Some(ext) = ext {
        let icon_name = path_buf
            .file_stem()
            .and_then(OsStr::to_str)
            .map(str::to_lowercase)
            .ok_or_else(|| anyhow!("Failed to get icon name: {path_buf:?}"))?;

        // 若图标非ICO格式，且数据文件夹中无该名称图标，则将转换图片到数据文件夹中
        let icon_path = match ext.as_str() {
            "ico" | "exe" => path_buf.to_string_lossy().to_string(),
            _ => {
                let logo_path = icon_data_path.join(format!("{icon_name}.ico"));
                if !logo_path.exists() {
                    icongen::image_to_ico(path_buf, logo_path.clone(), &icon_name)?;
                    info!("{}: {icon_name}.{ext}", t!("SUCCESS_IMG_TO_ICO"));
                };
                logo_path
                    .to_str()
                    .ok_or(anyhow!("Failed to get icon path"))?
                    .to_string()
            }
        };

        Ok(Some((icon_path, icon_name)))
    } else {
        Ok(None)
    }
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

        let mut link_list_write = link_list.write();
        link_list_write.items[index].icon_path = link_prop.target_path.clone();
        link_list_write.items[index].status = Status::Unchanged;
        link_list_write.items[index].icon_base64 = get_img_base64_by_path(&link_prop.target_path);

        info!(
            "{}:\n{}\n{}",
            t!("SUCCESS_RESTORE_ONE"),
            link_prop.path,
            link_prop.target_path
        );
    }

    Ok(())
}

pub fn restore_single_shortcut_icon(mut link_list: Signal<LinkList>) -> Result<Option<String>> {
    let index = link_list.read().state.select.ok_or(anyhow!(
        "LinkList's State prompt does not have a selection icon"
    ))?;
    let link_prop = link_list.read().items[index].clone();
    let link_name = link_prop.name.clone();
    let link_target_path = link_prop.target_path.clone();
    if link_prop.status == Status::Unchanged || link_prop.target_ext == *"uwp|app" {
        return Ok(None);
    };

    let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;

    restore_shortcut_icon(&link_prop, &shell_link, &persist_file)
        .map_err(|e| anyhow!("{}: {link_name}\n{e}", t!("ERROR_RESTORE_ONE")))?;

    let mut link_list_write = link_list.write();
    link_list_write.items[index].icon_path = link_target_path.clone();
    link_list_write.items[index].status = Status::Unchanged;
    link_list_write.items[index].icon_base64 = get_img_base64_by_path(&link_target_path);

    info!(
        "{}:\n{}\n{}",
        t!("SUCCESS_RESTORE_ONE"),
        link_prop.path,
        link_target_path
    );

    Ok(Some(link_name))
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
