use windows::Win32::System::Ole::IEnumVARIANT;

pub trait EnumVariant {
    fn next(
        &self,
        rgvar: &mut [windows_core::VARIANT],
        pceltfetched: *mut u32,
    ) -> windows_core::HRESULT;
}

impl EnumVariant for IEnumVARIANT {
    fn next(
        &self,
        rgvar: &mut [windows_core::VARIANT],
        pceltfetched: *mut u32,
    ) -> windows_core::HRESULT {
        unsafe { self.Next(rgvar, pceltfetched) }
    }
}
