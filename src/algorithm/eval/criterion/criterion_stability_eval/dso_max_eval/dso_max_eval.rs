use super::dso_max_ctx::DSOMaxCtx;
use crate::algorithm::entities::math::curve::*;
use crate::algorithm::eval::{CriterionData, CriterionID};
use crate::{
    ContextWrite,
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        eval::LeverDiagramCtx,
    },
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия максимум диаграммы статической остойчивости
pub struct DSOMaxEval {
    dbg: Dbg,
    value: Option<DSOMaxCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl DSOMaxEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "DSOMaxEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for DSOMaxEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let lever_diagram: LeverDiagramCtx = ctx.read();
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .expect("DSOMaxEval eval error: no ship_parameters");
                let ship_length = *ship_parameters
                    .get("LBP")
                    .ok_or(error.err("No LBP in ship_parameters"))?;
                let data = match Curve::new_linear(&[(105., 0.20), (80., 0.25)]) {
                    Ok(curve) => match (lever_diagram.dso_lever_max(30., 90.), curve.value(ship_length)) {
                        (Ok(result), Ok(target)) => CriterionData::new_result(CriterionID::MaximumLC, result, target),
                        _ => { 
                            let error = error.err("lever_diagram.dso_lever_max + curve.value");
                            log::error!("DSOMaxEval eval error: {}", error);
                            CriterionData::new_error(
                                CriterionID::MaximumLC,
                                "Ошибка вычисления значения кривой в расчете максимума диаграммы статической остойчивости: ".to_owned() + &error.to_string(),
                            )
                        }
                    },
                    Err(err) => {
                        let error = error.pass_with("Curve::new_linear", err);
                        log::error!("DSOMaxEval eval error: {}", error);
                        CriterionData::new_error(
                            CriterionID::MaximumLC,
                            "Ошибка создания кривой в расчете максимума диаграммы статической остойчивости: ".to_owned() + &error.to_string(),
                        )
                    }
                };
                let result = DSOMaxCtx { data };
                self.value = Some(result.clone());
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for DSOMaxEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DSOMaxEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
