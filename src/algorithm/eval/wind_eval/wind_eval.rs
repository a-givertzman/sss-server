use super::wind_ctx::WindCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextParamsRead, ContextParamsWrite, ContextRead, ContextReadRef}, eval::{parameters::ParameterID, zg_eval::Zg, WindageCtx}
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ContextWrite,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет плеча кренящего момента от давления ветра
pub struct WindEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult>>,
}
//
//
impl WindEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<Zg, EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "WindEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for WindEval {
    fn eval(&mut self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(mut ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let windage: WindageCtx = ctx.read(); 
                let gravity_g = 9.81;
                let ship = initial.ship.as_ref().unwrap();
                let p_v = ship.p_v;
                let m = ship.m;
                let a_v = windage.a_v;
                let z_v = windage.z_v;
                let mass = ctx.read_params(ParameterID::Displacement);
                let arm_wind_static = (p_v * a_v * z_v) / (1000. * gravity_g * mass);
                let arm_wind_dynamic = (1. + m) * arm_wind_static;          
                log::trace!("\t Wind arm_wind_static mass_sum:{mass} p_v:{p_v} a_v:{a_v} z_v:{z_v} arm_wind_static:{arm_wind_static} arm_wind_dynamic:{arm_wind_dynamic}");
                ctx.write_params(ParameterID::DynamicWindageHeelingLever, arm_wind_dynamic);
                ctx.write_params(ParameterID::WindPressure, p_v);
                ctx.write_params(ParameterID::WindageArea, a_v);
                let draught_mean = ctx.read_params(ParameterID::DraughtMean);
                ctx.write_params(ParameterID::WindageAreaLever, z_v - draught_mean/2.);
                ctx.write_params(ParameterID::StaticWindageHeelingLever, arm_wind_static);
                let result = WindCtx {
                    arm_wind_static,
                    arm_wind_dynamic,
                }; 
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for WindEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
