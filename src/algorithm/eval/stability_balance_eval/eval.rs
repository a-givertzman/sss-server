use crate::algorithm::eval::StabilityBalanceCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::ship_model::{
            BalanceStabilityQuery, ship_model::ShipModel, stability_result::BalanceStabilityResult,
        },
        eval::{StaticMassCtx, parameters::ParameterID},
    },
    kernel::{
        Eval,
        types::{RwLock, eval_result::EvalResult},
    },
    prelude::{ContextParamsWrite, ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
use std::collections::HashMap;
use std::sync::Arc;

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
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let static_mass: StaticMassCtx = ctx.read();
                // Расчет баланса для остойчивости в модели
                let stability_query = BalanceStabilityQuery {
                    water_density: voyage.density,
                    mass_const: static_mass.mass_const,
                    moment_const: static_mass.moment_const,
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
                //        dbg!(&result);
            /*    let liquid_data: HashMap<usize, String> = <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                        .liquid
                        .as_ref()
                        .ok_or(error.err("Read liquid error: no data!"))?
                        .iter()
                        .map(|(_, v)| (v.assignment_id, v.space_id.clone()))
                        .collect();             
                result.liquid.iter().for_each(|v| println!("'{}' {};", liquid_data.get(&v.assignment_id).unwrap(), v.trans_moment_of_inertia));
            */
                ctx.write_params(ParameterID::DraughtMid, result.draught_mid);
                ctx.write_params(ParameterID::DraughtBow, result.draught_bow);
                ctx.write_params(ParameterID::DraughtStern, result.draught_stern);
                ctx.write_params(ParameterID::DraughtMean, result.draught_mean);
                ctx.write_params(ParameterID::TrimDeg, result.trim_degree);
                ctx.write_params(ParameterID::TrimMeter, result.trim_meter);
                ctx.write_params(ParameterID::Roll, result.heel);
                ctx.write_params(ParameterID::MetacentricTransRad, result.rad_trans);
                ctx.write_params(ParameterID::MetacentricLongRad, result.rad_long);
                ctx.write_params(ParameterID::CenterMassZ, result.mass_center.z());
                ctx.write_params(
                    ParameterID::CenterVolumeXFromStern,
                    result.displacement_center.x(),
                );
                ctx.write_params(ParameterID::CenterVolumeY, result.displacement_center.y());
                ctx.write_params(ParameterID::CenterVolumeZ, result.displacement_center.z());
                let bulk = result.bulk.clone();
                let liquid = result.liquid.clone();
                log::info!(
                    "StabilityBalance heel:{:.3} trim_degree:{:.3} trim_meter:{:.3} draught_mid:{:.3} displacement:{:.3} 
                    length_wl:{:.3} breadth_wl:{:.3} bow_area:{:.3} rad_trans:{:.3} rad_long:{:.3}
                    mass_center:{} displacement_center:{}\n",
                    result.heel,
                    result.trim_degree,
                    result.trim_meter,
                    result.draught_mid,
                    result.displacement,
                    result.length_wl,
                    result.breadth_wl,
                    result.bow_area,
                    result.rad_trans,
                    result.rad_long,
                    result.mass_center.print(),
                    result.displacement_center.print(),
                );
                let result = StabilityBalanceCtx {
                    heel: result.heel,
                    trim: result.trim_degree,
                    draught_mid: result.draught_mid,
                    mass_center: result.mass_center,
                    displacement: result.displacement,
                    bulk,
                    liquid,
                    length_wl: result.length_wl,
                    breadth_wl: result.breadth_wl,
                    bow_area: result.bow_area,
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
