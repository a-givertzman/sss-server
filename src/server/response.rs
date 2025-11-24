use crate::server::{Cot, Reply};

///
/// The [Response] contains information about the `Request` and `Rply` data
#[derive(Debug, Clone, PartialEq)]
pub struct Response<QueryId> {
    /// Event id, used internal only to identify incoming request message
    pub event_id: u32,
    /// Name of the [Query], correspond with `query` variant
    pub query_id: QueryId,
    /// Cause of the transmission
    pub cot: Cot,
    /// Response data, optionally may contains error if Cot::..Err
    pub reply: Reply,
}
