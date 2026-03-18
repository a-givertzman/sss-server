use crate::algorithm::entities::Moment;
use crate::algorithm::entities::ship_model::ship_model::ShipModel;
use crate::algorithm::eval::icing_timber::ctx::IcingTimberCtx;
use crate::algorithm::eval::parameters::ParameterID;
use crate::algorithm::eval::{IcingCoeffCtx, UnitAreaCtx};
use crate::algorithm::eval::stability::IcingStabCtx;
use crate::prelude::ContextParamsWrite;
use crate::{
    kernel::{Eval, types::{Arc, RwLock, eval_result::EvalResult}},
    prelude::*,
};
///
/// Учет обледенения судна [https://github.com/a-givertzman/sss/blob/master/design/algorithm/part02_mass/chapter02_icing.md]
sal_core::define_dbg! {
    pub struct IcingStabEval {
        model: Arc<RwLock<ShipModel>>,
        ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
    }
}
// 2. Реализация трейта Eval с использованием автоматической области видимости (area)
#[sal_core::error::auto_impl] // Макрос вставит: let e = self.scope("eval");
impl Eval<(), EvalResult> for IcingStabEval {
    fn eval(&self, _: ()) -> EvalResult {
        // Теперь `self.ctx` доступен (это обычный метод), а `e` видна (вставлена макросом)
        
        let mut ctx = self.ctx.eval(())
            .map_err(|err| e.err_with("Read context error", err))?;

        // Чтение данных из контекста (используем макрос ContextRead из начала беседы)
        let icing_coeff: IcingCoeffCtx = ctx.read();
        let icing_timber: IcingTimberCtx = ctx.read();
        let unit_area: UnitAreaCtx = ctx.read();

        // Работа с моделью судна
        let model_guard = self.model.read();
        
        let (av_cs_dmin, mv_cs_dmin) = model_guard.static_area_v()
            .map_err(|err| e.err_with("model.static_area_v failed", err))?;
            
        let (a_ice_hdeck, a_ice_shift) = model_guard.static_area_h()
            .map_err(|err| e.err_with("model.static_area_h failed", err))?;

        // --- Расчетная логика обледенения ---
        
        // Масса и момент льда на вертикальных поверхностях
        let p_ice_v = (av_cs_dmin + unit_area.av_dc) * icing_coeff.w_ice_v;
        let m_ice_v = (mv_cs_dmin + unit_area.mv_dc).scale(icing_coeff.w_ice_v);

        // Масса и момент на открытых палубах
        let p_ice_hdeck = icing_coeff.w_desc * (a_ice_hdeck - icing_timber.area_no_icing);
        let m_ice_hdeck = Moment::from_pos(a_ice_shift, p_ice_hdeck);

        // Поправки на палубный груз (лес)
        let delta_moment_unit = unit_area.delta_moment_h.scale(icing_coeff.w_desc);
        let delta_moment_timber_no_icing = icing_timber.moment_no_icing.scale(icing_coeff.w_desc);                
        let delta_p_ice_hdeck = icing_timber.area_icing * (icing_coeff.w_timber - icing_coeff.w_desc);
        
        let delta_moment_timber_icing = 
            icing_timber.delta_moment_icing.scale(icing_coeff.w_desc) + 
            icing_timber.moment_icing.scale(icing_coeff.w_timber - icing_coeff.w_desc);

        // Итоговые значения
        let p_ice_h = p_ice_hdeck + delta_p_ice_hdeck;
        let m_ice_h = m_ice_hdeck + delta_moment_unit + delta_moment_timber_icing - delta_moment_timber_no_icing;
        
        let mass = p_ice_h + p_ice_v;
        let moment = m_ice_v + m_ice_h;
        
        let mass_shift = moment.to_pos(mass);
        let shift_h = m_ice_h.to_pos(p_ice_h);
        let shift_v = m_ice_v.to_pos(p_ice_v);

        // Логирование через e.info (area "eval" подставлена автоматически)
        e.info(format!("Icing mass: {:.3} at {:?}", mass, mass_shift));

        // Запись параметров в контекст
        ctx.write_params(ParameterID::MassIcing, mass);
        ctx.write_params(ParameterID::MassIcingX, mass_shift.x());
        ctx.write_params(ParameterID::MassIcingY, mass_shift.y());
        ctx.write_params(ParameterID::MassIcingZ, mass_shift.z());

        ctx.write_params(ParameterID::WeightOfIceOnHorizontalSurfaces, p_ice_h);
        ctx.write_params(ParameterID::LongitudinalCenterOfWeightOfIceOnHorizontalSurfaces, shift_h.x());
        ctx.write_params(ParameterID::WeightOfIceOnVerticalSurfaces, p_ice_v);
        ctx.write_params(ParameterID::VerticalCenterOfWeightOfIceOnVerticalSurfaces, shift_v.z());

        // Возврат результата
        ctx.write(IcingStabCtx { mass, moment })
    }
}