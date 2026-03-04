use debugging::session::debug_session::*;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
use testing::stuff::max_test_duration::TestDuration;

use crate::algorithm::entities::model_cached::WindageArea;
#[cfg(test)]
use crate::{
    algorithm::entities::{
        Position,
        model_cached::{AreaShape, LocalCache, Shape},
    },
    kernel::types::{Arc, RwLock},
};
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
#[ignore = "too slow, run only in release mode"]
#[test]
fn calculated_windage_area_sofia() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let dbg = Dbg::new("test models", "calculated_windage_area_sofia");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(3000));
    test_duration.run().unwrap();
    let dbg = Dbg::new("ShipModel", "calculated_windage_area_sofia");
    let model_path = "assets/model/sofia/hill.stl";
    let additionals_path = "assets/model/sofia/additionals/";
    let cache_dir = "src/algorithm/entities/cache/tests/";
    let center_coord = Some(65.250);
    let mut shape = AreaShape::new_uninit(
        &dbg,
        model_path.into(),
        Some(additionals_path.into()),
        center_coord,
        1000.,
    );
    shape.init().unwrap();
    let mut area = WindageArea::new(&dbg, Arc::new(RwLock::new(shape)), cache_dir, 2.);
    let error = area.rebuild();
    assert!(error.is_ok(), "*error*: {:?}", error);
    let epsilon_p = 0.01; //1%
    let epsilon_abs = 0.01; //1см
    let target = (1619.008, 62.501);
    let result = area.windage_area();
    assert!(result.is_ok(), "*error*: {:?}", result.unwrap_err());
    let result = result.unwrap();
    {
        let (r, t) = (result.0, target.0);
        let delta = (r - t).abs();
        assert!(
            delta < epsilon_p * (r.abs().max(t.abs())) || delta < epsilon_abs,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
    }
    {
        let (r, t) = (result.1, target.1);
        let delta = (r - t).abs();
        assert!(
            delta < epsilon_p * (r.abs().max(t.abs())) || delta < epsilon_abs,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
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
