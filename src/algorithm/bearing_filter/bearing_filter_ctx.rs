use crate::algorithm::entities::bearing::Bearing;
///
/// Calculation context store: [filtered bearings](design\docs\algorithm\part02\chapter_01_choose_hook.md)
#[derive(Debug, Clone, Default)]
pub struct BearingFilterCtx {
    /// vector of [filtered bearings](design\docs\algorithm\part02\chapter_01_choose_hook.md)
    pub result: Vec<Bearing>,
}
