use super::ctx::ShearForceCtx;
use crate::{
    algorithm::{
        context::context_access::ContextReadRef,
        entities::{MultipleSingle, SubVec, SumAbove},
        eval::{BalanceCtx, MassCtx, TotalForceCtx},
    },
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::{ContextRead, ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Срезающая сила, действующая на корпус судна
pub struct ShearForceEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl ShearForceEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "ShearForceEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for ShearForceEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let total_force: TotalForceCtx = ctx.read();
                let result = total_force.values.sum_above();
                log::trace!("\t ShearForce result:{:?}", result); 
                // println!("\n\n ShearForce result\n");  result.iter().for_each(|b| print!("{:.3} ", b));
                ctx.write(ShearForceCtx::new(result))
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ShearForceEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShearForceEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
