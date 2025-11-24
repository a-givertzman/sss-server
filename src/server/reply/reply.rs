use serde::{Deserialize, Serialize};
use crate::server::{Bytes, Device, DeviceDoc, DeviceInfo};

///
/// Wrapper for all variants of API [Reply]'s
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Reply {
    /// Use if nothing to be sent in the data fiekld of the `Message`
    Empty,
    /// Use if error info to be sent
    Error(String),
    /// Use if no information to be sent
    Bytes(Bytes),
    DeviceStream(Device),
    DeviceInfo(DeviceInfo),
    DeviceDoc(DeviceDoc),
    ///
    /// Used for testing only
    TestString(String),
}
//
//
impl Reply {
    ///
    /// Returns [Reply::Error] from any impl Into<String>
    pub fn error(err: impl Into<String>) -> Self {
        Self::Error(err.into())
    }
}
