use crate::
    algorithm::{
        initial::initial_ctx::InitialCtx,
    }
;
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
    // pub(super) lifting_speed: LiftingSpeedCtx,
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
            // lifting_speed: LiftingSpeedCtx::default(),
            testing: None,
        }
    }
}
