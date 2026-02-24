use crate::{
    algorithm::eval::seakeeping::{
        entities::{
            clusterization::Clusterization,
            graham::graham::GrahamScan,
            recalculation_course_angular::RecalculationCourseAngular
            },
        eval::{
            apparent_frequencies::apparent_frequencies_ctx::ApparentFrequenciesCtx,
            main_resonant_zone::main_resonant_zone_ctx::MainResonantZoneCtx,
            main_resonant_zone_speed_filter::main_resonant_zone_speed_filter_ctx::MainResonantZoneSpeedFilterCtx
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
pub struct MainResonantZoneSpeedFilterEval {
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
impl MainResonantZoneSpeedFilterEval {
    pub fn new(ctx: impl Eval<(), EvalResult> + Send + Sync + 'static) -> Self {
        Self {
            ctx: Box::new(ctx),
        }
    }
}
impl Eval<(), EvalResult> for MainResonantZoneSpeedFilterEval {
    fn eval(&self, _: ()) -> EvalResult {
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let course_angle = ContextReadRef::<InitialCtx>::read_ref(&ctx).course_angle.expect("No `course angle` in initial data");
                let MainResonantZoneCtx { left_side, right_side } = ContextRead::read(&ctx);
                let main_zone: Vec<(f64, f64)> = RecalculationCourseAngular::to_northeastern(
                    course_angle, 
                    ContextRead::<ApparentFrequenciesCtx>::read(&ctx)
                        .apparent_frequencies
                        .iter()
                        .filter(|(_, _, freq)| left_side <= *freq && *freq <= right_side)
                        .map(|(angle, speed, _)| (*angle, *speed))
                        .collect()
                );
                match Clusterization::new(main_zone.clone(), course_angle).eval() {
                    Some(clusters) => {
                        let mut result: Vec<Vec<(f64, f64)>> = Vec::new();
                        for cluster in clusters {
                            let contour = GrahamScan::new(cluster).eval();
                            result.push(contour);
                        }
                        return ctx.write(
                            MainResonantZoneSpeedFilterCtx {
                                main_resonant_zone_speed_filter: result,
                            }
                        )
                    },
                    None => {
                        let mut result: Vec<Vec<(f64, f64)>> = Vec::new();
                        result.push(main_zone.clone());
                        return ctx.write(
                            MainResonantZoneSpeedFilterCtx {
                                main_resonant_zone_speed_filter: result,
                            }
                        )
                    },
                }
            }
            Err(err) => Err(err),
        }
    }
}