use crate::{
    FileDialog, LinkList, LinkProp, Path, PathBuf, Status, glob, icongen,
    link::link_info::initialize_com_and_create_shell_link,
    t,
    utils::{ensure_local_app_folder_exists, get_img_base64_by_path, write_log},
};
use anyhow::{Context, Result};
use dioxus::signals::{Readable, Signal, Writable};
use std::ffi::OsStr;
use winsafe::{IPersistFile, co, prelude::*};

pub fn change_all_shortcuts_icons(mut link_list: Signal<LinkList>) -> Result<bool> {
    // Initialize COM library and create IShellLink object
    let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;

    // Vec of matched objects - 存储已匹配的对象们
    let mut match_same_vec: Vec<String> = Vec::new();

    // Get data folder path - 获取数据文件夹路径
    let app_data_path = ensure_local_app_folder_exists().expect("Failed to get the app data path");
    let icon_data_path = app_data_path.join("icons");
    std::fs::create_dir_all(&icon_data_path)
        .context("Failed to create LinkEcho's icons directory at Appdata/Local")?;

    // Select the folder with the icons - 选择存放图标文件夹
    let select_icons_folder_path = match FileDialog::new()
        .set_title(t!("SELECT_ICONS_FOLDER"))
        .pick_folder()
    {
        Some(path_buf) => format!(r"{}\**\*.*", path_buf.display()),
        None => return Ok(false),
    };

    // Iterate through the folder of icons - 遍历快捷方式目录中的图标（包括子目录）
    for path_buf in glob(&select_icons_folder_path)
        .unwrap()
        .filter_map(Result::ok)
    {
        if let Some((icon_path, icon_name)) = process_icon(path_buf, &icon_data_path)? {
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
                    (true, true) => match_same_vec.push(link_prop.path.clone()), // 需排在let ink_prop.icon_location == icon_path上方
                    _ => (),
                }

                // Skip cases with identical icons - 跳过图标相同的情况
                if link_prop.icon_location == icon_path {
                    continue;
                }

                // Load the shortcut file (LNK file) - 载入快捷方式的文件
                if persist_file.Load(&link_prop.path, co::STGM::WRITE).is_err() {
                    write_log(format!(
                        "{}: {}",
                        t!("ERROR_LOAD_LNK_FILE"),
                        &link_prop.path
                    ))?;
                    continue;
                }

                // Set the icon location - 设置图标位置
                if shell_link.SetIconLocation(&icon_path, 0).is_err() {
                    write_log(format!(
                        "{}:\n{}\n{icon_path}",
                        t!("ERROR_SET_ICON_LOCATION"),
                        &link_prop.path
                    ))?;
                    continue;
                }

                // Save a copy of the object to the specified file - 将对象的副本保存到指定文件
                match persist_file.Save(None, true) {
                    Ok(_) => {
                        let mut link_list_write = link_list.write();
                        link_list_write.items[index].icon_location = icon_path.clone();
                        link_list_write.items[index].status = Status::Changed;
                        link_list_write.items[index].icon = get_img_base64_by_path(&icon_path);

                        write_log(format!(
                            "{}:\n{}\n{icon_path}",
                            t!("SHORTCUT"),
                            &link_prop.path
                        ))?;
                    }
                    Err(err) => {
                        write_log(format!(
                            "{}:\n{}\n{err}",
                            t!("ERROR_SAVE_OBJECT_COPY_TO_FILE"),
                            &link_prop.path
                        ))?;
                        continue;
                    }
                }
            }
        }
    }
    Ok(true)
}

