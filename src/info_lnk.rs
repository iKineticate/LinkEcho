use crate::{HashMap, Path, PathBuf, env, LinkInfo, ShowInfo, glob};
use lnk::ShellLink;

#[allow(unused)]
// 获取环境变量路径，排除主盘非C盘的情况
pub fn get_path_from_env(name: &str) -> PathBuf {
    let mut default_path = PathBuf::new();
    let mut var_name = String::new();
    let mut relative_path: &str = "";
    match name {
        "USERS_DESKTOP" => {
            var_name = String::from("HOMEPATH");
            relative_path = "Desktop";
            default_path = dirs::desktop_dir().unwrap_or(PathBuf::new())
        },
        "PUBLIC_DESKTOP" => {
            var_name = String::from("PUBLIC"); 
            relative_path = "Desktop";
            default_path = PathBuf::from(r"C:\Users\Public\Desktop")
        },
        "USERS_START_MENU" => {
            var_name = String::from("APPDATA");
            relative_path = r"Microsoft\Windows\Start Menu"
        },
        "PUBLIC_START_MENU" => {
            var_name = String::from("PROGRAMDATA"); 
            relative_path = r"Microsoft\Windows\Start Menu";
            default_path = PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu")
        },
        _ => panic!("Error: Unable to determine {} path.", name)
    }

    if !default_path.is_dir() {
        match env::var_os(&var_name) {
            Some(path_buf) => Path::new(&path_buf).join(relative_path).to_path_buf(),
            None => panic!("Error: Unable to determine {} path.", name),
        }
    } else {
        default_path
    }
}


fn get_link_info(path_buf: &PathBuf) -> (String, String, String, String, String, String, String, String) {
    let link_name = path_buf.file_stem()      //  Option<&OsStr> -> OsStr -> Cow<str> -> String
        .map_or_else(|| String::from("unnamed_file")
        , |no_ext| no_ext.to_string_lossy().into_owned());
    // 快捷方式实际路径
    let link_path = path_buf.to_string_lossy().into_owned();      // to_string_lossy : 任何非 Unicode 序列都将替换为 �
    // 引用 lnk 库
    let objlnk = lnk::ShellLink::open(&link_path).unwrap();
    // 快捷方式目标目录和目标路径
    let link_relative_path = objlnk.relative_path().clone().unwrap_or(String::new());
    let mut link_target_dir = objlnk.working_dir().clone().unwrap_or(String::new());
    let mut link_target_path = objlnk.link_info().clone().map_or(String::new(), |i| i.local_base_path().clone().unwrap_or(String::new()));
    match (link_target_dir.is_empty(), link_target_path.contains('�')) {
        // 当link_target_path存在无效字符串时，使用相对和目代替
        (false, true) if !link_relative_path.is_empty() => {
            // 排除目标目录末端是 "\" 的情况
            if link_target_dir.ends_with("\\") {
                link_target_dir = link_target_dir.trim_end_matches("\\").to_string()
            }
            link_target_path = link_target_dir.clone();
            for component in Path::new(&link_relative_path).components() {
                let path_string = component.as_os_str().to_string_lossy().into_owned();
                if !path_string.is_empty() && path_string != ".." && !link_target_dir.contains(&path_string) {
                    link_target_path = format!("{}\\{}", link_target_path, path_string);
                }
            }
        },
        (true, false) if !link_target_path.is_empty() => {
            link_target_dir = Path::new(&link_target_path).parent().map_or(String::new(), |i| i.to_string_lossy().into_owned());
        },
        _ => {},
    };
    // 快捷方式图标路径、是否被更换    
    let mut link_icon_location = String::new();
    let mut link_icon_status = String::new();
    match &objlnk.icon_location() {
        Some(path_string) => {
            link_icon_location = path_string.to_string();
            if Path::new(path_string).is_file() && (link_target_path.is_empty()            // 排除UWP|APP情况
            || (link_icon_location != link_target_path                       // 排除图标源于目标
            && !link_icon_location.contains("WindowsSubsystemForAndroid")    // WSA应用
            && !link_icon_location.contains(&link_target_dir)))             // 图标位于工作目录下
            {
                link_icon_status = String::from("√");
            };
        }, 
        None => {},
    };
    // 快捷方式图标索引号
    let link_icon_index = objlnk.header().icon_index().to_string();
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
            "wscript.exe"    => {link_target_ext = String::from("wscript")},  // 脚本
            "cscript.exe"    => {link_target_ext = String::from("cscript")},  // 脚本
            "mstsc.exe"      => {link_target_ext = String::from("mstsc")},    // 远程连接
            "regsvr32.exe"   => {link_target_ext = String::from("regsvr32")}, // 注册COM组件
            "rundll32.exe"   => {link_target_ext = String::from("rundll32")}, // 执行32位的DLL文件
            "mshta.exe"      => {link_target_ext = String::from("mshta")},    // 执行.HTA文件
            "msiexec.exe"    => {link_target_ext = String::from("msiexec")},  // 安装Windows Installer安装包(MSI)
            "control.exe"    => {link_target_ext = String::from("control")},  // 控制面板执行
            "msdt.exe"       => {link_target_ext = String::from("msdt")},     // Microsoft 支持诊断工具
            "wmic.exe"       => {link_target_ext = String::from("wmic")},     // WMI 命令行
            _ => {
                match (&target_extension, link_target_path.contains("WindowsSubsystemForAndroid")) {
                    (None, _) => {},
                    (Some(_), true) => {link_target_ext = String::from("app")},
                    (Some(os_str), false) => {link_target_ext = os_str.to_string_lossy().into_owned().to_lowercase()},
                }
            }
        }
    };

    (link_name, link_path, link_target_dir, link_target_path, link_target_ext, link_icon_location, link_icon_index, link_icon_status)
}

// #[allow(unused)]
pub fn collect_link_info_in_folder(directory: &Path, link_map: &mut HashMap<(String, String), LinkInfo>, show_info: &mut Vec<ShowInfo>) {
    let directory_pattern = format!(r"{}\**\*.lnk", directory.to_string_lossy());
    for path_buf in glob(&directory_pattern).unwrap().filter_map(Result::ok) {
        let (link_name, link_path, link_target_dir, link_target_path, link_target_ext, link_icon_location, link_icon_index, link_icon_status) = get_link_info(&path_buf);
        // 排除重复项目
        if link_map.contains_key(&(link_name.clone(), link_target_ext.clone())) {
            continue;
        }
        // 添加至命令行列表
        let show_name = if link_name.chars().count() > 30 {
            link_name.chars().take(28).collect::<String>() + "..."
        } else {
            link_name.clone()
        };
        show_info.push(ShowInfo{name: show_name, types: link_target_ext.clone(), status: link_icon_status.clone()});
        // 最后添加至 HashMap，&str会影响生命周期，所以用String
        link_map.insert((link_name, link_target_ext), LinkInfo {
            link_path,
            link_target_path,
            link_target_dir,
            link_icon_location,
            link_icon_index,
            link_icon_status,
        });
    }
}




