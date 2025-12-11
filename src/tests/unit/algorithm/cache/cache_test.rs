use crate::algorithm::entities::cache::*;
#[cfg(test)]
use debugging::session::debug_session::{DebugSession, LogLevel};
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
#[test]
fn init_cache() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let dbg = Dbg::new("cache", "cache_get");
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
        let vals = [target[0], target[1], target[2]];
        let result = cache.get(&vals);
        println!(
            "step={} vals={:?} target={:?} result={:?}",
            step, vals, target, result
        );
        assert_eq!(
            target[3], result[0],
            "step={} vals={:?} target={:?} result={:?}",
            step, vals, target, result
        );
    }
    test_duration.exit();
}
///
#[test]
fn cache_value_disp() {
   // DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbg = Dbg::new("cache", "cache_get");
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
    cache.init(data.clone()).unwrap();
    let result = cache.value_disp(0);
    assert_eq!(0., result.0, "min index=0 target=0. result={:?}", result.0);
    assert_eq!(5.4, result.1, "max index=0 target=0. result={:?}", result.1);
    let result = cache.value_disp(1);
    assert_eq!(0., result.0, "min index=1 target=0. result={:?}", result.0);
    assert_eq!(4.6, result.1, "max index=1 target=0. result={:?}", result.1);
    let result = cache.value_disp(2);
    assert_eq!(0., result.0, "min index=1 target=0. result={:?}", result.0);
    assert_eq!(4.7, result.1, "max index=1 target=0. result={:?}", result.1);
    let result = cache.value_disp(3);
    assert_eq!(0., result.0, "min index=1 target=0. result={:?}", result.0);
    assert_eq!(80.7, result.1, "max index=1 target=0. result={:?}", result.1);
    test_duration.exit();
}
///
#[test]
fn cache_value_disp_opt() {
   // DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbg = Dbg::new("cache", "cache_get");
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
    cache.init(data.clone()).unwrap();
    let result = cache.value_disp_opt(0, &vec![None, Some(0.3)]).unwrap();
    assert_eq!(4.3, result.0, "min index=0 target=0. result={:?}", result.0);
    assert_eq!(4.3, result.1, "max index=0 target=0. result={:?}", result.1);
    test_duration.exit();
}
/*
///
/// Test failure initializing of [Cache] instance.
#[test]
fn init_cache_table_from_inconsistent_files() {
    DebugSession::new().filter(LogLevel::Info).init();
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
