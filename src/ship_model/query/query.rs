//!
//! Queries to the `ShipModel`
//!
use bincode::{Decode, Encode};
use super::balance_query::BalanceQuery;
///
/// All possible variants of queries to the `ShipModel`
#[derive(Debug, Decode, Encode)]
pub enum Query {
    Bounds,
    BoundAreas,
    ComputeBalance(BalanceQuery),
   // StabilityAreas,
   // ComputePantocaren,
}
