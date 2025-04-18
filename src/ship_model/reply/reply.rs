//!
//! All possible replies from the `ShipModel` listed here
use bincode::{Decode, Encode};
use sal_core::error::Error;
use crate::algorithm::{entities::Bounds, eval::BalanceCtx};
use super::BoundArea;

///
/// Replies from the `ShipModel`
#[derive(Debug, Decode, Encode)]
pub enum Reply {
    Bounds(Bounds),
    BoundAreas(Result<BoundArea, Error>),
    ComputeBalance(BalanceCtx),
}
