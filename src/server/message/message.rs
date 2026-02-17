//!
//! # Messages transmitted over socket.
//! 
//! - `Data` can be encoded using varius data `Kind`, `Size` and payload data
//! 
//! - Message format
//!     Field name | Start |  Id   | Kind | Cot  | Name |  Size  | Data       |
//!     ---        |  ---  |  ---  | ---  | ---  | ---  |  ---   | ---        |
//!     Data type  |  u8   | u32   | u8   | u8   | u32  | u32    | [u8; Size] |
//!     Value      |  22   | 111   | 02   | 04   | 12   | xxx    | [..., ...] |
//!     
//!     - Start - Each message starts with SYN (22)
//!     - Id - Message id = 111
//!     - Kind - The `Kind` of the data stored in the `Data` field is 02 - `Bytes`
//!     - Cot - Cause of transmission is 03 - `Cot::Act`
//!     - Name - User name of Message / Event / Request
//!     - Size - The length of the `Data` field - xxx bytes
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
//!     - 40, String
//!     - 48, Timestamp
//!     - 49, Duration
//!     - .., ...
//! 
use sal_core::{dbg::Dbg, error::Error};
use super::{Field, FieldConf};
///
/// 
pub type Bytes = Vec<u8>;
///
/// Parse [Message] from structured bytes
pub trait MessageParse<FieldIn, FieldOut, Out> {
    ///
    /// Extracting some pattern from input `bytes`
    fn parse(&mut self, bytes: Bytes) -> Result<(FieldIn, FieldOut, Bytes), Error>;
}
///
/// Socket [Message] | Parse / Build structured bytes
/// 
/// - Message format
///     Field name | Start |  Id   | Kind | Cot  | Name |  Size  | Data       |
///     ---        |  ---  |  ---  | ---  | ---  | ---  |  ---   | ---        |
///     Data type  |  u8   | u32   | u8   | u8   | u32  | u32    | [u8; Size] |
///     Value      |  22   | 111   | 02   | 04   | 12   | xxx    | [..., ...] |
pub struct Message<FieldIn, FieldOut> {
    build: Vec<FieldConf>,
    parse: Box<dyn MessageParse<FieldIn, FieldOut, Bytes>>,
    remainder: Bytes,
    dbg: Dbg,
}

//
//
impl<FieldIn, FieldOut> std::fmt::Debug for Message<FieldIn, FieldOut> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("dbg", &self.dbg)
            .field("build", &self.build)
            .finish()
    }
}
//
//
impl<FieldIn, FieldOut> Message<FieldIn, FieldOut> {
    ///
    /// Returns `Message` new instance 
    pub fn new(
        parent: impl Into<String>,
        build: Vec<FieldConf>,
        parse: impl MessageParse<FieldIn, FieldOut, Bytes> + 'static
    ) -> Self {
        Self {
            dbg: Dbg::new(parent.into(), "Message"),
            build,
            parse: Box::new(parse),
            remainder: vec![],
        }
    }
    ///
    /// Returns message built according to specified fields and passed `bytes`
    pub fn build(&mut self, fields: &[Field]) -> Vec<u8> {
        self.build.iter().zip(fields).fold(vec![], |mut acc, (conf, field)| {
            match conf {
                FieldConf::Const(bytes) => acc.extend(bytes),
                _ => acc.extend(field.to_be_bytes()),
            }
            acc
        })
    }
    ///
    /// Extracting [Message] fields from the input bytes
    /// - returns `Id`, `Kind`, `Size` & `Bytes` following by the `Size`
    /// - call this method multiple times, until the end of message
    pub fn parse(&mut self, bytes: Bytes) -> Result<(FieldIn, FieldOut), Error> {
        let bytes = [std::mem::take(&mut self.remainder), bytes].concat();
        match self.parse.parse(bytes) {
            Ok((din, dout, remainder)) => {
                self.remainder = remainder;
                Ok((din, dout))
            }
            Err(err) => Err(Error::new(&self.dbg, "parse").pass(err)),
        }
    }
}
