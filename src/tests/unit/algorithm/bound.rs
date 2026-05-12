#[cfg(test)]

mod tests {
    use std::time::Duration;
    use debugging::session::debug_session::{DebugSession, LogLevel};
    use testing::stuff::max_test_duration::TestDuration;
    use crate::algorithm::entities::Bound;
    
    #[test]
    fn bound_intersect() {
            DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
        println!();
        let self_id = "test Bound intersect";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (Bound::new(2., 4.).unwrap(), Bound::new(0., 1.,).unwrap(), Bound::None),
            (Bound::new(2., 4.).unwrap(), Bound::new(4., 5.,).unwrap(), Bound::None),
            (Bound::new(2., 4.).unwrap(), Bound::new(1., 3.,).unwrap(), Bound::new(2., 3.).unwrap()),
            (Bound::new(2., 4.).unwrap(), Bound::new(1., 5.,).unwrap(), Bound::new(2., 4.).unwrap()),
            (Bound::new(2., 4.).unwrap(), Bound::new(3., 5.,).unwrap(), Bound::new(3., 4.).unwrap()),
            (Bound::new(2., 4.).unwrap(), Bound::new(2., 3.,).unwrap(), Bound::new(2., 3.).unwrap()),
        ];
        for (left, right, target) in test_data {
            let result = left.intersect(&right).unwrap();
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }

    #[test]
    fn bound_part_ratio() {
    DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
        println!();
        let self_id = "test Bound part_ratio";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (Bound::new(2., 4.).unwrap(), Bound::new(0., 1.,).unwrap(), 0.),
            (Bound::new(2., 4.).unwrap(), Bound::new(4., 5.,).unwrap(), 0.),
            (Bound::new(2., 4.).unwrap(), Bound::new(1., 3.,).unwrap(), 0.5),
            (Bound::new(2., 4.).unwrap(), Bound::new(1., 5.,).unwrap(), 1.),
            (Bound::new(2., 4.).unwrap(), Bound::new(3., 5.,).unwrap(), 0.5),
            (Bound::new(2., 4.).unwrap(), Bound::new(2., 3.,).unwrap(), 0.5),
            (Bound::new(-3., 3.).unwrap(), Bound::new(-5., 0.,).unwrap(), 0.5),
        ];

        for (left, right, target) in test_data {
            let result = left.part_ratio(&right).unwrap();
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }

    #[test]
    fn bound_center() {
    DebugSession::new()
        .filter(LogLevel::Info)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
        println!();
        let self_id = "test Bound center";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let result = Bound::new(-2., 4.).unwrap().center().unwrap();   
        let target = 1.;
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        test_duration.exit();
    }
}