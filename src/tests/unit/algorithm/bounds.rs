#[cfg(test)]

mod tests {
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use std::time::Duration;
    use testing::stuff::max_test_duration::TestDuration;
    use crate::algorithm::entities::{Bound, Bounds};

    #[test]
    fn bounds_from_n() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        println!();
        let self_id = "test Bounds create from_n";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let result = Bounds::from_n(20., 10., 4).unwrap();
        let target = Bounds::new(vec![
            Bound::new(-10., -5.).unwrap(),
            Bound::new(-5., 0.).unwrap(),
            Bound::new(0., 5.).unwrap(),
            Bound::new(5., 10.).unwrap(),
        ]).unwrap();
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );

        test_duration.exit();
    }

    #[test]
    fn bounds_from_frames() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        println!();
        let self_id = "test Bounds create from_frames";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let result = Bounds::from_frames(&vec![(-10., -5.), (-5., 0.), (0., 5.), (5., 10.),]).unwrap();
        let target = Bounds::new(vec![
            Bound::new(-10., -5.).unwrap(),
            Bound::new(-5., 0.).unwrap(),
            Bound::new(0., 5.).unwrap(),
            Bound::new(5., 10.).unwrap(),
        ]).unwrap();
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );

        test_duration.exit();
    }

    #[test]
    fn bounds_intersect() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        println!();
        let self_id = "test Bounds intersect";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let b_src = Bounds::from_frames(&vec![(-10., -5.), (-5., 0.), (0., 5.), (5., 10.)]).unwrap();
        let v_src = [1., 2., 3., 4.];
        let b_trg = Bounds::from_frames(&vec![(-10., 0.), (0., 10.)]).unwrap();     
        let result = b_trg.intersect(&b_src, &v_src).unwrap();
        let target = [3., 7.];
        assert!(
            result.as_slice() == &target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );

        let b_src = Bounds::from_frames(&vec![(-15., -5.), (-5., 5.), (5., 15.)]).unwrap();
        let v_src = [10., 10., 10.];
        let b_trg = Bounds::from_frames(&vec![(-15., 0.), (0., 15.)]).unwrap();     
        let result = b_trg.intersect(&b_src, &v_src).unwrap();
        let target = [15., 15.];
        assert!(
            result.as_slice() == &target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );

        let b_src = Bounds::from_frames(&vec![(-10., 0.), (0., 10.)]).unwrap();
        let v_src = [10.,10.];
        let b_trg = Bounds::from_min_max(-10., 10., 10).unwrap();     
        let result = b_trg.intersect(&b_src, &v_src).unwrap();
        let target = [2.; 10];
        assert!(
            result.as_slice() == &target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );

        let b_src = Bounds::from_min_max(-10., 10., 10).unwrap(); 
        let v_src = [2.; 10];
        let b_trg = Bounds::from_frames(&vec![(-10., 0.), (0., 10.)]).unwrap();
        let result = b_trg.intersect(&b_src, &v_src).unwrap();
        let target = [10.,10.];
        assert!(
            result.as_slice() == &target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );

        test_duration.exit();
    }
}
