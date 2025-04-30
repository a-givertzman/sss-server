use super::bow_board_ctx::BowBoardCtx;
use crate::algorithm::context::context_access::ContextParamsRead;
use crate::algorithm::eval::parameters::ParameterID;
use crate::algorithm::eval::{CriterionData, CriterionID};
use crate::{
    ContextWrite, CtxResult,
    kernel::{eval::Eval, types::eval_result::EvalResult},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия высоты на носовом перпендикуляре
pub struct BowBoardEval {
    dbg: Dbg,
    value: Option<BowBoardCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl BowBoardEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "BowBoardEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for BowBoardEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let draught_bow = ctx.read_params(ParameterID::DraughtBow);    
                let draught_stern = ctx.read_params(ParameterID::DraughtStern);  
                let forward_trim = CriterionData::new_result(
                        CriterionID::MaximumForwardBowBoard,
                        draught_bow,
                        self.forward_trim,
                    );
                let aft_trim = CriterionData::new_result(CriterionID::MaximumAftBowBoard, draught_stern, self.aft_trim);
                let result = BowBoardCtx {
                    data: vec![aft_trim, forward_trim],
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
impl std::fmt::Debug for BowBoardEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BowBoardEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
