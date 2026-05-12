#[cfg(test)]
use crate::algorithm::entities::ship_model::ship_model::ShipModel;
use crate::algorithm::entities::{Bounds, Moment, Position};
use crate::algorithm::eval::icing_timber::ctx::IcingTimberCtx;
use crate::algorithm::eval::strength::IcingStrCtx;
use crate::algorithm::eval::strength::icing::eval::IcingStrEval;
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
fn strength_icing() {
    DebugSession::new()
        .filter(LogLevel::Debug)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
    init_once();
    init_each();
    let dbg = "strength_icing";
    let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
    test_duration.run().unwrap();

    // 1. Подготовка Bounds (2 шпации)
    let bounds = Bounds::from_min_max(0.0, 20.0, 2).unwrap();

    // 2. Подготовка коэффициентов (w_ice_v = 0.03, w_desc = 0.03, w_timber = 0.04)
    let icing_coeff = IcingCoeffCtx {
        w_desc: 0.03,
        w_timber: 0.04,
        w_ice_v: 0.03,
        is_some: true,
        coef_v_area: 0.1,
        coef_v_ds_area: 0.05,
        coef_v_moment: 0.2,
    };

    // 3. Распределенные данные по грузам (по 2 элемента для 2 шпаций)
    let unit_area = UnitAreaCtx {
        av_dc: 10.0,
        mv_dc: Moment::from_pos(Position::new(20.0, 0.0, 12.0), 10.0),
        delta_moment_h: Moment::from_pos(Position::new(0.0, 0.0, 2.0), 5.0),
        distr_v: vec![5.0, 5.0],
    };

    let icing_timber = IcingTimberCtx {
        area_icing: 50.0,
        area_no_icing: 10.0,
        moment_icing: Moment::from_pos(Position::new(20.0, 0.0, 12.0), 50.0),
        moment_no_icing: Moment::from_pos(Position::new(20.0, 0.0, 8.0), 10.0), // Часть палубы под лесом
        delta_moment_icing: Moment::from_pos(Position::new(0.0, 0.0, 4.0), 50.0),
        area_icing_distr: vec![10.0, 0.0],    // Лес только в первой шпации
        area_no_icing_distr: vec![10.0, 0.0], // Площадь под лесом
    };

    // 4. Мок модели (базовая парусность 100, палуба 200)
    let ship = ShipModel::create_test_fake_strength(
        vec![100.0, 100.0], 
        vec![200.0, 200.0]
    );
    let model = Arc::new(RwLock::new(ship));

    let ctx = Context::new(InitialCtx::new("1", "NULL", bounds))
    .write(icing_coeff)
    .unwrap()
    .write(unit_area)
    .unwrap()
    .write(icing_timber)
    .expect("Failed to build context for test");
    
    let mock_eval = MockEval { ctx };

    // 5. Расчет
    let result: IcingStrCtx = IcingStrEval::new(dbg, model, mock_eval)
        .eval(())
        .unwrap()
        .read();

    // РУЧНОЙ РАСЧЕТ (Шпация 0):
    // p_ice_v = (100 + 5) * 0.03 = 3.15
    // p_ice_hdeck = 0.03 * (200 - 10) = 5.7
    // delta_p_ice = 10 * (0.04 - 0.03) = 0.1
    // Total = 3.15 + 5.7 + 0.1 = 8.95
    
    let epsilon = 1e-5;
    assert!((result.mass_values[0] - 8.95).abs() < epsilon, "Result: {}", result.mass_values[0]);
    
    test_duration.exit();
}
//
#[derive(Debug, Clone)]
struct MockEval {
    pub ctx: Context,
}
//
impl Eval<(), EvalResult> for MockEval {
    fn eval(&self, _: ()) -> EvalResult {
        Result::Ok(self.ctx.clone())
    }
}

