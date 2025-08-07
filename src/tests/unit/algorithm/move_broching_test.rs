use std::collections::HashMap;
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
            move_broching_filter::{move_broching_filter_ctx::MoveBrochingFilterCtx, move_broching_filter_eval::MoveBrochingFilterEval}, vessel_max_speed::vessel_max_speed_ctx::VesselMaxSpeedCtx, Zg
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
/// Testing [move_broching_filter](src/algorithm/eval/move_broching_filter)
#[test]
fn move_broching_filter() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = "MoveBroching";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (
            1,
            0.0,
            1.0,
            1.0,
            vec![
                (135.0, 0.0), 
                (137.0, 0.0), 
                (137.1, 0.0), 
                (137.2, 0.0), 
                (137.3, 0.0), 
                (137.4, 0.0), 
                (137.5, 0.0), 
                (140.8, 0.0), 
                (140.9, 0.0), 
                (141.0, 0.0)
            ],
        ),
        (
            2,
            0.0,
            0.1,
            1.0,
            vec![
                (137.5, 0.0), 
                (140.8, 0.0), 
                (140.9, 0.0), 
                (143.7, 0.0), 
                (143.8, 0.0), 
                (147.1, 0.0), 
                (150.0, 0.0), 
                (150.1, 0.0), 
                (153.3, 0.0), 
                (153.4, 0.0)
            ],
        ),
    ];
    for (step, course_angle, length_lbp, vmax,target) in test_data.iter() {
        let mut initial = InitialCtx::new(
            0,
            "Unit-test",
        );
        initial.course_angle = Some(*course_angle);
        let mut ship_params = HashMap::new();
        ship_params.insert("LBP".to_owned(), *length_lbp);
        initial.ship_parameters = Some(ship_params);
        let mut ctx = MocEval {
            ctx: Context::new(
                initial
            ),
        };
        ctx.ctx = ctx.ctx
        .clone()
        .write(
            VesselMaxSpeedCtx {
                vmax: *vmax
            }
        ).unwrap();
        let result = MoveBrochingFilterEval::new("Test", ctx).eval(Zg::empty());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<MoveBrochingFilterCtx>::read(&ctx).move_broching_filter.clone();
                assert!(*target == result[0..10], "step {} \nresult: {:?}\ntarget: {:?}", step, &result[0..10], target);
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
