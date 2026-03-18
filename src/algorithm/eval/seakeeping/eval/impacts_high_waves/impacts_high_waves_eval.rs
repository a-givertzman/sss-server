use sal_core::{dbg::Dbg, error::Error};

use crate::{
    algorithm::eval::seakeeping::{
        entities::{
            graham::graham::GrahamScan, recalculation_course_angular::RecalculationCourseAngular,
        },
        eval::{
            impacts_high_waves::impacts_high_waves_ctx::ImpactsHighWavesCtx,
            period_excitement::period_excitement_ctx::PeriodExcitementCtx,
        },
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextRead, ContextReadRef, ContextWrite, InitialCtx},
};
///
/// Расчет [массива скоростей движения, при которых происходит явление последовательных ударов высоких волн](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
pub struct ImpactsHighWavesEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl ImpactsHighWavesEval {
    ///
    /// Новый экземпляр [ImpactsHighWavesEval]
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
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
impl Eval<(), EvalResult> for ImpactsHighWavesEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial = ContextReadRef::<InitialCtx>::read_ref(&ctx);
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let course_angle = voyage.course_angle;
                let period_excitement = ContextRead::<PeriodExcitementCtx>::read(&ctx)
                    .period_excitement;
                let course_angle_of_wave: Vec<f64> =
                    (1350..=2250).map(|x| x as f64 / 10.0).collect();
                let vessel_speed = Self::vessel_speed();
                let mut result = RecalculationCourseAngular::to_northeastern(
                    course_angle,
                    course_angle_of_wave
                        .iter()
                        .flat_map(|angle| {
                            vessel_speed
                                .iter()
                                .map(move |speed| {
                                    let speed = (speed
                                        / (angle * std::f64::consts::PI / 180.0).cos().abs())
                                        * period_excitement;
                                    (*angle, speed)
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>(),
                );
                result.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                ctx.write(ImpactsHighWavesCtx {
                    impacts_high_waves: GrahamScan::new(result).eval(),
                })
            }
            Err(err) => Err(err),
        }
    }
}
