use crate::algorithm::entities::model::local_cache::floating_position_cache::calculated_floating_position_cache::CalculatedFloatingPositionCache;
#[cfg(test)]
use crate::algorithm::entities::model::{
    local_cache::floating_position_cache::{
        floating_position_cache_conf::FloatingPositionCacheConf,
        FloatingPositionCache,
    },
    ModelTree,
};
use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use sal_3dlib::{props::Center, topology::shape::Shape};
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
fn calculated_floating_position_cache() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbg = Dbg::new("test models", "Calculated_floating_position_cache");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(300));
    test_duration.run().unwrap();
    let thread_pool = ThreadPool::new(&dbg, Some(12));
    let model_key = "/cube_1_1_1_centered";
    let model_path =
        "src/assets/cube_1_1_1.step";
    let target_path =
        "src/algorithm/entities/model/local_cache/floating_position_cache/tests/assets/fpc_target";
    let result_path =
        "src/algorithm/entities/model/local_cache/floating_position_cache/tests/assets/fpc_result";
    // create model tree with empty attribute for each model
    let model_tree = ModelTree::<()>::new(&dbg, model_path)
        .load()
        .unwrap_or_else(|err| panic!("Failing building *model_tree*: {}", err));
    // set waterline init position to target model center
    let waterline_position = model_tree
        .get(model_key)
        .and_then(|shape| match shape {
            Shape::Solid(model) => Some(model.center().point()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Expected Solid by model_key='{}'", model_key));
    let conf = FloatingPositionCacheConf {
        waterline_position,
        heel_steps: (-10..=10).step_by(5).map(|n| n as f64).collect(),
        trim_steps: (-10..=10).step_by(5).map(|n| n as f64).collect(),
        draught_steps: vec![0.0, 0.25],
    };
    let heel_steps = conf.heel_steps.clone();
    let trim_steps = conf.trim_steps.clone();
    let draught_steps = conf.draught_steps.clone();
    let errors = CalculatedFloatingPositionCache::new(
        &dbg,
        result_path.into(),
        model_tree.iter().map(|(_, shape)| shape).cloned().collect(),
        FloatingPositionCache::new(&dbg, model_tree, result_path, conf, thread_pool.scheduler())
            .create_waterline()
            .unwrap_or_else(|err| panic!("Failed creating *waterline*: {}", err)),
        heel_steps,
        trim_steps,
        draught_steps,
        thread_pool.scheduler(),
        Arc::default(),
    )
    .build();
    assert!(errors.is_empty(), "*errors*: {:?}", errors);
    // read target file
    let mut target_reader = {
        let target_file = File::open(target_path)
            .unwrap_or_else(|err| panic!("Failed opening target file='{}': {}", target_path, err));
        BufReader::new(target_file)
    };
    // read result file
    let mut result_reader = {
        let result_file = File::open(result_path)
            .unwrap_or_else(|err| panic!("Failed opening result file='{}': {}", result_path, err));
        BufReader::new(result_file)
    };
    // check files line-by-line
    for ((try_target_line, try_result_line), line_id) in target_reader
        .by_ref()
        .lines()
        .zip(result_reader.by_ref().lines())
        .zip(1..)
    {
        let target = try_target_line
            .unwrap_or_else(|err| panic!("line={} | Failed getting target line: {}", line_id, err));
        let result = try_result_line
            .unwrap_or_else(|err| panic!("line={} | Failed getting result line: {}", line_id, err));
        assert_eq!(
            target, result,
            "line={} target='{}' result='{}'",
            line_id, target, result
        );
    }
    // check remaining lines in both files
    let remaining_target_lines = target_reader.lines().count();
    assert_eq!(
        remaining_target_lines, 0,
        "*result_file*.lines.count < *target_file*.lines.count"
    );
    assert_eq!(
        0,
        result_reader.lines().count(),
        "*result_file*.lines.count > *target_file*.lines.count"
    );
    // clean up
    if let Err(why) = fs::remove_file(result_path) {
        log::warn!(
            "Clean up (optional) | Failed removing result file='{}': {}",
            result_path,
            why
        );
    }
    test_duration.exit();
}

