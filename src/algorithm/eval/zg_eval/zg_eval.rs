use std::{collections::HashMap, sync::Arc};

use crate::{
    algorithm::{
        context::context_access::{ContextParamsRead, ContextParamsWrite, ContextRead, ContextReadRef},
        eval::*,
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::{Context, ContextWrite, InitialCtx}, ship_model::ship_model::ShipModel, CtxResult
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::{JoinHandle, scheduler::Scheduler};

use super::zg_ctx::ZgCtx;

type Before = StabilityAreaEval;
//type After = Fn(Context, Option<f64>) -> EvalResult + 'static;

unsafe impl Send for Before {}
unsafe impl Sync for Before {} 

//unsafe impl Send for after {}
//unsafe impl Sync for after {} 

///
/// Расчет равновесного положения судна
pub struct ZgEval<'a> {
    dbg: Dbg,
    scheduler: Scheduler,
    ship_model: &'a ShipModel,
    ctx_before: Before,
  //  ctx_after: Box<dyn After>,
}
//
//
impl<'a> ZgEval<'a> {
    ///
    pub fn new(
        scheduler: Scheduler,
        parent: impl Into<String>,
        ship_model: &'a ShipModel,
        ctx_before: Before,
   //     ctx_after: impl After,
    ) -> Self {
        let dbg = Dbg::new(parent, "ZgEval");
        Self {
            dbg,
            scheduler,
            ship_model,
            ctx_before,
     //       ctx_after: Box::new(ctx_after),
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
              //  let z_m = ctx_before.read_params(ParameterID::MetacentricTransRadZ);
              //  let delta_m_h = ctx_before.read_params(ParameterID::MetacentricTransSum);
                // базовый контекст
                let mut base_ctx = Arc::<Context>::new_uninit();
                let base_task = {
                    let dbg = self.dbg.clone();
                    let link= self.ship_model.link();
                    let ctx = ctx_before.clone();
                    self
                    .scheduler
                    .spawn(move || {
                            let ctx = MetacentricHeightEval::new(
                                &dbg,
                                None,
                                DSOMaxEval::new(
                                    &dbg,
                                    CriterionStabilityEval::new(
                                        &dbg,
                                        DSOAreaEval::new(
                                            &dbg,
                                            StaticAngleEval::new(
                                                &dbg,
                                                WheatherEval::new(
                                                    &dbg,
                                                    RollingAmplitudeEval::new(
                                                        &dbg,
                                                        RollingPeriodEval::new(
                                                            &dbg,
                                                            WindEval::new(
                                                                &dbg,
                                                                WindageEval::new(
                                                                    &dbg,
                                                                    LeverDiagramEval::new(&dbg, link, ctx),
                                                                ),
                                                            ),
                                                        ),
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            ).eval(())?;                    
                        base_ctx.write(ctx);
                    //   base_ctx.write((self.ctx_after)(ctx_before, None)?);
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
                    let link= self.ship_model.link();
                    let ctx = ctx_before.clone();               
                    let mut criterion = Arc::<CriterionStabilityCtx>::new_uninit();
                    let task = self
                        .scheduler
                        .spawn( move || {
                            let ctx = MetacentricHeightEval::new(
                                &dbg,
                                Some(z_g_fix),
                                DSOMaxEval::new(
                                    &dbg,
                                    CriterionStabilityEval::new(
                                        &dbg,
                                        DSOAreaEval::new(
                                            &dbg,
                                            StaticAngleEval::new(
                                                &dbg,
                                                WheatherEval::new(
                                                    &dbg,
                                                    RollingAmplitudeEval::new(
                                                        &dbg,
                                                        RollingPeriodEval::new(
                                                            &dbg,
                                                            WindEval::new(
                                                                &dbg,
                                                                WindageEval::new(
                                                                    &dbg,
                                                                    LeverDiagramEval::new(&dbg, link, ctx),
                                                                ),
                                                            ),
                                                        ),
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            ).eval(())?;       
                            *criterion.write(ctx.read());
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
                base_ctx.clone().write(result)
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
