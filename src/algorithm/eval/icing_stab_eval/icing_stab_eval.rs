use super::icing_stab_ctx::IcingStabCtx;
use crate::algorithm::context::context_access::*;
use crate::{
    algorithm::entities::icing_stab::IcingStabType,
    kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ContextWrite, CtxResult,
};
use sal_sync::services::entity::error::str_err::StrErr;

///
/// Коэффициенты для расчета обледенения судна
pub struct IcingStabEval {
    dbg: DbgId,
    value: Option<IcingStabCtx>,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl IcingStabEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + Send + 'static) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "IcingStabEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for IcingStabEval {
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
                    let icing = match initial.icing.clone() {
                        Some(data) => data.data(),
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let icing_stab = match IcingStabType::from_str(&voyage.icing_type) {
                        Ok(data) => data,
                        Err(err) => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_stab error: {:?}",
                                self.dbg, err
                            )))
                        }
                    };
                    let icing_m_timber = *match icing.get("icing_m_timber") {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_m_timber error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let icing_m_v_full = *match icing.get("icing_m_v_full") {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_m_v_full error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let icing_m_v_half = *match icing.get("icing_m_v_half") {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_m_v_half error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let icing_m_h_full = *match icing.get("icing_m_h_full") {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_m_h_full error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let icing_m_h_half = *match icing.get("icing_m_h_half") {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_m_h_half error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let icing_coef_v_area_full = *match icing.get("icing_coef_v_area_full") {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_coef_v_area_full error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let icing_coef_v_area_half = *match icing.get("icing_coef_v_area_half") {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_coef_v_area_half error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let icing_coef_v_area_zero = *match icing.get("icing_coef_v_area_zero") {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_coef_v_area_zero error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let icing_coef_v_moment_full = *match icing.get("icing_coef_v_moment_full") {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_coef_v_moment_full error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let icing_coef_v_moment_half = *match icing.get("icing_coef_v_moment_half") {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_coef_v_moment_half error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let icing_coef_v_moment_zero = *match icing.get("icing_coef_v_moment_zero") {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_coef_v_moment_zero error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let mass_desc_h = match icing_stab {
                        IcingStabType::Full => icing_m_h_full,
                        IcingStabType::Half => icing_m_h_half,
                        _ => 0.,
                    };
                    let mass_timber_h = match icing_stab {
                        IcingStabType::Full | IcingStabType::Half => icing_m_timber,
                        _ => 0.,
                    };
                    let mass_v = match icing_stab {
                        IcingStabType::Full => icing_m_v_full,
                        IcingStabType::Half => icing_m_v_half,
                        _ => 0.,
                    };
                    let coef_v_area = match icing_stab {
                        IcingStabType::Full => icing_coef_v_area_full,
                        IcingStabType::Half => icing_coef_v_area_half,
                        _ => icing_coef_v_area_zero,
                    };
                    let coef_v_ds_area = icing_coef_v_area_zero;
                    let coef_v_moment = match icing_stab {
                        IcingStabType::Full => icing_coef_v_moment_full,
                        IcingStabType::Half => icing_coef_v_moment_half,
                        _ => icing_coef_v_moment_zero,
                    };
                    let is_some =
                        matches!(icing_stab, IcingStabType::Full | IcingStabType::Half);
                    let result = IcingStabCtx {
                        mass_desc_h,
                        mass_timber_h,
                        mass_v,
                        coef_v_area,
                        coef_v_ds_area,
                        coef_v_moment,
                        is_some,
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
impl std::fmt::Debug for IcingStabEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcingStabEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
