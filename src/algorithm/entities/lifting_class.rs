use crate::kernel::str_err::str_err::StrErr;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
///
/// Represents [lifting class types](design\docs\algorithm\part02\chapter_01_choose_hook.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiftClass {
    Hc1,
    Hc2,
    Hc3,
    Hc4,
}
//
//
impl FromStr for LiftClass {
    type Err = StrErr;
    ///
    /// Method translates from string into enuming structure LiftClass
    /// - 's' - value to translate
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hc1" => Ok(Self::Hc1),
            "hc2" => Ok(Self::Hc2),
            "hc3" => Ok(Self::Hc3),
            "hc4" => Ok(Self::Hc4),
            _ => Err(format!("LiftClass.from_str | Invalid LiftClass: {}", s).into()),
        }
    }
}
