use crate::algorithm::eval::ScrewCtx;
use crate::algorithm::context::context_access::{ContextParamsRead, ContextReadRef};
use crate::algorithm::eval::parameters::ParameterID;
use crate::algorithm::eval::{CriterionData, CriterionID};
use crate::prelude::InitialCtx;
use crate::{
    prelude::*,
    kernel::{Eval, types::eval_result::EvalResult},
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
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + Send + Sync + 'static) -> Self {
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
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .unwrap();          
                let ship_length = *ship_parameters
                    .get("LBP")
                    .ok_or(error.err("No LBP in ship_parameters"))?;
                let roll = ctx.read_params(ParameterID::Roll);  
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
                    let z_fix = v.pos.z()  - v.pos.y() * roll.sin() - draught_value(v.pos.x());
                    let percent = (1. - z_fix/v.d).clamp(0., 2.)*50.;
        //         dbg!(&v.pos, v.pos.y() * (roll * PI / 180.).sin(), self.draught.value(v.pos.x())?, z_fix, percent);
                    result.push(if v.pos.y() < -1. {
                        CriterionData::new_result(CriterionID::ScrewImmersionPS, percent, 100.)
                    } else if v.pos.y() > 1. {
                        CriterionData::new_result(CriterionID::ScrewImmersionSB, percent, 100.)
                    } else {
                        CriterionData::new_result(CriterionID::ScrewImmersionCL, percent, 100.)
                    });
                }
                let result = ScrewCtx {
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
impl std::fmt::Debug for ScrewEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrewEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
