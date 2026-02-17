use crate::algorithm::eval::criterion::*;
use crate::algorithm::eval::stability::*;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        eval::zg::Zg,
    }, kernel::{Eval, types::eval_result::EvalResult}, prelude::*,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет угла, соответствующий максимуму диаграммы статической остойчивости
pub struct DSOAngleMaxEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl DSOAngleMaxEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "DSOAngleMaxEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for DSOAngleMaxEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let mut results = Vec::new();
                let lever_diagram: LeverDiagramCtx = ctx.read();
                let initial: &InitialCtx = ctx.read_ref();
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .ok_or(error.err("ship_parameters error: no data!"))?;
                let breadth = *ship_parameters
                    .get("MouldedBreadth")
                    .ok_or(error.err("breadth error: no data!"))?;
                let moulded_depth = *ship_parameters
                    .get("Moulded depth")
                    .ok_or(error.err("moulded_depth error: no data!"))?;
                let wheather: WheatherCtx = ctx.read();
                let metacentric_height: MetacentricHeightCtx = ctx.read();
                let k = if let Some(error_message) = wheather.data.error_message {
                    let error = error.pass_with("no wheather k: {}", error_message);
                    log::error!("{error}");
                    results.push(CriterionData::new_error(
                        CriterionID::HeelMaximumLC,
                        error.to_string(),
                    ));
                    let result = DSOAngleMaxCtx { data: results };
                    return ctx.write(result);
                } else {
                    wheather.data.result
                };
                let angles = lever_diagram.max_angles();
                let b_div_d = breadth / moulded_depth;
                let mut target = 30.;
                if b_div_d > 2. {
                    target -= (40. * (b_div_d.min(2.5) - 2.) * (k.min(1.5) - 1.) * 0.5).round();
                }
                if let Some(angle) = angles.first() {
                    if b_div_d > 2.5 {
                        target = 15.;
                        let src_area = metacentric_height.h_trans_fix;
                        let target_area = if angle.0 <= 15.0 {
                            0.07
                        } else if angle.0 >= 30.0 {
                            0.055
                        } else {
                            0.05 + 0.001 * (30.0 - angle.0)
                        };
                        results.push(CriterionData::new_result(
                            CriterionID::AreaLc0Thetalmax,
                            src_area,
                            target_area,
                        ));
                    } else if angles.len() > 1 {
                        results.push(CriterionData::new_result(
                            CriterionID::HeelFirstMaximumLC,
                            angle.0,
                            25.,
                        ));
                    }
                    results.push(CriterionData::new_result(
                        CriterionID::HeelMaximumLC,
                        angle.0,
                        target,
                    ));
                    log::info!(
                        "Criterion DSOAngleMax breadth:{:.3} moulded_depth:{:.3} b_div_d:{:.3} angle:{:.3} target:{:.3}",
                        breadth, moulded_depth, b_div_d, results[0].result, results[0].target
                    );
                } else {
                    let error = error.err("no angle for first maximum lever of DSO!");
                    log::error!("{error}");
                    results.push(CriterionData::new_error(
                        CriterionID::HeelMaximumLC,
                        "Нет угла соответствующего максимуму DSO для текущих условий".to_owned(),
                    ));
                }
                let result = DSOAngleMaxCtx { data: results };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for DSOAngleMaxEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DSOAngleMaxEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
