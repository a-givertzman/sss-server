use sal_sync::services::entity::error::str_err::StrErr;
use crate::{
    algorithm::{context::context_access::ContextReadRef, entities::data::loads::UnitCargoType}, kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ship_model::model_link::ModelLink, ContextWrite, CtxResult
};
use super::stability_area_ctx::StabilityAreaCtx;
///
/// Площади боковой и горизонтальной поверхностей для расчета остойчивости
pub struct StabilityAreaEval {
    dbg: DbgId,
    model: ModelLink,
    value: Option<AreaCtx>,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl StabilityAreaEval {
    ///
    pub fn new(parent: impl Into<String>, model: ModelLink, ctx: impl Eval<(), EvalResult> + Send + 'static) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "Area");
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
impl Eval<(), EvalResult> for AreaEval {
    fn eval(&mut self, _: ()) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            match self.ctx.eval(()).await {
                CtxResult::Ok(ctx) => {
                    let initial: &InitialCtx = ctx.read_ref();
                    let unit = match initial.unit.clone() {
                        Some(data) => data.data(),
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read unit error: no data!",
                                self.dbg
                            )))
                        }
                    };                    
                    let (const_area_v, const_area_h) = match self.model.areas().await {
                        Ok((const_area_v, const_area_h)) => {
                            (const_area_v, const_area_h)
                        },                        
                        Err(err) => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read areas error: {:?}",
                                self.dbg, err
                            )));
                        },
                    };
                    let area_timber_h = 
                        unit.iter()
                            .filter(|v| v.cargo_type == UnitCargoType::Timber && v.icing_area.is_some() && v.centre_of_icing_area.is_some())
                            .map(|v| (v.icing_area.unwrap(), v.centre_of_icing_area.unwrap())).collect();
                    let result = AreaCtx {
                        const_area_v,
                        const_area_h,
                        area_timber_h,
                    };
                    self.value = Some(result.clone());
                    ctx.write(result)                    
                },
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
impl std::fmt::Debug for StabilityAreaEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StabilityAreaEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}