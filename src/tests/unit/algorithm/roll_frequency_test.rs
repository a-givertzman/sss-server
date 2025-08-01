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
            RollingFrequencyCtx, 
            RollingPeriodCtx, 
            RollingPeriodEval, 
            Zg
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
/// Testing [roll_frequency_eval](src/algorithm/eval/roll_frequency_eval)
#[test]
fn roll_frequency() {
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
            1.0,
            1.0,
            6.28
        ),
        (
            2,
            2.0,
            1.0,
            3.14
        ),
        (
            3,
            6.28,
            1.0,
            1.0,
        )
    ];
    for (step, roll_period, c, target) in test_data.iter() {
        let mut ctx = MocEval {
            ctx: Context::new(
                InitialCtx::new(
                    0,
                    "Unit-test",
                )
            ),
        };
        ctx.ctx = ctx.ctx
        .clone()
        .write(RollingPeriodCtx { roll_period: *roll_period, c: *c })
        .unwrap();
        let result = RollingPeriodEval::new("Test", ctx).eval(Zg::empty());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<RollingFrequencyCtx>::read(&ctx).roll_frequency.clone();
                assert!(result == *target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
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
