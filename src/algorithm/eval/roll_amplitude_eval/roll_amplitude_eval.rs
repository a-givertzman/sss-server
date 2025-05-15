use super::roll_amplitude_ctx::RollingAmplitudeCtx;
use crate::algorithm::context::context_access::ContextParamsRead;
use crate::algorithm::entities::math::curve::*;
use crate::algorithm::eval::zg_eval::Zg;
use crate::{
    ContextWrite,
    algorithm::{
        context::context_access::{ContextParamsWrite, ContextRead, ContextReadRef},
        eval::{BalanceCtx, MetacentricHeightCtx, RollingPeriodCtx, parameters::ParameterID},
    },
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет амплитуды качки судна
pub struct RollingAmplitudeEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl RollingAmplitudeEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "RollingAmplitudeEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for RollingAmplitudeEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(mut ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let balance: BalanceCtx = ctx.read();
                let metacentric_height: MetacentricHeightCtx = ctx.read();
                let rolling_period: RollingPeriodCtx = ctx.read();
                let volume = balance.volume;
                let length_wl = balance.length_wl;
                let breadth_wl = balance.breadth_wl;
                let mean_draught = ctx.read_params(ParameterID::DraughtMean);
                let navigation_area = initial.navigation_area.unwrap();
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .ok_or(error.err("no ship_parameter"))?;
                let keel_area = ship_parameters.get("Keel area");
                let width = ship_parameters.get("MouldedBreadth")
                    .ok_or(error.err("No MouldedBreadth in ship_parameters"))?;
                let coefficient_k = Curve::new_linear(
                    &initial
                        .coefficient_k
                        .clone()
                        .ok_or(error.err("No coefficient_k in initial"))?,
                )
                .map_err(|e| error.pass_with("coefficient_k", e))?;
                let multipler_x1 = Curve::new_linear(
                    &initial
                        .multipler_x1
                        .clone()
                        .ok_or(error.err("No multipler_x1 in initial"))?,
                )
                .map_err(|e| error.pass_with("multipler_x1", e))?;
                let multipler_x2 = Curve::new_linear(
                    &initial
                        .multipler_x2
                        .clone()
                        .ok_or(error.err("No multipler_x2 in initial"))?,
                )
                .map_err(|e| error.pass_with("multipler_x2", e))?;
                let multipler_s = Curve::new_linear(
                    &initial
                        .multipler_s
                        .clone()
                        .ok_or(error.err("No multipler_s in initial"))?
                        .get_area(&navigation_area),
                )
                .map_err(|e| error.pass_with("multipler_s Curve::new_linear", e))?;
                // Коэффициент полноты судна
                let c_b = volume / (length_wl * breadth_wl * mean_draught);
                let k = if let Some(a_k) = keel_area {
                    coefficient_k.value(a_k * 100. / (length_wl * width))?
                } else {
                    1.
                };
                let x_1 = multipler_x1.value(width / mean_draught)?;
                let x_2 = multipler_x2.value(c_b)?;
                let r = (0.73 + 0.6 * (metacentric_height.z_g_fix - mean_draught) / mean_draught)
                    .min(1.);
                let t = rolling_period.roll_period;
                let s = multipler_s.value(t)?;
                let amplitude = 109. * k * x_1 * x_2 * (r * s).sqrt();
                log::trace!(
                    "\t RollingAmplitude volume:{volume} l_wl:{length_wl} b:{width} b_wl:{breadth_wl} d:{mean_draught} z_g_fix:{} c_b:{c_b} k:{k} x_1:{x_1} x_2:{x_2} r:{r} t:{t} s:{s} a:{amplitude}",
                    metacentric_height.z_g_fix
                );
                let amplitude = amplitude.round();
                let result = RollingAmplitudeCtx { amplitude };
                ctx.write_params(ParameterID::RollAmplitude, amplitude);
                ctx.write_params(ParameterID::RollPeriod, t);
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for RollingAmplitudeEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RollingAmplitudeEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
