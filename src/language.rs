use windows::Win32::Globalization::GetSystemDefaultLCID;
pub const ZH: u32 = 0x7804; // Chinese
pub const ZH_HANS: u32 = 0x0004; // Chinese (Simplified)
pub const ZH_CN: u32 = 0x0804; // Chinese (Simplified, China)
pub const ZH_SG: u32 = 0x1004; // Chinese (Simplified, Singapore)
pub const ZH_HANT: u32 = 0x7C04; // Chinese (Traditional)
pub const ZH_HK: u32 = 0x0C04; // Chinese (Traditional, Hong Kong SAR)
pub const ZH_MO: u32 = 0x1404; // Chinese (Traditional, Macao SAR)
pub const ZH_TW: u32 = 0x0404; // Chinese (Traditional, Taiwan)

pub fn set_locale() {
    let sys_lcid = unsafe { GetSystemDefaultLCID() };

    match sys_lcid {
        ZH => rust_i18n::set_locale("zh-CN"),
        ZH_CN => rust_i18n::set_locale("zh-CN"),
        ZH_HANS => rust_i18n::set_locale("zh-CN"),
        ZH_HANT => rust_i18n::set_locale("zh-CN"),
        ZH_HK => rust_i18n::set_locale("zh-CN"),
        ZH_MO => rust_i18n::set_locale("zh-CN"),
        ZH_SG => rust_i18n::set_locale("zh-CN"),
        ZH_TW => rust_i18n::set_locale("zh-CN"),
        _ => rust_i18n::set_locale("en"),
    };
}