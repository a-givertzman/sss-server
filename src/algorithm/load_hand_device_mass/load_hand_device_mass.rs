use futures::future::BoxFuture;
use crate::{
    algorithm::{context::{context_access::{ContextRead, ContextWrite}, ctx_result::CtxResult}, 
    initial_ctx::initial_ctx::InitialCtx}, 
    kernel::{
        dbgid::dbgid::DbgId, eval::Eval, str_err::str_err::StrErr,
        types::eval_result::EvalResult,
        user_setup::user_hook_ctx::UserHookCtx,
    },
};
use super::load_hand_device_mass_ctx::LoadHandDeviceMassCtx;
///
/// Calculation step: [total mass and net weight](design\docs\algorithm\part02\chapter_02_choose_another_load_handing_device.md)
pub struct LoadHandDeviceMass {
    dbg: DbgId,
    /// value of [total mass and net weight](design\docs\algorithm\part02\chapter_01_choose_hook.md)
    value: Option<LoadHandDeviceMassCtx>,
    /// [Context] instance, where store all info about initial data and each algorithm result's
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl LoadHandDeviceMass {
    ///
    /// New instance [LoadHandDeviceMass]
    /// - `ctx` - [Context]
    pub fn new(ctx: impl Eval<(), EvalResult> + Send + 'static) -> Self {
        Self {
            dbg: DbgId("LoadHandDeviceMass".to_string()),
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for LoadHandDeviceMass {
    fn eval(&mut self, _: ()) -> BoxFuture<'_, EvalResult> {
        log::debug!("{}.eval | Start", self.dbg);
        let result = Box::pin(async {
            let result = self.ctx.eval(()).await;
            match result {
                CtxResult::Ok(ctx) => {
                    let initial = ContextRead::<InitialCtx>::read(&ctx);
                    let user_hook = ContextRead::<UserHookCtx>::read(&ctx).result.clone();
                    let result = match &initial.user_alt_lift_device {
                        Some(lift_device) => {
                            LoadHandDeviceMassCtx {
                                total_mass: user_hook.weight + lift_device.weight,
                                net_weight: initial.load_capacity - lift_device.weight,
                            }
                        },
                        None => {
                            LoadHandDeviceMassCtx {
                                total_mass: user_hook.weight,
                                net_weight: initial.load_capacity,
                            }
                        },
                    };
                    self.value = Some(result.clone());
                    ctx.write(result)
                },
                CtxResult::Err(err) => CtxResult::Err(StrErr(format!(
                    "{}.eval | Read context error: {:?}",
                    self.dbg, err
                ))),
                CtxResult::None => CtxResult::None,
            }
        });
        log::debug!("{}.eval | Exit", self.dbg);
        result
    }
}
//
//
impl std::fmt::Debug for LoadHandDeviceMass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadHandDeviceMass")
            .field("dbgid", &self.dbg)
            .field("value", &self.value)
            // .field("ctx", &self.ctx)
            .finish()
    }
}