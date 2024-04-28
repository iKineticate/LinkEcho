/// The ShimDataBlock structure specifies the name of a shim that can
/// be applied when activating a link target.
#[derive(Clone, Debug)]
pub struct ShimDataBlock {
    /// A Unicode string that specifies the name of a shim layer to apply
    /// to a link target when it is being activated.
    layer_name: String,
}

impl ShimDataBlock {
    /// A Unicode string that specifies the name of a shim layer to apply
    /// to a link target when it is being activated.s
    pub fn layer_name(&self) -> &String {
        &self.layer_name
    }
}

impl From<&[u8]> for ShimDataBlock {
    fn from(value: &[u8]) -> Self {
        let layer_name = String::from_utf8_lossy(value).to_string();
        Self { layer_name }
    }
}
