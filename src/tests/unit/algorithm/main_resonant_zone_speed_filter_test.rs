#[cfg(test)]
use std::{
    sync::Once, 
    time::Duration
};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{
    DebugSession, 
    LogLevel, 
};
use crate::{
    algorithm::{
        context::context_access::ContextRead, entities::{Bounds, data::Voyage}, eval::seakeeping::eval::{apparent_frequencies::apparent_frequencies_ctx::ApparentFrequenciesCtx, main_resonant_zone::main_resonant_zone_ctx::MainResonantZoneCtx, main_resonant_zone_speed_filter::{main_resonant_zone_speed_filter_ctx::MainResonantZoneSpeedFilterCtx, main_resonant_zone_speed_filter_eval::MainResonantZoneSpeedFilterEval}}
    }, 
    kernel::{
        Eval, 
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
/// Testing [main_resonant_zone_speed_filter](src/algorithm/eval/main_resonant_zone_speed_filter)
#[test]
fn main_resonant_zone_speed_filter() {
    DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = "MainResonantZoneSpeedFilterEval";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (
            1,
            0.0,
            MainResonantZoneCtx {
                left_side:  1.0,
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
            MainResonantZoneCtx {
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
    for (step, course_angle, main_resonant_zone, apparent_frequencies, target) in test_data.iter() {
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
            course_angle: *course_angle,
            wave_heading_angle: 90.,
            wave_length: 10.,
            current_speed: 10.,
        });            
        let mut ctx = MocEval {
            ctx: Context::new(
                initial,
            ),
        };
        ctx.ctx = ctx.ctx
        .clone()
        .write(main_resonant_zone.clone())
        .unwrap();
        ctx.ctx = ctx.ctx
        .clone()
        .write(apparent_frequencies.clone())
        .unwrap();
        let result = MainResonantZoneSpeedFilterEval::new("main_resonant_zone_speed_filter", ctx).eval(());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<MainResonantZoneSpeedFilterCtx>::read(&ctx).main_resonant_zone_speed_filter.clone();
                // assert!(result == *target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
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
impl Eval<(), EvalResult> for MocEval {
    fn eval(&self, _: ()) -> EvalResult {
        Result::Ok(self.ctx.clone())
    }
}
