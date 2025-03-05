use sal_sync::services::entity::error::str_err::StrErr;

use crate::algorithm::entities::{area::HAreaStrength, strength};

///
/// Queries to the `ShipModel`
#[derive(Debug)]
pub enum Query {
    AreasStrength,
}
