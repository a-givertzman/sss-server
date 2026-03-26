#[cfg(test)]
mod recalculation_course_angular {
    use std::{
        sync::Once, 
        time::Duration
    };
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{
        DebugSession, 
        LogLevel, 
    };

    use crate::algorithm::entities::recalculation_course_angular::RecalculationCourseAngular;
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
    /// Testing `to_northeastern`
    #[test]
    fn to_northeastern() {
        DebugSession::new()
            .filter(LogLevel::Info)
            .module("api_tools", LogLevel::Error)
            .module("sal_sync", LogLevel::Error)
            .module("ena", LogLevel::Error)
            .init();
        init_once();
        init_each();
        log::debug!("");
        let dbg = "eval";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data = [
            (
                1,
                10.0,
                vec![
                    (10.0, 10.0)
                ],
                vec![
                    (20.0, 10.0)
                ]
            ),
            (
                1,
                355.0,
                vec![
                    (10.0, 10.0)
                ],
                vec![
                    (5.0, 10.0)
                ]
            )
        ];
        for (step, course_angle, array, target) in test_data.iter() {
            let result = RecalculationCourseAngular::to_northeastern(*course_angle, array.to_vec());
            assert!(result == *target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
        test_duration.exit();
    }
    ///
    /// Testing `to_course_angle`
    #[test]
    fn to_course_angle() {
        DebugSession::new()
            .filter(LogLevel::Info)
            .module("api_tools", LogLevel::Error)
            .module("sal_sync", LogLevel::Error)
            .module("ena", LogLevel::Error)
            .init();
        init_once();
        init_each();
        log::debug!("");
        let dbg = "eval";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data = [
            (
                1,
                10.0,
                vec![
                    (10.0, 10.0)
                ],
                vec![
                    (0.0, 10.0)
                ]
            ),
            (
                1,
                356.0,
                vec![
                    (355.0, 10.0)
                ],
                vec![
                    (359.0, 10.0)
                ]
            )
        ];
        for (step, course_angle, array, target) in test_data.iter() {
            let result = RecalculationCourseAngular::to_course_angle(*course_angle, array.to_vec());
            assert!(result == *target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
        test_duration.exit();
    }
}
