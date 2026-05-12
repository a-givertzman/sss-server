use crate::{
    algorithm::{
        context::context_access::ContextRead,
        entities::{Bounds, data::Voyage},
        eval::seakeeping::eval::move_broching_filter::{move_broching_filter_ctx::MoveBrochingFilterCtx, move_broching_filter_eval::MoveBrochingFilterEval},
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{Context, InitialCtx},
};
use debugging::session::debug_session::{DebugSession, LogLevel};
use std::collections::HashMap;
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
/// Testing [move_broching_filter](src/algorithm/eval/move_broching_filter)
#[test]
fn move_broching_filter() {
    DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = "MoveBroching";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
    test_duration.run().unwrap();
    let test_data = [
        (1, 0.0, 1.0, 1.0, vec![]),
        (
            2,
            0.0,
            0.1,
            1.0,
            vec![
                (161.6, 0.6),
                (161.7, 0.6),
                (161.8, 0.6),
                (161.9, 0.6),
                (162.0, 0.6),
                (162.1, 0.6),
                (162.2, 0.6),
                (162.3, 0.6),
                (162.4, 0.6),
                (162.5, 0.6),
            ],
        ),
    ];
    for (step, course_angle, length_lbp, vmax, target) in test_data.iter() {
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
            course_angle: *course_angle,
            wave_heading_angle: 90.,
            wave_length: 10.,
            current_speed: 10.,
        }); 
        let mut ship_params = HashMap::new();
        ship_params.insert("LBP".to_owned(), *length_lbp);
        initial.ship_parameters = Some(ship_params);
        let ctx = MocEval {
            ctx: Context::new(initial),
        };
        let result = MoveBrochingFilterEval::new("move_broching_filter", ctx).eval(());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<MoveBrochingFilterCtx>::read(&ctx)
                    .move_broching_filter
                    .clone();
                if result.len() == 0 {
                    assert!(
                        *target == result,
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        &result,
                        target
                    );
                } else {
                    assert!(
                        *target == result[0..10],
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        &result[0..10],
                        target
                    );
                }
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
