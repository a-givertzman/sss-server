use super::ctx::BendingMomentCtx;
use crate::{
    algorithm::{
        context::context_access::ContextReadRef, entities::IntegralSum, eval::ShearForceCtx,
    },
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::{ContextRead, ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Изгибающий момент
pub struct BendingMomentEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl BendingMomentEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "BendingMomentEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for BendingMomentEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let bounds = initial
                    .bounds
                    .as_ref()
                    .ok_or(error.err("initial error: no bounds!"))?;
                let shear_force: ShearForceCtx = ctx.read();
                let mut result: Vec<f64> = shear_force.values.integral_sum();
           //     println!("\n\n BendingMoment integral_sum\n");   
           //     result.iter().for_each(|b| print!("{:.3} ", b));

            //    println!("\n\n BendingMoment length\n");  

                result = result
                    .into_iter()
                    .zip(bounds.iter())
                    .map(|(v, b)| {
                 //       print!("{:.3} ", b.length().unwrap_or(0.) / 2.);
                        v * b.length().unwrap_or(0.) / 2.
                    }).collect();
                println!("\n\n BendingMoment result\n");   result.iter().for_each(|b| print!("{:.3} ", b));
                ctx.write(BendingMomentCtx::new(result))
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for BendingMomentEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BendingMomentEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
