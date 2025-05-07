use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use std::{sync::Once, time::Duration};
use testing::stuff::max_test_duration::TestDuration;

use crate::common::cache::bound::Bound;
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
/// Bound intersection.
#[test]
fn bitand_test() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbgid = "bitand_test";
    log::debug!("\n{}", dbgid);
    let test_duration = TestDuration::new(dbgid, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        // 0
        (Bound::None, Bound::None, Bound::None),
        (Bound::None, Bound::Single(0), Bound::None),
        (Bound::None, Bound::Range(0, 1), Bound::None),
        (Bound::Single(0), Bound::None, Bound::None),
        (Bound::Single(0), Bound::Single(0), Bound::Single(0)),
        // 5
        (Bound::Single(0), Bound::Single(1), Bound::None),
        (Bound::Single(0), Bound::Range(0, 1), Bound::Single(0)),
        (Bound::Single(1), Bound::Range(0, 1), Bound::Single(1)),
        (Bound::Single(2), Bound::Range(0, 1), Bound::None),
        (Bound::Single(1), Bound::Range(0, 2), Bound::Single(1)),
        // 10
        (Bound::Range(0, 1), Bound::None, Bound::None),
        (Bound::Range(0, 1), Bound::Single(0), Bound::Single(0)),
        (Bound::Range(0, 1), Bound::Single(1), Bound::Single(1)),
        (Bound::Range(0, 1), Bound::Single(2), Bound::None),
        (Bound::Range(0, 2), Bound::Single(1), Bound::Single(1)),
        // 15
        (Bound::Range(0, 1), Bound::Range(2, 3), Bound::None),
        (Bound::Range(2, 3), Bound::Range(0, 1), Bound::None),
        (Bound::Range(0, 1), Bound::Range(1, 2), Bound::Single(1)),
        (Bound::Range(1, 2), Bound::Range(0, 1), Bound::Single(1)),
        (Bound::Range(0, 2), Bound::Range(0, 1), Bound::Range(0, 1)),
        // 20
        (Bound::Range(0, 1), Bound::Range(0, 2), Bound::Range(0, 1)),
        (Bound::Range(0, 2), Bound::Range(1, 3), Bound::Range(1, 2)),
        (Bound::Range(1, 3), Bound::Range(0, 2), Bound::Range(1, 2)),
        (Bound::Range(0, 3), Bound::Range(1, 2), Bound::Range(1, 2)),
        (Bound::Range(1, 2), Bound::Range(0, 3), Bound::Range(1, 2)),
    ];
    for (step, (lhs, rhs, target)) in test_data.into_iter().enumerate() {
        let result = lhs & rhs;
        println!("step={} result: {:?} target: {:?}\n", step, result, target);
        assert_eq!(
            result, target,
            "step={} lhs={:?} rhs={:?} target={:?} result={:?}",
            step, lhs, rhs, target, result
        );
    }
    test_duration.exit();
}
