use sal_core::error::Error;

///
/// Identifier of API `Query`'s
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, bincode::Encode, bincode::Decode)]
#[repr(u32)]
pub enum QueryId {
    DeviceStream = 16,
    DeviceInfo   = 20,
    DeviceDoc    = 24,
    BytesExample = 28,
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
impl Into<u32> for QueryId {
    fn into(self) -> u32 {
        self as u32
    }
}
