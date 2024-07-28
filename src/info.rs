use crate::{utils::{read_log, write_log}, Error, glob, LinkProp, Status, Path, PathBuf};
use winsafe::{IPersistFile, prelude::*, co};
use std::env;

#[allow(unused)]
pub enum SystemLinkDirs {
    Desktop,
    StartMenu,
    StartUp,
}
impl SystemLinkDirs {
    pub fn get_path(&self) -> Result<Vec<PathBuf>, winsafe::co::HRESULT> {
        let mut path_vec = Vec::new();

        // Get the GUID of the shortcut's folder - 获取快捷方式文件夹的GUID
        let know_folder_id_vec = match self {
            SystemLinkDirs::Desktop => vec![&co::KNOWNFOLDERID::Desktop, &co::KNOWNFOLDERID::PublicDesktop],
            SystemLinkDirs::StartMenu => vec![&co::KNOWNFOLDERID::StartMenu, &co::KNOWNFOLDERID::CommonStartMenu],
            SystemLinkDirs::StartUp => vec![&co::KNOWNFOLDERID::Startup, &co::KNOWNFOLDERID::CommonStartup],
        };

        // Get the path to the shortcut's folder - 获取快捷方式文件夹的路径
        for folder_id in know_folder_id_vec.iter() {
            let path = winsafe::SHGetKnownFolderPath(
                folder_id,
                co::KF::NO_ALIAS,    //  确保返回文件夹的物理路径，避免别名路径
                None,
            )?;
            path_vec.push(PathBuf::from(path));
        };

        Ok(path_vec)
    }
}

