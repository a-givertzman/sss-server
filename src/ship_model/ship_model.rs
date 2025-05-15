use super::query::*;
use super::reply::*;
use super::{query::Query, reply::Reply};
use crate::algorithm::entities::data::ComputedFrameDataArray;
use crate::algorithm::entities::data::HStrAreaArray;
use crate::algorithm::entities::data::PhysicalFrameArray;
use crate::algorithm::entities::data::serde_parser::IFromJson;
use crate::algorithm::entities::data::strength;
use crate::algorithm::entities::{Bound, Bounds};
use crate::algorithm::entities::model;
use crate::algorithm::eval::BalanceCtx;
use crate::infrostructure::api::client::api_client::ApiClient;
use crate::kernel::sync::Hub;
use crate::kernel::sync::Link;
use coco::Stack;
use sal_core::dbg::Dbg;
use sal_core::error::Error;
use sal_sync::services::entity::Name;
use sal_sync::services::entity::PointTxId;
use sal_sync::thread_pool::Scheduler;
use std::path::PathBuf;
use std::thread::JoinHandle;
use std::{
    fmt::Debug,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
///
///
pub struct ShipModel {
    txid: usize,
    name: Name,
    ship_id: usize,
    project_id: String,
    n_parts: usize,
    hub: Hub,
    scheduler: Stack<Scheduler>,
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
        scheduler: Scheduler,
    ) -> Self {
        let name = Name::new(parent, "ShipModel");
        let hub = Hub::new(&name);
        let sheduler_stk = Stack::new();
        sheduler_stk.push(scheduler);
        let client = Stack::new();
        client.push(api_client);
        Self {
            txid: PointTxId::from_str(&name.join()),
            name,
            ship_id,
            project_id,
            n_parts,
            hub,
            timeout: Self::DEFAULT_TIMEOUT,
            api_client: client,
            scheduler: sheduler_stk,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns connected `Link`
    pub fn link(&self) -> Link {
        self.hub.link()
    }
    ///
    /// Entry point
    pub fn run(&self) -> Result<JoinHandle<()>, Error> {
        let dbg = self.name.join();
        let error = Error::new(&dbg, "run");
        log::info!("{}.run | Starting...", dbg);
        // let timeout = self.timeout;
        // let interval = self.timeout; //Duration::from_millis(1000);
        let api_client = self.api_client.pop().unwrap();
        let exit = self.exit.clone();
        let ship_id = self.ship_id;
        // TODO read key by ship_id
        let model_key = "/cube_1_1_1_centered";
        // TODO read path by ship_id
        let model_path = "src/assets/cube_1_1_1.step";
        let project_id = self.project_id.clone();
        let n_parts = self.n_parts;
        let cache_dir = "src/assets/cashe/";
        let scheduler = self.scheduler.pop().unwrap();
        let bounds = match get_bounds(&api_client, ship_id, project_id, n_parts) {
            Ok(data) => data,
            Err(err) => return Err(error.pass_with("get_bounds error", err)),
        };
        let handle = self.hub.listen(move |query, send| {
            let error = Error::new(&dbg, "hub.listen");
            log::trace!("{}.run | Received query: {:?}", dbg, query);
            match query {
                Query::Bounds => {
                    if let Err(err) = send.send(Reply::Bounds(bounds.clone())) {
                        log::warn!("{}.run | Send error: {:?}", dbg, err);
                    }
                }
                Query::BoundAreas => {
                    let bounds = bounds.clone();
                    let api_client = api_client.clone();
                    let exit = exit.clone();
                    if let Err(err) = scheduler.spawn(move || {
                        let result = bound_areas(bounds, ship_id, &api_client, exit);
                        if let Err(err) = send.send(Reply::BoundAreas(result)) {
                            let err = error.pass_with("Send error", err);
                            log::warn!("{}", err);
                        }
                        Ok(())
                    }) {
                        log::warn!("{}.run | Schedule error: {:?}", dbg, err);
                    }
                }
                Query::ComputeBalance(balance_src_data) => {
                    let bounds = bounds.clone();
                    let exit = exit.clone();
                    if let Err(err) = scheduler.spawn(move || {
                        let result = compute_balance(
                            model_key,
                            model_path,
                            cache_dir,
                            bounds.clone(),
                            balance_src_data,
                            ship_id,
                            exit,
                        );
                        if let Err(err) = send.send(Reply::ComputeBalance(result)) {
                            let err = error.pass_with("Send error", err);
                            log::warn!("{}", err);
                        };
                        Ok(())
                    }) {
                        log::warn!("{}.run | Schedule error: {:?}", dbg, err);
                    }
                } //          Query::StabilityAreas => todo!(),
                  //          Query::ComputePantocaren => todo!(),
            };
            None::<()>
        });
        let dbg = self.name.join();
        log::info!("{}.run | Starting - Ok", dbg);
        handle.map_err(|err| error.pass(err.to_string()))
    }
    ///
    /// Sends "exit" signal to the service's task
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
        self.hub.exit();
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
/// Type doc comment
fn get_bounds(
    api_client: &ApiClient,
    ship_id: usize,
    project_id: String,
    n_parts: usize,
) -> Result<Bounds, Error> {
    let error = Error::new("ShipModel", "get_bounds");
    let data = api_client.fetch(&format!(
            "SELECT index, start_x, end_x FROM computed_frame_space WHERE n_parts = {n_parts} AND ship_id={ship_id} AND project_id IS NOT DISTINCT FROM {project_id} ORDER BY index ASC;"
        ));
    let bounds = match data {
        Ok(data) => {
            // TODO: если шпации для разбиения на n_parts есть, значит есть кэш для этого разбиения - читаем их
            match ComputedFrameDataArray::parse(&data) {
                Ok(data) => data.data(),
                Err(err) => return Err(error.pass_with("parse error", err)),
            }
        }
        Err(err1) => {
            // TODO: если шпаций для n_parts нет, значит создаем такое разбиение и считаем кэш
            let data = api_client.fetch(&format!(
                "SELECT pos_x, frame_index as index FROM physical_frame WHERE ship_id={ship_id} AND project_id IS NOT DISTINCT FROM {project_id} ORDER BY index ASC;"
            ));
            match data {
                Ok(data) => {
                    let physical_frames = match PhysicalFrameArray::parse(&data) {
                        Ok(data) => data.data(),
                        Err(err2) => {
                            return Err(error.pass(format!(
                                "error: {err1}, physical_frames parse error: {err2}"
                            )));
                        }
                    };
                    if let (Some(bow_x), Some(stern_x)) =
                        (physical_frames.first(), physical_frames.last())
                    {
                        return match Bounds::from_min_max(stern_x.1, bow_x.1, n_parts) {
                            Ok(bounds) => Ok(bounds),
                            Err(err2) => {
                                return Err(error
                                    .pass(format!("error: {err1}, create_bounds error: {err2}")));
                            }
                        };
                    } else {
                        return Err(error.pass(format!("error: {err1}, can't get bow_x, stern_x!")));
                    };
                }
                Err(err2) => {
                    return Err(error.pass(format!("error: {err1}, physical_frames error: {err2}")));
                }
            };
        }
    };
    let bounds: Bounds = match Bounds::from_frames(&bounds) {
        Ok(data) => data,
        Err(err) => return Err(error.pass(err)),
    };
    Ok(bounds)
}
///
/// Computes ...
/// - `exit` - used to breake long havy computation if possible
fn bound_areas(
    bounds: Bounds,
    ship_id: usize,
    api_client: &ApiClient,
    _: Arc<AtomicBool>,
) -> Result<BoundArea, Error> {
    let err = Error::new("ShipModel", "bound_areas");
    let area_h_str = HStrAreaArray::parse(
        &api_client.fetch(&format!(
            "SELECT name, value, bound_x1, bound_x2 FROM horizontal_area_strength WHERE ship_id={} ORDER BY bound_x1 ASC;",
            ship_id
        )).map_err(|e| err.pass(e.to_string()))?
    ).map_err(|e| err.pass(e.to_string()))?;
    let area_v_str = strength::VerticalAreaArray::parse(
        &api_client.fetch(&format!(
            "SELECT name, value, bound_x1, bound_x2 FROM vertical_area_strength WHERE ship_id={} ORDER BY bound_x1 ASC;",
            ship_id
        )).map_err(|e| err.pass(e.to_string()))?
    ).map_err(|e| err.pass(e.to_string()))?;
    let area_h_str: Vec<_> = area_h_str
        .data()
        .into_iter()
        .map(|v| (v.value, Bound::new(v.bound_x1, v.bound_x2).unwrap()))
        .collect();
    let area_v_str: Vec<_> = area_v_str
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
    Ok(BoundArea {
        v: area_v_str,
        h: area_h_str,
    })
}
///
/// Computes ...
/// - `exit` - used to breake long havy computation if possible
fn compute_balance(
    model_key: &str,
    model_path: &str,
    cache_dir: &str,
    bounds: Bounds,
    src_data: BalanceQuery,
    ship_id: usize,
    exit: Arc<AtomicBool>,
) -> Result<BalanceCtx, Error> {
    let dbg = Dbg::new("ShipModel", "compute_balance");
    let error = Error::new(&dbg, "compute_balance");
    let model: model::ShipModel<()> = model::ShipModel::new(
        &dbg, 
        model::ship_model_conf::ShipModelConf {
            model_path: PathBuf::from(model_path),
            cache_dir: PathBuf::from(cache_dir),
            floating_position_cache_conf: model::local_cache::floating_position_cache::floating_position_cache_conf::FloatingPositionCacheConf {
                waterline_position: [0., 0., 0.],
                heel_steps: (-10..=10).step_by(5).map(|n| n as f64).collect(),
                trim_steps: (-10..=10).step_by(5).map(|n| n as f64).collect(),
                draught_steps: vec![0.0, 0.25],
            },
        }
    );
    let floating_position = model.floating_position(
        src_data.mass_sum/src_data.water_density,
        src_data.mass_shift.values(),
        0.1,
    ).eval().map_err(|err| error.pass_with("floating_position", err))?;

    let result = BalanceCtx {
        trim: floating_position.trim_angle,
        roll: floating_position.heel_angle,
        draught_mid: floating_position.draught_at_amidships,
        bulk: todo!(),
        liquid: todo!(),
        bounds_volume: todo!(),
        volume: floating_position.displacement_volume,
        area_wl: todo!(),
        length_wl: todo!(),
        breadth_wl: todo!(),
        volume_shift_z: floating_position.displacement_volume_center[2],
        entry_angle: todo!(),
        flooding_angle: todo!(),
        bow_area: todo!(),
        const_area_v: todo!(),
        const_area_h: todo!(),
        rad_long: todo!(),
        rad_trans: todo!(),
        pantocaren: todo!(),
    };
    Ok(result)
    /*
    let mut cache = Cache::new(&dbg, cashe_path);
    if !cache.init().is_ok() {
        // нет кэша, считаем модель
        // create model tree with empty attribute for each model
        let model_tree = ModelTree::<()>::new(&dbg, model_path)
            .load()
            .map_err(|err| error.pass_with("model_tree", err))?;
        let waterline_position = model_tree
            .get(model_key)
            .and_then(|shape| match shape {
                Shape::Solid(model) => Some(model.center().point()),
                _ => None,
            })
            .map_err(|err| {
                error.err(format!(
                    "model_tree Expected Solid by model_key='{model_key}'"
                ))
            })?;
        let heel_steps = conf.heel_steps.clone();
        let trim_steps = conf.trim_steps.clone();
        let draught_steps = conf.draught_steps.clone();
        let waterline = FloatingPositionCache::new(&dbg, model_tree, result_path, conf)
            .create_waterline()
            .map_err(|err| error.pass_with("waterline", err))?;
        let handlers = CalculatedFloatingPositionCache::new(
            &dbg,
            cashe_path.into(),
            model_tree.iter().map(|(_, shape)| shape).cloned().collect(),
            waterline,
            heel_steps,
            trim_steps,
            draught_steps,
            Arc::default(),
        )
        .build()
        .map_err(|err| error.pass_with("handlers", err))?;
        for (_, handler) in handlers {
            match handler.join() {
                Err(why) => Err(format!("Failed preparing thread: {:?}", why)),
                Ok(res) => {
                    if let Err(why) = res {
                        Err(format!("Failed executing thread: {:?}", why))
                    } else {
                        Ok(())
                    }
                }
            }
            .map_err(|err| error.pass_with("handler.join", err))?;
        }
        // read result file
        /*      let mut result_reader = {
            let result_file = File::open(result_path)
                .map_err(|err| panic!("Failed opening result file='{}': {}", cashe_path, err))?;
            BufReader::new(result_file)
            };
        */
        cache
            .init()
            .map_err(|err| error.pass_with("cache.init", err))?;
    }
*/
}
