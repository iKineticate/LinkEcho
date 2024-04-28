use byteorder::{ByteOrder, LE};

use crate::strings;

/// The TrackerDataBlock structure specifies data that can be used to
/// resolve a link target if it is not found in its original location
/// when the link is resolved. This data is passed to the Link
/// Tracking service [MS-DLTW] to find the link target.
#[derive(Clone, Debug)]
pub struct TrackerDataBlock {
    /// A NULL–terminated character string, as defined by the system default
    /// code page, which specifies the NetBIOS name of the machine where
    /// the link target was last known to reside.
    machine_id: String,
    /// Two values in GUID packet representation ([MS-DTYP] section 2.3.4.2)
    /// that are used to find the link target with the Link Tracking service,
    /// as described in [MS-DLTW].
    droid: [u128; 2],
    /// Two values in GUID packet representation that are used to find the
    /// link target with the Link Tracking service
    droid_birth: [u128; 2],
}

impl TrackerDataBlock {
    /// Get the machine ID
    pub fn machine_id(&self) -> &String {
        &self.machine_id
    }

    /// Get the droid GUIDs
    pub fn droid(&self) -> &[u128; 2] {
        &self.droid
    }

    /// Get the droid birth GUIDs
    pub fn droid_birth(&self) -> &[u128; 2] {
        &self.droid_birth
    }
}

impl From<&[u8]> for TrackerDataBlock {
    fn from(data: &[u8]) -> Self {
        let machine_id =
            strings::trim_nul_terminated_string(String::from_utf8_lossy(&data[8..]).to_string());
        let droid_1 = LE::read_u128(&data[24..]);
        let droid_2 = LE::read_u128(&data[40..]);
        let droid_birth_1 = LE::read_u128(&data[56..]);
        let droid_birth_2 = LE::read_u128(&data[72..]);

        Self {
            machine_id,
            droid: [droid_1, droid_2],
            droid_birth: [droid_birth_1, droid_birth_2],
        }
    }
}
