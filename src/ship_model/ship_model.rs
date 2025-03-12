use std::{fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc::{self, Receiver, Sender}, Arc}, time::{Duration, Instant}};
use coco::Stack;
use sal_sync::services::entity::{error::str_err::StrErr, name::Name, point::point_tx_id::PointTxId};
use tokio::task::JoinHandle;
use crate::{algorithm::entities::{area::HAreaStrength, data::strength::VerticalArea, Bound}, infrostructure::api::client::api_client::ApiClient, kernel::types::fx_map::FxIndexMap, prelude::Error};
use super::{model_link::ModelLink, query::Query, reply::Reply, temp_data::area_v_str};
use super::temp_data::*;
///
/// 
pub struct ShipModel {
    txid: usize,
    name: Name,
    ship_id: usize,
    // bounds: Bounds,
    clients_tx: Sender<(String, Sender<Reply>, Receiver<Query>)>,
    clients_rx: Stack<Receiver<(String, Sender<Reply>, Receiver<Query>)>>,
    clients: Arc<AtomicUsize>,
    timeout: Duration,
    api_client: Stack<ApiClient>,
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
    pub fn new(parent: impl Into<String>, ship_id: usize, api_client: ApiClient) -> Self {
        let name = Name::new(parent, "ShipModel");
        let (receivers_tx, receivers_rx) = mpsc::channel();
        let receivers_rx_stack = Stack::new();
        receivers_rx_stack.push(receivers_rx);
        let client = Stack::new();
        client.push(api_client);
        Self {
            txid: PointTxId::from_str(&name.join()),
            name,
            ship_id,
            clients: Arc::new(AtomicUsize::new(0)),
            clients_tx: receivers_tx,
            clients_rx: receivers_rx_stack,
            timeout: Self::DEFAULT_TIMEOUT,
            api_client: client,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns connected `Link`
    pub async fn link(&self) -> ModelLink {
        let (loc_send, rem_recv) = mpsc::channel();
        let (rem_send, loc_recv) = mpsc::channel();
        let receivers = self.clients.clone();
        let remote = ModelLink::new(&format!("{}:{}", self.name, receivers.load(Ordering::SeqCst)), rem_send, rem_recv);
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
    pub async fn run(&self) -> Result<JoinHandle<()>, StrErr> {
        let dbg = self.name.join();
        log::info!("{}.run | Starting...", dbg);
        let timeout = self.timeout;
        let interval = self.timeout;    //Duration::from_millis(1000);
        let self_receivers = self.clients.clone();
        let clients_rx = self.clients_rx.pop().unwrap();
        let ship_id = self.ship_id;
        let api_client = self.api_client.pop().unwrap();
        let exit = self.exit.clone();
        let handle = tokio::task::spawn_blocking(move|| {
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
                                    let result = areas_strength(&api_client, ship_id);
                                    if let Err(err) = send.send(Reply::AreasStrength(result)) {
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
        let dbg = self.name.join();
        log::info!("{}.run | Starting - Ok", dbg);
        Ok(handle)
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

fn areas_strength(api_client: &ApiClient, ship_id: usize) -> Result<(Vec<VerticalArea>, Vec<HAreaStrength>), StrErr> {
 /*   let area_h_str = HStrAreaArray::parse(
        &api_client
            .fetch(&format!(
        "SELECT name, value, bound_x1, bound_x2 FROM horizontal_area_strength WHERE ship_id={};",
        ship_id
    ))
            .map_err(|e| StrErr(format!("api_server get_data area_h_str error: {e}")))?,
    )
    .map_err(|e| StrErr(format!("api_server get_data area_h_str error: {e}")))?;
    let area_v_str = strength::VerticalAreaArray::parse(
        &api_client
            .fetch(&format!(
        "SELECT name, value, bound_x1, bound_x2 FROM vertical_area_strength WHERE ship_id={};",
        ship_id
    ))
            .map_err(|e| StrErr(format!("api_server get_data area_v_str error: {e}")))?,
    )
    .map_err(|e| StrErr(format!("api_server get_data area_v_str error: {e}")))?;
*/
    let area_h_str: Result<Vec<HAreaStrength>, Error> = area_h_str::area_h_str().data().into_iter().map(|v| {
        match Bound::new(v.bound_x1, v.bound_x2) {
            Ok(bound) => Ok(HAreaStrength::new(
                v.value,
                bound,
            )),
            Err(err) => Err(err),
        }
    }).collect();
    let area_h_str = area_h_str
        .map_err(|e|StrErr(format!("api_server get_data area_v_str error: {e}")))?;

    Ok((area_v_str::area_v_str().data(), area_h_str))
}