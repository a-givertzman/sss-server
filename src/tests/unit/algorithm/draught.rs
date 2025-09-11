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

        let (draught_bow, draught_stern, draught_mean) = Draught::new(
            65.250,         
            130.5,
            6.,
            65.250,
            10.,
            10.,
            2.,        
        ).calculate();     
           
        let target =  IcingCtx {
            mass: 67.460925,
            mass_shift_x:1.2989097,
            mass_values: Vec::new(),
        };
        assert!(
            result.mass == target.mass,
            "\nresult: {:?}\ntarget: {:?}",
            result.mass,
            target.mass
        );
        assert!(
            result.mass_shift_x == target.mass_shift_x,
            "\nresult: {:?}\ntarget: {:?}",
            result.mass_shift_x,
            target.mass_shift_x
        );
        test_duration.exit();
    }
}
