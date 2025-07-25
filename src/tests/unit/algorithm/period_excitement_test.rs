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
            PeriodExcitementCtx, 
            PeriodExcitementEval, 
            Zg
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
/// Testing 'eval'
#[test]
fn eval() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = "PeriodExcitemennt";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (
            1,
            Some(1.0),
            None,
            0.8,
        ),
        (
            2,
            Some(2.0),
            None,
            1.1
        ),
        (
            3,
            None,
            Some(5.0),
            5.0
        )
    ];
    let epsilon = 1e-1;
    for (step, wave_length, period_excitement, target) in test_data.iter() {
        let mut initial = InitialCtx::new(
            0, 
            "Unit-test"
        );
        initial.wave_length = *wave_length;
        if !period_excitement.is_none() {
            initial.period_excitement = Some(PeriodExcitementCtx { period_excitement: period_excitement.unwrap() });
        }
        let ctx = MocEval {
            ctx: Context::new(
                initial
            ),
        };
        let result = PeriodExcitementEval::new("Test", ctx).eval(Zg::empty());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<PeriodExcitementCtx>::read(&ctx).period_excitement.clone();
                assert!((*target - result) < epsilon, "step {} \nresult: {:?}\ntarget: {:?}", step, target, result);
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
