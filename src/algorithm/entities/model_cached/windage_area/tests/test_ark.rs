use debugging::session::debug_session::*;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
use testing::stuff::max_test_duration::TestDuration;

#[cfg(test)]
use crate::algorithm::entities::model_cached::local_cache::displacement_cache::DisplacementCache;
use crate::algorithm::entities::{model_cached::{LocalCache, Shape}, Position};
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
#[ignore = "no target values"]
#[test]
fn calculated_windage_area_ark() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let dbg = Dbg::new("test models", "calculated_windage_area_ark");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(3000));
    test_duration.run().unwrap();
    let dbg = Dbg::new("ShipModel", "calculated_windage_area_ark");
    let model_path = "assets/ark.stl";
    let additionals_path = "assets/ark_additionals/";
    let cache_dir = "src/algorithm/entities/cache/tests/";
    let center_coord = Some(Position::new(59.195, 0., 0.));
    let mut shape = AreaShape::new_uninit(&dbg, model_path.into(), Some(additionals_path.into()), center_coord, 1000.);
    shape.init().unwrap();
    let thread_pool = ThreadPool::new(&dbg, Some(20));
    let mut cache = AreaCache::new(
        &dbg,
        Arc::new(RwLock::new(shape)),
        cache_dir,
        vec![0.],
        vec![0., 2.],
        thread_pool.scheduler().clone(),
    );
    let error = cache.rebuild();
    assert!(error.is_ok(), "*error*: {:?}", error);
    let epsilon_p = 0.01; //1%
    let epsilon_abs = 0.01; //1см
    let target = [
        [0., 0., 1871.534, 63.109], 
        [0., 2., 1619.008, 62501.799], 
    ];
    for target in target {
        let mut key = [None; 4];
        key[0] = Some(target[0]);
        key[1] = Some(target[1]);
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
