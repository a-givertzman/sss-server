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
                let windage: WindageCtx = ctx.read();
        // Коэффициент полноты судна
        let c_b = self.volume / (self.l_wl * self.b_wl * self.d);
        let k = if let Some(a_k) = self.a_k {
            self.k.value(a_k * 100. / (self.l_wl * self.b))?
        } else {
            1.
        };
        let x_1 = self.x_1.value(self.b / self.d)?;
        let x_2 = self.x_2.value(c_b)?;
        let r = (0.73 + 0.6 * (self.metacentric_height.z_g_fix()? - self.d) / self.d).min(1.);
        let t = self.t.calculate()?;
        let s = self.s.value(t)?;
        // (2.1.5.1)
        let res = 109. * k * x_1 * x_2 * (r * s).sqrt();
        log::trace!("\t RollingAmplitude volume:{} l_wl:{} b:{} b_wl:{} d:{} z_g_fix:{} c_b:{} k:{k} x_1:{x_1} x_2:{x_2} r:{r} t:{t} s:{s} angle:{res}",
            self.volume, self.l_wl, self.b, self.b_wl, self.d, self.metacentric_height.z_g_fix()?, c_b);
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
