use sal_sync::services::conf::ConfDuration;
use serde::Deserialize;

///
/// The `Server`'s `connection` configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionConf {
    /// TCP socket read/write timeout in ms, default 100 ms
    pub timeout: ConfDuration,
}
