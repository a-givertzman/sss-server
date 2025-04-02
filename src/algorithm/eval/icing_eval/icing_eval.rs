use super::icing_ctx::IcingCtx;
use crate::algorithm::context::context_access::*;
use crate::algorithm::entities::Moment;
use crate::algorithm::eval::{IcingStabCtx, StrengthAreaCtx};
use crate::{
    kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ContextWrite, CtxResult,
};
use sal_sync::services::entity::error::str_err::StrErr;

///
/// Учет обледенения судна.
pub struct IcingEval {
    dbg: DbgId,
    value: Option<IcingCtx>,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl IcingEval {
    ///
    /// Fetches all initiall data
    /// - 'api_client' - access to the database
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + 'static,
    ) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "IcingEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
    //
}
//
impl Eval<(), EvalResult> for IcingEval {
    fn eval(&mut self, _: ()) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            match self.ctx.eval(()).await {
                CtxResult::Ok(ctx) => {
                    let initial: &InitialCtx = ctx.read_ref();
                    let bounds = match initial.bounds.clone() {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read bounds error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let area_strength: StrengthAreaCtx = ctx.read();
                    let icing_stab: IcingStabCtx = ctx.read();
                    let mut mass_values = Vec::new();
                    let mut mass_moment_x_sum = 0.;
                    for (i, bound) in bounds.iter().enumerate() { 
                        let current_x = match bound.center() {
                            Some(data) => data,
                            None => {
                                return CtxResult::Err(StrErr(format!(
                                    "{}.eval | bound.center error: no center for bound {i}", self.dbg
                                )));
                            }
                        };
                        let current_area_h = match area_strength.area_h_values.get(i) {
                            Some(&data) => data,
                            None => {
                                return CtxResult::Err(StrErr(format!(
                                    "{}.eval | area_strength.area_h.get error: no value for bound {i}", self.dbg
                                )));
                            }
                        };
                        let current_area_v = match area_strength.area_v_values.get(i) {
                            Some(&data) => data,
                            None => {
                                return CtxResult::Err(StrErr(format!(
                                    "{}.eval | area_strength.area_v.get error: no value for bound {i}", self.dbg
                                )));
                            }
                        };
                        let current_area_timber_h = match area_strength.area_timber_h_values.get(i) {
                            Some(&data) => data,
                            None => {
                                return CtxResult::Err(StrErr(format!(
                                    "{}.eval | area_strength.area_timber_h.get error: no value for bound {i}", self.dbg
                                )));
                            }
                        };
                        let current_mass = current_area_h * icing_stab.mass_desc_h
                            + current_area_timber_h
                                * (icing_stab.mass_timber_h - icing_stab.mass_desc_h)
                            + current_area_v
                                * (1. + icing_stab.coef_v_ds_area)
                                * icing_stab.mass_v;
                        mass_moment_x_sum += current_mass * current_x;
                        mass_values.push(current_mass);
                    }
                    let mass_sum = mass_values.iter().sum();
                    let mass_shift_x = mass_moment_x_sum/mass_sum;
                    let result = IcingCtx {
                        mass: mass_sum,
                        mass_shift_x,
                        mass_values,
                    };
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
impl std::fmt::Debug for IcingEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcingEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
