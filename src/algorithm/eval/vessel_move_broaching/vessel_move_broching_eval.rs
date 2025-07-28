use crate::algorithm::context::context_access::{ContextRead, ContextReadRef};
use crate::algorithm::eval::zg_eval::Zg;
use crate::algorithm::eval::{
    VesselMoveBroachingCtx, 
    VesselSpeedFilterCtx
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
/// Расчет [массива скоростей хода, уз, при которых возникает движение судна на гребне волны и брочинг](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
pub struct VesselMoveBroachingEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl VesselMoveBroachingEval {
    ///
    /// Новый экземпляр [VesselMoveBroachingEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "VesselMoveBroachingEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for VesselMoveBroachingEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .unwrap();
                let ship_length_lbp = *ship_parameters
                    .get("LBP")
                    .ok_or(error.err("No LBP in ship_parameters"))?;
                let vessel_speed_filter = ContextRead::<VesselSpeedFilterCtx>::read(&ctx).vessel_speed_filter.clone();
                let mut result: Vec<(f64, f64)> = Vec::new();
                let course_angle_of_wave: Vec<f64> = (135..=2250).map(|x| x as f64 / 10.0).collect();
                for speed in vessel_speed_filter {
                    for angle in &course_angle_of_wave {
                        let formula = 1.8 * ship_length_lbp.sqrt() / (180.0 - angle).cos();
                        if speed >= formula {
                            result.push(
                                (*angle, speed)
                            );
                        }
                    }
                }
                ctx.write(
                    VesselMoveBroachingCtx {
                        vessel_move_broaching: result,
                    }
                )
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for VesselMoveBroachingEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VesselMoveBroachingEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
