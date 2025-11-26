use sal_core::error::Error;

use crate::server::{Cot, ErrorReply, Reply};

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
//
//
impl<K> Response<K> {
    ///
    /// Returns [Response] transformed into error
    pub fn into_err(self, e: impl Into<ErrorReply>) -> Self {
        Self {
            event_id: self.event_id,
            query_id: self.query_id,
            cot: match self.cot {
                Cot::ActCon => Cot::ActErr,
                Cot::ActErr => Cot::ActErr,
                Cot::ReqCon => Cot::ReqErr,
                Cot::ReqErr => Cot::ReqErr,
                _ => {
                    log::warn!("{}", Error::new("Response", "into_err").err(format!("Can't find error 'Cot' variant for {:?}", self.cot)));
                    self.cot
                },
            },
            reply: Reply::error(e),
        }
    }
}