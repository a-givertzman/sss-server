use super::bow_board_ctx::BowBoardCtx;
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
/// Расчет критерия высоты на носовом перпендикуляре
pub struct BowBoardEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl BowBoardEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "BowBoardEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for BowBoardEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let data = initial.bow_board.as_ref().unwrap();   
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .unwrap();          
                let ship_length_lbp = *ship_parameters
                    .get("LBP")
                    .ok_or(error.err("No LBP in ship_parameters"))?;
                let bow_h_min = *ship_parameters
                    .get("Calculated minimum bow height")
                    .ok_or(error.err("No bow_h_min in ship_parameters"))?;
                let roll = ctx.read_params(ParameterID::Roll).to_degrees();  
                let trim = ctx.read_params(ParameterID::TrimDeg).to_radians();
                let draught_bow = ctx.read_params(ParameterID::DraughtBow);    
                let draught_stern = ctx.read_params(ParameterID::DraughtStern);    
                let draught_mid = ctx.read_params(ParameterID::DraughtMid);
                let delta_draught = (draught_bow - draught_stern) / ship_length_lbp;
                let mut result = Vec::new();
                let draught_value = |pos_x: f64| -> f64 {
                    draught_mid
                    + delta_draught 
                    * pos_x
                };  
                for v in data {
                    let delta_h = (v.pos.z() - v.pos.y() * roll.sin() - draught_value(v.pos.x()))*trim.cos();
                    result.push(if v.pos.y() <= 0. {
                        CriterionData::new_result(
                            CriterionID::DepthAtForwardPerpendicularPS,
                            delta_h,
                            bow_h_min,
                        )
                    } else {
                        CriterionData::new_result(
                            CriterionID::DepthAtForwardPerpendicularSB,
                            delta_h,
                            bow_h_min,
                        )
                    });
                }
                let result = BowBoardCtx {
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
impl std::fmt::Debug for BowBoardEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BowBoardEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
