///
/// Represents [β2 and ϕ2 coefficients](design\docs\algorithm\part02\chapter_01_choose_hook.md) for calculation [dynamic coefficient](design\docs\algorithm\part02\chapter_01_choose_hook.md)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BetPhi {
    /// β2 coefficient
    pub bet: f64,
    /// -ϕ2 coefficient
    pub phi: f64,
}
//
//
impl BetPhi {
    ///
    /// Struct constuctor
    /// - 'bet' - β2 coefficient
    /// - 'phi' - ϕ2 coefficient
    pub fn new(bet: f64, phi: f64) -> Self {
        Self { bet, phi }
    }
}
