use crate::LoadLineCtx;
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
                let ship_length = *ship_parameters
                    .get("LBP")
                    .ok_or(error.err("No LBP in ship_parameters"))?;
                let roll = ctx.read_params(ParameterID::Roll).to_degrees();  
                let draught_bow = ctx.read_params(ParameterID::DraughtBow);    
                let draught_stern = ctx.read_params(ParameterID::DraughtStern);    
                let draught_mid = ctx.read_params(ParameterID::DraughtMid);
                let delta_draught = (draught_bow - draught_stern) / ship_length;
                let mut result = Vec::new();
                let draught_value = |pos_x: f64| -> f64 {
                    draught_mid
                    + delta_draught 
                    * pos_x
                };                
                for v in data.iter() {
                    let z_fix = draught_value(v.pos.x()) + v.pos.y() * roll.sin();
                    let z_target = v.pos.z();

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
