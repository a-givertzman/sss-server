use crate::algorithm::entities::{model::{local_cache::displacement_cache::build_displacement_cache::BuildDisplacementCache, LocalCache}, Position};
#[cfg(test)]
use crate::algorithm::entities::model::{
    local_cache::displacement_cache::{
        displacement_cache_conf::DisplacementCacheConf,
        DisplacementCache,
    },
};
use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::{Scheduler, ThreadPool};
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
fn calculated_displacement_cache() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbg = Dbg::new("test models", "Calculated_displacement_cache");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(300));
    test_duration.run().unwrap();
    let thread_pool = ThreadPool::new(&dbg, Some(12));
    let model_key = "/cube_1_1_1_centered";
    let model_path =
        "src/assets/cube_1_1_1.step";
    let target_path =
        "src/algorithm/entities/model/local_cache/displacement_cache/tests/assets/fpc_target";
    let result_path = "src/algorithm/entities/model/local_cache/displacement_cache/tests/assets";
  //      "src/algorithm/entities/model/local_cache/displacement_cache/tests/assets/fpc_result";
    // set waterline init position to target model center
    let center_coord = Position::new(5., 5., 5.);
    let conf = DisplacementCacheConf {
        center_coord,
        heel_steps: (-10..=10).step_by(5).map(|n| n as f64).collect(),
        trim_steps: (-10..=10).step_by(5).map(|n| n as f64).collect(),
        draught_steps: vec![0.0, 0.25],
    };
    // let error = DisplacementCache::new(
    //     &dbg,
    //     model_path.into(),
    //     1.,
    //     result_path,
    //     conf,
    //     thread_pool.scheduler(),
    // ).rebuild();
    // assert!(error.is_ok(), "*error*: {:?}", error);
 /*   let errors = BuildDisplacementCache::new(
        &dbg,
        model_shape.iter().map(|(_, shape)| shape).cloned().collect(),
        center_coord,
        heel_steps,
        trim_steps,
        draught_steps,
        thread_pool.scheduler(),
        Arc::default(),
    )
    .build();
    assert!(errors.is_empty(), "*errors*: {:?}", errors);*/
    // read target file
    let mut target_reader = {
        let target_file = File::open(target_path)
            .unwrap_or_else(|err| panic!("Failed opening target file='{}': {}", target_path, err));
        BufReader::new(target_file)
    };
    // read result file
    let result_path = result_path.to_owned() + &"/floating_position_cache";
    let mut result_reader = {
        let result_file = File::open(&result_path)
            .unwrap_or_else(|err| panic!("Failed opening result file='{}': {}", &result_path, err));
        BufReader::new(result_file)
    };
    let target: Vec<String> = target_reader.by_ref().lines().filter_map(|v| v.ok()).collect();
    let result: Vec<String> = result_reader.by_ref().lines().filter_map(|v| v.ok()).collect();

    assert_eq!(
        target.len(), result.len(),
        "target.len='{}' result.len()='{}'", 
        target.len(), result.len()
    );
    for line in &result {
        assert!(
            target.contains(line),
            "line={} target='{:?}' result='{:?}'",
            line, target, result
        );
    }
    // clean up
    if let Err(why) = fs::remove_file(&result_path) {
        log::warn!(
            "Clean up (optional) | Failed removing result file='{}': {}",
            &result_path,
            why
        );
    }
    test_duration.exit();
}

