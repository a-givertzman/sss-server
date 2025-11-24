use std::time::Duration;
use serde::{Deserialize, Deserializer, Serialize};

///
/// The `Server`'s `connection` configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConf {
    /// TCP socket read/write timeout in ms, default 100 ms
    #[serde(deserialize_with = "ConnectionConf::timeout")]
    pub timeout: Duration,
}
//
//
impl ConnectionConf {
    const DEFAULT_TIMEOUT: Duration = Duration::from_millis(100);
    ///
    /// Used to deserialize `timeout`
    fn timeout<'de, D>(deserializer: D) -> Result<Duration, D::Error> where D: Deserializer<'de> {
        match Deserialize::deserialize(deserializer) {
            Ok(val) => {
                let val: Option<u64> = val;
                match val {
                    Some(val) => Ok(Duration::from_millis(val)),
                    None => {
                        log::info!("ConnectionConf.timeout | Field `timeout` is missing, used default {:?}", Self::DEFAULT_TIMEOUT);
                        Ok(Self::DEFAULT_TIMEOUT)
                    }
                }
            },
            Err(err) => {
                log::info!("ConnectionConf.timeout | Field `timeout` deserialize error {:?}, used default {:?}", err, Self::DEFAULT_TIMEOUT);
                Ok(Self::DEFAULT_TIMEOUT)
            }
        }
    }
}