pub fn change_single_shortcut_icon(mut link_list: Signal<LinkList>) -> Result<Option<String>> {
    let index = link_list.read().state.select.ok_or(anyhow::anyhow!(
        "LinkList's State prompt does not have a selection icon"
    ))?;
    let link_prop = link_list.read().items[index].clone();
    let link_name = link_prop.name.clone();
    let link_path = link_prop.path.clone();
    let link_target_path = link_prop.target_path.clone();
    let link_icon_location = link_prop.icon_location.clone();

    let icon_path_buf = match FileDialog::new()
        .set_title(t!("SELECT_ONE_ICON"))
        .add_filter("ICONs", &["ico", "png", "bmp", "svg", "tiff", "exe"])
        .pick_file()
    {
        Some(path_buf) => path_buf,
        None => return Ok(None),
    };
    let icon_path = icon_path_buf.to_string_lossy().to_string();

    if icon_path == link_icon_location || icon_path == link_target_path {
        return Ok(None);
    };

    let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;
    persist_file.Load(&link_path, co::STGM::WRITE)?;

    let app_data_path = ensure_local_app_folder_exists().expect("Failed to get the app data path");
    let icon_data_path = app_data_path.join("icons");
    std::fs::create_dir_all(&icon_data_path)?;

    let (icon_path, _) = match process_icon(icon_path_buf, &icon_data_path)? {
        Some((icon_path, icon_name)) => (icon_path, icon_name),
        None => return Ok(None),
    };

    let icon_base64 = get_img_base64_by_path(&icon_path);

    shell_link.SetIconLocation(&icon_path, 0)?;
    persist_file.Save(None, true)?;

    let mut link_list_write = link_list.write();
    link_list_write.items[index].icon = icon_base64;
    link_list_write.items[index].icon_location = icon_path.clone();
    link_list_write.items[index].status = Status::Changed;

    write_log(format!("{}:\n{link_path}\n{icon_path}", t!("SHORTCUT")))?;

    Ok(Some(link_name))
}

fn process_icon(path_buf: PathBuf, icon_data_path: &Path) -> Result<Option<(String, String)>> {
    let ext = path_buf
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_lowercase)
        .filter(|ext| ["ico", "png", "bmp", "svg", "tiff", "exe"].contains(&ext.as_str()));

    if let Some(ext) = ext {
        let icon_name = path_buf
            .file_stem()
            .and_then(OsStr::to_str)
            .map(str::to_lowercase)
            .ok_or_else(|| anyhow::anyhow!("Failed to get icon name: {}", path_buf.display()))?;

        // 若图标非ICO格式，且数据文件夹中无该名称图标，则将转换图片到数据文件夹中
        let icon_path = match ext.as_str() {
            "ico" | "exe" => path_buf.to_string_lossy().to_string(),
            _ => {
                let logo_path = icon_data_path.join(format!("{icon_name}.ico"));
                if !logo_path.exists() {
                    icongen::image_to_ico(path_buf, logo_path.clone(), &icon_name)?;
                    write_log(format!("{}: {icon_name}.{ext}", t!("SUCCESS_IMG_TO_ICO")))?;
                };
                logo_path.to_string_lossy().to_string()
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
        if link_prop.status == Status::Unchanged || link_prop.target_ext == String::from("uwp|app")
        {
            continue;
        }

        if let Err(err) = restore_shortcut_icon(&link_prop, &shell_link, &persist_file) {
            write_log(format!("{}: {}", t!("ERROR_RESTORE_ONE"), err))?;
            continue;
        }

        let mut link_list_write = link_list.write();
        link_list_write.items[index].icon_location = link_prop.target_path.clone();
        link_list_write.items[index].status = Status::Unchanged;
        link_list_write.items[index].icon = get_img_base64_by_path(&link_prop.target_path);

        write_log(format!(
            "{}:\n{}\n{}",
            t!("SUCCESS_RESTORE_ONE"),
            &link_prop.path,
            &link_prop.target_path
        ))?;
    }

    Ok(())
}

pub fn restore_single_shortcut_icon(mut link_list: Signal<LinkList>) -> Result<Option<String>> {
    let index = link_list.read().state.select.ok_or(anyhow::anyhow!(
        "LinkList's State prompt does not have a selection icon"
    ))?;
    let link_prop = link_list.read().items[index].clone();
    let link_name = link_prop.name.clone();
    let link_target_path = link_prop.target_path.clone();
    if link_prop.status == Status::Unchanged || link_prop.target_ext == String::from("uwp|app") {
        return Ok(None);
    };

    let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;

    restore_shortcut_icon(&link_prop, &shell_link, &persist_file).map_err(|err| {
        write_log(format!(
            "{}: {}\n{}",
            t!("ERROR_RESTORE_ONE"),
            link_name,
            err
        ))
        .unwrap();
        err
    })?;

    let mut link_list_write = link_list.write();
    link_list_write.items[index].icon_location = link_target_path.clone();
    link_list_write.items[index].status = Status::Unchanged;
    link_list_write.items[index].icon = get_img_base64_by_path(&link_target_path);

    write_log(format!(
        "{}:\n{}\n{}",
        t!("SUCCESS_RESTORE_ONE"),
        &link_prop.path,
        &link_target_path
    ))?;

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