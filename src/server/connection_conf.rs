use std::str::FromStr;
use sal_sync::services::conf::{ConfDuration, ConfDurationUnit};
use serde::{Deserialize, Deserializer};

///
/// The `Server`'s `connection` configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionConf {
    /// TCP socket read/write timeout in ms, default 100 ms
    #[serde(deserialize_with = "ConnectionConf::timeout")]
    pub timeout: ConfDuration,
}
//
//
impl ConnectionConf {
    ///
    /// Used to deserialize `timeout`
    fn timeout<'de, D>(deserializer: D) -> Result<ConfDuration, D::Error> where D: Deserializer<'de> {
        match Deserialize::deserialize(deserializer) {
            Ok(val) => {
                match ConfDuration::from_str(val) {
                    Ok(val) => Ok(val),
                    Err(err) => {
                        log::warn!("ConnectionConf.timeout | Field `timeout` - wrong format, used default 100 ms, error: {:?}", err);
                        Ok(ConfDuration::new(100, ConfDurationUnit::Millis))
                    }
                }
            },
            Err(_) => {
                log::info!("ConnectionConf.timeout | Field `timeout` is missing of wrong format, used default 100 ms");
                Ok(ConfDuration::new(100, ConfDurationUnit::Millis))
            }
        }
    }
}
