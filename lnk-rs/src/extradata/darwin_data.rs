use crate::strings;

/// The DarwinDataBlock structure specifies an application identifier
/// that can be used instead of a link target IDList to install an
/// application when a shell link is activated.
#[derive(Clone, Debug)]
pub struct DarwinDataBlock {
    /// A NULL–terminated string, defined by the system default code
    /// page, which specifies an application identifier. This field
    /// SHOULD be ignored.
    darwin_data_ansi: String,
    /// An optional, NULL–terminated, Unicode string that specifies
    /// an application identifier.
    darwin_data_unicode: Option<String>,
}

impl DarwinDataBlock {
    /// A NULL–terminated string, defined by the system default code
    /// page, which specifies an application identifier. This field
    /// SHOULD be ignored.
    pub fn darwin_data_ansi(&self) -> &String {
        &self.darwin_data_ansi
    }

    /// An optional, NULL–terminated, Unicode string that specifies
    /// an application identifier.
    pub fn darwin_data_unicode(&self) -> &Option<String> {
        &self.darwin_data_unicode
    }
}

impl From<&[u8]> for DarwinDataBlock {
    fn from(data: &[u8]) -> Self {
        let darwin_data_ansi =
            strings::trim_nul_terminated_string(String::from_utf8_lossy(&data[0..260]));
        let darwin_data_unicode_raw =
            strings::trim_nul_terminated_string(String::from_utf8_lossy(&data[260..520]));
        let darwin_data_unicode = if darwin_data_unicode_raw.len() == 0 {
            None
        } else {
            Some(darwin_data_unicode_raw)
        };
        Self {
            darwin_data_ansi,
            darwin_data_unicode,
        }
    }
}
