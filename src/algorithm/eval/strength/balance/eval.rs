use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::ship_model::{BalanceStrengthQuery, ship_model::ShipModel},
        eval::{parameters::ParameterID, strength::StaticMassStrCtx},
    },
    kernel::{
        Eval,
        types::{RwLock, eval_result::EvalResult},
    },
    prelude::{ContextParamsRead, ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
use std::sync::Arc;

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
    //
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
                let static_mass: StaticMassStrCtx = ctx.read();
                let trim = ctx.read_params(ParameterID::TrimDeg);
                let draught = ctx.read_params(ParameterID::DraughtMid);
                // dbg!(&static_mass);
                // Расчет баланса для прочности в модели
                let strength_query = BalanceStrengthQuery {
                    trim,
                    draught,
                    water_density: voyage.density,
                    distr_static: static_mass.distr_static,
                    bulk: static_mass.bulk.clone(),
                    liquid: static_mass.liquid.clone(),                    
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
                //   println!("\n\n Balance displacement mass_sum: {} result\n", result.displacement_distr.iter().sum::<f64>()*1.025);  result.displacement_distr.iter().for_each(|b| print!("{:.3} ", b));
                log::info!(
                    "StrengthBalance mass_displacement_sum:{:.3}",
                    result.displacement_distr.iter().sum::<f64>() * voyage.density
                );
                log::trace!(
                    "StrengthBalance mass_displacement_distr:{}",
                    result
                        .displacement_distr
                        .iter()
                        .fold(String::new(), |s, v| s + &format!("{:.3} ", v * voyage.density))
                );
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
