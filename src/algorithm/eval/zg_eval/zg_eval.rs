use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::scheduler::Scheduler;
use crate::{
    algorithm::{context::context_access::{ContextParamsRead, ContextRead}, eval::{metacentric_height_eval::metacentric_height_ctx, MetacentricHeightCtx}}, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::{Context, ContextWrite}, CtxResult
};

use super::{zg::Zg, zg_ctx::ZgCtx};

///
/// Расчет равновесного положения судна
pub struct ZgEval {
    dbg: Dbg,
    scheduler: Scheduler,   
    model: ModelLink, 
    ctx_before: Box<dyn Eval<(), EvalResult>>,
    ctx_after: impl Fn(Context) -> impl Eval<(), EvalResult> + 'static,
}
//
//
impl ZgEval {
    ///
    pub fn new(
        scheduler: Scheduler,
        model: ModelLink,
        parent: impl Into<String>,
        ctx_before: impl Eval<(), EvalResult> + 'static,
        ctx_after: impl Fn(Context) -> impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "ZgEval");
        Self {
            dbg,
            scheduler,    
            model,        
            ctx_before: Box::new(ctx_before),
            ctx_after: Box::new(ctx_after),
        }
    }
}
//
impl Eval<(), EvalResult> for ZgEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        let ctx = match self.ctx_before.eval(()) {
            CtxResult::Ok(ctx_before) => {
                let mut tasks: Vec<JoinHandle<Context>> = vec![];
                let mut criterions = vec![];

                let mut base_ctx = Arc::<Context>::new_uninit(); 
                let base_task = self.scheduler.spawn(|| {
                    let ctx = self.ctx_after(ctx_before).eval()?;
                    Arc::get_mut(&mut base_ctx).unwrap().write(ctx);
                    Ok(())
                });

                let mut zg_ctx = Vec::new();
                for zg in vec![0.0f64] {
                    let mut current_ctx = Arc::<Context>::new_uninit(); 
                    ctx_before.write_params(ParameterID::CenterMassZFix, );
                    let task = self.scheduler.spawn(|| {
                        let ctx = self.ctx_after.eval(())?;
                        Arc::get_mut(&mut current_ctx).unwrap().write(ctx);
                        Ok(())
                    });
                    zg_ctx.push((zg, current_ctx));
                    tasks.push(task);
                }

                *Arc::try_unwrap(x).unwrap_err()
                

       /*       
                let base_task = self.scheduler.spawn(|| {
                    match self.ctx_zg.eval(Some((Zg(zg), ctx.clone()))) {
                        CtxResult::Ok(ctx) => {
                            CtxResult::Ok(ctx)
                        }
                        CtxResult::Err(err) => CtxResult::Err(error.pass_with("Read context error", err)),
                        CtxResult::None => CtxResult::None,
                    }
                });
                for zg in vec![0.0f64] {
                    let task = self.scheduler.spawn(|| {
                        match self.ctx_zg.eval(Some((Zg(zg), ctx.clone()))) {
                            CtxResult::Ok(ctx) => {
                                CtxResult::Ok(ctx)
                            }
                            CtxResult::Err(err) => CtxResult::Err(error.pass_with("Read context error", err)),
                            CtxResult::None => CtxResult::None,
                        }
                    });
                    tasks.push(task);
                }
                let ctx = base_task.join();
                for task in tasks {
                    let ctx_zg: Context = task.join();
                    let criterion_ctx: CriterionCtx = ctx_zg.read();
                    criterions.push(criterion_ctx);
                }
                
                */

                let five = unsafe { five.assume_init() };

                let result = ZgCtx {

                };
                ctx_before.write(result)
            }
            CtxResult::Err(err) => CtxResult::Err(error.pass_with("Read context error", err)),
            CtxResult::None => CtxResult::None,
        };
    }
}
//
//
impl std::fmt::Debug for ZgEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZgEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
