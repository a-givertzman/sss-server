use super::lever_diagram_ctx::LeverDiagramCtx;
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
pub struct LeverDiagramEval {
    dbg: Dbg,
    model: ModelLink,
    value: Option<LeverDiagramCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl LeverDiagramEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        model: ModelLink,
        ctx: impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "LeverDiagramEval");
        Self {
            dbg,
            model,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for LeverDiagramEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let pantocaren = match self.model.pantocaren() {
                    Ok(data) => data,
                    Err(err) => {
                        return CtxResult::Err(error.pass_with("Read pantocaren error", err));
                    }
                };


                    /// Нагрузка на корпус судна: конструкции, груз, экипаж и т.п.
    ship_moment: Rc<dyn IShipMoment>,
    /// Отстояние центра величины погруженной части судна
    center_draught_shift: Position,
    /// Кривая плечей остойчивости формы для разных осадок
    pantocaren: Rc<dyn IPantocaren>,
    /// Продольная и поперечная исправленная метацентрическая высота.
    metacentric_height: Rc<dyn IMetacentricHeight>,
    /// Набор результатов расчетов для записи в БД
    parameters: Rc<dyn IParameters>,
    /// Максимальный угол при расчетах
    max_angle_calc: f64,

                let result = LeverDiagramCtx {
                    area_v,
                    moment_v,
                    moment_h,
                    moment_timber_h,
                    delta_moment_timber_h,
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
impl std::fmt::Debug for LeverDiagramEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LeverDiagramEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
