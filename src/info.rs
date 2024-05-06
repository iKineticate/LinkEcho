use crate::{HashMap, Path, PathBuf, env, LinkInfo, glob};

pub struct SystemLinkDirs<'a>(pub &'a str);
impl SystemLinkDirs<'_> {
    pub fn path(&self) -> Vec<PathBuf> {
        match self.0 {
            "DESKTOP" => {
                let mut users_desktop = PathBuf::new();
                let mut public_desktop = PathBuf::from(r"C:\Users\Public\Desktop");
                for tuple in [(&mut users_desktop, "USERPROFILE"), (&mut public_desktop, "PUBLIC")].iter_mut() {
                    if !tuple.0.is_dir() {
                        match env::var_os(&tuple.1) {
                            Some(val) => {*tuple.0 = Path::new(&val).join("Desktop")},
                            None => panic!("Error: Unable to determine {} path.", self.0),
                        };
                    };
                };
                vec![users_desktop, public_desktop]
            },
            "START_MENU" => {
                let mut users_start_menu = PathBuf::new();
                let mut public_start_menu = PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu");
                for tuple in [(&mut users_start_menu, "APPDATA"), (&mut public_start_menu, "PROGRAMDATA")].iter_mut() {
                    if !tuple.0.is_dir() {
                        match env::var_os(&tuple.1) {
                            Some(val) => {*tuple.0 = Path::new(&val).join(r"Microsoft\Windows\Start Menu")},
                            None => panic!("Error: Unable to determine {} path.", self.0),
                        };
                    };
                };
                vec![users_start_menu, public_start_menu]
            },
            _ => {panic!("Unsupported input value: {}. Supported values are 'DESKTOP' or 'START_MENU'.", self.0)}
        }
    }
}

fn get_link_info(path_buf: &PathBuf) -> (String, String, String, String, String, String, String, String) {
    let link_name = path_buf.file_stem()
        .map_or(String::from("unnamed_file")
        , |no_ext| no_ext.to_string_lossy().into_owned());
    // 快捷方式实际路径
    let link_path = path_buf.to_string_lossy().into_owned();      // to_string_lossy : 任何非 Unicode 序列都将替换为 �
    // 引用 lnk 库
    let objlnk = lnk::ShellLink::open(&link_path).unwrap();
    // 快捷方式目标目录和目标路径
    let link_relative_path = objlnk.relative_path().clone().unwrap_or(String::new());
    let mut link_target_dir = objlnk.working_dir().clone().unwrap_or(String::new());
    let mut link_target_path = objlnk.link_info().clone().map_or(String::new(), |i| i.local_base_path().clone().unwrap_or(String::new()));
    // lnk库的link_info可能返回None或返回的目标路径存在非 Unicode 序列
    let link_taget_path_error = if link_target_path.is_empty() || link_target_path.contains('�') {true} else {false};
    match (link_target_dir.is_empty(), link_taget_path_error) {
        (false, true) if !link_relative_path.is_empty() => {
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
        }
        (true, false) => {link_target_dir = Path::new(&link_target_path).parent().map_or(String::new(), |i| i.to_string_lossy().into_owned())},
        (_, _) => {},
    };
    // 快捷方式图标路径、是否被更换    
    let mut link_icon_location = String::new();
    let mut link_icon_status = String::new();
    match &objlnk.icon_location() {
        Some(path_string) => {
            link_icon_location = path_string.to_string();
            if Path::new(path_string).is_file()
            && (link_target_path.is_empty()                                 // 排除UWP|APP情况
            || (link_icon_location != link_target_path                      // 排除图标源于目标
            && !link_icon_location.contains("WindowsSubsystemForAndroid")   // 排除WSA应用
            && !link_icon_location.contains(&link_target_dir)))             // 排除图标位于目标(子)目录
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
            "taskmgr.exe"    => {link_target_ext = String::from("taskmgr")},  // 任务管理器
            "explorer.exe"   => {link_target_ext = String::from("explorer")}, // 资源管理器
            "msconfig.exe"   => {link_target_ext = String::from("msconfig")}, // 系统配置实用工具
            "services.exe"   => {link_target_ext = String::from("services")}, // 管理启动和停止服务
            "sc.exe"         => {link_target_ext = String::from("sc")},       // 管理系统服务
            "cmd.exe"        => {link_target_ext = String::from("cmd")},      // 命令提示符
            "powershell.exe" => {link_target_ext = String::from("psh")},      // PowerShell
            "wscript.exe"    => {link_target_ext = String::from("wscript")},  // 脚本
            "cscript.exe"    => {link_target_ext = String::from("cscript")},  // 脚本
            "regedit.exe"    => {link_target_ext = String::from("regedit")},  // 注册表
            "mstsc.exe"      => {link_target_ext = String::from("mstsc")},    // 远程连接
            "regsvr32.exe"   => {link_target_ext = String::from("regsvr32")}, // 注册COM组件
            "rundll32.exe"   => {link_target_ext = String::from("rundll32")}, // 执行32位的DLL文件
            "mshta.exe"      => {link_target_ext = String::from("mshta")},    // 执行.HTA文件
            "msiexec.exe"    => {link_target_ext = String::from("msiexec")},  // 安装Windows Installer安装包(MSI)
            "control.exe"    => {link_target_ext = String::from("control")},  // 控制面板执行
            "msdt.exe"       => {link_target_ext = String::from("msdt")},     // Microsoft 支持诊断工具
            "wmic.exe"       => {link_target_ext = String::from("wmic")},     // WMI 命令行
            "net.exe"        => {link_target_ext = String::from("net")},      // 工作组连接安装程序
            "netscan.exe"    => {link_target_ext = String::from("netscan")},  // 网络扫描
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
pub fn collect_link_info(dirs: Vec<impl AsRef<Path>>, link_map: &mut HashMap<(String, String), LinkInfo>) {
    for dir in dirs {
        let directory = format!(r"{}\**\*.lnk", dir.as_ref().to_string_lossy());
        for path_buf in glob(&directory).unwrap().filter_map(Result::ok) {
            let (link_name, link_path, link_target_dir, link_target_path, link_target_ext, link_icon_location, link_icon_index, link_icon_status) = get_link_info(&path_buf);
            // 排除重复项目
            if link_map.contains_key(&(link_name.clone(), link_target_ext.clone())) {
                continue;
            }
            link_map.insert((link_name, link_target_ext), LinkInfo {
                link_path,
                link_target_path,
                link_target_dir,
                link_icon_location,
                link_icon_index,
                link_icon_status,
            });
        }
    };
}




