use serde::Deserialize;
use super::ConnectionConf;

///
/// The `Server` configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConf {
    pub address: String,
    pub connection: ConnectionConf,
}
