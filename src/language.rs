use windows::Win32::GetSystemDefaultLCID;

pub fn set_locale() {
    let sys_lcid = unsafe { GetSystemDefaultLCID().0 };

    match sys_lcid {
        0x7804 => rust_i18n::set_locale("zh"), // Chinese
        0x0804 => rust_i18n::set_locale("zh"), // Chinese (Simplified, China)
        0x0004 => rust_i18n::set_locale("zh"), // Chinese (Simplified)
        0x7C04 => rust_i18n::set_locale("zh"), // Chinese (Traditional)
        0x0C04 => rust_i18n::set_locale("zh"), // Chinese (Traditional, Hong Kong SAR)
        0x1404 => rust_i18n::set_locale("zh"), // Chinese (Traditional, Macao SAR)
        0x1004 => rust_i18n::set_locale("zh"), // Chinese (Simplified, Singapore)
        0x0404 => rust_i18n::set_locale("zh"), // Chinese (Traditional, Taiwan)
        _ => rust_i18n::set_locale("en"),
    };
}