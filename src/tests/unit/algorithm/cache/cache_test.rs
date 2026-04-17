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
    DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
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
        vec![0.0, 0.0, 0.0, 0.0],
        vec![0.0, 0.0, 0.3, 1.0],
        vec![0.0, 0.2, 0.0, 10.0],
        vec![0.0, 0.2, 0.3, 11.0],
        vec![0.1, 0.0, 0.0, 100.0],
        vec![0.1, 0.0, 0.3, 101.0],
        vec![0.1, 0.2, 0.0, 110.0],
        vec![0.1, 0.2, 0.3, 111.0],
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
    DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
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
        vec![0.0, 0.0, 0.0, 0.0],
        vec![0.0, 0.0, 0.3, 1.0],
        vec![0.0, 0.2, 0.0, 10.0],
        vec![0.0, 0.2, 0.3, 11.0],
        vec![0.1, 0.0, 0.0, 100.0],
        vec![0.1, 0.0, 0.3, 101.0],
        vec![0.1, 0.2, 0.0, 110.0],
        vec![0.1, 0.2, 0.3, 111.0],
    ];
    let cache = Cache::new(&dbg);
    cache.init(data.clone()).unwrap();
    let result = cache.disp(0);
    assert_eq!(0., result.0, "min index=0 target=0. result={:?}", result.0);
    assert_eq!(0.1, result.1, "max index=0 target=0. result={:?}", result.1);
    let result = cache.disp(1);
    assert_eq!(0., result.0, "min index=1 target=0. result={:?}", result.0);
    assert_eq!(0.2, result.1, "max index=1 target=0. result={:?}", result.1);
    let result = cache.disp(2);
    assert_eq!(0., result.0, "min index=1 target=0. result={:?}", result.0);
    assert_eq!(0.3, result.1, "max index=1 target=0. result={:?}", result.1);
    let result = cache.disp(3);
    assert_eq!(0., result.0, "min index=1 target=0. result={:?}", result.0);
    assert_eq!(111.0, result.1, "max index=1 target=0. result={:?}", result.1);
    test_duration.exit();
}
///
#[test]
fn cache_value_disp_opt() {
    DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
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
        vec![0.0, 0.0, 0.0, 0.0],
        vec![0.0, 0.0, 0.3, 1.0],
        vec![0.0, 0.2, 0.0, 10.0],
        vec![0.0, 0.2, 0.3, 11.0],
        vec![0.1, 0.0, 0.0, 100.0],
        vec![0.1, 0.0, 0.3, 101.0],
        vec![0.1, 0.2, 0.0, 110.0],
        vec![0.1, 0.2, 0.3, 111.0],
    ];
    let cache = Cache::new(&dbg);
    cache.init(data.clone()).unwrap();
    let result = cache.values_disp(&vec![Some(0.1), Some(0.2), Some(0.3)]);
    assert_eq!(111.0, result[0][0], "result={:?}", result[0][0]);
    let result = cache.values_disp(&vec![None, Some(0.2), Some(0.3)]);
    assert_eq!(11.0, result[0][0], "result={:?}", result[0][0]);
    assert_eq!(111.0, result[1][0], "result={:?}", result[1][0]);    
    let result = cache.values_disp(&vec![Some(0.1), Some(0.2), None]);
    assert_eq!(110.0, result[0][0], "result={:?}", result[0][0]);
    assert_eq!(111.0, result[1][0], "result={:?}", result[1][0]);  
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
