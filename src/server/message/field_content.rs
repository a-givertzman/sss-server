//!
//! # Messages transmitted over socket.
//! 
//! - Data can be encoded using varius data `Kind`, `Size` and payload Data
//! 
//! - Message format
//!     Field name | Start | Kind |  Size  | Data |
//!     ---       |  ---  | ---  |  ---   | ---  |
//!     Data type |  u8   | u8   | u32    | [u8; Size] |
//!     Value     |  22   | StringValue | xxx    | [..., ...]  |
//!     
//!     - Start - Each message starts with SYN (22)
//!     - Kind - The `Kind` of the data stored in the `Data` field, refer to
//!     - Size - The length of the `Data` field in bytes
//!     - Data - Data structured depending on it `Kind`
//! 
//! - `Kind` of data
//!     - 00, Any
//!     - 01, Empty
//!     - 02, Bytes
//!     - 08, Bool
//!     - 16, UInt16
//!     - 17, UInt32
//!     - 18, UInt64
//!     - 24, Int16
//!     - 25, Int32
//!     - 26, Int64
//!     - 32, F32
//!     - 33, F64
//!     - 38, Json
//!     - 40, String
//!     - 48, Timestamp
//!     - 49, Duration
//!     - .., ...
//! 
use sal_core::error::Error;
///
/// Internal Kind of Message
/// - Used for build / parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
#[repr(u8)]
pub enum Content {
    Any       = 00,
    Bool      = 08,
    Bytes     = 02,
    Duration  = 49,
    Empty     = 01,
    F32       = 32,
    F64       = 33,
    I16       = 24,
    I32       = 25,
    I64       = 26,
    Json      = 38,
    String    = 40,
    Timestamp = 48,
    U16       = 16,
    U32       = 17,
    U64       = 18,
}
//
//
impl Content {
    // ///
    // /// Returns bytes of the `MessageKund` variant    
    // pub fn to_bytes(&self) -> u8 {
    // }
}
impl TryFrom<&[u8]> for Content {
    type Error = Error;
    ///
    /// Returns [MessageKind] converted from `bytes`
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        match bytes {
            [00] => Ok(Self::Any),
            [08] => Ok(Self::Bool),
            [02] => Ok(Self::Bytes),
            [49] => Ok(Self::Duration),
            [01] => Ok(Self::Empty),
            [32] => Ok(Self::F32),
            [33] => Ok(Self::F64),
            [24] => Ok(Self::I16),
            [25] => Ok(Self::I32),
            [26] => Ok(Self::I64),
            [38] => Ok(Self::Json),
            [40] => Ok(Self::String),
            [48] => Ok(Self::Timestamp),
            [16] => Ok(Self::U16),
            [17] => Ok(Self::U32),
            [18] => Ok(Self::U64),
            [..] => Err(Error::new("MessageKind", "from_bytes").err(format!("Wrong or Empty input: {:?}", &bytes[..16]))),
        }
    }
}
impl From<Content> for u8 {
    ///
    /// Returns u8 representation of the [MessageKind]
    fn from(val: Content) -> Self {
        val as u8
    }
}
