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
fn calculated_damaged_compartments_sofia() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbg = Dbg::new("test models", "calculated_damaged_compartments_sofia");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(3000));
    test_duration.run().unwrap();
    let dbg = Dbg::new("ShipModel", "calculated_damaged_compartments_sofia");
    let model_path = "src/assets/sofia.stl";
    let cache_dir = "src/algorithm/entities/cache/tests/";
    let center_coord = Some(Position::new(65.250, 0., 0.));
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
        thread_pool.scheduler().clone(),
    );
    let error = cache.rebuild();
    assert!(error.is_ok(), "*error*: {:?}", error);
    let epsilon_p = 0.01; //1%
    let epsilon_abs = 0.01; //1см
    let target = [
        [20., 20., 4.,   9758.8, 96.072, 0.369, 5.662],
        [20., -20., 4.,  9809.0, 33.856, 0.367, 5.787],
        [-20., -20., 4., 9809.0, 33.856, -0.367, 5.787],
        [-20., 20., 4.,  9758.9, 96.072, -0.369, 5.662],
        [0., 0., 4.,     6456.3, 66.877, 0., 2.053],
        [20., 0., 4.,    6527.4, 66.603, 1.769, 2.397],
        [-20., 0., 4.,   6527.4, 66.603, -1.769, 2.397],
        [0., 20., 4.,    9699.2, 96.306, 0., 5.623],
        [0., -20., 4.,   9749.2, 33.620, 0., 5.749],
    ];
    for target in target {
        let mut key = [None; 13];
        key[0] = Some(target[0]);
        key[1] = Some(target[1]);
        key[2] = Some(target[2]);
        let result: Result<Vec<f64>, Error> = cache.get(&key);
        assert!(result.is_ok(), "*error*: {:?}", result.unwrap_err());
        let result = result.unwrap();
        for (r, t) in result.iter().zip(target.iter()) {
            let delta = (r - t).abs();
            assert!(
                delta < epsilon_p * (r.abs().max(t.abs())) || delta < epsilon_abs,
                "\nresult: {:?}\ntarget: {:?}",
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
