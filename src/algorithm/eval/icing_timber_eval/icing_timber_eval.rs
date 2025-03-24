use crate::algorithm::context::context_access::*;
use crate::{
    kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ContextWrite, CtxResult,
};
use sal_sync::services::entity::error::str_err::StrErr;
use super::icing_timber_ctx::{IcingTimberCtx, IcingTimberType};

///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
pub struct IcingTimberEval {
    dbg: DbgId,
    value: Option<IcingTimberCtx>,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl IcingTimberEval {
    ///
    /// Fetches all initiall data
    /// - 'api_client' - access to the database
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + 'static,
    ) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "IcingTimberEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for IcingTimberEval {
    fn eval(&mut self, _: ()) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            match self.ctx.eval(()).await {
                CtxResult::Ok(ctx) => {
                    let initial: &InitialCtx = ctx.read_ref();
                    let voyage = match initial.voyage.clone() {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read voyage error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let icing_timber_stab = match IcingTimberType::from_str(&voyage.icing_timber_type) {
                        Ok(data) => data,
                        Err(err) => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_timber_stab error: {:?}",
                                self.dbg, err
                            )))
                        }
                    };
                    let ship_parameters = match initial.ship_parameters.as_ref() {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read voyage error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let length_loa = match ship_parameters.get("L.O.A") {
                        Some(data) => *data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read length_loa error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let width = match ship_parameters.get("MouldedBreadth") {
                        Some(data) => *data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read width error: no data!",
                                self.dbg
                            )))
                        }
                    };                 
                    let result = IcingTimberCtx::new(width, length_loa, icing_timber_stab);
                    self.value = Some(result.clone());
                    ctx.write(result)
                }
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
impl std::fmt::Debug for IcingTimberEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcingTimberEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
