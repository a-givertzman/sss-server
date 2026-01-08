use bincode::{Decode, Encode};

mod constant;
mod liquid;
mod gaseous;
mod bulk;
mod unit;
mod container;

pub use constant::*;
pub use liquid::*;
pub use gaseous::*;
pub use bulk::*;
pub use unit::*;
pub use container::*;

use serde::{Deserialize, Serialize};
/// Тип назначения груза
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Decode, Encode)]
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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Decode, Encode)]
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
/// Тип штучного груза судна
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum UnitCargoType {
    #[serde(alias = "timber")]
    Timber,
    #[serde(alias = "container")]
    Container,
    #[serde(alias = "grain_bulkhead")]
    GrainBulkhead,
    #[serde(alias = "undefined")]
    Undefined,
}
//
impl std::fmt::Display for UnitCargoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UnitCargoType::Timber => "Timber",
                UnitCargoType::Container => "Container",
                UnitCargoType::GrainBulkhead => "GrainBulkhead",
                UnitCargoType::Undefined => "Undefined",                
            },
        )
    }
}
/// Тип груза для отсека
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CompartmentPurpose {
    #[serde(alias = "subdivision_compartment")] // Отсек деления на отсеки. Является независимым отсеком, в
                                // рамках которого ограничивается поступление воды при его повреждении
    SubdivisionCompartment,
    #[serde(alias = "hold")] // Отсек является грузовым трюмом, предназначенным для перевозки сухих грузов
    Hold,
    #[serde(alias = "cargo_tank")] // Отсек является грузовым танком, предназначенным для перевозки жидких грузов
    CargoTank,
    #[serde(alias = "cargo_gaseous_tank")] // Отсек является грузовым танком, предназначенным для перевозки газообразных грузов
    CargoGaseousTank,
    #[serde(alias = "ballast_tank")] // Отсек предназначен для перевозки водяного балласта
    BallastTank,
    #[serde(alias = "lubricating_oil_tank")]
    LubricatingOilTank,
    #[serde(alias = "fresh_water_tank")] // Отсек предназначен для перевозки пресной воды
    FreshWaterTank,
    #[serde(alias = "urea_tank")] // Отсек предназначен для перевозки мочевины
    UreaTank,
    #[serde(alias = "sundry_tank")] // Отсек предназначен для перевозки грязных жидкостей
    SundryTank,
    #[serde(alias = "fuel_tank")] // Отсек предназначен для перевозки топлива
    FuelTank,
    #[serde(alias = "deck_well")] // Помещение является палубным колодцем
    DeckWell,
    #[serde(alias = "undefined")] // Назначение отсека не определено
    Undefined,
}
//
impl std::fmt::Display for CompartmentPurpose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CompartmentPurpose::SubdivisionCompartment => "subdivision_compartment",
                CompartmentPurpose::Hold => "hold",
                CompartmentPurpose::CargoTank => "cargo_tank",
                CompartmentPurpose::CargoGaseousTank => "cargo_gaseous_tank",
                CompartmentPurpose::BallastTank => "ballast_tank",
                CompartmentPurpose::LubricatingOilTank => "lubricating_oil_tank",
                CompartmentPurpose::FreshWaterTank => "fresh_water_tank",
                CompartmentPurpose::UreaTank => "urea_tank",
                CompartmentPurpose::SundryTank => "sundry_tank",
                CompartmentPurpose::FuelTank => "fuel_tank",
                CompartmentPurpose::DeckWell => "deck_well",
                CompartmentPurpose::Undefined => "undefined",
            },
        )
    }
}
