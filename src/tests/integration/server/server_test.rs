use std::{io::{Read, Write}, net::TcpStream, sync::{Arc, atomic::{AtomicBool, Ordering}}};
#[cfg(test)]

use std::{sync::Once, time::{Duration, Instant}};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
use testing::{entities::test_value::Value, stuff::max_test_duration::TestDuration};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{
    algorithm::context::testing_ctx::TestingCtx,
    conf::Conf,
    infrostructure::{SelectCalculus, SelectDevDoc, SelectDevInfo},
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{Context, ContextWrite, InitialCtx},
    server::{CalculusQuery, Connection, Content, Cot, Event, Field, FieldId, Query, QueryId, Request, SelectAct, SelectContent, SelectCot, SelectReq, Server},
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
/// Testing such functionality / behavior
#[test]
fn query_calculus() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("query_calculus");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(10));
    test_duration.run().unwrap();
    let tp = ThreadPool::new(&dbg, Some(4));
    let conf: Conf = serde_yaml::from_str(r#"
        thread-pool:
            size: 15
        api:
            address:
                host: "0.0.0.0"
                port: "8081"
                database: "sss-computing"
            params:
                ship_id: 2
                project_id: NULL
        server:
            address: 0.0.0.0:3838
            connection:
                timeout: 100 ms  # ms
        calculus:
            max-time: 3 s
    "#).unwrap();
    let server = Server::new(
        &dbg,
        conf.clone(),
        tp.scheduler(),
        move |dbg, conf| { Box::new(
            //
            // Select handler for incomong messages by Content
            SelectContent::new(vec![
                // Handler for Content::Bytes
                (Content::Bytes, Box::new(SelectCot::new(vec![
                    // Handling incomong messages with `Cot::Act` by field `cmd`
                    (Cot::Act, Box::new(SelectAct::new(vec![
                        // Handling incomong commands
                        // ...
                    ]))),
                    // Handling incomong messages with Cot::Req by field `req`
                    (Cot::Req, Box::new(SelectReq::new(vec![
                        // Handling incomong requests
                        // ...
                    ]))),
                ]))),
                // Handler for Content::Empty
                (Content::Empty, Box::new(SelectCot::new(vec![
                    // Handling incomong messages with `Cot::Act` by field `cmd`
                    (Cot::Act, Box::new(SelectAct::new(vec![
                        // Handling incomong commands
                        // ...
                    ]))),
                    // Handling incomong messages with Cot::Req by field `req`
                    (Cot::Req, Box::new(SelectReq::new(vec![
                        // Handling incomong requests
                        // ...
                    ]))),
                ]))),
                // Handler for Content::Json
                (Content::Json, Box::new(SelectCot::new(vec![
                    // Handling incomong messages with `Cot::Act` by field `cmd`
                    (Cot::Act, Box::new(SelectAct::new(vec![
                        // Handling incomong command `DeviceStream`
                        (QueryId::Calculus, Box::new(SelectCalculus::new(
                            dbg,
                            conf.calculus.clone(),
                            tp.scheduler(),
                            FakeCalculus::new(dbg, Duration::from_millis(10)),
                        ))),
                        // Handling incomong command `DeviceStream`
                    ]))),
                    // Handling incomong messages with Cot::Req by field `req`
                    (Cot::Req, Box::new(SelectReq::new(vec![
                        // Handling incomong request `DeviceInfo`
                        (QueryId::DeviceInfo, Box::new(SelectDevInfo::new("assets/info/"))),
                        // Handling incomong request `DeviceDoc`
                        (QueryId::DeviceDoc, Box::new(SelectDevDoc::new("assets/info/"))),
                    ]))),
                ]))),
            ])
        )}
    );
    server.run().unwrap();
    server.wait().unwrap();
    // assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    test_duration.exit();
}
///
/// Evaluates entair ship calculations
struct FakeCalculus {
    dbg: Dbg,
    calc_time: Duration,
}
//
//
impl FakeCalculus {
    ///
    /// Returns [FakeCalculus] new instance
    /// - `calc_time` - duration until result will returned
    pub fn new(
        parent: impl Into<String>,
        calc_time: Duration,
    ) -> Self {
        Self {
            dbg: Dbg::new(parent, "FakeCalculus"),
            calc_time,
        }
    }
}
//
//
impl Eval<CalculusQuery, EvalResult> for FakeCalculus {
    fn eval(&self, query: CalculusQuery) -> EvalResult {
        let dbg = self.dbg.clone();
        let error = Error::new(&dbg, "eval");
        log::debug!("{dbg}.eval | query: {:?}", query);
        log::debug!("{dbg}.eval | Calculations...");
        let time = Instant::now();
        let mut r = 0.0;
        while time.elapsed().cmp(&self.calc_time).is_le() {
            for i in 0..10000 {
                let v = ( ((i as f32 * i as f32) + (i as f32 * i as f32) - (i as f32 * i as f32) * (i as f32 * i as f32)) / (i as f32 * i as f32)).sqrt();
                match i % 2 == 0 {
                    true => r = r * v,
                    false => r = r / v,
                }
            }
        }
        log::debug!("{dbg}.eval | result: {:?}", r);
        log::debug!("{dbg}.eval | Calculations - Ok, elapsed {:?}", time.elapsed());
        let result = TestingCtx {
            value: Value::Real(r),
        };
        let ctx = Context::new(InitialCtx {
            ..Default::default()
        });
        ctx.write(result).map_err(|err| error.pass(err))
    }
}
//
//
unsafe impl Send for FakeCalculus {}
///
/// Fake client for testing [Server]
struct FakeClient {
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
            loop {
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
                        match stream.write_all(&bytes) {
                            Ok(_) => {
                                let mut buf = vec![0; 1024 * 4];
                                match stream.read(&mut buf) {
                                    Ok(_) => {
                                        match message.parse(buf) {
                                            Ok(((((((_, FieldId(event_id)), content), cot), query_id), _len), bytes)) => {
                                                let response = Event::new(event_id, query_id, cot, content, bytes);
                                                log::debug!("{dbg}.run | Response received'{addr}', error: {:#?}", response);

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
                    Err(err) => log::warn!("{dbg}.run | Can't connect to '{addr}', error: {:?}", err),
                }
                if exit.load(Ordering::Acquire) {
                    break;
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
