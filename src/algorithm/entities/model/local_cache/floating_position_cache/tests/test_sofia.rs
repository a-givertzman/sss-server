#[cfg(test)]
use crate::algorithm::entities::model::{
    local_cache::floating_position_cache::{
        floating_position_cache_conf::FloatingPositionCacheConf, CalculatedFloatingPositionCache,
        FloatingPositionCache,
    },
    ModelTree,
};
use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use sal_3dlib::{props::Center, topology::shape::Shape};
use sal_core::{dbg::Dbg, error::Error};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    sync::{Arc, Once},
    time::Duration,
};
use testing::stuff::max_test_duration::TestDuration;
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
#[test]
fn calculated_floating_position_sofia() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbg = Dbg::new("test models", "calculated_floating_position_sofia");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(3000));
    test_duration.run().unwrap();
    let model_path = "src/assets/sofia.stp";
    let result_path = "src/algorithm/entities/model/local_cache/floating_position_cache/tests/assets/sofia_result";
    // create model tree with empty attribute for each model
    let model_tree = ModelTree::<()>::new(&dbg, model_path)
        .load()
        .unwrap_or_else(|err| panic!("Failing building *model_tree*: {}", err));
    // set waterline init position to target model center
    //  (2, 'LCG from middle', 59.837, 2),
    // (2, 'TCG from CL', -0.44, 2),  
    // (2, 'VCG from BL', 7.81, 2),
    let waterline_position = [59.837, -0.44, 7.81];
    let conf = FloatingPositionCacheConf {
        waterline_position,
        heel_steps: vec![0.],//vec![-2., -1., 0., 1., 2.],//(-10..=10).step_by(1).map(|n| n as f64).collect(),
        trim_steps: vec![0.],//vec![-2., -1., 0., 1., 2.],//(-8..=8).step_by(1).map(|n| (n as f64)*0.25).collect(),
        draught_steps: vec![4.],//vec![2., 3., 4., 5., 6., 7., 8.,],//(8..=16).step_by(1).map(|n| (n as f64)*0.25).collect(), 
    };
    let heel_steps = conf.heel_steps.clone();
    let trim_steps = conf.trim_steps.clone();
    let draught_steps = conf.draught_steps.clone();
    let errors = CalculatedFloatingPositionCache::new(
        &dbg,
        result_path.into(),
        model_tree.iter().map(|(_, shape)| shape).cloned().collect(),
        FloatingPositionCache::new(&dbg, model_tree, result_path, conf)
            .create_waterline()
            .unwrap_or_else(|err| panic!("Failed creating *waterline*: {}", err)),
        heel_steps,
        trim_steps,
        draught_steps,
        Arc::default(),
    )
    .build();
    assert!(errors.is_empty(), "*errors*: {:?}", errors);
    // clean up
 /*   if let Err(why) = fs::remove_file(result_path) {
        log::warn!(
            "Clean up (optional) | Failed removing result file='{}': {}",
            result_path,
            why
        );
    }*/
    test_duration.exit();
}

