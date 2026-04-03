use crate::algorithm::eval::icing_coeff::ctx::IcingCoeffCtx;
use crate::{
    algorithm::{context::context_access::ContextReadRef, entities::icing_coeff::IcingCoeffType},
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Коэффициенты для расчета обледенения судна
pub struct IcingCoeffEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl IcingCoeffEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "IcingCoeffEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for IcingCoeffEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let icing = initial
                    .icing
                    .clone()
                    .ok_or(error.err("icing error: no data!"))?
                    .data();
                let icing_stab = IcingCoeffType::from_str(&voyage.icing_type)
                    .map_err(|err| error.pass_with("icing_stab", err))?;
                let icing_m_timber = *icing
                    .get("icing_m_timber")
                    .ok_or(error.err("icing_m_timber error: no data!"))?;
                let icing_m_v_full = *icing
                    .get("icing_m_v_full")
                    .ok_or(error.err("icing_m_v_full error: no data!"))?;
                let icing_m_v_half = *icing
                    .get("icing_m_v_half")
                    .ok_or(error.err("icing_m_v_half error: no data!"))?;
                let icing_m_h_full = *icing
                    .get("icing_m_h_full")
                    .ok_or(error.err("icing_m_h_full error: no data!"))?;
                let icing_m_h_half = *icing
                    .get("icing_m_h_half")
                    .ok_or(error.err("icing_m_h_half error: no data!"))?;
                let icing_coef_v_area_full = *icing
                    .get("icing_coef_v_area_full")
                    .ok_or(error.err("icing_coef_v_area_full error: no data!"))?;
                let icing_coef_v_area_half = *icing
                    .get("icing_coef_v_area_half")
                    .ok_or(error.err("icing_coef_v_area_half error: no data!"))?;
                let icing_coef_v_area_zero = *icing
                    .get("icing_coef_v_area_zero")
                    .ok_or(error.err("icing_coef_v_area_zero error: no data!"))?;
                let icing_coef_v_moment_full = *icing
                    .get("icing_coef_v_moment_full")
                    .ok_or(error.err("icing_coef_v_moment_full error: no data!"))?;
                let icing_coef_v_moment_half = *icing
                    .get("icing_coef_v_moment_half")
                    .ok_or(error.err("icing_coef_v_moment_half error: no data!"))?;
                let icing_coef_v_moment_zero = *icing
                    .get("icing_coef_v_moment_zero")
                    .ok_or(error.err("icing_coef_v_moment_zero error: no data!"))?;
                let mass_desc_h = match icing_stab {
                    IcingCoeffType::Full => icing_m_h_full,
                    IcingCoeffType::Half => icing_m_h_half,
                    _ => 0.,
                };
                let mass_timber_h = match icing_stab {
                    IcingCoeffType::Full | IcingCoeffType::Half => icing_m_timber,
                    _ => 0.,
                };
                let mass_v = match icing_stab {
                    IcingCoeffType::Full => icing_m_v_full,
                    IcingCoeffType::Half => icing_m_v_half,
                    _ => 0.,
                };
                let coef_v_area = match icing_stab {
                    IcingCoeffType::Full => icing_coef_v_area_full,
                    IcingCoeffType::Half => icing_coef_v_area_half,
                    _ => icing_coef_v_area_zero,
                };
                let coef_v_ds_area = icing_coef_v_area_zero;
                let coef_v_moment = match icing_stab {
                    IcingCoeffType::Full => icing_coef_v_moment_full,
                    IcingCoeffType::Half => icing_coef_v_moment_half,
                    _ => icing_coef_v_moment_zero,
                };
                let is_some = matches!(icing_stab, IcingCoeffType::Full | IcingCoeffType::Half);
                let result = IcingCoeffCtx {
                    w_desc: mass_desc_h,
                    w_timber: mass_timber_h,
                    w_ice_v: mass_v,
                    coef_v_area,
                    coef_v_ds_area,
                    coef_v_moment,
                    is_some,
                };
                log::info!(
                    "IcingCoeff mass_desc_h:{:.3} mass_timber_h:{:.3} mass_v:{:.3} coef_v_area:{:.3} coef_v_ds_area:{:.3} coef_v_moment:{:.3} is_some:{})",
                    result.w_desc,
                    result.w_timber,
                    result.w_ice_v,
                    result.coef_v_area,
                    result.coef_v_ds_area,
                    result.coef_v_moment,
                    result.is_some,
                );
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for IcingCoeffEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcingCoeffEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
