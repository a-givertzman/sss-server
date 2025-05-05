use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        eval::*,
    }, kernel::{eval::Eval, sync::Link, types::eval_result::EvalResult}, prelude::{Context, ContextWrite, InitialCtx}, ship_model::ship_model::ShipModel, CtxResult
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::{JoinHandle, scheduler::Scheduler};

use super::zg_ctx::ZgCtx;

unsafe impl Send for StabilityAreaEval {}
unsafe impl Sync for StabilityAreaEval {}

///
/// Расчет равновесного положения судна
pub struct ZgEval<'a> {
    dbg: Dbg,
    scheduler: Scheduler,
    ship_model: &'a ShipModel,
    ctx_before: StabilityAreaEval,
    ctx_after: fn(Dbg, Option<f64>, Link, Context) -> MetacentricHeightEval,
}
//
//
impl<'a> ZgEval<'a> {
    ///
    pub fn new(
        scheduler: Scheduler,
        parent: impl Into<String>,
        ship_model: &'a ShipModel,
        ctx_before: StabilityAreaEval,
        ctx_after: fn(Dbg, Option<f64>, Link, Context) -> MetacentricHeightEval,
    ) -> Self {
        let dbg = Dbg::new(parent, "ZgEval");
        Self {
            dbg,
            scheduler,
            ship_model,
            ctx_before,
            ctx_after,
        }
    }
}
//
impl<'a> Eval<(), EvalResult> for ZgEval<'a> {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx_before.eval(()) {
            CtxResult::Ok(ctx_before) => {
                let initial: &InitialCtx = ctx_before.read_ref();
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .expect("ZgEval eval error: no ship_parameters");
                let overall_height = *ship_parameters
                    .get("Overall height up to non-removable parts")
                    .ok_or(error.err("No LBP in ship_parameters"))?;
                // базовый контекст
                let base_ctx = Arc::new(Mutex::new(Option::<Context>::None));
                let base_task = {
                //    let temp_fn: fn(Dbg, Option<f64>, Link, Context) -> MetacentricHeightEval = create_after_ctx;
                    let dbg = self.dbg.clone();
                    let link = self.ship_model.link();
                    let ctx = ctx_before.clone();
                    let base_ctx = base_ctx.clone();
                    let ctx_after = self.ctx_after.clone();
                    self.scheduler
                        .spawn(move || {
                            let ctx = (ctx_after)(dbg, None, link, ctx).eval(())?;
                            let mut base_ctx = base_ctx.lock().unwrap();
                            *base_ctx = Some(ctx);
                            Ok(())
                        })
                        .map_err(|err| error.pass_with(format!("base_task"), err))?
                };
                // перебор значений z_g_fix, вычисление контекста для zg
                let mut tasks: Vec<JoinHandle<()>> = vec![];
                let mut zg_criterion: Vec<(f64, _)> = Vec::new();
                let delta = 0.1;
                let max_index = (overall_height / delta).floor() as i32;
                for index in 0..max_index {
                    let z_g_fix = index as f64 * delta;
                    let dbg = self.dbg.clone();
                    let link = self.ship_model.link();
                    let ctx = ctx_before.clone();
                    let criterion = Arc::new(Mutex::new(Option::<CriterionStabilityCtx>::None));
                    let moved_criterion = criterion.clone();
                    let ctx_after = self.ctx_after.clone();
                    let task = self
                        .scheduler
                        .spawn(move || {
                            let ctx = (ctx_after)(dbg, Some(z_g_fix), link, ctx).eval(())?;
                            let mut criterion = moved_criterion.lock().unwrap();
                            *criterion = Some(ctx.read());
                            // criterion.clone().write(ctx.read());
                            Ok(())
                        })
                        .map_err(|err| error.pass_with(format!("task {}", z_g_fix), err))?;
                    zg_criterion.push((z_g_fix, criterion));
                    tasks.push(task);
                }
                // получаем базовый контекст
                base_task.join()?;
                let base_ctx = base_ctx.lock().unwrap().clone().unwrap();
                // получаем массив рассчитанных критериев для разных zg
                for task in tasks {
                    task.join();
                }
                let mut results = Vec::new(); //<(f64, Vec<(usize, Option<f64>)>)>'
                for (z_g_fix, criterion) in zg_criterion {
                    // отбрасываем ошибки, оставляем только значения, считаем дельту с целевым значением
                    //    let criterion = unsafe { &*criterion.assume_init() };
                    let criterion = criterion.lock().unwrap().clone().unwrap();
                    let tmp: Vec<(usize, Option<(f64, f64)>)> = criterion
                        .data
                        .iter()
                        .map(|v| {
                            let delta = if v.error_message.is_none() {
                                Some((v.result, v.target))
                            } else {
                                None
                            };
                            (v.criterion_id, delta)
                        })
                        .collect();
                    results.push((z_g_fix, tmp));
                }
                // создаем коллекцию векторов, сортируем значения по id
                #[allow(clippy::type_complexity)]
                let mut values: HashMap<usize, Vec<(f64, (f64, f64))>> = HashMap::new();
                for (z_g_fix, tmp) in results.into_iter() {
                    tmp.into_iter()
                        .filter(|(_, value)| value.is_some())
                        .for_each(|(id, value)| {
                            values
                                .entry(id)
                                .and_modify(|v| v.push((z_g_fix, value.unwrap())))
                                .or_insert(vec![(z_g_fix, value.unwrap())]);
                        });
                }
                let mut result = HashMap::new();
                for (id, mut values) in values.into_iter() {
                    // сортируем значения по увеличению дельты с целевым
                    values.sort_by(|&(_, v1), &(_, v2)| {
                        (v1.0 - v1.1)
                            .abs()
                            .partial_cmp(&(v2.0 - v2.1).abs())
                            .expect("CriterionComputer calculate error: sort values!")
                    });
                    // берем первое значение как ближайшее значение к целевому
                    let closest_value = values
                        .first()
                        .expect("CriterionComputer calculate error, no values!");
                    result.insert(id, closest_value.0);
                }
                let result = ZgCtx { zg: result };
                base_ctx.write(result)
            }
            CtxResult::Err(err) => CtxResult::Err(error.pass_with("Read context error", err)),
            CtxResult::None => CtxResult::None,
        }
    }
}
//
//
impl<'a> std::fmt::Debug for ZgEval<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZgEval").field("dbg", &self.dbg).finish()
    }
}

