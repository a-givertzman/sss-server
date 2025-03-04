use futures::future::BoxFuture;
use crate::{algorithm::{context::{context_access::{ContextRead, ContextWrite}, ctx_result::CtxResult}, dynamic_coefficient::dynamic_coefficient_ctx::DynamicCoefficientCtx, entities::bearing::Bearing, initial_ctx::initial_ctx::InitialCtx}, kernel::{dbgid::dbgid::DbgId, eval::Eval, str_err::str_err::StrErr, types::eval_result::EvalResult, user_setup::user_hook_ctx::UserHookCtx}};
use super::bearing_filter_ctx::BearingFilterCtx;
///
/// Calculation step: [filtering bearings](design\docs\algorithm\part02\chapter_01_choose_hook.md)
pub struct BearingFilter {
    dbgid: DbgId,
    /// vector of [filtered bearings](design\docs\algorithm\part02\chapter_01_choose_hook.md)
    value: Option<BearingFilterCtx>,
    /// [Context] instance, where store all info about initial data and each algorithm result's
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl  BearingFilter {
    ///
    /// [Acceleration of gravity](design\docs\algorithm\part02\chapter_01_choose_hook.md)
    const G: f64 = 9.81;
    ///
    /// New instance [BearingFilter]
    /// - `ctx` - [Context]
    pub fn new(ctx: impl Eval<(), EvalResult> + Send + 'static) -> Self {
        Self {
            dbgid: DbgId("HookFilter".to_string()),
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl  Eval<(), EvalResult> for BearingFilter {
    fn eval(&mut self, _: ()) -> BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            let result = self.ctx.eval(()).await;
            match result {
                CtxResult::Ok(ctx) => {
                    let initial = ContextRead::<InitialCtx>::read(&ctx);
                    let user_loading_capacity = initial.load_capacity.clone(); 
                    let dynamic_coefficient = ContextRead::<DynamicCoefficientCtx>::read(&ctx).result;
                    let user_hook = ContextRead::<UserHookCtx>::read(&ctx).result.clone();
                    let result: Vec<Bearing> = initial
                    .bearings
                    .iter()
                    .cloned()
                    .filter(|bearing| {
                        (bearing.static_load_capacity >= dynamic_coefficient * user_loading_capacity * Self::G) &&
                        (bearing.outer_diameter >= user_hook.shank_diameter)
                    })
                    .collect();
                    if result.is_empty() {
                        CtxResult::Err(StrErr(format!(
                            "{}.eval | No available variants of hook for specified requirements",
                            self.dbgid,
                        )))
                    } else {
                        let result = BearingFilterCtx { result };
                        self.value = Some(result.clone());
                        ctx.write(result)
                    }
                },
                CtxResult::Err(err) => CtxResult::Err(StrErr(format!(
                    "{}.eval | Read context error: {:?}",
                    self.dbgid, err
                ))),
                CtxResult::None => CtxResult::None,
            }
        })
    }
}
//
//
impl std::fmt::Debug for BearingFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookFilter")
            .field("dbgid", &self.dbgid)
            .field("value", &self.value)
            // .field("ctx", &self.ctx)
            .finish()
    }
}
