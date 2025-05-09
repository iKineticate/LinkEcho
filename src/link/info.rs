use super::{
    list::{LinkProp, Status},
    utils::initialize_com_and_create_shell_link,
};
use crate::image::base64::get_img_base64_by_path;

use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Local};
use glob::glob;
use log::*;
use winsafe::{IPersistFile, co, prelude::*};

#[allow(unused)]
pub enum SystemLinkDirs {
    Desktop,
    StartMenu,
    StartUp,
}
impl SystemLinkDirs {
    pub fn get_path(&self) -> Result<Vec<PathBuf>> {
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
        let path = know_folder_id_vec
            .into_iter()
            .map(|id| {
                winsafe::SHGetKnownFolderPath(id, co::KF::NO_ALIAS, None)
                    .map(PathBuf::from)
                    .with_context(|| format!("Failed to get the path from Folder Id: {id}"))
            })
            .collect::<Result<Vec<PathBuf>>>()?;

        Ok(path)
    }
}

pub struct ManageLinkProp;
impl ManageLinkProp {
    pub fn get_info(
        path_buf: &Path,
        shell_link: &winsafe::IShellLink,
        persist_file: &IPersistFile,
    ) -> Result<LinkProp> {
        let link_path = path_buf
            .to_str()
            .with_context(|| format!("Invalid Unicode: {path_buf:?}"))?;

        persist_file
            .Load(link_path, co::STGM::READ)
            .map_err(|e| anyhow!("Failed to load the link {path_buf:?} - {e}"))?;

        let link_name = path_buf
            .file_stem()
            .and_then(OsStr::to_str)
            .map(str::to_owned)
            .with_context(|| format!("Failed to get the lnk name {path_buf:?}"))?;

        let link_target_path = shell_link
            .GetPath(
                Some(&mut winsafe::WIN32_FIND_DATA::default()), // 注意：提供的路径可能不存在（比如UWP、APP、未提供路径的lnk）
                co::SLGP::RAWPATH,                              // Absolute path - 绝对路径
            )
            .as_deref()
            .map_or(String::new(), ManageLinkProp::convert_env_to_path);

        let link_target_dir = shell_link
            .GetWorkingDirectory()
            .ok()
            .filter(|p| !p.is_empty())
            .as_deref()
            .map_or(
                ManageLinkProp::get_parent_path(&link_target_path),
                ManageLinkProp::convert_env_to_path,
            );
            
        let link_target_ext = if link_target_path.is_empty() {
            String::from("uwp|app")
        } else {
            let link_target_file_name = Path::new(&link_target_path)
                .file_name()
                .and_then(OsStr::to_str)
                .map(str::to_lowercase)
                .unwrap_or_default();

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
                    let ext = Path::new(&link_target_path)
                        .extension()
                        .and_then(OsStr::to_str)
                        .map(str::to_lowercase)
                        .unwrap_or_default();

                    let is_app = link_target_path
                        .to_lowercase()
                        .contains("windowssubsystemforandroid")
                        .then_some("app".to_owned());

                    let is_uwp = link_target_path
                        .to_lowercase()
                        .contains(r"appdata\local\microsoft\windowsapps")
                        .then_some("uwp".to_owned());

                    match (is_app, is_uwp) {
                        (Some(app), _) => app,
                        (_, Some(uwp)) => uwp,
                        _ => ext,
                    }
                }
            }
        };

        // 不使用目标图标作为Base64是因为Base64内存占用大，性能差
        let link_icon_base64 = get_img_base64_by_path(link_path);
        let link_target_icon_base64 = get_img_base64_by_path(&link_target_path);

        let (unconverted_icon_path, link_icon_path, link_icon_index) = shell_link
            .GetIconLocation()
            .map(|(icon_path, icon_index)| {
                let converted_icon_path = ManageLinkProp::convert_env_to_path(&icon_path);
                match (Path::new(&icon_path).is_file(), icon_path.ends_with(".dll")) {
                    (false, false) => (String::new(), String::new(), String::new()),
                    _ => (icon_path, converted_icon_path, icon_index.to_string()),
                }
            })
            .with_context(|| format!("Failed get the shortcut icon location: {link_name}"))?;

        let link_icon_dir = ManageLinkProp::get_parent_path(&link_icon_path);

        let link_icon_status = if link_icon_path.is_empty() // unchanged、non-existent、inaccessible - 未更换图标、图标不存在、图标不可访问（UWP/APP）
            || link_icon_path == link_target_path // Icon from target file - 图标源于目标文件
            || link_target_ext == "app" // Windows Subsystem for Android - WSA应用
            || link_target_ext == "uwp" // Universal Windows Platform - UWP应用
            || unconverted_icon_path.starts_with("%")  // Icon From System icon - 系统图标 (%windir%/.../powershell.exe  ,  %windir%/.../imageres.dll)
            || (link_icon_dir == link_target_dir && Path::new(&link_target_dir).is_dir())
        // Icons come from the target file's (sub)dir - 图标来源于目标目录
        {
            Status::Unchanged
        } else {
            Status::Changed
        };

        let link_arguments = shell_link
            .GetArguments()
            .with_context(|| format!("Failed to get the shortcut's arguments: {link_name}"))?;

        fn format_system_time(time: SystemTime) -> String {
            let datetime: DateTime<Local> = time.into();
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        }

        let metadata = std::fs::metadata(link_path)
            .with_context(|| format!("Failed to get the shortcut's metadata: {link_name}"))?;

        let link_file_size = format!("{:.2} KB", metadata.len() as f64 / 1024.0);

        let link_created_at = metadata
            .created()
            .map(format_system_time)
            .with_context(|| format!("Failed to get the shortcut's creation: {link_name}"))?;

        let link_updated_at = metadata
            .modified()
            .map(format_system_time)
            .with_context(|| format!("Failed to get the shortcut's updated time: {link_name}"))?;

        let link_accessed_at = metadata
            .accessed()
            .map(format_system_time)
            .with_context(|| format!("Failed to get the shortcut's accessed time: {link_name}"))?;

        Ok(LinkProp {
            name: link_name,
            path: link_path.to_owned(),
            status: link_icon_status,
            target_ext: link_target_ext,
            target_dir: link_target_dir,
            target_path: link_target_path,
            icon_base64: link_icon_base64,
            target_icon_base64: link_target_icon_base64,
            icon_path: link_icon_path,
            icon_index: link_icon_index,
            arguments: link_arguments,
            file_size: link_file_size,
            created_at: link_created_at,
            updated_at: link_updated_at,
            accessed_at: link_accessed_at,
        })
    }

    fn get_parent_path(path: &str) -> String {
        Path::new(path)
            .parent()
            .and_then(Path::to_str)
            .map(str::to_owned)
            .unwrap_or_default()
    }

    fn get_path_from_env(known_folder_id: Option<&co::KNOWNFOLDERID>, env: &str) -> String {
        if let Some(id) = known_folder_id {
            winsafe::SHGetKnownFolderPath(id, co::KF::NO_ALIAS, None).unwrap_or(
                env::var_os(env)
                    .unwrap_or_default()
                    .into_string()
                    .unwrap_or_default(),
            )
        } else {
            env::var_os(env)
                .unwrap_or_default()
                .into_string()
                .unwrap_or_default()
        }
    }

    pub fn convert_env_to_path(env_path: &str) -> String {
        if !env_path.starts_with('%') {
            return env_path.to_owned();
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
        envs.iter()
            .find_map(|(env, root)| {
                env_path_lowercase
                    .starts_with(env)
                    .then(|| env_path_lowercase.replacen(env, root, 1))
            })
            .unwrap_or(env_path.to_owned())
    }

    pub fn collect(dirs_vec: &[impl AsRef<Path>]) -> Result<Vec<LinkProp>> {
        let (shell_link, persist_file) = initialize_com_and_create_shell_link()?;

        let mut link_vec = dirs_vec
            .iter()
            .filter_map(|p| p.as_ref().join("**\\*.lnk").to_str().map(str::to_owned))
            // 使用 flat_map 合并两层迭代器
            .flat_map(|pattern| {
                glob(&pattern)
                    .inspect_err(|e| error!("Glob failed for {pattern}: {e}"))
                    .into_iter()
                    .flatten() // 展开 Result 迭代器
                    .filter_map(Result::ok)
            })
            .filter_map(|path| {
                ManageLinkProp::get_info(&path, &shell_link, &persist_file)
                    .inspect_err(|e| error!("Failed to get info:\n{path:?}\n{e}"))
                    .ok()
            })
            .collect::<Vec<LinkProp>>();

        // 首字母排序
        link_vec.sort_by_key(|prop| prop.name.chars().next().map(|c| c.to_ascii_lowercase()));

        Ok(link_vec)
    }
}
