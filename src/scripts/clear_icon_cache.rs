use crate::{t, utils::notify};
use anyhow::{Result, anyhow};
use log::*;
use std::env;
use std::ffi::OsStr;
use std::path::Path;

pub fn clear_icon_cache() {
    let local_app_data = env::var("LOCALAPPDATA").expect("Failed to get the local app data path");
    let explorer_path = Path::new(&local_app_data).join("Microsoft\\Windows\\Explorer");

    if !explorer_path.exists() {
        return notify(&t!("EXPLORER_NOT_EXIST"));
    }

    if !explorer_path.is_dir() {
        return notify(&t!("ERROR_ITERTATOR_EXPLORER"));
    }

    if let Ok(entries) = std::fs::read_dir(&explorer_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if should_delete_file(&path) {
                if std::fs::remove_file(&path).is_err() {
                    let text = format!("{}\n{path:?}", t!("ERROR_DELETE_ICON_DB"));
                    log::error!("{text}");
                    return notify(&text);
                }
            }
        }

        if let Err(e) = restart_explorer() {
            trace!("{}\n{e}", t!("ERROR_RESTART_EXPLORER"));
            notify(&t!("ERROR_RESTART_EXPLORER"));
        } else {
            info!("{}", t!("SUCCESS_CLEAR_ICON_CACHE"));
            notify(&t!("SUCCESS_CLEAR_ICON_CACHE"));
        }
    } else {
        error!("{}", t!("ERROR_ITERTATOR_EXPLORER"));
        notify(&t!("ERROR_ITERTATOR_EXPLORER"));
    }
}

fn should_delete_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    if !path.extension().is_some_and(|e| e == "db") {
        return false;
    }

    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|n| n.starts_with("iconcache_") || n.starts_with("thumbcache_"))
}

fn restart_explorer() -> Result<()> {
    // use std::time::Duration;
    use restart_explorer::{
        core::{
            // operations::explorer::wait_for_explorer_stable,
            // operations::location::get_explorer_windows,
            operations::process::{kill_process_by_name, start_process},
        },
        // infrastructure::windows_os::windows_api::Win32WindowApi,
    };
    use windows::Win32::System::Com::{
        COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx,
    };

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
    };

    // let window_api = Win32WindowApi;

    // let windows = get_explorer_windows(&window_api);

    kill_process_by_name("explorer.exe");
    start_process("explorer.exe");

    // let mut already_open_explorer_windows: Vec<isize> = vec![];

    // if let Err(e) = wait_for_explorer_stable(Duration::from_secs(10)) {
    //     return Err(anyhow!("{e}"));
    // } else {
    //     for window in windows {
    //         if let Some(id) = open_location(&window, &already_open_explorer_windows, &window_api) {
    //             already_open_explorer_windows.push(id);
    //         }
    //     }
    // }

    Ok(())
}
