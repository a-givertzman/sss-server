use super::wheather_ctx::WheatherCtx;
use crate::{
    algorithm::{context::context_access::{ContextRead, ContextReadRef}, eval::MetacentricHeightCtx}, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ship_model::model_link::{IModelLink, ModelLink}, ContextWrite, CtxResult
};
use crate::algorithm::entities::math::curve::*;
use crate::algorithm::entities::data::stability::{*, multipler_s::*};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия погоды К
pub struct WheatherEval {
    dbg: Dbg,
    value: Option<WheatherCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl WheatherEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "WheatherEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for WheatherEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let parameters: Parameters = ctx.read();
                let l_w1 = self.wind.arm_wind_static().map_err(|e| Error::FromString(format!("Stability k l_w1 error: {e}")))?;
                let l_w2 = self.wind.arm_wind_dynamic().map_err(|e| Error::FromString(format!("Stability k l_w2 error: {e}")))?;
                let theta_w1 = *self
                    .lever_diagram
                    .angle(l_w1)
                    .map_err(|e| Error::FromString(format!("Stability k theta_w1 error: {e}")))?
                    .first()
                    .ok_or(Error::Calculate(
                        "Stability k error: no angle for l_w1".to_owned(),
                    ))?;
                let sunset_angle = *self.lever_diagram.angle(0.)
                    .map_err(|e| Error::FromString(format!("Stability k sunset_angle error: {e}")))?
                    .get(1).unwrap_or(&90.);
                let theta_w2: f64 = 50.;
                let theta_f = self.flooding_angle;
                let l_w2_angles = self
                    .lever_diagram
                    .angle(l_w2)
                    .map_err(|e| Error::FromString(format!("Grain calculate dso_area error: {e}")))?;
                let l_w2_angle_first = *l_w2_angles.first().ok_or(Error::Calculate(
                    "Stability k error: no angle for l_w2".to_owned(),
                ))?;
                let theta_c = *l_w2_angles.get(1).ok_or(Error::Calculate(
                    "Stability k error: no second angle for l_w2".to_owned(),
                ))?;
                // расчет а
                let (rolling_period, rolling_amplitude) =
                    self.rolling_amplitude.calculate().map_err(|e| {
                        Error::FromString(format!(
                            "Stability k (rolling_period, rolling_amplitude) error: {e}"
                        ))
                    })?;
                let a_angle_first = theta_w1 - rolling_amplitude.round();
                let a_lever_first = self.lever_diagram.lever_moment(a_angle_first)
                    .map_err(|e| Error::FromString(format!("Stability k a_lever_first error: {e}")))?;
                let a_angle_second = l_w2_angle_first;
                let a_delta_angle = a_angle_second - a_angle_first;
                let a_s1 = self
                    .lever_diagram
                    .dso_area(a_angle_first, a_angle_second)
                    .map_err(|e| Error::FromString(format!("Stability k a_s1 error: {e}")))?;
                let a_s2 = a_delta_angle * l_w2 * PI / 180.;
                let a = a_s2 - a_s1;
                // расчет b
                let b_angle_first = l_w2_angle_first;
                let b_angle_second = theta_w2.min(theta_f).min(theta_c);
                let b_lever_second = self.lever_diagram.lever_moment(b_angle_second)
                    .map_err(|e| Error::FromString(format!("Stability k b_lever_second error: {e}")))?;
                let b_delta_angle = b_angle_second - b_angle_first;
                let b_s1 = self
                    .lever_diagram
                    .dso_area(b_angle_first, b_angle_second)
                    .map_err(|e| Error::FromString(format!("Stability k b_s1 error: {e}")))?;
                let b_s2 = b_delta_angle * l_w2 * PI / 180.;
                let b = b_s1 - b_s2;
                let k = b / a;
                log::trace!("\t Stability k l_w1:{l_w1} l_w2:{l_w2} theta_w1:{theta_w1}  theta_w2:{theta_w2} theta_c:{theta_c} theta_f:{theta_f}
                    a_angle1:{a_angle_first} a_angle2:{l_w2_angle_first} a_s1:{a_s1} a_s2:{a_s2} a:{a} 
                    b_angle1:{l_w2_angle_first} b_angle2:{b_angle_second} b_s1:{b_s1} b_s2:{b_s2} b:{b} k:{k}");
                parameters
                    .add(ParameterID::StaticWindageHeelingAngle, theta_w1);
                parameters
                    .add(ParameterID::DynamicWindageHeelingAngle, l_w2_angle_first);
                parameters.add(
                    ParameterID::HeelingAngleOfSecondPointOfIntersectionWith,
                    theta_c,
                );
                parameters
                    .add(ParameterID::RollAmplitude, rolling_amplitude);
                parameters.add(ParameterID::RollPeriod, rolling_period);
                parameters.add(ParameterID::AreaA, a);
                parameters.add(ParameterID::AreaB, b);
                parameters.add(ParameterID::SunsetAngle, sunset_angle);
                parameters.add(ParameterID::MinimumOfFludingAngleSecondIntersectionAnd50Degrees, b_angle_second);
                parameters.add(ParameterID::HeelingLeverOfDSOCorrespondingToTheMinimumAngle, b_lever_second);
                parameters.add(ParameterID::HeelingLeverOfDSOCorrespondingToTheRollToTheWindwardSide, a_lever_first);
                parameters.add(ParameterID::RollToTheWindwardSide, a_angle_first);
                let result = WheatherCtx {
                    k,
                };
                self.value = Some(result.clone());
                ctx.write(parameters)?;
                ctx.write(result)
            }
            CtxResult::Err(err) => CtxResult::Err(error.pass_with("Read context error", err)),
            CtxResult::None => CtxResult::None,
        }
    }
}
//
//
impl std::fmt::Debug for WindEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
