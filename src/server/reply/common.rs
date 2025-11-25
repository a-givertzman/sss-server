use serde::Serialize;

///
/// Reply to `Calculus` query
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CalculusReply {
    pub status: CalculusStatus,
}
///
/// Statuses of the `Calculus`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum CalculusStatus {
    /// Calculations done or not started
    Done,
    /// Calculations in progress
    Ongoing,
}