use futures::future::BoxFuture;
use crate::{
    algorithm::{bearing_filter::bearing_filter_ctx::BearingFilterCtx, context::{context_access::{ContextRead, ContextWrite}, ctx_result::CtxResult}},
    infrostructure::client::choose_user_bearing::ChooseUserBearingReply,
    kernel::{dbgid::dbgid::DbgId, eval::Eval, request::Request, str_err::str_err::StrErr, types::eval_result::EvalResult},
};
use super::user_bearing_ctx::UserBearingCtx;
///
/// Represents user bearing and make request to user for choosing one
pub struct UserBearing {
    dbgid: DbgId,
    /// value of user hook
    value: Option<UserBearingCtx>,
    /// Event interface
    req: Request<BearingFilterCtx, ChooseUserBearingReply>,
    /// [Context] instance, where store all info about initial data and each algorithm result's
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl UserBearing {
    ///
    /// New instance [UserBearing]
    /// - `ctx` - [Context]
    /// - `req` - [Request] for user
    pub fn new(req: Request<BearingFilterCtx, ChooseUserBearingReply>, ctx: impl Eval<(), EvalResult> + Send + 'static) -> Self {
        Self { 
            dbgid: DbgId("UserBearing".to_string()), 
            value: None,
            req,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for UserBearing {
    fn eval(&mut self, _: ()) -> BoxFuture<'_, EvalResult> {
        Box::pin(async {
            let result = self.ctx.eval(()).await;
            match result {
                CtxResult::Ok(ctx) => {
                    let variants: &BearingFilterCtx = ctx.read();
                    let variants = variants.to_owned();
                    let reply = self.req.fetch(variants).await;
                    let result = UserBearingCtx { result: reply.answer };
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
impl std::fmt::Debug for UserBearing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserBearing")
            .field("dbgid", &self.dbgid)
            .field("value", &self.value)
            .finish()
    }
}
