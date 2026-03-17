use crate::algorithm::entities::Draught;
use crate::algorithm::eval::criterion::*;
use crate::{
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::*,
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
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
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
                let ship_parameters = initial.ship_parameters.as_ref().unwrap();
                let delta_d = *ship_parameters
                    .get("Allowance for fresh water for freeboard")
                    .ok_or(error.err("Allowance for fresh water for freeboard in ship_parameters"))?;
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let water_density = voyage.density;
                let data = initial.load_line.as_ref().unwrap();
                let mut result = Vec::new();
                let draught = Draught::new(&self.dbg, &ctx).map_err(|err| error.pass(err))?;
                for v in data.iter() {
                    let z_fix = draught.value(&v.pos);
                    let z_target = v.pos.z() + delta_d*(1025.0 - water_density*1000.)/25.0;
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
                            log::error!("load_line criterion_id error: {}", e)
                        }
                    }
                }
                let result = LoadLineCtx { data: result };
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
