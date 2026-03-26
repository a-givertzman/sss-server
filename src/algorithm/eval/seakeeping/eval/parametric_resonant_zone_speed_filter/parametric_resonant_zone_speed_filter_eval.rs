use sal_core::{dbg::Dbg, error::Error};

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
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
impl ParametricResonantZoneSpeedFilterEval {
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ParametricResonantZoneSpeedFilterEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
impl Eval<(), EvalResult> for ParametricResonantZoneSpeedFilterEval {
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