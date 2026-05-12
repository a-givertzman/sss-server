use sal_core::dbg::Dbg;

use crate::algorithm::eval::seakeeping::eval::main_resonant_zone::main_resonant_zone_ctx::MainResonantZoneCtx;
use crate::kernel::Eval;
use crate::kernel::types::eval_result::EvalResult;
use crate::prelude::ContextParamsRead;
use crate::prelude::ContextWrite;
///
/// Расчет [основной зоны резонанса бортовой качки](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#условия-возникновения-опасных-явлений)
pub struct MainResonantZoneEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl MainResonantZoneEval {
    ///
    /// Новый экземпляр [MainResonantZoneEval]
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "MainResonantZoneEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for MainResonantZoneEval {
    fn eval(&self, _: ()) -> EvalResult {
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let roll_frequency = ctx.read_params(crate::algorithm::eval::parameters::ParameterID::RollPeriod);
                let left_side = 0.7 * roll_frequency;
                let right_side = 1.3 * roll_frequency;
                let result = MainResonantZoneCtx {
                    left_side,
                    right_side,
                };
                ctx.write(result)
            }
            Err(err) => Err(err),
        }
    }
}
