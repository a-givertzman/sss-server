use sal_core::{dbg::Dbg, error::Error};

use crate::{
    algorithm::eval::seakeeping::{
        entities::{
            clusterization::Clusterization, graham::graham::GrahamScan,
            recalculation_course_angular::RecalculationCourseAngular,
        },
        eval::{
            apparent_frequencies::apparent_frequencies_ctx::ApparentFrequenciesCtx,
            main_resonant_zone::main_resonant_zone_ctx::MainResonantZoneCtx,
            main_resonant_zone_speed_filter::main_resonant_zone_speed_filter_ctx::MainResonantZoneSpeedFilterCtx,
        },
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextRead, ContextReadRef, ContextWrite, InitialCtx},
};
//
//
pub struct MainResonantZoneSpeedFilterEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl MainResonantZoneSpeedFilterEval {
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "MainResonantZoneSpeedFilterEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
impl Eval<(), EvalResult> for MainResonantZoneSpeedFilterEval {
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
                let MainResonantZoneCtx {
                    left_side,
                    right_side,
                } = ContextRead::read(&ctx);
                let main_zone: Vec<(f64, f64)> = RecalculationCourseAngular::to_northeastern(
                    course_angle,
                    ContextRead::<ApparentFrequenciesCtx>::read(&ctx)
                        .apparent_frequencies
                        .iter()
                        .filter(|(_, _, freq)| left_side <= *freq && *freq <= right_side)
                        .map(|(angle, speed, _)| (*angle, *speed))
                        .collect(),
                );
                match Clusterization::new(main_zone.clone(), course_angle).eval() {
                    Some(clusters) => {
                        let mut result: Vec<Vec<(f64, f64)>> = Vec::new();
                        for cluster in clusters {
                            let contour = GrahamScan::new(cluster).eval();
                            result.push(contour);
                        }
                        ctx.write(MainResonantZoneSpeedFilterCtx {
                            main_resonant_zone_speed_filter: result,
                        })
                    }
                    None => {
                        let mut result: Vec<Vec<(f64, f64)>> = Vec::new();
                        result.push(main_zone.clone());
                        ctx.write(MainResonantZoneSpeedFilterCtx {
                            main_resonant_zone_speed_filter: result,
                        })
                    }
                }
            }
            Err(err) => Err(err),
        }
    }
}
