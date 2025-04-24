use serde::{Deserialize, Serialize};
///
/// Thread pool configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThreadPoolConf {
    /// MAximum allowed number of treads
    pub size: usize,
}