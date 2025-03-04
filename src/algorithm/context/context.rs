use crate::{
    algorithm::{
        bearing_filter::bearing_filter_ctx::BearingFilterCtx, dynamic_coefficient::dynamic_coefficient_ctx::DynamicCoefficientCtx, hoisting_tackle::hoisting_tackle_ctx::HoistingTackleCtx, hook_filter::hook_filter_ctx::HookFilterCtx, initial_ctx::initial_ctx::InitialCtx, lifting_speed::lifting_speed_ctx::LiftingSpeedCtx, load_hand_device_mass::load_hand_device_mass_ctx::LoadHandDeviceMassCtx, rope_count::rope_count_ctx::RopeCountCtx, rope_effort::rope_effort_ctx::RopeEffortCtx, select_betta_phi::select_betta_phi_ctx::SelectBetPhiCtx
    },
    kernel::user_setup::{user_bearing_ctx::UserBearingCtx, user_hook_ctx::UserHookCtx},
};
use super::testing_ctx::TestingCtx;
///
/// # Calculation context
/// - Provides read/write access to initial
/// - R/W access to the isoleted data of each step of computations
#[derive(Debug, Clone)]
pub struct Context {
    /// where store [initial data](design\docs\algorithm\part01\initial_data.md)
    pub(super) initial: InitialCtx,
    /// result of calculation [steady-state-lifting-speed](design/docs/algorithm/part02/chapter_01_choose_hook.md)
    pub(super) lifting_speed: LiftingSpeedCtx,
    /// result of calculation [ϕ2(phi) and β2(betta) coefficients](design/docs/algorithm/part02/chapter_01_choose_hook.md)
    pub(super) select_bet_phi: SelectBetPhiCtx,
    /// result of calculation [dynamic coefficient](design/docs/algorithm/part02/chapter_01_choose_hook.md)
    pub(super) dynamic_coefficient: DynamicCoefficientCtx,
    /// result of [filtering hooks](design/docs/algorithm/part02/chapter_01_choose_hook.md)
    pub(super) hook_filter: HookFilterCtx,
    /// user [crane hook](design/docs/algorithm/part02/chapter_01_choose_hook.md)
    pub(super) user_hook: UserHookCtx,
    /// result of [filtering bearings](design/docs/algorithm/part02/chapter_01_choose_hook.md)
    pub(super) bearing_filter: BearingFilterCtx,
    /// user [bearing hook](design/docs/algorithm/part02/chapter_01_choose_hook.md)
    pub(super) user_bearing: UserBearingCtx,
    /// result of calculation [total mass and net weight](design\docs\algorithm\part02\chapter_02_choose_another_load_handing_device.md)
    pub(super) load_device_mass: LoadHandDeviceMassCtx,
    /// result of calculation [rope effort](design\docs\algorithm\part02\chapter_03_choose_hoisting_tackle.md)
    pub(super) rope_effort: RopeEffortCtx,
    /// result of calculation [rope count](design\docs\algorithm\part02\chapter_03_choose_hoisting_tackle.md)
    pub(super) rope_count: RopeCountCtx,
    /// result of calculation [hoisting tackle](design\docs\algorithm\part02\chapter_03_choose_hoisting_tackle.md)
    pub(super) hoisting_tackle: HoistingTackleCtx,
    ///
    /// Uset for testing only
    #[allow(dead_code)]
    pub testing: Option<TestingCtx>,
}
//
//
impl Context {
    ///
    /// New instance [Context]
    /// - 'initial' - [InitialCtx] instance, where store initial data
    pub fn new(initial: InitialCtx) -> Self {
        Self {
            initial,
            lifting_speed: LiftingSpeedCtx::default(),
            select_bet_phi: SelectBetPhiCtx::default(),
            dynamic_coefficient: DynamicCoefficientCtx::default(),
            hook_filter: HookFilterCtx::default(),
            user_hook: UserHookCtx::default(),
            bearing_filter: BearingFilterCtx::default(),
            user_bearing: UserBearingCtx::default(),
            load_device_mass: LoadHandDeviceMassCtx::default(),
            rope_effort: RopeEffortCtx::default(),
            rope_count: RopeCountCtx::default(),
            hoisting_tackle: HoistingTackleCtx::default(),
            testing: None,
        }
    }
}
