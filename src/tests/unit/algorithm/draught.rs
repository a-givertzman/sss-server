#[cfg(test)]

mod tests {
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use sal_core::dbg::Dbg;
    use std::time::Duration;
    use testing::stuff::max_test_duration::TestDuration;

    use crate::algorithm::entities::model_cached::Draught;
    
    #[test]
    fn draught() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        let dbg = Dbg::own("test draught");
        let test_duration = TestDuration::new(&dbg, Duration::from_secs(10));
        test_duration.run().unwrap();
        let data = [
            [65.250, 130.500, 5.900, 70.000, 2.000, 25., 2., 8.4141, 3.3859, 7.0156],
            [65.250, 130.500, 3.000, 60.000, -3.000, 10., 4., 7.6331, 0./*-1.6331*/, 2.0982],
            [65.250, 130.500, 8.000, 50.000, 2.000, 15., -1., 6.8209, 9.1791, 8.8115],
        ];
        let epsilon = 0.0001;
        for data in data {
            let (target_draught_bow, target_draught_stern, target_draught_mean) = (data[7], data[8], data[9]);
            let (result_draught_bow, result_draught_stern, result_draught_mean) = Draught::new(
                data[0],         
                data[1],  
                data[2],  
                data[3],  
                data[4],  
                data[5],  
                data[6],     
            ).calculate();  
            assert!(
                (target_draught_bow - result_draught_bow).abs() < epsilon,
                "\nresult: {:?} target: {:?}",
                result_draught_bow,
                target_draught_bow,
            );
            assert!(
                (target_draught_stern - result_draught_stern).abs() < epsilon,
                "\nresult: {:?} target: {:?}",
                result_draught_stern,               
                target_draught_stern,
            );
            assert!(
                (target_draught_mean - result_draught_mean).abs() < epsilon,
                "\nresult: {:?} target: {:?}",
                result_draught_mean,                
                target_draught_mean,
            );
        }
    }
}
