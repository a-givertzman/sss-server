use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use sal_sync::services::entity::dbg_id::DbgId;
use std::{sync::Once, time::Duration};
use testing::stuff::max_test_duration::TestDuration;

use crate::common::cache::{column::Column, table::Table};
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
/// Test values, which [Table] returns for given 'value'.
#[test]
fn get_unchecked() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbgid = DbgId("test Table".to_string());
    let callee = "get_unchecked";
    log::debug!("\n{}", dbgid);
    let test_duration = TestDuration::new(&dbgid, Duration::from_secs(1));
    test_duration.run().unwrap();
    // init
    //
    let matrix = [
        // 0    1    2     3
        [0.0, 0.0, 0.0, 10.0], // 0
        [1.0, 0.0, 0.0, 20.0], // 1
        [0.0, 1.0, 0.0, 11.0], // 2
        [0.0, 0.0, 1.0, 10.1], // 3
        [1.0, 1.0, 0.0, 21.0], // 4
        [1.0, 1.0, 1.0, 21.1], // 5
        [0.0, 1.0, 1.0, 11.1], // 6
        [1.0, 0.0, 1.0, 20.1], // 7
    ];
    let mut columns = vec![];
    for col_id in 0..matrix[0].len() {
        let mut values = vec![];
        (0..matrix.len()).for_each(|row_id| {
            let var_name = matrix[row_id][col_id];
            values.push(var_name);
        });
        let dbgid = DbgId::with_parent(&dbgid, &format!("Column_{}", col_id));
        let column = Column::new(dbgid, values);
        columns.push(column);
    }
    let table = Table::new(&dbgid, columns);
    //
    ////
    #[rustfmt::skip]
    let test_data = [
        // 0
        ([Some(0.0), Some(1.0), Some(1.0)].as_slice(),   vec![vec![0.0, 1.0, 1.0, 11.1]],),
        (&[Some(0.0), Some(1.0), Some(1.0), None],       vec![vec![0.0, 1.0, 1.0, 11.1]],),
        (&[Some(0.0), Some(1.0), Some(1.0), Some(11.1)], vec![vec![0.0, 1.0, 1.0, 11.1]],),
        (&[Some(0.0), Some(1.0), None, Some(11.1)],      vec![vec![0.0, 1.0, 0.0, 11.0], vec![0.0, 1.0, 1.0, 11.1]],),
        (&[Some(0.0), None, Some(1.0), Some(11.1)],      vec![vec![0.0, 0.0, 1.0, 10.1], vec![0.0, 1.0, 1.0, 11.1]],),
        // 5
        (&[None, Some(1.0), Some(1.0), Some(11.1)], vec![vec![0.0, 1.0, 1.0, 11.1]],),
        (&[Some(0.0), Some(1.0)],                   vec![vec![0.0, 1.0, 0.0, 11.0], vec![0.0, 1.0, 1.0, 11.1]],),
        (&[Some(0.0), None, Some(1.0)],             vec![vec![0.0, 0.0, 1.0, 10.1], vec![0.0, 1.0, 1.0, 11.1]],),
        (&[Some(0.0), None, None, Some(11.0)],      vec![
                                                        vec![0.0, 0.0, 0.0, 10.0],
                                                        vec![0.0, 1.0, 0.0, 11.0],
                                                        vec![0.0, 0.0, 1.0, 10.1],
                                                    ],
        ),
        (&[None, Some(0.0), None, Some(10.1)],      vec![vec![0.0, 0.0, 0.0, 10.0], vec![0.0, 0.0, 1.0, 10.1]],),
        // 10
        (&[None, None, Some(0.0), Some(21.0)], vec![vec![1.0, 1.0, 0.0, 21.0]],),
        (&[Some(0.0)],                         vec![
                                                vec![0.0, 0.0, 0.0, 10.0],
                                                vec![0.0, 1.0, 0.0, 11.0],
                                                vec![0.0, 0.0, 1.0, 10.1],
                                                vec![0.0, 1.0, 1.0, 11.1],
                                            ]
        ),
        (&[None, Some(0.0)],                   vec![
                                                vec![0.0, 0.0, 0.0, 10.0],
                                                vec![0.0, 0.0, 1.0, 10.1],
                                                vec![1.0, 0.0, 1.0, 20.1],
                                            ]
        ),
        (&[None, None, Some(0.0)],             vec![
                                                vec![0.0, 0.0, 0.0, 10.0],
                                                vec![1.0, 1.0, 0.0, 21.0],
                                            ]
        ),
        (&[None, None, None, Some(21.0)],      vec![
                                                vec![1.0, 1.0, 0.0, 21.0],
                                                vec![0.5, 1.0, 1.0, 16.1]
                                            ]
        ,),
        // 15
        (&[Some(0.5)], vec![
                           vec![0.5, 0.0, 0.0, 15.0], 
                           vec![0.5, 0.5, 0.0, 15.5], 
                           vec![0.5, 0.5, 0.5, 15.55], 
                           vec![0.5, 1.0, 1.0, 16.1], 
                           vec![0.5, 0.5, 1.0, (11.1 + 20.1) / 2.]
                       ],
        ),
    ];
    for (step, (value, target)) in test_data.into_iter().enumerate() {
        let result = table.get_unchecked(value);
        println!(
            "{}.{} | step={} value={:?} result={:?} target={:?}",
            dbgid, callee, step, value, result, target
        );
        assert_eq!(
            target, result,
            "{}.{} | step={} value={:?} result={:?} target={:?}",
            dbgid, callee, step, value, result, target
        );
    }
    test_duration.exit();
}
