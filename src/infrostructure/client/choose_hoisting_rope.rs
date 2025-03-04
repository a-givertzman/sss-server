use serde::{Deserialize, Serialize};
use crate::algorithm::entities::hoisting_rope::hoisting_rope::HoistingRope;
///
/// User request | Asks user for choose [HoistingRope] from filtered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChooseHoistingRopeQuery {
    /// vector of hoisting ropes filtered by user characteristics
    pub variants: Vec<HoistingRope>,
}
//
//
impl ChooseHoistingRopeQuery {
    ///
    /// New instance [ChooseHoistingRopeQuery]
    pub fn new(variants: Vec<HoistingRope>) -> Self {
        Self {
            variants,
        }
    }
}
///
/// Reply to [ChooseHoistingRopeQuery]
#[derive(Debug, Serialize, Deserialize)]
pub struct ChooseHoistingRopeReply {
    pub answer: HoistingRope,
}
//
//
impl ChooseHoistingRopeReply {
    ///
    /// New instance [ChooseHoistingRopeReply]
    pub fn new(choosen: HoistingRope) -> Self {
        Self { 
            answer: choosen
        }
    }
}
