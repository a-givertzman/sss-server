use crate::algorithm::entities::model_cached::ModelCached;
use crate::algorithm::entities::Position;
use crate::algorithm::entities::Position2d;
use crate::algorithm::entities::data::ComputedFrameDataArray;
use crate::algorithm::entities::data::HStrAreaArray;
use crate::algorithm::entities::data::PhysicalFrameArray;
use crate::algorithm::entities::data::serde_parser::IFromJson;
use crate::algorithm::entities::data::strength;
use crate::algorithm::entities::model_cached;
use crate::algorithm::entities::ship_model::BalanceQuery;
use crate::algorithm::entities::ship_model::BoundArea;
use crate::algorithm::entities::{Bound, Bounds};
use crate::algorithm::eval::BalanceCtx;
use crate::infrostructure::api::client::api_client::ApiClient;
use crate::kernel::sync::Hub;
use crate::kernel::types::RwLock;
use sal_core::dbg::Dbg;
use sal_core::error::Error;
use sal_sync::services::entity::Name;
use sal_sync::services::entity::PointTxId;
use sal_sync::services::future::Future;
use sal_sync::sync::Handles;
use sal_sync::sync::Owner;
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
 //   txid: usize,
 //   name: Name,
    dbg: Dbg, 
    ship_id: usize,
 //   ship_file_name: String, // TODO - read by  ship_id
    project_id: String,
    bound_areas: Option<BoundArea>,
    model_cached: ModelCached,
    scheduler: Scheduler,
