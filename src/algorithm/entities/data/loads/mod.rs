pub mod constant;
pub mod liquid;
pub mod gaseous;
pub mod bulk;
pub mod unit;

pub use constant::*;
pub use liquid::*;
pub use gaseous::*;
pub use bulk::*;
pub use unit::*;

use serde::{Deserialize, Serialize};
/// Тип назначения груза
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum AssignmentType {
    #[serde(alias = "lightship")]
    Lightship,
    #[serde(alias = "ballast")]
    Ballast,
    #[serde(alias = "stores")]
    Stores,
    #[serde(alias = "cargo_load")]
    CargoLoad,
    #[serde(alias = "unspecified")]
    Unspecified,
}
//
impl std::fmt::Display for AssignmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AssignmentType::Lightship => "Lightship",
                AssignmentType::Ballast => "Ballast",
                AssignmentType::Stores => "Stores",
                AssignmentType::CargoLoad => "CargoLoad",
                AssignmentType::Unspecified => "Unspecified",
            },
        )
    }
}
/// Тип груза судна
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CargoType {
    #[serde(alias = "bulk")]
    Bulk,
    #[serde(alias = "unit")]
    Unit,
    #[serde(alias = "gaseous")]
    Gaseous,
    #[serde(alias = "liquid")]
    Liquid,
}
//
impl std::fmt::Display for CargoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CargoType::Bulk => "Bulk",
                CargoType::Unit => "Unit",
                CargoType::Gaseous => "Gaseous",                
                CargoType::Liquid => "Liquid",
            },
        )
    }
}
