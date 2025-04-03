use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::Moment,
        eval::{IcingCtx, LoadsCtx, WettingCtx},
    },
    kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ship_model::{model_link::ModelLink, query::BalanceSrcData},
    ContextWrite, CtxResult,
};
use sal_sync::services::entity::error::str_err::StrErr;
use super::balance_ctx::BalanceCtx;

///
/// Расчет равновесного положения судна
pub struct BalanceEval {
    dbg: DbgId,
    model: ModelLink,
    value: Option<BalanceCtx>,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl BalanceEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        model: ModelLink,
        ctx: impl Eval<(), EvalResult> + Send + 'static,
    ) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "BalanceEval");
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
    fn eval(&mut self, _: ()) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            match self.ctx.eval(()).await {
                CtxResult::Ok(ctx) => {
                    let initial: &InitialCtx = ctx.read_ref();
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
                    let balance_src_data = BalanceSrcData {
                        mass_sum,
                        moment_const,
                        bulk: loads.bulk.clone(),
                        liquid: loads.liquid.clone(),
                        grain_bulkhead: loads.grain_bulkhead,
                    };
                    // Расчет баланса в модели
                    let result = match self.model.compute_balance(balance_src_data).await {
                        Ok(data) => data,
                        Err(err) => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | model.compute_balance error: {:?}",
                                self.dbg, err
                            )));
                        }
                    };                    
                    let result = BalanceCtx {
                        parameters: result.parameters,
                        bulk: result.bulk,
                        liquid: result.liquid,
                    };
                    self.value = Some(result.clone());
                    ctx.write(result)
                }
                CtxResult::Err(err) => CtxResult::Err(StrErr(format!(
                    "{}.eval | Read context error: {:?}",
                    self.dbg, err
                ))),
                CtxResult::None => CtxResult::None,
            }
        })
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
