use std::os::windows::process::CommandExt;
use std::time::Instant;
use std::process::{Command, Stdio};
use crate::{LinkProp, FileDialog, glob};

#[allow(unused)]
pub fn change_all_links_icons(link_vec: &mut Vec<LinkProp>) -> Result<&'static str, String> {
    // 存储 PowerShell 命令
    let mut command = String::from(r#"$shell = New-Object -ComObject WScript.Shell"#);
    // 存储已匹配对象
    let mut match_same_vec = vec![];
    // 选择图标文件夹
    let select_icons_folder_path: String = match FileDialog::new()
        .set_title("Please select the folder where the icons are stored")
        .pick_folder() {
        Some(path_buf) => format!(r"{}\**\*.ico", path_buf.to_string_lossy().into_owned()),
        None => return Err("No folder selected".to_string()),
    };
    // 遍历文件夹图标
    for path_buf in glob(&select_icons_folder_path).unwrap().filter_map(Result::ok) {
        // 获取图标名称路径
        let icon_path = path_buf.to_string_lossy().into_owned();    // to_string_lossy：无非法符返回Cow::Borrowed(&str)，有非法符号返回Cow::Owned(String)
        let icno_name: String = match path_buf.file_stem() {
            Some(name) => name.to_string_lossy().into_owned().trim().to_lowercase(),    // 这里用小写来后续匹配
            None => return Err(format!("Icon without name: {}", icon_path)),
        };
        // 遍历 HashMap
        for link_prop in link_vec.iter_mut() {
            let lowercase_name = link_prop.name.clone().trim().to_lowercase();
            // 跳过已匹配对象
            if match_same_vec.contains(&link_prop.path) {   
                continue;
            }
            // 比较图标名称与LNK名称的包含关系
            match (lowercase_name.contains(&icno_name), icno_name.contains(&lowercase_name)) {
                (false, false) => continue,
                (true, true) => match_same_vec.push(link_prop.path.to_string()),
                _ => ()
            }
            // 跳过相等关系且已使用该图标情况
            if link_prop.icon_location == icon_path {
                continue;
            };
            // 更新LinkProp结构体的图标路径
            link_prop.icon_location = String::from(icon_path.clone());
            link_prop.icon_status = String::from("√");
            // 追加 PowerShell 命令
            command.push_str(&format!(
                r#"
                $shortcut = $shell.CreateShortcut("{link_path}")
                $shortcut.IconLocation = "{icon_path}"
                $shortcut.Save()
                "#,
                link_path = &link_prop.path,
                icon_path = icon_path
            ));
            println!("{}\n{}\n", link_prop.name, icno_name);
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
    Ok("Successfully changed the icons of all shortcuts!")
}

#[allow(unused)]
pub fn restore_all_links_icons(link_vec: &mut Vec<LinkProp>) -> Result<&'static str, String> {
    // 通知是否恢复所有默认

    // 存储 PowerShell 命令
    let mut command = String::from(r#"$shell = New-Object -ComObject WScript.Shell"#);
    // 遍历link_vec
    for link_prop in link_vec.iter_mut() {
        // 判断是否更换过或扩展为uwp|app
        if link_prop.icon_status.is_empty() || link_prop.target_ext == "uwp|app" {
            continue;
        }
        // 追加命令
        command.push_str(&format!(
            r#"
            $shortcut = $shell.CreateShortcut("{link_path}")
            $shortcut.IconLocation = "{icon_path}"
            $shortcut.Save()
            "#,
            link_path = &link_prop.path,
            icon_path = &link_prop.target_path
        ));
        // 更新LinkProp结构体的图标路径和更换标记
        link_prop.icon_location = String::from(link_prop.target_path.clone());
        link_prop.icon_status = String::new();
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

    Ok("Restore default icons for all shortcut icons")
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

    Ok("The icon of the shortcut has been changed")
}


// fn clear_date(link_vec: &mut Vec<LinkProp>) {
//     link_vec.clear();
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