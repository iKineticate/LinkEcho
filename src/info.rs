use crate::{Path, PathBuf, env, LinkProp, glob};
pub struct SystemLinkDirs;

impl SystemLinkDirs{
    pub fn Path(name: &str) -> Vec<PathBuf> {
        let mut users_path = PathBuf::new();
        let (mut public_path, users_env, public_env, env_push) = match name {
            "DESKTOP" => (
                PathBuf::from(r"C:\Users\Public\Desktop"),
                "USERPROFILE",
                "PUBLIC",
                "Desktop",
            ),
            "START_MENU" => (
                PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu"),
                "APPDATA",
                "PROGRAMDATA",
                r"Microsoft\Windows\Start Menu",
            ),
            "START_UP" => (
                PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs\Startup"),
                "APPDATA",
                "PROGRAMDATA",
                r"Microsoft\Windows\Start Menu\Programs\Startup",
            ),
            _ => panic!("Unsupported input value: {}. Supported values are 'DESKTOP' or 'START_MENU' or 'START_UP'.", name)
        };
        for tuple in [(&mut users_path, users_env, env_push), (&mut public_path, public_env, env_push)].iter_mut() {
            if !tuple.0.is_dir() {
                match env::var_os(&tuple.1) {
                    Some(val) => {*tuple.0 = Path::new(&val).join(env_push)},
                    None => panic!("Error: Unable to determine {} path.", name),
                };
            };
        };
        vec![users_path, public_path]
    }
}


pub struct ManageLinkProp;

impl ManageLinkProp {
    fn get(path_buf: PathBuf) -> (String, String, String, String, String, String, String, String) {
        let link_name = path_buf.file_stem()
            .map_or(String::from("unnamed_file")
            , |no_ext| no_ext.to_string_lossy().into_owned());
        let link_path = path_buf.to_string_lossy().into_owned();
        // 引用 lnk 库
        let objlnk = lnk::ShellLink::open(&link_path).unwrap();
        let link_relative_path = objlnk.relative_path().clone().unwrap_or(String::new());
        let mut link_target_dir = objlnk.working_dir().clone().unwrap_or(String::new());
        let mut link_target_path = objlnk.link_info().clone().map_or(String::new(), |i| i.local_base_path().clone().unwrap_or(String::new()));
        // 解决lnk库的link_info返回None或返回的目标路径存在非 Unicode 序列的问题
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
        let (link_icon_location, link_icon_status) = match &objlnk.icon_location() {
            Some(path_string) => {  // &String
                if Path::new(path_string).is_file()
                    && (path_string.is_empty()                               // 排除UWP|APP情况
                    || (path_string != &link_target_path                     // 排除图标源于目标
                    && !path_string.contains("WindowsSubsystemForAndroid")   // 排除WSA应用
                    && !path_string.contains(&link_target_dir)))             // 排除图标位于目标(子)目录
                {
                    (path_string.clone(), String::from("√"))
                } else {
                    (path_string.clone(), String::new())
                }
            }, 
            None => (String::new(), String::new()),
        };    
        let link_icon_index = objlnk.header().icon_index().to_string();
        let link_target_ext = if !Path::new(&link_target_path).is_file() {
            String::from("uwp|app")
        } else {
            let link_target_file_name = Path::new(&link_target_path).file_name()
                .map_or_else(|| String::new(), 
                |name| name.to_string_lossy().into_owned());
            let file_extension = Path::new(&link_target_path).extension();
            match &*link_target_file_name {
                "schtasks.exe"   => String::from("schtasks"), // 任务计划程序
                "taskmgr.exe"    => String::from("taskmgr"),  // 任务管理器
                "explorer.exe"   => String::from("explorer"), // 资源管理器
                "msconfig.exe"   => String::from("msconfig"), // 系统配置实用工具
                "services.exe"   => String::from("services"), // 管理启动和停止服务
                "sc.exe"         => String::from("sc"),       // 管理系统服务
                "cmd.exe"        => String::from("cmd"),      // 命令提示符
                "powershell.exe" => String::from("psh"),      // PowerShell
                "wscript.exe"    => String::from("wscript"),  // 脚本
                "cscript.exe"    => String::from("cscript"),  // 脚本
                "regedit.exe"    => String::from("regedit"),  // 注册表
                "mstsc.exe"      => String::from("mstsc"),    // 远程连接
                "regsvr32.exe"   => String::from("regsvr32"), // 注册COM组件
                "rundll32.exe"   => String::from("rundll32"), // 执行32位的DLL文件
                "mshta.exe"      => String::from("mshta"),    // 执行.HTA文件
                "msiexec.exe"    => String::from("msiexec"),  // 安装Windows Installer安装包(MSI)
                "control.exe"    => String::from("control"),  // 控制面板执行
                "msdt.exe"       => String::from("msdt"),     // Microsoft 支持诊断工具
                "wmic.exe"       => String::from("wmic"),     // WMI 命令行
                "net.exe"        => String::from("net"),      // 工作组连接安装程序
                "netscan.exe"    => String::from("netscan"),  // 网络扫描
                _ => {
                    match (&file_extension, link_target_path.contains("WindowsSubsystemForAndroid")) {
                        (None, _) => String::new(),
                        (Some(_), true) => String::from("app"),
                        (Some(os_str), false) => os_str.to_string_lossy().into_owned().to_lowercase()
                    }
                }
            }
        };

        (link_name, link_path, link_target_dir, link_target_path, link_target_ext, link_icon_location, link_icon_index, link_icon_status)
    }

    pub fn collect(path_vec: Vec<impl AsRef<Path>>, link_vec: &mut Vec<LinkProp>) {
        for dir in path_vec.iter() {
            let directory = format!(r"{}\**\*.lnk", dir.as_ref().to_string_lossy());
            for path_buf in glob(&directory).unwrap().filter_map(Result::ok) {
                let (name, path, target_dir, target_path, target_ext, icon_location, icon_index, icon_status) = ManageLinkProp::get(path_buf);
                link_vec.push( LinkProp {
                    name,
                    path,
                    target_ext,
                    target_dir,
                    target_path,
                    icon_status,
                    icon_location,
                    icon_index,
                })
            }
        };
    }
}