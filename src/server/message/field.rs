#![allow(unused)]

use std::usize;

use crate::server::Bytes;
///
/// Filed configuration for the [Message].build
/// 
/// Use such fields to specify a sequence of fields in the message built from values
/// 
/// ```ignire
/// //
/// // Configuring a fields squence in the message
/// Message::new(
///     &dbg,
///     vec![
///         FieldConf::U16Be,        // Transaction Identifier u16          , index 0
///         FieldConf::Const(vec![0x00, 0x00]),   // Protocol Identifier u16, index 1
///         FieldConf::U16Be,        // Length Field u16                    , index 2
///         FieldConf::Byte,         // Unit ID, u8                         , index 3
///         FieldConf::Byte,         // Function Code, u8                   , index 4
///         FieldConf::String,       // Bytes, Vec<u8>                      , index 5
///     ],
///     Terminator::new(),
/// );
/// //
/// // Build a message using defined fields configuration
/// let data = "Data pay load content";
/// let size = data.len();
/// let result = message.build(&[Field::U16(12), Field::Const, Field::U16(size), Field::Byte(1), Field::Byte(4), Field::String(data.to_owned())]);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum FieldConf {
    /// Const u8 to be converted to byte
    Const(Vec<u8>),
    /// Field configuration for variable u16 to be converted to byte
    U16Be,
    /// Field configuration for variable u16 to be converted to byte
    U16Le,
    /// Field configuration for variable u32 to be converted to byte
    U32Be,
    /// Field configuration for variable u32 to be converted to byte
    U32Le,
    /// Field configuration for variable u64 to be converted to byte
    U64Be,
    /// Field configuration for variable u64 to be converted to byte
    U64Le,
    /// Field configuration for variable u128 to be converted to byte
    U128Be,
    /// Field configuration for variable u128 to be converted to byte
    U128Le,

    /// Field configuration for variable i8 to be converted to byte
    I8Be,
    /// Field configuration for variable i8 to be converted to byte
    I8Le,
    /// Field configuration for variable i16 to be converted to byte
    I16Be,
    /// Field configuration for variable i16 to be converted to byte
    I16Le,
    /// Field configuration for variable i32 to be converted to byte
    I32Be,
    /// Field configuration for variable i32 to be converted to byte
    I32Le,
    /// Field configuration for variable i64 to be converted to byte
    I64Be,
    /// Field configuration for variable i64 to be converted to byte
    I64Le,
    /// Field configuration for variable i128 to be converted to byte
    I128Be,
    /// Field configuration for variable i128 to be converted to byte
    I128Le,

    /// Field configuration for variable f32 to be converted to byte
    F32Be,
    /// Field configuration for variable f32 to be converted to byte
    F32Le,
    /// Field configuration for variable f84 to be converted to byte
    F64Be,
    /// Field configuration for variable f84 to be converted to byte
    F64Le,
    /// Variable JSON String to be converted to the bytes
    Json,
    /// Variable String to be converted to the bytes
    String,
    /// Value passed by already converted to the bytes
    Byte,
    /// Value passed by already converted to the bytes
    Bytes,
}
///
/// Field kind for passing into `MessageParse`.build([]) 
#[derive(Debug, Clone, PartialEq)]
pub enum Field {
    /// Const u8 to be converted to byte
    Const,
    /// Variable u16 to be converted to byte
    U16(u16),
    /// Variable u32 to be converted to byte
    U32(u32),
    /// Variable u64 to be converted to byte
    U64(u64),
    /// Variable u128 to be converted to byte
    U128(u128),

    /// Variable i8 to be converted to byte
    I8(i8),
    /// Variable i16 to be converted to byte
    I16(i16),
    /// Variable i32 to be converted to byte
    I32(i32),
    /// Variable i64 to be converted to byte
    I64(i64),
    /// Variable i128 to be converted to byte
    I128(i128),

