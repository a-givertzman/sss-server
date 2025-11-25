use std::{io::{Read, Write}, net::TcpStream, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::sync::{Handles, Owner};
use crate::{
    server::{Connection, Event, Field, FieldId, Reply},
    tests::integration::server::server_test::TestCase,
};

///
/// Fake client for testing [Server]
pub(super) struct FakeClient {
    addr: String,
    test_data: Owner<Vec<(usize, TestCase)>>,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
impl FakeClient {
    pub fn new(parent: impl Into<String>, addr: impl Into<String>, test_data: impl Into<Vec<(usize, TestCase)>>) -> Self {
        let dbg = Dbg::new(parent, "FakeClient");
        Self {
            addr: addr.into(),
            test_data: Owner::new(test_data.into()),
            handles: Handles::new(&dbg),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    pub fn run(&self) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        log::warn!("{dbg}.run | Start");
        let addr = self.addr.clone();
        let test_data = self.test_data.take().unwrap();
        let exit = self.exit.clone();
        let handle = std::thread::spawn(move || {
            let mut message = Connection::tcp_message(&dbg);
            'main: loop {
                log::warn!("{dbg}.run | Connecting to {addr}...");
                match TcpStream::connect(&addr) {
                    Ok(mut stream) => {
                        log::warn!("{dbg}.run | Connecting to {addr} - Ok");
                        log::warn!("{dbg}.run | Handling {} Test events", test_data.len());
                        for (step, test_case) in &test_data {
                            match test_case {
                                TestCase::Request(request) => {
                                    log::debug!("{dbg}.run | Step {step} Query: {:#?}", serde_json::to_string(&request.query));
                                    let bytes = serde_json::to_vec(&request.query).unwrap();
                                    let event = Event::new(request.event_id, request.query_id, request.cot, request.content, bytes);
                                    let bytes = message.build(&[
                                        Field::Const,
                                        Field::U32(event.id),
                                        Field::Byte(event.content.into()),
                                        Field::Byte(event.cot as u8),
                                        Field::U32(event.query_id as u32),
                                        Field::U32(event.bytes.len() as u32),
                                        Field::Bytes(event.bytes),
                                    ]);
                                    match stream.write_all(&bytes) {
                                        Ok(_) => log::debug!("{dbg}.run | Step {step} Query {:?} - sent", request.query_id),
                                        Err(err) => log::warn!("{dbg}.run | Step {step} Can't write Query {:?} to socket '{addr}', error: {:?}", request.query, err),
                                    }
                                }
                                TestCase::Reply(target) => {
                                    'read: loop {
                                        let mut buf = vec![0; 1024 * 4];
                                        match stream.read(&mut buf) {
                                            Ok(_) => {
                                                match message.parse(buf) {
                                                    Ok(((((((_, FieldId(event_id)), content), cot), query_id), _len), bytes)) => {
                                                        let response = Event::new(event_id, query_id, cot, content, bytes);
                                                        log::debug!("{dbg}.run | Step {step} Response received from'{addr}': \n\tid {}, Query {:?}, Content {:?}, Cot {:?}", response.id, response.query_id, response.content, response.cot);
                                                        let result: Reply = serde_json::from_slice(&response.bytes).unwrap();
                                                        log::debug!("{dbg}.run | Step {step} Reply: {:#?}", str::from_utf8(&response.bytes));
                                                        // log::debug!("{dbg}.run | Step {step} Reply: {:#?}", result);
                                                        assert!(&result == target, "Step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                                                        break 'read;
                                                    }
                                                    Err(err) => log::trace!("{dbg}.run | Step {step} Can't parse message, error: {:?}", err),
                                                }
                                            }
                                            Err(err) => log::trace!("{dbg}.run | Step {step} Can't read socket, error: {:?}", err),
                                        }
                                    }
                                }
                                TestCase::Timeout(duration) => std::thread::sleep(*duration),
                            }
                            if exit.load(Ordering::Acquire) {
                                break 'main;
                            }
                        }
                        break 'main;
                    }
                    Err(err) => log::warn!("{dbg}.run | Can't connect to '{addr}', error: {:?}", err),
                }
                if exit.load(Ordering::Acquire) {
                    break 'main;
                }
            }
            log::warn!("{dbg}.run | Exit");
        });
        self.handles.push(handle);
        Ok(())
    }
    ///
    /// Returns when internal thread's will finished
    #[allow(unused)]
    pub fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    ///
    /// Sends exit signal to main tread
    #[allow(unused)]
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
