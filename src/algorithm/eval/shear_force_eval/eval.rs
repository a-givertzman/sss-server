use crate::algorithm::eval::ShearForceCtx;
use crate::{
    algorithm::{
        entities::SumAbove,
        eval::TotalForceCtx,
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextRead, ContextWrite},
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
                log::info!(
                    "ShearForce qnt:{} result:{}\n",
                    result.len(),
                    result
                        .iter()
                        .fold(String::new(), |s, v| s + &format!("{:.3} ", v))
                );
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
