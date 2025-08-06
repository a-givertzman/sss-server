#[cfg(test)]
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
    algorithm::{
        context::context_access::ContextRead, 
        eval::{
            ImpactsHighWavesCtx, 
            ImpactsHighWavesEval, 
            PeriodExcitementCtx, 
            Zg,
        }
    }, 
    kernel::{
        eval::Eval, 
        types::eval_result::EvalResult
    }, 
    prelude::{
        Context, 
        ContextWrite, 
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
/// Testing [impacts_high_waves](src/algorithm/eval/impacts_high_waves)
#[test]
fn impacts_high_waves() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = "ImpactsHighWaves";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (
            1,
            0.0,
            10.0,
            vec![
                (135.0, 13.0), 
                (135.1, 13.016666666666667), 
                (135.2, 13.033333333333331), 
                (135.3, 13.050000000000002), 
                (135.4, 13.066666666666666), 
                (135.5, 13.083333333333334), 
                (135.6, 13.1), 
                (135.7, 13.116666666666664), 
                (135.8, 13.133333333333336), 
                (135.9, 13.15)
            ]
        ),
        (
            2,
            0.0,
            1.0,
            vec![
                (135.0, 1.3), 
                (135.1, 1.3016666666666665), 
                (135.2, 1.3033333333333332), 
                (135.3, 1.3050000000000002), 
                (135.4, 1.3066666666666669), 
                (135.5, 1.3083333333333336), 
                (135.6, 1.3099999999999998), 
                (135.7, 1.3116666666666665), 
                (135.8, 1.3133333333333337), 
                (135.9, 1.3150000000000002)
            ]   
        ),
    ];
    for (step, course_angle, period_excitement, target) in test_data.iter() {
        let mut initial_data = InitialCtx::new(
            0,
            "Unit-test",
        );
        initial_data.course_angle = Some(*course_angle);
        let mut ctx = MocEval {
            ctx: Context::new(
                initial_data
            ),
        };
        ctx.ctx = ctx.ctx
        .clone()
        .write(PeriodExcitementCtx { period_excitement: *period_excitement })
        .unwrap();
        let result = ImpactsHighWavesEval::new("Test", ctx).eval(Zg::empty());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<ImpactsHighWavesCtx>::read(&ctx).impacts_high_waves.clone();
                assert!(result[0..10] == *target, "step {} \nresult: {:?}\ntarget: {:?}", step, &result[0..10], target);
            },
            Err(err) => panic!("step {} \nerror: {:#?}", step, err),

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
impl Eval<Zg, EvalResult> for MocEval {
    fn eval(&self, _zg: Zg) -> EvalResult {
        Result::Ok(self.ctx.clone())
    }
}
