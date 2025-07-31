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
    let dbg = "eval";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (
            1,
            10.0,
            1.0,
            vec![
                (0.0, 6.283185307179586), 
                (0.0, 6.492624817418906), 
                (0.0, 6.702064327658225), 
                (0.0, 6.911503837897545), 
                (0.0, 7.120943348136865), 
                (0.0, 7.330382858376184), 
                (0.0, 7.5398223686155035), 
                (0.0, 7.749261878854823), 
                (0.0, 7.958701389094142), 
                (0.0, 8.168140899333462)
            ]  
        ),
        (
            2,
            10.0,
            2.0,
            vec![
                (0.0, 50.26548245743669), 
                (0.0, 51.103240498393966), 
                (0.0, 51.94099853935125), 
                (0.0, 52.77875658030852), 
                (0.0, 53.6165146212658), 
                (0.0, 54.45427266222308), 
                (0.0, 55.29203070318036), 
                (0.0, 56.12978874413764), 
                (0.0, 56.96754678509492), 
                (0.0, 57.8053048260522)
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
