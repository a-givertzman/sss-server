//! Обледенение судна
use sal_core::error::Error;
use serde::{Deserialize, Serialize};

/// Тип обледенения судна
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum IcingCoeffType {
    #[serde(alias = "full")]
    Full,
    #[serde(alias = "half")]
    Half,
    #[serde(alias = "none")]
    None,
}
//
impl IcingCoeffType {
    pub fn from_str(src: &str) -> Result<Self, Error> {
        Ok(match src.trim().to_lowercase().as_str() {
            "full" => IcingCoeffType::Full,
            "half" => IcingCoeffType::Half,
            "none" => IcingCoeffType::None,
            src => return Err(Error::from(format!("IcingStabType.from_str | error: no type {src}"))),
        })
    }
}
