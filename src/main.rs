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
use kernel::{eval::Eval, run::Run};
//use prelude::*;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
//use ship_model::ship_model::ShipModel;
use crate::algorithm::entities::{Moment, model_cached};
use std::{collections::HashMap, path::PathBuf};
///
/// Application entry point
fn main() -> Result<(), Box<dyn std::error::Error>> {
    //   DebugSession::init(LogLevel::Debug, Backtrace::Short);

    /*    let dbg = Dbg::new("ShipModel", "compute_balance");
        let model_path = "src/assets/cube_1_1_1.step";

       let cache_dir = "src/assets/cache/";
        let center_coord = Position::new(0., 0., 0.);

        let thread_pool = ThreadPool::new(&dbg, Some(12));
        let mut model: model::ShipModel = model::ShipModel::new(
            &dbg,
            model::ShipModelConf {
                model_path: PathBuf::from(model_path),
                model_scale: 1.,
                cache_dir: PathBuf::from(cache_dir),
                floating_position_cache_conf: model::DisplacementCacheConf {
                    center_coord: center_coord,
          //                  heel_steps: (-10..=10).step_by(5).map(|n| n as f64).collect(),
          //  trim_steps: (-10..=10).step_by(5).map(|n| n as f64).collect(),
          //  draught_steps: vec![0.0, 0.25],
                    heel_steps: vec![-180., -90., -45., 0., 45., 90., 180.], //vec![0.,],//vec![-10., -5., 0., 5., 10.],//(-10..=10).step_by(1).map(|n| n as f64).collect(),
                    trim_steps: vec![-180., -90., -45., 0., 45., 90., 180.],//vec![-5., -2., 0., 2., 5.],//(-8..=8).step_by(1).map(|n| (n as f64)*0.25).collect(),
                    draught_steps: vec![-10., -5., -2.5, 0., 2.5, 5., 10.],//vec![2.5, 2.8, 3., 3.2, 3.3, 3.5, 3.6, 3.8, 3.9, 4.,],vec![2., 3., 4., 5., 6., 7., 8.,],//(8..=16).step_by(1).map(|n| (n as f64)*0.25).collect(),
                },
            },
            thread_pool.scheduler(),
        );
        let res = model.rebuild_caches(&[&CacheKey::FloatingPostion]);
        dbg!(&res);
    */
    let dbg = Dbg::new("ShipModel", "compute_balance");
    //   let model_path = "src/assets/sofia3.stp";
    //    let center_coord = Position::new(65.22, 0., 0.);
    //  let model_path = "src/assets/model_1510.stp";
    //    let model_path = "src/assets/ark-Part3.obj";
    //  let model_path = "src/assets/ark.stl";
    let cache_dir = "src/assets/cache/sofia".into();
    let model_dir = "src/assets/model/sofia".into();
    let model_center_coord = Position::new(59.195, 0., 0.);
    let thread_pool = ThreadPool::new(&dbg, Some(30));
    let mut model = model_cached::ModelCached::new(
        &dbg,
        model_cached::ModelCachedConf {
            model_dir,
            cache_dir,
            model_scale: 1000.,
            model_center_coord,
            heel_steps: vec![-20., 0., 20.],
            trim_steps: vec![-20., 0., 20.],
            draught_min: 2.,
            draught_max: 22.,
            hull_draught_step: 10.,
            compartment_qnt_steps: 3,
            compartment_data: HashMap::new(),
        },
        thread_pool.scheduler(),
    )
    .unwrap();
//    let res = model.rebuild_caches();   dbg!(&res);

    let query = ship_model::query::BalanceQuery {
        water_density: 1.025,
        mass_const: 10000.,
        moment_const: Moment::from_pos(Position::new(1., -0.5, -1.), 5000.),
        bulk: vec![ship_model::query::BulkData {
            cargo_id: 1,
            space_id: "212".to_owned(),
            mass: 50.,
            volume: 50.,
        }],
        liquid: vec![ship_model::query::LiquidData {
            cargo_id: 2,
            space_id: "212".to_owned(),
            mass: 50.,
            volume: 50.,
        }],
        grain_bulkhead: Vec::new(),
        damaged_compartment: vec!["212".to_owned()],
        precision: 0.001,
    };

    let result = model.floating_position(query); 
    dbg!(result);

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
