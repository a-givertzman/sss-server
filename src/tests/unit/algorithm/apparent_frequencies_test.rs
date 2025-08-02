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
            ApparentFrequenciesCtx,
            ApparentFrequenciesEval, 
            PeriodExcitementCtx, 
            VesselMaxSpeedCtx, 
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
fn apparent_frequencies() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = "ApparentFrequencies";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (
            1,
            10.0,
            1.0,
            vec![
                (0.0, 0.0, 6.283185307179586), 
                (0.0, 0.1, 6.492624817418906), 
                (0.0, 0.2, 6.702064327658225), 
                (0.0, 0.3, 6.911503837897545), 
                (0.0, 0.4, 7.120943348136865), 
                (0.0, 0.5, 7.330382858376184), 
                (0.0, 0.6, 7.5398223686155035), 
                (0.0, 0.7, 7.749261878854823), 
                (0.0, 0.8, 7.958701389094142), 
                (0.0, 0.9, 8.168140899333462)
            ]  
        ),
        (
            2,
            10.0,
            2.0,
            vec![
                (0.0, 0.0, 3.141592653589793), 
                (0.0, 0.1, 3.193952531149623), 
                (0.0, 0.2, 3.246312408709453), 
                (0.0, 0.3, 3.2986722862692823), 
                (0.0, 0.4, 3.3510321638291125), 
                (0.0, 0.5, 3.4033920413889427), 
                (0.0, 0.6, 3.4557519189487724), 
                (0.0, 0.7, 3.5081117965086026), 
                (0.0, 0.8, 3.5604716740684323), 
                (0.0, 0.9, 3.6128315516282625)
            ]   
        ),
    ];
    for (step, vmax, period_excitement, target) in test_data.iter() {
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
        .write(VesselMaxSpeedCtx { vmax: *vmax })
        .unwrap();
        ctx.ctx = ctx.ctx
        .clone()
        .write(PeriodExcitementCtx { period_excitement: *period_excitement })
        .unwrap();
        let result = ApparentFrequenciesEval::new("Test", ctx).eval(Zg::empty());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<ApparentFrequenciesCtx>::read(&ctx).apparent_frequencies.clone();
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
