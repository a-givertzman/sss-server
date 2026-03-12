use sal_core::{dbg::Dbg, error::Error};

use crate::{
    algorithm::eval::seakeeping::{
        entities::recalculation_course_angular::RecalculationCourseAngular,
        eval::move_broching_filter::move_broching_filter_ctx::MoveBrochingFilterCtx,
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextReadRef, ContextWrite, InitialCtx},
};
///
/// Расчет [массива скоростей хода, уз, при которых возникает движение судна на гребне волны и брочинг](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
pub struct MoveBrochingFilterEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl MoveBrochingFilterEval {
    ///
    /// Новый экземпляр [MoveBrochingFilterEval]
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "MoveBrochingFilterEval");
        Self { dbg, ctx: Box::new(ctx) }
    }
}
//
//
impl Eval<(), EvalResult> for MoveBrochingFilterEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial = ContextReadRef::<InitialCtx>::read_ref(&ctx);
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .ok_or(error.err("no ship_parameters"))?;
                let ship_length_lbp = *ship_parameters
                    .get("LBP")
                    .ok_or(error.err("No LBP in ship_parameters"))?;            
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let course_angle = voyage.course_angle;
                let course_angle_of_wave: Vec<f64> =
                    (1350..=2250).map(|x| x as f64 / 10.0).collect();
                let vmax = voyage.operational_speed;
                let vessel_speeds: Vec<f64> = (0..=(vmax.ceil() as isize * 10))
                    .map(|x| x as f64 / 10.0)
                    .collect();
                let mut result: Vec<(f64, f64)> = Vec::new();
                for speed in vessel_speeds.iter() {
                    for angle in course_angle_of_wave.iter() {
                        let formula =
                            1.8 * ship_length_lbp.sqrt() / (180.0 - angle).to_radians().cos();
                        if speed >= &formula {
                            result.push((*angle, *speed));
                        }
                    }
                }
                let mut result = RecalculationCourseAngular::to_northeastern(course_angle, result);
                result.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                ctx.write(MoveBrochingFilterCtx {
                    move_broching_filter: result,
                })
            }
            Err(err) => Err(err),
        }
    }
}
