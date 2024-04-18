// #![windows_subsystem = "windows"]       // 隐藏 CMD 和 Powershel
#[cfg(target_os = "windows")]

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::os::windows::process::CommandExt;
use std::collections::HashMap;
use glob::glob;
use rfd::FileDialog;
use parselnk::Lnk;

mod modify;
struct LinkInfo {
    link_path: String,
    link_target_path: String,
    link_target_dir: String,
    link_icon_location: String,
    link_has_been_changed: String,
}
fn main() {
    // 获取管理员权限

    // 存储快捷方式的属性
    let mut link_map: HashMap<(String, String), LinkInfo> = HashMap::new();     // Rc<RefCell<HashMap>>: 适于多函数修改，相对而言可避免不必要的复杂性和潜在的错误

    // 获取当前用户的"桌面文件夹"的完整路径
    let users_desktop_path = dirs::desktop_dir().expect("Failed to get desktop directory");

    // 获取公共用户的"桌面文件夹"的完整路径
    const PUBLIC_PATH: &str = r"C:\Users\Public\Desktop";
    let public_desktop_path = get_path_from_env("PUBLIC", PUBLIC_PATH);    // PathBuf

    // 获取当前用户的"开始菜单"的完整路径
    let users_start_menu_path = if let Some(user_profile) = env::var_os("APPDATA") {
        let start_menu_path = PathBuf::from(user_profile).join("Microsoft\\Windows\\Start Menu");
        if start_menu_path.is_dir() {
            start_menu_path
        } else {
            panic!("Error: Start menu directory does not exist or is not a directory.");
        }
    } else {
        panic!("Unable to determine APPDATA environment variable.");
    };

    // 获取公共用户的"开始菜单"的完整路径
    const PROGRAMDATA_PATH: &str = r"C:\ProgramData\Microsoft\Windows\Start Menu";
    let pubilc_start_menu_path = get_path_from_env("PROGRAMDATA", PROGRAMDATA_PATH);    // PathBuf


    println!{"当前用户桌面快捷方式: {}", users_desktop_path.display()};
    println!("");
    collect_link_info_in_folder(&users_desktop_path, &mut link_map);

    println!("公共用户桌面快捷方式: {}", public_desktop_path.display());
    println!("");
    collect_link_info_in_folder(&public_desktop_path, &mut link_map);

    println!("当前用户的开始菜单快捷方式: {}", users_start_menu_path.display());
    println!("");
    collect_link_info_in_folder(&users_start_menu_path, &mut link_map);

    println!("公共用户的开始菜单快捷方式: {}", pubilc_start_menu_path.display());
    println!("");
    collect_link_info_in_folder(&pubilc_start_menu_path, &mut link_map);

    // clear_thumbnails()

    // 更换所有图标
    // match modify::change_all_links_icons(&mut link_map) {
    //     Ok(yes) => println!("{}", yes),
    //     Err(error) => println!("{}", error),
    // }
}

// 获取环境变量路径，排除主盘非C盘的情况
fn get_path_from_env(var_name: &str, default_path: &str) -> PathBuf {
    let default_path = PathBuf::from(default_path);
    if default_path.is_dir() {
        return default_path;
    }

    match env::var_os(var_name) {
        Some(path_buf) => {
            let sub_path = if var_name == "PUBLIC" {
                Path::new(&path_buf).join("Desktop").to_path_buf()
            } else {
                Path::new(&path_buf).join("Microsoft\\Windows\\Start Menu").to_path_buf()
            };
            if sub_path.is_dir() {
                sub_path.to_path_buf()
            } else {
                panic!("Error: Unable to determine {} path.", var_name);
            }
        }
        None => panic!("Error: Unable to determine {} path.", var_name),
    }
}

