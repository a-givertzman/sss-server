use crate::{
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::Context,
};
#[cfg(test)]
mod seakeeping {
    use crate::{
        algorithm::{
            context::context_access::ContextRead, entities::{data::Voyage}, eval::{parameters::ParameterID, seakeeping::eval::{apparent_frequencies::apparent_frequencies_eval::ApparentFrequenciesEval, impacts_high_waves::impacts_high_waves_eval::ImpactsHighWavesEval, main_resonant_zone::main_resonant_zone_eval::MainResonantZoneEval, main_resonant_zone_speed_filter::main_resonant_zone_speed_filter_eval::MainResonantZoneSpeedFilterEval, move_broching_filter::{move_broching_filter_ctx::MoveBrochingFilterCtx, move_broching_filter_eval::MoveBrochingFilterEval}, parametric_resonant_zone::parametric_resonant_zone_eval::ParametricResonantZoneEval, parametric_resonant_zone_speed_filter::parametric_resonant_zone_speed_filter_eval::ParametricResonantZoneSpeedFilterEval, period_excitement::{period_excitement_ctx::PeriodExcitementCtx, period_excitement_eval::PeriodExcitementEval}}}
        }, kernel::Eval, prelude::{Context, ContextParamsWrite, ContextWrite, InitialCtx}, tests::complex::seakeeping_complex::MocEval
    };
    use debugging::session::debug_session::{DebugSession, LogLevel};
    use sal_3dlib_core::math::Bounds;
use serde_json::to_writer;
    use std::{collections::HashMap, fs::File, sync::Once, time::Duration};
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
    fn write_json(path: &str, data: &[(f64, f64)]) -> std::io::Result<()> {
        let file = File::create(path)?;
        to_writer(file, data)?;
        Ok(())
    }
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
        let dbg = "ComplexTest | eval";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [(1, 270.0, 7.933569184169254, 6.0, 20.0, 50.0, vec![()])];
        for (step, course_angle, roll_period, period_excitement, vmax, length_lbp, target) in
            test_data.iter()
        {
            let mut initial = InitialCtx::new(
                "0",
                "Unit-test",
                Bounds::from_min_max(0., 100., 20).unwrap(),
            );
            initial.voyage = Some(Voyage {
                density: 1.025,
                operational_speed: *vmax,
                icing_type: "none".to_owned(),
                icing_timber_type: "full".to_owned(),
                area: Some("sea".to_owned()),
                course_angle: *course_angle,
                wave_heading_angle: 90.,
                wave_length: 10.,
                current_speed: 10.,
            });
            let mut ship_params = HashMap::new();
            ship_params.insert("LBP".to_owned(), *length_lbp);
            initial.ship_parameters = Some(ship_params);
            let mut ctx = MocEval {
                ctx: Context::new(initial),
            };
            ctx.ctx = ctx
                .ctx
                .clone()
                .write(PeriodExcitementCtx {
                    period_excitement: *period_excitement,
                })
                .unwrap();
            ctx.ctx.write_params(ParameterID::RollPeriod, *roll_period);
            let result = ImpactsHighWavesEval::new(
                dbg,
                MoveBrochingFilterEval::new(
                    dbg,
                    MainResonantZoneSpeedFilterEval::new(
                        dbg,
                        ParametricResonantZoneSpeedFilterEval::new(
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
                    ),
                ),
            )
            .eval(());
            match result {
                Ok(ctx) => {
                    let result = ContextRead::<MoveBrochingFilterCtx>::read(&ctx)
                        .move_broching_filter
                        .clone();
                    write_json("broching.json", &result).unwrap();
                    //assert!(result == *target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                }
                Err(err) => panic!("step {} \nerror: {:#?}", step, err),
            }
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
