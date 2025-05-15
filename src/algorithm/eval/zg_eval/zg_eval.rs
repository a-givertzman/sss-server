use std::{
    collections::HashMap,
};
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        eval::*,
    }, kernel::{eval::Eval, types::{eval_result::EvalResult, Arc, RwLock}}, prelude::{Context, ContextWrite, InitialCtx}
};
use coco::Stack;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::{JoinHandle, Scheduler};

use super::{zg_ctx::ZgCtx, Zg};

// unsafe impl Send for StabilityAreaEval {}
// unsafe impl Sync for StabilityAreaEval {}

///
/// Расчет равновесного положения судна
pub struct ZgEval {
    dbg: Dbg,
    scheduler: Scheduler,
 //   ship_model: &'a ShipModel,
    ctx: Arc<Box<dyn Eval<Zg, EvalResult> + Send + Sync>>,
}
//
//
impl ZgEval {
    ///
    pub fn new(
        scheduler: Scheduler,
        parent: impl Into<String>,
      //  ship_model: &'a ShipModel,
        ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "ZgEval");
        Self {
            dbg,
            scheduler,
         //   ship_model,
            ctx: Arc::new(Box::new(ctx)),
        }
    }
}
//
impl Eval<(), EvalResult> for ZgEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(Zg::empty()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .expect("ZgEval eval error: no ship_parameters");
                let overall_height = *ship_parameters
                    .get("Overall height up to non-removable parts")
                    .ok_or(error.err("No LBP in ship_parameters"))?;
                // базовый контекст
                // перебор значений z_g_fix, вычисление контекста для zg
                let mut tasks: Vec<JoinHandle<()>> = vec![];
                let zg_results: Arc<Stack<(f64, _)>> = Arc::new(Stack::new());
                let delta = 0.1;
                let max_index = (overall_height / delta).floor() as i32;
                for index in 0..max_index {
                    let z_g_fix = index as f64 * delta;
                    let dbg = self.dbg.clone();
                //    let link = self.ship_model.link();
                    let ctx_ = ctx.clone();
                    // let moved_criterion = criterion.clone();
                    let zg_results_ = zg_results.clone();
                    let self_ctx = self.ctx.clone();
                    let task = self
                        .scheduler
                        .spawn(move || {
                            let ctx = self_ctx.eval(Zg(z_g_fix))?;
                            // let criterion = Arc::new(Mutex::new(Option::<CriterionStabilityCtx>::None));
                            let criterion: CriterionStabilityCtx = ctx.read();
                            zg_results_.push((z_g_fix, criterion));
                            Ok(())
                        })
                        .map_err(|err| error.pass_with(format!("task {}", z_g_fix), err))?;
                    tasks.push(task);
                }
                // получаем массив рассчитанных критериев для разных zg
                for task in tasks {
                    // TODO try to handle errors
                    task.join().unwrap();
                }
                let mut results = Vec::new(); //<(f64, Vec<(usize, Option<f64>)>)>'
                
                let mut zg_criterion: Vec<(f64, _)> = vec![];
                while !zg_results.is_empty() {
                    if let Some(r) = zg_results.pop() {
                        zg_criterion.push(r);
                    }
                }
                for (z_g_fix, criterion) in zg_criterion {
                    // отбрасываем ошибки, оставляем только значения, считаем дельту с целевым значением
                    //    let criterion = unsafe { &*criterion.assume_init() };
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
                            .expect("ZgEval eval values error: sort values!")
                    });
                    // берем первое значение как ближайшее значение к целевому
                    let closest_value = values
                        .first()
                        .expect("ZgEval eval closest_value error, no values!");
                    result.insert(id, closest_value.0);
                }
                let result = ZgCtx { zg: result };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("self.ctx.eval error", err)),
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
