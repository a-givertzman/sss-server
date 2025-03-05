#[cfg(test)]

mod request {
    use std::{sync::Once, time::Duration};
    use testing::{entities::test_value::Value, stuff::max_test_duration::TestDuration};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        algorithm::{context::{context::Context, testing_ctx::{MokUserReplyTestCtx, TestingCtx}}, initial::initial_ctx::InitialCtx},
        kernel::{request::Request, sync::link::Link},
    };
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
    /// Testing 'Request::fetch'
    #[tokio::test(flavor = "multi_thread")]
    async fn basic() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "request";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data: [(usize, InitialCtx, MokUserReplyTestCtx); 2] = [
            (
                1,
                InitialCtx::default(),
                MokUserReplyTestCtx { value: Value::String("Hello World!".to_string()) },
            ),
            (
                2,
                InitialCtx::default(),
                MokUserReplyTestCtx {value: Value::Real(123.456) },
            )
        ];
        let (link, _) = Link::split(dbg);
        let request = Request::new(
            link,
            async |ctx: MokUserReplyTestCtx, link: Link| {
                let reply: MokUserReplyTestCtx = ctx;
                (reply, link)
            },
        );
        for (step, initial, target) in test_data {
            let value = target.clone();
            let mut ctx = Context::new(initial.clone());
            ctx.testing = Some(TestingCtx { mok_user_reply: value });
            let ctx = ctx.testing.unwrap().mok_user_reply;
            let result = request.fetch(ctx).await;
            assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
        test_duration.exit();
    }
}
