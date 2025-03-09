use windows::Win32::{System::Ole::IEnumVARIANT, UI::Shell::IShellWindows};
use windows_core::Interface;

use super::enum_variant::EnumVariant;

pub trait ShellWindows {
    fn new_enum_variant(self) -> windows_core::Result<impl EnumVariant>;
}

impl ShellWindows for IShellWindows {
    fn new_enum_variant(self) -> windows_core::Result<impl EnumVariant> {
        let unk_enum = unsafe { self._NewEnum() }?;
        let enum_variant = unk_enum.cast::<IEnumVARIANT>()?;
        Ok(enum_variant)
    }
}
