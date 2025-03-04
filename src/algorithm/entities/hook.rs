use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
///
/// Represents a [crane hook](docs\catalogsPurchasedEquipment.xlsx) with specifications and load capacities.
pub struct Hook {
    /// GOST number of hook
    pub gost: String,
    /// hook type
    pub r#type: String,
    /// loading capacity for [M1-M3 types of mechanism work](design\docs\algorithm\part01\initial_data.md)
    pub load_capacity_m13: f64,
    /// loading capacity for [M4-M6 types of mechanism work](design\docs\algorithm\part01\initial_data.md)
    pub load_capacity_m46: f64,
    /// loading capacity for [M7-M8 types of mechanism work](design\docs\algorithm\part01\initial_data.md)
    pub load_capacity_m78: f64,
    /// shank diameter
    pub shank_diameter: f64,
    /// weight of hook
    pub weight: f64,
}
