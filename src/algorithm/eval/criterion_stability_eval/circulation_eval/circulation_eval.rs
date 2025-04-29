use super::circulation_ctx::CirculationCtx;
use crate::algorithm::eval::{CriterionData, CriterionID};
use crate::{
    MetacentricHeightCtx, RollingAmplitudeCtx, BalanceCtx, RollingPeriodCtx,
    ContextWrite, CtxResult,
    algorithm::context::context_access::{ContextRead, ContextReadRef},
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
};
use crate::algorithm::entities::{ Curve, ICurve };
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия крена на циркуляции
pub struct CirculationEval {
    dbg: Dbg,
    value: Option<CirculationCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl CirculationEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "CirculationEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for CirculationEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .ok_or(error.err("ship_parameters error: no data!"))?;
                let b = *ship_parameters
                    .get("MouldedBreadth")
                    .ok_or(error.err("breadth error: no data!"))?;
                let balance: BalanceCtx = ctx.read();
                let d = balance.mean_draught;

                let balance: BalanceCtx = ctx.read();
                let flooding_angle = balance.flooding_angle;

                let target = 16.0f64.min(flooding_angle / 2.);
                let angle = match self.circulation.angle() {
                    Ok(value) => value,
                    Err(error) => {
                        let error = Error::FromString(format!(
                            "CriterionStability circulation angle error: {}",
                            error
                        ));
                        log::error!("{error}");
                        return CriterionData::new_error(
                            CriterionID::HeelTurning,
                            "Ошибка вычисления крена на циркуляции: ".to_owned() + &error.to_string(),
                        );
                    }
                };
                if let Some(angle) = angle {
                    CriterionData::new_result(CriterionID::HeelTurning, angle, target)
                } else {
                    match self.circulation.velocity(target) {
                        Ok(velocity) => CriterionData::new_error(
                            CriterionID::HeelTurning,
                            format!(
                                "Крен {target} градусов, рекомендуемая скорость {} m/s');",
                                velocity,
                            ),
                        ),
                        Err(error) => {
                            let error = Error::FromString(format!(
                                "CriterionStability circulation velocity error: {}",
                                error
                            ));
                            log::error!("{error}");
                            CriterionData::new_error(
                                CriterionID::HeelTurning,
                                "Ошибка вычисления рекомендуемой скорости: ".to_owned()
                                    + &error.to_string(),
                            )
                        }
                    }
                }
                let result = CirculationCtx {
                    data: CriterionData::new_result(CriterionID::Circulation, k, 1.),
                };
                self.value = Some(result.clone());
                ctx.write(result)
                // TODO: В случаях, когда палубный груз контейнеров размещается только на крышках грузовых
                // люков, вместо угла входа кромки верхней палубы может приниматься меньший из углов
                // входа в воду верхней кромки комингса люка или входа контейнера в воду (в случае, когда
                // контейнеры выходят за пределы этого комингса).
            }
            CtxResult::Err(err) => CtxResult::Err(error.pass_with("Read context error", err)),
            CtxResult::None => CtxResult::None,
        }
    }
}
//
//
impl std::fmt::Debug for CirculationEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CirculationEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
