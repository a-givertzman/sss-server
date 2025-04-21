use super::metacentric_height_ctx::MetacentricHeightCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::{data::loads::UnitCargoType, parameters::{IParameters, Parameters}, Bound, Moment, Position},
        eval::IcingTimberCtx,
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ship_model::model_link::{IModelLink, ModelLink}, ContextWrite, CtxResult
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Диаграмма плеч статической и динамической остойчивости
pub struct MetacentricHeightEval {
    dbg: Dbg,
    value: Option<MetacentricHeightCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl MetacentricHeightEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "MetacentricHeightEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for MetacentricHeightEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let parameters: Parameters = ctx.read(); 
                // суммарная масса судна
                let mass = parameters.get(ParameterID::Displacement).ok_or(CtxResult::Err(error.err("calculate mass error: no Displacement in parameters")))?;
                // Смещение центра массы по оси Z
                let mass_shift_z = parameters.get(ParameterID::CenterMassZ).ok_or(CtxResult::Err(error.err("calculate mass_shift_z error: no CenterMassZ in parameters")))?;
                // Продольный - метацентрические радиус
                let rad_long = todo()!;
                // Поперечный метацентрические радиус
                let rad_trans = todo()!;
                // Отстояние центра величины погруженной части судна    
                let center_draught_shift_z = parameters.get(ParameterID::CenterVolumeZ).unwrap();  
                // Все жидкие грузы судна
                let liquid: Vec<_> = match initial.liquid.as_ref() {
                    Some(data) => data
                        .into_iter()
                        .collect(),
                    None => return CtxResult::Err(error.err("Read liquid error: no data!")),
                };
                // Аппликата продольного метацентра (2)
                let Z_m = center_draught_shift_z + rad_long;
                // Поправка к продольной метацентрической высоте на влияние
                // свободной поверхности жидкости в цистернах балласта и запасов (2)
                let delta_m_h_ballast = DeltaMH::from_moment(
                    liquid
                        .iter()
                        .filter(|v| v.assigment_type == AssignmentType::Ballast)
                        .map(|c| c.moment_surface() todo()! )
                        .sum::<FreeSurfaceMoment>(),
                    mass,
                );
                let delta_m_h_store = DeltaMH::from_moment(
                    liquid
                        .iter()
                        .filter(|v| v.assigment_type != AssignmentType::Ballast)
                        .map(|c| c.moment_surface() todo()! )
                        .sum::<FreeSurfaceMoment>(),
                    mass,
                );
                let delta_m_h = delta_m_h_ballast + delta_m_h_store;
                // Продольная метацентрическая высота без учета влияния
                // поправки на влияние свободной поверхности (3)
                let h_long_0 = Z_m - mass_shift_z;
                // Продольная исправленная метацентрическая высота (3)
                let h_long_fix = h_long_0 - delta_m_h.long();
                // Аппликата поперечного метацентра (8)
                let z_m = center_draught_shift_z + rad_trans; //
                // Поперечная метацентрическая высота без учета влияния
                // поправки на влияние свободной поверхности (9)
                let h_trans_0 = z_m - mass_shift_z;
                // Поперечная исправленная метацентрическая высота (9)
                let h_trans_fix = h_trans_0 - delta_m_h.trans();
                // Исправленное отстояние центра масс судна по высоте (10)
                let z_g_fix: f64 = mass_shift_z + delta_m_h.trans();
       //             log::info!("\t MetacentricHeight mass:{} shift_z:{} center_draught:{} rad_trans:{} rad_long:{} delta_m_h_ballast:{} delta_m_h_store:{} Z_m:{Z_m} H_0:{h_long_0} H:{h_long_fix} z_m:{z_m} h_0:{h_trans_0} h:{h_trans_fix} z_g_fix:{z_g_fix}", 
       //                 self.mass.sum()?, self.moment.shift()?.z(), self.center_draught_shift, self.rad_trans, self.rad_long, delta_m_h_ballast.trans, delta_m_h_store.trans() );
                parameters.add(ParameterID::CenterMassZFix, z_g_fix);
                parameters.add(ParameterID::MetacentricLongRadZ, Z_m);
                parameters.add(ParameterID::MetacentricTransRadZ, z_m);
                parameters.add(
                        ParameterID::MetacentricTransBallast,
                        delta_m_h_ballast.trans(),
                    );
                parameters.add(
                        ParameterID::MetacentricTransSum,
                        delta_m_h.trans(),
                    );
                parameters.add(
                        ParameterID::MetacentricLongBallast,
                        delta_m_h_ballast.long(),
                    );
                parameters
                        .add(ParameterID::MetacentricTransStore, delta_m_h_store.trans());
                parameters
                        .add(ParameterID::MetacentricLongStore, delta_m_h_store.long());
                parameters
                        .add(ParameterID::MetacentricTransRad, rad_trans);
                parameters
                        .add(ParameterID::MetacentricLongRad, rad_long);
                parameters
                        .add(ParameterID::MetacentricTransHeight, h_trans_0);
                parameters
                        .add(ParameterID::MetacentricTransHeightFix, h_trans_fix);
                parameters
                        .add(ParameterID::MetacentricLongHeight, h_long_0);
                parameters
                        .add(ParameterID::MetacentricLongHeightFix, h_long_fix);
                let result = MetacentricHeightCtx {
                    h_trans_0,
                    h_long_fix,
                    h_trans_fix,
                    z_g_fix,
                    delta_m_h,
                };
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
impl std::fmt::Debug for MetacentricHeightEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetacentricHeightEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
