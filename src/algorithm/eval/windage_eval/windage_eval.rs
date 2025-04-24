use super::windage_ctx::WindageCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextParamsRead, ContextRead, ContextReadRef}, eval::{IcingStabCtx, StabilityAreaCtx}
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ContextWrite, CtxResult,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Парусность судна, площадь и положение
/// центра относительно миделя и ОП
pub struct WindageEval {
    dbg: Dbg,
    value: Option<WindageCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl WindageEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "WindageEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for WindageEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let volume_shift_z = ctx.read_params(ParameterID::CenterVolumeZ);
                let stability_area: StabilityAreaCtx = ctx.read();
                let icing_stab: IcingStabCtx = ctx.read();
                let area_v = stability_area.area_v;
                let coef = 1. + icing_stab.coef_v_area;
                let a_v = area_v * coef;              
                let m_vz = stability_area.moment_v.z();
                let z_v_bp = m_vz / a_v;
                let z_v = z_v_bp - volume_shift_z;
                let result = WindageCtx {
                    a_v,
                    z_v,
                };
                self.value = Some(result.clone());
                ctx.write(result)
            }
            CtxResult::Err(err) => CtxResult::Err(error.pass_with("Read context error", err)),
            CtxResult::None => CtxResult::None,
        }
    }
}
//
//
impl std::fmt::Debug for WindageEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindageEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
