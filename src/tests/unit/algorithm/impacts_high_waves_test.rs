use crate::{
    algorithm::{
        entities::{Bounds, data::Voyage}, eval::seakeeping::eval::{impacts_high_waves::{impacts_high_waves_ctx::ImpactsHighWavesCtx, impacts_high_waves_eval::ImpactsHighWavesEval}, period_excitement::period_excitement_ctx::PeriodExcitementCtx},
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
                (224.99999999999997, 28.183862084533555), 
                (224.99999999999997, 22.761767286395028), 
                (224.89999999999998, 18.21159747479397), 
                (135.0, 18.24335495461293), 
                (135.0, 28.08345292160507), 
                (221.2, 26.486715991910295)
            ],
        ),
        (
            2,
            0.0,
            1.0,
            vec![
                (224.99999999999997, 2.8183862084533557), 
                (224.99999999999997, 2.065317486489671), 
                (224.89999999999998, 1.821159747479397), 
                (135.0, 1.8243354954612927), 
                (135.0, 2.808345292160507), 
                (224.49999999999997, 2.794109702087563)
            ],
        ),
    ];
    for (step, course_angle, period_excitement, target) in test_data.iter() {
        let mut initial = InitialCtx::new(
            "0",
            "Unit-test",
            Bounds::from_min_max(0., 100., 20).unwrap(),
        );
        initial.voyage = Some(Voyage {
            density: 1.025,
            operational_speed: 12.,
            icing_type: "none".to_owned(),
            icing_timber_type: "full".to_owned(),
            area: Some("sea".to_owned()),
            course_angle: *course_angle,
            wave_heading_angle: 90.,
            wave_length: 10.,
            current_speed: 10.,
        });
        let mut ctx = MocEval {
            ctx: Context::new(initial),
        };
        ctx.ctx = ctx
            .ctx
            .clone()
            .write(PeriodExcitementCtx {
                period_excitement: *period_excitement,
            })
            .unwrap();
        let result = ImpactsHighWavesEval::new("impacts_high_waves", ctx).eval(());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<ImpactsHighWavesCtx>::read(&ctx)
                    .impacts_high_waves
                    .clone();
                assert!(
                    result[0..result.len()] == *target,
                    "step {} \nresult: {:?}\ntarget: {:?}",
                    step,
                    &result[0..result.len()],
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
