use crate::TotalForceCtx;
use crate::{
    algorithm::{
        context::context_access::ContextReadRef,
        entities::{MultipleSingle, SubVec},
        eval::{DynamicMassCtx, StrengthBalanceCtx},
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextRead, ContextWrite, InitialCtx},
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
                let mass_values = mass.mass_distr;
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
                log::trace!(
                    "\t TotalForce mass:{:?} volume:{:?} result:{:?}, mass_sum:{}, volume_mass_sum:{}",
                    mass_values,
                    volume_values,
                    result,
                    mass_values.iter().sum::<f64>(),
                    volume_values.iter().sum::<f64>()
                );
        //        println!("\n\n TotalForce qnt:{} result\n", result.len());   result.iter().for_each(|b| print!("{:.3} ", b)); 
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
