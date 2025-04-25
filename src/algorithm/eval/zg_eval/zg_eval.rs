use std::{collections::HashMap, sync::Arc};

use crate::{
    algorithm::{
        context::context_access::{ContextParamsRead, ContextParamsWrite, ContextRead, ContextReadRef},
        eval::{parameters::ParameterID, CriterionStabilityCtx, StabilityAreaEval},
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::{Context, ContextWrite, InitialCtx}, CtxResult
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::{JoinHandle, scheduler::Scheduler};

use super::zg_ctx::ZgCtx;

///
/// Расчет равновесного положения судна
pub struct ZgEval {
    dbg: Dbg,
    scheduler: Scheduler,
    ctx_before: StabilityAreaEval,
    ctx_after: Box<dyn Fn(Context, Option<f64>) -> EvalResult + 'static>,
}
//
//
impl ZgEval {
    ///
    pub fn new(
        scheduler: Scheduler,
        parent: impl Into<String>,
        ctx_before: StabilityAreaEval,
        ctx_after: impl Fn(Context, Option<f64>) -> EvalResult + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "ZgEval");
        Self {
            dbg,
            scheduler,
            ctx_before,
            ctx_after: Box::new(ctx_after),
        }
    }
}
//
impl Eval<(), EvalResult> for ZgEval {
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
              //  let z_m = ctx_before.read_params(ParameterID::MetacentricTransRadZ);
              //  let delta_m_h = ctx_before.read_params(ParameterID::MetacentricTransSum);
                // базовый контекст
                let mut base_ctx = Arc::<Context>::new_uninit();
                let base_task = self
                .scheduler
                .spawn(|| {
                    base_ctx.write((self.ctx_after)(ctx_before, None)?);
                    Ok(())
                })
                .map_err(|err| error.pass_with(format!("base_task"), err))?;
                // перебор значений z_g_fix
                let mut tasks: Vec<JoinHandle<()>> = vec![];
                let mut criterions = vec![];
                let mut zg_criterion: Vec<(f64, _)> = Vec::new();
                let delta = 0.1;
                let max_index = (overall_height / delta).floor() as i32;
                for index in 0..max_index {
                    let z_g_fix = index as f64 * delta;
                 //   let ctx_before = self.ctx_before.clone();
                  //  let ctx_after = self.ctx_after.clone();                    
                    let mut criterion = Arc::<CriterionStabilityCtx>::new_uninit();
                    let task = self
                        .scheduler
                        .spawn(|| {
                            let ctx = (self.ctx_after)(ctx_before,  Some(z_g_fix))?;
                            criterion.write(ctx.read());
                            Ok(())
                        })
                        .map_err(|err| error.pass_with(format!("task {}", z_g_fix), err))?;
                    zg_criterion.push((z_g_fix, criterion));
                    tasks.push(task);
                }
                // получаем базовый контекст
                base_task.join()?;
                let base_ctx = unsafe { &*base_ctx.assume_init_mut() };
                // получаем массив рассчитанных критериев для разных zg
                for task in tasks {
                    task.join();
                }
                let mut results = Vec::new(); //<(f64, Vec<(usize, Option<f64>)>)>'
                for (z_g_fix, criterion) in zg_criterion {
                    // отбрасываем ошибки, оставляем только значения, считаем дельту с целевым значением
                    let criterion = unsafe { &*criterion.assume_init() };
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
impl std::fmt::Debug for ZgEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZgEval").field("dbg", &self.dbg).finish()
    }
}
