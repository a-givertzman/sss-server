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

use super::strength_mass_ctx::StrengthMassCtx;

///
/// Площади боковой и горизонтальной поверхностей для расчета прочности
pub struct StrengthMassEval {
    dbg: DbgId,
    model: ModelLink,
    value: Option<StrengthMassCtx>,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl StrengthMassEval {
    ///
    /// Fetches all initiall data
    /// - 'api_client' - access to the database
    pub fn new(
        parent: impl Into<String>,
        model: ModelLink,
        ctx: impl Eval<(), EvalResult> + Send + 'static,
    ) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "StrengthMassEval");
        Self {
            dbg,
            model,
            value: None,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for StrengthMassEval {
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
                    };
                    let (const_area_v, const_area_h) = match self.model.bound_areas().await {
                        Ok((area_v, area_h)) => (area_v, area_h),
                        Err(err) => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read bound_areas error: {:?}",
                                self.dbg, err
                            )));
                        }
                    };
                    
                    let result = StrengthAreaCtx {
                        area_v_array: area_v,
                        area_h: const_area_h,
                        area_timber_h,
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
impl std::fmt::Debug for MassEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MassEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
