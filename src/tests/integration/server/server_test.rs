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
    server::{CalculusQuery, Connection, Content, Cot, Event, Field, FieldId, Query, QueryId, Request, SelectAct, SelectContent, SelectCot, SelectReq, Server}, tests::integration::server::fake_client::FakeClient,
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
    let dbg = Dbg::own("query_calculus");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(30));
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
    let fake_client = FakeClient::new(&dbg, "0.0.0.0:3838");
    fake_client.run().unwrap();
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
