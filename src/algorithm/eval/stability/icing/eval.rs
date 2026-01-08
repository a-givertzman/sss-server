use crate::algorithm::context::context_access::{ContextRead, ContextReadRef};
use crate::algorithm::entities::Moment;
use crate::algorithm::entities::ship_model::ship_model::ShipModel;
use crate::algorithm::eval::icing_timber::ctx::IcingTimberCtx;
use crate::algorithm::eval::{IcingCoeffCtx, UnitAreaCtx};
use crate::algorithm::eval::stability::IcingStabCtx;
use crate::{
    kernel::{Eval, types::{Arc, RwLock, eval_result::EvalResult}},
    prelude::{ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Учет обледенения судна [https://github.com/a-givertzman/sss/blob/master/design/algorithm/part02_mass/chapter02_icing.md]
pub struct IcingStabEval {
    dbg: Dbg,
    model: Arc<RwLock<ShipModel>>,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl IcingStabEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        model: Arc<RwLock<ShipModel>>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "IcingStabEval");
        Self {
            dbg,
            model,
            ctx: Box::new(ctx),
        }
    }
    //
}
//
impl Eval<(), EvalResult> for IcingStabEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let icing_stab: IcingCoeffCtx = ctx.read();
                let icing_timber: IcingTimberCtx = ctx.read();
                let unit_area: UnitAreaCtx = ctx.read();
                let (av_cs_dmin, mv_cs_dmin) = match self.model.read().static_area_v() {
                    Ok((area, moment)) => (area, moment),
                    Err(err) => return Err(error.pass_with("model.static_area_h", err)),
                };                
                let (a_ice_hdeck, a_ice_shift) = match self.model.read().static_area_h() {
                    Ok((area, moment)) => (area, moment),
                    Err(err) => return Err(error.pass_with("model.static_area_h", err)),
                };  
                // Масса льда на площади парусности          
                let p_ice_v = (av_cs_dmin + unit_area.av_dc)*icing_stab.w_ice_v;
                // Момент льда на площади парусности    
                let m_ice_v = (mv_cs_dmin + unit_area.mv_dc).scale(icing_stab.w_ice_v);
                // Масса льда на горизонтальной проекции открытых палуб
                let p_ice_hdeck = icing_stab.w_desc*(a_ice_hdeck - icing_timber.area_no_icing);
                // Момент льда на горизонтальной проекции открытых палуб
                let m_ice_hdeck = Moment::from_pos(a_ice_shift, p_ice_hdeck);
                // Разница в моменте льда от палубного груза (масса льда как на остальной части корпуса) 
                // Масса льда такая же, но высота больше
                let delta_moment_unit = unit_area.delta_moment_h.scale(icing_stab.w_desc);
                // Разница в моменте льда от небледеневающей части палубного груза - леса (льда нет вообще) 
                // Масса льда нулевая, считаем путем вычитания полного момента посчитанного от площади и массы льда на палубе.
                // До этого этот момент получили из момента льда на палубе и дельты момента льда от палубного груза из unit_area.
                let delta_moment_timber_no_icing = icing_timber.moment_no_icing.scale(icing_stab.w_desc);                
                // Разница в массе льда от обледенения палубного груза - леса
                let delta_p_ice_hdeck = icing_timber.area_icing * (icing_stab.w_timber - icing_stab.w_desc);
                // Разница в моменте льда от обледеневающей части палубного груза - леса (масса льда для леса).
                // Состоит из изменения момента льда палубы + момент от дополнительной массы льда.
                let delta_moment_timber_icing = 
                    icing_timber.delta_moment_icing.scale(icing_stab.w_desc) + 
                    icing_timber.moment_icing.scale(icing_stab.w_timber - icing_stab.w_desc);
                // Суммарная масса льда на горизонтальной проекции для каждой составляющей открытых палуб
                let p_ice_h = p_ice_hdeck + delta_p_ice_hdeck;
                // Суммарный момент от масса льда на горизонтальной проекции для каждой составляющей открытых палуб
                let m_ice_h = m_ice_hdeck + delta_moment_unit + delta_moment_timber_icing - delta_moment_timber_no_icing;
                // Суммарная масса льда и его моменты
                let p_ice = p_ice_h + p_ice_v;
                let m_ice = m_ice_v + m_ice_h;
                let result = IcingStabCtx {
                    p_ice,
                    m_ice,
                };
                log::info!(
                    "Icing mass:{:.3} shift:({:.3}, {:.3},{:.3})",
                    result.p_ice,
                    result.m_ice.x(),
                    result.m_ice.y(),
                    result.m_ice.z()
                );
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
        f.debug_struct("IcingStabEval").field("dbg", &self.dbg).finish()
    }
}
