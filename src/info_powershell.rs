use crate::{HashMap, Path, LinkInfo, glob};
use encoding::{Encoding, DecoderTrap};
use encoding::all::GBK;
use std::process::Command;
use std::time::Instant;

#[allow(unused)]
pub fn collect_link_info(folder_path: &str, link_map: &mut HashMap<(String, String), LinkInfo>) {
    let start_time = Instant::now();
    // link_map.clear();
    let mut command = String::from(r#"$shell = New-Object -ComObject WScript.Shell"#);
    let mut lines: Vec<String> = vec![];
    match folder_path {
        "desktop" => {
            command.push_str(r#"
                $shell.SpecialFolders.Item("AllUsersDesktop")
                $shell.SpecialFolders.Item("Desktop")
                "#)
        },
        "start_menu" => {
            command.push_str(r#"
                $shell.SpecialFolders.Item("AllUsersStartMenu")
                $shell.SpecialFolders.Item("StartMenu")
                "#)
        },
        _ => {
            match Path::new(folder_path).is_dir() {
                true => lines = vec![String::from(folder_path)],
                false => panic!("Error: Unable to determine {} path.", folder_path)
            }
        }
    }
    // 获取桌面或开始菜单路径
    if lines.is_empty() {
        // 执行命令
        let output = Command::new("powershell")
            .args(&["-Command", &command])
            .output()
            .expect("Failed to execute PowerShell command");
        // 重置命令
        command = String::from(r#"$shell = New-Object -ComObject WScript.Shell"#);
        // 尝试解码为UTF-8，若解码失败，则尝试使用GBK解码
        let output_u8 = match String::from_utf8(output.stdout.clone()) {
            Ok(u8_string) => u8_string,
            Err(_) => GBK.decode(&output.stdout, DecoderTrap::Strict).expect("GBK编码失败"),
        };
        // 收集路径
        lines = output_u8.lines().map(|line| line.to_string()).collect();
    }
    // 遍历目录中所有.lnk并追加命令
    for line in &lines {
        let subdirectories = format!(r"{}\**\*.lnk", line);
        for path_buf in glob(&subdirectories).unwrap().filter_map(Result::ok) {
            let link_name = path_buf.file_stem()      //  Option<&OsStr> -> OsStr -> Cow<str> -> String
                .map_or_else(|| String::from("unnamed_file")
                , |no_ext| no_ext.to_string_lossy().into_owned());

            let link_path = path_buf.to_string_lossy().into_owned();

            command.push_str(&format!(
                r#"
                Write-Host "{link_name}"
                Write-Host "{link_path}"
                $shortcut = $shell.CreateShortcut("{obj_path}")
                $shortcut.WorkingDirectory
                $shortcut.TargetPath
                $shortcut.IconLocation
                "#,
                link_name = link_name,
                link_path = link_path,
                obj_path = link_path,
            ));
        }
    }
    // 执行命令
    let output = Command::new("powershell")
        .args(&["-Command", &command])
        .output()
        .expect("Failed to execute PowerShell command");
    // 尝试解码为UTF-8，若解码失败，则尝试使用GBK解码
    let output_u8: String = match String::from_utf8(output.stdout.clone()) {
        Ok(u8_string) => u8_string,
        Err(_) => GBK.decode(&output.stdout, DecoderTrap::Strict).expect("GBK编码失败"),
    };
    // 使用lines()方法将字符串按行分割并转换为迭代器
    lines = output_u8.lines().map(|line| line.to_string()).collect::<Vec<_>>();
    // 使用chunks()方法将行按照每五行一个块进行分组
    for chunk in lines.chunks(5) {
        // 忽略不完整的块
        if chunk.len() != 5 {
            continue;
        }
        // 提取信息并填充到LinkInfo结构体中
        let link_name = chunk[0].to_string();
        let link_path = chunk[1].to_string();
        let mut link_target_dir = String::new();
        let mut link_target_path = String::new();
        let mut link_target_ext = String::new();
        let mut link_icon_location = String::new();
        let mut link_icon_index = String::new();
        let mut link_icon_status = String::new();
        // 快捷方式目标目录和目标路径
        match (chunk[2].is_empty(), chunk[3].is_empty()) {
            (true, false) => {
                link_target_dir = Path::new(&chunk[3]).parent().map_or(String::new(), |i| i.to_string_lossy().into_owned());
                link_target_path = chunk[3].to_string();
            },
            (false, false) => {
                link_target_dir = chunk[2].to_string();
                link_target_path = chunk[3].to_string();
            },
            (_, _) => {},
        }
        // 快捷方式目标扩展名
        if !Path::new(&link_target_path).is_file() {
            link_target_ext = String::from("uwp|app");
        } else {
            let target_file_name = Path::new(&link_target_path).file_name()
                .map_or_else(|| String::new(), 
                |name| name.to_string_lossy().into_owned());
            let target_extension = Path::new(&link_target_path).extension();    // Option<OsStr>
            match &*target_file_name {
                "schtasks.exe"   => {link_target_ext = String::from("schtasks")}, // 任务计划程序
                "explorer.exe"   => {link_target_ext = String::from("explorer")}, // 资源管理器
                "cmd.exe"        => {link_target_ext = String::from("cmd")},      // 命令提示符
                "powershell.exe" => {link_target_ext = String::from("psh")},      // PowerShell
                "wscript.exe"    => {link_target_ext = String::from("wscript")},  // WScript 对象
                "mstsc.exe"      => {link_target_ext = String::from("mstsc")},    // 远程连接
                "control.exe"    => {link_target_ext = String::from("control")},  // 控制面板
                _ => {
                    match (&target_extension, link_target_path.contains("WindowsSubsystemForAndroid")) {
                        (None, _) => {},
                        (Some(_), true) => {link_target_ext = String::from("app")},
                        (Some(os_str), false) => {link_target_ext = os_str.to_string_lossy().into_owned().to_lowercase()},
                    }
                }
            }
        };
        // 快捷方式图标路径、图标索引号
        let v: Vec<&str> = chunk[4].rsplitn(2, ',').collect();
        link_icon_index = v[0].to_string();
        link_icon_location = v[1].to_string();
        // 快捷方式是否已更换图标
        if Path::new(&link_icon_location).is_file() 
        && (link_target_path.is_empty()                                  // 排除UWP|APP情况
        || (link_icon_location != link_target_path                       // 排除图标源于目标
        && !link_icon_location.contains("WindowsSubsystemForAndroid")    // WSA应用
        && !link_icon_location.contains(&link_target_dir)))             // 图标位于工作目录下
        {
            link_icon_status = String::from("√")
        };

        let link_info = LinkInfo {
            link_path: link_path,
            link_target_dir: link_target_dir,
            link_target_path: link_target_path,
            link_icon_location: link_icon_location,
            link_icon_index: link_icon_index,
            link_icon_status: link_icon_status,
        };
        // 将结构体存储到HashMap中，使用链接名称作为键
        link_map.insert((link_name, link_target_ext), link_info);
    }
    let elapsed_time = start_time.elapsed();
    println!("Elapsed time: {:?}", elapsed_time);
}