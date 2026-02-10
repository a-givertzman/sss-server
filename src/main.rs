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
use std::{collections::HashMap, path::PathBuf};
//use prelude::*;
use crate::algorithm::entities::{
    Bounds,
    model_cached::{self, BowAreaCache, DisplacementCache},
};
use crate::{
    algorithm::{
        Calculus,
        entities::ship_model::ship_model::ShipModel,
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
       let model_dir: PathBuf = "src/assets/model/sofia/compartments/201.stl".into();
       let mut shape = Arc::new(RwLock::new(DisplacementShape::new_uninit(
           &dbg, model_dir, None, 1000.,
       )));
       shape.write().init().unwrap();
       //  shape.write().inertia(-40., 0., 1.5158751543224378).unwrap();
       let mut cache = model_cached::CompartmentCache::new(
           &dbg,
           shape.clone(),
           cache_dir,
           "201".to_owned(),
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
       cache.calc_coeff(168.4).unwrap();
       //    cache.calc_coeff(99.7776).unwrap();

       //  cache.get_for_dso(-10., 0., 0., 0.000001, true, false).unwrap();

       dbg!(cache.get(-15.0, 3., 135., 0.000001));
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

    let ship_id = "2";
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
                -60., -50., -45., -40., -35., -30., -25., -20., -15., -10., -5., -2., -1., -0.5,
                -0.2, 0., 0.2, 0.5, 1., 2., 5., 10., 15., 20., 25., 30., 35., 40., 45., 50., 60.,
            ],
            hull_trim_steps: vec![
                -40., -30., -25., -20., -15., -12.5, -10., -7.5, -5., -3., -2., -1., -0.5, -0.2,
                0., 0.2, 0.5, 1., 2., 3., 5., 7.5, 10., 12.5, 20., 25., 30., 40.,
            ],
            compartment_heel_steps: vec![
                -60., -30., -15., -10., -5., -2., 0., 2., 5., 10., 15., 30., 60.,
            ],
            compartment_trim_steps: vec![-40., -20., -10., -5., -2., 0., 2., 5., 10., 20., 40.],
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

    let api_client = Arc::new(ApiClient::new(
        &dbg,
        conf.api.address.database.clone(),
        conf.api.address.host.clone(),
        conf.api.address.port.clone(),
    ));
  /*  let physical_frames = [
        -3.6, -3.0, -2.4, -1.8, -1.2, -0.6, 0.0, 0.6, 1.2, 1.8, 2.4, 3.0, 3.6, 4.2, 4.8, 5.4, 6.0,
        6.7, 7.4, 8.1, 8.8, 9.5, 10.2, 10.9, 11.6, 12.3, 13.0, 13.7, 14.4, 15.1, 15.8, 16.5, 17.2,
        17.9, 18.6, 19.34, 20.08, 20.82, 21.56, 22.3, 23.04, 23.78, 24.52, 25.26, 26.0, 26.74,
        27.48, 28.22, 28.96, 29.7, 30.44, 31.18, 31.92, 32.66, 33.4, 34.14, 34.88, 35.62, 36.36,
        37.1, 37.84, 38.58, 39.32, 40.06, 40.80, 41.54, 42.28, 43.02, 43.76, 44.5, 45.24, 45.98,
        46.72, 47.46, 48.2, 48.94, 49.68, 50.42, 51.16, 51.9, 52.64, 53.38, 54.12, 54.86, 55.6,
        56.34, 57.08, 57.82, 58.56, 59.30, 60.04, 60.78, 61.52, 62.26, 63.0, 63.74, 64.48, 65.22,
        65.96, 66.7, 67.44, 68.18, 68.92, 69.66, 70.4, 71.14, 71.88, 72.62, 73.36, 74.1, 74.84,
        75.58, 76.32, 77.06, 77.8, 78.54, 79.28, 80.02, 80.76, 81.5, 82.24, 82.98, 83.72, 84.46,
        85.2, 85.94, 86.68, 87.42, 88.16, 88.9, 89.64, 90.38, 91.12, 91.86, 92.6, 93.34, 94.08,
        94.82, 95.56, 96.3, 97.04, 97.78, 98.52, 99.26, 100.0, 100.74, 101.48, 102.22, 102.96,
        103.7, 104.44, 105.18, 105.92, 106.66, 107.4, 108.14, 108.88, 109.62, 110.36, 111.1,
        111.84, 112.58, 113.32, 114.06, 114.8, 115.54, 116.28, 117.02, 117.76, 118.5, 119.24,
        119.98, 120.72, 121.46, 122.2, 122.94, 123.68, 124.42, 125.16, 125.9, 126.5, 127.1, 127.7,
        128.3, 128.9, 129.5, 130.1, 130.7, 131.3, 131.9, 132.5, 133.1, 133.7, 134.3, 134.9, 135.5,
    ];
    let bounds = Bounds::from_array(&physical_frames, 0.).unwrap();
    let mut compartments_volume_max = HashMap::new();
    compartments_volume_max.insert("H104".to_owned(), 2560.3);
    compartments_volume_max.insert("H206".to_owned(), 719.3);
    compartments_volume_max.insert("SP101".to_owned(), 94.35);
    compartments_volume_max.insert("SP102".to_owned(), 94.35);
    compartments_volume_max.insert("SP201".to_owned(), 94.35);
    compartments_volume_max.insert("SP202".to_owned(), 94.35);
    let res = model_cached.reload_shapes();
    dbg!(&res);
    let res = model_cached.init(compartments_volume_max, &bounds);
    dbg!(&res);
    let res = model_cached.rebuild_caches();
    dbg!(&res);
    let res = model_cached.rebuild_bounds(&bounds);
    dbg!(&res);
    //  let res = model_cached.init();                     dbg!(&res);
    //  let res = model_cached.init_bounded(&bounds);      dbg!(&res);
    return Ok(());
*/
    let ship_model = ShipModel::new(
        &dbg,
        ship_id.to_owned(),
        project_id.to_owned(),
        model_cached,
        Arc::clone(&api_client),
    );
    let ship_model = Arc::new(RwLock::new(ship_model));
    ship_model.write().init().unwrap();
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
                                                Arc::clone(&api_client),
                                                Arc::clone(&ship_model),
                                                Arc::clone(&thread_pool),
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
