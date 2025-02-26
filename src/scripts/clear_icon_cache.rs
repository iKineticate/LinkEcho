use crate::{t, utils::{notify, write_log}};
use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

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
                    let text = format!("{}\n{}", t!("ERROR_DELETE_ICON_DB"), path.display());
                    write_log(text.clone()).expect("Failure to write to the log");
                    return notify(&text);
                }
            }
        }

        if restart_explorer() {
            write_log(t!("SUCCESS_CLEAR_ICON_CACHE").into_owned())
                .expect("Failure to write to the log");
            notify(&t!("SUCCESS_CLEAR_ICON_CACHE"));
        } else {
            notify(&t!("ERROR_RESTART_EXPLORER"));
        }
    } else {
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

fn restart_explorer() -> bool {
    let script = r#"
if (-not (Get-Process -Name explorer -ErrorAction SilentlyContinue)) {
    Start-Sleep -Seconds 2
    if (-not (Get-Process -Name explorer -ErrorAction SilentlyContinue)) {
        Start-Process explorer
    }
}    
"#;

    let status = Command::new("PowerShell")
        .arg("-Command")
        .args(["taskkill", "/IM", "explorer.exe", "/F;", "explorer"])
        .arg(script)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("failed to execute process");

    status.success()
}
