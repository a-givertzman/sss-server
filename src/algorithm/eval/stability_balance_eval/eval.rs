use std::sync::Arc;
use sal_core::{dbg::Dbg, error::Error};
use super::ctx::StabilityBalanceCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::{
            Curve, ICurve, Moment, ship_model::{BalanceStabilityQuery, ship_model::ShipModel, stability_result::BalanceStabilityResult}
        },
        eval::{IcingCtx, StaticMassCtx, WettingCtx, parameters::ParameterID},
    },
    kernel::{
        eval::Eval,
        types::{RwLock, eval_result::EvalResult},
    },
    prelude::{ContextParamsWrite, ContextWrite, InitialCtx},
};

///
/// Расчет равновесного положения судна
pub struct StabilityBalanceEval {
    dbg: Dbg,
    model: Arc<RwLock<ShipModel>>,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl StabilityBalanceEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        model: Arc<RwLock<ShipModel>>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "StabilityBalanceEval");
        Self {
            dbg,
            model,
            ctx: Box::new(ctx),
        }
    }
}
//
impl Eval<(), EvalResult> for StabilityBalanceEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(mut ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let bulk_data = initial
                    .bulk
                    .clone()
                    .ok_or(error.err("Read bulk error: no data!"))?;
                let liquid_data = initial
                    .liquid
                    .clone()
                    .ok_or(error.err("Read liquid error: no data!"))?;
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let static_mass: StaticMassCtx = ctx.read();
                let icing: IcingCtx = ctx.read();
                let wetting: WettingCtx = ctx.read();
                // Суммарная масса корпуса, обледенения с намоканием и грузов за вычетом смещяемых и насыпных грузов
                let mass_const = static_mass.mass_const
                    + static_mass.mass_unit
                    + static_mass.mass_gaseous
                    + icing.mass
                    + wetting.mass;
                // Сумарный момент за вычетом смещяемых и насыпных груов
                let moment_const = Moment::from_pos(static_mass.shift_const, static_mass.mass_const)
                    + Moment::from_pos(static_mass.shift_unit, static_mass.mass_unit)
                    + Moment::from_pos(static_mass.shift_gaseous, static_mass.mass_gaseous)
                    + Moment::new(icing.mass * icing.mass_shift_x, 0., 0.)
                    + Moment::from_pos(wetting.mass_shift, wetting.mass);
                //  dbg!(loads.shift_const, loads.shift_unit, loads.shift_gaseous, icing.mass_shift_x, wetting.mass_shift);
                let liquid: f64 = static_mass.liquid.iter().map(|v| v.mass).sum();
                let bulk: f64 = static_mass.bulk.iter().map(|v| v.mass).sum();
                let sum = liquid + bulk + static_mass.mass_const + static_mass.mass_unit + static_mass.mass_gaseous + icing.mass + wetting.mass;
                //  dbg!(&static_mass); 
                  dbg!(sum, liquid, bulk, static_mass.mass_const, static_mass.mass_unit, static_mass.mass_gaseous, icing.mass, wetting.mass);
                // Расчет баланса для остойчивости в модели
                let stability_query = BalanceStabilityQuery {
                    water_density: voyage.density,
                    mass_const,
                    moment_const,
                    bulk: static_mass.bulk.clone(),
                    liquid: static_mass.liquid.clone(),
                    grain_bulkhead: static_mass.grain_bulkhead,
                    damaged_compartment: Vec::new(), //TODO: damaged_compartment, только для аварийного расчета
                };
                let result: BalanceStabilityResult = self
                    .model
                    .read()
                    .compute_stability(stability_query)
                    .map_err(|err| error.pass_with("model.compute_balance", err))?;
          //      dbg!(&result);
                //    dbg!(result.roll, result.trim_degree, result.draught_mean);
                ctx.write_params(ParameterID::DraughtMid, result.draught_mid);
                ctx.write_params(ParameterID::DraughtBow, result.draught_bow);
                ctx.write_params(ParameterID::DraughtStern, result.draught_stern);
                ctx.write_params(ParameterID::DraughtMean, result.draught_mean);
                ctx.write_params(ParameterID::TrimDeg, result.trim_degree);
                ctx.write_params(ParameterID::TrimMeter, result.trim_meter);
                ctx.write_params(ParameterID::Roll, result.roll);
                ctx.write_params(ParameterID::MetacentricTransRad, result.rad_trans);
                ctx.write_params(ParameterID::MetacentricLongRad, result.rad_long);
                ctx.write_params(ParameterID::CenterMassZ, result.mass_z);
                ctx.write_params(
                    ParameterID::CenterVolumeXFromStern,
                    result.displacement_center.x()
                );
                ctx.write_params(ParameterID::CenterVolumeY, result.displacement_center.y());
                ctx.write_params(ParameterID::CenterVolumeZ, result.displacement_center.z());
                let bulk = result
                    .bulk
                    .iter()
                    .filter_map(|res| {
                        bulk_data.get(&res.assignment_id).map(|data| {
                            super::bulk_result::BulkResult::new(data.space_id.clone(), res.moment)
                        })
                    })
                    .collect();
                let liquid = result
                    .liquid
                    .iter()
                    .filter_map(|res| {
                        liquid_data.get(&res.assignment_id).map(|data| {
                            super::liquid_result::LiquidResult::new(
                                data.space_id.clone(),
                                data.assigment_type,
                                res.long_moment_of_inertia,
                                res.trans_moment_of_inertia,
                            )
                        })
                    })
                    .collect();
                let entry_angle = Curve::new_linear(&result.entry_angle)
                    .map_err(|err| error.pass_with("entry_angle curve", err))?
                    .value(0.)
                    .map_err(|err| error.pass_with("entry_angle value", err))?;
                let flooding_angle = Curve::new_linear(&result.flooding_angle)
                    .map_err(|err| error.pass_with("flooding_angle curve", err))?
                    .value(0.)
                    .map_err(|err| error.pass_with("flooding_angle value", err))?;
                ctx.write_params(ParameterID::OpenDeckEdgeImmersionAngle, entry_angle);
                ctx.write_params(ParameterID::AngleOfDownFlooding, flooding_angle);
                let result = StabilityBalanceCtx {
                    displacement: result.displacement,
                    bulk,
                    liquid,
                    length_wl: result.length_wl,
                    breadth_wl: result.breadth_wl,
                    dso: result.dso,
                    entry_angle,
                    flooding_angle,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for StabilityBalanceEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StabilityBalanceEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
