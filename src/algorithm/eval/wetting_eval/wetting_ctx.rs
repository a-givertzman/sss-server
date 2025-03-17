//! Учет намокания груза
use crate::algorithm::entities::Position;
///
#[derive(Debug, Clone)]
pub struct WettingCtx {
    pub mass: f64,
    pub mass_shift: Position,
}
