use serde::{Deserialize, Serialize};
use super::{rope_durability_class::RopeDurabilityClass, rope_type::RopeType};
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
///
/// Represents a [hoisting rope](docs\catalogsPurchasedEquipment.xlsx) with its key characteristics.
pub struct HoistingRope {
    /// Full name of the rope
    pub name: String,
    /// Rope diameter
    pub rope_diameter: f64,
    /// Type of the rope
    pub r#type: RopeType,
    /// Class of rope durability
    pub rope_durability: RopeDurabilityClass,
    /// Rope breaking force
    pub rope_force: f64,
    /// Rope cross-sectional area
    pub s: f64,
    /// Specific gravity of the rope
    pub m: f64,
}
