use std::collections::HashMap;
use std::env;
use std::sync::LazyLock;

use winsafe::{SHGetKnownFolderPath, co};

static ENV_MAP: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let vars: Vec<(&str, Option<co::KNOWNFOLDERID>, &str)> = vec![
        (
            "%commonprogramfiles(x86)%",
            Some(co::KNOWNFOLDERID::ProgramFilesCommonX86),
            "CommonProgramFiles(x86)",
        ),
        ("%commonprogramfiles(arm)%", None, "CommonProgramFiles(Arm)"),
        (
            "%programfiles(x86)%",
            Some(co::KNOWNFOLDERID::ProgramFiles),
            "ProgramFiles(x86)",
        ),
        ("%programfiles(arm)%", None, "ProgramFiles(Arm)"),
        (
            "%commonprogramfiles%",
            Some(co::KNOWNFOLDERID::ProgramFilesCommonX64),
            "CommonProgramFiles",
        ),
        (
            "%allusersprofile%",
            Some(co::KNOWNFOLDERID::ProgramData),
            "AllUsersProfile",
        ),
        (
            "%localappdata%",
            Some(co::KNOWNFOLDERID::LocalAppData),
            "LocalAppData",
        ),
        (
            "%programfiles%",
            Some(co::KNOWNFOLDERID::ProgramFiles),
            "ProgramFiles",
        ),
        (
            "%userprofile%",
            Some(co::KNOWNFOLDERID::Profile),
            "UserProfile",
        ),
        (
            "%programdata%",
            Some(co::KNOWNFOLDERID::ProgramData),
            "ProgramData",
        ),
        (
            "%systemroot%",
            Some(co::KNOWNFOLDERID::Windows),
            "SystemRoot",
        ),
        (
            "%appdata%",
            Some(co::KNOWNFOLDERID::RoamingAppData),
            "AppData",
        ),
        ("%cmdcmdline%", None, "CMDCMDLINE"),
        ("%windir%", Some(co::KNOWNFOLDERID::Windows), "WinDir"),
        ("%comspec%", None, "COMSPEC"),
        ("%public%", Some(co::KNOWNFOLDERID::Public), "Public"),
        ("%temp%", None, "TEMP"),
        ("%tmp%", None, "TMP"),
    ];

    let mut map = HashMap::with_capacity(vars.len());
    for (env_var, known_folder, fallback_env) in vars {
        let path = get_path(known_folder, fallback_env);
        map.insert(env_var.to_lowercase(), path);
    }
    map
});

pub fn resolve_env_vars(path: &str) -> String {
    if !path.starts_with('%') {
        return path.to_owned();
    }

    let env_path = path.to_lowercase();
    ENV_MAP
        .iter()
        .find_map(|(env, root)| {
            env_path
                .starts_with(env)
                .then(|| env_path.replacen(env, root, 1))
        })
        .unwrap_or(path.to_owned())
}

fn get_path(known_folder: Option<co::KNOWNFOLDERID>, fallback_env: &str) -> String {
    if let Some(id) = known_folder {
        SHGetKnownFolderPath(&id, co::KF::NO_ALIAS, None)
            .ok()
            .map(|p| p.to_string())
            .unwrap_or_else(|| read_env_var(fallback_env))
    } else {
        read_env_var(fallback_env)
    }
}

fn read_env_var(name: &str) -> String {
    env::var_os(name)
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}
