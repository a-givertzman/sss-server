use super::wheather_ctx::WheatherCtx;
use crate::{
    ContextWrite, CtxResult,
    algorithm::{
        context::context_access::{ContextParamsWrite, ContextRead},
        eval::{
            BalanceCtx, LeverDiagramCtx, RollingAmplitudeCtx, WindCtx,
            parameters::ParameterID,
        },
    },
    kernel::{eval::Eval, types::eval_result::EvalResult},
};
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
            CtxResult::Ok(mut ctx) => {
                let wind: WindCtx = ctx.read();
                let lever_diagram: LeverDiagramCtx = ctx.read();
                let balance: BalanceCtx = ctx.read();
                let rolling_amplitude: RollingAmplitudeCtx = ctx.read();
                let l_w1 = wind.arm_wind_static;
                let l_w2 = wind.arm_wind_dynamic;
                let theta_w1 = lever_diagram
                    .angle(l_w1)
                    .map_err(|e| error.pass_with("theta_w1", e))?
                    .first()                    
                    .ok_or(error.err("No angle for l_w1"))?
                    .clone();
                let sunset_angle = lever_diagram
                    .angle(0.)
                    .map_err(|e|error.pass_with("sunset_angle", e))?
                    .get(1)
                    .unwrap_or(&90.)
                    .clone();
                let theta_w2: f64 = 50.;
                let theta_f = balance.flooding_angle;
                let l_w2_angles = lever_diagram
                    .angle(l_w2)
                    .map_err(|e| error.pass_with("l_w2_angles", e))?;
                let l_w2_angle_first = *l_w2_angles.first().ok_or(error.err("l_w2_angle_first"))?;
                let theta_c = *l_w2_angles.get(1).ok_or(error.err("theta_c"))?;
                // расчет а
                let a_angle_first = theta_w1 - rolling_amplitude.amplitude;
                let a_lever_first = lever_diagram
                    .lever_moment(a_angle_first)
                    .map_err(|e| error.pass_with("a_lever_first", e))?;
                let a_angle_second = l_w2_angle_first;
                let a_delta_angle = a_angle_second - a_angle_first;
                let a_s1 = lever_diagram
                    .dso_area(a_angle_first, a_angle_second)
                    .map_err(|e| error.pass_with("a_s1", e))?;
                let a_s2 = a_delta_angle * l_w2.to_radians();
                let a = a_s2 - a_s1;
                // расчет b
                let b_angle_first = l_w2_angle_first;
                let b_angle_second = theta_w2.min(theta_f).min(theta_c);
                let b_lever_second = lever_diagram
                    .lever_moment(b_angle_second)
                    .map_err(|e| error.pass_with("b_lever_second", e))?;
                let b_delta_angle = b_angle_second - b_angle_first;
                let b_s1 = lever_diagram
                    .dso_area(b_angle_first, b_angle_second)
                    .map_err(|e| error.pass_with("b_s1", e))?;
                let b_s2 = b_delta_angle * l_w2.to_radians();
                let b = b_s1 - b_s2;
                let k = b / a;
                log::trace!("\t l_w1:{l_w1} l_w2:{l_w2} theta_w1:{theta_w1}  theta_w2:{theta_w2} theta_c:{theta_c} theta_f:{theta_f}
                    a_angle1:{a_angle_first} a_angle2:{l_w2_angle_first} a_s1:{a_s1} a_s2:{a_s2} a:{a} 
                    b_angle1:{l_w2_angle_first} b_angle2:{b_angle_second} b_s1:{b_s1} b_s2:{b_s2} b:{b} k:{k}");
                ctx.write_params(ParameterID::StaticWindageHeelingAngle, theta_w1);
                ctx.write_params(ParameterID::DynamicWindageHeelingAngle, l_w2_angle_first);
                ctx.write_params(
                    ParameterID::HeelingAngleOfSecondPointOfIntersectionWith,
                    theta_c,
                );
                ctx.write_params(ParameterID::AreaA, a);
                ctx.write_params(ParameterID::AreaB, b);
                ctx.write_params(ParameterID::SunsetAngle, sunset_angle);
                ctx.write_params(
                    ParameterID::MinimumOfFludingAngleSecondIntersectionAnd50Degrees,
                    b_angle_second,
                );
                ctx.write_params(
                    ParameterID::HeelingLeverOfDSOCorrespondingToTheMinimumAngle,
                    b_lever_second,
                );
                ctx.write_params(
                    ParameterID::HeelingLeverOfDSOCorrespondingToTheRollToTheWindwardSide,
                    a_lever_first,
                );
                ctx.write_params(ParameterID::RollToTheWindwardSide, a_angle_first);
                let result: WheatherCtx = WheatherCtx { k };
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
impl std::fmt::Debug for WheatherEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WheatherEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
