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
use sal_core::error::Error;
use crate::{
    algorithm::{
        context::context_access::ContextRead, entities::{Bounds, data::Voyage}, eval::seakeeping::eval::{apparent_frequencies::apparent_frequencies_ctx::ApparentFrequenciesCtx, parametric_resonant_zone::parametric_resonant_zone_ctx::ParametricResonantZoneCtx, parametric_resonant_zone_speed_filter::{parametric_resonant_zone_speed_filter_ctx::ParametricResonantZoneSpeedFilterCtx, parametric_resonant_zone_speed_filter_eval::ParametricResonantZoneSpeedFilterEval}}
    }, kernel::{
        Eval, 
        types::{
            Arc, eval_result::EvalResult
        }
    }, prelude::{
        Context, 
        ContextWrite, 
        InitialCtx
    }
};
///
///
static INIT: Once = Once::new();
// Mock for ApiClient
struct MockApiClient;
//
impl MockApiClient {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
    // Заглушка для запроса к БД
    fn fetch(&self, _query: &str) -> Result<Vec<u8>, Error> {
        Ok(Vec::new())
    }
}
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
    DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
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
                left_side:  6.797646651599361,
                right_side: 7.513188404399294,
            },
            ApparentFrequenciesCtx {
                apparent_frequencies: vec![
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
            },
            vec![
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
        .write(parametric_resonant_zone.clone())
        .unwrap();
        ctx.ctx = ctx.ctx
        .clone()
        .write(apparent_frequencies.clone())
        .unwrap();
        let result = ParametricResonantZoneSpeedFilterEval::new(
            "parametric_resonant_zone_speed_filter",
            ctx
        ).eval(());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<ParametricResonantZoneSpeedFilterCtx>::read(&ctx).parametric_resonant_zone_speed_filter.clone();
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
