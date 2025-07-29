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
            MainResonantZoneCtx, 
            ParametricResonantZoneCtx, 
            VesselSpeedFilterCtx, 
            VesselSpeedFilterEval, 
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
/// Testing 'eval'
#[test]
fn eval() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = "eval";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (
            1,
            MainResonantZoneCtx {
                left_side:  1.0,
                right_side: 10.0,
            },
            ParametricResonantZoneCtx {
                left_side:  5.0,
                right_side: 7.0,
            },
            ApparentFrequenciesCtx {
                apparent_frequencies: vec![
                    (0.0, 6.283185307179586), 
                    (0.0, 6.492624817418906), 
                    (0.0, 6.702064327658225), 
                    (0.0, 6.911503837897545), 
                    (0.0, 7.120943348136865), 
                    (0.0, 7.330382858376184), 
                    (0.0, 7.5398223686155035), 
                    (0.0, 7.749261878854823), 
                    (0.0, 7.958701389094142), 
                    (0.0, 8.168140899333462)
                ],
            },
            vec![
                6.283185307179586, 
                6.492624817418906, 
                6.702064327658225, 
                6.911503837897545,             
            ] 
        ),
        (
            1,
            MainResonantZoneCtx {
                left_side:  7.0,
                right_side: 10.0,
            },
            ParametricResonantZoneCtx {
                left_side:  7.0,
                right_side: 10.0,
            },
            ApparentFrequenciesCtx {
                apparent_frequencies: vec![
                    (0.0, 6.283185307179586), 
                    (0.0, 6.492624817418906), 
                    (0.0, 6.702064327658225), 
                    (0.0, 6.911503837897545), 
                    (0.0, 7.120943348136865), 
                    (0.0, 7.330382858376184), 
                    (0.0, 7.5398223686155035), 
                    (0.0, 7.749261878854823), 
                    (0.0, 7.958701389094142), 
                    (0.0, 8.168140899333462)
                ],
            },
            vec![
                7.120943348136865,
                7.330382858376184,
                7.5398223686155035,
                7.749261878854823,
                7.958701389094142,
                8.168140899333462,             
            ] 
        ),
    ];
    for (step, main_resonant_zone, parametric_resonant_zone, apparent_frequencies, target) in test_data.iter() {
        let mut ctx = MocEval {
            ctx: Context::new(
                InitialCtx::new(
                    0,
                    "Unit-test",
                )
            ),
        };
        ctx.ctx = ctx.ctx
        .clone()
        .write(main_resonant_zone.clone())
        .unwrap();
        ctx.ctx = ctx.ctx
        .clone()
        .write(parametric_resonant_zone.clone())
        .unwrap();
            ctx.ctx = ctx.ctx
        .clone()
        .write(apparent_frequencies.clone())
        .unwrap();
        let result = VesselSpeedFilterEval::new("Test", ctx).eval(Zg::empty());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<VesselSpeedFilterCtx>::read(&ctx).vessel_speed_filter.clone();
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
