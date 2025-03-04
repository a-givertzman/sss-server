#[cfg(test)]

mod link_listen {
    use std::{sync::Once, time::Duration};
    use sal_sync::services::entity::point::point::Point;
    use serde::{Deserialize, Serialize};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::kernel::{str_err::str_err::StrErr, sync::link::Link};
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
    async fn listen() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "link_listen";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(5));
        test_duration.run().unwrap();
        let test_data: [(i32, Message, Result<Message, StrErr>); 4] = [
            (1, Message("Query-1".into()), Ok(Message("Reply-1".into()))),
            (2, Message("Query-2".into()), Ok(Message("Reply-2".into()))),
            (3, Message("Query-3".into()), Ok(Message("Reply-3".into()))),
            (4, Message("Query-4".into()), Ok(Message("Reply-4".into()))),
        ];
        let (local, mut remote) = Link::split(dbg);
        let remote_handle = remote.listen(|query| {
            log::debug!("Link.remote.listen | Query {:#?}", query);
            let query = query.as_string().value;
            let query: String = serde_json::from_str(&query).unwrap();
            Some(match query.as_str() {
                "Query-1" => Point::new(0, "name", serde_json::to_string(&Message("Reply-1".into())).unwrap()),
                "Query-2" => Point::new(0, "name", serde_json::to_string(&Message("Reply-2".into())).unwrap()),
                "Query-3" => Point::new(0, "name", serde_json::to_string(&Message("Reply-3".into())).unwrap()),
                "Query-4" => Point::new(0, "name", serde_json::to_string(&Message("Reply-4".into())).unwrap()),
                _ => panic!("Link.remote.listen | Unknown event {:#?}", query),
            })
        }).await.unwrap();
        for (step, query, target) in test_data {
            let result: Result<Message, StrErr> = local.req(query).await;
            log::debug!("step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
            assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
        remote.exit();
        // remote_handle.await.unwrap().await;
        log::debug!("{} | All - Done", dbg);
        test_duration.exit();
    }
    ///
    /// Message container
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Message(pub String);
}
