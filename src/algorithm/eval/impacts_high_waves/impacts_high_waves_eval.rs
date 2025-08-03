use crate::algorithm::eval::zg_eval::Zg;
use crate::algorithm::eval::{
    ImpactsHighWavesCtx, 
    PeriodExcitementCtx
};
use crate::{
    ContextWrite,
    algorithm::context::context_access::ContextRead,
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
/// Расчет [массива скоростей движения, при которых происходит явление последовательных ударов высоких волн](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
pub struct ImpactsHighWavesEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl ImpactsHighWavesEval {
    ///
    /// Новый экземпляр [ImpactsHighWavesEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ImpactsHighWavesEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    ///
    /// [Linear interpolation](https://en.wikipedia.org/wiki/Linear_interpolation)
    fn linear_interpolation(speed_min: f64, speed_max: f64, angle_min: f64, angle_max: f64, angle_curr: f64 ) -> f64{
        (speed_min * (angle_max - angle_curr) + speed_max * (angle_curr - angle_min)) /
        (angle_max - angle_min)
    }
}
//
//
impl Eval<Zg, EvalResult> for ImpactsHighWavesEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let period_excitement = ContextRead::<PeriodExcitementCtx>::read(&ctx).period_excitement.clone();
                let speed_min = 1.3 * period_excitement;
                let speed_max = 2.8 * period_excitement;
                let course_angle_of_wave: Vec<f64> = (1350..=2250).map(|x| x as f64 / 10.0).collect();
                let angle_min = *course_angle_of_wave.first().unwrap(); 
                let angle_max = *course_angle_of_wave.last().unwrap();
                let result = course_angle_of_wave.iter().map(| &angle_curr| {
                    let speed = Self::linear_interpolation(
                        speed_min, 
                        speed_max,  
                        angle_min,
                        angle_max,
                        angle_curr
                    );
                    (
                        angle_curr,
                        speed,
                    )
                }).collect::<Vec<(f64, f64)>>();
                let result = ImpactsHighWavesCtx {
                    impacts_high_waves: result,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ImpactsHighWavesEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImpactsHighWavesEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
