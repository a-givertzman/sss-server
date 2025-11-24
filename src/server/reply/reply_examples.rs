use serde::{Deserialize, Serialize};

///
/// ## Structure to be serialized and sent by the `Server`
/// TODO: To be deleted, implemented just as example for the `Server`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {}
///
/// ## Structure to be serialized and sent by the `Server`
/// TODO: To be deleted, implemented just as example for the `Server`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceDoc {}
///
/// ## Structure to be serialized and sent by the `Server`
/// TODO: To be deleted, implemented just as example for the `Server`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Device {}
///
/// ## Structure to be serialized and sent by the `Server`
/// TODO: To be deleted, implemented just as example for the `Server`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DevStreamConf {
    /// TCP socket read/write timeout in ms, default 100 ms
    // #[serde(deserialize_with = "DevStreamConf::timeout")]
    pub devices: Vec<(String, DevConf)>,
}
///
/// ## Structure to be serialized and sent by the `Server`
/// TODO: To be deleted, implemented just as example for the `Server`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DevConf {}

