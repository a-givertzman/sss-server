use crate::algorithm::entities::bearing::Bearing;
///
/// Calculation context store: [user bearing](design\docs\algorithm\part02\chapter_01_choose_hook.md)
#[derive(Debug, Clone, Default)]
pub struct UserBearingCtx {
    /// value of [user bearing](design\docs\algorithm\part02\chapter_01_choose_hook.md)
    pub result: Bearing,
}
