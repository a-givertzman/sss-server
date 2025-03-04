use crate::kernel::str_err::str_err::StrErr;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
///
/// Represents [loading combination types](design\docs\algorithm\part02\initial_data.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadingCombination {
    A1,
    B1,
    C1,
}
//
//
impl FromStr for LoadingCombination {
    type Err = StrErr;
    ///
    /// Method translates from string into enuming structure LoadingCombination
    /// - 's' - value to translate
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "a1" => Ok(Self::A1),
            "b1" => Ok(Self::B1),
            "c1" => Ok(Self::C1),
            _ => Err(format!(
                "LoadingCombination.from_str | Invalid LoadingCombination: {}",
                s
            )
            .into()),
        }
    }
}
