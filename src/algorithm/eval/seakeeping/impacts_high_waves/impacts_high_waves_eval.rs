use crate::algorithm::context::context_access::ContextReadRef;
use crate::algorithm::entities::recalculation_course_angular::RecalculationCourseAngular;
use crate::algorithm::eval::impacts_high_waves::impacts_high_waves_ctx::ImpactsHighWavesCtx;
use crate::algorithm::eval::period_excitement::period_excitement_ctx::PeriodExcitementCtx;
use crate::algorithm::eval::zg_eval::Zg;
use crate::prelude::InitialCtx;
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
    /// Array of vessel speed
    fn vessel_speed() -> Vec<f64> {
        let mut start = 1.29;
        let end = 2.0;
        let step = 0.0071;
        let mut result: Vec<f64> = Vec::new();
        while start < end {
            result.push(start);
            start += step;
        }
        result
    }
}
//
//
impl Eval<Zg, EvalResult> for ImpactsHighWavesEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let course_angle = ContextReadRef::<InitialCtx>::read_ref(&ctx).course_angle.unwrap();
                let period_excitement = ContextRead::<PeriodExcitementCtx>::read(&ctx).period_excitement.clone();
                let course_angle_of_wave: Vec<f64> = (1350..=2250).map(|x| x as f64 / 10.0).collect(); 
                let vessel_speed = Self::vessel_speed();  
                let result = course_angle_of_wave.iter()
                .flat_map(|angle| {
                    vessel_speed.iter()
                    .map(move |speed| {
                        let speed = (speed / (angle * std::f64::consts::PI / 180.0).cos().abs()) * period_excitement;
                        (
                            *angle,
                            speed,
                        )
                    }).collect::<Vec<_>>()
                }).collect::<Vec<_>>();
                let result = ImpactsHighWavesCtx {
                    impacts_high_waves: RecalculationCourseAngular::to_northeastern(
                        course_angle, 
                        result
                    ),
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
