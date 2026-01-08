use crate::algorithm::entities::ship_model::StabilityArea;
use crate::algorithm::entities::ship_model::ship_model::ShipModel;
use crate::algorithm::eval::stability::WindageCtx;
use crate::algorithm::eval::{UnitAreaCtx};
use crate::kernel::Eval;
use crate::kernel::types::Arc;
use crate::prelude::{ContextParamsRead, ContextReadRef, InitialCtx};
use crate::{
    algorithm::{
        context::context_access::ContextRead,
        eval::{IcingCoeffCtx, parameters::ParameterID, zg::Zg},
    },
    kernel::types::eval_result::EvalResult,
    prelude::ContextWrite,
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::sync::RwLock;
///
/// Парусность судна, площадь и положение
/// центра относительно миделя и ОП
pub struct WindageEval {
    dbg: Dbg,
    model: Arc<RwLock<ShipModel>>,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl WindageEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        model: Arc<RwLock<ShipModel>>,
        ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "WindageEval");
        Self {
            dbg,
            model,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for WindageEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .ok_or(error.err("ship_parameters error: no data!"))?;
                let midship = *ship_parameters
                    .get("X midship from Fr0")
                    .ok_or(error.err("midship: no data!"))?;
                let icing_stab: IcingCoeffCtx = ctx.read();
                let draught = ctx.read_params(ParameterID::DraughtMid);
                let UnitAreaCtx {
                    av_dc,
                    mv_dc,
                    delta_moment_h,
                    distr_v,
                } = ctx.read();
                let StabilityArea {
                    av_cs_dmin1,
                    mv_x_cs_dmin1,
                    mv_z_cs_dmin1,
                    delta_av,
                    delta_mv_x,
                    delta_mv_z,
                    area_volume_z,
                    ..
                } = match self.model.read().stability_area(draught) {
                    Ok(data) => data,
                    Err(err) => {
                        return Err(error.pass_with("model.stability_area", err));
                    }
                };
                // Площадь парусности сплошных поверхностей для осадки dmin
                let av_cs_dmin = av_cs_dmin1 + av_dc;
                // Cтатический момент площади парусности сплошных поверхностей для осадки dmin
                let mv_x_cs_dmin = mv_x_cs_dmin1 + mv_dc.x();
                let mv_z_cs_dmin = mv_z_cs_dmin1 + mv_dc.z();
                // Парусность несплошных поверхностей
                let av_ds = av_cs_dmin * icing_stab.coef_v_area;                 
                // Центр площади парусности несплошных поверхностей по длине принимается на миделе
                let mv_x_ds = av_cs_dmin * midship; 
                // статический момент площади парусности несплошных поверхностей
                let mv_z_ds = mv_z_cs_dmin * icing_stab.coef_v_moment; 
                // Площадь парусности судна для осадки dmin
                let av_dmin = av_cs_dmin + av_ds;
                // статические моменты площади парусности судна для осадки dmin
                let mv_x_dmin = mv_x_cs_dmin + mv_x_ds;                 
                let mv_z_dmin = mv_z_cs_dmin + mv_z_ds;
                // Площадь парусности судна для текущей осадки
                let av = av_dmin - delta_av; 
                // Разница в статических моментах для текущей осадки и
                // осадки dmin относительно начала координат, м^3
                let mv_x = mv_x_dmin - delta_mv_x;
                // Разница в статических моментах для текущей осадки и
                // осадки dmin относительно ОП, м^3
                let mv_z = mv_z_dmin - delta_mv_z;
                // Отстояние центра площади парусности судна для текущей загрузки относительно начала координат, м
                let xv = mv_x / av;
                // Отстояние центра площади парусности судна для текущей загрузки относительно ОП, м
                let zv_bp = mv_z / av;
                // Плечо парусности определяется как вертикальное расстояние [м] между центром парусности и
                // центром площади проекции подводной части корпуса на диаметральную плоскость
                // в прямом положении судна на спокойной воде
                let zv = zv_bp - area_volume_z;
                let result = WindageCtx { av, xv, zv };
                log::info!(
                    "Windage av:{:.3} xv:{:.3} zv:{:.3}",
                    av, xv, zv
                ); 
                log::trace!("\t Wind av_cs_dmin:{av_cs_dmin} mv_x_cs_dmin:{mv_x_cs_dmin} mv_z_cs_dmin:{mv_z_cs_dmin}
                    av_ds:{av_ds} mv_x_ds:{mv_x_ds} mv_z_ds:{mv_z_ds}
                    av_dmin:{av_dmin} mv_x_dmin:{mv_x_dmin} mv_z_dmin:{mv_z_dmin}
                    av:{av} mv_x:{mv_x} mv_z:{mv_z} zv_bp:{zv_bp}");
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for WindageEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindageEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
