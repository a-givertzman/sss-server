use super::dso_area_ctx::DSOAreaCtx;
use crate::{
    ContextWrite,
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::data::ship_type::ShipType,
        eval::{BalanceCtx, CriterionData, CriterionID, LeverDiagramCtx},
    },
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия площади под диаграммой статической остойчивости
pub struct DSOAreaEval {
    dbg: Dbg,
    value: Option<DSOAreaCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl DSOAreaEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "DSOAreaEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for DSOAreaEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let ship_type = initial.ship_type.unwrap();
                let lever_diagram: LeverDiagramCtx = ctx.read();
                let balance: BalanceCtx = ctx.read();
                let flooding_angle = balance.flooding_angle;
                let mut data = Vec::new();
                let theta = lever_diagram.angle(0.).unwrap_or(vec![0., 0.]);
                let theta_0 = *theta.first().unwrap_or(&0.);
                let theta_max = *theta.last().unwrap_or(&0.);
                let second_angle_30 = theta_max.min(30.).min(flooding_angle);
                match lever_diagram.dso_area(theta_0, second_angle_30) {
                    Ok(result) => data.push(CriterionData::new_result(
                        CriterionID::AreaLC0_30,
                        result,
                        0.055,
                    )),
                    Err(err) => {
                        let error = error.pass_with("lever_diagram.dso_area", err);
                        log::error!("DSOAreaEval eval 0-30 error: {}", error);
                        data.push(CriterionData::new_error(
                        CriterionID::AreaLC0_30,
                        "Ошибка расчета площади под положительной частью диаграммы статической остойчивости 0-30 градусов: ".to_owned() + &error.to_string(),
                    ))
                    }
                };
                let second_angle_40 = theta_max.min(40.).min(flooding_angle);
                let target_area = if ship_type != ShipType::TimberCarrier {
                    0.09
                } else {
                    0.08
                };
                match lever_diagram.dso_area(theta_0, second_angle_40) {
                    Ok(result) => data.push(CriterionData::new_result(
                        CriterionID::AreaLC0_40,
                        result,
                        target_area,
                    )),
                    Err(err) => {
                        let error = error.pass_with("ship_type lever_diagram.dso_area", err);
                        log::error!("DSOAreaEval 0-40 error: {}", error);
                        data.push(CriterionData::new_error(
                                    CriterionID::AreaLC0_40,
                                    "Ошибка расчета площади под положительной частью диаграммы статической остойчивости 0-40 градусов: ".to_owned() + &error.to_string(),
                                ))
                    }
                };
                let first_angle_30 = theta_0.max(30.);
                match lever_diagram.dso_area(first_angle_30, second_angle_40) {
                    Ok(result) => data.push(CriterionData::new_result(
                        CriterionID::AreaLC30_40,
                        result,
                        0.03,
                    )),
                    Err(error) => {
                        log::error!("DSOAreaEval 30-40 error: {}", error);
                        data.push(CriterionData::new_error(
                        CriterionID::AreaLC30_40,
                        "Ошибка расчета площади под положительной частью диаграммы статической остойчивости 30-40 градусов: ".to_owned() + &error.to_string(),
                    ))
                    }
                };
                //    log::info!("Criterion dso: zg:{} theta_0:{theta_0} theta_max:{theta_max} first_angle_30:{first_angle_30} second_angle_30:{second_angle_30} second_angle_40:{second_angle_40}", self.metacentric_height.z_g_fix().unwrap_or(-1.));
                let result = DSOAreaCtx { data };
                self.value = Some(result.clone());
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for DSOAreaEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DSOAreaEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
