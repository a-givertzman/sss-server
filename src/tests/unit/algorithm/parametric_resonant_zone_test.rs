use crate::{
    algorithm::{
        entities::Bounds, eval::{parameters::ParameterID, seakeeping::eval::parametric_resonant_zone::{parametric_resonant_zone_ctx::ParametricResonantZoneCtx, parametric_resonant_zone_eval::ParametricResonantZoneEval}},
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::*,
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
/// Testing [parametric_resonant_zone_eval](src/algorithm/eval/parametric_resonant_zone)
#[test]
fn parametric_resonant_zone() {
    DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = "ParametricResonantZone";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (1, 3.577708763999664, 6.797646651599361, 7.513188404399294),
        (2, 2.0, 3.8, 4.2),
        (3, 3.23, 6.137, 6.783),
    ];
    let epsilon = 1e-1;
    for (step, roll_frequency, left_side_target, right_side_target) in test_data.iter() {
        let mut ctx = MocEval {
            ctx: Context::new(InitialCtx::new(
                "0",
                "Unit-test",
                Bounds::from_min_max(0., 100., 20).unwrap(),
            )),
        };
        ctx.ctx.write_params(ParameterID::RollPeriod, 1./roll_frequency);
        let result = ParametricResonantZoneEval::new("parametric_resonant_zone", ctx).eval(());
        match result {
            Ok(ctx) => {
                let left_side_result = ContextRead::<ParametricResonantZoneCtx>::read(&ctx)
                    .left_side
                    .clone();
                assert!(
                    (left_side_result - *left_side_target) < epsilon,
                    "step {} \nleft side result: {:?}\nleft side target: {:?}",
                    step,
                    left_side_result,
                    left_side_target
                );
                let right_side_result = ContextRead::<ParametricResonantZoneCtx>::read(&ctx)
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
