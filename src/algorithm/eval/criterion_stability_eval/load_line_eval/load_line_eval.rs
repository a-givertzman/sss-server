use super::load_line_ctx::LoadLineCtx;
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
/// Расчет критерия осадки по грузовой марке
pub struct LoadLineEval {
    dbg: Dbg,
    value: Option<LoadLineCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl LoadLineEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "LoadLineEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for LoadLineEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();


                let roll = self
                    .parameters
                    .get(ParameterID::Roll)
                    .ok_or(Error::FromString("LoadLine calculate error: no ParameterID::Roll!".to_string()))? * PI / 180.;        
                let mut result = Vec::new();
                for v in self.data.iter() {
                    let z_fix = self.draught.value(v.pos.x())? + v.pos.y() * roll.sin();
                    let z_target = v.pos.z();
                    result.push((v.criterion_id, z_fix, z_target));
                }
                match load_line.calculate() {
                    Ok(load_line) => {
                        for (criterion_id, z_fix, z_target) in load_line {
                            match CriterionID::from(criterion_id) {
                                Ok(criterion_id) => {
                                    res.push(CriterionData::new_result(criterion_id, z_fix, z_target))
                                }
                                Err(e) => {
                                    log::error!("CriterionDraught load_line id error: {}", e.to_string())
                                }
                            }
                        }
                    }
                    Err(err) => log::error!("CriterionDraught load_line error: {}", err.to_string()),
                };
                
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .ok_or(error.err("ship_parameters error: no data!"))?;
                let b = *ship_parameters
                    .get("MouldedBreadth")
                    .ok_or(error.err("breadth error: no data!"))?;
                let balance: BalanceCtx = ctx.read();
                let d = balance.mean_draught;
                let metacentric_height: MetacentricHeightCtx = ctx.read();
                let rolling_amplitude: RollingAmplitudeCtx = ctx.read();
                let rolling_period: RollingPeriodCtx = ctx.read();
                let h_trans_0 = metacentric_height.h_trans_0;
                let k_theta_data = match &initial.coefficient_k_theta {
                    Some(array) => array.data(),
                    None => {
                        let error = error.err("coefficient_k_theta error: no data!");
                        log::error!("{}", error);
                        let result = CriterionData::new_error(
                            CriterionID::LoadLine,
                            "Ошибка расчета критерия ускорения 𝐾∗".to_owned() + &error.to_string(),
                        );
                        let result = LoadLineCtx { data: result };
                        self.value = Some(result.clone());
                        return ctx.write(result);
                    }
                };
                let curve = match Curve::new_linear(&k_theta_data) {
                    Ok(curve) => curve,
                    Err(err) => {
                        let error = error.pass_with("Curve::new_linear", err);
                        log::error!("{}", error);
                        let result = CriterionData::new_error(
                            CriterionID::LoadLine,
                            "Ошибка расчета критерия ускорения 𝐾∗".to_owned() + &error.to_string(),
                        );
                        let result = LoadLineCtx { data: result };
                        self.value = Some(result.clone());
                        return ctx.write(result);
                    }
                };
                let k_theta = match curve.value(b / d) {
                    Ok(curve) => curve,
                    Err(err) => {
                        let error = error.pass_with("k_theta curve.value", err);
                        log::error!("{}", error);
                        let result = CriterionData::new_error(
                            CriterionID::LoadLine,
                            "Ошибка расчета критерия ускорения 𝐾∗".to_owned() + &error.to_string(),
                        );
                        let result = LoadLineCtx { data: result };
                        self.value = Some(result.clone());
                        return ctx.write(result);
                    }
                };
                let c = rolling_period.c;
                let theta_1_r = rolling_amplitude.amplitude;
                let a = 0.0105 * h_trans_0 / (c * c * b) * k_theta * theta_1_r;
                let k = 0.3 / a; // >= 1;
                let result = LoadLineCtx {
                    data: CriterionData::new_result(CriterionID::LoadLine, k, 1.),
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
impl std::fmt::Debug for LoadLineEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadLineEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