//    timeout: Duration,
    api_client: Arc<RwLock<ApiClient>>,
    exit: Arc<AtomicBool>,
}
//
//
impl ShipModel {
    ///
    /// Default timeout to await `recv`` operation, 300 ms
 //   const DEFAULT_TIMEOUT: Duration = Duration::from_millis(10);
    /// TODO
    /// Returns [ShipModel] new instance
    /// - `send` - local side of channel.send
    /// - `recv` - local side of channel.recv
    /// - `exit` - exit signal for `recv_query` method
    pub fn new(
        parent: impl Into<String>,
        ship_id: usize,        
    //    ship_file_name: String,
        project_id: String,
        model_cached: ModelCached,
        api_client: ApiClient,
        scheduler: Scheduler,
    ) -> Self {
    //    let name = Name::new(parent, "ShipModel");
        let dbg = Dbg::new(parent, "ShipModel");
        Self {
        //    txid: PointTxId::from_str(&name.join()),
        //    name,
            dbg,        
            ship_id,
      //      ship_file_name,
            project_id,
            bound_areas: None,
            model_cached,
            scheduler,
        //    timeout: Self::DEFAULT_TIMEOUT,
            api_client: Arc::new(RwLock::new(api_client)),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// TODO: Doc
  /*    pub fn bounds(&mut self, qnt_bounds: usize) -> Result<Bounds, Error> {
        let error = Error::new(&self.dbg, "bounds");

      match &self.bounds {
            Some(bounds) => Ok(bounds.clone()),
            None => match get_bounds(
                &self.api_client.read(),
                self.ship_id,
                self.project_id.clone(),
                qnt_bounds,
            ) {
                Ok(bounds) => {
                    self.bounds = Some(bounds.clone());
                    Ok(bounds)
                },
                Err(_) => {
                    let bounds = self.model_cached.rebuild_bounds(qnt_bounds).map_err(|err| error.pass_with("self.model_cached.rebuild_bounds", err))?;
                    self.bounds = Some(bounds.clone());
                    let sql = format!("INSERT INTO computed_frame_space\n\t(ship_id, qnt_bounds, index, start_x, end_x)\nVALUES{};",
                        bounds.iter().filter(|v| v.is_value()).enumerate().map(|(i, v)| format!("\n\t({}, '{}', {i}, {}, {})", self.ship_id, qnt_bounds, v.start().unwrap(), v.end().unwrap())).collect::<Vec<_>>().join(","));
                    self.api_client.read().fetch(&sql).map_err(|err| error.pass_with("self.api_client.fetch", err))?;         
                    Ok(bounds)
                }
            }
        }
    }
 */   
    ///
    /// TODO: Doc
    pub fn bound_areas(&self, bounds: &Bounds) -> Result<BoundArea, Error> {
        let error = Error::new(&self.dbg, "bound_areas");
        match &self.bound_areas {
            Some(bound_areas) => Ok(bound_areas.clone()),
            None => match self.bound_areas(
                bounds,
                self.ship_id,
                &self.api_client.read(),
                self.exit.clone(),
            ) {
                Ok(bound_areas) => {
                    self.bound_areas = Some(bound_areas.clone());
                    Ok(bound_areas)
                },
                Err(_) => {                    

                    let windage_area = self.model_cached.bounded_windage_area(bounds).map_err(|err| error.pass_with("self.model_cached.rebuild_bounds", err))?;
                    self.bound_areas = Some(bound_areas.clone());
                    let sql = format!("INSERT INTO computed_frame_space\n\t(ship_id, qnt_bounds, index, start_x, end_x)\nVALUES{};",
                        bounds.iter().filter(|v| v.is_value()).enumerate().map(|(i, v)| format!("\n\t({}, '{}', {i}, {}, {})", self.ship_id, qnt_bounds, v.start().unwrap(), v.end().unwrap())).collect::<Vec<_>>().join(","));
                    self.api_client.read().fetch(&sql).map_err(|err| error.pass_with("self.api_client.fetch", err))?;         
                    Ok(bound_areas)
                }
            }
        }
    }
    ///
    /// TODO: Doc
    pub fn compute_balance(&self, query: BalanceQuery) -> Result<BalanceCtx, Error> {
        let error = Error::new(&self.dbg, "compute_balance");
        let result = self.model_cached.balance(query)
            .map_err(|err| error.pass_with("model_cached.balance", err))?;
        
        Ok(BalanceCtx {
            trim: todo!(),
            draught_mid: todo!(),
            roll: todo!(),
            bounds: todo!(),
            bulk: todo!(),
            liquid: todo!(),
            bounds_volume: todo!(),
            volume: todo!(),
            area_wl: todo!(),
            length_wl: todo!(),
            breadth_wl: todo!(),
            volume_shift_z: todo!(),
            entry_angle: todo!(),
            flooding_angle: todo!(),
            bow_area: todo!(),
            const_area_v: todo!(),
            const_area_h: todo!(),
            rad_long: todo!(),
            rad_trans: todo!(),
            pantocaren: todo!(),
        })
    }
    ///
/*    fn bounded_windage_area(
        bounds: Bounds,
        ship_id: usize,
    ) -> Result<BoundArea, Error> {
        let err = Error::new("ShipModel", "bounded_windage_area");
        let area = strength::VerticalAreaArray::parse(
            &api_client.fetch(&format!(
                "SELECT name, value, bound_x1, bound_x2 FROM vertical_area_strength WHERE ship_id={} ORDER BY bound_x1 ASC;",
                ship_id
            )).map_err(|e| err.pass(e.to_string()))?
        ).map_err(|e| err.pass(e.to_string()))?;
        let area: Vec<_> = area
            .data()
            .into_iter()
            .map(|v| (v.value, Bound::new(v.bound_x1, v.bound_x2).unwrap()))
            .collect();
        let area: Vec<f64> = bounds
            .iter()
            .map(|b1| {
                (
                    area.iter().fold(0., |sum, &(v, b2)| 
                        sum + v * b1.part_ratio(&b2).unwrap_or(0.)
                    ),
                )
            })
            .collect();
        Ok(area)
    }*/
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
            .field("ship_id", &self.ship_id)
            .field("project_id", &self.project_id)
            // .field("send", &self.send)
            // .field("recv", &self.recv)
            // .field("subscribers", &self.subscribers)
            // .field("receivers", &self.receivers)
         //   .field("timeout", &self.timeout)
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
    qnt_bounds: usize,
) -> Result<Bounds, Error> {
    let error = Error::new("ShipModel", "get_bounds");
    let data = api_client.fetch(&format!(
            "SELECT index, start_x, end_x FROM computed_frame_space WHERE qnt_bounds = {qnt_bounds} AND ship_id={ship_id} AND project_id IS NOT DISTINCT FROM {project_id} ORDER BY index ASC;"
        ));
    let bounds = match data {
        Ok(data) => {
            // если шпации для разбиения на qnt_bounds есть, значит есть кэш для этого разбиения - читаем их
            match ComputedFrameDataArray::parse(&data) {
                Ok(data) => data.data(),
                Err(err) => return Err(error.pass_with("parse error", err)),
            }
        }
        Err(err) => return Err(err), 
        // если шпаций для qnt_bounds нет, 
        // значит нет кэшей для такого разбиения и надо их пересчитать
    /*    {
           // TODO: если шпаций для qnt_bounds нет, значит создаем такое разбиение и считаем кэш
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
                        return match Bounds::from_min_max(stern_x.1, bow_x.1, qnt_bounds) {
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
        }*/
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
fn bounded_horisontal_area(
    bounds: Bounds,
    ship_id: usize,
    api_client: &ApiClient,
    _: Arc<AtomicBool>,
) -> Result<Vec<f64>, Error> {
    let err = Error::new("ShipModel", "bounded_horisontal_area");
    let area = HStrAreaArray::parse(
        &api_client.fetch(&format!(
            "SELECT name, value, bound_x1, bound_x2 FROM horizontal_area_strength WHERE ship_id={} ORDER BY bound_x1 ASC;",
            ship_id
        )).map_err(|e| err.pass(e.to_string()))?
    ).map_err(|e| err.pass(e.to_string()))?;
    let area: Vec<_> = area
        .data()
        .into_iter()
        .map(|v| (v.value, Bound::new(v.bound_x1, v.bound_x2).unwrap()))
        .collect();
    let area: Vec<f64> = bounds
        .iter()
        .map(|b1| {
                area.iter().fold(0., |sum, &(v, b2)| 
                    sum + v * b1.part_ratio(&b2).unwrap_or(0.)
                )
        })
        .collect();
    Ok(area)
}



