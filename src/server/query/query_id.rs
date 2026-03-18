use sal_core::error::Error;

///
/// Identifier of API `Query`'s
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, bincode::Encode, bincode::Decode)]
#[repr(u32)]
pub enum QueryId {
    None         = 00,
    Calculus     = 16,
    /// TODO; To be deleted, just for example
    DeviceStream = 20,
    /// TODO; To be deleted, just for example
    DeviceInfo   = 24,
    /// TODO; To be deleted, just for example
    DeviceDoc    = 28,
    /// TODO; To be deleted, just for example
    BytesExample = 32,
}
//
//
impl QueryId {
    ///
    /// Returns [Query] from `bytes`
    pub fn from_be_bytes(bytes: &[u8]) -> Result<Self, Error> {
        match bytes {
            [b0, b1, b2, b3] | [b0, b1, b2, b3, ..] => {
                match u32::from_be_bytes([*b0, *b1, *b2, *b3]) {
                    val if val == Self::Calculus as u32 => Ok(Self::Calculus),
                    val if val == Self::DeviceStream as u32 => Ok(Self::DeviceStream),
                    val if val == Self::DeviceInfo as u32   => Ok(Self::DeviceInfo),
                    val if val == Self::DeviceDoc as u32    => Ok(Self::DeviceDoc),
                    val if val == Self::BytesExample as u32 => Ok(Self::BytesExample),
                    _ => Err(Error::new("Query", "from_be_bytes").err(format!("Can't parse from bytes {:?}", &bytes[..12]))),
                }
            }
            [] => Err(Error::new("Query", "from_be_bytes").err("Can't parse u32, empty bytes")),
            [..] => Err(Error::new("Query", "from_be_bytes").err(format!("Can't parse u32 from bytes {:?}", bytes))),
        }
    }
}
//
//
impl From<QueryId> for u32 {
    fn from(val: QueryId) -> Self {
        val as u32
    }
}
