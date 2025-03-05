use sal_sync::services::entity::error::str_err::StrErr;
use crate::{
    algorithm::{
        context::{context::Context, context_access::{ContextRead, ContextWrite}, ctx_result::CtxResult},
        entities::{serde_parser::IFromJson, strength::{ComputedFrameData, ComputedFrameDataArray}, DataArray},
    },
    infrostructure::{api::client::api_client::ApiClient, query::restart_eval::RestartEvalQuery},
    kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult}
};

use super::initial_ctx::InitialCtx;

///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct Initial {
    dbg: DbgId,
    api_client: ApiClient,
    ctx: Context,
}
//
//
impl Initial {
    ///
    /// Fetches all initiall data
    /// - 'api_client' - access to the database
    pub fn new(parent: impl Into<String>, api_client: ApiClient, ctx: Context) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "Initial");
        Self {
            dbg,
            api_client,
            ctx
        }
    }
    //
    //
}
impl Eval<RestartEvalQuery, EvalResult> for Initial {
    fn eval(&mut self, query: RestartEvalQuery) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            let initial_ctx: &InitialCtx = ContextRead::read(&self.ctx);
            let mut initial_ctx = initial_ctx.to_owned();
            let bounds = self.api_client
                .fetch(&format!(
                    "SELECT index, start_x, end_x FROM computed_frame_space WHERE ship_id={};",
                    query.ship_id
                ));
            match bounds {
                Ok(bounds) => {
                    match ComputedFrameDataArray::parse(&bounds) {
                        Ok(bounds) => {
                            let bounds: DataArray<ComputedFrameData> = bounds;
                            initial_ctx.bounds = Some(bounds.data());
                            self.ctx.clone().write(initial_ctx.to_owned())
                        }
                        Err(err) => CtxResult::Err(StrErr(format!("{}.eval | Error: {err}", self.dbg))),
                    }
                }
                Err(err) => CtxResult::Err(StrErr(format!("{}.eval | Error: {err}", self.dbg))),
            }
        })
    }
}
