use super::dso_timber_max_ctx::DSOTimberMaxCtx;
use crate::{
    algorithm::{
        context::context_access::ContextRead,
        eval::{
            CriterionData, CriterionID, LeverDiagramCtx
        },
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, ContextWrite
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет максимума диаграммы статической остойчивости для лесовозов
pub struct DSOTimberMaxEval {
    dbg: Dbg,
    value: Option<DSOTimberMaxCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl DSOTimberMaxEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "DSOTimberMaxEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for DSOTimberMaxEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let lever_diagram: LeverDiagramCtx = ctx.read();
                let target = 0.25;
                let data  = match lever_diagram.dso_lever_max(0., 90.) {
                    Ok(result) => CriterionData::new_result(CriterionID::MaximumLcTimber, result, target),
                    Err(err) => {
                            let error = error.pass_with("lever_diagram.dso_lever_max", err);
                            log::error!("DSOTimberMaxEval eval error: {}", error);
                            CriterionData::new_error(
                            CriterionID::MaximumLcTimber,
                            "Ошибка вычисления максимального плеча диаграммы статической остойчивости в расчете максимума диаграммы статической остойчивости для лесовозов: ".to_owned() + &error.to_string(),
                        )
                    }
                };
                let result = DSOTimberMaxCtx { data };
                self.value = Some(result.clone());
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for DSOTimberMaxEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DSOTimberMaxEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
