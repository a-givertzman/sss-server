use super::Zg;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        eval::*,
    },
    kernel::{
        Eval,
        types::{Arc, eval_result::EvalResult},
    },
    prelude::{ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    sync::Stack,
    thread_pool::{JoinHandle, Scheduler, ThreadPool},
};
use std::collections::{HashMap, VecDeque};

// unsafe impl Send for StabilityAreaEval {}
// unsafe impl Sync for StabilityAreaEval {}

///
/// Расчет равновесного положения судна
pub struct ZgEval {
    dbg: Dbg,
    thread_pool: Arc<ThreadPool>,
    //   ship_model: &'a ShipModel,
    ctx: Arc<Box<dyn Eval<Zg, EvalResult> + Send + Sync>>,
}
//
//
impl ZgEval {
    ///
    pub fn new(
        thread_pool: Arc<ThreadPool>,
        parent: impl Into<String>,
        //  ship_model: &'a ShipModel,
        ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "ZgEval");
        Self {
            dbg,
            thread_pool,
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
                let mut tasks: VecDeque<JoinHandle<_>> = VecDeque::new();
                let mut errors = Vec::new();
                let mut pass = |message: &str, err: Error| {
                    let error = error.pass_with(message, err);
                    log::error!("{:?}", &error);
                    errors.push(error);
                };
                let results = Arc::new(Stack::new());
                let delta = 0.1;
                let max_index = (overall_height / delta).floor() as i32;
                let scheduler = self.thread_pool.scheduler();
                for index in 0..=max_index {
                    let z_g_fix = index as f64 * delta;
                    let results_ = results.clone();
                    let self_ctx = self.ctx.clone();
                    let thread_name = format!("ZgEval z_g_fix {:.3}", z_g_fix);
                    log::trace!("{}.build | Starting thread {thread_name}", &self.dbg);
                    //  println!("Starting thread {thread_name}");
                    let handle = scheduler
                        .spawn_named(thread_name, move || {
                            let ctx = self_ctx.eval(Zg(Some(z_g_fix)))?;
                            // let criterion = Arc::new(Mutex::new(Option::<CriterionStabilityCtx>::None));
                            let criterion: CriterionStabilityCtx = ctx.read();
                            results_.push((z_g_fix, criterion));
                            Ok(())
                        })
                        .map_err(|err| error.pass_with(format!("ZgEval z_g_fix:{}", z_g_fix), err));
                    match handle {
                        Ok(task) => tasks.push_back(task),
                        Err(err) => pass("task handle", err),
                    };
                }
                // получаем массив рассчитанных критериев для разных zg
                for task in tasks {
                    log::trace!("{}.eval | join thread {}", &self.dbg, task.name());
                    if let Err(err) = task.join() {
                        pass("task join", err);
                    }
                }
                let mut zg_criterion: Vec<(f64, _)> = vec![];
                while !results.is_empty() {
                    if let Some(r) = results.pop() {
                        zg_criterion.push(r);
                    }
                }
                let mut vec_results = Vec::new();
                for (z_g_fix, criterion) in zg_criterion {
                    // отбрасываем ошибки, оставляем только значения, считаем дельту с целевым значением
                    //    let criterion = unsafe { &*criterion.assume_init() };
                    let tmp: Vec<(usize, Option<(f64, f64)>)> = criterion
                        .data
                        .iter()
                        .filter(|v| v.error_message.is_none())
                        .map(|v| (v.criterion_id, Some((v.result, v.target))))
                        .collect();
                    vec_results.push((z_g_fix, tmp));
                }
                // создаем коллекцию векторов, сортируем значения по id
                #[allow(clippy::type_complexity)]
                let mut values: HashMap<usize, Vec<(f64, (f64, f64))>> = HashMap::new();
                for (z_g_fix, tmp) in vec_results.into_iter() {
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
