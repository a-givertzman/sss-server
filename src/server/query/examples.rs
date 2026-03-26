//!
//! A list of queries for the server for example
//! 
//! TODO: To be deleted !
//! 
use bincode::Decode;
use serde::{Serialize, Deserialize};

///
/// Request for `DeviceStream`
/// TODO: To be deleted !
#[derive(Debug, Clone, Serialize, Deserialize, Decode)]
pub struct DeviceStreamQuery {
    #[serde(rename="devId")]
    pub dev_id: String,
}
///
/// Request for `DeviceInfo`
/// TODO: To be deleted !
#[derive(Debug, Clone, Serialize, Deserialize, Decode)]
pub struct DeviceInfoQuery {
    #[serde(rename="devId")]
    pub dev_id: String,
}
///
/// Request for `DeviceInfo`
/// TODO: To be deleted !
#[derive(Debug, Clone, Serialize, Deserialize, Decode)]
pub struct DeviceDocQuery {
    #[serde(rename="devId")]
    pub dev_id: String,
}
///
/// Request example with `Content::Bytes`
/// TODO: To be deleted !
#[derive(Debug, Clone, Serialize, Deserialize, Decode)]
pub struct BytesExampleQuery {
    val: f64,
    name: String,
    data: Vec<Pt>,
}
///
/// TODO: To be deleted !
#[derive(Debug, Clone, Serialize, Deserialize, Decode)]
pub struct Pt {
    x: f64,
    y: f64,
}
