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

use algorithm::entities::Position;
use app::app::App;
use conf::Conf;
use debugging::session::debug_session::{DebugSession, LogLevel};
use infrostructure::ApiClient;
use kernel::{
    run::Run,
    types::{Arc, RwLock},
};
//use prelude::*;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
use crate::{
    algorithm::{Calculus, entities::ship_model::ship_model::ShipModel},
    infrostructure::{DevStream, SelectCalculus, SelectDevDoc, SelectDevInfo},
    server::{Content, Cot, DevConf, DevStreamConf, QueryId, SelectAct, SelectContent, SelectCot, SelectReq, Server},
};
use crate::algorithm::entities::{
    Bounds, model_cached::{self},
};
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
        .filter(LogLevel::Debug)
        .module("api_tools", LogLevel::Debug)
        .module("sal_sync::thread_pool", LogLevel::Info)
        .init();
    
    let dbg = Dbg::own("main");
    let path = "config.yaml";
    let mut app = App::new(path);
    if let Err(err) = app.run() {
        log::error!("main | Error: {:#?}", err);
    }
    let conf = "./config.yaml";
    let conf = Conf::new(&dbg, conf);
    let ship_id = 2;
    let project_id = "NULL";
    let cache_dir = "src/assets/cache/sofia".into();
    let model_dir = "src/assets/model/sofia".into();
    let model_center_coord = Position::new(65.250, 0., 0.);
    let tp = Arc::new(ThreadPool::new(&dbg, Some(conf.thread_pool.size)));
    let model_cached = model_cached::ModelCached::new(
        &dbg,
        model_cached::ModelCachedConf {
            model_dir,
            cache_dir,
            model_scale: 1000.,
            model_center_coord,
            hull_heel_steps: vec![
                -60., -50., -45., -40., -35., -30., -25., -20., -15., -10., -5., -2., 0., 2., 5.,
                10., 15., 20., 25., 30., 35., 40., 45., 50., 60.,
            ],
            hull_trim_steps: vec![
                -40., -30., -25., -20., -15., -12.5, -10., -7.5, -5., -3., -2., -1., 0., 1., 2.,
                3., 5., 7.5, 10., 12.5, 20., 25., 30., 40.,
            ],
            compartment_heel_steps: vec![
                -60., -15., -5., 0., 5., 15., 60.,
            ],
            compartment_trim_steps: vec![
                -40., -10., -5., 0., 5., 10., 40.,
            ],
            ship_length_lbp: 130.5,
            hull_draught_min: 0.5,
            hull_draught_max: 14.,
            hull_draught_step: 0.5,
            bounds_level_step: 0.1,
            compartment_level_step: 1.
        },
        tp.clone(),
    )
    .unwrap();
/*
    let res = model_cached.reload_shapes();            dbg!(&res);
 //   let res = model_cached.rebuild_caches();   dbg!(&res);
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
    let bounds = Bounds::from_array(&Calculus::PHYSICAL_FRAMES, 0.).unwrap();
    ship_model.init().unwrap();
    ship_model.init_cache_bounded(&bounds).unwrap();
    let ship_model = Arc::new(RwLock::new(ship_model));
    let server = Server::new(
        &dbg,
        conf.clone(),
        tp.scheduler(),
        move |dbg, conf| { Box::new(
            //
            // Select handler for incomong messages by Content
            SelectContent::new(vec![
                // Handler for Content::Bytes
                (Content::Bytes, Box::new(SelectCot::new(vec![
                    // Handling incomong messages with `Cot::Act` by field `cmd`
                    (Cot::Act, Box::new(SelectAct::new(vec![
                        // Handling incomong commands
                        // ...
                    ]))),
                    // Handling incomong messages with Cot::Req by field `req`
                    (Cot::Req, Box::new(SelectReq::new(vec![
                        // Handling incomong requests
                        // ...
                    ]))),
                ]))),
                // Handler for Content::Empty
                (Content::Empty, Box::new(SelectCot::new(vec![
                    // Handling incomong messages with `Cot::Act` by field `cmd`
                    (Cot::Act, Box::new(SelectAct::new(vec![
                        // Handling incomong commands
                        // ...
                    ]))),
                    // Handling incomong messages with Cot::Req by field `req`
                    (Cot::Req, Box::new(SelectReq::new(vec![
                        // Handling incomong requests
                        // ...
                    ]))),
                ]))),
                // Handler for Content::Json
                (Content::Json, Box::new(SelectCot::new(vec![
                    // Handling incomong messages with `Cot::Act` by field `cmd`
                    (Cot::Act, Box::new(SelectAct::new(vec![
                        // Handling incomong command `DeviceStream`
                        (QueryId::Calculus, Box::new(SelectCalculus::new(
                            dbg,
                            conf.calculus.clone(),
                            tp.scheduler(),
                            Calculus::new(dbg, conf.clone(), api_client.clone(), ship_model.clone()),
                        ))),
                        // Handling incomong command `DeviceStream`
                        (QueryId::DeviceStream, Box::new(DevStream::new(
                            dbg,
                            DevStreamConf { devices: vec![("Dev1".into(), DevConf {}), ("Dev2".into(), DevConf {})] },
                            tp.scheduler(),
                        ))),
                    ]))),
                    // Handling incomong messages with Cot::Req by field `req`
                    (Cot::Req, Box::new(SelectReq::new(vec![
                        // Handling incomong request `DeviceInfo`
                        (QueryId::DeviceInfo, Box::new(SelectDevInfo::new("assets/info/"))),
                        // Handling incomong request `DeviceDoc`
                        (QueryId::DeviceDoc, Box::new(SelectDevDoc::new("assets/info/"))),
                    ]))),
                ]))),
            ])
        )}
    );
    server.run().map_err(|err| Error::new(&dbg, "").pass(err))?;
    server.wait().map_err(|err| Error::new(dbg, "").pass(err))?;
    Ok(())
}
