use std::{fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::JoinHandle, time::Duration};
use bincode::{Decode, Encode};
use coco::Stack;
use sal_core::error::Error;
use sal_sync::services::entity::{name::Name, point::point_tx_id::PointTxId};
use crate::{algorithm::context::ctx_result::CtxResult, kernel::types::channel::{Receiver, RecvTimeoutError, Sender}};
use super::DEFAULT_TIMEOUT;

///
/// Contains local side `send` & `recv` of `channel`
/// - provides simple direct to `send` & `recv`
/// - provides request operation
pub struct Link {
    txid: usize,
    name: Name,
    send: Sender<Vec<u8>>,
    recv: Stack<Receiver<Vec<u8>>>,
    timeout: Duration,
    bincode_config: bincode::config::Configuration,
    exit: Arc<AtomicBool>,
}
//
//
impl Link {
    ///
    /// Returns [Link] new instance
    /// - `send` - local side of channel.send
    /// - `recv` - local side of channel.recv
    /// - `exit` - exit signal for `recv_query` method
    pub fn new(parent: impl Into<String>, send: Sender<Vec<u8>>, recv: Receiver<Vec<u8>>) -> Self {
        let name = Name::new(parent, "Link");
        let loc_recv_st = Stack::new();
        loc_recv_st.push(recv);
        Self {
            txid: PointTxId::from_str(&name.join()),
            name,
            send, 
            recv: loc_recv_st,
            timeout: DEFAULT_TIMEOUT,
            bincode_config: bincode::config::standard(),
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
        let loc_recv_st = Stack::new();
        loc_recv_st.push(loc_recv);
        let rem_recv_st = Stack::new();
        rem_recv_st.push(rem_recv);
        (
            Self { 
                txid: PointTxId::from_str(&name.join()),
                name: name.clone(),
                send: loc_send, recv: loc_recv_st,
                timeout: DEFAULT_TIMEOUT,
                bincode_config: bincode::config::standard(),
                exit: Arc::new(AtomicBool::new(false)),
            },
            Self { 
                txid: PointTxId::from_str(&name.join()),
                name,
                send: rem_send, recv: rem_recv_st,
                timeout: DEFAULT_TIMEOUT,
                bincode_config: bincode::config::standard(),
                exit: Arc::new(AtomicBool::new(false)),
            },
        )
    }
    ///
    /// - Sends a request,
    /// - Await reply,
    /// - Returns parsed reply
    pub fn call<T: Decode<()> + Debug>(&self, query: impl Encode + Debug) -> Result<T, Error> {
        let error = Error::new(&self.name, "call");
        let q = format!("{:#?}", query);
        match bincode::encode_to_vec(query, self.bincode_config) {
            Ok(query) => match self.send.send(query) {
                Ok(_) => {
                    log::trace!("{}.req | Sent request: {q}", self.name);
                    match self.recv.pop() {
                        Some(recv) => match recv.recv() {
                            Ok(reply) => {
                                self.recv.push(recv);
                                match bincode::decode_from_slice(&reply, self.bincode_config) {
                                    Ok((reply, _)) => {
                                        log::trace!("{}.req | Reply received: {:#?}", self.name, reply);
                                        Ok(reply)
                                    }
                                    Err(err) => Err(error.pass_with(format!("Reply {:#?} decode error", reply), err.to_string())),
                                }
                            }
                            Err(err) => {
                                self.recv.push(recv);
                                Err(error.pass(err.to_string()))
                            }
                        }
                        None => Err(error.err("Recv - not found")),
                    }
                },
                Err(err) => Err(error.pass_with("Send request error", err.to_string())),
            }
            Err(err) => Err(error.pass_with(format!("Query encode {:?} error", q), err.to_string())),
        }
    }
    ///
    /// Listenning incomong events in the callback
    /// - Callback receives `Event`
    /// - Callback returns `Some<Event>` - to be sent
    /// - Callback returns None - nothing to be sent
    pub fn listen<In: Decode<()> + Debug, Out: Encode + Debug>(&mut self, op: impl Fn(In) -> Option<Out> + Send + 'static) -> Result<JoinHandle<()>, Error> {
        let error = Error::new(&self.name, "listen");
        let dbg = self.name.join();
        let send = self.send.clone();
        let recv = self.recv.pop().unwrap();
        let timeout = self.timeout;
        let config = self.bincode_config;
        let exit = self.exit.clone();
        log::debug!("{}.listen | Starting...", dbg);
        let handle = std::thread::Builder::new().name(dbg.clone()).spawn(move|| {
            'main: loop {
                match recv.recv_timeout(timeout) {
                    Ok(query) => {
                        log::trace!("{}.listen | Received query: {:#?}", dbg, query);
                        match bincode::decode_from_slice(&query, config) {
                            Ok((query, _)) => {
                                match (op)(query) {
                                    Some(reply) => {
                                        match bincode::encode_to_vec(&reply, config) {
                                            Ok(reply) => if let Err(err) = send.send(reply) {
                                                let err = error.pass_with("Send request error", err.to_string());
                                                log::error!("{}", err);
                                            }
                                            Err(err) => log::warn!("{}.listen | Encode error: {:#?}", dbg, err),
                                        }
                                    }
                                    None => {}
                                }
                            },
                            Err(err) => log::warn!("{}.listen | Decode error: {:#?}", dbg, err),
                        };
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
    pub fn recv_query<T: Decode<()> + Debug>(&self) -> CtxResult<T, Error> {
        let error = Error::new(&self.name, "recv_query");
        match self.recv.pop() {
            Some(recv) => match recv.recv_timeout(self.timeout) {
                Ok(query) => {
                    self.recv.push(recv);
                    match bincode::decode_from_slice(&query, self.bincode_config) {
                        Ok((query, _)) => {
                            log::trace!("{}.recv_query | Received query: {:#?}", self.name, query);
                            return CtxResult::Ok(query)
                        }
                        Err(err) => CtxResult::Err(
                            error.pass_with("Decode error", err.to_string()),
                        ),
                    }
                }
                Err(err) => {
                    self.recv.push(recv);
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
    pub fn send_reply(&self, reply: impl Encode + Debug) -> Result<(), Error> {
        let error = Error::new(&self.name, "send_reply");
        match bincode::encode_to_vec(reply, self.bincode_config) {
            Ok(reply) => match self.send.send(reply) {
                Ok(_) => Ok(()),
                Err(err) => Err(error.pass(err.to_string())),
            }
            Err(err) => Err(error.pass_with("Encode error", err.to_string())),
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
