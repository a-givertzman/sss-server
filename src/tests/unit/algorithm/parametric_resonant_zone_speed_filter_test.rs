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
            ParametricResonantZoneCtx, 
            ParametricResonantZoneSpeedFilterCtx, 
            ParametricResonantZoneSpeedFilterEval, 
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
/// Testing [parametric_resonant_zone_speed_filter](src/algorithm/eval/parametric_resonant_zone_speed_filter)
#[test]
fn parametric_resonant_zone_speed_filter() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = "ParametricResonantZoneSpeedFilterEval";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (
            1,
            0.0,
            ParametricResonantZoneCtx {
                left_side:  5.0,
                right_side: 7.0,
            },
            ApparentFrequenciesCtx {
                apparent_frequencies: vec![
                    (0.0, 0.1, 6.283185307179586), 
                    (0.0, 0.2, 6.492624817418906), 
                    (0.0, 0.3, 6.702064327658225), 
                    (0.0, 0.4, 6.911503837897545), 
                    (0.0, 0.0, 7.120943348136865), 
                    (0.0, 0.0, 7.330382858376184), 
                    (0.0, 0.0, 7.5398223686155035), 
                    (0.0, 0.0, 7.749261878854823), 
                    (0.0, 0.0, 7.958701389094142), 
                    (0.0, 0.0, 8.168140899333462)
                ],
            },
            vec![
                (0.0, 0.1),
                (0.0, 0.2),
                (0.0, 0.3),
                (0.0, 0.4),
            ] 
        ),
        (
            1,
            0.0,
            ParametricResonantZoneCtx {
                left_side:  7.0,
                right_side: 10.0,
            },
            ApparentFrequenciesCtx {
                apparent_frequencies: vec![
                    (0.0, 0.0, 6.283185307179586), 
                    (0.0, 0.0, 6.492624817418906), 
                    (0.0, 0.0, 6.702064327658225), 
                    (0.0, 0.0, 6.911503837897545), 
                    (0.0, 0.1, 7.120943348136865), 
                    (0.0, 0.2, 7.330382858376184), 
                    (0.0, 0.3, 7.5398223686155035), 
                    (0.0, 0.4, 7.749261878854823), 
                    (0.0, 0.5, 7.958701389094142), 
                    (0.0, 0.6, 8.168140899333462)
                ],
            },
            vec![
                (0.0, 0.1),
                (0.0, 0.2),
                (0.0, 0.3),
                (0.0, 0.4),
                (0.0, 0.5),
                (0.0, 0.6),           
            ] 
        ),
    ];
    for (step, course_angle, parametric_resonant_zone, apparent_frequencies, target) in test_data.iter() {
        let mut initial_data = InitialCtx::new(
            0,
            "Unit-test",
        );
        initial_data.course_angle = Some(*course_angle);
        let mut ctx = MocEval {
            ctx: Context::new(
                initial_data,
            ),
        };
        ctx.ctx = ctx.ctx
        .clone()
        .write(parametric_resonant_zone.clone())
        .unwrap();
        ctx.ctx = ctx.ctx
        .clone()
        .write(apparent_frequencies.clone())
        .unwrap();
        let result = ParametricResonantZoneSpeedFilterEval::new("Test", ctx).eval(Zg::empty());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<ParametricResonantZoneSpeedFilterCtx>::read(&ctx).parametric_resonant_zone_speed_filter.clone();
                assert!(result == *target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
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
