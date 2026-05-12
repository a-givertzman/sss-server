use sal_core::{dbg::Dbg, error::Error};

use crate::{
    algorithm::eval::seakeeping::eval::{
        apparent_frequencies::apparent_frequencies_ctx::ApparentFrequenciesCtx,
        period_excitement::period_excitement_ctx::PeriodExcitementCtx,
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextRead, ContextReadRef, ContextWrite, InitialCtx},
};
///
/// Расчет массива [кажущихся частот волнения](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
pub struct ApparentFrequenciesEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl ApparentFrequenciesEval {
    ///
    /// Новый экземпляр [ApparentFrequenciesEval]
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "ApparentFrequenciesEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for ApparentFrequenciesEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial = <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx);
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let vmax = voyage.operational_speed;
                let period_excitement = ContextRead::<PeriodExcitementCtx>::read(&ctx)
                    .period_excitement;
                let course_angle_of_wave: Vec<f64> = (0..=3600).map(|x| x as f64 / 10.0).collect();
                let vessel_speeds: Vec<f64> = (0..=(vmax.ceil() as isize * 10))
                    .map(|x| x as f64 / 10.0)
                    .collect();
                let result: Vec<(f64, f64, f64)> = course_angle_of_wave
                    .iter()
                    .flat_map(|&angle| {
                        vessel_speeds
                            .iter()
                            .map(move |&speed| {
                                let apparent_frequency = 2.0
                                    * std::f64::consts::PI
                                    * (3.0 * period_excitement
                                        + speed * (angle * std::f64::consts::PI / 180.0).cos())
                                    .abs()
                                    / (3.0 * period_excitement.powf(2.0));
                                (angle, speed, apparent_frequency)
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect();
                ctx.write(ApparentFrequenciesCtx {
                    apparent_frequencies: result,
                })
            }
            Err(err) => Err(err),
        }
    }
}
