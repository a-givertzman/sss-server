use crate::algorithm::entities::recalculation_course_angular::RecalculationCourseAngular;
use crate::algorithm::eval::seakeeping::apparent_frequencies::apparent_frequencies_ctx::ApparentFrequenciesCtx;
use crate::algorithm::eval::seakeeping::main_resonant_zone::main_resonant_zone_ctx::MainResonantZoneCtx;
use crate::algorithm::eval::seakeeping::main_resonant_zone_speed_filter::main_resonant_zone_speed_filter_ctx::MainResonantZoneSpeedFilterCtx;
use crate::prelude::{ContextRead, ContextReadRef, ContextWrite, InitialCtx};
use crate::kernel::{
        Eval, 
        types::eval_result::EvalResult
    };
use sal_core::{
    dbg::Dbg, 
    error::Error
};
///
/// Расчет расчета массива скоростей хода, при которых 
/// [кажущаяся частота волнения](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#условия-возникновения-опасных-явлений) 
/// находится в диапазоне основного резонанса бортовой качки](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#условия-возникновения-опасных-явлений)
pub struct MainResonantZoneSpeedFilterEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl MainResonantZoneSpeedFilterEval {
    ///
    /// Новый экземпляр [MainResonantZoneSpeedFilterEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,) -> Self {
        let dbg = Dbg::new(parent, "MainResonantZoneSpeedFilterEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for MainResonantZoneSpeedFilterEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let course_angle = ContextReadRef::<InitialCtx>::read_ref(&ctx).course_angle.unwrap();
                let MainResonantZoneCtx { left_side, right_side } = ContextRead::read(&ctx);
                let result: Vec<(f64, f64)> = RecalculationCourseAngular::to_northeastern(
                    course_angle, 
                    ContextRead::<ApparentFrequenciesCtx>::read(&ctx)
                        .apparent_frequencies
                        .iter()
                        .filter(
                        |(_, _, freq)| left_side <= *freq && *freq <= right_side
                        ).map(|(angle, speed, _)| (*angle, *speed))
                    .collect()
                );
                ctx.write(
                    MainResonantZoneSpeedFilterCtx {
                        main_resonant_zone_speed_filter: result
                    }
                )
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for MainResonantZoneSpeedFilterEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MainResonantZoneSpeedFilterEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
