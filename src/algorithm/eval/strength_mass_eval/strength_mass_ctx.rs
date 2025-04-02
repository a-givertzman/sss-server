use crate::algorithm::entities::Position;

///
#[derive(Debug, Clone)]
pub struct StrengthMassCtx {
    pub hull: f64, 
    pub hull_shift: Position,
    pub unit: f64,
    pub unit_shift: Position,
    pub icing: f64,
    pub icing_shift: Position,
    pub wetting: f64,
    pub wetting_shift: Position,
    pub liquid: Vec<(usize, f64, )>,
    pub gazeous: Vec<(usize, f64, )>,
    pub bulk: Vec<(usize, f64, )>,
}
