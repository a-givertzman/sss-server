use sal_sync::services::entity::error::str_err::StrErr;
use crate::{
    algorithm::{
        context::{context::Context, context_access::{ContextReadRef, ContextWrite}, ctx_result::CtxResult},
        entities::{serde_parser::IFromJson, strength::{ComputedFrameData, ComputedFrameDataArray}, DataArray}, initial::initial_ctx::InitialCtx,
    },
    infrostructure::{api::client::api_client::ApiClient, query::restart_eval::RestartEvalQuery},
    kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult}, ship_model::model_link::ModelLink
};

use super::areas_strength_ctx::AreasStrengthCtx;


///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
pub struct AreasStrength {
    dbg: DbgId,
    model: ModelLink,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl AreasStrength {
    ///
    /// Fetches all initiall data
    /// - 'api_client' - access to the database
    pub fn new(parent: impl Into<String>, model: ModelLink, ctx: impl Eval<(), EvalResult> + Send + 'static) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "AreasStrength");
        Self {
            dbg,
            model,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for AreasStrength {
    fn eval(&mut self, _: ()) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            match self.ctx.eval(()).await {
                CtxResult::Ok(ctx) => {
                    match self.model.areas().await {
                        Ok(areas) => {
                            let result = AreasStrengthCtx { areas };
                            ctx.write(result)
                        },
                        Err(err) => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read context error: {:?}",
                                self.dbg, err
                            )));
                        },
                    }
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
impl std::fmt::Debug for AreasStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AreasStrength")
            .field("dbg", &self.dbg)
            .finish()
    }
}