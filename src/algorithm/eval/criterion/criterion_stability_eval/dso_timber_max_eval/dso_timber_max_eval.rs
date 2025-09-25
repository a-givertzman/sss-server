use super::dso_timber_max_ctx::DSOTimberMaxCtx;
use crate::{
    prelude::*,
    algorithm::eval::{CriterionData, CriterionID, LeverDiagramCtx, zg_eval::Zg},
    kernel::{eval::Eval, types::eval_result::EvalResult},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет максимума диаграммы статической остойчивости для лесовозов
pub struct DSOTimberMaxEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl DSOTimberMaxEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "DSOTimberMaxEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for DSOTimberMaxEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let lever_diagram: LeverDiagramCtx = ctx.read();
                let target = 0.25;
                let data = match lever_diagram.dso_lever_max(0., 90.) {
                    Ok(result) => {
                        CriterionData::new_result(CriterionID::MaximumLcTimber, result, target)
                    }
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
            .finish()
    }
}