    /// Variable f32 to be converted to byte
    F32(f32),
    /// Variable f64 to be converted to byte
    F64(f64),
    /// Variable JSON String to be converted to the bytes
    Json(String),
    /// Variable String to be converted to the bytes
    String(String),
    /// Value passed by already converted to the bytes
    Byte(u8),
    /// Value passed by already converted to the bytes
    Bytes(Vec<u8>),
}
impl Field {
    pub fn to_be_bytes(&self) -> Bytes {
        match self {
            Field::Const => vec![],
            Field::U16(val) => {
                let [v0, v1] = val.to_be_bytes();
                vec![v0, v1]
            }
            Field::U32(val) => {
                let [v0, v1, v2, v3] = val.to_be_bytes();
                vec![v0, v1, v2, v3]
            }
            Field::U64(val) => {
                let [v0, v1, v2, v3, v4, v5, v6, v7] = val.to_be_bytes();
                vec![v0, v1, v2, v3, v4, v5, v6, v7]
            }
            Field::U128(val) => {
                let [v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15] = val.to_be_bytes();
                vec![v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15]
            }
            Field::I8(val) => {
                let [v0] = val.to_be_bytes();
                vec![v0]
            }
            Field::I16(val) => {
                let [v0, v1] = val.to_be_bytes();
                vec![v0, v1]
            }
            Field::I32(val) => {
                let [v0, v1, v2, v3] = val.to_be_bytes();
                vec![v0, v1, v2, v3]
            }
            Field::I64(val) => {
                let [v0, v1, v2, v3, v4, v5, v6, v7] = val.to_be_bytes();
                vec![v0, v1, v2, v3, v4, v5, v6, v7]
            }
            Field::I128(val) => {
                let [v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15] = val.to_be_bytes();
                vec![v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15]
            }
            Field::F32(val) => {
                let [v0, v1, v2, v3] = val.to_be_bytes();
                vec![v0, v1, v2, v3]
            }
            Field::F64(val) => {
                let [v0, v1, v2, v3, v4, v5, v6, v7] = val.to_be_bytes();
                vec![v0, v1, v2, v3, v4, v5, v6, v7]
            }
            Field::Json(val) => val.as_bytes().to_vec(),
            Field::String(val) => val.as_bytes().to_vec(),
            Field::Byte(val) => vec![*val],
            Field::Bytes(val) => val.to_vec(),
        }
    }
    pub fn as_u16(&self) -> &u16 {
        match self {
            Field::U16(val) => val,
            _ => panic!("Field.as_u16 | Field::U16 expected, but found {:?}", self)
        }
    }
    pub fn as_u32(&self) -> &u32 {
        match self {
            Field::U32(val) => val,
            _ => panic!("Field.as_u32 | Field::U32 expected, but found {:?}", self)
        }
    }
    pub fn as_u64(&self) -> &u64 {
        match self {
            Field::U64(val) => val,
            _ => panic!("Field.as_u64 | Field::U64 expected, but found {:?}", self)
        }
    }
    pub fn as_u128(&self) -> &u128 {
        match self {
            Field::U128(val) => val,
            _ => panic!("Field.as_u128 | Field::U128 expected, but found {:?}", self)
        }
    }
    pub fn as_i8(&self) -> &i8 {
        match self {
            Field::I8(val) => val,
            _ => panic!("Field.as_i8 | Field::I8 expected, but found {:?}", self)
        }
    }
    pub fn as_i16(&self) -> &i16 {
        match self {
            Field::I16(val) => val,
            _ => panic!("Field.as_i16 | Field::I16 expected, but found {:?}", self)
        }
    }
    pub fn as_i32(&self) -> &i32 {
        match self {
            Field::I32(val) => val,
            _ => panic!("Field.as_i32 | Field::I32 expected, but found {:?}", self)
        }
    }
    pub fn as_i64(&self) -> &i64 {
        match self {
            Field::I64(val) => val,
            _ => panic!("Field.as_i64 | Field::I64 expected, but found {:?}", self)
        }
    }
    pub fn as_i128(&self) -> &i128 {
        match self {
            Field::I128(val) => val,
            _ => panic!("Field.as_i128 | Field::I128 expected, but found {:?}", self)
        }
    }
    pub fn as_f32(&self) -> &f32 {
        match self {
            Field::F32(val) => val,
            _ => panic!("Field.as_f32 | Field::F32 expected, but found {:?}", self)
        }
    }
    pub fn as_f64(&self) -> &f64 {
        match self {
            Field::F64(val) => val,
            _ => panic!("Field.as_f64 | Field::F64 expected, but found {:?}", self)
        }
    }
    pub fn as_json(&self) -> &String {
        match self {
            Field::Json(val) => val,
            _ => panic!("Field.as_string | Field::String expected, but found {:?}", self)
        }
    }
    pub fn as_string(&self) -> &String {
        match self {
            Field::String(val) => val,
            _ => panic!("Field.as_string | Field::String expected, but found {:?}", self)
        }
    }
    pub fn as_byte(&self) -> &u8 {
        match self {
            Field::Byte(val) => val,
            _ => panic!("Field.as_byte | Field::Byte expected, but found {:?}", self)
        }
    }
    pub fn as_bytes(&self) -> &Vec<u8> {
        match self {
            Field::Bytes(val) => val,
            _ => panic!("Field.as_bytes | Field::Bytes expected, but found {:?}", self)
        }
    }
}
