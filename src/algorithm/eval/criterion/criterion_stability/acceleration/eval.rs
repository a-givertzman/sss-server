use crate::algorithm::eval::criterion::*;
use crate::algorithm::eval::stability::*;
use crate::algorithm::context::context_access::ContextParamsRead;
use crate::algorithm::eval::parameters::ParameterID;
use crate::algorithm::eval::zg::Zg;
use crate::{
    prelude::*,
    kernel::{Eval, types::eval_result::EvalResult},
};
use sal_3dlib_core::math::*;
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия ускорения 𝐾∗
pub struct AccelerationEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl AccelerationEval {
    //
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "AccelerationEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for AccelerationEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .ok_or(error.err("ship_parameters error: no data!"))?;
                let b = *ship_parameters
                    .get("MouldedBreadth")
                    .ok_or(error.err("breadth error: no data!"))?;
                let d = ctx.read_params(ParameterID::DraughtMean);
                let metacentric_height: MetacentricHeightCtx = ctx.read();
                let rolling_amplitude: RollingAmplitudeCtx = ctx.read();
                let rolling_period: RollingPeriodCtx = ctx.read();
                let h_trans_0 = metacentric_height.h_trans_0;
                let k_theta_data = match &initial.coefficient_k_theta {
                    Some(array) => array.data(),
                    None => {
                        let error = error.err("coefficient_k_theta error: no data!");
                        log::error!("{}", error);
                        let result = CriterionData::new_error(
                            CriterionID::Acceleration,
                            "Ошибка расчета критерия ускорения 𝐾∗".to_owned() + &error.to_string(),
                        );
                        let result = AccelerationCtx { data: result };
                        return ctx.write(result);
                    }
                };
                let curve = match Curve::new_linear(&k_theta_data) {
                    Ok(curve) => curve,
                    Err(err) => {
                        let error = error.pass_with("Curve::new_linear", err);
                        log::error!("{}", error);
                        let result = CriterionData::new_error(
                            CriterionID::Acceleration,
                            "Ошибка расчета критерия ускорения 𝐾∗".to_owned() + &error.to_string(),
                        );
                        let result = AccelerationCtx { data: result };
                        return ctx.write(result);
                    }
                };
                let k_theta = match curve.value(b / d) {
                    Ok(curve) => curve,
                    Err(err) => {
                        let error = error.pass_with("k_theta curve.value", err);
                        log::error!("{}", error);
                        let result = CriterionData::new_error(
                            CriterionID::Acceleration,
                            "Ошибка расчета критерия ускорения 𝐾∗".to_owned() + &error.to_string(),
                        );
                        let result = AccelerationCtx { data: result };
                        return ctx.write(result);
                    }
                };
                let c = rolling_period.c;
                let theta_1_r = rolling_amplitude.amplitude;
                let a = 0.0105 * h_trans_0 / (c * c * b) * k_theta * theta_1_r;
                let k = 0.3 / a; // >= 1;
                let result = AccelerationCtx {
                    data: CriterionData::new_result(CriterionID::Acceleration, k, 1.),
                };
                log::info!(
                    "Criterion Acceleration c:{:.3} theta_1_r:{:.3} a:{:.3} k:{:.3}",
                    c, theta_1_r, a, k
                ); 
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for AccelerationEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccelerationEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
