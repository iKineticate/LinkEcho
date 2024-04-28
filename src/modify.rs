use std::os::windows::process::CommandExt;
use std::time::Instant;
use core::cmp::Ordering;
use std::process::{Command, Stdio};
use crate::{LinkInfo, HashMap, FileDialog, glob};

#[allow(unused)]
pub fn change_all_links_icons(link_map: &mut HashMap<(String, String), LinkInfo>) -> Result<&'static str, String> {
    // 存储 PowerShell 命令
    let mut command = String::from(r#"$shell = New-Object -ComObject WScript.Shell"#);
    // 存储已匹配对象
    let mut match_same_vec = vec![];
    // 选择图标文件夹
    let select_icons_folder_path: String = match FileDialog::new()
        .set_title("请选择存放图标(.ico)的文件夹")
        .pick_folder() 
    {
        Some(path_buf) => format!(r"{}\**\*.ico", path_buf.to_string_lossy().into_owned()),
        None => return Err("未选择文件夹".to_string()),
    };
    // 遍历文件夹图标
    for path_buf in glob(&select_icons_folder_path).unwrap().filter_map(Result::ok) {
        // 获取图标名称路径
        let icon_path = path_buf.to_string_lossy().into_owned();    // to_string_lossy：无非法符返回Cow::Borrowed(&str)，有非法符号返回Cow::Owned(String)
        let icno_name: String = match path_buf.file_stem() {
            Some(name) => name.to_string_lossy().into_owned().trim().to_lowercase(),    // 这里用小写来后续匹配
            None => return Err(format!("获取{}的图标名称失败", icon_path)),
        };
        // 遍历 HashMap
        for ((link_name, link_target_ext), link_info) in link_map.iter_mut() {  // iter_mut: 键的不可变引用和值的可变引用
            let lowercase_name = link_name.clone().trim().to_lowercase();
            // 跳过已匹配对象
            if match_same_vec.contains(&(link_name.clone(), link_target_ext.clone())) {
                continue;
            }
            // 匹配图标与lnk名称之间的包含关系
            match lowercase_name.chars().count().cmp(&icno_name.chars().count()) {
                Ordering::Equal if lowercase_name == icno_name => match_same_vec.push((link_name.to_string(), link_target_ext.to_string())),
                Ordering::Greater if lowercase_name.contains(&icno_name) => {},
                Ordering::Less if icno_name.contains(&lowercase_name) => {},
                _ => continue,
            }
            // 跳过相等关系且已使用该图标情况
            if link_info.link_icon_location == icon_path {
                continue;
            };
            // 更新LinkInfo结构体的图标路径
            link_info.link_icon_location = String::from(icon_path.clone());
            link_info.link_icon_status = String::from("√");
            // 追加 PowerShell 命令
            let link_path = &link_info.link_path;
            command.push_str(&format!(
                r#"
                $shortcut = $shell.CreateShortcut("{link_path}")
                $shortcut.IconLocation = "{icon_path}"
                $shortcut.Save()
                "#,
                link_path = link_path,
                icon_path = icon_path
            ));
            println!("{}", link_name);
            println!("{}", icno_name);
            println!("");
        }
    }
    // 执行 Powershell 命令: 更换图标
    let start_time = Instant::now();
    Command::new("powershell")
        .creation_flags(0x08000000)     // 隐藏控制台
        .args(&["-Command", &command])
        .stdout(Stdio::null())          // 丢弃子进程的输出
        .stderr(Stdio::null())          // 丢弃子进程的错误输出
        .output()
        .expect("Failed to execute PowerShell command");
    // 刷新图标
    // 日志记录
    // 刷新桌面
    let elapsed_time = start_time.elapsed();
    println!("Elapsed time: {:?}", elapsed_time);
    Ok("已更换桌面所有图标")
}

#[allow(unused)]
pub fn restroe_all_links_icons(link_map: &mut HashMap<(String, String), LinkInfo>) -> Result<&'static str, String> {
    // 通知是否恢复所有默认

    // 存储 PowerShell 命令
    let mut command = String::from(r#"$shell = New-Object -ComObject WScript.Shell"#);
    // 遍历link_map
    for ((_, link_target_ext), link_info) in link_map.iter_mut() {
        // 判断是否更换过或扩展为uwp|app（给link_map添加is_change:Y/N)
        if link_info.link_icon_status.is_empty() 
        || link_target_ext == "uwp|app" 
        || !std::path::Path::new(&link_info.link_target_path).is_file() {
            continue;
        }
        // 追击命令
        let link_path = &link_info.link_path;
        let icon_path = &link_info.link_target_path;
        command.push_str(&format!(
            r#"
            $shortcut = $shell.CreateShortcut("{link_path}")
            $shortcut.IconLocation = "{icon_path}"
            $shortcut.Save()
            "#,
            link_path = link_path,
            icon_path = icon_path
        ));
        // 更新LinkInfo结构体的图标路径和更换标记
        link_info.link_icon_location = String::from(icon_path.clone());
        link_info.link_icon_status = String::new();
        // 若更换过图标，则更新更换的显示数据
        // 刷新列表
    };
    // 执行 Powershell 命令: 恢复默认图标
    Command::new("powershell")
        .creation_flags(0x08000000)
        .args(&["-Command", &command])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .expect("Failed to execute PowerShell command");
    // 刷新

    Ok("所有快捷方式图标恢复默认图标")
}

#[allow(unused)]
pub fn change_single_shortcut_icon(link_path: String, icon_path: String) -> Result<&'static str, String> {
    let command = String::from(&format!(
        r#"
        $shell = New-Object -ComObject WScript.Shell
        $shortcut = $shell.CreateShortcut("{link_path}")
        $shortcut.IconLocation = "{icon_path}"
        $shortcut.Save()
        "#,
        link_path = link_path,
        icon_path = icon_path,
    ));

    Command::new("powershell")
        .creation_flags(0x08000000)     // 隐藏控制台
        .args(&["-Command", &command])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .expect("Failed to execute PowerShell command");

    Ok("已更换图标")
}


// fn clear_date(link_map: &mut HashMap<(String, String), LinkInfo>) {
//     link_map.clear();
// }

// fn clear_thumbnails() {
//     // https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cleanmgr
//     Command::new("cmd")
//         .creation_flags(0x08000000)     // 隐藏控制台
//         .args(&["/c", r#"cleanmgr"#])
//         .stdout(Stdio::null())
//         .stderr(Stdio::null())
//         .output()
//         .expect("cmd exec error!");

//     // Choose C: and press OK.
//     // 请选择C盘，并点击OK.

//     // 选择缩略图选项，取消其他所有选项，然后点击OK并确认删除
//     // Uncheck all the entries except Thumbnails. Click OK and click Delete Files to confirm

//     // 重启资源管理器
// }