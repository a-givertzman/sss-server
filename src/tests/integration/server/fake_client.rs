use std::{io::{Read, Write}, net::TcpStream, sync::{Arc, atomic::{AtomicBool, Ordering}}};

use sal_core::{dbg::Dbg, error::Error};

use crate::server::{CalculusQuery, CalculusStatus, Connection, Content, Cot, Event, Field, FieldId, Query, QueryId, Reply, extract};

///
/// Fake client for testing [Server]
pub(super) struct FakeClient {
    addr: String,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
impl FakeClient {
    pub fn new(parent: impl Into<String>, addr: impl Into<String>,) -> Self {
        let dbg = Dbg::new(parent, "FakeClient");
        Self {
            addr: addr.into(),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    pub fn run(&self) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        let addr = self.addr.clone();
        let exit = self.exit.clone();
        let event_id = 0;
        std::thread::spawn(move || {
            let mut message = Connection::tcp_message(&dbg);
            'main: loop {
                match TcpStream::connect(&addr) {
                    Ok(mut stream) => {
                        let query_id = QueryId::Calculus;
                        let cot = Cot::Act;
                        let content = Content::Json;
                        let query = Query::Calculus(CalculusQuery { ship_id: 111, project_id: "TestProj".into() });
                        // let request = Request::new(event_id, query_id, cot, content, query);
                        let bytes = serde_json::to_vec(&query).unwrap();
                        let event = Event::new(event_id, query_id, cot, content, bytes);
                        let bytes = message.build(&[
                            Field::Const,
                            Field::U32(event.id),
                            Field::Byte(event.content.into()),
                            Field::Byte(event.cot as u8),
                            Field::U32(event.query_id as u32),
                            Field::U32(event.bytes.len() as u32),
                            Field::Bytes(event.bytes),
                        ]);
                        'read: loop {
                            match stream.write_all(&bytes) {
                                Ok(_) => {
                                    let mut buf = vec![0; 1024 * 4];
                                    match stream.read(&mut buf) {
                                        Ok(_) => {
                                            match message.parse(buf) {
                                                Ok(((((((_, FieldId(event_id)), content), cot), query_id), _len), bytes)) => {
                                                    let response = Event::new(event_id, query_id, cot, content, bytes);
                                                    log::debug!("{dbg}.run | Response received'{addr}': {:?}", response);
                                                    let reply: Reply = serde_json::from_slice(&response.bytes).unwrap();
                                                    log::debug!("{dbg}.run | Reply: {:#?}", reply);
                                                    if let Ok(reply) = extract!(reply, Reply::Calculus) {
                                                        if reply.status == CalculusStatus::Done {
                                                            break 'read;
                                                        }

                                                    }
                                                }
                                                Err(err) => log::trace!("{dbg}.run | Can't parse message, error: {:?}", err),
                                            }
                                        }
                                        Err(err) => log::trace!("{dbg}.run | Can't read socket, error: {:?}", err),
                                    }
                                }
                                Err(err) => log::warn!("{dbg}.run | Can't write to socket '{addr}', error: {:?}", err),
                            }
                        }
                    }
                    Err(err) => log::warn!("{dbg}.run | Can't connect to '{addr}', error: {:?}", err),
                }
                if exit.load(Ordering::Acquire) {
                    break 'main;
                }
            }
        });
        Ok(())
    }
    ///
    /// Sends exit signal to main tread
    #[allow(unused)]
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
