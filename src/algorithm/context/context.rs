use crate::
    algorithm::{
        areas_strength::areas_strength_ctx::AreasStrengthCtx, icing_stab_eval::icing_stab_ctx::IcingStabCtx, initial::initial_ctx::InitialCtx
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
            testing: None,
        }
    }
}
