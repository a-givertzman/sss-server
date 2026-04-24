use crate::{
    algorithm::{
        context::context_access::ContextRead, entities::Bounds, eval::{
            parameters::ParameterID, 
            seakeeping::eval::main_resonant_zone::{
                main_resonant_zone_ctx::MainResonantZoneCtx, 
                main_resonant_zone_eval::MainResonantZoneEval
            }
        }
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{Context, ContextParamsWrite, InitialCtx},
};
use debugging::session::debug_session::{DebugSession, LogLevel};
#[cfg(test)]
use std::{sync::Once, time::Duration};
use testing::stuff::max_test_duration::TestDuration;
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
/// Testing [main_resonant_zone_eval](src/algorithm/eval/main_resonant_zone)
#[test]
fn main_resonant_zone() {
    DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = "MainResonantZone";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (1, 1.0, 0.7, 1.3),
        (2, 2.0, 1.4, 2.6),
        (3, 3.23, 2.261, 4.199),
    ];
    let epsilon = 1e-1;
    for (step, roll_frequency, left_side_target, right_side_target) in test_data.iter() {
        let mut ctx = MocEval {
            ctx: Context::new(InitialCtx::new(
                "0",
                "Unit-test",
                Bounds::from_min_max(0., 100., 20).unwrap(),
                Vec::new()
            )),
        };
        ctx.ctx.write_params(ParameterID::RollPeriod, 1./roll_frequency);
        let result = MainResonantZoneEval::new("main_resonant_zone", ctx).eval(());
        match result {
            Ok(ctx) => {
                let left_side_result = ContextRead::<MainResonantZoneCtx>::read(&ctx)
                    .left_side
                    .clone();
                assert!(
                    (left_side_result - *left_side_target) < epsilon,
                    "step {} \nleft side result: {:?}\nleft side target: {:?}",
                    step,
                    left_side_result,
                    left_side_target
                );
                let right_side_result = ContextRead::<MainResonantZoneCtx>::read(&ctx)
                    .right_side
                    .clone();
                assert!(
                    (right_side_result - *right_side_target) < epsilon,
                    "step {} \nright side result: {:?}\nright side target: {:?}",
                    step,
                    right_side_result,
                    right_side_target
                );
            }
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
impl Eval<(), EvalResult> for MocEval {
    fn eval(&self, _: ()) -> EvalResult {
        Result::Ok(self.ctx.clone())
    }
}
