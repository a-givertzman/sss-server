use std::{fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}, Arc}, thread::JoinHandle, time::Duration};
use sal_core::error::Error;
use sal_sync::services::entity::{cot::Cot, name::Name, point::{point::Point, point_hlr::PointHlr, point_tx_id::PointTxId}, status::status::Status};
use serde::{de::DeserializeOwned, Serialize};
use crate::algorithm::context::ctx_result::CtxResult;
///
/// Contains local side `send` & `recv` of `channel`
/// - provides simple direct to `send` & `recv`
/// - provides request operation
pub struct Link {
    txid: usize,
    name: Name,
    send: Sender<Point>,
    recv: Option<Receiver<Point>>,
    timeout: Duration,
    exit: Arc<AtomicBool>,
}
//
//
impl Link {
    ///
    /// Default timeout to await `recv`` operation, 300 ms
    const DEFAULT_TIMEOUT: Duration = Duration::from_millis(10);
    ///
    /// Returns [Link] new instance
    /// - `send` - local side of channel.send
    /// - `recv` - local side of channel.recv
    /// - `exit` - exit signal for `recv_query` method
    pub fn new(parent: impl Into<String>, send: Sender<Point>, recv: Receiver<Point>) -> Self {
        let name = Name::new(parent, "Link");
        Self {
            txid: PointTxId::from_str(&name.join()),
            name,
            send, 
            recv: Some(recv),
            timeout: Self::DEFAULT_TIMEOUT,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns it's name
    pub fn name(&self) -> Name {
        self.name.clone()
    }
    ///
    /// Returns `local: [Link] remote: [Link]` new instance
    pub fn split(parent: impl Into<String>) -> (Self, Self) {
        let name = Name::new(parent, "Link");
        let (loc_send, rem_recv) = kanal::unbounded();
        let (rem_send, loc_recv) = kanal::unbounded();
        (
            Self { 
                txid: PointTxId::from_str(&name.join()),
                name: name.clone(),
                send: loc_send, recv: Some(loc_recv),
                timeout: Self::DEFAULT_TIMEOUT,
                exit: Arc::new(AtomicBool::new(false)),
            },
            Self { 
                txid: PointTxId::from_str(&name.join()),
                name,
                send: rem_send, recv: Some(rem_recv),
                timeout: Self::DEFAULT_TIMEOUT,
                exit: Arc::new(AtomicBool::new(false)),
            },
        )
    }
    ///
    /// - Sends a request, 
    /// - Await reply,
    /// - Returns parsed reply
    pub fn req<T: DeserializeOwned + Debug>(&self, query: impl Serialize + Debug) -> Result<T, Error> {
        let error = Error::new(&self.name, "req");
        match serde_json::to_string(&query) {
            Ok(query) => {
                let query = Point::String(PointHlr::new(
                    self.txid, &self.name.join(),
                    query, Status::Ok, Cot::Req,
                    chrono::offset::Utc::now(),
                ));
                let timeout = self.timeout;
                let timeout = Duration::from_secs(1000);
                match self.send.send(query.clone()) {
                    Ok(_) => {
                        log::trace!("{}.req | Sent request: {:#?}", self.name, query);
                        match &self.recv {
                            Some(recv) => match recv.recv_timeout(timeout) {
                                Ok(reply) => {
                                    log::trace!("{}.req | Received reply: {:#?}", self.name, reply);
                                    let reply = reply.as_string().value;
                                    match serde_json::from_str::<T>(reply.as_str()) {
                                        Ok(reply) => {
                                            Ok(reply)
                                        }
                                        Err(err) => Err(error.pass_with(format!("Deserialize error for {:?} in {}", std::any::type_name::<T>(), reply), err.to_string())),
                                    }
                                }
                                _ => Err(error.err(format!("{}.req | Request timeout ({:?})", self.name, timeout))),
                            }
                            None => todo!(),
                        }
                    },
                    Err(err) => Err(error.pass_with("Send request error", err.to_string())),
                }
            }
            Err(err) => Err(error.pass_with(format!("Serialize query error in query: {:#?}", query), err.to_string())),
        }
    }
    ///
    /// Listenning incomong events in the callback
    /// - Callback receives `Point`
    /// - Callback returns `Some<Point>` - to be sent
    /// - Callback returns None - nothing to be sent
    pub fn listen(&mut self, op: impl Fn(Point) -> Option<Point> + Send + 'static) -> Result<JoinHandle<()>, Error> {
        let error = Error::new(&self.name, "listen");
        let dbg = self.name.join();
        let send = self.send.clone();
        let recv = self.recv.take().unwrap();
        let timeout = self.timeout;
        let exit = self.exit.clone();
        log::debug!("{}.listen | Starting...", dbg);
        let handle = std::thread::Builder::new().name(dbg.clone()).spawn(move|| {
            'main: loop {
                match recv.recv_timeout(timeout) {
                    Ok(query) => {
                        log::trace!("{}.listen | Received query: {:#?}", dbg, query);
                        match (op)(query) {
                            Some(reply) => if let Err(err) = send.send(reply) {
                                let err = error.pass_with("Send request error", err.to_string());
                                log::error!("{}", err);
                            }
                            None => {}
                        }
                    }
                    Err(err) => match err {
                        mpsc::RecvTimeoutError::Timeout => {}
                        mpsc::RecvTimeoutError::Disconnected => {
                            if log::max_level() >= log::LevelFilter::Trace {
                                log::warn!("{}.listen | Recv error: {:#?}", dbg, err);
                            }
                            std::thread::sleep(timeout);
                        }
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    break 'main;
                }
            }
            log::debug!("{}.listen | Exit", dbg);
        });
        let dbg = self.name.join();
        let error = Error::new(&self.name, "listen");
        log::debug!("{}.listen | Starting - Ok", dbg);
        handle.map_err(|err| error.pass(err.to_string()))
    }
    ///
    /// Receiving incomong events
    /// - Returns Ok<T> if channel has query
    /// - Returns None if channel is empty for now
    /// - Returns Err if channel is closed
    pub fn recv_query<T: DeserializeOwned + Debug>(&self) -> CtxResult<T, Error> {
        let error = Error::new(&self.name, "recv_query");
        match &self.recv {
            Some(recv) => match recv.recv_timeout(self.timeout) {
                Ok(query) => {
                    log::trace!("{}.recv_query | Received query: {:#?}", self.name, query);
                    let quyru = query.as_string().value;
                    match serde_json::from_str::<T>(quyru.as_str()) {
                        Ok(query) => {
                            return CtxResult::Ok(query)
                        }
                        Err(err) => CtxResult::Err(
                            error.pass_with(
                                format!("Deserialize error for {:?} in {}", std::any::type_name::<T>(), quyru),
                                err.to_string()
                            ),
                        ),
                    }
                }
                Err(err) => {
                    match err {
                        std::sync::mpsc::RecvTimeoutError::Timeout => CtxResult::None,
                        std::sync::mpsc::RecvTimeoutError::Disconnected => CtxResult::Err(
                            error.pass_with("Recv error", err.to_string()),
                        ),
                    }
                }
            }
            None => todo!(),
        }
    }
    ///
    /// Receiving incomong events with sender name
    /// - Returns Ok<T> if channel has query
    /// - Returns None if channel is empty for now
    /// - Returns Err if channel is closed
    pub fn recv_query_from<T: DeserializeOwned + Debug>(&self) -> CtxResult<(String, T), Error> {
        let error = Error::new(&self.name, "recv_query_from");
        match &self.recv {
            Some(recv) => match recv.recv_timeout(self.timeout) {
                Ok(query) => {
                    log::debug!("{}.recv_query_from | Received query: {:#?}", self.name, query);
                    let name = query.name();
                    let quyru = query.as_string().value;
                    match serde_json::from_str::<T>(quyru.as_str()) {
                        Ok(query) => {
                            return CtxResult::Ok((name, query))
                        }
                        Err(err) => CtxResult::Err(
                            error.pass_with(
                                format!("Deserialize error for {:?} in {}", std::any::type_name::<T>(), quyru),
                                err.to_string()
                            ),
                        ),
                    }
                }
                Err(err) => {
                    match err {
                        std::sync::mpsc::RecvTimeoutError::Timeout => CtxResult::None,
                        std::sync::mpsc::RecvTimeoutError::Disconnected => CtxResult::Err(
                            error.pass_with("Recv error", err.to_string()),
                        ),
                    }
                }
            }
            None => todo!(),
        }
    }
    ///
    /// Sending event
    pub fn send_reply(&self, reply: impl Serialize + Debug) -> Result<(), Error> {
        let error = Error::new(&self.name, "send_reply");
        match serde_json::to_string(&reply) {
            Ok(reply) => {
                let reply = Point::new(self.txid, &self.name.join(), reply);
                match self.send.send(reply) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(error.pass_with("Send request error", err.to_string())),
                }
            }
            Err(err) => Err(error.pass_with(format!("Serialize reply error in: {:#?}", reply), err.to_string())),
        }
    }
    ///
    /// Returns internal `exit` signal to be paired
    pub fn exit_pair(&self) -> Arc<AtomicBool> {
        self.exit.clone()
    }
    ///
    /// Sends "exit" signal to the `listen` task
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
//
//
unsafe impl Sync for Link {}
//
//
impl Debug for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Link")
        .field("txid", &self.txid)
        .field("name", &self.name)
        // .field("send", &self.send)
        // .field("recv", &self.recv)
        .field("timeout", &self.timeout)
        .finish()
    }
}
