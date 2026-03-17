use crate::{
    algorithm::{
        entities::{Bounds, data::Voyage}, eval::seakeeping::eval::{apparent_frequencies::{apparent_frequencies_ctx::ApparentFrequenciesCtx, apparent_frequencies_eval::ApparentFrequenciesEval}, period_excitement::period_excitement_ctx::PeriodExcitementCtx}
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
/// Testing [apparent_frequencies](src/algorithm/eval/apparent_frequencies)
#[test]
fn apparent_frequencies() {
    DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
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
            20.0,
            6.0,
            vec![
                (0.0, 0.0, 1.0471975511965976),
                (0.0, 0.1, 1.0530153153699122),
                (0.0, 0.2, 1.0588330795432266),
                (0.0, 0.3, 1.0646508437165412),
                (0.0, 0.4, 1.0704686078898553),
                (0.0, 0.5, 1.07628637206317),
                (0.0, 0.6, 1.0821041362364843),
                (0.0, 0.7, 1.0879219004097986),
                (0.0, 0.8, 1.0937396645831132),
                (0.0, 0.9, 1.0995574287564276),
            ],
        ),
        // (
        //     2,
        //     10.0,
        //     2.0,
        //     vec![
        //         (0.0, 0.0, 3.141592653589793),
        //         (0.0, 0.1, 3.193952531149623),
        //         (0.0, 0.2, 3.246312408709453),
        //         (0.0, 0.3, 3.2986722862692823),
        //         (0.0, 0.4, 3.3510321638291125),
        //         (0.0, 0.5, 3.4033920413889427),
        //         (0.0, 0.6, 3.4557519189487724),
        //         (0.0, 0.7, 3.5081117965086026),
        //         (0.0, 0.8, 3.5604716740684323),
        //         (0.0, 0.9, 3.6128315516282625)
        //     ]
        // ),
    ];
    for (step, vmax, period_excitement, target) in test_data.iter() {
        let mut initial = InitialCtx::new(
            "0",
            "Unit-test",
            Bounds::from_min_max(0., 100., 20).unwrap(),
        );        
        initial.voyage = Some(Voyage {
            density: 1.025,
            operational_speed: *vmax,
            icing_type: "none".to_owned(),
            icing_timber_type: "full".to_owned(),
            area: Some("sea".to_owned()),
            course_angle: 90.,
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
        let result = ApparentFrequenciesEval::new("apparent_frequencies", ctx).eval(());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<ApparentFrequenciesCtx>::read(&ctx)
                    .apparent_frequencies
                    .clone();
                println!("{:?}", result);
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
