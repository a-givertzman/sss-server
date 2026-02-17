use crate::algorithm::context::context_access::{
    ContextRead, 
    ContextReadRef
};
use crate::algorithm::entities::recalculation_course_angular::RecalculationCourseAngular;
use crate::algorithm::eval::move_broching_filter::move_broching_filter_ctx::MoveBrochingFilterCtx;
use crate::algorithm::eval::vessel_max_speed::vessel_max_speed_ctx::VesselMaxSpeedCtx;
use crate::algorithm::eval::zg_eval::Zg;
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
/// Расчет [массива скоростей хода, уз, при которых возникает движение судна на гребне волны и брочинг](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
pub struct MoveBrochingFilterEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl MoveBrochingFilterEval {
    ///
    /// Новый экземпляр [MoveBrochingFilterEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "MoveBrochingFilterEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for MoveBrochingFilterEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let course_angle = ContextReadRef::<InitialCtx>::read_ref(&ctx).course_angle.unwrap();
                let initial: &InitialCtx = ctx.read_ref();
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .unwrap();
                let ship_length_lbp = *ship_parameters
                    .get("LBP")
                    .ok_or(error.err("No LBP in ship_parameters"))?;
                let course_angle_of_wave: Vec<f64> = (1350..=2250).map(|x| x as f64 / 10.0).collect();
                let vmax = ContextRead::<VesselMaxSpeedCtx>::read(&ctx).vmax.clone();
                let vessel_speeds: Vec<f64> = (0..=(vmax.ceil() as isize * 10)).map(|x| x as f64 / 10.0).collect();
                let mut result = RecalculationCourseAngular::to_northeastern(
                    course_angle, 
                    course_angle_of_wave.iter().flat_map(|&a| {
                        let formula = 1.8 * ship_length_lbp.sqrt() / ((180.0 - a) * std::f64::consts::PI / 180.0).cos();
                        let mut res = vessel_speeds
                        .iter()
                        .filter(|&&v| v >= formula)
                        .map(|&v| {
                            (a, v)
                        })
                        .collect::<Vec<_>>();
                        if let Some(last) = res.last_mut() {
                            *last = (a, vmax);
                        }
                        res
                    }).collect::<Vec<_>>()
                );
                result.sort_by(
                    |a, b| 
                    a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
                );
                ctx.write(
                    MoveBrochingFilterCtx {
                        move_broching_filter: result
                    }
                )
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for MoveBrochingFilterEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MoveBrochingFilterEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
