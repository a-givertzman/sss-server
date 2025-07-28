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
        sync::Once, 
        time::Duration
    };
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{
        DebugSession, 
        LogLevel, 
        Backtrace
    };
    use crate::{
        algorithm::{context::context_access::ContextRead, eval::{ApparentFrequenciesEval, MainResonantZoneEval, ParametricResonantZoneEval, PeriodExcitementEval, RollingFrequencyEval, RollingPeriodCtx, VesselSpeedFilterCtx, VesselSpeedFilterEval, Zg}}, kernel::eval::Eval, prelude::{
            Context, ContextWrite, InitialCtx
        }, tests::complex::main_parametric_resonant_complex::MocEval
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
                10.0,
                10.0,
                10.0,
            )
        ];
        for (step, wave_length, roll_period, c) in test_data.iter() {
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
            todo!("вписать макс speed");
            ctx.ctx = ctx.ctx
            .clone()
            .write(
                RollingPeriodCtx { 
                    roll_period: *roll_period, 
                    c: *c,
                }
            ).unwrap();
            let dbg = "ComplexTest";
            let result = VesselSpeedFilterEval::new(
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
            ).eval(Zg::empty());
            match result {
                Ok(ctx) => {
                    let result = ContextRead::<VesselSpeedFilterCtx>::read(&ctx).vessel_speed_filter.clone();
                    println!("{:?}", result);
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
