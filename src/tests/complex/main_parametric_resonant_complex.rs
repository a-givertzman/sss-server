use crate::{
    algorithm::eval::Zg, 
    kernel::{
        eval::Eval, 
        types::eval_result::EvalResult
    }, 
    prelude::Context
};
#[cfg(test)]
mod main_parametric_resonant_complex {
    use std::{
        sync::{
            Once
        }, 
        time::Duration
    };
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{
        DebugSession, 
        LogLevel, 
        Backtrace
    };
    use crate::{
        algorithm::{
            context::context_access::ContextRead, 
            eval::{
                apparent_frequencies::apparent_frequencies_eval::ApparentFrequenciesEval, 
                main_resonant_zone::{
                    main_resonant_zone_ctx::MainResonantZoneCtx, 
                    main_resonant_zone_eval::MainResonantZoneEval
                }, 
                main_resonant_zone_speed_filter::main_resonant_zone_speed_filter_eval::MainResonantZoneSpeedFilterEval, 
                parametric_resonant_zone::parametric_resonant_zone_eval::ParametricResonantZoneEval, 
                parametric_resonant_zone_speed_filter::parametric_resonant_zone_speed_filter_eval::ParametricResonantZoneSpeedFilterEval, 
                period_excitement::period_excitement_eval::PeriodExcitementEval, 
                roll_frequency_eval::roll_frequency_eval::RollingFrequencyEval, 
                vessel_max_speed::vessel_max_speed_ctx::VesselMaxSpeedCtx, 
                RollingPeriodCtx, 
                Zg
            }
        }, 
        infrostructure::api::client::api_client::ApiClient, 
        kernel::{
            eval::Eval, types::Arc
        }, 
        prelude::{
            Context, 
            ContextWrite, 
            InitialCtx
        }, 
        tests::complex::main_parametric_resonant_complex::MocEval
    };
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
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "eval";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data = [
            (
                1,
                30.0,
                1.0,
                1.0,
                1.0,
            )
        ];
        for (step, wave_length, roll_period, vmax, c) in test_data.iter() {
            let api_client = Arc::new(ApiClient::new(
                "db_logger".to_owned(), 
                "ship_database".to_owned(), 
                "api.example.com".to_owned(), 
                "5432".to_owned()
            ));
            let mut initial_data= InitialCtx::new(
                        0,
                        "Unit-test",
            );
            initial_data.wave_length = Some(*wave_length);
            let mut ctx = MocEval {
                ctx: Context::new(
                    initial_data,
                ),
            };    
            ctx.ctx = ctx.ctx
            .clone()
            .write(
                VesselMaxSpeedCtx { 
                    vmax: *vmax, 
                }
            ).unwrap();    
            ctx.ctx = ctx.ctx
            .clone()
            .write(
                RollingPeriodCtx { 
                    roll_period: *roll_period, 
                    c: *c,
                }
            ).unwrap();
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
                            ParametricResonantZoneEval::new(
                                dbg, 
                                    RollingFrequencyEval::new(
                                        dbg,
                                        ctx,
                                    )
                                )
                            )
                        )
                    )
                ),
                Box::new(move |sql| {
                    let client = Arc::clone(&api_client);
                    client.fetch(sql)
                })
            ).eval(Zg::empty());
            match result {
                Ok(ctx) => {
                    let app_freq = ContextRead::<MainResonantZoneCtx>::read(&ctx).clone();
                    println!("{:?}", app_freq);
                },
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
impl Eval<Zg, EvalResult> for MocEval {
    fn eval(&self, _zg: Zg) -> EvalResult {
        Result::Ok(self.ctx.clone())
    }
}
