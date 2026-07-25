use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::result::Result;
use std::str::FromStr;

use super::LinkInfo;

use crate::image::surface::ImageSurfaceContext;
use crate::link::IconResourceType;
use crate::link::env_path::resolve_env_vars;

type LResult<T> = Result<T, Box<dyn std::error::Error>>;

use getset::{Getters, Setters};
use indexmap::IndexMap;
use strum::EnumString;
use windows_icons::get_icon_by_path;
use winsafe::{
    CoCreateInstance, IPersistFile, IShellLink, co, guard::CoUninitializeGuard, prelude::*,
};

#[derive(Setters, Getters)]
#[getset(set = "pub", get = "pub")]
struct LinkResolver {
    _com_guard: CoUninitializeGuard,
    image_surface_context: ImageSurfaceContext,
    shell_link: IShellLink,
    persist_file: IPersistFile,
}

impl LinkResolver {
    pub fn new() -> LResult<Self> {
        let image_surface_context = ImageSurfaceContext::new()?;

        // winsafe's CoInitializeEx Call CoUninitialize as a cleanup operation when dropping.
        let _com_guard =
            winsafe::CoInitializeEx(co::COINIT::APARTMENTTHREADED | co::COINIT::DISABLE_OLE1DDE)
                .map_err(|e| format!("failed to initialize COM library: {e}"))?;

        let shell_link = CoCreateInstance::<IShellLink>(
            &co::CLSID::ShellLink,
            None::<&winsafe::IUnknown>,
            co::CLSCTX::INPROC_SERVER,
        )
        .map_err(|e| format!("failed to create ShellLink COM object: {e}"))?;

        let persist_file: IPersistFile = shell_link
            .QueryInterface()
            .map_err(|e| format!("failed to query IPersistFile interface from ShellLink: {e}"))?;

        Ok(Self {
            _com_guard,
            image_surface_context,
            shell_link,
            persist_file,
        })
    }

    pub fn resolve(&self, link_path: &str) -> LResult<LinkInfo> {
        self.persist_file().Load(link_path, co::STGM::READ)?;

        let link_name = PathBuf::from(link_path)
            .file_stem()
            .and_then(OsStr::to_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("failed to get shortcut name from {link_path}"))?;

        let link_target_path = self
            .shell_link
            .GetPath(
                Some(&mut winsafe::WIN32_FIND_DATA::default()), // 注意：提供的路径可能不存在（比如UWP、APP、未提供路径的lnk）
                co::SLGP::RAWPATH,                              // Absolute path - 绝对路径
            )
            .ok()
            .map(|p| resolve_env_vars(&p))
            .unwrap_or_default();

        let link_work_dir = self
            .shell_link
            .GetWorkingDirectory()
            .ok()
            .and_then(|p| {
                let path = resolve_env_vars(&p);
                if path.is_empty() {
                    Path::new(&link_target_path)
                        .parent()
                        .and_then(Path::to_str)
                        .map(str::to_owned)
                } else {
                    Some(path)
                }
            })
            .unwrap_or_default();

        let link_icon_path = self
            .shell_link
            .GetIconLocation()
            .ok()
            .map(|(p, icon_index)| {
                let icon_path = resolve_env_vars(&p);
                let mut icon_type = IconResourceType::from(icon_path);
                let _ = icon_type.set_dll_index(icon_index);
                icon_type
            });

        let icon_buffer = get_icon_by_path(link_path)
            .inspect_err(|e| log::error!("{e}"))
            .ok();
        let icon_surface = icon_buffer.and_then(|buffer| {
            self.image_surface_context
                .build_surface_from_buffer(buffer.width(), buffer.height(), &buffer)
                .inspect_err(|e| log::error!("{e}"))
                .ok()
        });

        // let target_icon_buffer = get_icon_by_path(&link_target_path)
        //     .inspect_err(|e| error!("{e}"))
        //     .ok();
        // let target_icon_surface = target_icon_buffer.and_then(|b| {
        //     self.image_surface_context
        //         .build_surface_from_buffer(b)
        //         .inspect_err(|e| error!("{e}"))
        //         .ok()
        // });

        let change_status = !link_icon_path
            .as_ref()
            .map(|p| p.as_ref().to_owned())
            .is_none_or(|p| {
                let file_name = Path::new(&p)
                    .file_name()
                    .and_then(OsStr::to_str)
                    .map(str::to_lowercase)
                    .unwrap_or_default();

                let is_app_or_uwp = [
                    r"Appdata\Local\Microsoft\Windowsapps",
                    "WindowsSubsystemForAndroid",
                ]
                .iter()
                .any(|p| link_target_path.contains(p));

                p.is_empty()
                    || p.eq_ignore_ascii_case(&link_target_path)
                    || is_app_or_uwp
                    || SystemApp::from_str(&file_name).is_ok()
            });

        Ok(LinkInfo::default()
            .set_name(link_name)
            .set_path(link_path.to_owned())
            .set_icon_path(link_icon_path)
            .set_icon_surface(icon_surface)
            .set_target_path(link_target_path)
            // .set_target_icon_surface(target_icon_surface)
            .set_working_directory(link_work_dir)
            .set_status(change_status)
            .to_owned())
    }
}

#[derive(EnumString)]
#[strum(serialize_all = "snake_case", suffix = ".exe")]
enum SystemApp {
    Schtasks,
    Taskmgr,
    Explorer,
    Msconfig,
    Services,
    Sc,
    Cmd,
    Powershell,
    Wscript,
    Cscript,
    Regedit,
    Mstsc,
    Regsvr32,
    Rundll32,
    Mshta,
    Msiexec,
    Msinfo32,
    Control,
    Msdt,
    Wmic,
    Net,
    Netscan,
}

pub fn get_link_info_map_from_dirs(dirs: Vec<String>) -> IndexMap<String, LinkInfo> {
    let link_resolver = LinkResolver::new().expect("failed to create link resolver");

    let mut link_info_map = IndexMap::from_iter({
        dirs.into_iter()
            .flat_map(|dir| {
                let pattern = format!("{dir}/**/*.lnk");
                glob::glob(&pattern)
                    .into_iter()
                    .flatten()
                    .filter_map(Result::ok)
                    .map(|p| p.to_string_lossy().into_owned())
            })
            .filter_map(|p| {
                link_resolver
                    .resolve(&p)
                    .inspect_err(|e| log::error!("failed to resolve link: {p}: {e}"))
                    .map(|i| (p, i))
                    .ok()
            })
    });

    link_info_map.sort_by_key(|_, i| i.name.chars().next().map(|c| c.to_ascii_lowercase()));

    link_info_map
}
