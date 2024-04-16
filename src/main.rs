// #![windows_subsystem = "windows"]       // 隐藏 CMD 和 Powershel

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::os::windows::process::CommandExt;
use std::collections::HashMap;
use core::cmp::Ordering;
use glob::glob;
use rfd::FileDialog;
use parselnk::Lnk;

struct LinkInfo {
    link_path:  String,
    link_target_path: String,
    link_target_dir: String,
    link_icon_location: String,
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
    find_link_files(&users_desktop_path, &mut link_map);

    println!("公共用户桌面快捷方式: {}", public_desktop_path.display());
    println!("");
    find_link_files(&public_desktop_path, &mut link_map);

    println!("当前用户的开始菜单快捷方式: {}", users_start_menu_path.display());
    println!("");
    find_link_files(&users_start_menu_path, &mut link_map);

    println!("公共用户的开始菜单快捷方式: {}", pubilc_start_menu_path.display());
    println!("");
    find_link_files(&pubilc_start_menu_path, &mut link_map);


    // 更换所有图标
    // match change_all_shortcuts_icons(&mut link_map) {
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

fn get_link_info(path_buf: &PathBuf) -> (String, String, String, String, String, String, String) {
    let link_name = path_buf.file_stem()      //  Option<&OsStr> -> OsStr -> Option<&str> -> &string
        .map_or_else(|| "unnamed_file".to_string()
        , |no_extension| no_extension.to_string_lossy().into_owned().trim().to_lowercase());

    let link_path = path_buf.to_string_lossy().into_owned();      //  PathBuf -> Cow<str> -> String ( to_string_lossy : 任何非 Unicode 序列都将替换为 �)

    let objlnk = Lnk::try_from(path_buf.as_path()).unwrap();

    let link_working_dir = objlnk.working_dir().unwrap_or_else(|| PathBuf::new()).to_string_lossy().into_owned();     // Option<PathBuf> -> PathBuf -> Cow<str> ->  String ( to_string_lossy : 任何非 Unicode 序列都将替换为 �)

    // 解决 parselnk 库不能获取 link_target_path 的问题
    let mut link_target_path = String::new();
    if let Some(have_string) = objlnk.link_info.local_base_path {
        link_target_path = have_string;
    } else if let Some(path_buf) = objlnk.relative_path() {
        link_target_path = link_working_dir.clone();
        for component in path_buf.components() {
            let _string = component.as_os_str().to_string_lossy().into_owned();
            if !_string.is_empty() && _string != ".." && !link_working_dir.contains(&_string) {
                link_target_path.push_str("\\");
                link_target_path.push_str(&_string.trim_start_matches('.'));
            }
        }
    } else {
        link_target_path = link_working_dir.clone();
    }

    let link_working_dir = if link_working_dir.is_empty() {
        Path::new(&link_target_path).parent().unwrap_or_else(|| Path::new("")).to_string_lossy().into_owned()    // Option<&OsStr> -> OsStr -> Cow<str> ->  String
    } else {
        link_working_dir
    };

    let link_icon_location = objlnk.string_data.icon_location.unwrap_or_else(|| PathBuf::new()).to_string_lossy().into_owned();     // Option<PathBuf> -> PathBuf -> Cow<str> ->  String ( to_string_lossy : 任何非 Unicode 序列都将替换为 �)
    
    let icon_index = objlnk.header.icon_index;      // u32，返回值不太对，比如ahk返回-108，它返回4294967188

    let link_ext = if Path::new(&link_target_path).exists() {
        Path::new(&link_target_path).extension()
            .map_or_else(|| "".to_string()
            , |os_str| os_str.to_string_lossy().into_owned().trim().to_lowercase())    // Option<&OsStr> -> OsStr -> Cow<str> ->  String
    } else {
        String::from("uwp/app")
    };

    (link_name, link_path, link_working_dir, link_target_path, link_icon_location, icon_index.to_string(), link_ext)
}

fn find_link_files(directory: &Path, link_map: &mut HashMap<(String, String), LinkInfo>) {
    let directory_pattern = format!(r"{}\**\*.lnk", directory.to_string_lossy());

    for path_buf in glob(&directory_pattern).unwrap().filter_map(Result::ok) {
        let (link_name, link_path, link_working_dir, link_target_path, link_icon_location, icon_index, link_ext) = get_link_info(&path_buf);

        println!("名称：{}, 扩展名：{}", link_name, link_ext);
        println!("{}", link_path);
        println!("目标目录：{}", link_working_dir);
        println!("目标位置：{}", link_target_path);
        println!("图标位置：{}", link_icon_location);
        println!("图标索引：{}", icon_index);
        println!("");

        // 排除重复项目
        if link_map.contains_key(&(link_name.clone(), link_ext.clone())) {
            continue;
        }

        // 添加至列表

        // 最后添加至 HashMap，&str会影响生命周期，所以用String了
        link_map.insert((link_name, link_ext), LinkInfo {
            link_path: link_path,
            link_target_path: link_target_path,
            link_target_dir: link_working_dir,
            link_icon_location: link_icon_location,
        });
    }
}

fn change_single_shortcut_icon(link_path: String, icon_path: String) -> Result<&'static str, String> {
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

fn change_all_shortcuts_icons(link_map: &mut HashMap<(String, String), LinkInfo>) -> Result<&'static str, String> {
    // 存储 PowerShell 命令
    let mut command = String::from(r#"$shell = New-Object -ComObject WScript.Shell"#);
    // 存储已匹配对象
    let mut match_same_vec = vec![];
    // 选择图标文件夹
    let select_icons_folder_path: String = match FileDialog::new()
        .set_title("请选择存放图标(.ico)的文件夹")
        .pick_folder() 
    {
        Some(path_buf) => path_buf.to_string_lossy().into_owned(),
        None =>  return Err("未选择文件夹".to_string()),
    };
    let select_icons_folder_path = format!(r"{}\**\*.ico", select_icons_folder_path);
    // 遍历文件夹图标
    for path_buf in glob(&select_icons_folder_path).unwrap().filter_map(Result::ok) {
        // 获取图标名称路径
        let icon_path = path_buf.to_string_lossy().into_owned();    // to_string_lossy：无非法符返回Cow::Borrowed(&str)，有非法符号返回Cow::Owned(String)
        let icno_name: String = match path_buf.file_stem() {
            Some(name) => name.to_string_lossy().into_owned().trim().to_lowercase(),
            None => return Err(format!("获取{}的图标名称失败", icon_path)),
        };
        let icno_name_len = icno_name.len();
        // 遍历 HashMap
        for (link_name, link_ext) in link_map.keys() {
            // 跳过已匹配对象
            if match_same_vec.contains(&(link_name, link_ext)) {
                continue;
            }
            // 匹配图标与lnk名称之间的包含关系
            match (link_name.len().cmp(&icno_name_len), link_name.cmp(&icno_name)) {
                (Ordering::Equal, Ordering::Equal) => match_same_vec.push((link_name, link_ext)),
                (Ordering::Greater, _)  if link_name.contains(&icno_name) => {},
                (Ordering::Less, _)  if icno_name.contains(&*link_name) => {},
                _ => continue,
            }
            // 跳过存在包含关系但已为使用图标情况
            if link_map.get(&(link_name.to_string(), link_ext.to_string())).unwrap().link_icon_location == icon_path {
                continue;
            };
            // 追加 PowerShell 命令
            let link_path = &link_map.get(&(link_name.to_string(), link_ext.to_string())).unwrap().link_path;
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
    let output = Command::new("powershell")
        .creation_flags(0x08000000)     // 隐藏控制台
        .args(&["-Command", &command])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .expect("Failed to execute PowerShell command");
    // 刷新图标
    // 日志记录
    // 刷新桌面
    // ...
    Ok("已更换桌面所有图标")
}

fn clear_date(link_map: &mut HashMap<(String, String), LinkInfo>) {
    link_map.clear();
}