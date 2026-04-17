use crate::algorithm::eval::stability::{WindCtx, WindageCtx};
use crate::{
    algorithm::{
        context::context_access::{ContextParamsRead, ContextParamsWrite, ContextRead, ContextReadRef}, eval::{parameters::ParameterID, zg::Zg}
    }, kernel::{Eval, types::eval_result::EvalResult}, prelude::InitialCtx, prelude::ContextWrite,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет плеча кренящего момента от давления ветра
pub struct WindEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl WindEval {
    //
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static,
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
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(mut ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let windage: WindageCtx = ctx.read(); 
                let gravity_g = 9.81;
                let ship = initial.ship.as_ref().unwrap();
                let pv = ship.p_v;
                let m = ship.m;
                let av = windage.av;
                let zv = windage.zv;
                let mass = ctx.read_params(ParameterID::Displacement);
                let arm_wind_static = (pv * av * zv) / (1000. * gravity_g * mass);
                let arm_wind_dynamic = (1. + m) * arm_wind_static;      
                log::info!(
                    "Wind arm_wind_static:{:.3} arm_wind_dynamic:{}",
                    arm_wind_static, arm_wind_dynamic
                );    
                log::trace!("\t Wind arm_wind_static mass_sum:{mass} pv:{pv} av:{av} zv:{zv} arm_wind_static:{arm_wind_static} arm_wind_dynamic:{arm_wind_dynamic}");
                ctx.write_params(ParameterID::DynamicWindageHeelingLever, arm_wind_dynamic);
                ctx.write_params(ParameterID::WindPressure, pv);
                ctx.write_params(ParameterID::WindageArea, av);
                let draught_mean = ctx.read_params(ParameterID::DraughtMean);
                ctx.write_params(ParameterID::WindageAreaLever, zv - draught_mean/2.);
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
