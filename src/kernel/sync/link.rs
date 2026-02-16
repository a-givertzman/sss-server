use std::{fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::JoinHandle, time::Duration};
use bincode::{Decode, Encode};
use sal_core::error::Error;
use sal_sync::{services::entity::{Name, PointTxId}, sync::Owner};
use crate::kernel::types::channel::{Receiver, RecvTimeoutError, Sender};
use super::{LinkSend, DEFAULT_TIMEOUT};

///
/// Contains local side `send` & `recv` of `channel`
/// - provides simple direct to `send` & `recv`
/// - provides request operation
pub struct Link {
    txid: usize,
    name: Name,
    send: Sender<Vec<u8>>,
    recv: Owner<Receiver<Vec<u8>>>,
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
    /// - `exit` - exit signal for `listen` method
    pub fn new(parent: impl Into<String>, send: Sender<Vec<u8>>, recv: Receiver<Vec<u8>>) -> Self {
        let name = Name::new(parent, "Link");
        Self {
            txid: PointTxId::from_str(&name.join()),
            name,
            send, 
            recv: Owner::new(recv),
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
        (
            Self { 
                txid: PointTxId::from_str(&name.join()),
                name: name.clone(),
                send: loc_send, recv: Owner::new(loc_recv),
                timeout: DEFAULT_TIMEOUT,
                bincode_config: bincode::config::standard(),
                exit: Arc::new(AtomicBool::new(false)),
            },
            Self { 
                txid: PointTxId::from_str(&name.join()),
                name,
                send: rem_send, recv: Owner::new(rem_recv),
                timeout: DEFAULT_TIMEOUT,
                bincode_config: bincode::config::standard(),
                exit: Arc::new(AtomicBool::new(false)),
            },
        )
    }
    ///
    /// Returns Sender
    pub fn sender(&self) -> LinkSend {
        LinkSend::new(
            self.name.join(),
            self.send.clone(),
            self.bincode_config,
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
                    match self.recv.take() {
                        Some(recv) => match recv.recv() {
                            Ok(reply) => {
                                self.recv.replace(recv);
                                match bincode::decode_from_slice(&reply, self.bincode_config) {
                                    Ok((reply, _)) => {
                                        log::trace!("{}.req | Reply received: {:#?}", self.name, reply);
                                        Ok(reply)
                                    }
                                    Err(err) => Err(error.pass_with(format!("Reply {:#?} decode error", reply), err.to_string())),
                                }
                            }
                            Err(err) => {
                                self.recv.replace(recv);
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
        let recv = self.recv.take().unwrap();
        let timeout = self.timeout;
        let config = self.bincode_config;
        let exit = self.exit.clone();
        log::debug!("{}.listen | Starting...", dbg);
        let handle = std::thread::Builder::new().name(dbg.clone()).spawn(move|| {
            'main: loop {
                match recv.recv_timeout(Duration::from_micros(100)) {
                    Ok(query) => {
                        log::trace!("{}.listen | Received query: {:#?}", dbg, query);
                        match bincode::decode_from_slice(&query, config) {
                            Ok((query, _)) => {
                                match (op)(query) {
                                    Some(reply) => {
                                        match bincode::encode_to_vec(&reply, config) {
                                            Ok(reply) => if let Err(err) = send.send(reply) {
                                                let err = error.pass_with("Send reply error", err.to_string());
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
                if exit.load(Ordering::Acquire) {
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
    /// Tries receiving incoming `event` without waiting.
    /// - Returns Ok<Some<T>> if `Link` has `event`
    /// - Returns Ok<None> if `Link` is empty for now
    /// - Returns Err if `Link` is closed
    /// 
    /// **Important note**: this function is not lock-free as it acquires a mutex guard of the channel internal for a short time.
    pub fn try_recv<T: Decode<()> + Debug>(&self) -> Result<Option<T>, Error> {
        let error = Error::new(&self.name, "try_recv");
        match self.recv.take() {
            Some(recv) => match recv.try_recv() {
                Ok(query) => {
                    self.recv.replace(recv);
                    match query {
                        Some(query) => {
                            match bincode::decode_from_slice(&query, self.bincode_config) {
                                Ok((query, _)) => {
                                    log::trace!("{}.try_recv | Received query: {:#?}", self.name, query);
                                    return Ok(Some(query))
                                }
                                Err(err) => Err(
                                    error.pass_with("Decode error", err.to_string()),
                                ),
                            }
                        }
                        None => Ok(None),
                    }
                }
                Err(err) => {
                    self.recv.replace(recv);
                    Err(error.pass_with("Recv error", err.to_string()))
                }
            }
            None => Err(error.err("Recv - not found")),
        }
    }
    ///
    /// Tries receiving incoming `event` within a duration
    /// - Returns Ok<Some<T>> if `Link` has `event`
    /// - Returns Ok<None> if `Link` is empty within a duration
    /// - Returns Err if `Link` is closed
    pub fn recv_timeout<T: Decode<()> + Debug>(&self, duration: Duration) -> Result<Option<T>, Error> {
        let error = Error::new(&self.name, "recv_timeout");
        match self.recv.take() {
            Some(recv) => match recv.recv_timeout(duration) {
                Ok(query) => {
                    self.recv.replace(recv);
                    match bincode::decode_from_slice(&query, self.bincode_config) {
                        Ok((query, _)) => {
                            log::trace!("{}.try_recv | Received query: {:#?}", self.name, query);
                            return Ok(Some(query))
                        }
                        Err(err) => Err(
                            error.pass_with("Decode error", err.to_string()),
                        ),
                    }
                }
                Err(err) => {
                    self.recv.replace(recv);
                    match err {
                        kanal::ReceiveErrorTimeout::Timeout => Ok(None),
                        _ => Err(error.pass_with("Recv error", err.to_string()))
                    }
                }
            }
            None => Err(error.err("Recv - not found")),
        }
    }
    ///
    /// Receiving incomong events, bloking method
    /// - Returns Ok<T> if channel has query
    /// - Returns None if channel is empty for now
    /// - Returns Err if channel is closed
    pub fn recv<T: Decode<()> + Debug>(&self) -> Result<T, Error> {
        let error = Error::new(&self.name, "recv");
        match self.recv.take() {
            Some(recv) => match recv.recv() {
                Ok(query) => {
                    self.recv.replace(recv);
                    match bincode::decode_from_slice(&query, self.bincode_config) {
                        Ok((query, _)) => {
                            log::trace!("{}.recv | Received query: {:#?}", self.name, query);
                            return Ok(query)
                        }
                        Err(err) => Err(
                            error.pass_with("Decode error", err.to_string()),
                        ),
                    }
                }
                Err(err) => {
                    self.recv.replace(recv);
                    Err(error.pass_with("Recv error", err.to_string()))
                }
            }
            None => Err(error.err("Recv - not found")),
        }
    }
    ///
    /// Sending event
    pub fn send(&self, event: impl Encode + Debug) -> Result<(), Error> {
        let error = Error::new(&self.name, "send");
        match bincode::encode_to_vec(event, self.bincode_config) {
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
        self.exit.store(true, Ordering::Release);
        if let Err(err) = self.send.close() {
            log::trace!("{}.exit | Error: {:#?}", self.name, err);
        }
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
