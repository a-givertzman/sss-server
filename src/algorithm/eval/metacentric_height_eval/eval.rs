use crate::algorithm::entities::math::liquid::*;
use crate::algorithm::eval::MetacentricHeightCtx;
use crate::kernel::Eval;
use crate::{
    algorithm::{
        context::context_access::{
            ContextParamsRead, ContextParamsWrite, ContextRead, ContextReadRef,
        },
        entities::data::loads::AssignmentType,
        eval::{StabilityBalanceCtx, Zg, parameters::ParameterID},
    },
    kernel::types::{Arc, RwLock, eval_result::EvalResult},
    prelude::{Context, ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Диаграмма плеч статической и динамической остойчивости
pub struct MetacentricHeightEval {
    dbg: Dbg,
    context: Arc<RwLock<Option<Context>>>,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl MetacentricHeightEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "MetacentricHeightEval");
        Self {
            dbg,
            context: Arc::new(RwLock::new(None)),
            ctx: Box::new(ctx),
        }
    }
    ///
    ///
    fn calc(&self, mut ctx: Context, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "calc");
        // суммарная масса судна
        let mass = ctx.read_params(ParameterID::Displacement);
        // Смещение центра массы по оси Z
        let mass_shift_z = ctx.read_params(ParameterID::CenterMassZ);
        let initial: &InitialCtx = ctx.read_ref();
        let ship_parameters = initial.ship_parameters.as_ref().unwrap();
        let ship_length_lbp = *ship_parameters
            .get("LBP")
            .ok_or(error.err("No LBP in ship_parameters"))?;
        let balance: StabilityBalanceCtx = ctx.read();
        // Продольный метацентрический радиус
        let rad_long = ctx.read_params(ParameterID::MetacentricLongRad);
        // Поперечный метацентрические радиус
        let rad_trans = ctx.read_params(ParameterID::MetacentricTransRad);
        // Отстояние центра величины погруженной части судна
        let center_draught_shift_z = ctx.read_params(ParameterID::CenterVolumeZ);
        // Все жидкие грузы судна
        let liquid = &balance.liquid;
        // Аппликата продольного метацентра (2)
        let z_m_long = center_draught_shift_z + rad_long;
        // Поправка к продольной метацентрической высоте на влияние
        // свободной поверхности жидкости в цистернах балласта и запасов (2)
        let delta_m_h_ballast: DeltaMH = DeltaMH::from_moment(
            liquid
                .iter()
                .filter(|v| v.assigment_type == AssignmentType::Ballast)
                .map(|c| {
                    FreeSurfaceMoment::new(c.trans_moment_of_inertia, c.long_moment_of_inertia)
                })
                .sum::<FreeSurfaceMoment>(),
            mass,
        );
        let delta_m_h_store: DeltaMH = DeltaMH::from_moment(
            liquid
                .iter()
                .filter(|v| v.assigment_type != AssignmentType::Ballast)
                .map(|c| {
                    FreeSurfaceMoment::new(c.trans_moment_of_inertia, c.long_moment_of_inertia)
                })
                .sum::<FreeSurfaceMoment>(),
            mass,
        );
        let delta_m_h = delta_m_h_ballast + delta_m_h_store;
        // Продольная метацентрическая высота без учета влияния
        // поправки на влияние свободной поверхности (3)
        let h_long_0 = z_m_long - mass_shift_z;
        // Продольная исправленная метацентрическая высота (3)
        let h_long_fix = h_long_0 - delta_m_h.long();
        // Момент дифферентующий на 1 см осадки (4)
        let trim_moment = (mass * h_long_fix) / (100. * ship_length_lbp);
        // Аппликата поперечного метацентра (8)
        let z_m_trans = center_draught_shift_z + rad_trans; //
        // Поперечная метацентрическая высота без учета влияния
        // поправки на влияние свободной поверхности (9)
        let (h_trans_0, h_trans_fix, z_g_fix) = if let Some(z_g_fix) = z_g_fix.0 {
            let h_trans_fix = z_m_trans - z_g_fix;
            let h_trans_0 = h_trans_fix + delta_m_h.trans();
            (h_trans_0, h_trans_fix, z_g_fix)
        } else {
            let h_trans_0 = z_m_trans - mass_shift_z;
            // Поперечная исправленная метацентрическая высота (9)
            let h_trans_fix = h_trans_0 - delta_m_h.trans();
            // Исправленное отстояние центра масс судна по высоте (10)
            let z_g_fix: f64 = mass_shift_z + delta_m_h.trans();
            (h_trans_0, h_trans_fix, z_g_fix)
        };
        ctx.write_params(ParameterID::CenterMassZFix, z_g_fix);
        ctx.write_params(ParameterID::MetacentricLongRadZ, z_m_long);
        ctx.write_params(ParameterID::MetacentricTransRadZ, z_m_trans);
        ctx.write_params(
            ParameterID::MetacentricTransBallast,
            delta_m_h_ballast.trans(),
        );
        ctx.write_params(ParameterID::MetacentricTransSum, delta_m_h.trans());
        ctx.write_params(
            ParameterID::MetacentricLongBallast,
            delta_m_h_ballast.long(),
        );
        ctx.write_params(ParameterID::MetacentricTransStore, delta_m_h_store.trans());
        ctx.write_params(ParameterID::MetacentricLongStore, delta_m_h_store.long());
        ctx.write_params(ParameterID::MetacentricTransRad, rad_trans);
        ctx.write_params(ParameterID::MetacentricLongRad, rad_long);
        ctx.write_params(ParameterID::MetacentricTransHeight, h_trans_0);
        ctx.write_params(ParameterID::MetacentricTransHeightFix, h_trans_fix);
        ctx.write_params(ParameterID::MetacentricLongHeight, h_long_0);
        ctx.write_params(ParameterID::MetacentricLongHeightFix, h_long_fix);
        ctx.write_params(ParameterID::MomentTrimPerCm, trim_moment);
        ctx.write_params(
            ParameterID::MomentRollPerDeg,
            mass * h_trans_fix.to_radians().sin(),
        );
        let result = MetacentricHeightCtx {
            h_trans_0,
            h_long_fix,
            h_trans_fix,
            z_g_fix,
            delta_m_h,
        };
        log::info!(
            "\t MetacentricHeight delta_m_h:{:.3} h_long_fix:{:.3} h_trans_0:{:.3} h_trans_fix:{:.3} z_g_fix:{:.3}",
            result.delta_m_h,
            result.h_long_fix,
            result.h_trans_0,
            result.h_trans_fix,
            result.z_g_fix
        );
        ctx.write(result)
    }
}
//
//
impl Eval<Zg, EvalResult> for MetacentricHeightEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        let tmp_context = self.context.read().clone();
        match tmp_context {
            Some(ctx) => self.calc(ctx, z_g_fix),
            None => match self.ctx.eval(()) {
                Ok(ctx) => {
                    *self.context.write() = Some(ctx.clone());
                    self.calc(ctx, Zg(None))
                }
                Err(err) => Err(error.pass_with("Read context error", err)),
            },
        }
    }
}
//
//
impl std::fmt::Debug for MetacentricHeightEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetacentricHeightEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
