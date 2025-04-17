use super::roll_amplitude_ctx::RollingAmplitudeCtx;
use crate::{
    ContextWrite, CtxResult,
    algorithm::context::context_access::{ContextRead, ContextReadRef},
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ship_model::model_link::{IModelLink, ModelLink},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет амплитуды качки судна
pub struct RollingAmplitudeEval {
    dbg: Dbg,
    value: Option<RollingAmplitudeCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl RollingAmplitudeEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "RollingAmplitudeEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for RollingAmplitudeEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let parameters: Parameters = ctx.read();
                let balance: BalanceCtx = ctx.read();
                let rolling_period: RollingPeriodCtx  = ctx.read();
                let volume = balance.volume;
                let length_wl = balance.length_wl;
                let breadth_wl = balance.breadth_wl;
                let mean_draught = balance.mean_draught;
                let keel_area = initial.ship_parameters.expect("RollingAmplitudeEval eval error: no ship_parameters").get("Keel area");
                let width = initial.ship_parameters.expect("RollingAmplitudeEval eval error: no ship_parameters").get("MouldedBreadth");
                // Коэффициент полноты судна
                let c_b = volume / (length_wl * breadth_wl * mean_draught);
                let k = if let Some(a_k) = keel_area {
                    self.k.value(a_k * 100. / (length_wl * width))?
                } else {
                    1.
                };
                let coefficient_k =
                Rc::new(Curve::new_linear(&initial.coefficient_k.expect("RollingAmplitudeEval eval error: no coefficient_k").data()).map_err(|e| {
                    Error::FromString(format!(
                        "Computer calculate_stability coefficient_k error: {e}"
                    ))
                })?);
                let multipler_x1 =
                    Curve::new_linear(&initial.multipler_x1.expect("RollingAmplitudeEval eval error: no multipler_x1").data()).map_err(|e| {
                        Error::FromString(format!(
                            "Computer calculate_stability multipler_x1 error: {e}"
                        ))
                    })?;
                let multipler_x2 = Curve::new_linear(&initial.multipler_x2.expect("RollingAmplitudeEval eval error: no multipler_x2").data())?;
                let multipler_s = 
                    Curve::new_linear(&initial.multipler_s.expect("RollingAmplitudeEval eval error: no multipler_s").get_area(&initial.ship.navigation_area)).map_err(
                        |e| {
                            Error::FromString(format!(
                                "Computer calculate_stability multipler_s error: {e}"
                            ))
                        },
                    )?;
                let coefficient_k_theta: Rc<dyn ICurve<f64>> = Rc::new(
                    Curve::new_linear(&initial.coefficient_k_theta.expect("RollingAmplitudeEval eval error: no coefficient_k_theta").data()).map_err(|e| {
                        Error::FromString(format!(
                            "Computer calculate_stability coefficient_k_theta error: {e}"
                        ))
                    })?,
                );

        let x_1 = self.x_1.value(width / mean_draught)?;
        let x_2 = self.x_2.value(c_b)?;
        let r = (0.73 + 0.6 * (self.metacentric_height.z_g_fix()? - mean_draught) / mean_draught).min(1.);
        let t = rolling_period.roll_period;
        let s = self.s.value(t)?;
        // (2.1.5.1)
        let res = 109. * k * x_1 * x_2 * (r * s).sqrt();
        log::trace!("\t RollingAmplitude volume:{} l_wl:{} b:{} b_wl:{} d:{} z_g_fix:{} c_b:{} k:{k} x_1:{x_1} x_2:{x_2} r:{r} t:{t} s:{s} angle:{res}",
            self.volume, self.l_wl, width, breadth_wl, mean_draught, self.metacentric_height.z_g_fix()?, c_b);
        Ok((t, res.round()))
                self.value = Some(result.clone());
                ctx.write(parameters)?;
                ctx.write(result)
            }
            CtxResult::Err(err) => CtxResult::Err(error.pass_with("Read context error", err)),
            CtxResult::None => CtxResult::None,
        }
    }
}
//
//
impl std::fmt::Debug for WindEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
