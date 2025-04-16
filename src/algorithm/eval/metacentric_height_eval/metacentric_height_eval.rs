use super::metacentric_height_ctx::MetacentricHeightCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::{data::loads::UnitCargoType, Bound, Moment, Position},
        eval::IcingTimberCtx,
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ship_model::model_link::{IModelLink, ModelLink}, ContextWrite, CtxResult
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Диаграмма плеч статической и динамической остойчивости
pub struct MetacentricHeightEval {
    dbg: Dbg,
    value: Option<MetacentricHeightCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl MetacentricHeightEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "MetacentricHeightEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for MetacentricHeightEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();


    /// * center_draught_shift - Отстояние центра величины погруженной части судна       
    /// * rad_long Продольный - метацентрические радиус
    /// * rad_trans - Поперечный метацентрические радиус
    /// * tanks - Все жидкие грузы судна
    /// * mass - Все грузы судна
    /// * moment - Момент массы судна
    /// * parameters - Набор результатов расчетов для записи в БД

                let result = MetacentricHeightCtx {
                    h_trans_0: todo!(),
                    h_long_fix: todo!(),
                    h_trans_fix: todo!(),
                    z_g_fix: todo!(),
                    delta_m_h: todo!(),
                };
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
impl std::fmt::Debug for MetacentricHeightEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetacentricHeightEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
