use sal_sync::services::entity::error::str_err::StrErr;
use crate::{
    kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult}, ship_model::model_link::ModelLink, ContextWrite, CtxResult
};

use super::mass_ctx::MassCtx;

///
/// Площади боковой и горизонтальной поверхностей для расчета прочности
pub struct MassEval {
    dbg: DbgId,
    value: Option<MassCtx>,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl MassEval {
    ///
    /// Fetches all initiall data
    /// - 'api_client' - access to the database
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + Send + 'static) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "MassEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for MassEval {
    fn eval(&mut self, _: ()) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            match self.ctx.eval(()).await {
                CtxResult::Ok(ctx) => {


                },
                CtxResult::Err(err) => CtxResult::Err(StrErr(format!(
                    "{}.eval | Read context error: {:?}",
                    self.dbg, err
                ))),
                CtxResult::None => CtxResult::None,
            }
        })
    }
}
//
//
impl std::fmt::Debug for MassEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MassEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}