pub struct ManageLinkProp;
impl ManageLinkProp {
    fn get_info(
        path_buf: PathBuf,
        shell_link: winsafe::IShellLink,
        persist_file: IPersistFile
    ) -> Result<LinkProp, Box<dyn std::error::Error>> {
        let link_path = path_buf.to_string_lossy().into_owned();

        // Load the shortcut file (LNK file)
        persist_file.Load(&link_path, co::STGM::READ)?;

        let link_name = match path_buf.file_stem() {
            Some(name) => name.to_string_lossy().into_owned(),
            None => String::from("unnamed_file")
        };

        let link_target_path = match shell_link.GetPath(
            Some(&mut winsafe::WIN32_FIND_DATA::default()),
            co::SLGP::RAWPATH   // Absolute path - 绝对路径
        ) {
            Ok(path) => ManageLinkProp::convert_env_to_path(path),    // 开头为环境变量时转换为绝对路径
            Err(_) => String::new(),
        };

        let link_target_dir = match shell_link.GetWorkingDirectory() {
            Ok(path) => {
                match Path::new(&path).try_exists() {
                    Ok(true) => ManageLinkProp::convert_env_to_path(path),    // 开头为环境变量时转换为绝对路径,
                    Ok(false) => ManageLinkProp::get_working_directory(&link_target_path),
                    Err(_) => return Err(format!("{link_name}: Failed get the working directory").into()),
                }
            },
            Err(_) => ManageLinkProp::get_working_directory(&link_target_path)
        };

        let link_target_ext = if link_target_path.is_empty() {
            String::from("uwp|app")
        } else {
            let link_target_file_name = Path::new(&link_target_path)
                .file_name()
                .map_or( 
                    String::new(), 
                    |name| name.to_string_lossy().into_owned()
                );
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
                    let ext = Path::new(&link_target_path).extension();
                    let is_app = link_target_path.contains("WindowsSubsystemForAndroid");
                    match (&ext, is_app) {
                        (_, true) => String::from("app"),
                        (None, false) => String::new(),
                        (Some(os_str), false) => os_str.to_string_lossy().into_owned().to_lowercase()
                    }
                }
            }   
        };

        #[allow(unused_assignments)]
        let mut unconverted_icon_path = String::new();

        let (link_icon_location, link_icon_index) = match shell_link.GetIconLocation() {
            Ok((icon_path, icon_index)) => {
                unconverted_icon_path = icon_path.clone();
                match (Path::new(&icon_path).is_file(), icon_path.ends_with(".dll")) {
                    (false, false) => (String::new(), String::new()),
                    (false, true) => (ManageLinkProp::convert_env_to_path(icon_path), icon_index.to_string()),
                    _ => (ManageLinkProp::convert_env_to_path(icon_path), icon_index.to_string())    // 开头为环境变量（如%windir%）时修改为路径
                }
            },
            Err(err) => return Err(format!("{err}\n{link_name}: Failed get the icon location").into()), // (String::new(), String::new()),
        };

        let link_icon_status = if link_icon_location.is_empty()    // unchanged、non-existent、inaccessible - 未更换图标、图标不存在、图标不可访问（UWP/APP）
            || link_icon_location == link_target_path    // unchanged(from target file) - 未更换图标（图标源于目标文件）
            || link_icon_location.contains("WindowsSubsystemForAndroid")   //Android App - WSA应用
            || unconverted_icon_path.starts_with("%")    // System icon - 系统图标(如%windir%/.../powershell.exe)
            || (Path::new(&link_icon_location).parent().unwrap_or(Path::new("")) == Path::new(&link_target_dir)    // Icons come from the target file's (sub)directory - 图标来源于目标文件的(子)目录
                && !link_target_dir.is_empty()) {
            Status::Unchanged
        } else {
            Status::Changed
        };

        let link_arguments = match shell_link.GetArguments() {
            Ok(arguments) => arguments,
            Err(err) => return Err(format!("{err}\n{link_name}: Failed get the Arguments").into()),
        };

        Ok(LinkProp {
            name: link_name,
            path: link_path,
            status: link_icon_status,
            target_ext: link_target_ext,
            target_dir: link_target_dir,
            target_path: link_target_path,
            icon_location: link_icon_location,
            icon_index: link_icon_index,
            arguments: link_arguments,
        })
    }

    fn get_working_directory(path: &String) -> String {
        match path.is_empty() {
            true => String::new(),
            false => {
                Path::new(path).parent().map_or(
                    String::new(),
                    |path| path.to_string_lossy().into_owned()
                )
            }
        }
    }

    fn convert_env_to_path(env_path: String) -> String {
        match env_path.starts_with("%") {
            true => {
                let envs = vec![
                    ("%windir%",
                        winsafe::SHGetKnownFolderPath(
                            &co::KNOWNFOLDERID::Windows,
                            co::KF::NO_ALIAS,
                            None,
                        ).unwrap_or("C:/Windows".to_string()),
                    ),
                    ("%systemroot%",
                        winsafe::SHGetKnownFolderPath(
                            &co::KNOWNFOLDERID::Windows,
                            co::KF::NO_ALIAS,
                            None,
                        ).unwrap_or("C:/Windows".to_string()),
                    ),
                    ("%programfiles%",
                        winsafe::SHGetKnownFolderPath(
                            &co::KNOWNFOLDERID::ProgramFiles,
                            co::KF::NO_ALIAS,
                            None,
                        ).unwrap_or("C:/Windows/Program Files".to_string()),
                    ),
                    ("%programfiles(x86)%",
                        winsafe::SHGetKnownFolderPath(
                            &co::KNOWNFOLDERID::ProgramFilesX86,
                            co::KF::NO_ALIAS,
                            None,
                        ).unwrap_or("C:/Windows/Program Files(x86)".to_string()),
                    ),
                    ("%programfiles(arm)%",
                        env::var_os("PROGRAMFILES(Arm)").map_or(
                            "C:/Program Files (Arm)".to_string(),
                            |path| path.to_string_lossy().into_owned()
                        )
                    ),
                    ("%commonprogramfiles%",
                        winsafe::SHGetKnownFolderPath(
                            &co::KNOWNFOLDERID::ProgramFilesCommonX64,
                            co::KF::NO_ALIAS,
                            None,
                        ).unwrap_or("C:/Program Files/Common Files".into()),
                    ),
                    ("%commonprogramfiles(arm)%",
                        env::var_os("CommonProgramFiles(Arm)").map_or(
                            "C:/Program Files (Arm)/Common Files".to_string(),
                            |path| path.to_string_lossy().into_owned()
                        )
                        ),
                    ("%commonprogramfiles(x86)%",
                        winsafe::SHGetKnownFolderPath(
                            &co::KNOWNFOLDERID::ProgramFilesCommonX86,
                            co::KF::NO_ALIAS,
                            None,
                        ).unwrap_or("C:/Program Files (x86)/Common Files".into()),
                    ),
                    ("%programdata%",
                        winsafe::SHGetKnownFolderPath(
                            &co::KNOWNFOLDERID::ProgramData,
                            co::KF::NO_ALIAS,
                            None,
                        ).unwrap_or("C:/ProgramData".into()),
                    ),
                    ("%allusersprofile%",
                        winsafe::SHGetKnownFolderPath(
                            &co::KNOWNFOLDERID::ProgramData,
                            co::KF::NO_ALIAS,
                            None,
                        ).unwrap_or("C:/ProgramData".into()),
                    ),
                    ("%cmdcmdline%",
                        env::var_os("CMDCMDLINE").map_or(
                            "C:/Windows/System32/cmd.exe".to_string(),
                            |path| path.to_string_lossy().into_owned()
                        )
                    ),
                    ("%comspec%",
                        env::var_os("COMSPEC").map_or(
                            "C:/Windows/System32/cmd.exe".to_string(),
                            |path| path.to_string_lossy().into_owned()
                        )
                    ),
                    ("%usersprofile%",
                        winsafe::SHGetKnownFolderPath(
                            &co::KNOWNFOLDERID::Profile,
                            co::KF::NO_ALIAS,
                            None,
                        ).unwrap_or(String::new()),
                    ),
                    ("%localappdata%",
                        winsafe::SHGetKnownFolderPath(
                            &co::KNOWNFOLDERID::LocalAppData,
                            co::KF::NO_ALIAS,
                            None,
                        ).unwrap_or(String::new()),
                    ),
                    ("%appdata%",
                        winsafe::SHGetKnownFolderPath(
                            &co::KNOWNFOLDERID::RoamingAppData,
                            co::KF::NO_ALIAS,
                            None,
                        ).unwrap_or(String::new()),
                    ),
                    ("%public%",
                        winsafe::SHGetKnownFolderPath(
                            &co::KNOWNFOLDERID::Public,
                            co::KF::NO_ALIAS,
                            None,
                        ).unwrap_or(String::new()),
                    ),
                    ("%temp%",
                        env::var_os("TEMP").map_or(
                            String::new(),
                            |path| path.to_string_lossy().into_owned()
                        )
                    ),
                    ("%tmp%",
                        env::var_os("TMP").map_or(
                            String::new(),
                            |path| path.to_string_lossy().into_owned()
                        )
                    ),
                ];
                for (env, root) in envs.iter() {
                    if Path::new(&env_path.to_lowercase()).starts_with(env) {
                        let new_path = env_path.replacen(env, root, 1);
                        match (Path::new(&new_path).is_file(), Path::new(&new_path).is_dir()) {
                            (false, false) => return env_path,
                            _ => return new_path
                        };
                    };
                };
                String::new()
            },
            false => env_path
        }
    }

    pub fn collect(dirs_vec: Vec<impl AsRef<Path>>, link_vec: &mut Vec<LinkProp>) -> Result<(), Box<dyn Error>> {
        // Clear
        link_vec.clear();

        // Initialize COM library - 初始化 COM 库
        let _com_lib = winsafe::CoInitializeEx( // keep guard alive
            co::COINIT::APARTMENTTHREADED
            | co::COINIT::DISABLE_OLE1DDE,
        )?;

        // Create IShellLink object - 创建 IShellLink 对象
        let shell_link = winsafe::CoCreateInstance::<winsafe::IShellLink>(
            &co::CLSID::ShellLink,
            None,
            co::CLSCTX::INPROC_SERVER,
        )?;

        // Query for IPersistFile interface - 查询并获取 IPersistFile 接口实例
        let persist_file: IPersistFile = shell_link.QueryInterface()?;

        // Open Log File - 打开日志文件
        let mut log_file = read_log().expect("Failed to open 'LinkEcho.log'");

        // Iterate through directories - 遍历快捷方式目录
        for dir in dirs_vec.iter() {
            let directory = dir.as_ref().join("**\\*.lnk").to_string_lossy().into_owned();
            for path_buf in glob(&directory).unwrap().filter_map(Result::ok) {
                match ManageLinkProp::get_info(path_buf, shell_link.clone(), persist_file.clone()) {
                    Ok(link_prop) => link_vec.push(link_prop),
                    Err(err) => {
                        write_log(&mut log_file, err.to_string())?;
                        continue
                    }
                }
            }
        }

        Ok(())
    }
}