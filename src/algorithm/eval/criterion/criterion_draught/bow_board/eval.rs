use crate::algorithm::context::context_access::ContextReadRef;
use crate::algorithm::entities::Draught;
use crate::algorithm::eval::criterion::*;
use crate::prelude::InitialCtx;
use crate::{
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::*,
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
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
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
                let ship_parameters = initial.ship_parameters.as_ref().unwrap();
                let bow_h_min = *ship_parameters
                    .get("Calculated minimum bow height")
                    .ok_or(error.err("No bow_h_min in ship_parameters"))?;
                let mut result = Vec::new();
                let draught = Draught::new(&self.dbg, &ctx).map_err(|err| error.pass(err))?;
                for v in data {
                    let delta_h = v.pos.z() - draught.value(&v.pos);
                    log::info!(
                        "Criterion DepthAtForwardPerpendicular point:{} delta_h:{:.3} bow_h_min:{:.3}",
                        v.pos.print(),
                        delta_h,
                        bow_h_min
                    );
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
                let result = BowBoardCtx { data: result };
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
