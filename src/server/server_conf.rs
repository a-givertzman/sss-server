use serde::{Deserialize, Serialize};
use super::{ConnectionConf, DevStreamConf};

///
/// The `Server` configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConf {
    pub address: String,
    pub connection: ConnectionConf,
    #[serde(rename = "dev-stream")]
    pub dev_stream: DevStreamConf,
}