fn get_link_info(path_buf: &PathBuf) -> (String, String, String, String, String, String, String, String) {
    let link_name = path_buf.file_stem()      //  Option<&OsStr> -> OsStr -> Option<&str> -> &string
        .map_or_else(|| String::from("unnamed_file")
        , |no_ext| no_ext.to_string_lossy().into_owned());
    // 快捷方式实际路径
    let link_path = path_buf.to_string_lossy().into_owned();      // to_string_lossy : 任何非 Unicode 序列都将替换为 �
    // 引用 parselnk 库
    let objlnk = Lnk::try_from(path_buf.as_path()).unwrap();
    // 快捷方式目标目录和目标路径
    let mut link_working_dir = String::new();
    let mut link_target_path = String::new();
    match (objlnk.working_dir(), objlnk.relative_path(), objlnk.link_info.local_base_path) {
        (Some(work_buf), _, Some(base_buf)) => {
            link_working_dir = work_buf.to_string_lossy().into_owned();
            link_target_path = base_buf;
        },
        (None, _, Some(base_buf)) => {
            link_working_dir = Path::new(&base_buf).parent().unwrap().to_string_lossy().into_owned();
            link_target_path = base_buf;
        },
        (Some(work_buf), Some(relative_buf), None) => {
            link_working_dir = work_buf.to_string_lossy().into_owned();
            // 排除目标目录末端是 "\" 的情况
            if link_working_dir.ends_with("\\") {
                link_working_dir = link_working_dir.trim_end_matches("\\").to_string()
            }
            link_target_path = link_working_dir.clone();
            for component in relative_buf.components() {
                let _string = component.as_os_str().to_string_lossy().into_owned();
                if !_string.is_empty() && _string != ".." && !link_working_dir.contains(&_string) {
                    link_target_path.push_str("\\");
                    link_target_path.push_str(&_string.trim_start_matches('.'));
                }
            }
        },
        _ => {},
    };
    // 快捷方式图标路径、是否被更换    
    let mut link_icon_location = String::new();
    let mut link_has_been_changed = String::new();
    match objlnk.string_data.icon_location {
        Some(path_buf) => {
            link_icon_location = path_buf.to_string_lossy().into_owned();
            if path_buf.is_file()
            && link_icon_location != link_target_path
            && !link_icon_location.contains("WindowsSubsystemForAndroid")
            && path_buf.parent().unwrap() != Path::new(&link_working_dir)   // 父目
            {
                link_has_been_changed = String::from("√");
            };
        }, 
        None => {},
    };
    // 快捷方式图标索引号
    let icon_index = objlnk.header.icon_index;      // u32，返回值不太对，比如ahk返回-108，它返回4294967188
    // 快捷方式目标扩展名
    let mut link_target_ext = String::new();
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
                match &target_extension {
                    None => {},
                    Some(_) if link_target_path.contains("WindowsSubsystemForAndroid") => {link_target_ext = String::from("app")},
                    Some(os_str) => {link_target_ext = os_str.to_string_lossy().into_owned().to_lowercase()},
                }
            }
        }
    };

    (link_name, link_path, link_working_dir, link_target_path, link_target_ext, link_icon_location, icon_index.to_string(), link_has_been_changed)
}

fn collect_link_info_in_folder(directory: &Path, link_map: &mut HashMap<(String, String), LinkInfo>) {
    let directory_pattern = format!(r"{}\**\*.lnk", directory.to_string_lossy());

    for path_buf in glob(&directory_pattern).unwrap().filter_map(Result::ok) {
        let (link_name, link_path, link_working_dir, link_target_path, link_target_ext, link_icon_location, icon_index, link_has_been_changed) = get_link_info(&path_buf);

        // println!("名称：{}, 扩展名：{}", link_name, link_target_ext);
        // println!("{}", link_has_been_changed);
        // println!("{}", link_path);
        // println!("目标目录：{}", link_working_dir);println!("目标位置：{}", link_target_path);
        // println!("图标位置：{}", link_icon_location);println!("图标索引：{}", icon_index)
        // ;println!("");

        // 排除重复项目
        if link_map.contains_key(&(link_name.clone(), link_target_ext.clone())) {
            continue;
        }

        // 添加至列表

        // 最后添加至 HashMap，&str会影响生命周期，所以用String
        link_map.insert((link_name, link_target_ext), LinkInfo {
            link_path: link_path,
            link_target_path: link_target_path,
            link_target_dir: link_working_dir,
            link_icon_location: link_icon_location,
            link_has_been_changed: link_has_been_changed,
        });
    }
}







fn clear_date(link_map: &mut HashMap<(String, String), LinkInfo>) {
    link_map.clear();
}

fn clear_thumbnails() {
    // https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cleanmgr
    Command::new("cmd")
        .creation_flags(0x08000000)     // 隐藏控制台
        .args(&["/c", r#"cleanmgr"#])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .expect("cmd exec error!");

    // Choose C: and press OK.
    // 请选择C盘，并点击OK.

    // 选择缩略图选项，取消其他所有选项，然后点击OK并确认删除
    // Uncheck all the entries except Thumbnails. Click OK and click Delete Files to confirm

    // 重启资源管理器
}