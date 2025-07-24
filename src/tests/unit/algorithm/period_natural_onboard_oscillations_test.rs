#[cfg(test)]
mod period_natural_onboard_oscillations {
    use std::{
        sync::Once, 
        time::{
            Duration, 
            Instant
        }
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
            period_natural_onboard_oscillations::{
                PeriodNaturalOnBoardOscillations, 
                PeriodNaturalOnBoardOscillationsCtx
            }
        }, 
        kernel::{
            eval::Eval, 
            types::eval_result::EvalResult
        }, 
        prelude::{
            Context, 
            InitialCtx
        }
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
    /// Testing `eval`
    #[test]
    fn eval() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let self_id = "eval";
        log::debug!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data = [
            (
                1,
                1.0,
                1.0,
                1.0,
                1.0,
                0.79114,
            )
        ];
        for (step, length_lbp, b, h, d, target) in test_data {
            let ctx = MocEval {
                ctx: Context::new(
                    InitialCtx::new(
                        0, 
                        "Unit-test",
                        length_lbp,
                        b,
                        h,
                        d,
                    )
                ),
            };
            let result = PeriodNaturalOnBoardOscillations::new(ctx).eval(());
            match result {
                Ok(result) => {
                    let result = ContextRead::<PeriodNaturalOnBoardOscillationsCtx>::read(&result).result.clone();
                    assert!(result == target, "step {:?} \nresult: {:?}\ntarget: {:?}", step, result, target);
                },
                Err(err) => panic!("step {:?} \nerror: {:#?}", step, err),
            }
        }
        test_duration.exit();
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
}
