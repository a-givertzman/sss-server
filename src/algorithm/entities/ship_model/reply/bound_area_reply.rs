use bincode::{Decode, Encode};

///
/// Reply to `ShipModel` `BoundAreaRequest`
#[derive(Debug, Clone, Decode, Encode)]
pub struct BoundArea {
    pub v: Vec<f64>,
    pub h: Vec<f64>,
}
