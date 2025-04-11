use std::{any::Any, fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread::JoinHandle, time::Duration};
use sal_core::error::Error;
use sal_sync::services::entity::{name::Name, point::point_tx_id::PointTxId};
use crate::{algorithm::context::ctx_result::CtxResult, kernel::types::channel::{Receiver, RecvTimeoutError, Sender}};

use super::link_event::LinkEvent;
///
/// Contains local side `send` & `recv` of `channel`
/// - provides simple direct to `send` & `recv`
/// - provides request operation
pub struct Link {
    txid: usize,
    name: Name,
    send: Sender<Box<dyn Any + Send>>,
    recv: Option<Receiver<Box<dyn Any + Send>>>,
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
    pub fn new(parent: impl Into<String>, send: Sender<Box<dyn Any + Send>>, recv: Receiver<Box<dyn Any + Send>>) -> Self {
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
    pub fn split(parent: impl Into<String>) -> (Link, Link) {
        let name = Name::new(parent, "Link");
        let (loc_send, rem_recv) = kanal::unbounded();
        let (rem_send, loc_recv) = kanal::unbounded();
        (
            Link { 
                txid: PointTxId::from_str(&name.join()),
                name: name.clone(),
                send: loc_send, recv: Some(loc_recv),
                timeout: Self::DEFAULT_TIMEOUT,
                exit: Arc::new(AtomicBool::new(false)),
            },
            Link { 
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
    pub fn call<'a, T: Debug + 'static>(&self, query: impl Any + Debug + Clone + Send) -> Result<T, Error> {
        let error = Error::new(&self.name, "call");
        let q = format!("{:#?}", query);
        match self.send.send(Box::new(query.clone())) {
            Ok(_) => {
                log::trace!("{}.req | Sent request: {q}", self.name);
                match &self.recv {
                    Some(recv) => match recv.recv() {
                        Ok(reply) => {
                            let reply = reply.downcast::<T>().unwrap();
                            log::trace!("{}.req | Reply received: {:#?}", self.name, *reply);
                            Ok(*reply)
                        }
                        Err(err) => Err(error.pass(err.to_string())),
                    }
                    None => todo!(),
                }
            },
            Err(err) => Err(error.pass_with("Send request error", err.to_string())),
        }
    }
    ///
    /// Listenning incomong events in the callback
    /// - Callback receives `Point`
    /// - Callback returns `Some<Point>` - to be sent
    /// - Callback returns None - nothing to be sent
    pub fn listen<T: Send + 'static>(&mut self, op: impl Fn(T) -> Option<T> + Send + 'static) -> Result<JoinHandle<()>, Error> {
        let error = Error::new(&self.name, "listen");
        let dbg = self.name.join();
        let send = Arc::new(Mutex::new(self.send.clone()));
        let recv = Arc::new(Mutex::new(self.recv.take().unwrap()));
        let timeout = self.timeout;
        let exit = self.exit.clone();
        log::debug!("{}.listen | Starting...", dbg);
        let handle = std::thread::Builder::new().name(dbg.clone()).spawn(move|| {
            let send = send.lock().unwrap();
            let recv = recv.lock().unwrap();
            'main: loop {
                match recv.recv_timeout(timeout) {
                    Ok(query) => {
                        log::trace!("{}.listen | Received query: {:#?}", dbg, query);
                        let query: Box<T> = query.downcast().unwrap();
                        match (op)(*query) {
                            Some(reply) => if let Err(err) = send.send(Box::new(reply)) {
                                let err = error.pass_with("Send request error", err.to_string());
                                log::error!("{}", err);
                            }
                            None => {}
                        }
                    }
                    Err(err) => match err {
                        RecvTimeoutError::Timeout => {}
                        _ => {
                            if log::max_level() >= log::LevelFilter::Trace {
                                log::warn!("{}.listen | Recv error: {:#?}", dbg, err);
                            }
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
    pub fn recv_query<T: Debug + 'static>(&self) -> CtxResult<T, Error> {
        let error = Error::new(&self.name, "recv_query");
        match &self.recv {
            Some(recv) => match recv.recv_timeout(self.timeout) {
                Ok(query) => {
                    let query: T = *query.downcast().unwrap();
                    log::trace!("{}.recv_query | Received query: {:#?}", self.name, query);
                    return CtxResult::Ok(query)
                }
                Err(err) => {
                    match err {
                        RecvTimeoutError::Timeout => CtxResult::None,
                        _ => CtxResult::Err(
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
    pub fn send_reply(&self, reply: impl Any + Send) -> Result<(), Error> {
        let error = Error::new(&self.name, "send_reply");
        match self.send.send(Box::new(reply)) {
            Ok(_) => Ok(()),
            Err(err) => Err(error.pass(err.to_string())),
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
