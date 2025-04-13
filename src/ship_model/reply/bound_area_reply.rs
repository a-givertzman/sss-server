use bincode::{Decode, Encode};

///
/// Reply to `ShipModel` `BoundAreaRequest`
#[derive(Debug, Decode, Encode)]
pub struct BoundAreaReply {
    pub v: Vec<f64>,
    pub h: Vec<f64>,
}