use crate::{
    env, glob, icongen,
    utils::{read_log, show_notify, write_log},
    FileDialog, LinkProp, Path, PathBuf, Status,
};
use std::process::Command;
use winsafe::{co, prelude::*, IPersistFile};

pub fn change_all_shortcuts_icons(
    link_vec: &mut Vec<LinkProp>,
) -> Result<Option<&str>, Box<dyn std::error::Error>> {
    // Initialize COM library - 初始化 COM 库
    let _com_lib = winsafe::CoInitializeEx(
        // keep guard alive
        co::COINIT::APARTMENTTHREADED | co::COINIT::DISABLE_OLE1DDE,
    )?;

    // Create IShellLink object - 创建 IShellLink 对象实例
    let shell_link = winsafe::CoCreateInstance::<winsafe::IShellLink>(
        &co::CLSID::ShellLink,
        None,
        co::CLSCTX::INPROC_SERVER,
    )?;

    // Query for IPersistFile interface - 查询并获取 IPersistFile 接口实例
    let persist_file: IPersistFile = shell_link.QueryInterface()?;

    // Vec of matched objects - 存储已匹配的对象们
    let mut match_same_vec = vec![];

    // Open Log File - 打开日志文件
    let mut log_file = read_log().expect("Failed to open 'LinkEcho.log'");

    // Get data folder path - 获取数据文件夹路径
    let app_date =
        winsafe::SHGetKnownFolderPath(&co::KNOWNFOLDERID::LocalAppData, co::KF::NO_ALIAS, None)
            .map_or(String::new(), |path| format!("{path}\\LinkEcho"));

    // Create APP Date - 创建数据文件夹
    match Path::new(&app_date).try_exists() {
        Ok(true) => (),
        Ok(false) => std::fs::create_dir(&app_date)?,
        Err(err) => return Err(err.into()),
    };

    // Select the folder with the icons - 选择有图标的文件夹
    let select_icons_folder_path = match FileDialog::new()
        .set_title("Please select the folder where the icons are stored")
        .pick_folder()
    {
        Some(path_buf) => format!(r"{}\**\*.*", path_buf.to_string_lossy().to_string()),
        None => return Ok(None),
    };

    // Iterate through the folder of icons - 遍历快捷方式目录中的图标（包括子目录）
    for path_buf in glob(&select_icons_folder_path)
        .unwrap()
        .filter_map(Result::ok)
    {
        let ext = match path_buf.extension() {
            Some(ext_os_str) => {
                if let Some(extension) = ext_os_str.to_str() {
                    let ext_string = extension.to_lowercase();
                    if ["ico", "png", "svg", "tiff", "webp"].contains(&ext_string.as_str()) {
                        ext_string
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            None => continue,
        };

        // Get icon name and path
        let mut icon_path = path_buf.to_string_lossy().to_string();

        let icon_name = match path_buf.file_stem() {
            Some(name) => name.to_string_lossy().to_string().trim().to_lowercase(), // Subsequent case-insensitive matching - 后续不区分大小写进行匹配
            None => {
                write_log(&mut log_file, format!("Icon without name: {icon_path}"))?;
                continue;
            }
        };

        // 若图标非ICO格式，且数据文件夹中无该名称图标，则转换图片到数据文件夹中，改变icon_path路径
        if ext != "ico".to_string() {
            let logo_path_buf = Path::new(&app_date).join(format!("{icon_name}.ico"));
            let logo_path_string = logo_path_buf.to_string_lossy().to_string();

            match logo_path_buf.try_exists() {
                Ok(true) => icon_path = logo_path_string,
                Ok(false) => {
                    match icongen::convert_ico(path_buf, logo_path_buf, &icon_name) {
                        Ok(_) => {
                            icon_path = logo_path_string;
                            write_log(
                                &mut log_file,
                                format!("Successfully converted {icon_name}.{ext} to ICO"),
                            )?;
                        }
                        Err(err) => {
                            write_log(
                                &mut log_file,
                                format!("Failed to convert {icon_name}.{ext} to ICO: {err}"),
                            )?;
                            continue;
                        }
                    };
                }
                Err(err) => {
                    write_log(
                        &mut log_file,
                        format!("Error checking if {logo_path_string} exists: {err}"),
                    )?;
                    continue;
                }
            };
        };

        // Iterate over the vec that stores the shortcut properties - 遍历快捷方式的属性
        for link_prop in link_vec.iter_mut() {
            // Skip matched objects - 跳过已匹配的对象
            if match_same_vec.contains(&link_prop.path) {
                continue;
            };

            // Compare the containment relationship between two strings - 比较图标名称与LNK名称的包含关系
            let link_name_lowercase = &link_prop.name.trim().to_lowercase();
            match (
                link_name_lowercase.contains(&icon_name),
                icon_name.contains(link_name_lowercase),
            ) {
                (false, false) => continue,
                (true, true) => match_same_vec.push(link_prop.path.clone()), // 需排在ink_prop.icon_location == icon_path上方
                _ => (),
            };

            // Skip cases with identical icons - 跳过图标相同的情况
            if link_prop.icon_location == icon_path {
                continue;
            };

            // Load the shortcut file (LNK file) - 载入快捷方式的文件
            if let Err(_) = persist_file.Load(&link_prop.path, co::STGM::WRITE) {
                write_log(
                    &mut log_file,
                    format!("Failed to load the shortcut: {}", &link_prop.path),
                )?;
                continue;
            }

            // Set the icon location - 设置图标位置
            if let Err(_) = shell_link.SetIconLocation(&icon_path, 0) {
                write_log(
                    &mut log_file,
                    format!(
                        "Failed to set the icon location:\n{}\n{icon_path}",
                        &link_prop.path
                    ),
                )?;
                continue;
            }

            // Save a copy of the object to the specified file - 将对象的副本保存到指定文件
            match persist_file.Save(None, true) {
                Ok(_) => {
                    link_prop.icon_location = String::from(icon_path.clone()); //
                    link_prop.status = Status::Changed;
                    write_log(
                        &mut log_file,
                        format!(
                            "Successfully set the icon location:\n{}\n{icon_path}",
                            &link_prop.path
                        ),
                    )?
                }
                Err(err) => {
                    write_log(
                        &mut log_file,
                        format!(
                            "Failed to save a copy of the object to the specified file:\n{}\n{err}",
                            &link_prop.path
                        ),
                    )?;
                    continue;
                }
            };
        }
    }

    Ok(Some(""))
}

pub fn restore_all_shortcuts_icons(
    link_vec: &mut Vec<LinkProp>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 在main.rs里通知是否恢复所有默认

    // Initialize COM library - 初始化 COM 库
    let _com_lib = winsafe::CoInitializeEx(
        // keep guard alive
        co::COINIT::APARTMENTTHREADED | co::COINIT::DISABLE_OLE1DDE,
    )?;

    // Create IShellLink object - 创建 IShellLink 对象实例
    let shell_link = winsafe::CoCreateInstance::<winsafe::IShellLink>(
        &co::CLSID::ShellLink,
        None,
        co::CLSCTX::INPROC_SERVER,
    )?;

    // Query for IPersistFile interface - 查询并获取 IPersistFile 接口实例
    let persist_file: IPersistFile = shell_link.QueryInterface()?;

    // Open Log File - 打开日志文件
    let mut log_file = read_log().expect("Failed to open 'LinkEcho.log'");

    // Iterate over the vec that stores the shortcut properties - 遍历快捷方式的属性
    for link_prop in link_vec.iter_mut() {
        // Skip shortcuts that are not replaced or extend to uwp|app - 跳过未被更换图标或扩展为uwp|app的快捷方式
        if link_prop.status == Status::Unchanged || link_prop.target_ext == String::from("uwp|app")
        {
            continue;
        }
        let icon_path = &link_prop.target_path;

        // Load the shortcut file (LNK file) - 载入快捷方式的文件
        if let Err(_) = persist_file.Load(&link_prop.path, co::STGM::WRITE) {
            write_log(
                &mut log_file,
                format!("Failed to load the shortcut: {}", &link_prop.path),
            )?;
            continue;
        }

        // Set the icon location - 设置图标位置
        if let Err(_) = shell_link.SetIconLocation(&icon_path, 0) {
            write_log(
                &mut log_file,
                format!(
                    "Failed to restore the icon location:\n{}\n{icon_path}",
                    &link_prop.path
                ),
            )?;
            continue;
        }

        // Saves a copy of the object to the specified file - 将对象的副本保存到指定文件
        match persist_file.Save(None, true) {
            Ok(_) => write_log(
                &mut log_file,
                format!(
                    "Successfully restore the default icon:\n{}\n{icon_path}",
                    &link_prop.path
                ),
            )?,
            Err(_) => {
                write_log(
                    &mut log_file,
                    format!(
                        "Failed to save a copy of the object to the specified file:\n{}",
                        &link_prop.path
                    ),
                )?;
                continue;
            }
        };

        // Update the icon path and icon status in the LinkProp structure - 更新LinkProp结构体
        link_prop.icon_location = icon_path.clone();
        link_prop.status = Status::Unchanged;
    }

    Ok(())
}

pub fn change_single_shortcut_icon(
    link_path: String,
    link_prop: &mut LinkProp,
    filter_link_prop: Option<&mut LinkProp>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Initialize COM library - 初始化 COM 库
    let _com_lib = winsafe::CoInitializeEx(
        // keep guard alive
        co::COINIT::APARTMENTTHREADED | co::COINIT::DISABLE_OLE1DDE,
    )?;

    // Create IShellLink object - 创建 IShellLink 对象实例
    let shell_link = winsafe::CoCreateInstance::<winsafe::IShellLink>(
        &co::CLSID::ShellLink,
        None,
        co::CLSCTX::INPROC_SERVER,
    )?;

    // Query for IPersistFile interface - 查询并获取 IPersistFile 接口实例
    let persist_file: IPersistFile = shell_link.QueryInterface()?;

    // Load the shortcut file (LNK file) - 载入快捷方式的文件
    persist_file.Load(&link_path, co::STGM::WRITE)?;

    // Open Log File - 打开日志文件
    let mut log_file = read_log().expect("Failed to open 'LinkEcho.log'");

    // Get data folder path - 获取数据文件夹路径
    let app_date =
        winsafe::SHGetKnownFolderPath(&co::KNOWNFOLDERID::LocalAppData, co::KF::NO_ALIAS, None)
            .map_or(String::new(), |path| format!("{path}\\LinkEcho"));

    // Create APP Date - 创建数据文件夹
    match Path::new(&app_date).try_exists() {
        Ok(true) => (),
        Ok(false) => std::fs::create_dir(&app_date)?,
        Err(err) => return Err(err.into()),
    };

    // Select an icon - 选择有图标的文件夹
    let mut icon_path = match FileDialog::new()
        .set_title("Please select an icon")
        .add_filter("ICONs", &["ico", "png", "svg", "webp", "tiff"])
        .pick_file()
    {
        Some(path_buf) => path_buf.to_string_lossy().to_string(),
        None => return Ok(None),
    };

    let icon_name = match Path::new(&icon_path).file_stem() {
        // Subsequent case-insensitive matching - 后续不区分大小写进行匹配
        Some(name) => name.to_string_lossy().to_string().trim().to_lowercase(),
        None => {
            write_log(&mut log_file, format!("Icon without name: {icon_path}"))?;
            return Err(format!("Icon without name: {icon_path}").into());
        }
    };

    if let Some(ext_os_str) = Path::new(&icon_path).extension() {
        if let Some(ext) = ext_os_str.to_str() {
            if ext.to_lowercase() != "ico" {
                let logo_path_buf = Path::new(&app_date).join(format!("{icon_name}.ico"));
                let logo_path_string = logo_path_buf.to_string_lossy().to_string();

                match logo_path_buf.try_exists() {
                    Ok(true) => icon_path = logo_path_string,
                    Ok(false) => {
                        match icongen::convert_ico(
                            PathBuf::from(&icon_path),
                            logo_path_buf,
                            &icon_name,
                        ) {
                            Ok(_) => write_log(
                                &mut log_file,
                                format!("Successfully converted {icon_name}.{ext} to ICO"),
                            )?,
                            Err(err) => {
                                write_log(
                                    &mut log_file,
                                    format!("Failed to convert {icon_name}.{ext} to ICO: {err}"),
                                )?;
                            }
                        };
                        icon_path = logo_path_string;
                    }
                    Err(err) => {
                        write_log(
                            &mut log_file,
                            format!("Error checking if {logo_path_string} exists: {err}"),
                        )?;
                    }
                };
            };
        };
    };

    // Set the icon location - 设置主题图标位置
    // let index = match theme {
    //     Theme::Dark => (),
    //     Theme::Light  => (),
    // };
    shell_link.SetIconLocation(&icon_path, 0)?;

    // Saves a copy of the object to the specified file - 将对象的副本保存到指定文件
    persist_file.Save(None, true)?;

    // Update the shortcut properties - 更新快捷方式属性
    if link_prop.icon_location != icon_path && icon_path != link_prop.path {
        if let Some(prop) = filter_link_prop {
            prop.icon_location = icon_path.clone();
            prop.status = Status::Changed;
        };
        link_prop.icon_location = icon_path.clone();
        link_prop.status = Status::Changed;
    } else {
        link_prop.icon_location = icon_path.clone();
        link_prop.status = Status::Unchanged;
    };

    write_log(
        &mut log_file,
        format!("Successfully change the shortcut icon:\n{link_path}\n{icon_path}"),
    )?;

    Ok(Some(icon_name.clone()))
}

pub fn restore_single_shortcut_icon(
    link_path: String,
    link_prop: &mut LinkProp,
    filter_link_prop: Option<&mut LinkProp>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 在main.rs里通知是否恢复默认，不恢复则返回Ok(None)
    //

    // Skip shortcuts that are not replaced or extend to uwp|app - 跳过未被更换图标或扩展为uwp|app的快捷方式
    if link_prop.status == Status::Unchanged || link_prop.target_ext == String::from("uwp|app") {
        return Ok(());
    };

    let icon_path = link_prop.target_path.clone();

    // Initialize COM library - 初始化 COM 库
    let _com_lib = winsafe::CoInitializeEx(
        // keep guard alive
        co::COINIT::APARTMENTTHREADED | co::COINIT::DISABLE_OLE1DDE,
    )?;

    // Create IShellLink object - 创建 IShellLink 对象实例
    let shell_link = winsafe::CoCreateInstance::<winsafe::IShellLink>(
        &co::CLSID::ShellLink,
        None,
        co::CLSCTX::INPROC_SERVER,
    )?;

    // Query for IPersistFile interface - 查询并获取 IPersistFile 接口实例
    let persist_file: IPersistFile = shell_link.QueryInterface()?;

    // Load the shortcut file (LNK file) - 载入快捷方式的文件
    persist_file.Load(&link_path, co::STGM::WRITE)?;

    // Open Log File - 打开日志文件
    let mut log_file = read_log().expect("Failed to open 'LinkEcho.log'");

    // Set the icon location - 设置图标位置
    shell_link.SetIconLocation(&icon_path, 0)?;

    // Saves a copy of the object to the specified file - 将对象的副本保存到指定文件
    persist_file.Save(None, true)?;

    // Update the shortcut properties - 更新快捷方式属性
    link_prop.icon_location = icon_path.clone();
    link_prop.status = Status::Unchanged;
    // 若为筛选的快捷方式，则更新所有项目中的该快捷方式属性
    if let Some(prop) = filter_link_prop {
        prop.icon_location = icon_path.clone();
        prop.status = Status::Unchanged;
    };

    write_log(
        &mut log_file,
        format!("Successfully restore the shortcut icon:\n{link_path}"),
    )?;

    Ok(())
}

pub fn clear_icon_cache() {
    let mut log_file = read_log().expect("Failed to open 'LinkEcho.log'");

    let local_app_data =
        winsafe::SHGetKnownFolderPath(&co::KNOWNFOLDERID::LocalAppData, co::KF::NO_ALIAS, None)
            .unwrap_or(match env::var_os("LocalAppData") {
                Some(path) => path.to_string_lossy().to_string(),
                None => return show_notify(&format!("Failed to get icon cache path")),
            });

    let explorer_path = Path::new(&local_app_data).join("Microsoft\\Windows\\Explorer");
    // let icon_cache_path = Path::new(&local_app_data).join("IconCache.db");

    // match icon_cache_path.try_exists() {
    //     Ok(true) => {
    //         if std::fs::remove_file(&icon_cache_path).is_err() {
    //             let text = format!("Failed to delete the conCache.db");
    //             write_log(&mut log_file, text.clone()).expect("Failure to write to the log");
    //             return show_notify(vec![&text]);
    //         }
    //     },
    //     Ok(false) => return show_notify(vec!["IconCache.d does not exist"]),
    //     Err(err) => return show_notify(vec![&format!("Failed get the IconCache.db path: {err}")]),
    // };

    match explorer_path.try_exists() {
        Ok(true) => {
            if explorer_path.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&explorer_path) {
                    for entry in entries {
                        let entry = entry.expect("");
                        let path = entry.path();

                        if path.is_file()
                            && path.extension().map_or(false, |ext| ext == "db")
                            && (path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .starts_with("iconcache_")
                                || path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .starts_with("thumbcache_"))
                        {
                            if std::fs::remove_file(&path).is_err() {
                                let text = format!(
                                    "Failed to delete the icon cache file\n{}",
                                    path.display()
                                );
                                write_log(&mut log_file, text.clone())
                                    .expect("Failure to write to the log");
                                return show_notify(&text);
                            }
                        }
                    }

                    let status = Command::new("PowerShell")
                        .args(["taskkill", "/IM", "explorer.exe", "/F;", "explorer"])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                        .expect("failed to execute process");

                    if status.success() {
                        write_log(
                            &mut log_file,
                            "Successfully cleared the icon cache".to_string(),
                        )
                        .expect("Failure to write to the log");
                        return show_notify("Successfully cleared the icon cache");
                    } else {
                        return show_notify("Failed to restart Explorer");
                    }
                } else {
                    return show_notify(
                        "Failed iterator to return entries in the explorer dir",
                    );
                }
            }
        }
        Ok(false) => return show_notify("Explorer directory does not exist"),
        Err(err) => return show_notify(&format!("Failed get the explorer directory: {err}")),
    }
}
