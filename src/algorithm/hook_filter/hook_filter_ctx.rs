use crate::algorithm::entities::hook::Hook;
///
/// Calculation context store: [filtered hooks](design\docs\algorithm\part02\chapter_01_choose_hook.md)
#[derive(Debug, Clone, Default)]
pub struct HookFilterCtx {
    /// vector of [filtered hooks](design\docs\algorithm\part02\chapter_01_choose_hook.md)
    pub result: Vec<Hook>,
}
