use crate::{Path, PathBuf, LinkProp, glob};
use winsafe::{IPersistFile, prelude::*, co};

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
    fn get(path_buf: PathBuf, shell_link: winsafe::IShellLink, persist_file: IPersistFile) -> Result<LinkProp, winsafe::co::HRESULT> {
        let link_path = path_buf.to_string_lossy().into_owned();

        // Load the shortcut file (LNK file)
        persist_file.Load(&link_path, co::STGM::READ)?;

        let link_name = match path_buf.file_stem() {
            Some(name) => name.to_string_lossy().into_owned(),
            None => String::from("unnamed_file")
        };

        let link_target_path = shell_link.GetPath(
            Some(&mut winsafe::WIN32_FIND_DATA::default()),
            co::SLGP::RAWPATH   // Absolute path - 绝对路径
        ).unwrap_or(String::new());    // fn: 环境变量修改为绝对路径-----------------------  有路径参数但目标不存在，提醒是否删除或者标红？

        let link_target_dir = match shell_link.GetWorkingDirectory() {
            Ok(path) => {
                match Path::new(&path).is_dir() {
                    true => path,    // fn: 环境变量修改为绝对路径-----------------------
                    false => ManageLinkProp::get_working_directory(&link_target_path)
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
                    let app = link_target_path.contains("WindowsSubsystemForAndroid");
                    match (&ext, app) {
                        (_, true) => String::from("app"),
                        (None, false) => String::new(),
                        (Some(os_str), false) => os_str.to_string_lossy().into_owned().to_lowercase()
                    }
                }
            }   
        };

        let (link_icon_location, link_icon_index) = match shell_link.GetIconLocation() {
            Ok((icon_path, icon_index)) => {
                if Path::new(&icon_path).is_file() || icon_path.ends_with(".dll") {
                    (icon_path, icon_index.to_string())    // fn: 环境变量修改为绝对路径-----------------------
                } else {
                    (String::new(), String::new())
                }
            },
            Err(_) => (String::new(), String::new())
        };

        let link_icon_status = if link_icon_location.is_empty()    // 未更换图标、图标不存在、图标不可访问（UWP/APP）
            || link_icon_location == link_target_path    // 排除图标源于目标
            || link_icon_location.contains("WindowsSubsystemForAndroid")   // 排除WSA应用
            || (link_icon_location.contains(&link_target_dir)
                && !link_target_dir.is_empty()) {    // 排除图标位于目标(子)目录
            None
        } else {
            Some(String::from("√"))
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

    pub fn collect(dirs_vec: Vec<impl AsRef<Path>>, link_vec: &mut Vec<LinkProp>) -> Result<(), winsafe::co::HRESULT> {
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

        // Iterate through directories - 遍历快捷方式目录
        for dir in dirs_vec.iter() {
            let directory = dir.as_ref().join("**\\*.lnk").to_string_lossy().into_owned();
            for path_buf in glob(&directory).unwrap().filter_map(Result::ok) {
                match ManageLinkProp::get(path_buf, shell_link.clone(), persist_file.clone()) {
                    Ok(link_prop) => link_vec.push(link_prop),
                    Err(err) => {
                        println!("{}", err);
                        continue
                    }
                }
            }
        }

        winsafe::HrResult::Ok(())
    }
}