use crate::kernel::str_err::str_err::StrErr;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
///
/// Represents [lifting mechanism driver types](design\docs\algorithm\part01\initial_data.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriverType {
    Hd1,
    Hd2,
    Hd3,
    Hd4,
    Hd5,
}
//
//
impl FromStr for DriverType {
    type Err = StrErr;
    ///
    /// Method translates from string into enuming structure DriverType
    /// - 's' - value to translate
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hd1" => Ok(Self::Hd1),
            "hd2" => Ok(Self::Hd2),
            "hd3" => Ok(Self::Hd3),
            "hd4" => Ok(Self::Hd4),
            "hd5" => Ok(Self::Hd5),
            _ => Err(format!("DriverType.from_str | Invalid DriverType: {}", s).into()),
        }
    }
}
