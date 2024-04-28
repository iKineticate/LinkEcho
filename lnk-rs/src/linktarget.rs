use std::fmt;

use byteorder::{ByteOrder, LE};
#[allow(unused)]
use log::{debug, error, info, trace, warn};

/// The LinkTargetIDList structure specifies the target of the link. The presence of this optional
/// structure is specified by the HasLinkTargetIDList bit (LinkFlagssection 2.1.1) in the
/// ShellLinkHeader(section2.1).
#[derive(Clone, Debug)]
pub struct LinkTargetIdList {
    /// The size, in bytes, of the IDList field.
    pub size: u16,
    /// A stored IDList structure (section 2.2.1), which contains the item ID list. An IDList
    /// structure conforms to the following ABNF \[RFC5234\]:
    ///   `IDLIST = *ITEMID TERMINALID`
    id_list: Vec<ItemID>,
}

impl LinkTargetIdList {
    /// A stored IDList structure (section 2.2.1), which contains the item ID list.
    pub fn id_list(&self) -> &Vec<ItemID> {
        &self.id_list
    }
}

impl Default for LinkTargetIdList {
    fn default() -> Self {
        Self {
            size: 0,
            id_list: Vec::new(),
        }
    }
}

impl From<&[u8]> for LinkTargetIdList {
    /// Read data into this struct from a `[u8]`.
    fn from(data: &[u8]) -> Self {
        let mut id_list = Self::default();
        id_list.size = LE::read_u16(&data[0..]);
        trace!("ID List size: {}", id_list.size);
        let mut inner_data = &data[2..(id_list.size as usize)];
        assert!(inner_data.len() == id_list.size as usize - 2);
        let mut read_bytes = 2;
        while read_bytes < id_list.size {
            // Read an ItemID
            let id = ItemID::from(inner_data);
            debug!("Read {:?}", id);
            let size = id.size;
            id_list.id_list.push(id);
            inner_data = &inner_data[(size as usize)..];
            read_bytes += size;
        }
        id_list
    }
}

impl Into<Vec<u8>> for LinkTargetIdList {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::new();

        let size = 2u16;
        LE::write_u16(&mut data[0..2], size);
        for id in self.id_list {
            let mut other_data = id.into();
            data.append(&mut other_data);
        }

        data
    }
}

/// The stored IDList structure specifies the format of a persisted item ID list.
#[derive(Clone)]
pub struct ItemID {
    /// A 16-bit, unsigned integer that specifies the size, in bytes, of the ItemID structure,
    /// including the ItemIDSize field.
    pub(crate) size: u16,
    /// The shell data source-defined data that specifies an item.
    data: Vec<u8>,
}

impl ItemID {
    /// The shell data source-defined data that specifies an item.
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}

impl Default for ItemID {
    fn default() -> Self {
        Self {
            size: 0,
            data: Vec::new(),
        }
    }
}

impl fmt::Debug for ItemID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ItemID (raw data size {})", self.size)
    }
}

impl From<&[u8]> for ItemID {
    fn from(data: &[u8]) -> Self {
        let mut id = Self::default();

        id.size = LE::read_u16(data);
        id.data = Vec::from(&data[2..(id.size as usize)]);

        id
    }
}

impl Into<Vec<u8>> for ItemID {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::new();

        assert_eq!(self.data.len() as u16 + 2, self.size);

        LE::write_u16(&mut data, self.size);
        let mut other_data = self.data.clone();
        data.append(&mut other_data);

        data
    }
}
