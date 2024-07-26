use std::os::windows::process::CommandExt;
use std::process::Command;
use crate::{glob, utils::{open_log_file, write_log}, FileDialog, LinkProp, Status};
use winsafe::{co, prelude::*, IPersistFile};

pub fn change_all_shortcuts_icons(link_vec: &mut Vec<LinkProp>) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize COM library - 初始化 COM 库
    let _com_lib = winsafe::CoInitializeEx( // keep guard alive
        co::COINIT::APARTMENTTHREADED
        | co::COINIT::DISABLE_OLE1DDE,
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
    let mut log_file = open_log_file().expect("Failed to open 'LinkEcho.log'");

    // Select the folder with the icons - 选择有图标的文件夹
    let select_icons_folder_path = match FileDialog::new()
        .set_title("Please select the folder where the icons are stored")
        .pick_folder() {
            Some(path_buf) => format!(r"{}\**\*.ico", path_buf.to_string_lossy().into_owned()),
            None => return Ok(()),
        };

    // Iterate through the folder of icons - 遍历快捷方式目录中的图标（包括子目录）
    for path_buf in glob(&select_icons_folder_path).unwrap().filter_map(Result::ok) {
        // Get icon name and path
        let icon_path = path_buf.to_string_lossy().into_owned();
        let icno_name: String = match path_buf.file_stem() {
            Some(name) => name.to_string_lossy().into_owned().trim().to_lowercase(),    // Subsequent case-insensitive matching - 后续不区分大小写进行匹配
            None => return Err(format!("Icon without name: {}", icon_path).into()),
        };

        // Iterate over the vec that stores the shortcut properties - 遍历快捷方式的属性
        for link_prop in link_vec.iter_mut() {
            // Skip matched objects - 跳过已匹配的对象
            if match_same_vec.contains(&link_prop.path) {   
                continue;
            };

            // Compare the containment relationship between two strings - 比较图标名称与LNK名称的包含关系
            let link_name_lowercase = &link_prop.name.trim().to_lowercase();
            match (link_name_lowercase.contains(&icno_name), icno_name.contains(link_name_lowercase)) {
                (false, false) => continue,
                (true, true) => match_same_vec.push(link_prop.path.clone()),
                _ => ()
            };

            // Skip cases with identical icons - 跳过图标相同的情况
            if link_prop.icon_location == icon_path {
                continue;
            } else {    // Updating the icon path of a LinkProp structure - 更新LinkProp结构体的图标路径
                link_prop.icon_location = String::from(icon_path.clone());
                link_prop.status = Status::Changed;
            };
            
            // Load the shortcut file (LNK file) - 载入快捷方式的文件
            if let Err(_) = persist_file.Load(&link_prop.path, co::STGM::WRITE) {
                write_log(&mut log_file, format!("Failed to load shortcut file: {}", &link_prop.path))?;
                continue
            }

            // Set the icon location - 设置图标位置
            if let Err(_) = shell_link.SetIconLocation(&icon_path, 0) {
                write_log(&mut log_file, format!("Failed to set the icon location:\n{}\n{}", &link_prop.path, icon_path))?;
                continue
            }

            // Save a copy of the object to the specified file - 将对象的副本保存到指定文件
            match persist_file.Save(None, true) {
                Ok(_) => write_log(&mut log_file, format!("Successfully set the icon location:\n{}\n{}", &link_prop.path, icon_path))?,
                Err(err) => {
                    write_log(&mut log_file, format!("Failed to save a copy of the object to the specified file:\n{}\n{}", &link_prop.path, err))?;
                    continue
                }
            };
        }
    }
    // 刷新图标
    // 刷新桌面
    Ok(())
}

