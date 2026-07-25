use std::sync::LazyLock;

use image::{
    ExtendedColorType, ImageResult,
    codecs::ico::{IcoEncoder, IcoFrame},
};
use winsafe::{
    HKEY, RegistryValue, SHGetKnownFolderPath, SysResult,
    co::{KEY, KF, KNOWNFOLDERID, REG_OPTION},
};

static BLANK_ICON_PATH: LazyLock<String> = LazyLock::new(|| {
    SHGetKnownFolderPath(&KNOWNFOLDERID::Windows, KF::NO_ALIAS, None).map_or_else(
        |_| r"C:\Windows\Blank.ico".to_owned(),
        |dir| format!("{dir}/Blank.ico"),
    )
});

pub fn set_shortcut_arrow(should_delete: bool) -> Result<(), Box<dyn std::error::Error>> {
    clear_is_shortcut().map_err(|e| format!("failed to clear IsShortcut registry value: {e}"))?;

    if should_delete {
        create_blank_icon().map_err(|e| format!("failed to create Blank.ico file: {e}"))?;
        set_shell_icon29(BLANK_ICON_PATH.as_ref())
            .map_err(|e| format!("failed to set Shell Icon 29 registry value: {e}").into())
    } else {
        remove_shell_icons_key()
            .map_err(|e| format!("failed to remove Shell Icon 29 registry key: {e}").into())
    }
}

fn create_blank_icon() -> ImageResult<()> {
    if infer::get_from_path(BLANK_ICON_PATH.as_str())
        .ok()
        .flatten()
        .is_some_and(|t| t.extension().eq("ico"))
    {
        return Ok(());
    }

    let sizes: [usize; 5] = [16, 24, 32, 48, 256];

    let frames = sizes
        .into_iter()
        .filter_map(|sz| {
            let buffer = Vec::with_capacity(sz * sz * 4);
            IcoFrame::as_png(&buffer, sz as u32, sz as u32, ExtendedColorType::Rgba8).ok()
        })
        .collect::<Vec<_>>();

    let output_file = std::fs::File::create(BLANK_ICON_PATH.as_str())?;

    IcoEncoder::new(output_file).encode_images(&frames)
}

fn set_shell_icon29(icon_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let sub_key = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Icons";

    let (key_guard, _) = HKEY::LOCAL_MACHINE.RegCreateKeyEx(
        sub_key,
        None,
        REG_OPTION::NON_VOLATILE,
        KEY::SET_VALUE | KEY::WRITE,
        None,
    )?;

    key_guard.RegSetValueEx(Some("29"), RegistryValue::Sz(icon_path.to_owned()))?;

    Ok(())
}

fn remove_shell_icons_key() -> Result<(), Box<dyn std::error::Error>> {
    let parent = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer";

    let (key_guard, _) = HKEY::LOCAL_MACHINE.RegCreateKeyEx(
        parent,
        None,
        REG_OPTION::NON_VOLATILE,
        KEY::WRITE,
        None,
    )?;

    key_guard.RegDeleteKey("Shell Icons")?;

    Ok(())
}

fn clear_is_shortcut() -> SysResult<()> {
    let keys = ["IE.AssocFile.URL", "InternetShortcut", "lnkfile"];

    for sub_key in &keys {
        let (key_guard, _) = HKEY::CLASSES_ROOT.RegCreateKeyEx(
            sub_key,
            None,
            REG_OPTION::NON_VOLATILE,
            KEY::SET_VALUE | KEY::WRITE,
            None,
        )?;

        key_guard.RegSetValueEx(
            Some("IsShortcut"),
            RegistryValue::Sz("\0".to_owned()), // 一个 null 字符
        )?;
    }
    Ok(())
}

#[test]
fn test_set_shortcut_arrow() {
    set_shortcut_arrow(true).expect("failed to set shortcut arrow");
    super::restart_explorer::restart_explorer();
}
