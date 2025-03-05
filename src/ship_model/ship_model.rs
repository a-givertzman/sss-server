use std::{fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc::{self, Receiver, Sender}, Arc}, time::{Duration, Instant}};
use coco::Stack;
use sal_sync::services::entity::{error::str_err::StrErr, name::Name, point::point_tx_id::PointTxId};
use tokio::task::JoinSet;
use crate::{kernel::types::fx_map::FxIndexMap, ship_model::reply::Reply};

use super::{link::Link, query::Query};
///
/// 
pub struct ShipModel {
    txid: usize,
    name: Name,
    clients_tx: Sender<(String, Sender<Reply>, Receiver<Query>)>,
    clients_rx: Stack<Receiver<(String, Sender<Reply>, Receiver<Query>)>>,
    receivers: Arc<AtomicUsize>,
    timeout: Duration,
    exit: Arc<AtomicBool>,
}
//
//
impl ShipModel {
    ///
    /// Default timeout to await `recv`` operation, 300 ms
    const DEFAULT_TIMEOUT: Duration = Duration::from_millis(10);
    ///
    /// Returns [ShipModel] new instance
    /// - `send` - local side of channel.send
    /// - `recv` - local side of channel.recv
    /// - `exit` - exit signal for `recv_query` method
    pub fn new(parent: impl Into<String>) -> Self {
        let name = Name::new(parent, "ShipModel");
        let (receivers_tx, receivers_rx) = mpsc::channel();
        let receivers_rx_stack = Stack::new();
        receivers_rx_stack.push(receivers_rx);
        Self {
            txid: PointTxId::from_str(&name.join()),
            name,
            receivers: Arc::new(AtomicUsize::new(0)),
            clients_tx: receivers_tx,
            clients_rx: receivers_rx_stack,
            timeout: Self::DEFAULT_TIMEOUT,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns connected `Link`
    pub async fn link(&self) -> Link {
        let (loc_send, rem_recv) = mpsc::channel();
        let (rem_send, loc_recv) = mpsc::channel();
        let receivers = self.receivers.clone();
        let remote = Link::new(&format!("{}:{}", self.name, receivers.load(Ordering::SeqCst)), rem_send, rem_recv);
        let key = remote.name().join();
        let len = receivers.load(Ordering::SeqCst);
        self.clients_tx.send((key, loc_send, loc_recv)).unwrap();
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
        log::info!("{}.run | Starting...", dbg);
        let mut join_set = JoinSet::new();
        let dbg = self.name.join();
        let timeout = self.timeout;
        let interval = self.timeout;    //Duration::from_millis(1000);
        let self_receivers = self.receivers.clone();
        let clients_rx = self.clients_rx.pop().unwrap();
        let exit = self.exit.clone();
        join_set.spawn(async move {
            tokio::task::spawn_blocking(move|| {
                log::debug!("{}.run | Locals | Start", dbg);
                let mut clients = FxIndexMap::default();
                'main: loop {
                    for (key, sender, receiver) in clients_rx.try_iter() {
                        clients.insert(key, (sender, receiver));
                        self_receivers.fetch_add(1, Ordering::SeqCst);
                    }
                    log::debug!("{}.run | Locals | Receivers: {}", dbg, clients.len());
                    let cycle = Instant::now();
                    for (_key, (send, recv)) in &clients {
                        match recv.recv_timeout(timeout) {
                            Ok(query) => {
                                log::trace!("{}.run | Received query: {:?}", dbg, query);
                                match query {
                                    Query::AreasStrength => {
                                        if let Err(err) = send.send(Reply::AreasStrength(Ok(vec![]))) {
                                            log::warn!("{}.run | Send error: {:?}", dbg, err);
                                        }
                                    }
                                }
                            }
                            Err(err) => match err {
                                mpsc::RecvTimeoutError::Timeout => {
                                    log::trace!("{}.run | Listening...", dbg);
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
                    if clients.len() == 0 {
                        let elapsed = cycle.elapsed();
                        if elapsed < interval {
                            std::thread::sleep(interval - elapsed);
                        }
                    }
                }
                log::info!("{}.run | Exit", dbg);
            });
        });
        let dbg = self.name.join();
        log::info!("{}.run | Starting - Ok", dbg);
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
impl Debug for ShipModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShipModel")
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