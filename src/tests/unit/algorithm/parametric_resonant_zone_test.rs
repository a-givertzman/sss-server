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
            ParametricResonantZoneCtx, 
            ParametricResonantZoneEval, 
            RollingFrequencyCtx, 
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
/// Testing 'eval'
#[test]
fn parametric_resonant_zone() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = "MainResonantZone";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (
            1,
            1.0,
            1.9,
            2.1,
        ),
        (
            2,
            2.0,
            3.8,
            4.2,
        ),
        (
            3,
            3.23,
            6.137,
            6.783,
        )
    ];
    let epsilon = 1e-1;
    for (step, roll_frequency, left_side_target, right_side_target) in test_data.iter() {
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
        .write(RollingFrequencyCtx { roll_frequency: *roll_frequency })
        .unwrap();
        let result = ParametricResonantZoneEval::new("Test", ctx).eval(Zg::empty());
        match result {
            Ok(ctx) => {
                let left_side_result = ContextRead::<ParametricResonantZoneCtx>::read(&ctx).left_side.clone();
                assert!((left_side_result - *left_side_target) < epsilon, "step {} \nleft side result: {:?}\nleft side target: {:?}", step, left_side_result, left_side_target);
                let right_side_result = ContextRead::<ParametricResonantZoneCtx>::read(&ctx).right_side.clone();
                assert!((left_side_result - *left_side_target) < epsilon, "step {} \nright side result: {:?}\nright side target: {:?}", step, right_side_result, right_side_target);
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
