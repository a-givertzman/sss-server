use crate::algorithm::entities::hook::Hook;
///
/// Calculation context store: [user hook](design\docs\algorithm\part02\chapter_01_choose_hook.md)
#[derive(Debug, Clone, Default)]
pub struct UserHookCtx {
    /// value of [user hook](design\docs\algorithm\part02\chapter_01_choose_hook.md)
    pub result: Hook,
}
