//!
//! All possible replies from the `ShipModel` listed here
use bincode::{Decode, Encode};
use crate::algorithm::{entities::Bounds, eval::BalanceCtx};
use super::BoundAreaReply;

///
/// Replies from the `ShipModel`
#[derive(Debug, Decode, Encode)]
pub enum Reply {
    Bounds(Bounds),
    BoundAreas(BoundAreaReply),
    ComputeBalance(BalanceCtx),
}
