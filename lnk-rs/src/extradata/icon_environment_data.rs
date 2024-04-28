use crate::strings;

/// The IconEnvironmentDataBlock structure specifies the path to an
/// icon. The path is encoded using environment variables, which makes
/// it possible to find the icon across machines where the locations
/// vary but are expressed using environment variables.
#[derive(Clone, Debug)]
pub struct IconEnvironmentDataBlock {
    /// A NULL-terminated string, defined by the system default code
    /// page, which specifies a path that is constructed with
    /// environment variables.
    target_ansi: String,
    /// An optional, NULL-terminated, Unicode string that specifies a
    /// path that is constructed with environment variables.
    target_unicode: Option<String>,
}

impl IconEnvironmentDataBlock {
    /// A NULL-terminated string, defined by the system default code
    /// page, which specifies a path that is constructed with
    /// environment variables.
    pub fn target_ansi(&self) -> &String {
        &self.target_ansi
    }

    /// An optional, NULL-terminated, Unicode string that specifies a
    /// path that is constructed with environment variables.
    pub fn target_unicode(&self) -> &Option<String> {
        &self.target_unicode
    }
}

impl From<&[u8]> for IconEnvironmentDataBlock {
    fn from(data: &[u8]) -> Self {
        let target_ansi =
            strings::trim_nul_terminated_string(String::from_utf8_lossy(&data[0..260]));
        let target_unicode_raw =
            strings::trim_nul_terminated_string(String::from_utf8_lossy(&data[260..520]));
        let target_unicode = if target_unicode_raw.len() == 0 {
            None
        } else {
            Some(target_unicode_raw)
        };
        Self {
            target_ansi,
            target_unicode,
        }
    }
}
