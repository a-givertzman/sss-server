use std::{fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc::{self, Receiver, Sender}, Arc}, time::{Duration, Instant}};
use coco::Stack;
use sal_sync::services::entity::{cot::Cot, error::str_err::StrErr, name::Name, point::{point::Point, point_tx_id::PointTxId}};
use tokio::task::JoinSet;
use crate::kernel::types::fx_map::{FxDashMap, FxIndexMap};
use super::link::Link;
///
/// 
pub struct Switch {
    txid: usize,
    name: Name,
    send: Sender<Point>,
    recv: Stack<Receiver<Point>>,
    subscribers: Arc<FxDashMap<String, Sender<Point>>>,
    receivers_tx: Sender<(String, Receiver<Point>)>,
    receivers_rx: Stack<Receiver<(String, Receiver<Point>)>>,
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
    pub fn new(parent: impl Into<String>, send: Sender<Point>, recv: Receiver<Point>) -> Self {
        let name = Name::new(parent, "Switch");
        let stack = Stack::new();
        stack.push(recv);
        let (receivers_tx, receivers_rx) = mpsc::channel();
        let receivers_rx_stack = Stack::new();
        receivers_rx_stack.push(receivers_rx);
        Self {
            txid: PointTxId::from_str(&name.join()),
            name,
            send, 
            recv: stack,
            subscribers: Arc::new(FxDashMap::default()),
            receivers: Arc::new(AtomicUsize::new(0)),
            receivers_tx,
            receivers_rx: receivers_rx_stack,
            timeout: Self::DEFAULT_TIMEOUT,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns Self and `remote: [Link]` new instances
    pub fn split(parent: impl Into<String>) -> (Self, Link) {
        let name = Name::new(parent, "Switch");
        let (loc_send, rem_recv) = mpsc::channel();
        let (rem_send, loc_recv) = mpsc::channel();
        let remote = Link::new(name.join(), rem_send, rem_recv);
        let stack = Stack::new();
        stack.push(loc_recv);
        let (receivers_tx, receivers_rx) = mpsc::channel();
        let receivers_rx_stack = Stack::new();
        receivers_rx_stack.push(receivers_rx);
        (
            Self { 
                txid: PointTxId::from_str(&name.join()),
                name: name.clone(),
                send: loc_send, recv: stack,
                subscribers: Arc::new(FxDashMap::default()),
                receivers: Arc::new(AtomicUsize::new(0)),
                receivers_tx,
                receivers_rx: receivers_rx_stack,
                timeout: Self::DEFAULT_TIMEOUT,
                exit: Arc::new(AtomicBool::new(false)),
            },
            remote,
        )
    }
    ///
    /// Returns connected `Link`
    pub async fn link(&self) -> Link {
        let (loc_send, rem_recv) = mpsc::channel();
        let (rem_send, loc_recv) = mpsc::channel();
        let remote = Link::new(&format!("{}:{}", self.name, self.subscribers.len()), rem_send, rem_recv);
        let key = remote.name().join();
        self.subscribers.insert(key.clone(), loc_send);
        let receivers = self.receivers.clone();
        let len = receivers.load(Ordering::SeqCst);
        self.receivers_tx.send((key, loc_recv)).unwrap();
        let _ = tokio::task::spawn_blocking(async move || {
            while len == receivers.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(3)).await;
            }
        }).await;
        remote
    }
    ///
    /// Entry point
    pub async fn run(&self) -> Result<JoinSet<()>, StrErr> {
        let dbg = self.name.join();
        log::info!("{}.run | Remote | Starting...", dbg);
        let subscribers = self.subscribers.clone();
        let exit = self.exit.clone();
        let recv = self.recv.pop().unwrap();
        let timeout = self.timeout;
        let mut join_set = JoinSet::new();
        join_set.spawn(async move {
            tokio::task::spawn_blocking(move|| {
                log::debug!("{}.run | Remote | Start", dbg);
                'main: loop {
                    log::trace!("{}.run | Locals | Subscriber: {}", dbg, subscribers.len());
                    match recv.recv_timeout(timeout) {
                        Ok(event) => {
                            log::trace!("{}.run | Request: {:?}", dbg, event);
                            match event.cot() {
                                Cot::Inf | Cot::Act | Cot::Req => {
                                    for item in subscribers.iter() {
                                        let (_key, subscriber) = item.pair();
                                        if let Err(err) = subscriber.send(event.clone()) {
                                            log::warn!("{}.run | Send error: {:?}", dbg, err);
                                        }
                                    }
                                }
                                Cot::ReqCon | Cot::ReqErr => {
                                    let key = event.name();
                                    match subscribers.get(&key) {
                                        Some(subscriber) => {
                                            if let Err(err) = subscriber.send(event.clone()) {
                                                log::warn!("{}.run | Send error: {:?}", dbg, err);
                                            }
                                        },
                                        None => {
                                            log::warn!("{}.run | Subscriber not found: {:?}", dbg, key);
                                        },
                                    }
                                }
                                _ => log::warn!("{}.run | Uncnown message received: {:?}", dbg, event),
                            }
                        },
                        Err(err) => match err {
                            std::sync::mpsc::RecvTimeoutError::Timeout => {
                                log::trace!("{}.run | Remote | Listening...", dbg);
                            },
                            std::sync::mpsc::RecvTimeoutError::Disconnected => {
                                if log::max_level() >= log::LevelFilter::Trace {
                                    log::warn!("{}.run | Receive error, all receivers has been closed", dbg);
                                }
                            }
                        },
                    }
                    if exit.load(Ordering::SeqCst) {
                        break 'main;
                    }
                }
                log::info!("{}.run | Remote | Exit", dbg);
            });
        });
        let dbg = self.name.join();
        log::info!("{}.run | Remote | Starting - Ok", dbg);
        log::info!("{}.run | Locals | Starting...", dbg);
        let send = self.send.clone();
        let timeout = self.timeout;
        let interval = self.timeout;    //Duration::from_millis(1000);
        let self_receivers = self.receivers.clone();
        let receivers_rx = self.receivers_rx.pop().unwrap();
        let exit = self.exit.clone();
        join_set.spawn(async move {
            tokio::task::spawn_blocking(move|| {
                log::debug!("{}.run | Locals | Start", dbg);
                let mut receivers = FxIndexMap::default();
                'main: loop {
                    for (key, receiver) in receivers_rx.try_iter() {
                        receivers.insert(key, receiver);
                        self_receivers.fetch_add(1, Ordering::SeqCst);
                    }
                    log::debug!("{}.run | Locals | Receivers: {}", dbg, receivers.len());
                    let cycle = Instant::now();
                    for (_key, receiver) in &receivers {
                        match receiver.recv_timeout(timeout) {
                            Ok(event) => {
                                log::trace!("{}.run | Received from locals: {:?}", dbg, event);
                                if let Err(err) = send.send(event) {
                                    log::warn!("{}.run | Send error: {:?}", dbg, err);
                                }
                            }
                            Err(err) => match err {
                                mpsc::RecvTimeoutError::Timeout => {
                                    log::trace!("{}.run | Locals | Listening...", dbg);
                                }
                                mpsc::RecvTimeoutError::Disconnected => {
                                    if log::max_level() >= log::LevelFilter::Trace {
                                        log::warn!("{}.run | Receive error, all senders has been closed", dbg);
                                    }
                                }
                            }
                        }
                        if exit.load(Ordering::SeqCst) {
                            break 'main;
                        }
                    }
                    if exit.load(Ordering::SeqCst) {
                        break 'main;
                    }
                    if receivers.len() == 0 {
                        let elapsed = cycle.elapsed();
                        if elapsed < interval {
                            std::thread::sleep(interval - elapsed);
                        }
                    }
                }
                log::info!("{}.run | Locals | Exit", dbg);
            });
        });
        let dbg = self.name.join();
        log::info!("{}.run | Locals | Starting - Ok", dbg);
        Ok(join_set)
    }
    ///
    /// Sends "exit" signal to the service's task
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
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