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
/// Тип сыпучего груза судна
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum BulkCargoType {
    #[serde(alias = "timber")]
    Timber,
    #[serde(alias = "grain")]
    Grain,
    #[serde(alias = "undefined")]
    Undefined,
}
//
impl std::fmt::Display for BulkCargoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BulkCargoType::Timber => "Timber",
                BulkCargoType::Grain => "Grain",
                BulkCargoType::Undefined => "Undefined",                
            },
        )
    }
}
/// Тип жидкого груза судна
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum LiquidCargoType {
    #[serde(alias = "crude_oil")]
    CrudeOil,
    #[serde(alias = "fuel_oil")]
    FuelOil,
    #[serde(alias = "lubricating_oil")]
    LubricatingOil,
    #[serde(alias = "fresh_water")]
    FreshWater,
    #[serde(alias = "sullage")]    
    Sullage,
    #[serde(alias = "water_ballast")]
    WaterBallast,
    #[serde(alias = "undefined")]
    Undefined,
}
//
impl std::fmt::Display for LiquidCargoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LiquidCargoType::CrudeOil => "CrudeOil",
                LiquidCargoType::FuelOil => "FuelOil",
                LiquidCargoType::LubricatingOil => "LubricatingOil",
                LiquidCargoType::FreshWater => "FreshWater",
                LiquidCargoType::Sullage => "Sullage",
                LiquidCargoType::WaterBallast => "WaterBallast",
                LiquidCargoType::Undefined => "Undefined",                
            },
        )
    }
}
