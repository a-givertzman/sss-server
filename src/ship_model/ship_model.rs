use super::query::*;
use super::reply::*;
use super::temp_data::*;
use super::{model_link::ModelLink, query::Query, reply::Reply, temp_data::area_v_str};
use crate::algorithm::entities::data::serde_parser::IFromJson;
use crate::algorithm::entities::data::ComputedFrameDataArray;
use crate::algorithm::entities::data::{strength, HStrAreaArray};
use crate::algorithm::entities::{Bound, Bounds};
use crate::{
    infrostructure::api::client::api_client::ApiClient, kernel::types::fx_map::FxIndexMap,
};
use coco::Stack;
use sal_sync::services::entity::{
    error::str_err::StrErr, name::Name, point::point_tx_id::PointTxId,
};
use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::task::JoinHandle;
///
///
pub struct ShipModel {
    txid: usize,
    name: Name,
    ship_id: usize,
    project_id: String,
    n_parts: usize,
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
    pub fn new(
        parent: impl Into<String>,
        ship_id: usize,
        project_id: String,
        n_parts: usize,
        api_client: ApiClient,
    ) -> Self {
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
            project_id,
            n_parts,
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
        let remote = ModelLink::new(
            &format!("{}:{}", self.name, receivers.load(Ordering::SeqCst)),
            rem_send,
            rem_recv,
        );
        let key = remote.name().join();
        let len = receivers.load(Ordering::SeqCst);
        self.clients_tx.send((key, loc_send, loc_recv)).unwrap();
        let _ = tokio::task::spawn_blocking(async move || {
            while len == receivers.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(3)).await;
            }
        })
        .await;
        remote
    }
    ///
    /// Entry point
    pub async fn run(&self) -> Result<JoinHandle<()>, StrErr> {
        let dbg = self.name.join();
        log::info!("{}.run | Starting...", dbg);
        let timeout = self.timeout;
        let interval = self.timeout; //Duration::from_millis(1000);
        let self_receivers = self.clients.clone();
        let clients_rx = self.clients_rx.pop().unwrap();
        let ship_id = self.ship_id;
        let api_client = self.api_client.pop().unwrap();
        let exit = self.exit.clone();
        let ship_id = self.ship_id;
        let project_id = self.project_id;
        let n_parts = self.n_parts;
        let bounds = match get_bounds(&api_client, ship_id, project_id, n_parts) {
            Ok(data) => data,
            Err(err) => return Err(StrErr(format!("ShipModel get_bounds error: {err}"))),
        };
        let handle = tokio::task::spawn_blocking(move || {
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
                                Query::Bounds => {
                                    if let Err(err) = send.send(Reply::Bounds(bounds.clone())) {
                                        log::warn!("{}.run | Send error: {:?}", dbg, err);
                                    }
                                }
                                Query::AreasStrength => {
                                    let result = areas_strength(bounds.clone(), ship_id);
                                    if let Err(err) = send.send(Reply::AreasStrength(result)) {
                                        log::warn!("{}.run | Send error: {:?}", dbg, err);
                                    }
                                }
                                Query::ComputeBalance(balance_src_data) => {
                                    let result =
                                        compute_balance(bounds.clone(), balance_src_data, ship_id);
                                    if let Err(err) = send.send(Reply::ComputeBalance(result)) {
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
                                    log::warn!(
                                        "{}.run | Receive error, all senders has been closed",
                                        dbg
                                    );
                                }
                            }
                        },
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
///
fn get_bounds(
    api_client: &ApiClient,
    ship_id: usize,
    project_id: String,
    n_parts: usize,
) -> Result<Bounds, StrErr> {
    let data = api_client.fetch(&format!(
            "SELECT index, start_x, end_x FROM computed_frame_space WHERE n_parts = {n_parts} AND ship_id={ship_id} AND project_id IS NOT DISTINCT FROM {project_id} ORDER BY index ASC;"
        ));
    let bounds = match data {
        Ok(data) => match ComputedFrameDataArray::parse(&data) {
            Ok(data) => data.data(),
            Err(err) => return Err(StrErr(format!("ShipModel get_bounds parse error: {err}"))),
        },
        Err(err1) => match create_bounds(n_parts) {
            Ok(data) => data,
            Err(err2) => {
                return Err(StrErr(format!(
                    "ShipModel get_bounds error: {err1}, create_bounds error: {err2}"
                )))
            }
        },
    };
    let bounds: Bounds = match Bounds::from_frames(&bounds) {
        Ok(data) => data,
        Err(err) => return Err(StrErr(format!("ShipModel get_bounds error: {err}"))),
    };
    Ok(bounds)
}
///
fn create_bounds(n_parts: usize) -> Result<Vec<(f64, f64)>, StrErr> {
    SELECT 
        pos_x
    INTO 
        stern_x
    FROM (SELECT * FROM physical_frame WHERE ship_id = changed_ship_id ORDER BY pos_x ASC LIMIT 1);

    SELECT 
        pos_x
    INTO 
        bow_x
    FROM (SELECT * FROM physical_frame WHERE ship_id = changed_ship_id ORDER BY pos_x DESC LIMIT 1);

    SELECT 
        value
    INTO 
        n_parts
    FROM 
        "ship/ship_general_characteristics" s
    WHERE
        s.ship_id = changed_ship_id
        AND key = 'Number of Parts';

    IF ( bow_x IS NULL ) 
    THEN
        RAISE NOTICE 'update_computed_frame_space no bow_x for ship_id:[%]', changed_ship_id;
        RETURN NEW;
    END IF;

    IF ( stern_x IS NULL ) 
    THEN
        RAISE NOTICE 'update_computed_frame_space no stern_x for ship_id:[%]', changed_ship_id;
        RETURN NEW;
    END IF;

    IF ( n_parts IS NULL OR n_parts <= 0 ) 
    THEN
        RAISE NOTICE 'update_computed_frame_space no n_parts for ship_id:[%]', changed_ship_id;
        RETURN NEW;
    END IF;

    RAISE NOTICE 'update_computed_frame_space calculate frames with n_parts:[%] bow_x:[%] stern_x:[%] for ship_id:[%]', n_parts, bow_x, stern_x, changed_ship_id;

    DELETE FROM 
        computed_frame_space 
    WHERE 
        ship_id = changed_ship_id;

    FOR index in 0..(n_parts-1) LOOP
        INSERT INTO
            computed_frame_space (ship_id, index, start_x, end_x)
        VALUES
            (changed_ship_id, index, stern_x + (bow_x - stern_x)*index/n_parts, stern_x + (bow_x - stern_x)*(index+1)/n_parts);
    END LOOP;
}
///
fn areas_strength(bounds: Bounds, ship_id: usize) -> Result<(Vec<f64>, Vec<f64>), StrErr> {
    /*     let area_h_str = HStrAreaArray::parse(
               &api_client
                   .fetch(&format!(
               "SELECT name, value, bound_x1, bound_x2 FROM horizontal_area_strength WHERE ship_id={} ORDER BY bound_x1 ASC;",
               ship_id
           ))
                   .map_err(|e| StrErr(format!("api_server get_data area_h_str error: {e}")))?,
           )
           .map_err(|e| StrErr(format!("api_server get_data area_h_str error: {e}")))?;
           let area_v_str = strength::VerticalAreaArray::parse(
               &api_client
                   .fetch(&format!(
               "SELECT name, value, bound_x1, bound_x2 FROM vertical_area_strength WHERE ship_id={} ORDER BY bound_x1 ASC;",
               ship_id
           ))
                   .map_err(|e| StrErr(format!("api_server get_data area_v_str error: {e}")))?,
           )
           .map_err(|e| StrErr(format!("api_server get_data area_v_str error: {e}")))?;
    */
    let area_h_str: Vec<_> = area_h_str::area_h_str()
        .data()
        .into_iter()
        .map(|v| (v.value, Bound::new(v.bound_x1, v.bound_x2).unwrap()))
        .collect();
    let area_v_str: Vec<_> = area_v_str::area_v_str()
        .data()
        .into_iter()
        .map(|v| (v.value, Bound::new(v.bound_x1, v.bound_x2).unwrap()))
        .collect();
    let (area_v_str, area_h_str): (Vec<f64>, Vec<f64>) = bounds
        .iter()
        .map(|b1| {
            (
                area_v_str.iter().fold(0., |sum, &(v, b2)| {
                    sum + v * b1.part_ratio(&b2).unwrap_or(0.)
                }),
                area_h_str.iter().fold(0., |sum, &(v, b2)| {
                    sum + v * b1.part_ratio(&b2).unwrap_or(0.)
                }),
            )
        })
        .collect();
    Ok((area_v_str, area_h_str))
}
//
fn compute_balance(bounds: Bounds, src_data: BalanceSrcData, ship_id: usize) -> Result<BalanceResultData, StrErr> {
    TODO
}
