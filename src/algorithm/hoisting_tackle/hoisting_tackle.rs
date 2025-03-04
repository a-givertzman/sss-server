use futures::future::BoxFuture;
use crate::{
    algorithm::context::{context_access::ContextWrite, ctx_result::CtxResult},
    infrostructure::client::change_hoisting_tackle::ChangeHoistingTackleReply,
    kernel::{dbgid::dbgid::DbgId, eval::Eval, request::Request, str_err::str_err::StrErr, types::eval_result::EvalResult},
};
use super::hoisting_tackle_ctx::HoistingTackleCtx;
///
/// Represents hoisting tackle and make request to user for changing one
pub struct HoistingTackle {
    dbgid: DbgId,
    /// value of hoisting tackle
    value: Option<HoistingTackleCtx>,
    req: Request<(), ChangeHoistingTackleReply>,
    /// [Context] instance, where store all info about initial data and each algorithm result's
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl HoistingTackle {
    ///
    /// New instance [HoistingTackle]
    /// - `ctx` - [Context]
    /// - `req` - [Request] for user
    pub fn new(req: Request<(), ChangeHoistingTackleReply>, ctx: impl Eval<(), EvalResult> + Send + 'static) -> Self {
        Self { 
            dbgid: DbgId("HoistingTackle".to_string()), 
            value: None,
            req,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for HoistingTackle {
    fn eval(&mut self, _: ()) -> BoxFuture<'_, EvalResult> {
        Box::pin(async {
            match self.ctx.eval(()).await {
                CtxResult::Ok(ctx) => {
                    let reply = self.req.fetch(()).await;
                    let result = HoistingTackleCtx { result: reply.answer };
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
impl std::fmt::Debug for HoistingTackle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoistingTackle")
            .field("dbgid", &self.dbgid)
            .field("value", &self.value)
            .finish()
    }
}