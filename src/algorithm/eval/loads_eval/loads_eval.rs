use super::loads_ctx::LoadsCtx;
use crate::algorithm::context::context_access::*;
use crate::algorithm::entities::Position;
use crate::ship_model::model_link::ModelLink;
use crate::{
    kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ContextWrite, CtxResult,
};
use sal_sync::services::entity::error::str_err::StrErr;

///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
pub struct LoadsEval {
    dbg: DbgId,
    model: ModelLink,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl LoadsEval {
    ///
    /// Fetches all initiall data
    /// - 'api_client' - access to the database
    pub fn new(
        parent: impl Into<String>,
        model: ModelLink,
        ctx: impl Eval<(), EvalResult> + Send + 'static,
    ) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "LoadsEval");
        Self {
            dbg,
            model,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for LoadsEval {
    fn eval(&mut self, _: ()) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            match self.ctx.eval(()).await {
                CtxResult::Ok(ctx) => {
                    let initial: &InitialCtx = ctx.read_ref();
                    let shift_const = if let Some(ship_parameters) = initial.ship_parameters {
                        let ship_data = ship_parameters.data();
                        let const_mass_shift_x = *match ship_data.get("LCG from middle") { 
                            Some(data) => data,
                            None => {
                                return CtxResult::Err(StrErr(format!(
                                    "{}.eval | Read const_mass_shift_x error: no data!",
                                    self.dbg
                                )))
                            }
                        };
                        let const_mass_shift_y = *match ship_data.get("TCG from CL") { 
                            Some(data) => data,
                            None => {
                                return CtxResult::Err(StrErr(format!(
                                    "{}.eval | Read const_mass_shift_y error: no data!",
                                    self.dbg
                                )))
                            }
                        };     
                        let const_mass_shift_z = *match ship_data.get("VCG from BL") { 
                            Some(data) => data,
                            None => {
                                return CtxResult::Err(StrErr(format!(
                                    "{}.eval | Read const_mass_shift_z error: no data!",
                                    self.dbg
                                )))
                            }
                        }; 
                        Position::new(const_mass_shift_x, const_mass_shift_y, const_mass_shift_z)
                    } else {
                        return CtxResult::Err(StrErr(format!(
                            "{}.eval | Read const_mass_shift_z error: no ship_parameters!",
                            self.dbg
                        )));
                    };
                    let load_constant = match initial.load_constant.clone() {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read load_constant error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let bulk = match initial.bulk.clone() {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read bulk error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let liquid = match initial.liquid.clone() {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read liquid error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let unit = match initial.unit.clone() {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read unit error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let gaseous = match initial.gaseous.clone() {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read gaseous error: no data!",
                                self.dbg
                            )))
                        }
                    };


                    let result = LoadsCtx {
                        load_constant: initial.load_constant.data,
                        shift_const,
                        bulk,
                        liquid,
                        unit,
                        gaseous
                    };
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
impl std::fmt::Debug for IcingStabEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcingStabEval").field("dbg", &self.dbg).finish()
    }
}
