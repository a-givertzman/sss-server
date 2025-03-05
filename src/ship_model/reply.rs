use sal_sync::services::entity::error::str_err::StrErr;

use crate::algorithm::entities::{area::HAreaStrength, strength};

///
/// Replies from the `ShipModel`
#[derive(Debug)]
pub enum Reply {
    AreasStrength(Result<Vec<(strength::VerticalArea, HAreaStrength)>, StrErr>)
}
