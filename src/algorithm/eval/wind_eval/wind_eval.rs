use super::wind_ctx::WindCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::{data::loads::UnitCargoType, parameters::{IParameters, Parameters}, Bound, Moment, Position},
        eval::{IcingTimberCtx, WindageCtx},
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ship_model::model_link::{IModelLink, ModelLink}, ContextWrite, CtxResult
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет плеча кренящего момента от давления ветра
pub struct WindEval {
    dbg: Dbg,
    value: Option<WindCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl WindEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "WindEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for WindEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let parameters: Parameters = ctx.read(); 
                let windage: WindageCtx = ctx.read(); 
                let gravity_g = 9.81;
                let p_v = initial.ship.expect("WindEval eval error: no ship!").p_v;
                let m = initial.ship.expect("WindEval eval error: no ship!").m;
                let a_v = windage.a_v;
                let z_v = windage.z_v;
                let mass = parameters.get(ParameterID::Displacement).ok_or(CtxResult::Err(error.err("calculate mass error: no Displacement in parameters")))?;
                let arm_wind_static = (p_v * a_v * z_v) / (1000. * gravity_g * mass);
                let arm_wind_dynamic = (1. + m) * arm_wind_static;          
                log::trace!("\t Wind arm_wind_static mass_sum:{mass} p_v:{p_v} a_v:{a_v} z_v:{z_v} arm_wind_static:{arm_wind_static} arm_wind_dynamic:{arm_wind_dynamic}");
                parameters.add(ParameterID::DynamicWindageHeelingLever, arm_wind_dynamic);
                parameters.add(ParameterID::WindPressure, p_v);
                parameters
                    .add(ParameterID::WindageArea, a_v);
                if let Some(draught_mean) = parameters.get(ParameterID::DraughtMean) {
                    parameters
                        .add(ParameterID::WindageAreaLever, z_v - draught_mean/2.);
                }
                parameters.add(ParameterID::StaticWindageHeelingLever, arm_wind_static);
                let result = WindCtx {
                    arm_wind_static,
                    arm_wind_dynamic,
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
