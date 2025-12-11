use crate::WindageCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextParamsRead, ContextRead}, eval::{parameters::ParameterID, zg_eval::Zg, IcingStabCtx, StabilityAreaCtx}
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::ContextWrite,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Парусность судна, площадь и положение
/// центра относительно миделя и ОП
pub struct WindageEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl WindageEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "WindageEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for WindageEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let volume_shift_z = 
                let stability_area: StabilityAreaCtx = ctx.read();
                let icing_stab: IcingStabCtx = ctx.read();
                let area_v = stability_area.area_v;
                let volume_shift_z = stability_area.area_v;
                let coef = 1. + icing_stab.coef_v_area;
                let a_v = area_v * coef;              
                let m_vz = stability_area.moment_v.z();
                let z_v_bp = m_vz / a_v;
                let z_v = z_v_bp - volume_shift_z;
                let result = WindageCtx {
                    a_v,
                    z_v,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for WindageEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindageEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
