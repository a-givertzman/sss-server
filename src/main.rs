// #![feature(try_trait_v2)]
mod algorithm;
mod app;
mod conf;
mod infrostructure;
mod kernel;
//mod prelude;
mod ship_model;
#[cfg(test)]
mod tests;

use algorithm::entities::{Position, Position2d};

//use algorithm::eval::*;

use app::app::App;
use conf::conf::Conf;
use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use infrostructure::api::client::api_client::ApiClient;
use kernel::{eval::Eval, run::Run, types::{Arc, RwLock},};
//use prelude::*;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
//use ship_model::ship_model::ShipModel;
use crate::algorithm::entities::{Bounds, Moment, model_cached::{self, BoundDisplacementCache, DisplacementShape, Draught}};
use std::{collections::HashMap, path::PathBuf};
///
/// Application entry point
fn main() -> Result<(), Box<dyn std::error::Error>> {
    //   DebugSession::init(LogLevel::Debug, Backtrace::Short);
        let dbg = Dbg::new("main", "bound_cache");
        let cache_dir: PathBuf = "src/assets/cache/sofia".into();
        let model_dir: PathBuf = "src/assets/model/sofia".into();
        let model_center_coord = Position::new(65.250, 0., 0.);
        let bounds = Bounds::from_n(138.86, model_center_coord.x(), 20).unwrap();
        let bounds_length_mm = (bounds.length()*1000.).ceil() as usize;
        let thread_pool = ThreadPool::new(&dbg, Some(15));
        let displacement_shape = Arc::new(RwLock::new(DisplacementShape::new_uninit(
            &dbg,
            model_dir.clone().join(PathBuf::from("hull.stl")),
            Some(model_center_coord),
            1000.,
        )));
        displacement_shape.write().init().unwrap();
        dbg!("displacement_shape init ok");
        let mut bound_cache = BoundDisplacementCache::new(
            &dbg,
            displacement_shape,
            cache_dir
                .clone()
                .join("disp_bounded")
                .join(format!("{bounds_length_mm}")),
            1.,
            130.5,
            model_center_coord.x(),
            bounds.clone(),
            thread_pool.scheduler(),
        );
        dbg!("BoundCache::new ok");
        let res = bound_cache.rebuild();
        dbg!("BoundCache::new rebuild", res);
        let res = bound_cache.get(5.9, 0.);
        dbg!("BoundCache::new get", res);



/*
   
    let dbg = Dbg::new("ShipModel", "compute_balance");
    //   let model_path = "src/assets/sofia3.stp";
    //    let center_coord = Position::new(65.250, 0., 0.);
    //  let model_path = "src/assets/model_1510.stp";
    //    let model_path = "src/assets/ark-Part3.obj";
    //  let model_path = "src/assets/ark.stl";
    //  let model_center_coord = Position::new(59.195, 0., 0.);
    let cache_dir = "src/assets/cache/sofia".into();
    let model_dir = "src/assets/model/sofia".into();
    let model_center_coord = Position::new(65.250, 0., 0.);
    let thread_pool = ThreadPool::new(&dbg, Some(15));
    let mut model = model_cached::ModelCached::new(
        &dbg,
        model_cached::ModelCachedConf {
            model_dir,
            cache_dir,
            model_scale: 1000.,
            model_center_coord,
            heel_steps: vec![
                -60., -50., -45., -40., -35., -30., -25., -20., -15., -10., -5., -2., 0., 2., 5.,
                10., 15., 20., 25., 30., 35., 40., 45., 50., 60.,
            ],
            trim_steps: vec![
                -40., -30., -25., -20., -15., -12.5, -10., -7.5, -5., -3., -2., -1., 0., 1., 2., 3., 5., 7.5, 10., 12.5, 20., 25., 30., 40.,
            ],
            ship_length_lbp: 130.5,
            draught_min: 2.,
            draught_max: 14.,
            hull_draught_step: 1.,
            compartment_qnt_steps: 3,
            compartment_data: HashMap::new(),
        },
        thread_pool.scheduler(),
    )
    .unwrap();
    //  let res = model.rebuild_caches();   dbg!(&res);

    let mut result = |mass: f64, x: f64, y: f64, z: f64| {
        model.floating_position(model_cached::FloatingPositionQuery {
            water_density: 1.025,
            mass_const: mass,
            moment_const: Moment::from_pos(Position::new(x - model_center_coord.x(), y, z), mass),
            bulk: vec![],
            liquid: vec![],
            grain_bulkhead: Vec::new(),
            damaged_compartment: Vec::new(),
            epsilon: 0.0001,
        })
    };

 //   let data = [ [10317.65, 90., 0., 6.],];
   // let data = [ [10000., 63.371, -0.2, 6.]];

    let data = [
        [14194.5, 63.371, -0.001, 6.605],
        [13163.9, 63.933, 0., 6.212],
        [13987., 65.231, 0., 4.99],
        [14135.3, 64.898, 0., 4.882],
        [13238.467, 65.409, 0., 6.463],
        [10960.742, 66.471, 0.001, 5.810],
        [7212.705, 66.404, 0., 5.391],
        [10000., 63.371, -0.2, 6.],
        [10000., 63.371, 0.4, 6.],
        [10000., 70., 0., 6.],
        [10000., 55., 0., 6.],
        [10000., 70., 0.1, 6.],
    ];


    for [m, x, y, z] in data {
  //      print!("m:{m} x:{x} y:{y} z_fix:{z} result: ");
        result(m, x, y, z).unwrap();
    }
*/

    /*model.floating_position(ship_model::query::BalanceQuery {
        water_density: 1.025,
        mass_const: 14194.500,
        moment_const: Moment::from_pos(Position::new(63.371 - model_center_coord.x(), -0.001, 6.605), 14194.5),
        bulk: vec![],
        liquid: vec![],
        grain_bulkhead: Vec::new(),
        damaged_compartment: vec![],//"212".to_owned()],
        precision: 0.00001,
    }); */
    //    dbg!(result);

    /*
        let query = ship_model::query::BalanceQuery {
            water_density: 1.025,
            mass_const: 10000.,
        //    moment_const: Moment::from_pos(Position::new(1., -0.5, -1.), 5000.),
            moment_const: Moment::from_pos(Position::new(0., 0., 0.), 0.),
            bulk: vec![ship_model::query::BulkData {
                cargo_id: 1,
                space_id: "212".to_owned(),
                mass: 0.,
                volume: 0.,
            }],
            liquid: vec![ship_model::query::LiquidData {
                cargo_id: 2,
                space_id: "212".to_owned(),
                mass: 0.,
                volume: 0.,
            }],
            grain_bulkhead: Vec::new(),
            damaged_compartment: vec![],//"212".to_owned()],
            precision: 0.00001,
        };

        let result = model.floating_position(query);
        dbg!(result);
    */

    /*   let floating_position = model.floating_position(
            3230.55,
            center_mass,
        ).eval().map_err(|err| error.pass_with("floating_position", err))?;
    */
    /*
    let dbg = Dbg::own("main");
    let tmp_dbg = dbg.clone();
    let path = "config.yaml";
    let mut app = App::new(path);
    if let Err(err) = app.run() {
        log::error!("main | Error: {:#?}", err);
    }
    let conf = "./config.yaml";
    let conf = Conf::new(&dbg, conf);
    let ship_id = 2;
    let project_id = "NULL";
    let n_parts = 200;
    let thread_pool = ThreadPool::new(&dbg, Some(conf.thread_pool.size));
    let ship_model = ShipModel::new(
        &dbg,
        ship_id,
        project_id.to_owned(),
        n_parts,
        ApiClient::new(
            &dbg,
            conf.api.address.database.clone(),
            conf.api.address.host.clone(),
            conf.api.address.port.clone(),
        ),
        thread_pool.scheduler(),
    );
    let ship_model_handle = ship_model.run().unwrap();
    log::debug!("main | Calculations...");
    let ctx = CriterionStabilityEval::new(
        &dbg,
        MetacentricHeightSubdivisionEval::new(
            &dbg,
            GrainEval::new(
                &dbg,
                CirculationEval::new(
                    &dbg,
                    AccelerationEval::new(
                        &dbg,
                        MinMetacentricHeightEval::new(
                            &dbg,
                            DSOAngleMaxEval::new(
                                &dbg,
                                DSOTimberMaxEval::new(
                                    &dbg,
                                    DSOIcingMaxEval::new(
                                        &dbg,
                                        DSOMaxEval::new(
                                            &dbg,
                                            DSOAreaEval::new(
                                                &dbg,
                                                StaticAngleEval::new(
                                                    &dbg,
                                                    WheatherEval::new(
                                                        &dbg,
                                                        RollingAmplitudeEval::new(
                                                            &dbg,
                                                            RollingPeriodEval::new(
                                                                &dbg,
                                                                WindEval::new(
                                                                    &dbg,
                                                                    WindageEval::new(
                                                                        &dbg,
                                                                        LeverDiagramEval::new(
                                                                            &dbg,
                                                                            //   link,
                                                                            MetacentricHeightEval::new(
                                                                                &dbg,
                                                                                // Before ZG
                                                                                StabilityAreaEval::new(
                                                                                    &dbg,
                                                                                    ship_model.link(),
                                                                                    BalanceEval::new(
                                                                                        &dbg,
                                                                                        ship_model.link(),
                                                                                        LoadsEval::new(
                                                                                            &dbg,
                                                                                            WettingEval::new(
                                                                                                &dbg,
                                                                                                IcingEval::new(
                                                                                                    &dbg,
                                                                                                    StrengthAreaEval::new(
                                                                                                        &dbg,
                                                                                                        ship_model.link(),
                                                                                                        IcingTimberEval::new(
                                                                                                            &dbg,
                                                                                                            IcingStabEval::new(
                                                                                                                &dbg,
                                                                                                                Initial::new(
                                                                                                                    &dbg,
                                                                                                                    ship_model.link(),
                                                                                                                    ApiClient::new(
                                                                                                                        &dbg,
                                                                                                                        conf.api.address.database.clone(),
                                                                                                                        conf.api.address.host.clone(),
                                                                                                                        conf.api.address.port.clone(),
                                                                                                                    ),
                                                                                                                    Context::new(InitialCtx::new(ship_id, project_id)),
                                                                                                                ),
                                                                                                            ),
                                                                                                        ),
                                                                                                    ),
                                                                                                ),
                                                                                            ),
                                                                                        ),
                                                                                    ),
                                                                                ),
                                                                            ),
                                                                        ),
                                                                    ),
                                                                ),
                                                            ),
                                                        ),
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let _result = DraftMarkEval::new(
        &tmp_dbg,
        CriterionDraughtEval::new(
            &tmp_dbg,
            ReserveBuoyncyEval::new(
                &tmp_dbg,
                ScrewEval::new(
                    &tmp_dbg,
                    BowBoardEval::new(
                        &tmp_dbg,
                        LoadLineEval::new(
                            &tmp_dbg,
                            ZgEval::new(
                                    thread_pool.scheduler(),
                                    &tmp_dbg,
                              //      &ship_model,
                                    ctx,
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
    .eval(());
    ship_model.exit();
    ship_model_handle.join().unwrap();
    */
    Ok(())
}
