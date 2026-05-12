#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
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
use crate::algorithm::entities::model_cached;
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
    DebugSession::new()
        .filter(LogLevel::Info)
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
    let ship_id = conf.api.params.ship_id.clone();
    let project_id = conf.api.params.project_id.clone();
    let model_x = conf.api.model.midel_x;
    let model_name = conf.api.model.name.clone();
    let cache_dir: PathBuf = ("assets/cache/".to_owned() + &model_name).into();
    let model_dir: PathBuf = ("assets/model/".to_owned() + &model_name).into();
    let mut dso_angles: Vec<_> = (-120..=120).map(|v| (v as f64) * 0.5 as f64).collect();
    dso_angles.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    dso_angles.dedup();
    let model_cached = model_cached::ModelCached::new(
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
