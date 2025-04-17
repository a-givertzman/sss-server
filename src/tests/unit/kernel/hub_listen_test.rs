#[cfg(test)]

mod hub_listen {
    use std::{sync::Once, time::Duration};
    use bincode::{config, Decode, Encode};
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
    fn listen() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "link_listen";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(5));
        test_duration.run().unwrap();
        let test_data: [(i32, Message, Result<Message, Error>); 4] = [
            (1, Message("Query-1".into()), Ok(Message("Reply-1".into()))),
            (2, Message("Query-2".into()), Ok(Message("Reply-2".into()))),
            (3, Message("Query-3".into()), Ok(Message("Reply-3".into()))),
            (4, Message("Query-4".into()), Ok(Message("Reply-4".into()))),
        ];
        let hub = Hub::new(dbg);
        let local = hub.link();
        let remote_handle = hub.listen(|query: String| {
            log::debug!("Link.remote.listen | Query {:#?}", query);
            Some(match query.as_str() {
                "Query-1" => bincode::encode_to_vec(&Message("Reply-1".into()), config::standard()).unwrap(),
                "Query-2" => bincode::encode_to_vec(&Message("Reply-2".into()), config::standard()).unwrap(),
                "Query-3" => bincode::encode_to_vec(&Message("Reply-3".into()), config::standard()).unwrap(),
                "Query-4" => bincode::encode_to_vec(&Message("Reply-4".into()), config::standard()).unwrap(),
                _ => panic!("Link.remote.listen | Unknown event {:#?}", query),
            })
        }).unwrap();
        for (step, query, target) in test_data {
            let result: Result<Message, Error> = local.call(query);
            log::debug!("step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
            match (&result, &target) {
                (Ok(result), Ok(target)) => assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target),
                (Err(_), Err(_)) => {},
                _ => panic!("Error in step {} \nresult: {:?}\ntarget: {:?}", step, result, target)
            }
        }
        remote_handle.join().unwrap();
        log::debug!("{} | All - Done", dbg);
        test_duration.exit();
    }
    ///
    /// Message container
    #[derive(Debug, Encode, Decode, PartialEq)]
    struct Message(pub String);
}
