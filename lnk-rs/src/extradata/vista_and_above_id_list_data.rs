use byteorder::{ByteOrder, LE};
use log::debug;

use crate::linktarget::ItemID;

/// The VistaAndAboveIDListDataBlock structure specifies an alternate
/// IDList that can be used instead of the LinkTargetIDList structure
/// (section 2.2) on platforms that support it.
#[derive(Clone, Debug)]
pub struct VistaAndAboveIdListDataBlock {
    /// An IDList structure (section 2.2.1).
    id_list: Vec<ItemID>,
}

impl VistaAndAboveIdListDataBlock {
    /// Get the ID List
    pub fn id_list(&self) -> &Vec<ItemID> {
        &self.id_list
    }
}

impl From<&[u8]> for VistaAndAboveIdListDataBlock {
    fn from(data: &[u8]) -> Self {
        let mut id_list = Vec::new();
        let mut inner_data = &data[8..];
        let mut offset = 0;
        loop {
            // Check for terminator
            if LE::read_u16(&data[offset..]) == 0 {
                break;
            }

            // Read an ItemID
            let id = ItemID::from(inner_data);
            debug!("Read {:?}", id);
            let size = id.size;
            id_list.push(id);
            inner_data = &inner_data[(size as usize)..];
            offset += size as usize;
        }
        Self { id_list }
    }
}
