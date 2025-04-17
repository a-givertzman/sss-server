#[cfg(test)]

mod hub_listen {
    use std::{sync::Once, time::Duration};
    use bincode::{Decode, Encode};
    use sal_core::error::Error;
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::kernel::sync::Hub;
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
    #[test]
    fn listen() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "hub_listen";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data: [(i32, Query, Result<Reply, Error>); 4] = [
            (1, Query("Query-1".into()), Ok(Reply("Reply-1".into()))),
            (2, Query("Query-2".into()), Ok(Reply("Reply-2".into()))),
            (3, Query("Query-3".into()), Ok(Reply("Reply-3".into()))),
            (4, Query("Query-4".into()), Ok(Reply("Reply-4".into()))),
        ];
        let hub = Hub::new(dbg);
        let link = hub.link();
        let hub_handle = hub.listen(|query: Query| {
            log::debug!("Hub.listen | Query {:#?}", query);
            Some(match query.0.as_str() {
                "Query-1" => Reply("Reply-1".into()),
                "Query-2" => Reply("Reply-2".into()),
                "Query-3" => Reply("Reply-3".into()),
                "Query-4" => Reply("Reply-4".into()),
                _ => panic!("Hub.listen | Unknown event {:#?}", query),
            })
        }).unwrap();
        for (step, query, target) in test_data {
            let result: Result<Reply, Error> = link.call(query);
            log::debug!("step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
            match (&result, &target) {
                (Ok(result), Ok(target)) => assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target),
                (Err(_), Err(_)) => {},
                _ => panic!("Error in step {} \nresult: {:?}\ntarget: {:?}", step, result, target)
            }
        }
        hub.exit();
        hub_handle.join().unwrap();
        log::debug!("{} | All - Done", dbg);
        test_duration.exit();
    }
    ///
    /// Query container
    #[derive(Debug, Encode, Decode, PartialEq)]
    struct Query(pub String);
    ///
    /// Reply container
    #[derive(Debug, Encode, Decode, PartialEq)]
    struct Reply(pub String);
}
