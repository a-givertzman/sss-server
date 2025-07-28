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
            VesselMoveBroachingCtx, 
            VesselMoveBroachingEval, 
            VesselSpeedFilterCtx, 
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
            VesselSpeedFilterCtx {
                vessel_speed_filter: vec![
                    6.283185307179586, 
                    6.492624817418906, 
                    6.702064327658225, 
                    6.911503837897545, 
                    7.120943348136865, 
                    7.330382858376184, 
                    7.5398223686155035, 
                    7.749261878854823, 
                    7.958701389094142, 
                    8.168140899333462,
                ]
            },
            vec![
                todo!("CALCULATE TARGET")
            ]  
        ),
    ];
    for (step, length_lbp, vessel_speed_filter, target) in test_data.iter() {
        let initial_data = InitialCtx::new(
                0,
                "Unit-test",
        );
        initial_data.ship_parameters
        .clone()
        .unwrap()
        .insert(
            "LBP".to_owned(),
            *length_lbp
        );
        let mut ctx = MocEval {
            ctx: Context::new(
                initial_data,
            ),
        };
        ctx.ctx = ctx.ctx
        .clone()
        .write(vessel_speed_filter.clone())
        .unwrap();
        let result = VesselMoveBroachingEval::new("Test", ctx).eval(Zg::empty());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<VesselMoveBroachingCtx>::read(&ctx).vessel_move_broaching.clone();
                assert!(result == *target, "step {} \nresult: {:?}\ntarget: {:?}", step, &result, target);
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
