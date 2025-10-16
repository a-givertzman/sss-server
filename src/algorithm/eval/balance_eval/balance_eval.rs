use std::sync::Arc;

use sal_core::{dbg::Dbg, error::Error};

use super::balance_ctx::BalanceCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::{
            Moment,
            ship_model::{BalanceQuery, BalanceResult, ship_model::ShipModel},
        },
        eval::{IcingCtx, LoadsCtx, WettingCtx, parameters::ParameterID},
    },
    kernel::{
        eval::Eval,
        types::{RwLock, eval_result::EvalResult},
    },
    prelude::{ContextParamsWrite, ContextWrite, InitialCtx},
};

///
/// Расчет равновесного положения судна
pub struct BalanceEval {
    dbg: Dbg,
    model: Arc<RwLock<ShipModel>>,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl BalanceEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        model: Arc<RwLock<ShipModel>>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "BalanceEval");
        Self {
            dbg,
            model,
            ctx: Box::new(ctx),
        }
    }
}
//
impl Eval<(), EvalResult> for BalanceEval {
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
                let gaseous_data = initial
                    .gaseous
                    .clone()
                    .ok_or(error.err("Read gaseous error: no data!"))?;
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let bounds = initial
                    .bounds
                    .as_ref()
                    .ok_or(error.err("initial error: no bounds!"))?;
                let loads: LoadsCtx = ctx.read();
                let icing: IcingCtx = ctx.read();
                let wetting: WettingCtx = ctx.read();
                // Суммарная масса корпуса, грузов за вычетом смещяемых и насыпных груов и обледенения с намоканием
                let mass_const = loads.mass_const
                    + loads.mass_unit
                    + loads.mass_gaseous
                    + icing.mass
                    + wetting.mass;
                // Сумарный момент за вычетом смещяемых и насыпных груов
                let moment_const = Moment::from_pos(loads.shift_const, loads.mass_const)
                    + Moment::from_pos(loads.shift_unit, loads.mass_unit)
                    + Moment::from_pos(loads.shift_gaseous, loads.mass_gaseous)
                    + Moment::new(icing.mass * icing.mass_shift_x, 0., 0.)
                    + Moment::from_pos(wetting.mass_shift, wetting.mass);
                // Структура для передачи в модель
                let balance_query = BalanceQuery {
                    water_density: voyage.density,
                    mass_const,
                    moment_const,
                    bulk: loads.bulk.clone(),
                    liquid: loads.liquid.clone(),
                    grain_bulkhead: loads.grain_bulkhead,
                    gaseous: loads.gaseous,
                    //    damaged_compartment: loads.damaged_compartment, //TODO
                    bounds: bounds.clone(),
                    epsilon: 0.001,
                };
                // Расчет баланса в модели
                let result: BalanceResult = self
                    .model
                    .read()
                    .compute_balance(balance_query)
                    .map_err(|err| error.pass_with("model.compute_balance", err))?;
                ctx.write_params(ParameterID::DraughtMid, result.draught_mid);
                ctx.write_params(ParameterID::DraughtBow, result.draught_bow);
                ctx.write_params(ParameterID::DraughtStern, result.draught_stern);
                ctx.write_params(ParameterID::DraughtMean, result.draught_mean);
                ctx.write_params(ParameterID::TrimDeg, result.trim_degree);
                ctx.write_params(ParameterID::TrimMeter, result.trim_meter);
                ctx.write_params(ParameterID::Roll, result.roll);
                ctx.write_params(ParameterID::MetacentricTransRad, result.rad_trans);
                ctx.write_params(ParameterID::MetacentricLongRad, result.rad_long);
                let bulk = result
                    .bulk
                    .iter()
                    .filter_map(|res| {
                        bulk_data.get(&res.assigned_id).map(|data| {
                            super::bulk_result::BulkResult::new(
                                data.cargo_id,
                                data.space_id.clone(),
                                data.assigment_type,
                                res.moment,
                                res.mass_values.clone(),
                            )
                        })
                    })
                    .collect();
                let liquid = result
                    .liquid
                    .iter()
                    .filter_map(|res| {
                        liquid_data.get(&res.assigned_id).map(|data| {
                            super::liquid_result::LiquidResult::new(
                                data.cargo_id,
                                data.space_name.clone(),
                                data.assigment_type,
                                data.cargo_type,
                                res.long_moment_of_inertia,
                                res.trans_moment_of_inertia,
                                res.mass_values.clone(),
                            )
                        })
                    })
                    .collect();    
                let gaseous = result
                    .gaseous
                    .iter()
                    .filter_map(|res| {
                        gaseous_data.get(&res.assigned_id).map(|data| {
                            super::gaseous_result::GaseousResult::new(
                                data.cargo_id,
                                data.space_name.clone(),
                                data.assigment_type,
                                res.mass_values.clone(),
                            )
                        })
                    })
                    .collect();                             
                let result = BalanceCtx {
                    bulk,
                    liquid,
                    gaseous,
                    displacement_distr: result.displacement_distr,
                    length_wl: result.length_wl,
                    breadth_wl: result.breadth_wl,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for BalanceEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BalanceEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
