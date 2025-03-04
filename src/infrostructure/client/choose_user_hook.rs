use serde::{Serialize, Deserialize};
use crate::algorithm::entities::hook::Hook;
///
/// User request | Asks user for choose [Hook] from filtered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChooseUserHookQuery {
    /// vector of hooks filtered by user characteristics
    pub variants: Vec<Hook>,
    #[serde(skip_serializing_if = "testing")]
    pub testing: bool,
}
//
//
impl ChooseUserHookQuery {
    ///
    /// New instance [ChooseUserHookQuery]
    pub fn new(variants: Vec<Hook>) -> Self {
        Self {
            variants,
            testing: false,
        }
    }
    ///
    /// New instance [ChooseUserHookQuery] for testing
    pub fn test(variants: Vec<Hook>) -> Self {
        Self {
            variants,
            testing: true,
        }
    }
}
///
/// Used for `skip_serializing_if`
fn testing(v: &bool) -> bool {
    !v
}
///
/// Reply to [ChooseUserHookQuery]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChooseUserHookReply {
    pub choosen: Hook
}
//
//
impl ChooseUserHookReply {
    ///
    /// New instance [ChooseUserHookReply]
    pub fn new(choosen: Hook) -> Self {
        Self {
            choosen,
        }
    }
}