use sal_core::error::Error;
use serde::Deserialize;
use crate::server::BINCODE_CONFIG;

///
/// Wrapper for all variants of API [Query]'s
#[derive(Debug, Clone, Deserialize, bincode::Decode)]
pub enum Query {
    Empty,
    /// TODO: To be replaced with real Query
    DeviceStream(DeviceStreamQuery),
    /// TODO: To be replaced with real Query
    DeviceInfo(DeviceInfoQuery),
    /// TODO: To be replaced with real Query
    DeviceDoc(DeviceDocQuery),
    /// TODO: To be replaced with real Query
    BytesExample(BytesExampleQuery),
    ///
    /// Used for testing only
    #[allow(unused)]
    TestString(String),
}
//
//
impl Query {
    ///
    /// Returns [Query] parsed from JSON `bytes`
    pub fn from_json(bytes: &[u8]) -> Result<Self, Error> {
        match serde_json::from_slice(&bytes) {
            Ok(query) => Ok(query),
            Err(err) => Err(Error::new("Query", "from_json").pass(err.to_string())),
        }
    }
    ///
    /// Returns [Query] parsed from raw `bytes` using `bincode::Decode`
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        bincode::decode_from_slice(bytes, BINCODE_CONFIG)
            .map(|(v, _)| v)
            .map_err(|err| Error::new("Query", "from_bytes").pass(err.to_string()))
    }
}
///
/// Extract request [Query] variant or returns error
macro_rules! extract {
    ($e:expr, $p:path) => {
        match $e {
            $p(value) => Ok(value),
            _ => Err(()),
        }
    };
}
pub(crate) use extract;

///
/// Request for `DeviceStream`
#[derive(Debug, Clone, Deserialize, bincode::Decode)]
pub struct DeviceStreamQuery {
    #[serde(rename="devId")]
    pub dev_id: String,
}
///
/// Request for `DeviceInfo`
#[derive(Debug, Clone, Deserialize, bincode::Decode)]
pub struct DeviceInfoQuery {
    #[serde(rename="devId")]
    pub dev_id: String,
}
///
/// Request for `DeviceInfo`
#[derive(Debug, Clone, Deserialize, bincode::Decode)]
pub struct DeviceDocQuery {
    #[serde(rename="devId")]
    pub dev_id: String,
}
///
/// Request example with `Content::Bytes`
#[derive(Debug, Clone, Deserialize, bincode::Decode)]
pub struct BytesExampleQuery {
    val: f64,
    name: String,
    data: Vec<Pt>,
}
#[derive(Debug, Clone, Deserialize, bincode::Decode)]
pub struct Pt {
    x: f64,
    y: f64,
}
