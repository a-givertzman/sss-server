use super::dso_angle_max_ctx::DSOAngleMaxCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        eval::{
            CriterionData, CriterionID, LeverDiagramCtx
        },
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ContextWrite, CtxResult
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет угла, соответствующий максимуму диаграммы статической остойчивости
pub struct DSOAngleMaxEval {
    dbg: Dbg,
    value: Option<DSOAngleMaxCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl DSOAngleMaxEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "DSOAngleMaxEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for DSOAngleMaxEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let lever_diagram: LeverDiagramCtx = ctx.read();
                let initial: &InitialCtx = ctx.read_ref();
                let ship_parameters = initial.ship_parameters.as_ref().ok_or(error.err("ship_parameters error: no data!"))?; 
                let breadth = *ship_parameters.get("MouldedBreadth").ok_or(error.err("breadth error: no data!"))?;  

                let mut results = Vec::new();

                let angles = lever_diagram.max_angles();
                let b_div_d = breadth / self.moulded_depth;
                let mut target = 30.;
                if b_div_d > 2. {
                    let k = match self.stability.k() {
                        Ok(k) => k,
                        Err(error) => {
                            let error = Error::FromString(format!(
                                "CriterionStability dso_lever_max_angle stability.k() error: {}",
                                error
                            ));
                            log::error!("{error}");
                            results.push(CriterionData::new_error(
                                CriterionID::HeelMaximumLC,
                                error.to_string(),
                            ));
                            return results;
                        }
                    };
                    target -= (40. * (b_div_d.min(2.5) - 2.) * (k.min(1.5) - 1.) * 0.5).round();
                }
                if let Some(angle) = angles.first() {
                    if b_div_d > 2.5 {
                        target = 15.;
                        match self.metacentric_height.h_trans_fix() {
                            Ok(src_area) => {
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
                            }
                            Err(error) => {
                                let error = Error::FromString(format!(
                                    "CriterionStability dso_lever_max_angle h_trans_fix error: {}",
                                    error
                                ));
                                log::error!("{error}");
                                results.push(CriterionData::new_error(
                                CriterionID::AreaLc0Thetalmax,
                                "Ошибка вычисления поперечной исправленной метацентрической высоты в расчете угла, соответствующего максимуму диаграммы статической остойчивости: ".to_owned() + &error.to_string(),
                            ))
                            }
                        };
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
                } else {
                    results.push(CriterionData::new_error(
                        CriterionID::HeelMaximumLC,
                        "Нет угла соответствующего максимуму DSO для текущих условий".to_owned(),
                    ));
                }


                let target = 0.20;
                let data  = match lever_diagram.dso_lever_max(25., 90.) {
                    Ok(result) => CriterionData::new_result(CriterionID::MaximumLcIcing, result, target),
                    Err(err) => {
                            let error = error.pass_with("lever_diagram.dso_lever_max", err);
                            log::error!("DSOAngleMaxEval eval error: {}", error);
                            CriterionData::new_error(
                            CriterionID::MaximumLcIcing,
                            "Ошибка вычисления максимального плеча диаграммы статической остойчивости в расчете максимума диаграммы статической остойчивости с учетом обледенения: ".to_owned() + &error.to_string(),
                        )
                    }
                };
                let result: DSOAngleMaxCtx = DSOAngleMaxCtx { data };
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
impl std::fmt::Debug for DSOAngleMaxEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DSOAngleMaxEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
