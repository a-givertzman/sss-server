// #![feature(try_trait_v2)]
mod algorithm;
mod app;
mod conf;
mod infrostructure;
mod kernel;
mod prelude;
mod server;
#[cfg(test)]
mod tests;

use app::app::App;
use conf::Conf;
use debugging::session::debug_session::{DebugSession, LogLevel};
use infrostructure::ApiClient;
use kernel::{
    run::Run,
    types::{Arc, RwLock},
};
use std::path::PathBuf;
//use prelude::*;
use crate::algorithm::entities::{
    Bounds,
    model_cached::{self, BowAreaCache, DisplacementCache},
};
use crate::{
    algorithm::{
        Calculus,
        entities::{
            model_cached::{DisplacementShape, LocalCache, Shape},
            ship_model::ship_model::ShipModel,
        },
    },
    infrostructure::{DevStream, SelectCalculus, SelectDevDoc, SelectDevInfo},
    server::{
        Content, Cot, DevConf, DevStreamConf, QueryId, SelectAct, SelectContent, SelectCot,
        SelectReq, Server,
    },
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
///
/// Application entry point
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let _log2 = log2::open("log.txt")
    //     .level(Logger::from_default_env().filter().as_str())
    //     .size(5 * 1024 * 1024)
    //     .rotate(10)
    //     .tee(false)
    //     .module(true)
    //     .start();

    DebugSession::new()
        .filter(LogLevel::Trace)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();

    let dbg = Dbg::own("main");
    let path = "config.yaml";
    let mut app = App::new(path);
    if let Err(err) = app.run() {
        log::error!("main | Error: {:#?}", err);
    }
    let conf = "./config.yaml";
    let conf = Conf::new(&dbg, conf);
    let thread_pool = Arc::new(ThreadPool::new(&dbg, Some(conf.thread_pool.size)));

    /*
        let cache_dir: PathBuf = "src/assets/cache/sofia".into();
        let mut bow_area = BowAreaCache::new(
            &dbg,
            None,
            None,
            &cache_dir,
            Arc::clone(&thread_pool),
        );
        dbg!(bow_area.init());
        dbg!(bow_area.get(-0.64, 8.05));
        return Ok(());
    */

    /*
        let cache_dir: PathBuf = "src/assets/cache/sofia".into();
        let model_dir: PathBuf = "src/assets/model/sofia".into();
        let model_x = 65.250;
        let mut displacement_shape =DisplacementShape::new_uninit(
            &dbg,
            model_dir.join(PathBuf::from("hull.stl")),
            Some(model_x),
            1000.,
        );
        dbg!(displacement_shape.init());
        let mut displacement = DisplacementCache::new(
            &dbg,
            Arc::new(RwLock::new(displacement_shape)),
            cache_dir.clone(),
            vec![
                -60., -50., -45., -40., -35., -30., -25., -20., -15., -10., -5., -2., 0., 2., 5., 10.,
                15., 20., 25., 30., 35., 40., 45., 50., 60.,
            ],
            vec![
                -40., -30., -25., -20., -15., -12.5, -10., -7.5, -5., -3., -2., -1., 0., 1., 2., 3.,
                5., 7.5, 10., 12.5, 20., 25., 30., 40.,
            ],
            0.5,
            14.,
            0.5,
            Arc::clone(&thread_pool),
        );
        dbg!(displacement.init());
        let res = displacement.get(40., -0.6, 13600., 0.000001);
        dbg!(res);
        return Ok(());
    */
    /*
      let cache_dir: PathBuf = "src/assets/cache/sofia/compartments".into();
      let model_dir: PathBuf = "src/assets/model/sofia/compartments/306.stl".into();
      let mut shape = Arc::new(RwLock::new(DisplacementShape::new_uninit(
          &dbg, model_dir, None, 1000.,
      )));
      shape.write().init().unwrap();
      //  shape.write().inertia(-40., 0., 1.5158751543224378).unwrap();
      let mut cache = model_cached::CompartmentCache::new(
          &dbg,
          shape.clone(),
          cache_dir,
          "306".to_owned(),
          //     (-60..=60).map(|v| v as f64).collect(),
          //   vec![-5., 0., 5.,],
          //   vec![-40., -20., -10., 0., 10., 20., 40.,],
          //   40,
          vec![
              -80., -70., -60., -40., -30., -20., -15., -10., -5., -2., 0., 2., 5., 10., 15., 20.,
              30., 40., 60., 70., 80.,
          ],
          //    vec![-40., -30., -20., -15., -10., -5., -2., 0., 2., 5., 10., 15., 20., 30., 40.,],
          //    vec![-2., -1., 0., 1., 2.,],
          //    vec![-0.01, 0., 0.01,],
          vec![-40., -20., -10., 0., 10., 20., 40.],
          20,
          Arc::clone(&thread_pool),
      );
    // cache.rebuild().unwrap();
      cache.init().unwrap();
      //  cache.calc_coeff(221.692).unwrap(); //205
      //    cache.calc_coeff(19.034).unwrap(); // 501
      //    cache.calc_coeff(35.146).unwrap(); // 402
      cache.calc_coeff(104.765).unwrap();
      //    cache.calc_coeff(99.7776).unwrap();

      //  cache.get_for_dso(-10., 0., 0., 0.000001, true, false).unwrap();

      dbg!(cache.get(-0.6, -1.85, 102.68041237113401, 0.000001));
      /*
      let calc = |heel: f64| {
          let result = cache.get(heel, 0., 2., 0.000001).unwrap();
          //     println!("{:.1} {:.3} {:.3} {:.3} {:.3};", heel, result.inertia_trans_x, result.max_inertia_trans_x, result.abs_moment, result.max_abs_moment);
          let fix_moment = (result.volume_center.y() * heel.to_radians().cos()
              + result.volume_center.z() * heel.to_radians().sin())
              * result.volume
              * 1.025;
          println!(
              "{:.1} {:.6} {:.6} {:.6} {:.6} {:.6};",
              heel,
              result.volume,
              result.volume_center.y(),
              result.volume_center.z(),
              result.abs_moment * 1.025,
              fix_moment
          ); //result.inertia_trans_x*1.025);
      };

      calc(0.);
      calc(5.);
      calc(10.);
      calc(15.);
      calc(20.);
      calc(25.);
      calc(30.);
      calc(40.);
      calc(50.);
      calc(60.);
      calc(70.);
      calc(80.);*/
      return Ok(());
      */

    let ship_id = 2;
    let project_id = "NULL";
    let cache_dir: PathBuf = "src/assets/cache/sofia".into();
    let model_dir: PathBuf = "src/assets/model/sofia".into();
    let model_x = 65.250;
    let mut dso_angles = vec![-60., -50., -40., -30., -12., 12., 30., 40., 50., 60.];
    dso_angles.append(&mut ((-11..=11).map(|v| (v as f64) * 5.).collect())); // -55, -50 .. 55
    dso_angles.append(&mut ((-8..=8).map(|v| v as f64).collect()));
    dso_angles.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    dso_angles.dedup();
    let mut model_cached = model_cached::ModelCached::new(
        &dbg,
        model_cached::ModelCachedConf {
            model_dir,
            cache_dir,
            model_scale: 1000.,
            model_x,
            hull_heel_steps: vec![
                -60., -50., -45., -40., -35., -30., -25., -20., -15., -10., -5., -2., 0., 2., 5.,
                10., 15., 20., 25., 30., 35., 40., 45., 50., 60.,
            ],
            hull_trim_steps: vec![
                -40., -30., -25., -20., -15., -12.5, -10., -7.5, -5., -3., -2., -1., 0., 1., 2.,
                3., 5., 7.5, 10., 12.5, 20., 25., 30., 40.,
            ],
            compartment_heel_steps: vec![
                -60., -40., -30., -20., -15., -10., -5., -2., 0., 2., 5., 10., 15., 20., 30., 40.,
                60.,
            ],
            compartment_trim_steps: vec![
                -40., -30., -20., -15., -10., -5., -2., 0., 2., 5., 10., 15., 20., 30., 40.,
            ],
            //    compartment_heel_steps: (-60..=60).filter(|v| v%5 == 0).map(|v| v as f64).collect(),
            //    compartment_trim_steps: vec![0., 1.,],
            ship_length_lbp: 130.5,
            draught_min: 2.001,
            hull_draught_min: 0.5,
            hull_draught_max: 14.,
            hull_draught_step: 0.5,
            bounds_level_step: 0.1,
            compartment_level_step_qnt: 20,
            dso_angles,
        },
        Arc::clone(&thread_pool),
    )
    .unwrap();
    let mut dso_angles = vec![-60., -50., -40., -30., -12., 12., 30., 40., 50., 60.];
    dso_angles.append(&mut ((-11..=11).map(|v| (v as f64) * 5.).collect())); // -55, -50 .. 55
    dso_angles.append(&mut ((-8..=8).map(|v| v as f64).collect()));
    dso_angles.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    dso_angles.dedup();
    /*     let res = model_cached.reload_shapes();            dbg!(&res);
    //    let res = model_cached.rebuild_caches();   dbg!(&res);
        let res = model_cached.rebuild_bounds(&bounds);    dbg!(&res);
      //  let res = model_cached.init();                     dbg!(&res);
      //  let res = model_cached.init_bounded(&bounds);      dbg!(&res);
        return Ok(());
    */
    let api_client = Arc::new(ApiClient::new(
        &dbg,
        conf.api.address.database.clone(),
        conf.api.address.host.clone(),
        conf.api.address.port.clone(),
    ));
    let mut ship_model = ShipModel::new(
        &dbg,
        ship_id,
        project_id.to_owned(),
        model_cached,
        api_client.clone(),
    );
    ship_model.init().unwrap();
    let ship_model = Arc::new(RwLock::new(ship_model));
    let server = Server::new(
        &dbg,
        conf.clone(),
        thread_pool.scheduler(),
        move |dbg, conf| {
            Box::new(
                //
                // Select handler for incomong messages by Content
                SelectContent::new(vec![
                    // Handler for Content::Bytes
                    (
                        Content::Bytes,
                        Box::new(SelectCot::new(vec![
                            // Handling incomong messages with `Cot::Act` by field `cmd`
                            (
                                Cot::Act,
                                Box::new(SelectAct::new(vec![
                                    // Handling incomong commands
                                    // ...
                                ])),
                            ),
                            // Handling incomong messages with Cot::Req by field `req`
                            (
                                Cot::Req,
                                Box::new(SelectReq::new(vec![
                                    // Handling incomong requests
                                    // ...
                                ])),
                            ),
                        ])),
                    ),
                    // Handler for Content::Empty
                    (
                        Content::Empty,
                        Box::new(SelectCot::new(vec![
                            // Handling incomong messages with `Cot::Act` by field `cmd`
                            (
                                Cot::Act,
                                Box::new(SelectAct::new(vec![
                                    // Handling incomong commands
                                    // ...
                                ])),
                            ),
                            // Handling incomong messages with Cot::Req by field `req`
                            (
                                Cot::Req,
                                Box::new(SelectReq::new(vec![
                                    // Handling incomong requests
                                    // ...
                                ])),
                            ),
                        ])),
                    ),
                    // Handler for Content::Json
                    (
                        Content::Json,
                        Box::new(SelectCot::new(vec![
                            // Handling incomong messages with `Cot::Act` by field `cmd`
                            (
                                Cot::Act,
                                Box::new(SelectAct::new(vec![
                                    // Handling incomong command `DeviceStream`
                                    (
                                        QueryId::Calculus,
                                        Box::new(SelectCalculus::new(
                                            dbg,
                                            conf.calculus.clone(),
                                            thread_pool.scheduler(),
                                            Calculus::new(
                                                dbg,
                                                conf.clone(),
                                                api_client.clone(),
                                                ship_model.clone(),
                                            ),
                                        )),
                                    ),
                                    // Handling incomong command `DeviceStream`
                                    (
                                        QueryId::DeviceStream,
                                        Box::new(DevStream::new(
                                            dbg,
                                            DevStreamConf {
                                                devices: vec![
                                                    ("Dev1".into(), DevConf {}),
                                                    ("Dev2".into(), DevConf {}),
                                                ],
                                            },
                                            thread_pool.scheduler(),
                                        )),
                                    ),
                                ])),
                            ),
                            // Handling incomong messages with Cot::Req by field `req`
                            (
                                Cot::Req,
                                Box::new(SelectReq::new(vec![
                                    // Handling incomong request `DeviceInfo`
                                    (
                                        QueryId::DeviceInfo,
                                        Box::new(SelectDevInfo::new("assets/info/")),
                                    ),
                                    // Handling incomong request `DeviceDoc`
                                    (
                                        QueryId::DeviceDoc,
                                        Box::new(SelectDevDoc::new("assets/info/")),
                                    ),
                                ])),
                            ),
                        ])),
                    ),
                ]),
            )
        },
    );
    server.run().map_err(|err| Error::new(&dbg, "").pass(err))?;
    server.wait().map_err(|err| Error::new(dbg, "").pass(err))?;
    Ok(())
}
