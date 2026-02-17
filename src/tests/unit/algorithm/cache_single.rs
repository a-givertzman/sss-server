use std::path::PathBuf;

use crate::algorithm::entities::model_cached::{self, DisplacementShape, LocalCache, Shape};
#[cfg(test)]
use crate::app::app::App;
use crate::conf::Conf;
use crate::kernel::{
    run::Run,
    types::{Arc, RwLock},
};
use debugging::session::debug_session::{DebugSession, LogLevel};
use sal_core::dbg::Dbg;
use sal_sync::thread_pool::ThreadPool;
///
/// Application entry point
#[ignore = "too slow, run only in release mode"]
#[test]
fn cache_single() -> Result<(), Box<dyn std::error::Error>> {
    DebugSession::new()
        //  .filter(LogLevel::Info)
        .filter(LogLevel::Debug)
        //   .filter(LogLevel::Trace)
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
    let cache_dir: PathBuf = "src/tests/unit/algorithm/cache/assets".into();
    let model_dir: PathBuf = "src/assets/model/sofia/compartments/201.stl".into();
    let shape = Arc::new(RwLock::new(DisplacementShape::new_uninit(
        &dbg, model_dir, None, 1000.,
    )));
    shape.write().init().unwrap();
    let mut cache = model_cached::CompartmentCache::new(
        &dbg,
        shape.clone(),
        cache_dir,
        "201".to_owned(),
        vec![
            -60., -30., -15., -10., -5., -2., 0., 2., 5., 10., 15., 30., 60.,
        ],
        vec![-40., -20., -10., -5., -2., 0., 2., 5., 10., 20., 40.],
        20,
        Arc::clone(&thread_pool),
    );
    cache.rebuild().unwrap();
    cache.calc_coeff(168.4).unwrap();
    dbg!(cache.get_level(-15.0, 3., 135., 0.000001));
    Ok(())
}
