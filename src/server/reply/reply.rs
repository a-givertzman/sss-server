use serde::{Deserialize, Serialize};
use crate::server::{Bytes, CalculusReply, Device, DeviceDoc, DeviceInfo, ErrorReply};

///
/// Wrapper for all variants of API [Reply]'s
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Reply {
    /// Use if nothing to be sent in the data fiekld of the `Message`
    Empty,
    /// Use if error info to be sent
    Error(ErrorReply),
    /// String to be sent
    String(String),
    /// Use if no information to be sent
    Bytes(Bytes),
    ///
    ///  Calculations
    Calculus(CalculusReply),
    // ....
    ///
    /// Examples
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
    pub fn error(err: impl Into<ErrorReply>) -> Self {
        Self::Error(err.into())
    }
}
