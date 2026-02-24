use crate::{
    algorithm::{
        context::context_access::ContextRead,
        entities::Bounds, eval::seakeeping::eval::{impacts_high_waves::{impacts_high_waves_ctx::ImpactsHighWavesCtx, impacts_high_waves_eval::ImpactsHighWavesEval}, period_excitement::period_excitement_ctx::PeriodExcitementCtx},
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
/// Testing [impacts_high_waves](src/algorithm/eval/impacts_high_waves)
#[test]
fn impacts_high_waves() {
    DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
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
                (135.0, 18.24335495461293),
                (135.0, 18.34376411754142),
                (135.0, 18.44417328046991),
                (135.0, 18.544582443398404),
                (135.0, 18.644991606326894),
                (135.0, 18.745400769255383),
                (135.0, 18.845809932183876),
                (135.0, 18.946219095112365),
                (135.0, 19.046628258040858),
                (135.0, 19.14703742096935),
            ],
        ),
        (
            2,
            0.0,
            1.0,
            vec![
                (135.0, 1.824335495461293),
                (135.0, 1.834376411754142),
                (135.0, 1.844417328046991),
                (135.0, 1.8544582443398403),
                (135.0, 1.8644991606326893),
                (135.0, 1.8745400769255385),
                (135.0, 1.8845809932183877),
                (135.0, 1.8946219095112367),
                (135.0, 1.9046628258040859),
                (135.0, 1.914703742096935),
            ],
        ),
    ];
    for (step, course_angle, period_excitement, target) in test_data.iter() {
        let mut initial_data = InitialCtx::new(
            "0",
            "Unit-test",
            Bounds::from_min_max(0., 100., 20).unwrap(),
        );
        initial_data.course_angle = Some(*course_angle);
        let mut ctx = MocEval {
            ctx: Context::new(initial_data),
        };
        ctx.ctx = ctx
            .ctx
            .clone()
            .write(PeriodExcitementCtx {
                period_excitement: *period_excitement,
            })
            .unwrap();
        let result = ImpactsHighWavesEval::new(ctx).eval(());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<ImpactsHighWavesCtx>::read(&ctx)
                    .impacts_high_waves
                    .clone();
                assert!(
                    result[0..10] == *target,
                    "step {} \nresult: {:?}\ntarget: {:?}",
                    step,
                    &result[0..10],
                    target
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
