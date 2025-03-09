use windows::Win32::{
    System::Com::CoTaskMemFree,
    UI::Shell::{IShellItem, SIGDN_DESKTOPABSOLUTEPARSING},
};

pub trait ShellItem {
    fn get_display_name(&self) -> windows_core::Result<String>;
}

impl ShellItem for IShellItem {
    fn get_display_name(&self) -> windows_core::Result<String> {
        let ptr = unsafe { self.GetDisplayName(SIGDN_DESKTOPABSOLUTEPARSING) }?;

        let path: Vec<u16> = unsafe {
            let mut len = 0;
            while (*ptr.0.add(len)) != 0 {
                len += 1;
            }
            std::slice::from_raw_parts(ptr.0, len + 1)
        }
        .to_vec();

        unsafe { CoTaskMemFree(Option::Some(ptr.0 as _)) };
        Ok(String::from_utf16_lossy(&path))
    }
}
