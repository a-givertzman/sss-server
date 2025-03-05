use serde::{Deserialize, Serialize};

use super::restart_eval::RestartEvalQuery;
///
/// List of all possible requests in Client-Server interface
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Query {
    ///
    /// Client request | Restart of calculation
    RestartEval(RestartEvalQuery),
}
