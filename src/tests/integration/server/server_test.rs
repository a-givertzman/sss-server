use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
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
    kernel::{EvalEx, types::eval_result::EvalResult},
    prelude::{Context, ContextWrite, InitialCtx},
    server::{CalculusQuery, CalculusReply, CalculusStatus, Content, Cot, Query, QueryId, Reply, Request, SelectAct, SelectContent, SelectCot, SelectReq, Server}, tests::integration::server::fake_client::FakeClient,
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
    DebugSession::new()
        .filter(LogLevel::Debug)
        .module("sal_sync::thread_pool", LogLevel::Info)
        .init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("Server-query_calculus");
    log::debug!("{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(30));
    test_duration.run().unwrap();
    let test_data = [
        (01, TestCase::Timeout(Duration::from_millis(100))),
        (02, TestCase::Request(Request { event_id: 0, query_id: QueryId::Calculus, cot: Cot::Act, content: Content::Json, query: Query::Calculus(CalculusQuery { ship_id: 111, project_id: "Test Proj".into() }) })),
        (03, TestCase::Reply(Reply::Calculus(CalculusReply { status: CalculusStatus::Ongoing }))),
        (04, TestCase::Reply(Reply::Calculus(CalculusReply { status: CalculusStatus::Done }))),
    ];
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
                            FakeCalculus::new(dbg, Duration::from_millis(800)),
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
    let fake_client = FakeClient::new(&dbg, "0.0.0.0:3838", test_data);
    fake_client.run().unwrap();
    fake_client.wait().unwrap();
    log::debug!("{dbg} | fake_client - Done");
    server.exit();
    server.wait().unwrap();
    log::debug!("{dbg} | server - Done");
    // assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    test_duration.exit();
}
///
/// Evaluates entair ship calculations
struct FakeCalculus {
    dbg: Dbg,
    calc_time: Duration,
    exit: Arc<AtomicBool>,
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
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
}
//
//
impl EvalEx<CalculusQuery, EvalResult> for FakeCalculus {
    fn eval(&self, query: CalculusQuery) -> EvalResult {
        let dbg = self.dbg.clone();
        let error = Error::new(&dbg, "eval");
        let exit = self.exit.clone();
        log::debug!("{dbg}.eval | query: {:?}", query);
        log::debug!("{dbg}.eval | Calculations...");
        let time = Instant::now();
        let mut r = 1.0;
        'main: while time.elapsed().cmp(&self.calc_time).is_le() {
            for i in 1..10000 {
                r += 1.0 / ((i as f32 * i as f32) * 2.0 - i as f32).sqrt();
                if exit.load(Ordering::Acquire) {
                    break 'main;
                }
            }
            if exit.load(Ordering::Acquire) {
                break 'main;
            }
        }
        let ctx = Context::new(InitialCtx {
            ..Default::default()
        });
        let result = if exit.load(Ordering::Acquire) {
            log::debug!("{dbg}.eval | Calculations - Canceled, elapsed {:?}", time.elapsed());
            TestingCtx {
                value: Value::String("Canceled".into()),
            }
        } else {
            log::debug!("{dbg}.eval | result: {:?}", r);
            log::debug!("{dbg}.eval | Calculations - Ok, elapsed {:?}", time.elapsed());
            TestingCtx {
                value: Value::Real(r),
            }
        };
        ctx.write(result).map_err(|err| error.pass(err))
    }
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::Release);
    }
}
//
//
unsafe impl Send for FakeCalculus {}
///
/// Test case is a variant of data in the case sequence
/// Used for current test only
pub(super) enum TestCase {
    /// FakeClient sends request
    Request(Request<QueryId>),
    /// FakeClient expect reply
    Reply(Reply),
    /// FakeClient wait before next operation
    Timeout(Duration),
}
