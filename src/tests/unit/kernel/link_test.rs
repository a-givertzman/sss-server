#[cfg(test)]

mod link {
    use std::{fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, Arc, Once}, thread::JoinHandle, time::Duration};
    use bincode::{Decode, Encode};
    use sal_core::error::Error;
    use sal_sync::services::entity::name::Name;
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{algorithm::context::ctx_result::CtxResult, kernel::sync::link::Link};
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
    fn req() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "link";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(5));
        test_duration.run().unwrap();
        let test_data: [(i32, Message, Result<Message, Error>); 4] = [
            (1, Message("Query-1".into()), Ok(Message("Reply-1".into()))),
            (2, Message("Query-2".into()), Ok(Message("Reply-2".into()))),
            (3, Message("Query-3".into()), Ok(Message("Reply-3".into()))),
            (4, Message("Query-4".into()), Ok(Message("Reply-4".into()))),
        ];
        let (local, remote) = Link::split(dbg);
        let mut listener = Listener::new(dbg, remote);
        let listener_handle = listener.run().unwrap();
        for (step, query, target) in test_data {
            let result: Result<Message, Error> = local.call(query);
            log::debug!("step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
            match (&result, &target) {
                (Ok(result), Ok(target)) => {
                    assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                }
                (Err(_), Err(_)) => {},
                _ => panic!("Error in step {} \nresult: {:?}\ntarget: {:?}", step, result, target)
            }
        }
        listener.exit();
        listener_handle.join().unwrap();
        test_duration.exit();
    }
    ///
    /// Message container
    #[derive(Debug, Encode, Decode, PartialEq)]
    struct Message(pub String);
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
                    match link.recv_query::<String>() {
                        CtxResult::Ok(query) => match query.as_str() {
                            "Query-1" => send_reply(&dbg, &mut link, "Reply-1"),
                            "Query-2" => send_reply(&dbg, &mut link, "Reply-2"),
                            "Query-3" => send_reply(&dbg, &mut link, "Reply-3"),
                            "Query-4" => send_reply(&dbg, &mut link, "Reply-4"),
                            _ => panic!("Unknown Query: {:?}", query)
                        }
                        CtxResult::Err(err) => {
                            log::warn!("{}.run | Error: {:?}", dbg, err);
                            break;
                        }
                        CtxResult::None => {},
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
        }
    }
}
