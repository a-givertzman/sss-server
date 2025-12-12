use sal_core::{dbg::Dbg, error::Error};
use crate::algorithm::eval::IcingStabCtx;
use crate::{
    algorithm::{context::context_access::ContextReadRef, entities::icing_stab::IcingStabType},
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{InitialCtx, ContextWrite},
};

///
/// Коэффициенты для расчета обледенения судна
pub struct IcingStabEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl IcingStabEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "IcingStabEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for IcingStabEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let voyage = initial.voyage.as_ref().ok_or(error.err("voyage error: no data!"))?; 
                let icing = initial.icing.clone().ok_or(error.err("icing error: no data!"))?.data(); 
                let icing_stab = IcingStabType::from_str(&voyage.icing_type)
                    .map_err(|err| error.pass_with("icing_stab", err))?;
                let icing_m_timber = *icing.get("icing_m_timber").ok_or(error.err("icing_m_timber error: no data!"))?; 
                let icing_m_v_full = *icing.get("icing_m_v_full").ok_or(error.err("icing_m_v_full error: no data!"))?; 
                let icing_m_v_half = *icing.get("icing_m_v_half").ok_or(error.err("icing_m_v_half error: no data!"))?;
                let icing_m_h_full = *icing.get("icing_m_h_full").ok_or(error.err("icing_m_h_full error: no data!"))?;
                let icing_m_h_half = *icing.get("icing_m_h_half").ok_or(error.err("icing_m_h_half error: no data!"))?;
                let icing_coef_v_area_full = *icing.get("icing_coef_v_area_full").ok_or(error.err("icing_coef_v_area_full error: no data!"))?;
                let icing_coef_v_area_half = *icing.get("icing_coef_v_area_half").ok_or(error.err("icing_coef_v_area_half error: no data!"))?;
                let icing_coef_v_area_zero = *icing.get("icing_coef_v_area_zero").ok_or(error.err("icing_coef_v_area_zero error: no data!"))?;
                let icing_coef_v_moment_full = *icing.get("icing_coef_v_moment_full").ok_or(error.err("icing_coef_v_moment_full error: no data!"))?;
                let icing_coef_v_moment_half = *icing.get("icing_coef_v_moment_half").ok_or(error.err("icing_coef_v_moment_half error: no data!"))?;
                let icing_coef_v_moment_zero = *icing.get("icing_coef_v_moment_zero").ok_or(error.err("icing_coef_v_moment_zero error: no data!"))?;
                let mass_desc_h = match icing_stab {
                    IcingStabType::Full => icing_m_h_full,
                    IcingStabType::Half => icing_m_h_half,
                    _ => 0.,
                };
                let mass_timber_h = match icing_stab {
                    IcingStabType::Full | IcingStabType::Half => icing_m_timber,
                    _ => 0.,
                };
                let mass_v = match icing_stab {
                    IcingStabType::Full => icing_m_v_full,
                    IcingStabType::Half => icing_m_v_half,
                    _ => 0.,
                };
                let coef_v_area = match icing_stab {
                    IcingStabType::Full => icing_coef_v_area_full,
                    IcingStabType::Half => icing_coef_v_area_half,
                    _ => icing_coef_v_area_zero,
                };
                let coef_v_ds_area = icing_coef_v_area_zero;
                let coef_v_moment = match icing_stab {
                    IcingStabType::Full => icing_coef_v_moment_full,
                    IcingStabType::Half => icing_coef_v_moment_half,
                    _ => icing_coef_v_moment_zero,
                };
                let is_some =
                    matches!(icing_stab, IcingStabType::Full | IcingStabType::Half);
                let result = IcingStabCtx {
                    mass_desc_h,
                    mass_timber_h,
                    mass_v,
                    coef_v_area,
                    coef_v_ds_area,
                    coef_v_moment,
                    is_some,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for IcingStabEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcingStabEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
