use futures::future::BoxFuture;
use crate::{
    algorithm::{context::{context_access::{ContextRead, ContextWrite}, ctx_result::CtxResult}, hook_filter::hook_filter_ctx::HookFilterCtx},
    infrostructure::client::choose_user_hook::ChooseUserHookReply,
    kernel::{dbgid::dbgid::DbgId, eval::Eval, request::Request, str_err::str_err::StrErr, types::eval_result::EvalResult},
};
use super::user_hook_ctx::UserHookCtx;
///
/// Represents user hook and make request to user for choosing one
pub struct UserHook {
    dbgid: DbgId,
    /// value of user hook
    value: Option<UserHookCtx>,
    /// Event interface
    req: Request<HookFilterCtx, ChooseUserHookReply>,
    /// [Context] instance, where store all info about initial data and each algorithm result's
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl UserHook {
    ///
    /// New instance [UserHook]
    /// - `ctx` - [Context]
    /// - `req` - [Request] for user
    pub fn new(req: Request<HookFilterCtx, ChooseUserHookReply>, ctx: impl Eval<(), EvalResult> + Send + 'static) -> Self{
        Self { 
            dbgid: DbgId("UserHook".to_string()), 
            value: None,
            req: req,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for UserHook {
    fn eval(&mut self, _: ()) -> BoxFuture<'_, EvalResult> {
        Box::pin(async {
            let result = self.ctx.eval(()).await;
            match result {
                CtxResult::Ok(ctx) => {
                    let variants: &HookFilterCtx = ctx.read();
                    let reply = self.req.fetch(variants.to_owned()).await;
                    let result = UserHookCtx { result: reply.choosen };
                    self.value = Some(result.clone());
                    ctx.write(result)
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
impl std::fmt::Debug for UserHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserHook")
            .field("dbgid", &self.dbgid)
            .field("value", &self.value)
            // .field("ctx", &self.ctx)
            .finish()
    }
}