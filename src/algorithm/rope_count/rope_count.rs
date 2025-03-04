use futures::future::BoxFuture;
use crate::{algorithm::{context::{context_access::{ContextRead, ContextWrite}, ctx_result::CtxResult}, initial_ctx::initial_ctx::InitialCtx, load_hand_device_mass::load_hand_device_mass_ctx::LoadHandDeviceMassCtx, rope_effort::rope_effort_ctx::RopeEffortCtx}, kernel::{dbgid::dbgid::DbgId, eval::Eval, str_err::str_err::StrErr, types::eval_result::EvalResult}};
use super::rope_count_ctx::RopeCountCtx;
///
/// Calculation step: [rope count](design\docs\algorithm\part02\chapter_03_choose_hoisting_tackle.md)
pub struct RopeCount {
    dbgid: DbgId,
    /// value of [rope count](design\docs\algorithm\part02\chapter_03_choose_hoisting_tackle.md)
    value: Option<RopeCountCtx>,
    /// [Context] instance, where store all info about initial data and each algorithm result's
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl RopeCount {
    ///
    /// New instance [RopeCount]
    /// - `ctx` - [Context]
    pub fn new(ctx: impl Eval<(), EvalResult> + Send + 'static) -> Self {
        Self {
            dbgid: DbgId("RopeCount".to_string()),
            value: None,
            ctx: Box::new(ctx),
        }
    }
    ///
    /// [Rounded up value](design\docs\algorithm\part02\chapter_03_choose_hoisting_tackle.md) to recommended rope count
    fn round_up(n: f64) -> f64 {
        let recommended = [2.0, 4.0, 8.0, 12.0, 16.0];
        for &r in &recommended {
            if n <= r {
                return r;
            }
        }
        *recommended.last().unwrap()
    }
}
//
//
impl Eval<(), EvalResult> for RopeCount {
    fn eval(&mut self, _: ()) -> BoxFuture<'_, EvalResult> {
        Box::pin(async {
            let result = self.ctx.eval(()).await;
            match result {
                CtxResult::Ok(ctx) => {
                    let initial = ContextRead::<InitialCtx>::read(&ctx);
                    let hook_weight = ContextRead::<LoadHandDeviceMassCtx>::read(&ctx).total_mass.clone();
                    let rope_effort = ContextRead::<RopeEffortCtx>::read(&ctx).result.clone();
                    let result = Self::round_up((initial.load_capacity+hook_weight)/rope_effort);
                    let result = RopeCountCtx {
                        result,
                    };
                    self.value = Some(result.clone());
                    ctx.write(result)
                }
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
impl std::fmt::Debug for RopeCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RopeCount")
            .field("dbgid", &self.dbgid)
            .field("value", &self.value)
            .finish()
    }
}