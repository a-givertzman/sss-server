///
/// Calculation context store: [total mass and net weight](design\docs\algorithm\part02\chapter_02_choose_another_load_handing_device.md)
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LoadHandDeviceMassCtx {
    /// value of [total_mass](design\docs\algorithm\part02\chapter_01_choose_hook.md)
    pub total_mass: f64,
    /// value of [net weight](design\docs\algorithm\part02\chapter_01_choose_hook.md)
    pub net_weight: f64,
}
