use crate::{
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::Context,
};
#[cfg(test)]
mod main_parametric_resonant_complex {
    use crate::{
        algorithm::{
            context::context_access::ContextRead, entities::{Bounds, data::Voyage}, eval::{parameters::ParameterID, seakeeping::eval::{apparent_frequencies::apparent_frequencies_eval::ApparentFrequenciesEval, main_resonant_zone::{main_resonant_zone_ctx::MainResonantZoneCtx, main_resonant_zone_eval::MainResonantZoneEval}, main_resonant_zone_speed_filter::main_resonant_zone_speed_filter_eval::MainResonantZoneSpeedFilterEval, parametric_resonant_zone::parametric_resonant_zone_eval::ParametricResonantZoneEval, parametric_resonant_zone_speed_filter::parametric_resonant_zone_speed_filter_eval::ParametricResonantZoneSpeedFilterEval, period_excitement::period_excitement_eval::PeriodExcitementEval}}
        }, kernel::Eval, prelude::{Context, ContextParamsWrite, InitialCtx}, tests::complex::main_parametric_resonant_complex::MocEval
    };
    use debugging::session::debug_session::{DebugSession, LogLevel};
    use std::{sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    ///
    ///
    static INIT: Once = Once::new();
    ///
    /// once called initialisation
    fn init_once() {
        INIT.call_once(|| {
            // implement your initialisation code to be called only once for current test file
        })
    }
    ///
    /// returns:
    ///  - ...
    fn init_each() -> () {}
    ///
    /// Testing 'eval'
    #[test]
    fn eval() {
        DebugSession::new()
            .filter(LogLevel::Info)
            .module("api_tools", LogLevel::Error)
            .module("sal_sync", LogLevel::Error)
            .module("ena", LogLevel::Error)
            .init();
        init_once();
        init_each();
        log::debug!("");
        let dbg = "eval";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data = [(1, 30.0, 1.0, 1.0, 1.0)];
        for (step, wave_length, roll_period, vmax, _c) in test_data.iter() {
            let mut initial = InitialCtx::new(
                "0",
                "Unit-test",
                Bounds::from_min_max(0., 100., 20).unwrap(),
                Vec::new()
            );
            initial.voyage = Some(Voyage {
                density: 1.025,
                operational_speed: *vmax,
                icing_type: "none".to_owned(),
                icing_timber_type: "full".to_owned(),
                area: Some("sea".to_owned()),
                course_angle: 270.0,
                wave_heading_angle: 90.,
                wave_length: *wave_length,
                current_speed: 10.,
            });
            let mut ctx = MocEval {
                ctx: Context::new(initial),
            };
            ctx.ctx.write_params(ParameterID::RollPeriod, *roll_period);
            let dbg = "ComplexTest";
            let result = ParametricResonantZoneSpeedFilterEval::new(
                dbg,
                MainResonantZoneSpeedFilterEval::new(
                    dbg,
                    ApparentFrequenciesEval::new(
                        dbg,
                        PeriodExcitementEval::new(
                            dbg,
                            MainResonantZoneEval::new(
                                dbg,
                                ParametricResonantZoneEval::new(dbg, ctx),
                            ),
                        ),
                    ),
                ),
            )
            .eval(());
            match result {
                Ok(ctx) => {
                    let _app_freq = ContextRead::<MainResonantZoneCtx>::read(&ctx).clone();
                }
                Err(err) => panic!("step {} \nerror: {:#?}", step, err),
            }
            //assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
        test_duration.exit();
    }
}
///
///
#[derive(Debug, Clone)]
struct MocEval {
    pub ctx: Context,
}
//
//
impl Eval<(), EvalResult> for MocEval {
    fn eval(&self, _: ()) -> EvalResult {
        Result::Ok(self.ctx.clone())
    }
}
