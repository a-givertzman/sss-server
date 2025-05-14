use super::dso_icing_max_ctx::DSOIcingMaxCtx;
use crate::{
    algorithm::{
        context::context_access::ContextRead,
        eval::{
            zg_eval::Zg, CriterionData, CriterionID, LeverDiagramCtx
        },
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, ContextWrite
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет максимума диаграммы статической остойчивости с учетом обледенения
pub struct DSOIcingMaxEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult>>,
}
//
//
impl DSOIcingMaxEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "DSOIcingMaxEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for DSOIcingMaxEval {
    fn eval(&mut self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
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
            .finish()
    }
}
