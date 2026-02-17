use crate::algorithm::eval::parameters::ParameterID;
use crate::algorithm::eval::seakeeping::main_resonant_zone::main_resonant_zone_ctx::MainResonantZoneCtx;
use crate::algorithm::eval::zg::Zg;
use crate::prelude::{ContextParamsRead, ContextWrite};
use crate::kernel::{
        Eval, 
        types::eval_result::EvalResult
    };
use sal_core::{
    dbg::Dbg, 
    error::Error
};
///
/// Расчет [основной зоны резонанса бортовой качки](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#условия-возникновения-опасных-явлений)
pub struct MainResonantZoneEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl MainResonantZoneEval {
    ///
    /// Новый экземпляр [MainResonantZoneEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "MainResonantZoneEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for MainResonantZoneEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let roll_frequency = 1.0/ctx.read_params(ParameterID::RollPeriod).max(f64::MIN);
                let left_side = 0.7 * roll_frequency;
                let right_side = 1.3 * roll_frequency;
                let result = MainResonantZoneCtx {
                    left_side,
                    right_side,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for MainResonantZoneEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MainResonantZoneEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
