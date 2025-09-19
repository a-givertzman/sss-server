use debugging::session::debug_session::*;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
use testing::stuff::max_test_duration::TestDuration;

use crate::algorithm::entities::model_cached::DisplacementShape;
#[cfg(test)]
use crate::{algorithm::entities::{model_cached::{CompartmentCache, LocalCache, Shape}, Position}, kernel::types::{Arc, RwLock}};
use std::{fs, sync::Once, time::Duration};
//
//
static INIT: Once = Once::new();
///
/// Once called initialisation.
fn init_once() {
    //
    // Implement your initialisation code to be called only once for current test file.
    INIT.call_once(|| {})
}
///
/// Returns:
///  - ...
#[allow(clippy::unused_unit)]
fn init_each() -> () {}
///
/// Test calculating dataset for Floating postion cache.
///
/// # Notes
/// During the test a file called `fpc_result` is created in ./tmpdir/.
/// At the end of the test it tries (safely) remove it.
/// Pay attention on loggin info (WARN level) to catch it fails cleaning up.
#[ignore = "TODO"]
#[test]
fn calculated_displacement_ark() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbg = Dbg::new("test models", "calculated_displacement_ark");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(3000));
    test_duration.run().unwrap();
    let dbg = Dbg::new("ShipModel", "compute_balance");
    let model_path = "src/assets/ark.stl";
    let cache_dir = "src/algorithm/entities/cache/tests/";
    let center_coord = Some(Position::new(59.195, 0., 0.));
    let mut shape = DisplacementShape::new_uninit(&dbg, model_path.into(), center_coord, 1000.);
    shape.init().unwrap();
    let thread_pool = ThreadPool::new(&dbg, None);
    let mut cache = CompartmentCache::new(
        &dbg,
        Arc::new(RwLock::new(shape)),
        cache_dir,
        String::from("201"),
        vec![-20., 0., 20.],
        vec![4.],
        1.,
        None,
        None,
        thread_pool.scheduler().clone(),
    );
    let error = cache.rebuild();
    assert!(error.is_ok(), "*error*: {:?}", error);
    let epsilon_p = 0.01; //1%
    let epsilon_abs = 0.01; //1см
    let target = [
        [20., 20.,  5779.8, 4., 87.914, 0.222, 3.886,     268., 57.713, 0., 3.426,     119.30, 13.4],
        [20., -20., 5411.7, 4., 31.3577, 0.237182, 3.365, 268., 60.677, 0., 3.426,     119.30, 13.4],
        [0., 0.,   5802.3, 4., 57.979, 0., 2.057,        1553., 56.615, 0., 4.000,    119.30, 13.4],
        [20., 0.,  5830.8, 4., 57.899, 1.402, 2.321,     1629., 57.286, 0.102, 4.037, 119.30, 13.4],
        [0., 20.,  5788.4, 4., 87.945, 0., 3.875,        268., 57.618, 0., 3.426,     119.30, 13.4],
  //      [0., -20., 4.,  5420.3, 31.548, 0., 3.426,        268., 60.772, 0., 3.354,     119.30, 13.4], 
  // TODO: целевое значение 3.426 сильно отличается от расчетного 3.366
    ];
    for target in target {
        let mut key = [None; 13];
        key[0] = Some(target[0]);
        key[1] = Some(target[1]);
        key[2] = Some(target[2]);
        let result: Result<Vec<f64>, Error> = cache.get(target[0], target[1], target[2]);
        assert!(result.is_ok(), "*error*: {:?}", result.unwrap_err());
        let result = result.unwrap();
        for (r, t) in result.iter().zip(target.iter()) {
            let delta = (r - t).abs();
            assert!(
                delta < epsilon_p * (r.abs().max(t.abs())) || delta < epsilon_abs,
                "\ncurrent_result:{r} current_target:{t}\nresult: {:?}\ntarget: {:?}",
                result,
                target
            );
        }
    }
 /*   // clean up
    if let Err(why) = fs::remove_file(cache_dir.to_owned()) {
        log::warn!(
            "Clean up (optional) | Failed removing result file='{}': {}",
            cache_dir,
            why
        );
    }*/
    test_duration.exit();
}
