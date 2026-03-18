use crate::algorithm::eval::strength::{DynamicMassCtx, StrengthBalanceCtx, TotalForceCtx};
use crate::{
    algorithm::entities::{MultipleSingle, SubVec},
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::*,
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Результирующая нагрузка на шпацию
pub struct TotalForceEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl TotalForceEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "TotalForceEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for TotalForceEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let mass: DynamicMassCtx = ctx.read();
                let mass_values = mass.values;
                let balance: StrengthBalanceCtx = ctx.read();
                let mut volume_values = balance.displacement_distr;
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let water_density = voyage.density;
                let gravity_g = 9.81;
                if mass_values.len() != volume_values.len() {
                    let error = error.err("mass_values.len() != volume_values.len()");
                    log::error!("{error}");
                    return Err(error);
                }
                let mut result = mass_values.clone();
                volume_values.mul_single(water_density);
            //    println!("\n\n mass qnt:{} sum: {}\n", mass_values.len(), mass_values.iter().sum::<f64>());  mass_values.iter().for_each(|b| print!("{:.3} ", b)); 
            //    println!("\n\n volume qnt:{} sum: {}\n", volume_values.len(), volume_values.iter().sum::<f64>());  volume_values.iter().for_each(|b| print!("{:.3} ", b));
                result.sub_vec(&volume_values)?;
                result.mul_single(gravity_g);
                log::info!(
                    "TotalForce result_sum:{:.3}",
                    result.iter().sum::<f64>()
                );
                log::trace!(
                    "TotalForce result_distr:{}",
                    result
                        .iter()
                        .fold(String::new(), |s, v| s + &format!("{:.3} ", v))
                );
                ctx.write(TotalForceCtx::new(result))
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for TotalForceEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TotalForceEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
