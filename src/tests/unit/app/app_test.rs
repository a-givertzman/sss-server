#[cfg(test)]

mod app {
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use std::{
        sync::Once,
        time::{Duration, Instant},
    };
    use testing::stuff::max_test_duration::TestDuration;

    use crate::{app::app::App, kernel::run::Run};
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
    fn init_each() {}
    ///
    /// Testing such functionality / behavior
    #[tokio::test(flavor = "multi_thread")]
    async fn new() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg_id = "new";
        log::debug!("\n{}", dbg_id);
        let test_duration = TestDuration::new(dbg_id, Duration::from_secs(1));
        test_duration.run().unwrap();
        let path = "src/tests/unit/app/app_config.yaml";
        let time = Instant::now();
        let mut app = App::new(path);
        let step = 1;
        let target = Ok(());
        let result = app.run();
        let elapsed = time.elapsed();
        log::debug!("{}.new | elapsed: {:?}", dbg_id, elapsed);
        assert!(
            result == target,
            "step {} \nresult: {:?}\ntarget: {:?}",
            step,
            result,
            target
        );
        test_duration.exit();
    }
}
