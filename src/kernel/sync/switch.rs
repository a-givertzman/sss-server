use std::{fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc}, time::Duration};
use sal_sync::{services::entity::{Name, PointTxId}, sync::Owner};
use crate::kernel::types::{channel::{Receiver, Sender}, fx_map::FxDashMap};
use super::link::Link;
///
/// 
pub struct Switch {
    txid: usize,
    name: Name,
    send: Sender<Vec<u8>>,
    recv: Owner<Receiver<Vec<u8>>>,
    subscribers: Arc<FxDashMap<String, Sender<Vec<u8>>>>,
    receivers_tx: Sender<(String, Receiver<Vec<u8>>)>,
    receivers_rx: Owner<Receiver<(String, Receiver<Vec<u8>>)>>,
    receivers: Arc<AtomicUsize>,
    timeout: Duration,
    exit: Arc<AtomicBool>,
}
//
//
impl Switch {
    ///
    /// Default timeout to await `recv`` operation, 300 ms
    const DEFAULT_TIMEOUT: Duration = Duration::from_millis(10);
    ///
    /// Returns [Switch] new instance
    /// - `send` - local side of channel.send
    /// - `recv` - local side of channel.recv
    /// - `exit` - exit signal for `recv_query` method
    pub fn new(parent: impl Into<String>, send: Sender<Vec<u8>>, recv: Receiver<Vec<u8>>) -> Self {
        let name = Name::new(parent, "Switch");
        let (receivers_tx, receivers_rx) = kanal::unbounded();
        Self {
            txid: PointTxId::from_str(&name.join()),
            name,
            send, 
            recv: Owner::new(recv),
            subscribers: Arc::new(FxDashMap::default()),
            receivers: Arc::new(AtomicUsize::new(0)),
            receivers_tx,
            receivers_rx: Owner::new(receivers_rx),
            timeout: Self::DEFAULT_TIMEOUT,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns Self and `remote: [Link]` new instances
    pub fn split(parent: impl Into<String>) -> (Self, Link) {
        let name = Name::new(parent, "Switch");
        let (loc_send, rem_recv) = kanal::unbounded();
        let (rem_send, loc_recv) = kanal::unbounded();
        let remote = Link::new(name.join(), rem_send, rem_recv);
        let (receivers_tx, receivers_rx) = kanal::unbounded();
        (
            Self { 
                txid: PointTxId::from_str(&name.join()),
                name: name.clone(),
                send: loc_send, recv: Owner::new(loc_recv),
                subscribers: Arc::new(FxDashMap::default()),
                receivers: Arc::new(AtomicUsize::new(0)),
                receivers_tx,
                receivers_rx: Owner::new(receivers_rx),
                timeout: Self::DEFAULT_TIMEOUT,
                exit: Arc::new(AtomicBool::new(false)),
            },
            remote,
        )
    }
    ///
    /// Returns connected `Link`
    pub fn link(&self) -> Link {
        let (loc_send, rem_recv) = kanal::unbounded();
        let (rem_send, loc_recv) = kanal::unbounded();
        let remote = Link::new(&format!("{}:{}", self.name, self.subscribers.len()), rem_send, rem_recv);
        let key = remote.name().join();
        self.subscribers.insert(String::clone(&key), loc_send);
        let receivers = self.receivers.clone();
        let len = receivers.load(Ordering::SeqCst);
        self.receivers_tx.send((key.to_owned(), loc_recv)).unwrap();
        while len == receivers.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(3));
        }
        remote
    }
    // ///
    // /// Entry point
    // pub fn run(&self) -> Result<ServiceHandles<()>, Error> {
    //     let dbg = self.name.join();
    //     log::info!("{}.run | Remote | Starting...", dbg);
    //     let subscribers = self.subscribers.clone();
    //     let exit = self.exit.clone();
    //     let recv = self.recv.pop().unwrap();
    //     let timeout = self.timeout;
    //     let handle1 = std::thread::Builder::new().name(dbg.clone()).spawn(move|| {
    //         log::debug!("{}.run | Remote | Start", dbg);
    //         'main: loop {
    //             log::trace!("{}.run | Remote | Subscriber: {}", dbg, subscribers.len());
    //             match recv.recv_timeout(timeout) {
    //                 Ok(event) => {
    //                     log::trace!("{}.run | Request: {:?}", dbg, event);
    //                     match event.cot() {
    //                         Cot::Inf | Cot::Act | Cot::Req => {
    //                             for item in subscribers.iter() {
    //                                 let (_key, subscriber) = item.pair();
    //                                 if let Err(err) = subscriber.send(event.clone()) {
    //                                     log::warn!("{}.run | Send error: {:?}", dbg, err);
    //                                 }
    //                             }
    //                         }
    //                         Cot::ReqCon | Cot::ReqErr => {
    //                             let key = event.name();
    //                             match subscribers.get(&key) {
    //                                 Some(subscriber) => {
    //                                     if let Err(err) = subscriber.send(event.clone()) {
    //                                         log::warn!("{}.run | Send error: {:?}", dbg, err);
    //                                     }
    //                                 },
    //                                 None => {
    //                                     log::warn!("{}.run | Subscriber not found: {:?}", dbg, key);
    //                                 },
    //                             }
    //                         }
    //                         _ => log::warn!("{}.run | Uncnown message received: {:?}", dbg, event),
    //                     }
    //                 },
    //                 Err(err) => match err {
    //                     std::sync::mpsc::RecvTimeoutError::Timeout => {
    //                         log::trace!("{}.run | Remote | Listening...", dbg);
    //                     },
    //                     std::sync::mpsc::RecvTimeoutError::Disconnected => {
    //                         if log::max_level() >= log::LevelFilter::Trace {
    //                             log::warn!("{}.run | Receive error, all receivers has been closed", dbg);
    //                         }
    //                     }
    //                 },
    //             }
    //             if exit.load(Ordering::SeqCst) {
    //                 break 'main;
    //             }
    //         }
    //         log::info!("{}.run | Remote | Exit", dbg);
    //     });
    //     let dbg = self.name.join();
    //     log::info!("{}.run | Remote | Starting - Ok", dbg);
    //     log::info!("{}.run | Locals | Starting...", dbg);
    //     let send = self.send.clone();
    //     let timeout = self.timeout;
    //     let interval = self.timeout;    //Duration::from_millis(1000);
    //     let self_receivers = self.receivers.clone();
    //     let receivers_rx = self.receivers_rx.pop().unwrap();
    //     let exit = self.exit.clone();
    //     let handle2 = std::thread::Builder::new().name(dbg.clone()).spawn(move|| {
    //         log::debug!("{}.run | Locals | Start", dbg);
    //         let mut receivers = FxIndexMap::default();
    //         'main: loop {
    //             for (key, receiver) in receivers_rx.try_iter() {
    //                 receivers.insert(key, receiver);
    //                 self_receivers.fetch_add(1, Ordering::SeqCst);
    //             }
    //             log::debug!("{}.run | Locals | Receivers: {}", dbg, receivers.len());
    //             let cycle = Instant::now();
    //             for (_key, receiver) in &receivers {
    //                 match receiver.recv_timeout(timeout) {
    //                     Ok(event) => {
    //                         log::trace!("{}.run | Received from locals: {:?}", dbg, event);
    //                         if let Err(err) = send.send(event) {
    //                             log::warn!("{}.run | Send error: {:?}", dbg, err);
    //                         }
    //                     }
    //                     Err(err) => match err {
    //                         RecvTimeoutError::Timeout => {
    //                             log::trace!("{}.run | Locals | Listening...", dbg);
    //                         }
    //                         RecvTimeoutError::Disconnected => {
    //                             if log::max_level() >= log::LevelFilter::Trace {
    //                                 log::warn!("{}.run | Receive error, all senders has been closed", dbg);
    //                             }
    //                         }
    //                     }
    //                 }
    //                 if exit.load(Ordering::SeqCst) {
    //                     break 'main;
    //                 }
    //             }
    //             if exit.load(Ordering::SeqCst) {
    //                 break 'main;
    //             }
    //             if receivers.len() == 0 {
    //                 let elapsed = cycle.elapsed();
    //                 if elapsed < interval {
    //                     std::thread::sleep(interval - elapsed);
    //                 }
    //             }
    //         }
    //         log::info!("{}.run | Locals | Exit", dbg);
    //     });
    //     let dbg = self.name.join();
    //     let error = Error::new(&dbg, "run");
    //     log::info!("{}.run | Locals | Starting - Ok", dbg);
    //     match (handle1, handle2) {
    //         (Ok(h1), Ok(h2)) => Ok(ServiceHandles::new(vec![
    //             (format!("{dbg}/Remote"), h1),
    //             (format!("{dbg}/Locals"), h2),
    //         ])),
    //         (Ok(_), Err(err)) => {
    //             self.exit.store(true, Ordering::SeqCst);
    //             Err(error.pass_with("Failed to start 'Locals'", err.to_string()))
    //         }
    //         (Err(err), Ok(_)) => {
    //             self.exit.store(true, Ordering::SeqCst);
    //             Err(error.pass_with("Failed to start 'Remote'", err.to_string()))
    //         }
    //         (Err(err1), Err(err2)) => {
    //             self.exit.store(true, Ordering::SeqCst);
    //             Err(error.pass(format!("Failed to start \n\tRemote: {err1} \n\tLocals: {err2}")))
    //         }
    //     }
    // }
    // ///
    // /// Sends "exit" signal to the service's task
    // pub fn exit(&self) {
    //     self.exit.store(true, Ordering::SeqCst);
    // }
}
//
//
impl Debug for Switch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Switch")
            .field("txid", &self.txid)
            .field("name", &self.name)
            // .field("send", &self.send)
            // .field("recv", &self.recv)
            // .field("subscribers", &self.subscribers)
            // .field("receivers", &self.receivers)
            .field("timeout", &self.timeout)
            .field("exit", &self.exit)
            .finish()
    }
}