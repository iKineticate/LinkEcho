use std::fmt;

/// A PropertyStoreDataBlock structure specifies a set of properties
/// that can be used by applications to store extra data in the
/// shell link.
#[derive(Clone)]
pub struct PropertyStoreDataBlock {
    /// A serialized property storage structure ([MS-PROPSTORE] section 2.2).
    property_store: Vec<u8>,
}

impl PropertyStoreDataBlock {
    /// A serialized property storage structure ([MS-PROPSTORE] section 2.2).
    pub fn property_store(&self) -> &Vec<u8> {
        &self.property_store
    }
}

impl fmt::Debug for PropertyStoreDataBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PropertyStoreDataBlock {{ property_store: (serialized property storage structure) }}"
        )
    }
}

impl From<&[u8]> for PropertyStoreDataBlock {
    fn from(data: &[u8]) -> Self {
        Self {
            property_store: data.to_vec(),
        }
    }
}
