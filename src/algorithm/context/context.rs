use crate::
    algorithm::{eval::*, initial::initial_ctx::InitialCtx
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
    /// TODO: DOC
    pub(super) areas_strength: Option<AreasStrengthCtx>,
    /// TODO: DOC
    pub(super) icing_stab: Option<IcingStabCtx>,
    /// TODO: DOC
    pub(super) loads: Option<LoadsCtx>,
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
            areas_strength: None,
            icing_stab: None,
            loads: None,
            testing: None,
        }
    }
}
