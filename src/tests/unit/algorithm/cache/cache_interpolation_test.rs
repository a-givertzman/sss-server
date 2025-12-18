use crate::algorithm::entities::{cache::*, vec};
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
/// Test successfull initializing of [Cache] instance.
#[test]
fn cache_interpolation() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let dbg = Dbg::new("cache cache", "cache_interpolation");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    // init
    //
    #[rustfmt::skip]
    let test_data = vec![
        vec![0.0, 0.0, 10.0],
        vec![0.0, 1.0, 20.0],
        vec![1.0, 0.0, 15.0],
        vec![1.0, 1.0, 25.0],
        vec![-1.0, 0.0, 5.0],
        vec![-1.0, 1.0, 15.0],
    ];
    let cache = Cache::new(&dbg);
    let _ = cache.init(test_data);
    let target = vec![10.0];
    let result = cache.get(&[0.0, 0.0]);
    assert_eq!(target, result, "target={:?} result={:?}", target, result);
    let target = vec![15.];
    let result = cache.get(&[0.0, 0.5]);
    assert_eq!(target, result, "target={:?} result={:?}", target, result);
    let target = vec![20.];
    let result = cache.get(&[1.0, 0.5]);
    assert_eq!(target, result, "target={:?} result={:?}", target, result);
    let target = vec![12.5];
    let result = cache.get(&[-0.5, 0.5]);
    assert_eq!(target, result, "target={:?} result={:?}", target, result);
    test_duration.exit();
}
