use crate::algorithm::context::context_access::{ContextParamsRead, ContextReadRef};
use crate::algorithm::entities::Position;
use crate::algorithm::eval::ScrewCtx;
use crate::algorithm::eval::parameters::ParameterID;
use crate::algorithm::eval::{CriterionData, CriterionID};
use crate::prelude::InitialCtx;
use crate::{
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::*,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия заглубления винта
pub struct ScrewEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl ScrewEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "ScrewEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for ScrewEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let data = initial.screw.as_ref().unwrap();
                let ship_parameters = initial.ship_parameters.as_ref().unwrap();
                let midel_x = *ship_parameters
                    .get("X midship from Fr0")
                    .ok_or(error.err("Nomidship in ship_parameters"))?;
                let heel = ctx.read_params(ParameterID::Roll).to_degrees();
                let trim = ctx.read_params(ParameterID::TrimDeg).to_radians();
                let draught_mid = ctx.read_params(ParameterID::DraughtMid);
                let tg_t = trim.to_radians().tan();
                let tg_h = heel.to_radians().tan();
                let cos_h = heel.to_radians().cos();
                let current_draught = |p: &Position| {
                    let d_zi = p.y() * tg_h + (p.x() - midel_x) * tg_t / cos_h;
                    p.z() - draught_mid - d_zi
                };
                let mut result = Vec::new();
                for v in data.iter() {
                    let z_fix = current_draught(&v.pos);
                    let percent = (1. - z_fix / v.d).clamp(0., 2.) * 50.;
                    log::info!(
                        "Criterion ScrewImmersion point:{} z_fix:{:.3} percent:{:.3}",
                        v.pos.print(),
                        z_fix,
                        percent
                    );
                    result.push(if v.pos.y() < -1. {
                        CriterionData::new_result(CriterionID::ScrewImmersionPS, percent, 100.)
                    } else if v.pos.y() > 1. {
                        CriterionData::new_result(CriterionID::ScrewImmersionSB, percent, 100.)
                    } else {
                        CriterionData::new_result(CriterionID::ScrewImmersionCL, percent, 100.)
                    });
                }
                let result = ScrewCtx { data: result };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ScrewEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrewEval").field("dbg", &self.dbg).finish()
    }
}
