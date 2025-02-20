use crate::{
    glob,
    utils::{get_img_base64_by_path, write_log},
    LinkProp, Path, PathBuf, Status,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use std::{collections::HashMap, env};
use winsafe::{co, prelude::*, IPersistFile};

#[allow(unused)]
pub enum SystemLinkDirs {
    Desktop,
    StartMenu,
    StartUp,
}
impl SystemLinkDirs {
    pub fn get_path(&self) -> Result<Vec<PathBuf>> {
        let mut path_vec = Vec::new();

        // Get the GUID of the shortcut's folder
        let know_folder_id_vec = match self {
            SystemLinkDirs::Desktop => vec![
                &co::KNOWNFOLDERID::Desktop,
                &co::KNOWNFOLDERID::PublicDesktop,
            ],
            SystemLinkDirs::StartMenu => vec![
                &co::KNOWNFOLDERID::StartMenu,
                &co::KNOWNFOLDERID::CommonStartMenu,
            ],
            SystemLinkDirs::StartUp => vec![
                &co::KNOWNFOLDERID::Startup,
                &co::KNOWNFOLDERID::CommonStartup,
            ],
        };

        // Get the path to the shortcut's folder
        for folder_id in know_folder_id_vec.iter() {
            let path = winsafe::SHGetKnownFolderPath(
                folder_id,
                co::KF::NO_ALIAS, // 确保返回文件夹的物理路径，避免别名路径
                None,
            )
            .context(format!(
                "Failed to get the path from Folder Id: {folder_id}"
            ))?;
            path_vec.push(PathBuf::from(path));
        }

        Ok(path_vec)
    }
}

pub struct ManageLinkProp;
impl ManageLinkProp {
    fn get_info(
        path_buf: PathBuf,
        shell_link: &winsafe::IShellLink,
        persist_file: &IPersistFile,
    ) -> Result<LinkProp> {
        let link_path = path_buf.to_string_lossy().to_string();

        persist_file
            .Load(&link_path, co::STGM::READ)
            .context("Failed to load the shortcut by COM interface.")?;

        let link_name = path_buf
            .file_stem()
            .map_or(String::from("unnamed_file"), |n| {
                n.to_string_lossy().into_owned()
            });

        let link_target_path = shell_link
            .GetPath(
                // 注意：提供的路径可能不存在（比如UWP、APP、未提供路径的lnk）
                Some(&mut winsafe::WIN32_FIND_DATA::default()),
                co::SLGP::RAWPATH, // Absolute path - 绝对路径
            )
            .map_or(String::new(), ManageLinkProp::convert_env_to_path);

        let link_target_dir = match shell_link.GetWorkingDirectory() {
            Ok(p) if !p.is_empty() => ManageLinkProp::convert_env_to_path(p),
            _ => ManageLinkProp::get_parent_path(&link_target_path),
        };

        let link_target_ext = if link_target_path.is_empty() {
            String::from("uwp|app")
        } else {
            let link_target_file_name = Path::new(&link_target_path)
                .file_name()
                .map_or(String::new(), |n| n.to_string_lossy().to_lowercase());

            match &*link_target_file_name {
                "schtasks.exe" => String::from("schtasks"), // 任务计划程序
                "taskmgr.exe" => String::from("taskmgr"),   // 任务管理器
                "explorer.exe" => String::from("explorer"), // 资源管理器
                "msconfig.exe" => String::from("msconfig"), // 系统配置实用工具
                "services.exe" => String::from("services"), // 管理启动和停止服务
                "sc.exe" => String::from("sc"),             // 管理系统服务
                "cmd.exe" => String::from("cmd"),           // 命令提示符
                "powershell.exe" => String::from("psh"),    // PowerShell
                "wscript.exe" => String::from("wscript"),   // 脚本
                "cscript.exe" => String::from("cscript"),   // 脚本
                "regedit.exe" => String::from("regedit"),   // 注册表
                "mstsc.exe" => String::from("mstsc"),       // 远程连接
                "regsvr32.exe" => String::from("regsvr32"), // 注册COM组件
                "rundll32.exe" => String::from("rundll32"), // 执行32位的DLL文件
                "mshta.exe" => String::from("mshta"),       // 执行.HTA文件
                "msiexec.exe" => String::from("msiexec"),   // 安装Windows Installer安装包(MSI)
                "control.exe" => String::from("control"),   // 控制面板执行
                "msdt.exe" => String::from("msdt"),         // Microsoft 支持诊断工具
                "wmic.exe" => String::from("wmic"),         // WMI 命令行
                "net.exe" => String::from("net"),           // 工作组连接安装程序
                "netscan.exe" => String::from("netscan"),   // 网络扫描
                _ => {
                    let ext = Path::new(&link_target_path).extension();
                    let is_app = link_target_path
                        .to_lowercase()
                        .contains("windowssubsystemforandroid");
                    let is_uwp = link_target_path
                        .to_lowercase()
                        .contains(r"appdata\local\microsoft\windowsapps");
                    match (ext, is_app, is_uwp) {
                        (_, true, _) => String::from("app"),
                        (_, _, true) => String::from("uwp"),
                        (None, false, false) => String::new(),
                        (Some(os_str), false, false) => os_str.to_string_lossy().to_lowercase(),
                    }
                }
            }
        };

        let link_icon_base64 = get_img_base64_by_path(&link_path);
        let link_target_icon_base64 = get_img_base64_by_path(&link_target_path);

        let mut unconverted_icon_path = String::new();

        let (link_icon_location, link_icon_index) = shell_link
            .GetIconLocation()
            .map(|(icon_path, icon_index)| {
                unconverted_icon_path = icon_path.clone();
                let converted_icon_path = ManageLinkProp::convert_env_to_path(icon_path.clone());
                match (Path::new(&icon_path).is_file(), icon_path.ends_with(".dll")) {
                    (false, false) => (String::new(), String::new()),
                    _ => (converted_icon_path, icon_index.to_string()),
                }
            })
            .context(format!(
                "Failed get the shortcut icon location: {link_name}"
            ))?;

        let link_icon_dir = ManageLinkProp::get_parent_path(&link_icon_location);

        let link_icon_status = if link_icon_location.is_empty() // unchanged、non-existent、inaccessible - 未更换图标、图标不存在、图标不可访问（UWP/APP）
            || link_icon_location == link_target_path // Icon from target file - 图标源于目标文件
            || link_target_ext == "app" // Windows Subsystem for Android - WSA应用
            || link_target_ext == "uwp" // Universal Windows Platform - UWP应用
            || unconverted_icon_path.starts_with("%")  // Icon From System icon - 系统图标 (%windir%/.../powershell.exe  ,  %windir%/.../imageres.dll)
            || (link_icon_dir == link_target_dir && Path::new(&link_target_dir).is_dir())
        // Icons come from the target file's (sub)directory - 图标来源于目标目录
        {
            Status::Unchanged
        } else {
            Status::Changed
        };

        let link_arguments = shell_link.GetArguments().context(format!(
            "Failed to get the shortcut's arguments: {link_name}"
        ))?;

        let metadata = std::fs::metadata(&link_path).context(format!(
            "Failed to get the shortcut's metadata: {link_name}"
        ))?;

        let link_file_size = format!("{:.2} KB", metadata.len() as f64 / 1024.0);

        let link_created_at = metadata
            .created()
            .map(|time| {
                let datetime: DateTime<Local> = time.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .context(format!(
                "Failed to get the shortcut's creation: {link_name}"
            ))?;

        let link_updated_at = metadata
            .modified()
            .map(|time| {
                let datetime: DateTime<Local> = time.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .context(format!(
                "Failed to get the shortcut's updated time: {link_name}"
            ))?;

        let link_accessed_at = metadata
            .accessed()
            .map(|time| {
                let datetime: DateTime<Local> = time.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .context(format!(
                "Failed to get the shortcut's accessed time: {link_name}"
            ))?;

        Ok(LinkProp {
            name: link_name,
            path: link_path,
            status: link_icon_status,
            target_ext: link_target_ext,
            target_dir: link_target_dir,
            target_path: link_target_path,
            icon: link_icon_base64,
            target_icon: link_target_icon_base64,
            icon_location: link_icon_location,
            icon_index: link_icon_index,
            arguments: link_arguments,
            file_size: link_file_size,
            created_at: link_created_at,
            updated_at: link_updated_at,
            accessed_at: link_accessed_at,
        })
    }

    fn get_parent_path(path: &String) -> String {
        Path::new(path)
            .parent()
            .map_or(String::new(), |p| p.to_string_lossy().to_string())
    }

    fn get_path_from_env(known_folder_id: Option<&co::KNOWNFOLDERID>, env: &str) -> String {
        if let Some(id) = known_folder_id {
            winsafe::SHGetKnownFolderPath(id, co::KF::NO_ALIAS, None).unwrap_or(
                env::var_os(env).map_or(String::new(), |p| p.to_string_lossy().to_string()),
            )
        } else {
            env::var_os(env).map_or(String::new(), |p| p.to_string_lossy().to_string())
        }
    }

    fn convert_env_to_path(env_path: String) -> String {
        if !env_path.starts_with('%') {
            return env_path;
        }
        let envs: HashMap<&str, String> = [
            (
                "%windir%",
                ManageLinkProp::get_path_from_env(Some(&co::KNOWNFOLDERID::Windows), "WinDir"),
            ),
            (
                "%systemroot%",
                ManageLinkProp::get_path_from_env(Some(&co::KNOWNFOLDERID::Windows), "SystemRoot"),
            ),
            (
                "%programfiles%",
                ManageLinkProp::get_path_from_env(
                    Some(&co::KNOWNFOLDERID::ProgramFiles),
                    "ProgramFiles",
                ),
            ),
            (
                "%programfiles(x86)%",
                ManageLinkProp::get_path_from_env(
                    Some(&co::KNOWNFOLDERID::ProgramFiles),
                    "ProgramFiles(x86)",
                ),
            ),
            (
                "%programfiles(arm)%",
                ManageLinkProp::get_path_from_env(None, "ProgramFiles(Arm)"),
            ),
            (
                "%commonprogramfiles%",
                ManageLinkProp::get_path_from_env(
                    Some(&co::KNOWNFOLDERID::ProgramFilesCommonX64),
                    "CommonProgramFiles",
                ),
            ),
            (
                "%commonprogramfiles(arm)%",
                ManageLinkProp::get_path_from_env(None, "CommonProgramFiles(Arm)"),
            ),
            (
                "%commonprogramfiles(x86)%",
                ManageLinkProp::get_path_from_env(
                    Some(&co::KNOWNFOLDERID::ProgramFilesCommonX86),
                    "CommonProgramFiles(x86)",
                ),
            ),
            (
                "%programdata%",
                ManageLinkProp::get_path_from_env(
                    Some(&co::KNOWNFOLDERID::ProgramData),
                    "ProgramData",
                ),
            ),
            (
                "%allusersprofile%",
                ManageLinkProp::get_path_from_env(
                    Some(&co::KNOWNFOLDERID::ProgramData),
                    "AllUsersProfile",
                ),
            ),
            (
                "%cmdcmdline%",
                ManageLinkProp::get_path_from_env(None, "CMDCMDLINE"),
            ),
            (
                "%comspec%",
                ManageLinkProp::get_path_from_env(None, "COMSPEC"),
            ),
            (
                "%userprofile%",
                ManageLinkProp::get_path_from_env(Some(&co::KNOWNFOLDERID::Profile), "UserProfile"),
            ),
            (
                "%localappdata%",
                ManageLinkProp::get_path_from_env(
                    Some(&co::KNOWNFOLDERID::LocalAppData),
                    "LocalAppData",
                ),
            ),
            (
                "%appdata%",
                ManageLinkProp::get_path_from_env(
                    Some(&co::KNOWNFOLDERID::RoamingAppData),
                    "AppData",
                ),
            ),
            (
                "%public%",
                ManageLinkProp::get_path_from_env(Some(&co::KNOWNFOLDERID::Public), "Public"),
            ),
            ("%temp%", ManageLinkProp::get_path_from_env(None, "TEMP")),
            ("%tmp%", ManageLinkProp::get_path_from_env(None, "TMP")),
        ]
        .iter()
        .cloned()
        .collect();

        let env_path_lowercase = env_path.to_lowercase();
        for (env, root) in &envs {
            if env_path_lowercase.starts_with(env) {
                return env_path_lowercase.replacen(env, root, 1);
            }
        }

        env_path
    }

    pub fn collect(dirs_vec: &Vec<impl AsRef<Path>>) -> Result<Vec<LinkProp>> {
        let mut link_vec: Vec<LinkProp> = Vec::with_capacity(100);

        let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;

        // Iterate through directories
        for dir in dirs_vec.iter() {
            let directory = dir.as_ref().join("**\\*.lnk").to_string_lossy().to_string();
            link_vec.extend(glob(&directory).unwrap().filter_map(Result::ok).filter_map(
                |path_buf| match ManageLinkProp::get_info(path_buf, &shell_link, &persist_file) {
                    Ok(link_prop) => Some(link_prop),
                    Err(err) => {
                        write_log(err.to_string()).ok();
                        None
                    }
                },
            ));
        }

        // Sort `Vec<LinkProp>` by the first letter of `name` field
        link_vec.sort_by(|a, b| {
            let a_first_char = a.name.to_lowercase().chars().next().unwrap_or('\0');
            let b_first_char = b.name.to_lowercase().chars().next().unwrap_or('\0');
            a_first_char.cmp(&b_first_char)
        });

        Ok(link_vec)
    }
}

pub fn initialize_com_and_create_shell_link() -> Result<(winsafe::IShellLink, IPersistFile)> {
    let _com_lib =
        winsafe::CoInitializeEx(co::COINIT::APARTMENTTHREADED | co::COINIT::DISABLE_OLE1DDE)
            .context("Failed to initialize com library")?;

    let shell_link = winsafe::CoCreateInstance::<winsafe::IShellLink>(
        &co::CLSID::ShellLink,
        None,
        co::CLSCTX::INPROC_SERVER,
    )
    .context("Failed to create an IUnknown-derived COM object - ShellLink")?;

    let persist_file: IPersistFile = shell_link
        .QueryInterface()
        .context("Failed to query for ShellLink's IPersistFile interface")?;

    Ok((shell_link, persist_file))
}