pub fn restore_all_shortcuts_icons(link_vec: &mut Vec<LinkProp>) -> Result<(), Box<dyn std::error::Error>> {
    // 在main.rs里通知是否恢复所有默认

    // Initialize COM library - 初始化 COM 库
    let _com_lib = winsafe::CoInitializeEx( // keep guard alive
        co::COINIT::APARTMENTTHREADED
        | co::COINIT::DISABLE_OLE1DDE,
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
    let mut log_file = open_log_file().expect("Failed to open 'LinkEcho.log'");

    // Iterate over the vec that stores the shortcut properties - 遍历快捷方式的属性
    for link_prop in link_vec.iter_mut() {
        // Skip shortcuts that are not replaced or extend to uwp|app - 跳过未被更换图标或扩展为uwp|app的快捷方式
        if link_prop.status == Status::Unchanged 
            || link_prop.target_ext == String::from("uwp|app") {
            continue;
        }
        let icon_path = &link_prop.target_path;

        // Load the shortcut file (LNK file) - 载入快捷方式的文件
        if let Err(_) = persist_file.Load(&link_prop.path, co::STGM::WRITE) {
            write_log(&mut log_file, format!("Failed to load shortcut file: {}", &link_prop.path))?;
            continue
        }

        // Set the icon location - 设置图标位置
        if let Err(_) = shell_link.SetIconLocation(&icon_path, 0) {
            write_log(&mut log_file, format!("Failed to set the icon location:\n{}\n{}", &link_prop.path, icon_path))?;
            continue
        }

        // Saves a copy of the object to the specified file - 将对象的副本保存到指定文件
        match persist_file.Save(None, true) {
            Ok(_) => write_log(&mut log_file, format!("Successfully restore the default icon:\n{}\n{}", &link_prop.path, icon_path))?,
            Err(_) => {
                write_log(&mut log_file, format!("Failed to save a copy of the object to the specified file:\n{}", &link_prop.path))?;
                continue
            }
        };

        // Update the icon path and icon status in the LinkProp structure - 更新LinkProp结构体
        link_prop.icon_location = icon_path.clone();
        link_prop.status = Status::Unchanged;

        // 刷新列表
    };

    // 刷新
    Ok(())
}

pub fn change_single_shortcut_icon(link_path: String, link_prop: &mut LinkProp) -> Result<(), Box<dyn std::error::Error>> {
    // 在main.rs里通知是否恢复默认

    // Initialize COM library - 初始化 COM 库
    let _com_lib = winsafe::CoInitializeEx( // keep guard alive
        co::COINIT::APARTMENTTHREADED
        | co::COINIT::DISABLE_OLE1DDE,
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
    let mut log_file = open_log_file().expect("Failed to open 'LinkEcho.log'");

    // Select an icon - 选择有图标的文件夹
    let select_icon_path = match FileDialog::new()
        .set_title("Please select an icon")
        .add_filter("ICO File", &["ico"])
        .pick_file() {
            Some(path_buf) => path_buf.to_string_lossy().into_owned(),
            None => return Ok(()),
        };

    // Set the icon location - 设置图标位置
    shell_link.SetIconLocation(&select_icon_path, 0)?;

    // Saves a copy of the object to the specified file - 将对象的副本保存到指定文件
    persist_file.Save(None, true)?;

    // Update the shortcut properties - 更新快捷方式属性
    if link_prop.icon_location != select_icon_path && select_icon_path != link_prop.path {
        link_prop.icon_location = select_icon_path.clone();
        link_prop.status = Status::Changed;
    } else {
        link_prop.icon_location = select_icon_path.clone();
        link_prop.status = Status::Unchanged;
    };

    write_log(&mut log_file, format!("Successfully change the shortcut icon:\n{}\n{}", &link_path, select_icon_path))?;

    Ok(())
}

pub fn restore_single_shortcut_icon(link_path: String, link_prop: &mut LinkProp) -> Result<(), Box<dyn std::error::Error>> {
    // Skip shortcuts that are not replaced or extend to uwp|app - 跳过未被更换图标或扩展为uwp|app的快捷方式
    if link_prop.status == Status::Unchanged || link_prop.target_ext == String::from("uwp|app") {
        return Ok(())
    };

    let icon_path = link_prop.target_path.clone();
    
    // Initialize COM library - 初始化 COM 库
    let _com_lib = winsafe::CoInitializeEx( // keep guard alive
        co::COINIT::APARTMENTTHREADED
        | co::COINIT::DISABLE_OLE1DDE,
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
    let mut log_file = open_log_file().expect("Failed to open 'LinkEcho.log'");

    // Set the icon location - 设置图标位置
    shell_link.SetIconLocation(&icon_path, 0)?;

    // Saves a copy of the object to the specified file - 将对象的副本保存到指定文件
    persist_file.Save(None, true)?;

    // Update the shortcut properties - 更新快捷方式属性
    link_prop.icon_location = icon_path.clone();
    link_prop.status = Status::Unchanged;

    write_log(&mut log_file, format!("Successfully restore the shortcut icon:\n{}\n{}", &link_path, icon_path))?;

    Ok(())
}


pub fn clear_thumbnails() -> Result<(), Box<dyn std::error::Error>> {
    // https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cleanmgr
    let output = Command::new("cmd")
        .creation_flags(0x08000000)     // 隐藏控制台
        .args(&["/c", r#"cleanmgr"#])
        .output()
        .map_err(|e| format!("Failed to execute process: {}", e))?;
    if !output.status.success() {
        return Err(format!(
            "Failed to execute command: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(())
    // Choose C, and press OK - 请选择C盘，并点击OK
    
    // Uncheck all the entries except Thumbnails - 选择缩略图选项，取消其他所有选项

    // Click OK and click Delete Files to confirm - 点击OK并确认删除

    // Restart Explorer - 重启资源管理器
}