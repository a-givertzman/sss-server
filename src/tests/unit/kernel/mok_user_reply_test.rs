#[cfg(test)]

mod mok_user_reply {
    use std::{sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{algorithm::entities::hook::Hook, infrostructure::client::{choose_user_hook::{ChooseUserHookQuery, ChooseUserHookReply}, query::Query}, kernel::{sync::link::Link, mok_user_reply::mok_user_reply::MokUserReply}};
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
    /// Testing 'run' method
    #[tokio::test(flavor = "multi_thread")]
    async fn run() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "mok_user_reply";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data = [
            (
                1,
                ChooseUserHookQuery::test(vec![
                    Hook { 
                        gost: "".into(),
                        r#type: "".into(),
                        load_capacity_m13: 0.1,
                        load_capacity_m46: 0.2,
                        load_capacity_m78: 0.3,
                        shank_diameter: 0.4,
                        weight: 50.0,
                    }
                ]),
                Hook {
                    gost: "GOST 34567-85".into(),
                    r#type: "Forged".into(),
                    load_capacity_m13: 25.0,
                    load_capacity_m46: 23.0,
                    load_capacity_m78: 21.0,
                    shank_diameter: 85.0,
                    weight: 50.0,
                }
            )
        ];
        let (local, remote) = Link::split(dbg);
        let mut user_reply = MokUserReply::new(dbg, remote);
        let user_reply_handle = user_reply.run().await.unwrap();
        for (step, query, target) in test_data {
            let query = Query::ChooseUserHook(query);
            let result: ChooseUserHookReply = local.req(query).await.unwrap();
            let result = result.choosen;
            log::debug!("step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
            assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
        user_reply.exit();
        let _ = user_reply_handle.await.unwrap();
        test_duration.exit();
    }
}
