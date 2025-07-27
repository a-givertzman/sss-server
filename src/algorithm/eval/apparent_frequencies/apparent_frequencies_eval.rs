use crate::algorithm::context::context_access::ContextRead;
use crate::algorithm::eval::zg_eval::Zg;
use crate::algorithm::eval::{
    ApparentFrequenciesCtx, 
    PeriodExcitementCtx, 
    VesselMaxSpeedCtx
};
use crate::kernel::{
        eval::Eval, 
        types::eval_result::EvalResult
    };
use crate::prelude::ContextWrite;
use sal_core::{
    dbg::Dbg, 
    error::Error
};
///
/// Расчет массива [кажущихся частот волнения](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
pub struct ApparentFrequenciesEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl ApparentFrequenciesEval {
    ///
    /// Новый экземпляр [ApparentFrequenciesEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ApparentFrequenciesEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for ApparentFrequenciesEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let vmax = ContextRead::<VesselMaxSpeedCtx>::read(&ctx).vmax.clone();
                let period_excitement = ContextRead::<PeriodExcitementCtx>::read(&ctx).period_excitement.clone();
                let course_angle_of_wave: Vec<f64> = (0..=3600).map(|x| x as f64 / 10.0).collect();
                let vessel_speeds: Vec<f64> = (0..=(vmax.ceil() as isize * 10)).map(|x| x as f64 / 10.0).collect();
                let mut result: Vec<(f64,f64)> = Vec::new();
                for angle in course_angle_of_wave {
                    for speed in &vessel_speeds {
                        let apparent_frequency = 2.0 * std::f64::consts::PI 
                        * (3.0 * period_excitement + speed * angle.cos()).abs() 
                        / 3.0 * period_excitement.powf(2.0);
                        result.push(
                            (
                                angle, 
                                apparent_frequency,
                            )
                        );
                    }
                }
                ctx.write(
                    ApparentFrequenciesCtx {
                        apparent_frequencies: result,
                    }
                )
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ApparentFrequenciesEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApparentFrequenciesEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
