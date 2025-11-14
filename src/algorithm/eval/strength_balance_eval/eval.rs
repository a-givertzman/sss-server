use std::sync::Arc;
use sal_core::{dbg::Dbg, error::Error};
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::ship_model::{
                BalanceStrengthQuery,
                ship_model::ShipModel,
            },
        eval::{StaticMassCtx, parameters::ParameterID},
    },
    kernel::{
        eval::Eval,
        types::{RwLock, eval_result::EvalResult},
    },
    prelude::{ContextParamsRead, ContextWrite, InitialCtx},
};

///
/// Расчет равновесного положения судна
pub struct StrengthBalanceEval {
    dbg: Dbg,
    model: Arc<RwLock<ShipModel>>,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl StrengthBalanceEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        model: Arc<RwLock<ShipModel>>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "StrengthBalanceEval");
        Self {
            dbg,
            model,
            ctx: Box::new(ctx),
        }
    }
}
//
impl Eval<(), EvalResult> for StrengthBalanceEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let bounds = initial
                    .bounds
                    .as_ref()
                    .ok_or(error.err("initial error: no bounds!"))?;
                let static_mass: StaticMassCtx = ctx.read();
                let trim = ctx.read_params(ParameterID::TrimDeg);
                let draught = ctx.read_params(ParameterID::DraughtMid);                
                // Расчет баланса для прочности в модели
                let strength_query = BalanceStrengthQuery {
                    trim,
                    draught,
                    water_density: voyage.density,
                    distr_static: static_mass.distr_static,
                    bulk: static_mass.bulk.clone(),
                    liquid: static_mass.liquid.clone(),
                    grain_bulkhead: static_mass.grain_bulkhead,
                    gaseous: static_mass.gaseous,
                    //    damaged_compartment: loads.damaged_compartment, //TODO
                    bounds: bounds.clone(),
                    epsilon: 0.00000001,
                };
                //    damaged_compartment: loads.damaged_compartment, //TODO
                let result = self
                    .model
                    .read()
                    .compute_strength(strength_query)
                    .map_err(|err| error.pass_with("model.compute_balance", err))?;
         /*       let bulk = bulk
                    .iter()
                    .filter_map(|res| {
                        bulk_data.get(&res.assignment_id).map(|data| {
                            super::bulk_result::BulkResult::new(
                                data.space_id.clone(),
                                data.assigment_type,
                                res.mass_values.clone(),
                            )
                        })
                    })
                    .collect();
                let liquid = result
                    .liquid
                    .iter()
                    .filter_map(|res| {
                        liquid_data.get(&res.assignment_id).map(|data| {
                            super::liquid_result::LiquidResult::new(
                                data.cargo_id,
                                data.space_id.clone(),
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
                        gaseous_data.get(&res.assignment_id).map(|data| {
                            super::gaseous_result::GaseousResult::new(
                                data.cargo_id,
                                data.space_id.clone(),
                                data.assigment_type,
                                res.mass_values.clone(),
                            )
                        })
                    })
                    .collect();*/
                //   println!("\n\n Balance displacement mass_sum: {} result\n", result.displacement_distr.iter().sum::<f64>()*1.025);  result.displacement_distr.iter().for_each(|b| print!("{:.3} ", b));
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for StrengthBalanceEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BalanceEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
