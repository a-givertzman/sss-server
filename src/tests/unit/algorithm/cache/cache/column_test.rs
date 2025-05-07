use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use sal_sync::services::entity::dbg_id::DbgId;
use std::{sync::Once, time::Duration};
use testing::stuff::max_test_duration::TestDuration;

use crate::common::cache::{bound::Bound, column::Column};
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
/// Analyze values and extract inflextion points.
#[test]
fn get_inflextion_test() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbgid = DbgId("get_inflextion_test".to_string());
    log::debug!("\n{}", dbgid);
    let test_duration = TestDuration::new(&dbgid, Duration::from_secs(1));
    test_duration.run().unwrap();
    #[rustfmt::skip]
    let test_data = [
        // 0
        (vec![], vec![].into()),
        (vec![1.], vec![0].into()),
        (vec![2., 2.], vec![0, 1].into()),
        (vec![0., 1., 2.], vec![0, 2].into()),
        (vec![0., 1., 2., 2.], vec![0, 3].into()),
        (vec![0., 1., 1., 2.], vec![0, 3].into()),
        (vec![2., 1., 0.], vec![0, 2].into()),
        (vec![2., 2., 1., 0.], vec![0, 3].into()),
        (vec![2., 1., 0., 0.], vec![0, 3].into()),
        (vec![4., 4., 1., 4., 4.], vec![0, 2, 4].into()),
        // 10
        (vec![0., 0., 0., 0., 1.], vec![0, 4].into()),
        (vec![1., 0., 0., 0., 0.], vec![0, 4].into()),
        (vec![4., 0., 0., 0., 4.], vec![0, 3, 4].into()),
        (vec![4., 0., 0., 0., 4., 0.], vec![0, 3, 4, 5].into()),
        (vec![4., 4., -2., 4., 4.], vec![0, 2, 4].into()),
        (vec![0., 0., 5., 0., 0., 5.], vec![0, 2, 4, 5].into()),
        (vec![0., 0., 7., 0., 0., 7., 7., 7.], vec![0, 2, 4, 7].into()),
        (vec![6., 5., 4., 3., 4., 5., 6.], vec![0, 3, 6].into()),
        (vec![0., 1., 2., 3., 2., 1., 0., 1., 2., 3., 2., 1., 0.], vec![0, 3, 6, 9, 12].into()),
        (vec![0., 0., 2., 2., 3., 3., 2., 2., 1., 1., 0., 0., 1., 0., 0.], vec![0, 5, 11, 12, 14].into()),
        // 20
        // NOTE: zig-zag shape like '/\/\/' gives values one-by-one
        // need to handle that somehow?
        (vec![-1., 1., -0.5, 0.5, 0.], vec![0, 1, 2, 3, 4].into()),
    ];
    let dbgid = DbgId::with_parent(&dbgid, "Column_0");
    for (step, (values, target)) in test_data.into_iter().enumerate() {
        let result = Column::get_inflections(&dbgid, &values);
        println!(
            "step={} values={:?} result={:?} target={:?}",
            step, values, result, target
        );
        assert_eq!(
            result, target,
            "step={} values={:?} result={:?} target={:?}",
            step, values, result, target
        );
    }
    test_duration.exit();
}
///
/// Analyze [Bound]s of monotonic (~ with local extremum(s)) values.
#[test]
fn get_bounds_monotonic() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbgid = DbgId("get_bounds_monotonic".to_string());
    log::debug!("\n{}", dbgid);
    let test_duration = TestDuration::new(&dbgid, Duration::from_secs(1));
    test_duration.run().unwrap();
    // init
    //                0   1   2   3   4   5   6   7    8    9   10   11
    let values = vec![0., 1., 2., 3., 2., 1., 0., 0., -1., -1., 10., 9.];
    let dbgid = DbgId::with_parent(&dbgid, "Column_0");
    let column = Column::new(dbgid, values);
    //
    ////
    #[rustfmt::skip]
    let test_data = [
        // 0
        (3.5,  vec![Bound::Range(9, 10)]),
        (0.0,  vec![Bound::Single(0), Bound::Single(6), Bound::Single(7), Bound::Range(9, 10)]),
        (0.5,  vec![Bound::Range(0, 1), Bound::Range(5, 6), Bound::Range(9, 10)]),
        (1.0,  vec![Bound::Single(1), Bound::Single(5), Bound::Range(9, 10)]),
        (1.5,  vec![Bound::Range(1, 2), Bound::Range(4, 5), Bound::Range(9, 10)]),
        (2.5,  vec![Bound::Range(2, 3), Bound::Range(3, 4), Bound::Range(9, 10)]),
        (3.0,  vec![Bound::Single(3), Bound::Range(9, 10)]),
        // NOTE: take while left/right is same?
        // e.g. (7, 8) -> (7, 9)
        //     (9, 10) -> (8, 10)
        (-0.1, vec![Bound::Range(7, 8), Bound::Range(9, 10)]),
        (-1.0, vec![Bound::Single(8), Bound::Single(9)]),
        (-1.1, vec![]),
        // 10
        (10.0, vec![Bound::Single(10)]),
        (9.5,  vec![Bound::Range(9, 10), Bound::Range(10, 11)]),
        (8.5,  vec![Bound::Range(9, 10)]),
    ];
    for (step, (value, target)) in test_data.into_iter().enumerate() {
        let result = column.get_bounds(&value);
        println!(
            "step={} value={} result={:?} target={:?}",
            step, value, result, target
        );
        assert_eq!(
            result, target,
            "step={} value={} result={:?} target={:?}",
            step, value, result, target
        );
    }
    test_duration.exit();
}
///
/// Analyze [Bound]s of non-descreasing (~ no local extremum, values tends to grow up) values.
#[test]
fn get_bounds_non_descresing() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbgid = DbgId("get_bounds_non_descresing".to_string());
    log::debug!("\n{}", dbgid);
    let test_duration = TestDuration::new(&dbgid, Duration::from_secs(1));
    test_duration.run().unwrap();
    // init
    //                0   1   2   3   4   5   6   7
    let values = vec![0., 1., 1., 1., 2., 2., 3., 4.];
    let dbgid = DbgId::with_parent(&dbgid, "Column_0");
    let column = Column::new(dbgid, values);
    //
    ////
    #[rustfmt::skip]
    let test_data = [
        // 0
        (-1.0, vec![]),
        (0.0, vec![Bound::Single(0)]),
        (0.5, vec![Bound::Range(0, 1)]),
        (1.0, vec![Bound::Single(1), Bound::Single(2), Bound::Single(3)]),
        (1.5, vec![Bound::Range(3, 4)]),
        (2.0, vec![Bound::Single(4), Bound::Single(5)]),
        (5.0, vec![]),
    ];
    for (step, (value, target)) in test_data.into_iter().enumerate() {
        let result = column.get_bounds(&value);
        println!(
            "step={} value={} result={:?} target={:?}",
            step, value, result, target
        );
        assert_eq!(
            result, target,
            "step={} value={} result={:?} target={:?}",
            step, value, result, target
        );
    }
    test_duration.exit();
}
