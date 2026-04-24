use crate::{
    algorithm::{
        context::context_access::ContextRead,
        entities::{Bounds, data::Voyage},
        eval::seakeeping::eval::period_excitement::{
            period_excitement_ctx::PeriodExcitementCtx,
            period_excitement_eval::PeriodExcitementEval,
        },
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{Context, ContextWrite, InitialCtx},
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
/// Testing [period_excitement_eval](src/algorithm/eval/period_excitement)
#[test]
fn period_excitement() {
    DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = "PeriodExcitemennt";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (1, Some(1.0), None, 0.8),
        (2, Some(2.0), None, 1.1),
        (3, None, Some(5.0), 5.0),
    ];
    let epsilon = 1e-1;
    for (step, wave_length, period_excitement, target) in test_data.iter() {
        let mut initial = InitialCtx::new(
            "0",
            "Unit-test",
            Bounds::from_min_max(0., 100., 20).unwrap(),
            Vec::new()
        );
        initial.voyage = Some(Voyage {
            density: 1.025,
            operational_speed: 12.,
            icing_type: "none".to_owned(),
            icing_timber_type: "full".to_owned(),
            area: Some("sea".to_owned()),
            course_angle: 90.,
            wave_heading_angle: 90.,
            wave_length: wave_length.unwrap_or(10.),
            current_speed: 10.,
        });
        let mut ctx = MocEval {
            ctx: Context::new(initial),
        };
        if let Some(period_excitement) = period_excitement {
            ctx.ctx = ctx
                .ctx
                .clone()
                .write(
                    PeriodExcitementCtx {
                    period_excitement: *period_excitement,
                })
                .unwrap();
        }
        let result = PeriodExcitementEval::new("period_excitement_test", ctx).eval(());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<PeriodExcitementCtx>::read(&ctx)
                    .period_excitement
                    .clone();
                assert!(
                    (*target - result) < epsilon,
                    "step {} \nresult: {:?}\ntarget: {:?}",
                    step,
                    target,
                    result
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
