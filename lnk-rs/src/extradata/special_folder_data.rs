use byteorder::{ByteOrder, LE};

/// The SpecialFolderDataBlock structure specifies the location of a
/// special folder. This data can be used when a link target is a
/// special folder to keep track of the folder, so that the link target
/// IDList can be translated when the link is loaded.
#[derive(Clone, Copy, Debug)]
pub struct SpecialFolderDataBlock {
    /// A 32-bit, unsigned integer that specifies the folder integer ID.
    special_folder_id: u32,
    /// A 32-bit, unsigned integer that specifies the location of the
    /// ItemID of the first child segment of the IDList specified by
    /// SpecialFolderID. This value is the offset, in bytes, into the
    /// link target IDList.
    offset: u32,
}

impl SpecialFolderDataBlock {
    /// A 32-bit, unsigned integer that specifies the folder integer ID.
    pub fn special_folder_id(&self) -> u32 {
        self.special_folder_id
    }

    /// A 32-bit, unsigned integer that specifies the location of the
    /// ItemID of the first child segment of the IDList specified by
    /// SpecialFolderID. This value is the offset, in bytes, into the
    /// link target IDList.
    pub fn offset(&self) -> u32 {
        self.offset
    }
}

impl From<&[u8]> for SpecialFolderDataBlock {
    fn from(data: &[u8]) -> Self {
        let special_folder_id = LE::read_u32(data);
        let offset = LE::read_u32(&data[4..]);
        Self {
            special_folder_id,
            offset,
        }
    }
}
