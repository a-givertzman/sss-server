use sal_core::{dbg::Dbg, error::Error};

use crate::{
    algorithm::{
        context::context_access::ContextRead,
        entities::Moment,
        eval::{IcingCtx, LoadsCtx, WettingCtx},
    },
    kernel::{eval::Eval, sync::Link, types::eval_result::EvalResult},
    ship_model::query::{BalanceQuery, Query},
    ContextWrite, CtxResult,
};
use super::balance_ctx::BalanceCtx;

///
/// Расчет равновесного положения судна
pub struct BalanceEval {
    dbg: Dbg,
    model: Link,
    value: Option<BalanceCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl BalanceEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        model: Link,
        ctx: impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "BalanceEval");
        Self {
            dbg,
            model,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
impl Eval<(), EvalResult> for BalanceEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let loads: LoadsCtx = ctx.read();
                let icing: IcingCtx = ctx.read();
                let wetting: WettingCtx = ctx.read();
                // Суммарная масса корпуса, всех грузов и обледенения с намоканием
                let mass_sum = loads.mass_const
                    + loads.mass_unit
                    + loads.mass_bulk
                    + loads.mass_gaseous
                    + loads.mass_liquid
                    + icing.mass
                    + wetting.mass;
                // Сумарный момент за вычетом смещяемых и насыпных груов
                let moment_const = Moment::from_pos(loads.shift_const, loads.mass_const) +
                    Moment::from_pos(loads.shift_unit, loads.mass_unit) + 
                    Moment::from_pos(loads.shift_gaseous, loads.mass_gaseous) + 
                    Moment::new(icing.mass*icing.mass_shift_x, 0., 0.) + 
                    Moment::from_pos(wetting.mass_shift, wetting.mass);    
                // Структура для передачи в модель
                let balance_query = BalanceQuery {
                    mass_sum,
                    moment_const,
                    bulk: loads.bulk.clone(),
                    liquid: loads.liquid.clone(),
                    grain_bulkhead: loads.grain_bulkhead,
                };
                // Расчет баланса в модели
                let result: BalanceCtx = match self.model.call(Query::ComputeBalance(balance_query)) {
                    Ok(data) => data,
                    Err(err) => {
                        return CtxResult::Err(error.pass_with("model.compute_balance error", err));
                    }
                };
                //
                // TODO Propably additional BalanceResult is not required, sorry if not
                //
                // let result = BalanceCtx {
                //     parameters: result.parameters,
                //     bulk: result.bulk,
                //     liquid: result.liquid,
                // };
                self.value = Some(result.clone());
                ctx.write(result)
            }
            CtxResult::Err(err) => CtxResult::Err(error.pass_with("Read context error", err)),
            CtxResult::None => CtxResult::None,
        }
    }
}
//
//
impl std::fmt::Debug for BalanceEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BalanceEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
