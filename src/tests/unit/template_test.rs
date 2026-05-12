#[cfg(test)]

use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel};
///
///
static INIT: Once = Once::new();
///
/// once called initialisation
fn init_once() {
    INIT.call_once(|| {
        // implement your initialisation code to be called only once for current test file
    })
}
///
/// returns:
///  - ...
fn init_each() -> () {}
///
/// Testing such functionality / behavior
#[test]
fn test_task_cycle() {
    DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("query_calculus");
    log::debug!("{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
    test_duration.run().unwrap();
    assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    test_duration.exit();
}
