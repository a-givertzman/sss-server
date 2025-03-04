use serde::{Serialize, Deserialize};
///
/// User request | Asks user for change [HoistingTackle](design\docs\algorithm\part02\chapter_03_choose_hoisting_tackle.md)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeHoistingTackleQuery {
    /// vector of hoisting tackle variants
    pub variants: Vec<u8>,
}
//
//
impl ChangeHoistingTackleQuery {
    ///
    /// New instance [ChangeHoistingTackleQuery]
    pub fn new() -> Self {
        Self {
            variants: vec![1,2],
        }
    }
}
///
/// Reply to [ChangeHoistingTackleQuery]
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeHoistingTackleReply {
    pub answer: u8
}
//
//
impl ChangeHoistingTackleReply {
    ///
    /// New instance [ChangeHoistingTackleReply]
    pub fn new(choosen: u8) -> Self {
        Self {
            answer: choosen,
        }
    }
}
