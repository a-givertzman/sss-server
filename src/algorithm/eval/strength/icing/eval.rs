use crate::algorithm::context::context_access::{ContextRead, ContextReadRef};
use crate::algorithm::entities::ship_model::ship_model::ShipModel;
use crate::algorithm::eval::strength::IcingStrCtx;
use crate::algorithm::eval::{IcingCoeffCtx, UnitAreaCtx};
use crate::algorithm::eval::icing_timber::ctx::IcingTimberCtx;
use crate::kernel::types::{Arc, RwLock};
use crate::{
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Учет обледенения судна для расчета прочности
pub struct IcingStrEval {
    dbg: Dbg,
    model: Arc<RwLock<ShipModel>>,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl IcingStrEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        model: Arc<RwLock<ShipModel>>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "IcingStrEval");
        Self {
            dbg,
            model,
            ctx: Box::new(ctx),
        }
    }
    //
}
//
impl Eval<(), EvalResult> for IcingStrEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let icing_stab: IcingCoeffCtx = ctx.read();
                let bounds = match initial.bounds.clone() {
                    Some(data) => data,
                    None => return Err(error.err("Read bounds error: no data!")),
                };
                let icing_coeff: IcingCoeffCtx = ctx.read();
                let unit_area: UnitAreaCtx = ctx.read();
                let icing_timber: IcingTimberCtx = ctx.read();  
                let (model_area_v, model_area_h) = match self.model.read().strength_area() {
                    Ok(areas) => (areas.v, areas.h),
                    Err(err) => return Err(error.pass_with("model.strength_area", err)),
                };
                assert!(unit_area.distr_v.len() == bounds.len_qnt());
                assert!(icing_timber.area_icing_distr.len() == bounds.len_qnt());
                assert!(icing_timber.area_no_icing_distr.len() == bounds.len_qnt());
                assert!(model_area_v.len() == bounds.len_qnt());
                let mut mass_values = Vec::new();
                for (i, _) in bounds.iter().enumerate() {
                    // Масса льда на площади парусности          
                    let p_ice_v = (model_area_v[i] + unit_area.distr_v[i])*icing_stab.w_ice_v;
                    // Масса льда на горизонтальной проекции открытых палуб
                    let p_ice_hdeck = icing_stab.w_desc*(model_area_h[i] - icing_timber.area_no_icing_distr[i]);           
                    // Разница в массе льда от обледенения палубного груза - леса
                    let delta_p_ice_hdeck = icing_timber.area_icing_distr[i] * (icing_stab.w_timber - icing_stab.w_desc);
                    // Суммарная масса льда на горизонтальной проекции для каждой составляющей открытых палуб
                    let p_ice_h = p_ice_hdeck + delta_p_ice_hdeck;
                    // Суммарная масса льда 
                    let p_ice = p_ice_h + p_ice_v;
                    mass_values.push(p_ice);
                }
                let result = IcingStrCtx {
                    mass_values,
                };
                log::info!(
                    "IcingStrCtx mass:{:.3}",
                    result.mass_values.iter().sum::<f64>(),
                );
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for IcingStrEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcingStrEval").field("dbg", &self.dbg).finish()
    }
}
