//!
//! # Parse/Build messages transmitted over the socket.
//! 
//! Message may contains any number and combinations of fields:
//! - Start byte(s) - any number identifies the start of the message
//! - Fixed length field
//! - Variable length field
//! 
//! All message fields should be defined statically,
//! dynamic chancheg of the filds configuration is not supported
//! 
//! For example content of `Data` field can be encoded using variations of `Kind`, `Cot` and `Name`,
//! payload data length specified in the `Size` filed
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
mod cot;
mod field_id;
mod field;
mod find_field;
mod fixed_field;
mod field_content;
mod message;
mod sized_field;
mod terminator;

pub use cot::*;
pub use field_id::*;
pub use field::*;
pub use find_field::*;
pub use fixed_field::*;
pub use field_content::*;
pub use message::*;
pub use sized_field::*;
pub use terminator::*;
