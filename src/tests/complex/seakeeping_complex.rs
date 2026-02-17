use crate::{
    algorithm::eval::Zg, 
    kernel::{
        eval::Eval, 
        types::eval_result::EvalResult
    }, 
    prelude::Context
};
#[cfg(test)]
mod seakeeping {
    use std::{
        collections::HashMap, fs::File, sync::Once, time::Duration
    };
    use serde_json::to_writer;
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{
        DebugSession, 
        LogLevel, 
        Backtrace
    };
    use sal_core::error::Error;
    use crate::{
        algorithm::{
            context::context_access::ContextRead, 
            eval::{apparent_frequencies::apparent_frequencies_eval::ApparentFrequenciesEval, impacts_high_waves::impacts_high_waves_eval::ImpactsHighWavesEval, main_resonant_zone::main_resonant_zone_eval::MainResonantZoneEval, main_resonant_zone_speed_filter::main_resonant_zone_speed_filter_eval::MainResonantZoneSpeedFilterEval, move_broching_filter::{move_broching_filter_ctx::MoveBrochingFilterCtx, move_broching_filter_eval::MoveBrochingFilterEval}, parametric_resonant_zone::parametric_resonant_zone_eval::ParametricResonantZoneEval, parametric_resonant_zone_speed_filter::parametric_resonant_zone_speed_filter_eval::ParametricResonantZoneSpeedFilterEval, period_excitement::{period_excitement_ctx::PeriodExcitementCtx, period_excitement_eval::PeriodExcitementEval}, roll_frequency_eval::roll_frequency_eval::RollingFrequencyEval, vessel_max_speed::vessel_max_speed_ctx::VesselMaxSpeedCtx, RollingPeriodCtx, Zg}
        }, infrostructure::query::resonant_zone::resonant_zone::ResonantZoneQuery, kernel::{eval::Eval, types::Arc}, prelude::{
            Context, 
            ContextWrite, 
            InitialCtx
        }, tests::complex::seakeeping_complex::MocEval
    };
    ///
    ///
    static INIT: Once = Once::new();
    // Mock for ApiClient
    struct MockApiClient;
    //
    impl MockApiClient {
        fn new() -> Arc<Self> {
            Arc::new(Self)
        }
        // Заглушка для запроса к БД
        fn fetch(&self, _query: &str) -> Result<Vec<u8>, Error> {
            Ok(Vec::new())
        }
    }
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
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "ComplexTest | eval";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (
                1,
                270.0,
                7.933569184169254,
                0.4435,
                6.0,
                20.0,
                50.0,
                vec![
                    (

                    )
                ]
            )
        ];
        for (step, course_angle, roll_period, c, period_excitement, vmax, length_lbp, target) in test_data.iter() {
            let api_client = MockApiClient::new();
            let mut initial_data= InitialCtx::new(
                0,
                "Unit-test",
            );
            initial_data.course_angle = Some(*course_angle);
            initial_data.period_excitement = Some(
                PeriodExcitementCtx { 
                    period_excitement: *period_excitement 
                }
            );
            let mut ship_params = HashMap::new();
            ship_params.insert("LBP".to_owned(), *length_lbp);
            initial_data.ship_parameters = Some(ship_params);
            let mut ctx = MocEval {
                ctx: Context::new(
                    initial_data,
                ),
            };
            ctx.ctx = ctx.ctx
            .clone()
            .write(
                RollingPeriodCtx { 
                    roll_period: *roll_period,
                    c: *c, 
                }
            ).unwrap();
            ctx.ctx = ctx.ctx
            .clone()
            .write(
                VesselMaxSpeedCtx { 
                    vmax: *vmax,
                }
            ).unwrap();
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
                                        ParametricResonantZoneEval::new(
                                            dbg, 
                                            RollingFrequencyEval::new(
                                                dbg,
                                                ctx
                                            )
                                        )
                                    )
                                )
                            ),
                            Box::new(move |resonant_zone, zone_id|{
                                let client = Arc::clone(&api_client);
                                client.fetch(&ResonantZoneQuery::new(resonant_zone, zone_id).sql())
                            })
                        )
                    )
                )
            ).eval(Zg::empty());
            match result {
                Ok(ctx) => {
                    let result = ContextRead::<MoveBrochingFilterCtx>::read(&ctx).move_broching_filter.clone();
                    write_json("broching.json", &result).expect("error");
                    //assert!(result == *target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                },
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
impl Eval<Zg, EvalResult> for MocEval {
    fn eval(&self, _zg: Zg) -> EvalResult {
        Result::Ok(self.ctx.clone())
    }
}
