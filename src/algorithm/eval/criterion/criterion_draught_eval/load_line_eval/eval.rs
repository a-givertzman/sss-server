use crate::algorithm::entities::Position;
use crate::algorithm::eval::LoadLineCtx;
use crate::algorithm::context::context_access::ContextParamsRead;
use crate::algorithm::eval::parameters::ParameterID;
use crate::algorithm::eval::{CriterionData, CriterionID};
use crate::{
    prelude::*,
    kernel::{Eval, types::eval_result::EvalResult},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия осадки по грузовой марке
pub struct LoadLineEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl LoadLineEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "LoadLineEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for LoadLineEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let data = initial.load_line.as_ref().unwrap();
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .unwrap();
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
                    let z_target = v.pos.z();
                    log::info!(
                        "Criterion LoadLine point:{} z_fix:{:.3} z_target:{:.3}",
                        v.pos.print(),
                        z_fix,
                        z_target
                    );
                    match CriterionID::from(v.criterion_id) {
                        Ok(criterion_id) => {
                            result.push(CriterionData::new_result(criterion_id, z_fix, z_target))
                        }
                        Err(e) => {
                            log::error!("load_line criterion_id error: {}", e.to_string())
                        }
                    }
                }
                let result = LoadLineCtx {
                    data: result,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for LoadLineEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadLineEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
