use std::sync::Arc;
use sal_core::{dbg::Dbg, error::Error};
use crate::algorithm::eval::StabilityBalanceCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::{
            Curve, ICurve, Moment, ship_model::{BalanceStabilityQuery, ship_model::ShipModel, stability_result::BalanceStabilityResult}
        },
        eval::{IcingCtx, StaticMassCtx, WettingCtx, parameters::ParameterID},
    },
    kernel::{
        Eval,
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
             /*   let bulk_data = initial
                    .bulk
                    .clone()
                    .ok_or(error.err("Read bulk error: no data!"))?;
                let liquid_data = initial
                    .liquid
                    .clone()
                    .ok_or(error.err("Read liquid error: no data!"))?;*/
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
                    + Moment::from_pos(icing.mass_shift, icing.mass)
                    + Moment::from_pos(wetting.mass_shift, wetting.mass);
                //  dbg!(loads.shift_const, loads.shift_unit, loads.shift_gaseous, icing.mass_shift_x, wetting.mass_shift);
             //   let liquid: f64 = static_mass.liquid.iter().map(|v| v.mass).sum();
             //   let bulk: f64 = static_mass.bulk.iter().map(|v| v.mass).sum();
           //     let sum = liquid + bulk + static_mass.mass_const + static_mass.mass_unit + static_mass.mass_gaseous + icing.mass + wetting.mass;
                //  dbg!(&static_mass); 
          //      dbg!(sum, liquid, bulk, static_mass.mass_const, static_mass.mass_unit, static_mass.mass_gaseous, icing.mass, wetting.mass);
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

       /*         let (mass_ballast, moment_ballast) = result.liquid.iter()
                    .filter(|v| v.assigment_type == AssignmentType::Ballast)
                        .fold((0., Moment::zero()), |(mass, moment), v| (mass + v.mass, moment + Moment::from_pos(v.mass_shift, v.mass)));
                let ballast_shift = moment_ballast.to_pos(mass_ballast);
                dbg!(mass_ballast, ballast_shift);
                let (mass_liquid_stores, moment_liquid_stores) = result.liquid.iter()
                    .filter(|v| v.assigment_type == AssignmentType::Stores)
                        .fold((0., Moment::zero()), |(mass, moment), v| (mass + v.mass, moment + Moment::from_pos(v.mass_shift, v.mass)));
                let liquid_stores_shift = moment_liquid_stores.to_pos(mass_liquid_stores);
                dbg!(mass_liquid_stores, liquid_stores_shift);
                let (mass_bulk_cargo, moment_bulk_cargo) = result.bulk.iter()
                    .filter(|v| v.assigment_type == AssignmentType::CargoLoad)
                        .fold((0., Moment::zero()), |(mass, moment), v| (mass + v.mass, moment + Moment::from_pos(v.mass_shift, v.mass)));
                let bulk_cargo_shift = moment_bulk_cargo.to_pos(mass_bulk_cargo);     
                dbg!(mass_bulk_cargo, bulk_cargo_shift); 
*/
          //      dbg!(result.roll, result.trim_degree, result.draught_mean, result.mass_center ); 
        //        dbg!(&result);
           //     result.liquid.iter().for_each(|v| println!("'{}' {} {};", liquid_data.get(&v.assignment_id).unwrap().space_name, v.long_moment_of_inertia, v.trans_moment_of_inertia));
                ctx.write_params(ParameterID::DraughtMid, result.draught_mid);
                ctx.write_params(ParameterID::DraughtBow, result.draught_bow);
                ctx.write_params(ParameterID::DraughtStern, result.draught_stern);
                ctx.write_params(ParameterID::DraughtMean, result.draught_mean);
                ctx.write_params(ParameterID::TrimDeg, result.trim_degree);
                ctx.write_params(ParameterID::TrimMeter, result.trim_meter);
                ctx.write_params(ParameterID::Roll, result.roll);
                ctx.write_params(ParameterID::MetacentricTransRad, result.rad_trans);
                ctx.write_params(ParameterID::MetacentricLongRad, result.rad_long);
                ctx.write_params(ParameterID::CenterMassZ, result.mass_center.z());
                ctx.write_params(
                    ParameterID::CenterVolumeXFromStern,
                    result.displacement_center.x()
                );
                ctx.write_params(ParameterID::CenterVolumeY, result.displacement_center.y());
                ctx.write_params(ParameterID::CenterVolumeZ, result.displacement_center.z());
                let bulk = result
                    .bulk
                    .clone();
                let liquid = result
                    .liquid
                    .clone();
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
                    entry_angle,
                    flooding_angle,
                    bow_area: result.bow_area,
                    dso: result.dso,
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
