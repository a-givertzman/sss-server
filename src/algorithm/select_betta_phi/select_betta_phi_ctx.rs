use crate::
    algorithm::entities::bet_phi::BetPhi
;
///
/// Calculation context store: [β2 and ϕ2 coefficients](design\docs\algorithm\part02\chapter_01_choose_hook.md)
#[derive(Debug, Clone, Default)]
pub struct SelectBetPhiCtx {
    /// value of [β2 and ϕ2 coefficients](design\docs\algorithm\part02\chapter_01_choose_hook.md)
    pub result: BetPhi,
}
//
//
