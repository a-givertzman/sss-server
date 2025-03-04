use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
///
/// Represents a [hoisting rope core type](docs\catalogsPurchasedEquipment.xlsx)
pub enum RopeType {
    Metal,
    Synthetic
}