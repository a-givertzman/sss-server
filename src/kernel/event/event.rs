use serde::{Deserialize, Serialize};
use crate::infrostructure::query::query::Query;
use super::{diag::Diag, info::Info};

///
/// Complete structured list of Application Events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    Query(Query),
    /// Information event
    Inf(Info),
    /// Diagnostoc event
    Diag(Diag),
}
