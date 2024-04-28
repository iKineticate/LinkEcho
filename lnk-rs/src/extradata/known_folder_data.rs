use byteorder::{ByteOrder, LE};

/// The KnownFolderDataBlock structure specifies the location of a
/// known folder. This data can be used when a link target is a
/// known folder to keep track of the folder so that the link target
/// IDList can be translated when the link is loaded.
#[derive(Clone, Copy, Debug)]
pub struct KnownFolderDataBlock {
    /// A value in GUID packet representation ([MS-DTYP] section
    /// 2.3.4.2) that specifies the folder GUID ID.
    known_folder_id: u128,
    /// A 32-bit, unsigned integer that specifies the location
    /// of the ItemID of the first child segment of the IDList specified
    /// by KnownFolderID. This value is the offset, in bytes, into the
    /// link target IDList.
    offset: u32,
}

impl KnownFolderDataBlock {
    /// A value in GUID packet representation ([MS-DTYP] section
    /// 2.3.4.2) that specifies the folder GUID ID.
    pub fn known_folder_id(&self) -> u128 {
        self.known_folder_id
    }

    /// A 32-bit, unsigned integer that specifies the location
    /// of the ItemID of the first child segment of the IDList specified
    /// by KnownFolderID. This value is the offset, in bytes, into the
    /// link target IDList.
    pub fn offset(&self) -> u32 {
        self.offset
    }
}

impl From<&[u8]> for KnownFolderDataBlock {
    fn from(data: &[u8]) -> Self {
        let known_folder_id = LE::read_u128(data);
        let offset = LE::read_u32(&data[16..]);
        Self {
            known_folder_id,
            offset,
        }
    }
}
