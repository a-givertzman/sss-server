use debugging::session::debug_session::*;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
use testing::stuff::max_test_duration::TestDuration;

use crate::algorithm::entities::model_cached::{DamagedCompartmentCache, DisplacementShape};
#[cfg(test)]
use crate::{
    BoundDisplacementCache, Bounds,
    algorithm::entities::{
        Position,
        model_cached::{CompartmentCache, LocalCache, Shape},
    },
    kernel::types::{Arc, RwLock},
};
use std::{fs, path::PathBuf, sync::Once, time::Duration};
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
fn calculated_damaged_compartments_sofia() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbg = Dbg::new("test models", "bound_displacement_cache_sofia");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(3000));
    test_duration.run().unwrap();
    let cache_dir: PathBuf = "src/assets/cache/sofia".into();
    let model_dir: PathBuf = "src/assets/model/sofia".into();
    let model_center_coord = Position::new(65.250, 0., 0.);
    let bounds = Bounds::from_n(138.86, model_center_coord.x(), 20).unwrap();
    let bounds_length_mm = (bounds.length() * 1000.).ceil() as usize;
    let thread_pool = ThreadPool::new(&dbg, Some(15));
    let displacement_shape = Arc::new(RwLock::new(DisplacementShape::new_uninit(
        &dbg,
        model_dir.clone().join(PathBuf::from("hull.stl")),
        Some(model_center_coord),
        1000.,
    )));
    displacement_shape.write().init().unwrap();
    dbg!("displacement_shape init ok");
    let mut bound_cache = BoundDisplacementCache::new(
        &dbg,
        displacement_shape,
        cache_dir
            .clone()
            .join("disp_bounded")
            .join(format!("{bounds_length_mm}")),
        1.,
        bounds.clone(),
        thread_pool.scheduler(),
    );
    bound_cache.rebuild().unwrap();
    let result = bound_cache.get(5.9, 0.);
/*
    let target_draught = target[3];
    let target_center = Position::new(target[4], target[5], target[6]);
    let (result_draught, result_center) = cache.get(target[0], target[1], target[2]).unwrap();
    let delta = (result_draught - target_draught).abs();
    assert!(
        delta < epsilon_p * (result_draught.abs().max(target_draught.abs())) || delta < epsilon_abs,
        "\nresult: {:?}\ntarget: {:?}",
        result_draught,
        target_draught
    );
    let delta = (result_center - target_center).len();
    assert!(
        delta < epsilon_abs,
        "\nresult: {:?}\ntarget: {:?}",
        result_center,
        target_center
    );
*/
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
