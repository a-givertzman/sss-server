use crate::algorithm::context::context_access::{ContextRead, ContextReadRef};
use crate::algorithm::entities::recalculation_course_angular::RecalculationCourseAngular;
use crate::algorithm::eval::zg_eval::Zg;
use crate::algorithm::eval::{
    ApparentFrequenciesCtx,
    ParametricResonantZoneCtx, 
    ParametricResonantZoneSpeedFilterCtx,
};
use crate::prelude::InitialCtx;
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
                let course_angle = ContextReadRef::<InitialCtx>::read_ref(&ctx).course_angle.unwrap();
                let ParametricResonantZoneCtx { left_side, right_side } = ContextRead::read(&ctx);
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
                    ParametricResonantZoneSpeedFilterCtx {
                        parametric_resonant_zone_speed_filter: result
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
