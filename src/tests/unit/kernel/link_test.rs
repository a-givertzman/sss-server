#[cfg(test)]

mod link {
    use std::{fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, Arc, Once}, thread::{self, JoinHandle}, time::Duration};
    use bincode::{Decode, Encode};
    use sal_core::error::Error;
    use sal_sync::services::entity::name::Name;
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::kernel::sync::link::Link;
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
    /// Testing 'Link::call'
    #[test]
    fn call() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "Link.call";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(5));
        test_duration.run().unwrap();
        let test_data: [(i32, Query, Result<Reply, Error>); 4] = [
            (1, Query("Query-1".into()), Ok(Reply("Reply-1".into()))),
            (2, Query("Query-2".into()), Ok(Reply("Reply-2".into()))),
            (3, Query("Query-3".into()), Ok(Reply("Reply-3".into()))),
            (4, Query("Query-4".into()), Ok(Reply("Reply-4".into()))),
        ];
        let (local, remote) = Link::split(dbg);
        let mut listener = Listener::new(dbg, remote);
        let listener_handle = listener.run().unwrap();
        for (step, query, target) in test_data {
            let result: Result<Reply, Error> = local.call(query);
            log::debug!("{} | step {} \nresult: {:?}\ntarget: {:?}", dbg, step, result, target);
            match (&result, &target) {
                (Ok(result), Ok(target)) => {
                    assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                }
                (Err(_), Err(_)) => {},
                _ => panic!("Error in step {} \nresult: {:?}\ntarget: {:?}", step, result, target)
            }
        }
        local.exit();
        listener.exit();
        log::debug!("{dbg} | all done",);
        listener_handle.join().unwrap();
        test_duration.exit();
    }
    ///
    /// Testing 'Link::recv'
    #[test]
    fn recv() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "Link.recv";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(5));
        test_duration.run().unwrap();
        let test_data: [(i32, Query, Result<Reply, Error>); 4] = [
            (1, Query("Query-1".into()), Ok(Reply("Reply-1".into()))),
            (2, Query("Query-2".into()), Ok(Reply("Reply-2".into()))),
            (3, Query("Query-3".into()), Ok(Reply("Reply-3".into()))),
            (4, Query("Query-4".into()), Ok(Reply("Reply-4".into()))),
        ];
        let (local, remote) = Link::split(dbg);
        let mut listener = Listener::new(dbg, remote);
        let listener_handle = listener.run().unwrap();
        for (step, query, target) in test_data {
            if let Err(err) = local.send(query) {
                panic!("{} | step {} Error: {:?}", dbg, step, err)
            }
            let result: Result<Reply, Error> = local.recv();
            log::debug!("{} | step {} \nresult: {:?}\ntarget: {:?}", dbg, step, result, target);
            match (&result, &target) {
                (Ok(result), Ok(target)) => {
                    assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                }
                (Err(_), Err(_)) => {},
                _ => panic!("Error in step {} \nresult: {:?}\ntarget: {:?}", step, result, target)
            }
        }
        local.exit();
        listener.exit();
        log::debug!("{dbg} | all done",);
        listener_handle.join().unwrap();
        test_duration.exit();
    }
    ///
    /// Testing 'Link::try_recv'
    #[test]
    fn try_recv() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "Link.try_recv";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(5));
        test_duration.run().unwrap();
        let test_data: [(i32, Query, Result<Reply, Error>); 4] = [
            (1, Query("Query-1".into()), Ok(Reply("Reply-1".into()))),
            (2, Query("Query-2".into()), Ok(Reply("Reply-2".into()))),
            (3, Query("Query-3".into()), Ok(Reply("Reply-3".into()))),
            (4, Query("Query-4".into()), Ok(Reply("Reply-4".into()))),
        ];
        let (local, remote) = Link::split(dbg);
        let mut listener = Listener::new(dbg, remote);
        let listener_handle = listener.run().unwrap();
        for (step, query, target) in test_data {
            if let Err(err) = local.send(query) {
                panic!("{} | step {} Error: {:?}", dbg, step, err)
            }
            let mut try_again = 10;
            'tries: loop {
                let result: Result<Option<Reply>, Error> = local.try_recv();
                log::debug!("{} | step {} \nresult: {:?}\ntarget: {:?}", dbg, step, result, target);
                match (&result, &target) {
                    (Ok(result), Ok(target)) => {
                        match result {
                            Some(result) => {
                                assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                                break 'tries;
                            }
                            None => log::trace!("{} | step {} Reseive none", dbg, step),
                        }
                    }
                    (Err(_), Err(_)) => {
                        break 'tries;
                    },
                    _ => panic!("Error in step {} \nresult: {:?}\ntarget: {:?}", step, result, target)
                }
                try_again -= 1;
                if try_again < 0 {
                    panic!("{} | step {} Receive error", dbg, step)
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
        local.exit();
        listener.exit();
        log::debug!("{dbg} | all done",);
        listener_handle.join().unwrap();
        test_duration.exit();
    }
    ///
    /// Testing 'Link::recv_timeout'
    #[test]
    fn recv_timeout() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "Link.recv_timeout";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(5));
        test_duration.run().unwrap();
        let test_data: [(i32, Query, Result<Reply, Error>); 4] = [
            (1, Query("Query-1".into()), Ok(Reply("Reply-1".into()))),
            (2, Query("Query-2".into()), Ok(Reply("Reply-2".into()))),
            (3, Query("Query-3".into()), Ok(Reply("Reply-3".into()))),
            (4, Query("Query-4".into()), Ok(Reply("Reply-4".into()))),
        ];
        let (local, remote) = Link::split(dbg);
        let mut listener = Listener::new(dbg, remote);
        let listener_handle = listener.run().unwrap();
        for (step, query, target) in test_data {
            if let Err(err) = local.send(query) {
                panic!("{} | step {} Error: {:?}", dbg, step, err)
            }
            let result: Result<Option<Reply>, Error> = local.recv_timeout(Duration::from_millis(100));
            log::debug!("{} | step {} \nresult: {:?}\ntarget: {:?}", dbg, step, result, target);
            match (&result, &target) {
                (Ok(result), Ok(target)) => {
                    match result {
                        Some(result) => {
                            assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                        }
                        None => log::warn!("{} | step {} Reseive timeout", dbg, step),
                    }
                }
                (Err(_), Err(_)) => {},
                _ => panic!("Error in step {} \nresult: {:?}\ntarget: {:?}", step, result, target)
            }
        }
        local.exit();
        listener.exit();
        log::debug!("{dbg} | all done",);
        listener_handle.join().unwrap();
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
    ///
    /// Receives Query, send associated Reply
    struct Listener {
        name: Name,
        /// recieve and sender channel's
        link: Option<Link>,
        /// value to stop thread that await request's
        exit: Arc<AtomicBool>,
    }
    impl Listener {
        ///
        /// Struct constructor
        pub fn new(parent: impl Into<String>, link: Link) -> Self {
            let name = Name::new(parent, "Listener");
            Self { 
                name: name,
                link: Some(link),
                exit: Arc::new(AtomicBool::new(false)),
            }
        }
        ///
        /// Starts service's main loop in the individual task
        pub fn run(&mut self) -> Result<JoinHandle<()>, Error> {
            let mut link = self.link.take().unwrap_or_else(|| panic!("{}.run | Link not found", self.name));
            let dbg = self.name.join();
            log::info!("{}.run | Starting...", dbg);
            let exit = self.exit.clone();
            let handle = std::thread::Builder::new().name(dbg.clone()).spawn(move|| {
                log::info!("{}.run | Start", dbg);
                fn send_reply(dbg: &str, link: &mut Link, reply: impl Encode + Debug) {
                    if let Err(err) = link.send(reply) {
                        log::debug!("{}.run | Send reply error: {:?}", dbg, err);
                    };
                }
                'main: loop {
                    match link.recv::<Query>() {
                        Ok(query) => {
                            log::debug!("{}.run | Received: {:?}", dbg, query);
                            match query.0.as_str() {
                                "Query-1" => send_reply(&dbg, &mut link, Reply("Reply-1".into())),
                                "Query-2" => send_reply(&dbg, &mut link, Reply("Reply-2".into())),
                                "Query-3" => send_reply(&dbg, &mut link, Reply("Reply-3".into())),
                                "Query-4" => send_reply(&dbg, &mut link, Reply("Reply-4".into())),
                                _ => panic!("Unknown Query: {:?}", query)
                            }
                        }
                        Err(err) => {
                            log::trace!("{}.run | Error: {:?}", dbg, err);
                            break;
                        }
                    }
                    if exit.load(Ordering::SeqCst) {
                        break 'main;
                    }
                }
                log::debug!("{}.run | Exit", dbg);
            });
            handle.map_err(|err| Error::new(&self.name, "run").pass(err.to_string()))
        }
        ///
        /// Sends "exit" signal to the service's thread
        pub fn exit(&self) {
            self.exit.store(true, Ordering::SeqCst);
            if let Some(link) = &self.link {
                link.exit();
            }
        }
    }
}
