use futures::future::BoxFuture;
use super::hook_filter_ctx::HookFilterCtx;
use crate::{
    algorithm::{
        context::{context_access::{ContextRead, ContextWrite}, ctx_result::CtxResult},
        entities::{hook::Hook, mechanism_work_type::MechanismWorkType}, initial_ctx::initial_ctx::InitialCtx,
    },
    kernel::{dbgid::dbgid::DbgId, eval::Eval, str_err::str_err::StrErr, types::eval_result::EvalResult},
};
///
/// Calculation step: [filtering hooks](design\docs\algorithm\part02\chapter_01_choose_hook.md)
pub struct HookFilter {
    dbgid: DbgId,
    /// vector of [filtered hooks](design\docs\algorithm\part02\chapter_01_choose_hook.md)
    value: Option<HookFilterCtx>,
    /// [Context] instance, where store all info about initial data and each algorithm result's
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl  HookFilter {
    ///
    /// New instance [HookFilter]
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
impl Eval<(), EvalResult> for HookFilter {
    ///
    /// Method of filtering hooks by user loading capacity
    /// [reference to filtering documentation](design\docs\algorithm\part02\chapter_01_choose_hook.md)
    fn eval(&mut self, _: ()) -> BoxFuture<'_, EvalResult> {
        Box::pin(async {
            let result = self.ctx.eval(()).await;
            match result {
                CtxResult::Ok(ctx) => {
                    match self.value.clone() {
                        Some(hook_filter) => ctx.write(hook_filter),
                        None => {
                            let initial = ContextRead::<InitialCtx>::read(&ctx);
                            let user_loading_capacity = initial.load_capacity.clone();
                            let user_mech_work_type = initial.mechanism_work_type.clone();
                            let result: Vec<Hook> = initial
                                .hooks
                                .iter()
                                .cloned()
                                .filter(|hook| match user_mech_work_type {
                                    MechanismWorkType::M1
                                    | MechanismWorkType::M2
                                    | MechanismWorkType::M3 => {
                                        hook.load_capacity_m13 >= user_loading_capacity
                                    }
                                    MechanismWorkType::M4
                                    | MechanismWorkType::M5
                                    | MechanismWorkType::M6 => {
                                        hook.load_capacity_m46 >= user_loading_capacity
                                    }
                                    MechanismWorkType::M7 | MechanismWorkType::M8 => {
                                        hook.load_capacity_m78 >= user_loading_capacity
                                    }
                                })
                                .collect();
                            if result.is_empty() {
                                CtxResult::Err(StrErr(format!(
                                    "{}.eval | No available variants of hook for specified requirements",
                                    self.dbgid,
                                )))
                            } else {
                                let result = HookFilterCtx { result };
                                self.value = Some(result.clone());
                                ctx.write(result)
                            }
                        }
                    }
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
impl std::fmt::Debug for HookFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookFilter")
            .field("dbgid", &self.dbgid)
            .field("value", &self.value)
            // .field("ctx", &self.ctx)
            .finish()
    }
}
