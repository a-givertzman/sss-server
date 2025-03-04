use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
///
/// Represents a [bearing](docs\catalogsPurchasedEquipment.xlsx) with its main characteristics.
pub struct Bearing {
    /// Name of the bearing model
    pub name: String,
    /// Outer diameter of the bearing
    pub outer_diameter: f64,
    /// Inner diameter of the bearing
    pub inner_diameter: f64,
    /// Static load capacity of the bearing
    pub static_load_capacity: f64,
    /// Height of the bearing
    pub height: f64,
}