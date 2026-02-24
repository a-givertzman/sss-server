use crate::{
    algorithm::eval::seakeeping::{
        entities::{
            clusterization::Clusterization, 
            graham::graham::GrahamScan,
            recalculation_course_angular::RecalculationCourseAngular
        }, 
        eval::{
            apparent_frequencies::apparent_frequencies_ctx::ApparentFrequenciesCtx, 
            parametric_resonant_zone::parametric_resonant_zone_ctx::ParametricResonantZoneCtx, 
            parametric_resonant_zone_speed_filter::parametric_resonant_zone_speed_filter_ctx::ParametricResonantZoneSpeedFilterCtx
        }
    }, 
    kernel::{
        Eval, 
        types::eval_result::EvalResult
    }, 
    prelude::{
        ContextRead, ContextReadRef, ContextWrite, InitialCtx
    }
};
pub struct ParametricResonantZoneSpeedFilterEval {
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
impl ParametricResonantZoneSpeedFilterEval {
    pub fn new(ctx: impl Eval<(), EvalResult> + Send + Sync + 'static) -> Self {
        Self {
            ctx: Box::new(ctx),
        }
    }
}
impl Eval<(), EvalResult> for ParametricResonantZoneSpeedFilterEval {
    fn eval(&self, _: ()) -> EvalResult {
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let course_angle = ContextReadRef::<InitialCtx>::read_ref(&ctx).course_angle.expect("No `course angle` in initial data");
                let ParametricResonantZoneCtx { left_side, right_side } = ContextRead::read(&ctx);
                let param_zone: Vec<(f64, f64)> = RecalculationCourseAngular::to_northeastern(
                    course_angle, 
                    ContextRead::<ApparentFrequenciesCtx>::read(&ctx)
                        .apparent_frequencies
                        .iter()
                        .filter(|(_, _, freq)| left_side <= *freq && *freq <= right_side)
                        .map(|(angle, speed, _)| (*angle, *speed))
                        .collect()
                );
                match Clusterization::new(param_zone.clone(), course_angle).eval() {
                    Some(clusters) => {
                        let mut result: Vec<Vec<(f64, f64)>> = Vec::new();
                        for cluster in clusters {
                            let contour = GrahamScan::new(cluster).eval();
                            result.push(contour);
                        }
                        return ctx.write(
                            ParametricResonantZoneSpeedFilterCtx {
                                parametric_resonant_zone_speed_filter: result,
                            }
                        )
                    },
                    None => {
                        let mut result: Vec<Vec<(f64, f64)>> = Vec::new();
                        result.push(GrahamScan::new(param_zone).eval());
                        return ctx.write(
                            ParametricResonantZoneSpeedFilterCtx {
                                parametric_resonant_zone_speed_filter: result,
                            }
                        )
                    },
                }
            }
            Err(err) => Err(err),
        }
    }
}