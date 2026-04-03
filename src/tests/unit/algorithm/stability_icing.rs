#[cfg(test)]
use crate::algorithm::entities::ship_model::ship_model::ShipModel;
use crate::algorithm::entities::{Bounds, Moment, Position};
use crate::algorithm::eval::icing_timber::ctx::IcingTimberCtx;
use crate::algorithm::eval::stability::IcingStabCtx;
use crate::algorithm::eval::stability::icing::eval::IcingStabEval;
use crate::algorithm::eval::{IcingCoeffCtx, UnitAreaCtx};
use crate::kernel::Eval;
use crate::kernel::types::Arc;
use crate::kernel::types::eval_result::EvalResult;
use crate::prelude::*;
use debugging::session::debug_session::{DebugSession, LogLevel};
use sal_sync::sync::RwLock;
use std::sync::Once;
use std::time::Duration;
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
fn stability_icing() {
    DebugSession::new()
        .filter(LogLevel::Debug)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = "stability_icing";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
    test_duration.run().unwrap();
    // 1. Геометрия судна (FakeModel)
    // Палуба: 200 м2, центр (25.0, 0.0, 8.0)
    // Парусность: 100 м2, центр (20.0, 0.0, 10.0)
    let ship_area_v = 100.0;
    let ship_pos_v = Position::new(20.0, 0.0, 10.0);
    let ship_area_h = 200.0;
    let ship_pos_h = Position::new(25.0, 0.0, 8.0);

    let ship = ShipModel::create_test_fake(
        ship_area_h,
        ship_pos_h,
        ship_area_v,
        Moment::from_pos(ship_pos_v, ship_area_v),
    );
    let model = Arc::new(RwLock::new(ship));

    // 2. Входные параметры (коэффициенты и грузы)
    let w_v = 0.03; // кг/м2 вертикаль
    let w_h = 0.03; // кг/м2 горизонталь
    let w_t = 0.04; // кг/м2 лес

    let icing_coeff = IcingCoeffCtx {
        w_desc: w_h,
        w_timber: w_t,
        w_ice_v: w_v,
        is_some: true,
        coef_v_area: 0.1,
        coef_v_ds_area: 0.05,
        coef_v_moment: 0.2,
    };

    // Лесной груз: 50 м2, центр на 12.0м (на 4м выше палубы)
    let icing_timber = IcingTimberCtx {
        area_icing: 50.0,
        area_no_icing: 10.0,
        moment_icing: Moment::from_pos(Position::new(20.0, 0.0, 12.0), 50.0),
        moment_no_icing: Moment::from_pos(Position::new(20.0, 0.0, 8.0), 10.0), // Часть палубы под лесом
        delta_moment_icing: Moment::from_pos(Position::new(0.0, 0.0, 4.0), 50.0),
        area_icing_distr: vec![],
        area_no_icing_distr: vec![],
    };

    // Палубный груз: +10 м2 парусности на высоте 12.0м
    let unit_area = UnitAreaCtx {
        av_dc: 10.0,
        mv_dc: Moment::from_pos(Position::new(20.0, 0.0, 12.0), 10.0),
        delta_moment_h: Moment::from_pos(Position::new(0.0, 0.0, 2.0), 5.0),
        distr_v: vec![],
    };

    // 3. Расчет
    let initial = InitialCtx::new(
        "0",
        "MomentTest",
        Bounds::from_min_max(0., 100., 100).unwrap(),
    );
    let ctx = Context::new(initial)
        .write(icing_coeff)
        .unwrap()
        .write(icing_timber)
        .unwrap()
        .write(unit_area)
        .expect("Failed to build context for test");
    let evaluator = IcingStabEval::new("test", model, MockEval { ctx });
    match evaluator.eval(()) {
        Ok(res_ctx) => {
            let result: IcingStabCtx = res_ctx.read();

            let target_mass = 9.5;
            let target_moment_x = 212.5; 
            let target_moment_z = 89.1;

            let epsilon = 1e-5;

            // 2. Проверка массы
            assert!(
                (result.mass - target_mass).abs() < epsilon,
                "\n[Mass Mismatch]\nresult: {:?}\ntarget: {:?}",
                result.mass,
                target_mass
            );

            // 3. Проверка моментов по осям
            assert!(
                (result.moment.x() - target_moment_x).abs() < epsilon,
                "\n[Moment X Mismatch]\nresult: {:?}\ntarget: {:?}",
                result.moment.x(),
                target_moment_x
            );

            assert!(
                (result.moment.z() - target_moment_z).abs() < epsilon,
                "\n[Moment Z Mismatch]\nresult: {:?}\ntarget: {:?}",
                result.moment.z(),
                target_moment_z
            );

            log::info!(
                "Test passed for mass {} and moment Z {}",
                result.mass,
                result.moment.z()
            );
        }
        Err(err) => panic!("\nEvaluation error: {:?}", err),
    }
}

#[derive(Debug, Clone)]
struct MockEval {
    pub ctx: Context,
}
//
//
impl Eval<(), EvalResult> for MockEval {
    fn eval(&self, _: ()) -> EvalResult {
        Result::Ok(self.ctx.clone())
    }
}
