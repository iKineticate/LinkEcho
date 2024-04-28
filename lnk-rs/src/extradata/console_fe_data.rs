use byteorder::{ByteOrder, LE};

/// The ConsoleFEDataBlock structure specifies the code page to use
/// for displaying text when a link target specifies an application
/// that is run in a console window.
#[derive(Clone, Copy, Debug)]
pub struct ConsoleFEDataBlock {
    /// A 32-bit, unsigned integer that specifies a code page language
    /// code identifier. For details concerning the structure and
    /// meaning of language code identifiers, see [MS-LCID].
    code_page: u32,
}

impl ConsoleFEDataBlock {
    /// A 32-bit, unsigned integer that specifies a code page language
    /// code identifier. For details concerning the structure and
    /// meaning of language code identifiers, see [MS-LCID].
    pub fn code_page(&self) -> u32 {
        self.code_page
    }
}

impl From<&[u8]> for ConsoleFEDataBlock {
    fn from(data: &[u8]) -> Self {
        let code_page = LE::read_u32(data);
        Self { code_page }
    }
}
