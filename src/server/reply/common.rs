use serde::{Deserialize, Serialize};

///
/// Reply to `Calculus` query
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalculusReply {
    pub status: CalculusStatus,
}
///
/// Statuses of the `Calculus`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CalculusStatus {
    /// Calculations done or not started
    Done,
    /// Calculations in progress
    Ongoing,
    /// Calculations were canceled by the Client or internal reasons
    Canceled,
}