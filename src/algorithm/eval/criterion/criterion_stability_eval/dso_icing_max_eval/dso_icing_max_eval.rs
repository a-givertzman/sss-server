use super::dso_icing_max_ctx::DSOIcingMaxCtx;
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
/// Расчет максимума диаграммы статической остойчивости с учетом обледенения
pub struct DSOIcingMaxEval {
    dbg: Dbg,
    value: Option<DSOIcingMaxCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl DSOIcingMaxEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "DSOIcingMaxEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for DSOIcingMaxEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let lever_diagram: LeverDiagramCtx = ctx.read();
                let target = 0.20;
                let data  = match lever_diagram.dso_lever_max(25., 90.) {
                    Ok(result) => CriterionData::new_result(CriterionID::MaximumLcIcing, result, target),
                    Err(err) => {
                            let error = error.pass_with("lever_diagram.dso_lever_max", err);
                            log::error!("DSOIcingMaxEval eval error: {}", error);
                            CriterionData::new_error(
                            CriterionID::MaximumLcIcing,
                            "Ошибка вычисления максимального плеча диаграммы статической остойчивости в расчете максимума диаграммы статической остойчивости с учетом обледенения: ".to_owned() + &error.to_string(),
                        )
                    }
                };
                let result = DSOIcingMaxCtx { data };
                self.value = Some(result.clone());
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for DSOIcingMaxEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DSOIcingMaxEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
