use crate::strings;

/// The EnvironmentVariableDataBlock structure specifies a path to
/// environment variable information when the link target refers to
/// a location that has a corresponding environment variable.
#[derive(Clone, Debug)]
pub struct EnvironmentVariableDataBlock {
    /// A NULL-terminated string, defined by the system default code
    /// page, which specifies a path to environment variable information.
    target_ansi: String,
    /// An optional, NULL-terminated, Unicode string that specifies a path
    /// to environment variable information.
    target_unicode: Option<String>,
}

impl EnvironmentVariableDataBlock {
    /// A NULL-terminated string, defined by the system default code
    /// page, which specifies a path to environment variable information.
    pub fn target_ansi(&self) -> &String {
        &self.target_ansi
    }

    /// An optional, NULL-terminated, Unicode string that specifies a path
    /// to environment variable information.
    pub fn target_unicode(&self) -> &Option<String> {
        &self.target_unicode
    }
}

impl From<&[u8]> for EnvironmentVariableDataBlock {
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
