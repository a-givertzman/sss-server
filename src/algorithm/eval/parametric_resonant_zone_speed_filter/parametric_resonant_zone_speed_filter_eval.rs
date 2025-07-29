use crate::algorithm::context::context_access::ContextRead;
use crate::algorithm::eval::zg_eval::Zg;
use crate::algorithm::eval::{
    ApparentFrequenciesCtx, 
    ParametricResonantZoneCtx, 
    VesselSpeedFilterCtx
};
use crate::{
    ContextWrite,
    kernel::{
        eval::Eval, 
        types::eval_result::EvalResult
    },
};
use sal_core::{
    dbg::Dbg, 
    error::Error
};
///
/// Расчет расчета массива скоростей хода, при которых 
/// [кажущаяся частота волнения](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#условия-возникновения-опасных-явлений) 
/// находится в диапазоне [параметрического резонанса бортовой качки](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#условия-возникновения-опасных-явлений)
pub struct ParametricResonantZoneSpeedFilterEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl ParametricResonantZoneSpeedFilterEval {
    ///
    /// Новый экземпляр [ParametricResonantZoneSpeedFilterEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ParametricResonantZoneSpeedFilterEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for ParametricResonantZoneSpeedFilterEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let parametric_resonant_zone = ContextRead::<ParametricResonantZoneCtx>::read(&ctx).clone();
                let apparent_frequencies = ContextRead::<ApparentFrequenciesCtx>::read(&ctx).apparent_frequencies.clone();
                let mut result = Vec::new();
                for freq in apparent_frequencies {
                    if parametric_resonant_zone.left_side <= freq.1 && freq.1 <= parametric_resonant_zone.right_side {
                        result.push(freq.1);
                    }
                }
                ctx.write(
                    VesselSpeedFilterCtx {
                        vessel_speed_filter: result
                    }
                )
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ParametricResonantZoneSpeedFilterEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParametricResonantZoneSpeedFilterEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
