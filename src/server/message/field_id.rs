use sal_core::error::Error;

///
/// Id field
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldId(pub u32);
impl FieldId {
    // ///
    // /// Return the memory representation of this `value` as a byte array in big-endian (network) byte order.
    // pub fn to_be_bytes(&self) -> [u8; 4] {
    //     self.0.to_be_bytes()
    // }
    // ///
    // /// Returns field syze in bytes 
    // pub fn len(&self) -> usize {
    //     size_of::<u32>()
    // }
    pub fn from_be_bytes(bytes: &[u8]) -> Result<Self, Error> {
        match bytes {
            [b0, b1, b2, b3] => Ok(Self(u32::from_be_bytes([*b0, *b1, *b2, *b3]))),
            [b0, b1, b2, b3, ..] => Ok(Self(u32::from_be_bytes([*b0, *b1, *b2, *b3]))),
            _ => Err(Error::new("FieldId", "from_bytes").err(format!("Can't parse 'FieldId' u32 from bytes {:?}", &bytes[..16]))),
        }
    }
}
