use crate::algorithm::entities::cache::*;
#[cfg(test)]
use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use sal_core::dbg::Dbg;
use std::{sync::Once, time::Duration};
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
/// Test successfull initializing of [Cache] instance.
#[test]
fn init_cache() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbg = Dbg::new("cache", "init_cache");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    // init
    //
    #[rustfmt::skip]
    let data = vec![
        vec![0.0, 0.0, 0.0, 10.0],
        vec![2.1, 0.1, 0.1, 20.1],
        vec![3.2, 1.2, 0.2, 30.2],
        vec![4.3, 0.3, 1.3, 40.3],
        vec![5.4, 2.4, 2.4, 50.4],
        vec![0.5, 3.5, 0.5, 60.5],
        vec![0.6, 4.6, 3.6, 70.6],
        vec![0.7, 0.7, 4.7, 80.7],
    ];
    let cache = Cache::new(&dbg);
    let result = cache.init(data.clone());
    println!("cache.init result={:?}", result);
    for (step, target) in data.into_iter().enumerate() {
        let vals = [Some(target[0]), Some(target[1]), Some(target[2]), None];
        let result = cache.get(&vals).unwrap().first().unwrap().clone();
        println!(
            "step={} vals={:?} target={:?} result={:?}",
            step, vals, target, result
        );
        assert_eq!(
            target, result,
            "step={} vals={:?} target={:?} result={:?}",
            step, vals, target, result
        );
    }
    test_duration.exit();
}
/*
///
/// Test failure initializing of [Cache] instance.
#[test]
fn init_cache_table_from_inconsistent_files() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let callee = "init_cache_table_from_inconsistent_files";
    let dbg = Dbg::new("cache", "init_cache_table_from_inconsistent_files");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        ("src/tests/unit/algorithm/cache/assets/table-inc-row", 6),
        ("src/tests/unit/algorithm/cache/assets/table-inc-col", 8),
    ];
    for (path, target) in test_data {
        let result = Cache::<f64>::new(&dbg, path).init();
        match result {
            Err(error) => {
                let line_info = format!("line={}", target);
                assert!(error.to_string().contains(&line_info));
            }
            Ok(_) => panic!("{}.{} | Must fail to create Cache", dbg, callee),
        }
    }
    test_duration.exit();
}
*/
