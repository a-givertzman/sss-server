use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::scheduler::Scheduler;
use crate::{
    kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::Context, CtxResult
};

use super::zg::Zg;

///
/// Расчет равновесного положения судна
pub struct ZgEval {
    dbg: Dbg,
    tasks: Vec<Box<dyn Eval<(), EvalResult>>>,
    ctx_zg: Box<dyn Eval<Option<(Zg, Context)>, EvalResult>>,
    ctx: Box<dyn Eval<(), EvalResult>>,
    scheduler: Scheduler,
}
//
//
impl ZgEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx_zg: impl Eval<Option<(Zg, Context)>, EvalResult> + 'static,
        ctx: impl Eval<(), EvalResult> + 'static,
        scheduler: Scheduler,
    ) -> Self {
        let dbg = Dbg::new(parent, "ZgEval");
        Self {
            dbg,
            tasks: vec![],
            ctx_zg: Box::new(ctx_zg),
            ctx: Box::new(ctx),
            scheduler,
        }
    }
}
//
impl Eval<Option<(Zg, Context)>, EvalResult> for ZgEval {
    fn eval(&mut self, zg: Option<(Zg, Context)>) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        let ctx = match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let mut tasks = vec![];
                let mut criterions = vec![];
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
                ctx.write(results)
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
