use crate::{Path, PathBuf, env, LinkProp, glob};

pub struct SystemLinkDirs;
impl SystemLinkDirs {
    pub fn Path(name: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut users_path = PathBuf::new();
        let (mut public_path, users_env, public_env, sub_path) = match name {
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
                "Microsoft/Windows/Start Menu",
            ),
            "START_UP" => (
                PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs\Startup"),
                "APPDATA",
                "PROGRAMDATA",
                "Microsoft/Windows/Start Menu/Programs/Startup",
            ),
            _ => return Err(
                format!(
                    "Unsupported input value: '{}'.\
                    Supported values are 'DESKTOP' or 'START_MENU' or 'START_UP'.",
                    name
                ).into())
        };
        for tuple in [(&mut users_path, users_env, sub_path), (&mut public_path, public_env, sub_path)].iter_mut() {
            if !tuple.0.is_dir() {
                match env::var_os(&tuple.1) {
                    Some(val) => {*tuple.0 = Path::new(&val).join(sub_path)},
                    None => {
                        return Err(
                            format!(
                                "Error: Unable to determine {} path.",
                                name
                            ).into()
                        )
                    }
                }
            }
        };
        Ok(vec![users_path, public_path])
    }
}

pub struct ManageLinkProp;
impl ManageLinkProp {
    fn get(path_buf: PathBuf) -> Result<LinkProp, String> {
        let link_name = match path_buf.file_stem() {
            Some(no_ext) => no_ext.to_string_lossy().into_owned(),
            None => String::from("unnamed_file")
        };
        let link_path = path_buf.to_string_lossy().into_owned();
        // 引用 lnk 库
        let obj_lnk = lnk::ShellLink::open(&link_path).map_err(|_| {
            format!("Failed to open file with lnk-rs library: \n{}", &link_path)
        })?;
        let mut link_target_dir = obj_lnk.working_dir().clone();
        // 解决lnk库的link_info返回的目标路径存在非 Unicode 序列的问题
        let mut link_target_path = obj_lnk.link_info()
            .clone()
            .and_then(|i| i.local_base_path().clone())
            .filter(|path| !path.contains('�'));
        match (&link_target_dir, &link_target_path) {
            (Some(target_dir), None) => {
                if let Some(link_relative_path) = obj_lnk.relative_path().clone() {   
                    let mut new_target_path = PathBuf::from(target_dir.clone());
                    for component in Path::new(&link_relative_path).components() {
                        let path_string = component.as_os_str().to_string_lossy().into_owned();
                        if !path_string.is_empty() && path_string != ".." && !target_dir.contains(&path_string) {
                            new_target_path.push(path_string);
                        };
                    };
                    if new_target_path.is_file() {
                        link_target_path = Some(new_target_path.to_string_lossy().into_owned());
                    };
                };
            },
            (None, Some(_)) => {
                link_target_dir = link_target_path
                    // .clone()
                    .as_ref()
                    .and_then(|path| Path::new(path).parent())
                    .map(|parent| parent.to_string_lossy().into_owned());
            },
            (_, _) => {},
        };
        // 快捷方式图标路径、是否被更换
        let (link_icon_location, link_icon_status) = match &obj_lnk.icon_location() {
            Some(icon_path) => {
                if Path::new(icon_path).is_file() {
                    if Some(icon_path) == link_target_path.as_ref()    // 排除图标源于目标
                        || icon_path.contains("WindowsSubsystemForAndroid")   // 排除WSA应用
                        || icon_path.contains(&link_target_dir.clone().unwrap_or(String::from("none_dir"))) {    // 排除图标位于目标(子)目录
                        (Some(icon_path.clone()), None)
                    } else {
                        (Some(icon_path.clone()), Some(String::from("√")))
                    }
                } else {    // 排除UWP|APP情况
                    (None, None)
                }
            },
            None => (None, None),
        };    
        let link_icon_index = obj_lnk.header().icon_index().to_string();
        let link_target_ext = match &link_target_path {
            Some(path) => {
                let link_target_file_name = Path::new(&path)
                    .file_name()
                    .map_or_else(
                        || String::new(), 
                        |name| name.to_string_lossy().into_owned()
                    );
                let link_target_file_extension = Path::new(&path).extension();
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
                        match (&link_target_file_extension, &path.contains("WindowsSubsystemForAndroid")) {
                            (None, _) => String::new(),
                            (Some(_), true) => String::from("app"),
                            (Some(os_str), false) => os_str.to_string_lossy().into_owned().to_lowercase()
                        }
                    }
                }
            },
            None => String::from("uwp|app")
        };
        Ok( LinkProp {
            name: link_name,
            path: link_path,
            target_ext: link_target_ext,
            target_dir: link_target_dir,
            target_path: link_target_path,
            icon_status: link_icon_status,
            icon_location: link_icon_location,
            icon_index: link_icon_index,
        })
    }

    pub fn collect(dirs_vec: Vec<impl AsRef<Path>>, link_vec: &mut Vec<LinkProp>) {
        for dir in dirs_vec.iter() {
            let directory = dir.as_ref().join("**\\*.lnk").to_string_lossy().into_owned();
            for path_buf in glob(&directory).unwrap().filter_map(Result::ok) {
                match ManageLinkProp::get(path_buf) {
                    Ok(link_prop) => link_vec.push(link_prop),
                    Err(err) => {
                        println!("{}", err);
                        continue
                    }
                }
            }
        }
    }
}